//! Transactional publisher ownership. App IDs and publisher slugs are immutable
//! identifiers; display names are deliberately separate from ownership.
use crate::{auth::AuthError, Service};
use ed25519_dalek::{Signature, VerifyingKey};
use rand_core::TryRngCore;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Publisher {
    pub id: String,
    pub slug: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Claim {
    pub app_id: String,
    pub publisher_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct KeyChallenge {
    pub id: String,
    pub message: String,
    pub expires_in: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SigningKeyRecord {
    pub id: String,
    pub publisher_id: String,
    pub public_key: String,
}

#[derive(Debug)]
pub enum RegistryError {
    Invalid,
    Unauthorized,
    Conflict,
    Limit,
    Database(rusqlite::Error),
    Random,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Invalid => "invalid publisher or app identifier",
            Self::Unauthorized => "publisher authorization required",
            Self::Conflict => "publisher or app identifier already claimed",
            Self::Limit => "publisher quota reached",
            Self::Database(_) => "publisher store unavailable",
            Self::Random => "secure randomness unavailable",
        };
        f.write_str(message)
    }
}
impl std::error::Error for RegistryError {}
impl From<rusqlite::Error> for RegistryError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}
impl From<AuthError> for RegistryError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::Unauthorized => Self::Unauthorized,
            AuthError::Limit => Self::Limit,
            AuthError::Database(error) => Self::Database(error),
            AuthError::Random => Self::Random,
            AuthError::Pending | AuthError::Unavailable => Self::Unauthorized,
        }
    }
}

fn identifier(value: &str, max: usize) -> Result<String, RegistryError> {
    let canonical = value.trim().to_ascii_lowercase();
    let bytes = canonical.as_bytes();
    if bytes.is_empty()
        || bytes.len() > max
        || !bytes[0].is_ascii_alphanumeric()
        || !bytes[bytes.len() - 1].is_ascii_alphanumeric()
        || canonical.contains("..")
        || !bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-' || *b == b'.')
    {
        return Err(RegistryError::Invalid);
    }
    Ok(canonical)
}

fn new_id() -> Result<String, RegistryError> {
    let mut bytes = [0u8; 16];
    rand_core::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| RegistryError::Random)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

