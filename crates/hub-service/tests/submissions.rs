use octosense_hub_service::submissions::SubmissionInput;
use octosense_hub_service::{
    auth::{Clock, DeviceChallenge, IdentityAssertion, IdentityProvider},
    Service,
};
use std::sync::Arc;

struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> i64 {
        1_000_000
    }
}
struct Developer;
impl IdentityProvider for Developer {
    fn begin(&self) -> Result<DeviceChallenge, String> {
        Ok(DeviceChallenge {
            device_code: "upload-login".into(),
            user_code: "CODE".into(),
            verification_uri: "https://login.example.test/device".into(),
            expires_in: 300,
        })
    }
    fn poll(&self, _: &str) -> Result<Option<IdentityAssertion>, String> {
        Ok(Some(IdentityAssertion {
            provider: "mock".into(),
            subject: "uploader".into(),
            audience: "octosense-app-hub".into(),
        }))
    }
}

#[test]
fn oversize_or_digest_mismatch_never_enters_queue() {
    let dir = tempfile::tempdir().unwrap();
    let service = Service::open(&dir.path().join("db.sqlite3"))
        .unwrap()
        .with_auth(Arc::new(Developer), Arc::new(FixedClock));
    let code = service.begin_login().unwrap();
    let token = service
        .finish_login(&code.device_code)
        .unwrap()
        .access_token;
    let publisher = service
        .create_publisher(&token, "uploader", "Uploader")
        .unwrap();
    service
        .claim_app(&token, &publisher.id, "uploader.notes")
        .unwrap();

    let valid = br#"{"schema":1,"files":{"manifest.json":"e30="}}"#;
    let digest = blake3::hash(valid).to_hex().to_string();
    let upload = service
        .create_upload(&token, "uploader.notes", &digest, valid.len() as u64)
        .unwrap();
    assert_eq!(upload.status, "pending");
    assert_eq!(
        service
            .put_upload(&token, &upload.id, valid)
            .unwrap()
            .status,
        "complete"
    );
    assert_eq!(
        service
            .create_upload(&token, "uploader.notes", &digest, valid.len() as u64)
            .unwrap()
            .id,
        upload.id
    );

    let mismatch = service
        .create_upload(
            &token,
            "uploader.notes",
            &"0".repeat(64),
            valid.len() as u64,
        )
        .unwrap();
    assert!(service.put_upload(&token, &mismatch.id, valid).is_err());
    assert_eq!(
        service.upload_status(&token, &mismatch.id).unwrap().status,
        "pending"
    );
    assert!(service
        .create_upload(
            &token,
            "uploader.notes",
            &digest,
            octosense_hub_service::artifacts::MAX_PACK_BYTES + 1
        )
        .is_err());

    let traversal = br#"{"schema":1,"files":{"../escape.txt":"eA==","manifest.json":"e30="}}"#;
    let traversal_digest = blake3::hash(traversal).to_hex().to_string();
    let traversal_upload = service
        .create_upload(
            &token,
            "uploader.notes",
            &traversal_digest,
            traversal.len() as u64,
        )
        .unwrap();
    assert!(service
        .put_upload(&token, &traversal_upload.id, traversal)
        .is_err());
    assert_eq!(
        service
            .upload_status(&token, &traversal_upload.id)
            .unwrap()
            .status,
        "pending"
    );
    assert!(!dir.path().join("escape.txt").exists());

    let interrupted = service
        .create_upload(
            &token,
            "uploader.notes",
            &"1".repeat(64),
            valid.len() as u64,
        )
        .unwrap();
    assert!(service
        .put_upload(&token, &interrupted.id, &valid[..valid.len() - 1])
        .is_err());
    assert_eq!(
        service
            .upload_status(&token, &interrupted.id)
            .unwrap()
            .status,
        "pending"
    );
    assert_eq!(service.stats().unwrap().apps, 1);
}

