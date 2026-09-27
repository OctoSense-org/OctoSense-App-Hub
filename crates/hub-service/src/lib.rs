//! Publisher control-plane service and private artifact staging.
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path as RoutePath, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use rusqlite::{Connection, TransactionBehavior};
use serde::Serialize;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
pub mod artifacts;
pub mod auth;
pub mod publishers;
pub mod submissions;

#[derive(Debug)]
pub enum ServiceError {
    Io(std::io::Error),
    Database(rusqlite::Error),
    SchemaTooNew(i64),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "database file: {e}"),
            Self::Database(e) => write!(f, "database: {e}"),
            Self::SchemaTooNew(v) => write!(f, "database schema {v} is newer than this service"),
        }
    }
}
impl std::error::Error for ServiceError {}
impl From<std::io::Error> for ServiceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for ServiceError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}

pub struct Service {
    db: Mutex<Connection>,
    blobs: artifacts::BlobStore,
    provider: Option<Arc<dyn auth::IdentityProvider>>,
    clock: Arc<dyn auth::Clock>,
    limits: Limits,
}

#[derive(Clone, Copy)]
pub struct Limits {
    pub active_device_logins: i64,
    pub publishers_per_account: i64,
    pub apps_per_publisher: i64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            active_device_logins: 1_024,
            publishers_per_account: 16,
            apps_per_publisher: 1_000,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RegistryStats {
    pub accounts: i64,
    pub publishers: i64,
    pub apps: i64,
}

impl Service {
    pub fn open(path: &Path) -> Result<Self, ServiceError> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let db = Connection::open(path)?;
        db.busy_timeout(Duration::from_secs(5))?;
        db.pragma_update(None, "foreign_keys", "ON")?;
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "FULL")?;
        let service = Self {
            db: Mutex::new(db),
            blobs: artifacts::BlobStore::new(&path.with_extension("blobs"))?,
            provider: None,
            clock: Arc::new(auth::SystemClock),
            limits: Limits::default(),
        };
        service.migrate()?;
        Ok(service)
    }

    fn migrate(&self) -> Result<(), ServiceError> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: i64 = tx.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version > 5 {
            return Err(ServiceError::SchemaTooNew(version));
        }
        if version < 1 {
            tx.execute_batch(include_str!("../migrations/001_identity.sql"))?;
            tx.execute_batch("PRAGMA user_version = 1")?;
        }
        if version < 2 {
            tx.execute_batch(include_str!("../migrations/002_device_login.sql"))?;
            tx.execute_batch("PRAGMA user_version = 2")?;
        }
        if version < 3 {
            tx.execute_batch(include_str!("../migrations/003_claim_status.sql"))?;
            tx.execute_batch("PRAGMA user_version = 3")?;
        }
        if version < 4 {
            tx.execute_batch(include_str!("../migrations/004_uploads.sql"))?;
            tx.execute_batch("PRAGMA user_version = 4")?;
        }
        if version < 5 {
            tx.execute_batch(include_str!("../migrations/005_submissions.sql"))?;
            tx.execute_batch("PRAGMA user_version = 5")?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn migration_version(&self) -> Result<i64, ServiceError> {
        Ok(self
            .db
            .lock()
            .unwrap()
            .query_row("PRAGMA user_version", [], |row| row.get(0))?)
    }

    pub fn ready(&self) -> Result<bool, ServiceError> {
        let db = self.db.lock().unwrap();
        let one: i64 = db.query_row("SELECT 1", [], |row| row.get(0))?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        Ok(one == 1 && version == 5)
    }

    pub fn stats(&self) -> Result<RegistryStats, ServiceError> {
        let db = self.db.lock().unwrap();
        let count = |table| db.query_row(table, [], |row| row.get(0));
        Ok(RegistryStats {
            accounts: count("SELECT count(*) FROM accounts")?,
            publishers: count("SELECT count(*) FROM publishers")?,
            apps: count("SELECT count(*) FROM app_claims")?,
        })
    }

    pub fn with_limits(mut self, limits: Limits) -> Self {
        self.limits = limits;
        self
    }
}

