//! Publisher control-plane service. No production identity provider is wired yet.
use axum::{
    extract::{DefaultBodyLimit, Path as RoutePath, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use rusqlite::{Connection, TransactionBehavior};
use serde::Serialize;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
pub mod auth;
pub mod publishers;

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
        if version > 3 {
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
        Ok(one == 1 && version == 3)
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