impl Service {
    pub fn create_publisher(
        &self,
        token: &str,
        slug: &str,
        display_name: &str,
    ) -> Result<Publisher, RegistryError> {
        let account = self.authorize(token, "publisher.write")?;
        let slug = identifier(slug, 40)?;
        if slug.contains('.')
            || matches!(
                slug.as_str(),
                "octosense" | "system" | "official" | "admin" | "support" | "core"
            )
        {
            return Err(RegistryError::Invalid);
        }
        let display_name = display_name.trim();
        if display_name.is_empty()
            || display_name.len() > 80
            || display_name.chars().any(char::is_control)
        {
            return Err(RegistryError::Invalid);
        }
        let publisher = Publisher {
            id: new_id()?,
            slug,
            display_name: display_name.to_owned(),
        };
        let now = self.clock.now();
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let count: i64 = tx.query_row(
            "SELECT count(*) FROM publishers WHERE owner_account_id=?1 AND disabled_at IS NULL",
            params![account.id],
            |row| row.get(0),
        )?;
        if count >= self.limits.publishers_per_account {
            return Err(RegistryError::Limit);
        }
        let inserted = tx.execute(
            "INSERT INTO publishers(id,slug,display_name,owner_account_id,created_at) \
             SELECT ?1,?2,?3,a.id,?5 FROM accounts a WHERE a.id=?4 AND a.disabled_at IS NULL \
             ON CONFLICT(slug) DO NOTHING",
            params![
                publisher.id,
                publisher.slug,
                publisher.display_name,
                account.id,
                now
            ],
        )?;
        if inserted == 0 {
            let taken: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM publishers WHERE slug=?1)",
                params![publisher.slug],
                |row| row.get(0),
            )?;
            return Err(if taken {
                RegistryError::Conflict
            } else {
                RegistryError::Unauthorized
            });
        }
        tx.execute("INSERT INTO publisher_memberships(publisher_id,account_id,role,created_at) VALUES(?1,?2,'owner',?3)",
            params![publisher.id, account.id, now])?;
        tx.execute("INSERT INTO audit_events(actor_account_id,publisher_id,event,subject,created_at) VALUES(?1,?2,'publisher.created',?3,?4)",
            params![account.id, publisher.id, publisher.slug, now])?;
        tx.commit()?;
        Ok(publisher)
    }

    pub fn claim_app(
        &self,
        token: &str,
        publisher_id: &str,
        app_id: &str,
    ) -> Result<Claim, RegistryError> {
        let account = self.authorize(token, "publisher.write")?;
        let app_id = identifier(app_id, 64)?;
        let now = self.clock.now();
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let slug: String = tx
            .query_row(
                "SELECT p.slug FROM publishers p \
             JOIN publisher_memberships m ON m.publisher_id=p.id \
             JOIN accounts a ON a.id=m.account_id \
             WHERE p.id=?1 AND m.account_id=?2 AND m.role IN ('owner','submitter') \
             AND p.disabled_at IS NULL AND a.disabled_at IS NULL",
                params![publisher_id, account.id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(RegistryError::Unauthorized)?;
        let status = if app_id.starts_with(&format!("{slug}.")) {
            "active"
        } else {
            "review_required"
        };
        let existing: Option<String> = tx
            .query_row(
                "SELECT publisher_id FROM app_claims WHERE app_id=?1",
                params![app_id],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(owner) = existing {
            if owner != publisher_id {
                return Err(RegistryError::Conflict);
            }
        } else {
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM app_claims WHERE publisher_id=?1",
                params![publisher_id],
                |row| row.get(0),
            )?;
            if count >= self.limits.apps_per_publisher {
                return Err(RegistryError::Limit);
            }
        }
        let inserted = tx.execute(
            "INSERT INTO app_claims(app_id,publisher_id,claimed_by,claimed_at,status) \
             VALUES(?1,?2,?3,?4,?5) ON CONFLICT(app_id) DO NOTHING",
            params![app_id, publisher_id, account.id, now, status],
        )?;
        let claim = tx.query_row(
            "SELECT app_id,publisher_id,status FROM app_claims WHERE app_id=?1",
            params![app_id],
            |row| {
                Ok(Claim {
                    app_id: row.get(0)?,
                    publisher_id: row.get(1)?,
                    status: row.get(2)?,
                })
            },
        )?;
        if claim.publisher_id != publisher_id {
            return Err(RegistryError::Conflict);
        }
        if inserted == 1 {
            tx.execute("INSERT INTO audit_events(actor_account_id,publisher_id,event,subject,created_at) VALUES(?1,?2,'app.claimed',?3,?4)",
                params![account.id, publisher_id, app_id, now])?;
        }
        tx.commit()?;
        Ok(claim)
    }

    /// A challenge proves control of an Ed25519 key before it can become the
    /// publisher's immutable v1 signing binding. Rotation has a separate flow.
    pub fn begin_key_enrollment(
        &self,
        token: &str,
        publisher_id: &str,
        public_key_hex: &str,
    ) -> Result<KeyChallenge, RegistryError> {
        let account = self.authorize(token, "publisher.write")?;
        let public_key = key_bytes(public_key_hex)?;
        let id = new_id()?;
        let mut nonce = [0u8; 32];
        rand_core::OsRng
            .try_fill_bytes(&mut nonce)
            .map_err(|_| RegistryError::Random)?;
        let now = self.clock.now();
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        require_owner(&tx, publisher_id, &account.id)?;
        let bound: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM signing_keys WHERE id=?1 OR public_key=?2)",
            params![publisher_id, public_key.as_slice()],
            |row| row.get(0),
        )?;
        if bound {
            return Err(RegistryError::Conflict);
        }
        tx.execute(
            "INSERT INTO key_challenges(id,publisher_id,account_id,public_key,nonce,expires_at) \
                    VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                id,
                publisher_id,
                account.id,
                public_key.as_slice(),
                nonce.as_slice(),
                now + 300
            ],
        )?;
        tx.commit()?;
        Ok(KeyChallenge {
            message: key_message(publisher_id, &id, &nonce),
            id,
            expires_in: 300,
        })
    }

    pub fn finish_key_enrollment(
        &self,
        token: &str,
        publisher_id: &str,
        challenge_id: &str,
        signature_hex: &str,
    ) -> Result<SigningKeyRecord, RegistryError> {
        let account = self.authorize(token, "publisher.write")?;
        let signature = hex::decode(signature_hex).map_err(|_| RegistryError::Invalid)?;
        let signature: [u8; 64] = signature.try_into().map_err(|_| RegistryError::Invalid)?;
        let now = self.clock.now();
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        require_owner(&tx, publisher_id, &account.id)?;
        let (public_key, nonce): (Vec<u8>, Vec<u8>) = tx
            .query_row(
                "SELECT public_key,nonce FROM key_challenges WHERE id=?1 AND publisher_id=?2 \
             AND account_id=?3 AND consumed_at IS NULL AND expires_at>?4",
                params![challenge_id, publisher_id, account.id, now],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or(RegistryError::Unauthorized)?;
        let public_key: [u8; 32] = public_key.try_into().map_err(|_| RegistryError::Invalid)?;
        let verifying_key =
            VerifyingKey::from_bytes(&public_key).map_err(|_| RegistryError::Invalid)?;
        verifying_key
            .verify_strict(
                &key_message(publisher_id, challenge_id, &nonce).into_bytes(),
                &Signature::from_bytes(&signature),
            )
            .map_err(|_| RegistryError::Invalid)?;
        let inserted = tx.execute(
            "INSERT INTO signing_keys(id,publisher_id,public_key,enrolled_by,enrolled_at) \
            VALUES(?1,?1,?2,?3,?4)",
            params![publisher_id, public_key.as_slice(), account.id, now],
        );
        if let Err(error) = inserted {
            return Err(
                if matches!(error, rusqlite::Error::SqliteFailure(ref inner, _)
                if inner.code == rusqlite::ErrorCode::ConstraintViolation)
                {
                    RegistryError::Conflict
                } else {
                    RegistryError::Database(error)
                },
            );
        }
        tx.execute(
            "UPDATE key_challenges SET consumed_at=?2 WHERE id=?1",
            params![challenge_id, now],
        )?;
        tx.execute(
            "INSERT INTO audit_events(actor_account_id,publisher_id,event,subject,created_at) \
                    VALUES(?1,?2,'key.enrolled',?3,?4)",
            params![account.id, publisher_id, hex::encode(public_key), now],
        )?;
        tx.commit()?;
        Ok(SigningKeyRecord {
            id: publisher_id.to_owned(),
            publisher_id: publisher_id.to_owned(),
            public_key: hex::encode(public_key),
        })
    }
}

fn require_owner(
    tx: &rusqlite::Transaction<'_>,
    publisher_id: &str,
    account_id: &str,
) -> Result<(), RegistryError> {
    let owner: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM publishers p JOIN publisher_memberships m ON m.publisher_id=p.id \
         JOIN accounts a ON a.id=m.account_id WHERE p.id=?1 AND m.account_id=?2 AND m.role='owner' \
         AND p.disabled_at IS NULL AND a.disabled_at IS NULL)",
        params![publisher_id, account_id], |row| row.get(0),
    )?;
    if owner {
        Ok(())
    } else {
        Err(RegistryError::Unauthorized)
    }
}

fn key_bytes(public_key_hex: &str) -> Result<[u8; 32], RegistryError> {
    let bytes = hex::decode(public_key_hex).map_err(|_| RegistryError::Invalid)?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| RegistryError::Invalid)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| RegistryError::Invalid)?;
    Ok(bytes)
}

fn key_message(publisher_id: &str, challenge_id: &str, nonce: &[u8]) -> String {
    format!(
        "octosense.publisher-key.v1\n{publisher_id}\n{challenge_id}\n{}",
        hex::encode(nonce)
    )
}