pub fn router(service: Arc<Service>) -> Router {
    Router::new()
        .route(
            "/healthz",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route(
            "/readyz",
            get(
                |axum::extract::State(service): axum::extract::State<Arc<Service>>| async move {
                    if service.ready().unwrap_or(false) {
                        (StatusCode::OK, Json(serde_json::json!({"status":"ready"})))
                    } else {
                        (
                            StatusCode::SERVICE_UNAVAILABLE,
                            Json(serde_json::json!({"status":"unavailable"})),
                        )
                    }
                },
            ),
        )
        .route("/v1/login/device", post(begin_login))
        .route("/v1/login/token", post(finish_login))
        .route("/v1/me", get(me))
        .route("/v1/logout", post(logout))
        .route("/v1/publishers", post(create_publisher))
        .route(
            "/v1/publishers/{publisher_id}/keys/challenge",
            post(begin_key_enrollment),
        )
        .route(
            "/v1/publishers/{publisher_id}/keys",
            post(finish_key_enrollment),
        )
        .route("/v1/apps", post(claim_app))
        .route("/v1/apps/{app_id}/uploads", post(create_upload))
        .route("/v1/apps/{app_id}/submissions", post(create_submission))
        .route("/v1/uploads/{upload_id}", get(upload_status))
        .route("/v1/uploads/{upload_id}/content", put(put_upload))
        .layer(DefaultBodyLimit::max(4 * 1024))
        .with_state(service)
}

fn auth_failure(error: auth::AuthError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, code) = match error {
        auth::AuthError::Pending => (StatusCode::ACCEPTED, "authorization_pending"),
        auth::AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
        auth::AuthError::Limit => (StatusCode::TOO_MANY_REQUESTS, "limit_exceeded"),
        auth::AuthError::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"),
        auth::AuthError::Database(_) | auth::AuthError::Random => {
            (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable")
        }
    };
    (
        status,
        Json(serde_json::json!({"error":{"code":code,"message":error.to_string()}})),
    )
}

async fn begin_login(State(service): State<Arc<Service>>) -> (StatusCode, Json<serde_json::Value>) {
    // Provider calls include bounded network I/O; do not block the HTTP runtime.
    match tokio::task::spawn_blocking(move || service.begin_login())
        .await
        .unwrap_or(Err(auth::AuthError::Unavailable))
    {
        Ok(challenge) => (
            StatusCode::OK,
            Json(serde_json::to_value(challenge).unwrap()),
        ),
        Err(error) => auth_failure(error),
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenRequest {
    device_code: String,
}

async fn finish_login(
    State(service): State<Arc<Service>>,
    Json(request): Json<TokenRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match tokio::task::spawn_blocking(move || service.finish_login(&request.device_code))
        .await
        .unwrap_or(Err(auth::AuthError::Unavailable))
    {
        Ok(token) => (StatusCode::OK, Json(serde_json::to_value(token).unwrap())),
        Err(error) => auth_failure(error),
    }
}

fn bearer(headers: &HeaderMap) -> Result<&str, auth::AuthError> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(auth::AuthError::Unauthorized)
}

async fn me(
    State(service): State<Arc<Service>>,
    headers: HeaderMap,
) -> (StatusCode, Json<serde_json::Value>) {
    match bearer(&headers).and_then(|token| service.authenticate(token)) {
        Ok(account) => (StatusCode::OK, Json(serde_json::to_value(account).unwrap())),
        Err(error) => auth_failure(error),
    }
}

async fn logout(
    State(service): State<Arc<Service>>,
    headers: HeaderMap,
) -> (StatusCode, Json<serde_json::Value>) {
    match bearer(&headers).and_then(|token| service.logout(token)) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status":"revoked"})),
        ),
        Err(error) => auth_failure(error),
    }
}

fn registry_failure(error: publishers::RegistryError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, code) = match error {
        publishers::RegistryError::Invalid => (StatusCode::BAD_REQUEST, "invalid_request"),
        publishers::RegistryError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
        publishers::RegistryError::Conflict => (StatusCode::CONFLICT, "already_claimed"),
        publishers::RegistryError::Limit => (StatusCode::TOO_MANY_REQUESTS, "limit_exceeded"),
        publishers::RegistryError::Database(_) | publishers::RegistryError::Random => {
            (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable")
        }
    };
    (
        status,
        Json(serde_json::json!({"error":{"code":code,"message":error.to_string()}})),
    )
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PublisherRequest {
    slug: String,
    display_name: String,
}

async fn create_publisher(
    State(service): State<Arc<Service>>,
    headers: HeaderMap,
    Json(request): Json<PublisherRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match bearer(&headers)
        .map_err(publishers::RegistryError::from)
        .and_then(|token| service.create_publisher(token, &request.slug, &request.display_name))
    {
        Ok(publisher) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(publisher).unwrap()),
        ),
        Err(error) => registry_failure(error),
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ClaimRequest {
    publisher_id: String,
    app_id: String,
}

async fn claim_app(
    State(service): State<Arc<Service>>,
    headers: HeaderMap,
    Json(request): Json<ClaimRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match bearer(&headers)
        .map_err(publishers::RegistryError::from)
        .and_then(|token| service.claim_app(token, &request.publisher_id, &request.app_id))
    {
        Ok(claim) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(claim).unwrap()),
        ),
        Err(error) => registry_failure(error),
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyChallengeRequest {
    public_key: String,
}

async fn begin_key_enrollment(
    State(service): State<Arc<Service>>,
    RoutePath(publisher_id): RoutePath<String>,
    headers: HeaderMap,
    Json(request): Json<KeyChallengeRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match bearer(&headers)
        .map_err(publishers::RegistryError::from)
        .and_then(|token| service.begin_key_enrollment(token, &publisher_id, &request.public_key))
    {
        Ok(challenge) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(challenge).unwrap()),
        ),
        Err(error) => registry_failure(error),
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyEnrollmentRequest {
    challenge_id: String,
    signature: String,
}

async fn finish_key_enrollment(
    State(service): State<Arc<Service>>,
    RoutePath(publisher_id): RoutePath<String>,
    headers: HeaderMap,
    Json(request): Json<KeyEnrollmentRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match bearer(&headers)
        .map_err(publishers::RegistryError::from)
        .and_then(|token| {
            service.finish_key_enrollment(
                token,
                &publisher_id,
                &request.challenge_id,
                &request.signature,
            )
        }) {
        Ok(key) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(key).unwrap()),
        ),
        Err(error) => registry_failure(error),
    }
}

