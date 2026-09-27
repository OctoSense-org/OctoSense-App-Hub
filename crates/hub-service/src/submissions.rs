//! Idempotent, signed submission records. A submission is only queued for
//! validation; neither an API field nor this module can approve publication.
use crate::{artifacts::UploadError, auth::AuthError, publishers::new_id, Service};
use octosense_app_policy::SignatureVerifier;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionInput {
    pub upload_id: String,
    #[serde(default)]
    pub source_repository: Option<String>,
    #[serde(default)]
    pub source_commit: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Submission {
    pub id: String,
    pub app_id: String,
    pub upload_id: String,
    pub version: String,
    pub release_number: u64,
    pub status: String,
}

#[derive(Debug)]
pub enum SubmissionError {
    Invalid,
    Unauthorized,
    Conflict,
    Database(rusqlite::Error),
    Storage,
}
impl std::fmt::Display for SubmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid signed submission",
            Self::Unauthorized => "submission authorization required",
            Self::Conflict => "submission conflicts with an existing release",
            Self::Database(_) | Self::Storage => "submission storage unavailable",
        })
    }
}
impl std::error::Error for SubmissionError {}
impl From<rusqlite::Error> for SubmissionError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}
impl From<AuthError> for SubmissionError {
    fn from(_: AuthError) -> Self {
        Self::Unauthorized
    }
}
impl From<UploadError> for SubmissionError {
    fn from(error: UploadError) -> Self {
        match error {
            UploadError::Unauthorized => Self::Unauthorized,
            UploadError::Database(error) => Self::Database(error),
            UploadError::Io(_) => Self::Storage,
            UploadError::Invalid | UploadError::Conflict => Self::Invalid,
        }
    }
}

fn valid_source(input: &SubmissionInput) -> bool {
    match (&input.source_repository, &input.source_commit) {
        (None, None) => true,
        (Some(repository), Some(commit)) => {
            repository.starts_with("https://")
                && repository.len() <= 2048
                && !repository.chars().any(char::is_control)
                && (7..=64).contains(&commit.len())
                && commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        }
        _ => false,
    }
}

fn record(row: &rusqlite::Row<'_>) -> rusqlite::Result<Submission> {
    Ok(Submission {
        id: row.get(0)?,
        app_id: row.get(1)?,
        upload_id: row.get(2)?,
        version: row.get(3)?,
        release_number: row.get(4)?,
        status: row.get(5)?,
    })
}

