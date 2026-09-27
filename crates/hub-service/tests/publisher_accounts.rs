use octosense_hub_service::auth::{Clock, DeviceChallenge, IdentityAssertion, IdentityProvider};
use octosense_hub_service::{Limits, Service};
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::Arc;

struct FakeClock(AtomicI64);
impl Clock for FakeClock {
    fn now(&self) -> i64 {
        self.0.load(Ordering::Relaxed)
    }
}

struct FakeProvider {
    next: AtomicUsize,
    audience: std::sync::Mutex<String>,
    subject: std::sync::Mutex<String>,
}
impl IdentityProvider for FakeProvider {
    fn begin(&self) -> Result<DeviceChallenge, String> {
        let n = self.next.fetch_add(1, Ordering::Relaxed);
        Ok(DeviceChallenge {
            device_code: format!("device-{n}"),
            user_code: format!("USER-{n}"),
            verification_uri: "https://login.example.test/device".into(),
            expires_in: 300,
        })
    }
    fn poll(&self, _device_code: &str) -> Result<Option<IdentityAssertion>, String> {
        Ok(Some(IdentityAssertion {
            provider: "mock".into(),
            subject: self.subject.lock().unwrap().clone(),
            audience: self.audience.lock().unwrap().clone(),
        }))
    }
}

#[test]
fn identity_migration_is_repeatable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("publishers.sqlite3");
    let first = Service::open(&path).unwrap();
    assert!(first.ready().unwrap());
    assert_eq!(first.migration_version().unwrap(), 5);
    assert_eq!(first.stats().unwrap().accounts, 0);
    assert_eq!(first.stats().unwrap().publishers, 0);
    drop(first);
    let again = Service::open(&path).unwrap();
    assert!(again.ready().unwrap());
    assert_eq!(again.migration_version().unwrap(), 5);
    assert_eq!(again.stats().unwrap().accounts, 0);
}

#[tokio::test]
async fn health_and_readiness_are_accessible_without_an_account() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let dir = tempfile::tempdir().unwrap();
    let service = Arc::new(Service::open(&dir.path().join("db.sqlite3")).unwrap());
    let app = octosense_hub_service::router(service);
    for path in ["/healthz", "/readyz"] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[test]
fn newer_database_is_not_downgraded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("future.sqlite3");
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("PRAGMA user_version = 9")
        .unwrap();
    let error = Service::open(&path)
        .err()
        .expect("future schema must be refused");
    assert!(error.to_string().contains("newer"));
}

#[test]
fn old_claims_require_review_after_schema_upgrade() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("upgrade.sqlite3");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch(include_str!("../migrations/001_identity.sql"))
        .unwrap();
    db.execute_batch(include_str!("../migrations/002_device_login.sql"))
        .unwrap();
    db.execute(
        "INSERT INTO accounts(id,provider,provider_subject,created_at) VALUES('a','mock','a',1)",
        [],
    )
    .unwrap();
    db.execute("INSERT INTO publishers(id,slug,display_name,owner_account_id,created_at) VALUES('p','alpha','Alpha','a',1)", []).unwrap();
    db.execute("INSERT INTO app_claims(app_id,publisher_id,claimed_by,claimed_at) VALUES('legacy.app','p','a',1)", []).unwrap();
    db.execute_batch("PRAGMA user_version = 2").unwrap();
    drop(db);
    let service = Service::open(&path).unwrap();
    assert_eq!(service.migration_version().unwrap(), 5);
    assert_eq!(service.stats().unwrap().apps, 1);
    let db = rusqlite::Connection::open(&path).unwrap();
    let status: String = db
        .query_row(
            "SELECT status FROM app_claims WHERE app_id='legacy.app'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "review_required");
}

#[test]
fn expired_or_wrong_audience_token_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let provider = Arc::new(FakeProvider {
        next: AtomicUsize::new(1),
        audience: std::sync::Mutex::new("wrong".into()),
        subject: std::sync::Mutex::new("stable-subject".into()),
    });
    let clock = Arc::new(FakeClock(AtomicI64::new(1_000_000)));
    let service = Service::open(&dir.path().join("auth.sqlite3"))
        .unwrap()
        .with_auth(provider.clone(), clock.clone());
    let wrong = service.begin_login().unwrap();
    assert!(service.finish_login(&wrong.device_code).is_err());
    *provider.audience.lock().unwrap() = "octosense-app-hub".into();
    let challenge = service.begin_login().unwrap();
    let token = service.finish_login(&challenge.device_code).unwrap();
    let account = service.authenticate(&token.access_token).unwrap();
    assert_eq!(account.provider_subject, "stable-subject");
    assert!(
        service.finish_login(&challenge.device_code).is_err(),
        "device code cannot be replayed"
    );
    clock.0.store(1_003_601, Ordering::Relaxed);
    assert!(service.authenticate(&token.access_token).is_err());
}

