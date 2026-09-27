//! Provider-verified device login and host-issued, scoped API sessions.
//! Only hashes of device codes and API tokens enter the database.
use crate::Service;
use rand_core::TryRngCore;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::Serialize;
use std::sync::Arc;
pub mod github;

pub const AUDIENCE: &str = "octosense-app-hub";
const TOKEN_SECONDS: i64 = 3_600;
const SCOPES: &str = "publisher.read publisher.write apps.submit";

pub trait Clock: Send + Sync {
    fn now(&self) -> i64;
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DeviceChallenge {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
}

pub struct IdentityAssertion {
    pub provider: String,
    pub subject: String,
    pub audience: String,
}

/// The adapter must authenticate the person through the provider's protocol
/// and return a stable subject bound to this app's client identity. For opaque
/// OAuth tokens, this means calling the provider's authenticated user API.
pub trait IdentityProvider: Send + Sync {
    fn begin(&self) -> Result<DeviceChallenge, String>;
    fn poll(&self, device_code: &str) -> Result<Option<IdentityAssertion>, String>;
}

#[derive(Debug)]
pub enum AuthError {
    Unavailable,
    Pending,
    Unauthorized,
    Limit,
    Database(rusqlite::Error),
    Random,
}
impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => write!(f, "identity provider unavailable"),
            Self::Pending => write!(f, "authorization pending"),
            Self::Unauthorized => write!(f, "invalid or expired authorization"),
            Self::Limit => write!(f, "login capacity reached"),
            Self::Database(_) => write!(f, "authorization store unavailable"),
            Self::Random => write!(f, "secure randomness unavailable"),
        }
    }
}
impl std::error::Error for AuthError {}
impl From<rusqlite::Error> for AuthError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Account {
    pub id: String,
    pub provider: String,
    pub provider_subject: String,
    pub scopes: Vec<String>,
}

fn secret<const N: usize>() -> Result<[u8; N], AuthError> {
    let mut bytes = [0u8; N];
    rand_core::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| AuthError::Random)?;
    Ok(bytes)
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn hash(value: &str) -> [u8; 32] {
    *blake3::hash(value.as_bytes()).as_bytes()
}

impl Service {
    /// Install a verified provider and a clock. The normal service starts
    /// without login, rather than silently accepting a development identity.
    pub fn with_auth(mut self, provider: Arc<dyn IdentityProvider>, clock: Arc<dyn Clock>) -> Self {
        self.provider = Some(provider);
        self.clock = clock;
        self
    }

