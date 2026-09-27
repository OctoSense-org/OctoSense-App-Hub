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