#[tokio::test]
async fn authenticated_upload_http_journey_keeps_artifact_private() {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let dir = tempfile::tempdir().unwrap();
    let service = Service::open(&dir.path().join("http.sqlite3"))
        .unwrap()
        .with_auth(Arc::new(Developer), Arc::new(FixedClock));
    let code = service.begin_login().unwrap();
    let token = service
        .finish_login(&code.device_code)
        .unwrap()
        .access_token;
    let publisher = service
        .create_publisher(&token, "uploader", "Uploader")
        .unwrap();
    service
        .claim_app(&token, &publisher.id, "uploader.notes")
        .unwrap();
    let app = octosense_hub_service::router(Arc::new(service));
    let bytes = br#"{"schema":1,"files":{"manifest.json":"e30="}}"#;
    let digest = blake3::hash(bytes).to_hex().to_string();
    let bearer = format!("Bearer {token}");
    let response = app
        .clone()
        .oneshot(
            Request::post("/v1/apps/uploader.notes/uploads")
                .header("authorization", &bearer)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"expected_digest":digest,"expected_bytes":bytes.len()})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let upload: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    let id = upload["id"].as_str().unwrap();
    let content_path = format!("/v1/uploads/{id}/content");
    let status_path = format!("/v1/uploads/{id}");
    let unauthenticated = app
        .clone()
        .oneshot(
            Request::put(&content_path)
                .body(Body::from(bytes.to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
    let incomplete = app
        .clone()
        .oneshot(
            Request::put(&content_path)
                .header("authorization", &bearer)
                .body(Body::from(bytes[..bytes.len() - 1].to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(incomplete.status(), StatusCode::BAD_REQUEST);
    let mut too_long = bytes.to_vec();
    too_long.push(b' ');
    let oversized = app
        .clone()
        .oneshot(
            Request::put(&content_path)
                .header("authorization", &bearer)
                .body(Body::from(too_long))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(oversized.status(), StatusCode::BAD_REQUEST);
    let response = app
        .clone()
        .oneshot(
            Request::put(&content_path)
                .header("authorization", &bearer)
                .body(Body::from(bytes.to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = app
        .clone()
        .oneshot(
            Request::get(&status_path)
                .header("authorization", &bearer)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let status: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(status["status"], "complete");
    assert_eq!(
        std::fs::read_dir(dir.path().join("http.blobs/objects"))
            .unwrap()
            .count(),
        1
    );
}

fn signed_pack(
    app_id: &str,
    publisher_id: &str,
    key: &octosense_app_hub::signing::HubKey,
) -> Vec<u8> {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("page.card"), b"card source").unwrap();
    let mut manifest = octosense_app_policy::AppManifest::parse(include_str!(
        "../../app-hub/tests/fixtures/wire/v2-manifest.canonical.json"
    ))
    .unwrap();
    manifest.id = app_id.into();
    manifest.integrity.bundle_blake3 = octosense_app_policy::digest_dir(dir.path()).unwrap();
    octosense_app_hub::signing::sign_manifest(key, &mut manifest, publisher_id).unwrap();
    std::fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    serde_json::to_vec(&octosense_app_hub::pack::pack_dir(dir.path()).unwrap()).unwrap()
}

#[test]
fn retry_creates_one_submission_and_validation_job() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("submissions.sqlite3");
    let service = Arc::new(
        Service::open(&path)
            .unwrap()
            .with_auth(Arc::new(Developer), Arc::new(FixedClock)),
    );
    let code = service.begin_login().unwrap();
    let token = service
        .finish_login(&code.device_code)
        .unwrap()
        .access_token;
    let publisher = service
        .create_publisher(&token, "uploader", "Uploader")
        .unwrap();
    service
        .claim_app(&token, &publisher.id, "uploader.notes")
        .unwrap();
    let key = octosense_app_hub::signing::HubKey::from_bytes(&[11u8; 32]);
    let challenge = service
        .begin_key_enrollment(&token, &publisher.id, &key.public_hex())
        .unwrap();
    service
        .finish_key_enrollment(
            &token,
            &publisher.id,
            &challenge.id,
            &key.sign_hex(challenge.message.as_bytes()),
        )
        .unwrap();
    let pack = signed_pack("uploader.notes", &publisher.id, &key);
    let digest = blake3::hash(&pack).to_hex().to_string();
    let upload = service
        .create_upload(&token, "uploader.notes", &digest, pack.len() as u64)
        .unwrap();
    service.put_upload(&token, &upload.id, &pack).unwrap();
    let input = SubmissionInput {
        upload_id: upload.id,
        source_repository: Some("https://github.com/example/notes".into()),
        source_commit: Some("abcdef0123456789abcdef0123456789abcdef01".into()),
    };
    let contenders = ["request-one", "request-two"];
    let results = std::thread::scope(|scope| {
        contenders
            .into_iter()
            .map(|key| {
                let service = service.clone();
                let input = input.clone();
                let token = token.clone();
                scope
                    .spawn(move || service.create_submission(&token, "uploader.notes", key, &input))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let successful_key = contenders[results.iter().position(Result::is_ok).unwrap()];
    let first = results.into_iter().find_map(Result::ok).unwrap();
    assert_eq!(first.status, "submitted");
    assert_eq!(first.version, "1.2.0");
    assert_eq!(first.release_number, 12);
    assert_eq!(
        service
            .create_submission(&token, "uploader.notes", successful_key, &input)
            .unwrap()
            .id,
        first.id
    );
    let changed = SubmissionInput {
        source_commit: Some("abcdef0123456789abcdef0123456789abcdef02".into()),
        ..input.clone()
    };
    assert!(service
        .create_submission(&token, "uploader.notes", successful_key, &changed)
        .is_err());
    let db = rusqlite::Connection::open(path).unwrap();
    let submissions: i64 = db
        .query_row("SELECT count(*) FROM submissions", [], |row| row.get(0))
        .unwrap();
    let jobs: i64 = db
        .query_row("SELECT count(*) FROM validation_jobs", [], |row| row.get(0))
        .unwrap();
    assert_eq!((submissions, jobs), (1, 1));
}

#[test]
fn unregistered_signing_key_cannot_queue_a_submission() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("forged.sqlite3");
    let service = Service::open(&path)
        .unwrap()
        .with_auth(Arc::new(Developer), Arc::new(FixedClock));
    let code = service.begin_login().unwrap();
    let token = service
        .finish_login(&code.device_code)
        .unwrap()
        .access_token;
    let publisher = service
        .create_publisher(&token, "uploader", "Uploader")
        .unwrap();
    service
        .claim_app(&token, &publisher.id, "uploader.notes")
        .unwrap();
    let registered = octosense_app_hub::signing::HubKey::from_bytes(&[11u8; 32]);
    let challenge = service
        .begin_key_enrollment(&token, &publisher.id, &registered.public_hex())
        .unwrap();
    service
        .finish_key_enrollment(
            &token,
            &publisher.id,
            &challenge.id,
            &registered.sign_hex(challenge.message.as_bytes()),
        )
        .unwrap();
    let replacement = octosense_app_hub::signing::HubKey::from_bytes(&[12u8; 32]);
    let pack = signed_pack("uploader.notes", &publisher.id, &replacement);
    let digest = blake3::hash(&pack).to_hex().to_string();
    let upload = service
        .create_upload(&token, "uploader.notes", &digest, pack.len() as u64)
        .unwrap();
    service.put_upload(&token, &upload.id, &pack).unwrap();
    let input = SubmissionInput {
        upload_id: upload.id,
        source_repository: None,
        source_commit: None,
    };
    assert!(service
        .create_submission(&token, "uploader.notes", "forged-key", &input)
        .is_err());
    let db = rusqlite::Connection::open(path).unwrap();
    let jobs: i64 = db
        .query_row("SELECT count(*) FROM validation_jobs", [], |row| row.get(0))
        .unwrap();
    assert_eq!(jobs, 0);
}

#[tokio::test]
async fn submission_http_requires_idempotency_key_and_returns_same_record_on_retry() {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let dir = tempfile::tempdir().unwrap();
    let service = Service::open(&dir.path().join("http-submit.sqlite3"))
        .unwrap()
        .with_auth(Arc::new(Developer), Arc::new(FixedClock));
    let code = service.begin_login().unwrap();
    let token = service
        .finish_login(&code.device_code)
        .unwrap()
        .access_token;
    let publisher = service
        .create_publisher(&token, "uploader", "Uploader")
        .unwrap();
    service
        .claim_app(&token, &publisher.id, "uploader.notes")
        .unwrap();
    let key = octosense_app_hub::signing::HubKey::from_bytes(&[11u8; 32]);
    let challenge = service
        .begin_key_enrollment(&token, &publisher.id, &key.public_hex())
        .unwrap();
    service
        .finish_key_enrollment(
            &token,
            &publisher.id,
            &challenge.id,
            &key.sign_hex(challenge.message.as_bytes()),
        )
        .unwrap();
    let pack = signed_pack("uploader.notes", &publisher.id, &key);
    let digest = blake3::hash(&pack).to_hex().to_string();
    let upload = service
        .create_upload(&token, "uploader.notes", &digest, pack.len() as u64)
        .unwrap();
    service.put_upload(&token, &upload.id, &pack).unwrap();
    let app = octosense_hub_service::router(Arc::new(service));
    let request = |key: Option<&str>| {
        let mut builder = Request::post("/v1/apps/uploader.notes/submissions")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json");
        if let Some(key) = key {
            builder = builder.header("idempotency-key", key);
        }
        builder
            .body(Body::from(
                serde_json::json!({"upload_id":upload.id}).to_string(),
            ))
            .unwrap()
    };
    assert_eq!(
        app.clone().oneshot(request(None)).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
    let first = app
        .clone()
        .oneshot(request(Some("http-request")))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);
    let first: serde_json::Value =
        serde_json::from_slice(&to_bytes(first.into_body(), 4096).await.unwrap()).unwrap();
    let second = app.oneshot(request(Some("http-request"))).await.unwrap();
    assert_eq!(second.status(), StatusCode::CREATED);
    let second: serde_json::Value =
        serde_json::from_slice(&to_bytes(second.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(first["id"], second["id"]);
}