    pub fn begin_login(&self) -> Result<DeviceChallenge, AuthError> {
        let provider = self.provider.as_ref().ok_or(AuthError::Unavailable)?;
        let now = self.clock.now();
        let active: i64 = self.db.lock().unwrap().query_row(
            "SELECT count(*) FROM device_logins WHERE expires_at>?1 AND consumed_at IS NULL",
            params![now],
            |row| row.get(0),
        )?;
        if active >= self.limits.active_device_logins {
            return Err(AuthError::Limit);
        }
        let challenge = provider.begin().map_err(|_| AuthError::Unavailable)?;
        if challenge.device_code.is_empty()
            || challenge.device_code.len() > 256
            || challenge.user_code.is_empty()
            || challenge.user_code.len() > 128
            || challenge.verification_uri.len() > 2048
            || challenge.expires_in == 0
            || challenge.expires_in > 900
            || !challenge.verification_uri.starts_with("https://")
        {
            return Err(AuthError::Unavailable);
        }
        let now = self.clock.now();
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "DELETE FROM device_logins WHERE expires_at<=?1 OR consumed_at IS NOT NULL",
            params![now],
        )?;
        let active: i64 =
            tx.query_row("SELECT count(*) FROM device_logins", [], |row| row.get(0))?;
        if active >= self.limits.active_device_logins {
            return Err(AuthError::Limit);
        }
        tx.execute(
            "INSERT INTO device_logins(code_hash,audience,expires_at) VALUES(?1,?2,?3)",
            params![
                hash(&challenge.device_code).as_slice(),
                AUDIENCE,
                now + challenge.expires_in as i64
            ],
        )?;
        tx.commit()?;
        Ok(challenge)
    }

    pub fn finish_login(&self, code: &str) -> Result<TokenResponse, AuthError> {
        if code.is_empty() || code.len() > 256 {
            return Err(AuthError::Unauthorized);
        }
        let now = self.clock.now();
        let code_hash = hash(code);
        let valid: Option<i64> = self
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT expires_at FROM device_logins WHERE code_hash=?1 AND consumed_at IS NULL",
                params![code_hash.as_slice()],
                |row| row.get(0),
            )
            .optional()?;
        if valid.is_none_or(|expiry| expiry <= now) {
            return Err(AuthError::Unauthorized);
        }
        let provider = self.provider.as_ref().ok_or(AuthError::Unavailable)?;
        let assertion = provider
            .poll(code)
            .map_err(|_| AuthError::Unavailable)?
            .ok_or(AuthError::Pending)?;
        if assertion.audience != AUDIENCE
            || assertion.provider.is_empty()
            || assertion.provider.len() > 64
            || assertion.subject.is_empty()
            || assertion.subject.len() > 256
        {
            return Err(AuthError::Unauthorized);
        }
        let now = self.clock.now();
        let account_id = hex(&secret::<16>()?);
        let access_token = hex(&secret::<32>()?);
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            "UPDATE device_logins SET consumed_at=?2 WHERE code_hash=?1 AND consumed_at IS NULL AND expires_at>?2 AND audience=?3",
            params![code_hash.as_slice(), now, AUDIENCE],
        )?;
        if changed != 1 {
            return Err(AuthError::Unauthorized);
        }
        tx.execute(
            "INSERT INTO accounts(id,provider,provider_subject,created_at) VALUES(?1,?2,?3,?4) ON CONFLICT(provider,provider_subject) DO NOTHING",
            params![account_id, assertion.provider, assertion.subject, now],
        )?;
        let account_id: String = tx.query_row(
            "SELECT id FROM accounts WHERE provider=?1 AND provider_subject=?2 AND disabled_at IS NULL",
            params![assertion.provider, assertion.subject], |row| row.get(0),
        ).optional()?.ok_or(AuthError::Unauthorized)?;
        tx.execute(
            "INSERT INTO api_sessions(token_hash,account_id,scopes,expires_at) VALUES(?1,?2,?3,?4)",
            params![
                hash(&access_token).as_slice(),
                account_id,
                SCOPES,
                now + TOKEN_SECONDS
            ],
        )?;
        tx.commit()?;
        Ok(TokenResponse {
            access_token,
            token_type: "Bearer",
            expires_in: TOKEN_SECONDS,
        })
    }

    pub fn authenticate(&self, token: &str) -> Result<Account, AuthError> {
        if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(AuthError::Unauthorized);
        }
        let db = self.db.lock().unwrap();
        let account = db.query_row(
            "SELECT a.id,a.provider,a.provider_subject,s.scopes FROM api_sessions s JOIN accounts a ON a.id=s.account_id \
             WHERE s.token_hash=?1 AND s.expires_at>?2 AND s.revoked_at IS NULL AND a.disabled_at IS NULL",
            params![hash(token).as_slice(), self.clock.now()],
            |row| Ok(Account { id: row.get(0)?, provider: row.get(1)?, provider_subject: row.get(2)?,
                scopes: row.get::<_, String>(3)?.split_whitespace().map(str::to_owned).collect() }),
        ).optional()?;
        account.ok_or(AuthError::Unauthorized)
    }

    pub fn authorize(&self, token: &str, scope: &str) -> Result<Account, AuthError> {
        let account = self.authenticate(token)?;
        if !account.scopes.iter().any(|s| s == scope) {
            return Err(AuthError::Unauthorized);
        }
        Ok(account)
    }

    pub fn logout(&self, token: &str) -> Result<(), AuthError> {
        self.authenticate(token)?;
        self.db.lock().unwrap().execute(
            "UPDATE api_sessions SET revoked_at=?2 WHERE token_hash=?1 AND revoked_at IS NULL",
            params![hash(token).as_slice(), self.clock.now()],
        )?;
        Ok(())
    }
}