#[tokio::test]
async fn device_login_http_journey_revokes_its_token() {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let dir = tempfile::tempdir().unwrap();
    let provider = Arc::new(FakeProvider {
        next: AtomicUsize::new(1),
        audience: std::sync::Mutex::new("octosense-app-hub".into()),
        subject: std::sync::Mutex::new("stable-subject".into()),
    });
    let clock = Arc::new(FakeClock(AtomicI64::new(1_000_000)));
    let service = Service::open(&dir.path().join("http.sqlite3"))
        .unwrap()
        .with_auth(provider, clock);
    let app = octosense_hub_service::router(Arc::new(service));
    let start = app
        .clone()
        .oneshot(
            Request::post("/v1/login/device")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(start.status(), StatusCode::OK);
    let start: serde_json::Value =
        serde_json::from_slice(&to_bytes(start.into_body(), 65_536).await.unwrap()).unwrap();
    let request = serde_json::json!({"device_code":start["device_code"]}).to_string();
    let token = app
        .clone()
        .oneshot(
            Request::post("/v1/login/token")
                .header("content-type", "application/json")
                .body(Body::from(request))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(token.status(), StatusCode::OK);
    let token: serde_json::Value =
        serde_json::from_slice(&to_bytes(token.into_body(), 65_536).await.unwrap()).unwrap();
    let bearer = format!("Bearer {}", token["access_token"].as_str().unwrap());
    let request = || {
        Request::get("/v1/me")
            .header("authorization", &bearer)
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(
        app.clone().oneshot(request()).await.unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(
        app.clone()
            .oneshot(
                Request::post("/v1/logout")
                    .header("authorization", &bearer)
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.oneshot(request()).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn publisher_http_journey_claims_namespace_and_enrolls_key() {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use ed25519_dalek::{Signer, SigningKey};
    use tower::ServiceExt;
    let dir = tempfile::tempdir().unwrap();
    let provider = Arc::new(FakeProvider {
        next: AtomicUsize::new(1),
        audience: std::sync::Mutex::new("octosense-app-hub".into()),
        subject: std::sync::Mutex::new("http-publisher".into()),
    });
    let service = Service::open(&dir.path().join("http-publisher.sqlite3"))
        .unwrap()
        .with_auth(provider, Arc::new(FakeClock(AtomicI64::new(1_000_000))));
    let login = service.begin_login().unwrap();
    let token = service
        .finish_login(&login.device_code)
        .unwrap()
        .access_token;
    let app = octosense_hub_service::router(Arc::new(service));
    let send = |path: String, body: serde_json::Value| {
        Request::post(path)
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    };
    let response = app
        .clone()
        .oneshot(send(
            "/v1/publishers".into(),
            serde_json::json!({"slug":"Example","display_name":"Example Apps"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let publisher: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    let publisher_id = publisher["id"].as_str().unwrap();
    let response = app
        .clone()
        .oneshot(send(
            "/v1/apps".into(),
            serde_json::json!({"publisher_id":publisher_id,"app_id":"example.notes"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let claim: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(claim["status"], "active");
    let key = SigningKey::from_bytes(&[9u8; 32]);
    let response = app
        .clone()
        .oneshot(send(
            format!("/v1/publishers/{publisher_id}/keys/challenge"),
            serde_json::json!({"public_key":hex::encode(key.verifying_key().to_bytes())}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let challenge: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    let response = app.oneshot(send(format!("/v1/publishers/{publisher_id}/keys"),
        serde_json::json!({"challenge_id":challenge["id"],
            "signature":hex::encode(key.sign(challenge["message"].as_str().unwrap().as_bytes()).to_bytes())}))).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[test]
fn concurrent_app_claim_has_one_owner() {
    let dir = tempfile::tempdir().unwrap();
    let provider = Arc::new(FakeProvider {
        next: AtomicUsize::new(1),
        audience: std::sync::Mutex::new("octosense-app-hub".into()),
        subject: std::sync::Mutex::new("developer-one".into()),
    });
    let clock = Arc::new(FakeClock(AtomicI64::new(1_000_000)));
    let service = Arc::new(
        Service::open(&dir.path().join("claims.sqlite3"))
            .unwrap()
            .with_auth(provider.clone(), clock),
    );
    let first_code = service.begin_login().unwrap();
    let first = service
        .finish_login(&first_code.device_code)
        .unwrap()
        .access_token;
    *provider.subject.lock().unwrap() = "developer-two".into();
    let second_code = service.begin_login().unwrap();
    let second = service
        .finish_login(&second_code.device_code)
        .unwrap()
        .access_token;
    let alpha = service
        .create_publisher(&first, "Alpha", "Alpha Apps")
        .unwrap();
    let beta = service
        .create_publisher(&second, "Beta", "Beta Apps")
        .unwrap();
    assert_eq!(alpha.slug, "alpha");
    assert!(service
        .claim_app(&second, &alpha.id, "alpha.notes")
        .is_err());
    let live = service.claim_app(&first, &alpha.id, "ALPHA.Notes").unwrap();
    assert_eq!(live.app_id, "alpha.notes");
    assert_eq!(live.status, "active");
    let contenders = [(first, alpha.id), (second, beta.id)];
    let results = std::thread::scope(|scope| {
        contenders
            .iter()
            .map(|(token, publisher)| {
                let service = service.clone();
                scope.spawn(move || service.claim_app(token, publisher, "legacy.app"))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|r| r.is_err()).count(), 1);
    assert_eq!(
        results.into_iter().find_map(Result::ok).unwrap().status,
        "review_required"
    );
    assert_eq!(service.stats().unwrap().apps, 2);
}

#[test]
fn key_enrollment_requires_possession_and_owner() {
    use ed25519_dalek::{Signer, SigningKey};
    let dir = tempfile::tempdir().unwrap();
    let provider = Arc::new(FakeProvider {
        next: AtomicUsize::new(1),
        audience: std::sync::Mutex::new("octosense-app-hub".into()),
        subject: std::sync::Mutex::new("key-owner".into()),
    });
    let clock = Arc::new(FakeClock(AtomicI64::new(1_000_000)));
    let service = Service::open(&dir.path().join("keys.sqlite3"))
        .unwrap()
        .with_auth(provider.clone(), clock.clone());
    let owner_code = service.begin_login().unwrap();
    let owner = service
        .finish_login(&owner_code.device_code)
        .unwrap()
        .access_token;
    *provider.subject.lock().unwrap() = "other-person".into();
    let other_code = service.begin_login().unwrap();
    let other = service
        .finish_login(&other_code.device_code)
        .unwrap()
        .access_token;
    let publisher = service
        .create_publisher(&owner, "keyowner", "Key Owner")
        .unwrap();
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let public = hex::encode(key.verifying_key().to_bytes());
    assert!(service
        .begin_key_enrollment(&other, &publisher.id, &public)
        .is_err());
    let challenge = service
        .begin_key_enrollment(&owner, &publisher.id, &public)
        .unwrap();
    assert!(service
        .finish_key_enrollment(
            &owner,
            &publisher.id,
            &challenge.id,
            &hex::encode([0u8; 64])
        )
        .is_err());
    assert!(service
        .finish_key_enrollment(
            &other,
            &publisher.id,
            &challenge.id,
            &hex::encode(key.sign(challenge.message.as_bytes()).to_bytes())
        )
        .is_err());
    let signature = hex::encode(key.sign(challenge.message.as_bytes()).to_bytes());
    let enrolled = service
        .finish_key_enrollment(&owner, &publisher.id, &challenge.id, &signature)
        .unwrap();
    assert_eq!(enrolled.id, publisher.id);
    assert_eq!(enrolled.public_key, public);
    assert!(service
        .finish_key_enrollment(&owner, &publisher.id, &challenge.id, &signature)
        .is_err());
    let replacement = SigningKey::from_bytes(&[8u8; 32]);
    assert!(service
        .begin_key_enrollment(
            &owner,
            &publisher.id,
            &hex::encode(replacement.verifying_key().to_bytes())
        )
        .is_err());
    let other_pub = service
        .create_publisher(&other, "otherpub", "Other Publisher")
        .unwrap();
    assert!(
        service
            .begin_key_enrollment(&other, &other_pub.id, &public)
            .is_err(),
        "one public key cannot impersonate another publisher"
    );
    clock.0.store(1_000_301, Ordering::Relaxed);
}

#[test]
fn configured_limits_bound_logins_and_publisher_resources() {
    let dir = tempfile::tempdir().unwrap();
    let provider = Arc::new(FakeProvider {
        next: AtomicUsize::new(1),
        audience: std::sync::Mutex::new("octosense-app-hub".into()),
        subject: std::sync::Mutex::new("limited-developer".into()),
    });
    let service = Service::open(&dir.path().join("limits.sqlite3"))
        .unwrap()
        .with_auth(provider, Arc::new(FakeClock(AtomicI64::new(1_000_000))))
        .with_limits(Limits {
            active_device_logins: 1,
            publishers_per_account: 1,
            apps_per_publisher: 1,
        });
    let first = service.begin_login().unwrap();
    assert!(
        service.begin_login().is_err(),
        "only one active device login"
    );
    let token = service
        .finish_login(&first.device_code)
        .unwrap()
        .access_token;
    assert!(service
        .create_publisher(&token, "system", "System")
        .is_err());
    let publisher = service
        .create_publisher(&token, "limited", "Limited")
        .unwrap();
    assert!(service
        .create_publisher(&token, "another", "Another")
        .is_err());
    let claim = service
        .claim_app(&token, &publisher.id, "limited.one")
        .unwrap();
    assert_eq!(claim.status, "active");
    assert!(service
        .claim_app(&token, &publisher.id, "limited.two")
        .is_err());
    assert!(
        service
            .claim_app(&token, &publisher.id, "limited.one")
            .is_ok(),
        "retries are idempotent"
    );
}