impl Service {
    pub fn create_submission(
        &self,
        token: &str,
        app_id: &str,
        idempotency_key: &str,
        input: &SubmissionInput,
    ) -> Result<Submission, SubmissionError> {
        let account = self.authorize(token, "apps.submit")?;
        if !(8..=128).contains(&idempotency_key.len())
            || !idempotency_key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
            || !valid_source(input)
        {
            return Err(SubmissionError::Invalid);
        }
        let upload = self.upload_status(token, &input.upload_id)?;
        if upload.status != "complete" || upload.app_id != app_id {
            return Err(SubmissionError::Invalid);
        }
        let manifest = self.blobs.inspect(&upload.expected_digest)?;
        if manifest.id != app_id || manifest.schema != 2 {
            return Err(SubmissionError::Invalid);
        }
        let release_number = manifest
            .contract()
            .ok_or(SubmissionError::Invalid)?
            .release_number;
        if release_number > i64::MAX as u64 {
            return Err(SubmissionError::Invalid);
        }
        let signature = manifest
            .integrity
            .signature
            .as_ref()
            .ok_or(SubmissionError::Invalid)?;
        let signing_bytes = manifest
            .signing_bytes()
            .map_err(|_| SubmissionError::Invalid)?;
        let manifest_digest =
            blake3::hash(&serde_json::to_vec(&manifest).map_err(|_| SubmissionError::Invalid)?)
                .to_hex()
                .to_string();
        let request_digest = blake3::hash(
            &serde_json::to_vec(&(app_id, input)).map_err(|_| SubmissionError::Invalid)?,
        )
        .to_hex()
        .to_string();
        let now = self.clock.now();
        let id = new_id().map_err(|_| SubmissionError::Storage)?;
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let binding: (String, Vec<u8>) = tx.query_row(
            "SELECT c.publisher_id,k.public_key FROM app_claims c \
             JOIN publisher_memberships m ON m.publisher_id=c.publisher_id \
             JOIN publishers p ON p.id=c.publisher_id \
             JOIN accounts a ON a.id=m.account_id \
             JOIN signing_keys k ON k.publisher_id=c.publisher_id AND k.revoked_at IS NULL \
             JOIN uploads u ON u.app_id=c.app_id AND u.publisher_id=c.publisher_id \
             WHERE c.app_id=?1 AND c.status='active' AND m.account_id=?2 \
             AND m.role IN ('owner','submitter') AND a.disabled_at IS NULL AND p.disabled_at IS NULL \
             AND u.id=?3 AND u.status='complete' AND u.expected_digest=?4",
            params![app_id, account.id, input.upload_id, upload.expected_digest],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?.ok_or(SubmissionError::Unauthorized)?;
        if signature.key_id != binding.0 || binding.1.len() != 32 {
            return Err(SubmissionError::Invalid);
        }
        octosense_app_hub::signing::PublisherKeys::new()
            .with(&binding.0, &hex::encode(&binding.1))
            .verify(&signature.key_id, &signature.value, &signing_bytes)
            .map_err(|_| SubmissionError::Invalid)?;
        let previous: Option<(Submission, String)> = tx.query_row(
            "SELECT id,app_id,upload_id,version,release_number,status,request_digest FROM submissions \
             WHERE publisher_id=?1 AND idempotency_key=?2",
            params![binding.0, idempotency_key],
            |row| Ok((record(row)?, row.get(6)?)),
        ).optional()?;
        if let Some((previous, digest)) = previous {
            return if digest == request_digest {
                Ok(previous)
            } else {
                Err(SubmissionError::Conflict)
            };
        }
        let inserted = tx.execute(
            "INSERT INTO submissions(id,publisher_id,app_id,upload_id,version,release_number, \
             manifest_digest,bundle_digest,artifact_digest,source_repository,source_commit, \
             idempotency_key,request_digest,status,created_at,updated_at) \
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,'submitted',?14,?14)",
            params![
                id,
                binding.0,
                app_id,
                input.upload_id,
                manifest.version,
                release_number as i64,
                manifest_digest,
                manifest.integrity.bundle_blake3,
                upload.expected_digest,
                input.source_repository,
                input.source_commit,
                idempotency_key,
                request_digest,
                now
            ],
        );
        if let Err(error) = inserted {
            return Err(
                if matches!(error, rusqlite::Error::SqliteFailure(ref inner, _)
                if inner.code == rusqlite::ErrorCode::ConstraintViolation)
                {
                    SubmissionError::Conflict
                } else {
                    SubmissionError::Database(error)
                },
            );
        }
        tx.execute(
            "INSERT INTO validation_jobs(submission_id,status,created_at) VALUES(?1,'queued',?2)",
            params![id, now],
        )?;
        tx.execute("INSERT INTO submission_events(submission_id,actor_account_id,from_status,to_status,artifact_digest,created_at) \
                    VALUES(?1,?2,NULL,'submitted',?3,?4)",
            params![id, account.id, upload.expected_digest, now])?;
        tx.execute(
            "INSERT INTO audit_events(actor_account_id,publisher_id,event,subject,created_at) \
                    VALUES(?1,?2,'app.submitted',?3,?4)",
            params![account.id, binding.0, id, now],
        )?;
        tx.commit()?;
        Ok(Submission {
            id,
            app_id: app_id.into(),
            upload_id: input.upload_id.clone(),
            version: manifest.version,
            release_number,
            status: "submitted".into(),
        })
    }
}