fn upload_failure(error: artifacts::UploadError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, code) = match error {
        artifacts::UploadError::Invalid => (StatusCode::BAD_REQUEST, "invalid_upload"),
        artifacts::UploadError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
        artifacts::UploadError::Conflict => (StatusCode::CONFLICT, "upload_conflict"),
        artifacts::UploadError::Database(_) | artifacts::UploadError::Io(_) => {
            (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable")
        }
    };
    (
        status,
        Json(serde_json::json!({"error":{"code":code,"message":error.to_string()}})),
    )
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UploadRequest {
    expected_digest: String,
    expected_bytes: u64,
}

async fn create_upload(
    State(service): State<Arc<Service>>,
    RoutePath(app_id): RoutePath<String>,
    headers: HeaderMap,
    Json(request): Json<UploadRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match bearer(&headers)
        .map_err(artifacts::UploadError::from)
        .and_then(|token| {
            service.create_upload(
                token,
                &app_id,
                &request.expected_digest,
                request.expected_bytes,
            )
        }) {
        Ok(upload) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(upload).unwrap()),
        ),
        Err(error) => upload_failure(error),
    }
}

async fn upload_status(
    State(service): State<Arc<Service>>,
    RoutePath(upload_id): RoutePath<String>,
    headers: HeaderMap,
) -> (StatusCode, Json<serde_json::Value>) {
    match bearer(&headers)
        .map_err(artifacts::UploadError::from)
        .and_then(|token| service.upload_status(token, &upload_id))
    {
        Ok(upload) => (StatusCode::OK, Json(serde_json::to_value(upload).unwrap())),
        Err(error) => upload_failure(error),
    }
}

async fn put_upload(
    State(service): State<Arc<Service>>,
    RoutePath(upload_id): RoutePath<String>,
    headers: HeaderMap,
    body: Body,
) -> (StatusCode, Json<serde_json::Value>) {
    let token = match bearer(&headers) {
        Ok(token) => token.to_owned(),
        Err(error) => return upload_failure(error.into()),
    };
    if let Err(error) = service.authorize(&token, "apps.submit") {
        return upload_failure(error.into());
    }
    let upload = match service.upload_status(&token, &upload_id) {
        Ok(upload) => upload,
        Err(error) => return upload_failure(error),
    };
    let staged = match service.blobs.stage_body(body, upload.expected_bytes).await {
        Ok(staged) => staged,
        Err(error) => return upload_failure(error),
    };
    match tokio::task::spawn_blocking(move || service.put_upload_staged(&token, &upload_id, staged))
        .await
    {
        Ok(Ok(upload)) => (StatusCode::OK, Json(serde_json::to_value(upload).unwrap())),
        Ok(Err(error)) => upload_failure(error),
        Err(_) => upload_failure(artifacts::UploadError::Io(std::io::Error::other(
            "upload worker unavailable",
        ))),
    }
}

fn submission_failure(
    error: submissions::SubmissionError,
) -> (StatusCode, Json<serde_json::Value>) {
    let (status, code) = match error {
        submissions::SubmissionError::Invalid => (StatusCode::BAD_REQUEST, "invalid_submission"),
        submissions::SubmissionError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
        submissions::SubmissionError::Conflict => (StatusCode::CONFLICT, "submission_conflict"),
        submissions::SubmissionError::Database(_) | submissions::SubmissionError::Storage => {
            (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable")
        }
    };
    (
        status,
        Json(serde_json::json!({"error":{"code":code,"message":error.to_string()}})),
    )
}

async fn create_submission(
    State(service): State<Arc<Service>>,
    RoutePath(app_id): RoutePath<String>,
    headers: HeaderMap,
    Json(input): Json<submissions::SubmissionInput>,
) -> (StatusCode, Json<serde_json::Value>) {
    let token = match bearer(&headers) {
        Ok(token) => token.to_owned(),
        Err(error) => return submission_failure(error.into()),
    };
    let idempotency_key = match headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
    {
        Some(key) => key.to_owned(),
        None => return submission_failure(submissions::SubmissionError::Invalid),
    };
    match tokio::task::spawn_blocking(move || {
        service.create_submission(&token, &app_id, &idempotency_key, &input)
    })
    .await
    {
        Ok(Ok(submission)) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(submission).unwrap()),
        ),
        Ok(Err(error)) => submission_failure(error),
        Err(_) => submission_failure(submissions::SubmissionError::Storage),
    }
}
