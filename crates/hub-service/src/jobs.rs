//! Durable leases for validation workers. This module records evidence and
//! moves a submission to human review; it never creates an approval.
use crate::{
    submissions::{record, Submission},
    Service,
};
use rand_core::TryRngCore;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::Serialize;

const LEASE_SECONDS: i64 = 300;
const MAX_REPORT_BYTES: usize = 64 * 1024;

/// The token is a secret capability for one attempt and must not be logged.
#[derive(Clone)]
pub struct JobLease {
    pub job_id: i64,
    pub submission_id: String,
    pub app_id: String,
    pub artifact_digest: String,
    pub token: String,
    pub lease_until: i64,
}

#[derive(Clone, Serialize)]
pub struct ValidationOutcome {
    pub passed: bool,
    pub validator_id: String,
    pub policy_version: String,
    pub report: serde_json::Value,
}

#[derive(Debug)]
pub enum JobError {
    Invalid,
    Stale,
    Database(rusqlite::Error),
    Random,
}

struct CurrentJob {
    status: String,
    lease_until: Option<i64>,
    token_hash: Option<Vec<u8>>,
    outcome_digest: Option<String>,
    submission_id: String,
    artifact_digest: String,
}
impl std::fmt::Display for JobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid validation result",
            Self::Stale => "validation lease expired or superseded",
            Self::Database(_) => "validation job store unavailable",
            Self::Random => "secure randomness unavailable",
        })
    }
}
impl std::error::Error for JobError {}
impl From<rusqlite::Error> for JobError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}

fn lease_token() -> Result<String, JobError> {
    let mut bytes = [0u8; 32];
    rand_core::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| JobError::Random)?;
    Ok(hex::encode(bytes))
}

impl Service {
    /// Internal worker operation; no public HTTP route exposes this method.
    pub fn lease_validation_job(&self) -> Result<Option<JobLease>, JobError> {
        let now = self.clock.now();
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        // A repeatedly crashing worker must produce an explicit failed state,
        // rather than leaving a validating submission stranded indefinitely.
        let mut exhausted_stmt = tx.prepare("SELECT j.id,s.id,s.artifact_digest FROM validation_jobs j \
            JOIN submissions s ON s.id=j.submission_id WHERE j.status='leased' AND j.lease_until<=?1 \
            AND j.attempts>=3 AND s.status='validating'")?;
        let exhausted: Vec<(i64, String, String)> = exhausted_stmt
            .query_map(params![now], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<Result<_, _>>()?;
        drop(exhausted_stmt);
        for (job_id, submission_id, digest) in exhausted {
            tx.execute("UPDATE validation_jobs SET status='failed',lease_until=NULL,lease_token_hash=NULL WHERE id=?1",
                params![job_id])?;
            tx.execute("UPDATE submissions SET status='failed',updated_at=?2 WHERE id=?1 AND status='validating'",
                params![submission_id, now])?;
            tx.execute("INSERT INTO submission_events(submission_id,actor_account_id,from_status,to_status,artifact_digest,created_at) \
                        VALUES(?1,NULL,'validating','failed',?2,?3)",
                params![submission_id, digest, now])?;
        }
        let candidate: Option<(i64, String, String, String, String)> = tx.query_row(
            "SELECT j.id,s.id,s.app_id,s.artifact_digest,s.status FROM validation_jobs j \
             JOIN submissions s ON s.id=j.submission_id \
             WHERE (j.status='queued' OR (j.status='leased' AND j.lease_until<=?1)) \
             AND j.attempts<3 AND s.status IN ('submitted','validating') ORDER BY j.created_at,j.id LIMIT 1",
            params![now], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?)),
        ).optional()?;
        let Some((job_id, submission_id, app_id, artifact_digest, prior_status)) = candidate else {
            tx.commit()?;
            return Ok(None);
        };
        let token = lease_token()?;
        let token_hash = blake3::hash(token.as_bytes());
        tx.execute(
            "UPDATE validation_jobs SET status='leased',lease_until=?2, \
                    lease_token_hash=?3,attempts=attempts+1 WHERE id=?1",
            params![
                job_id,
                now + LEASE_SECONDS,
                token_hash.as_bytes().as_slice()
            ],
        )?;
        if prior_status == "submitted" {
            tx.execute("UPDATE submissions SET status='validating',updated_at=?2 WHERE id=?1 AND status='submitted'",
                params![submission_id, now])?;
            tx.execute("INSERT INTO submission_events(submission_id,actor_account_id,from_status,to_status,artifact_digest,created_at) \
                        VALUES(?1,NULL,'submitted','validating',?2,?3)",
                params![submission_id, artifact_digest, now])?;
        }
        tx.commit()?;
        Ok(Some(JobLease {
            job_id,
            submission_id,
            app_id,
            artifact_digest,
            token,
            lease_until: now + LEASE_SECONDS,
        }))
    }

    /// Accept only the current unexpired lease. A completed retry with the
    /// same token and result is idempotent, including after service restart.
    pub fn complete_validation_job(
        &self,
        lease: &JobLease,
        outcome: &ValidationOutcome,
    ) -> Result<Submission, JobError> {
        if outcome.validator_id.is_empty()
            || outcome.validator_id.len() > 128
            || outcome.validator_id.chars().any(char::is_control)
            || outcome.policy_version.is_empty()
            || outcome.policy_version.len() > 128
            || outcome.policy_version.chars().any(char::is_control)
            || !outcome.report.is_object()
        {
            return Err(JobError::Invalid);
        }
        let report_json = serde_json::to_string(&outcome.report).map_err(|_| JobError::Invalid)?;
        if report_json.len() > MAX_REPORT_BYTES {
            return Err(JobError::Invalid);
        }
        let result_digest =
            blake3::hash(&serde_json::to_vec(outcome).map_err(|_| JobError::Invalid)?)
                .to_hex()
                .to_string();
        let token_hash = blake3::hash(lease.token.as_bytes());
        let now = self.clock.now();
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current: Option<CurrentJob> = tx.query_row(
            "SELECT j.status,j.lease_until,j.lease_token_hash,j.outcome_digest,s.id,s.artifact_digest \
             FROM validation_jobs j JOIN submissions s ON s.id=j.submission_id WHERE j.id=?1",
            params![lease.job_id], |row| Ok(CurrentJob { status: row.get(0)?,
                lease_until: row.get(1)?, token_hash: row.get(2)?, outcome_digest: row.get(3)?,
                submission_id: row.get(4)?, artifact_digest: row.get(5)? }),
        ).optional()?;
        let current = current.ok_or(JobError::Stale)?;
        if current.token_hash.as_deref() != Some(token_hash.as_bytes().as_slice())
            || current.submission_id != lease.submission_id
            || current.artifact_digest != lease.artifact_digest
        {
            return Err(JobError::Stale);
        }
        if current.status == "complete"
            && current.outcome_digest.as_deref() == Some(result_digest.as_str())
        {
            return tx.query_row("SELECT id,app_id,upload_id,version,release_number,status FROM submissions WHERE id=?1",
                params![current.submission_id], record).map_err(JobError::from);
        }
        if current.status != "leased" || current.lease_until.is_none_or(|until| until <= now) {
            return Err(JobError::Stale);
        }
        let next_status = if outcome.passed {
            "awaiting_review"
        } else {
            "rejected"
        };
        tx.execute("INSERT INTO validation_evidence(submission_id,artifact_digest,result_digest,validator_id, \
            policy_version,passed,report_json,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![current.submission_id, current.artifact_digest, result_digest, outcome.validator_id,
                outcome.policy_version, outcome.passed as i64, report_json, now])?;
        tx.execute("UPDATE validation_jobs SET status='complete',lease_until=NULL,outcome_digest=?2 WHERE id=?1",
            params![lease.job_id, result_digest])?;
        let changed = tx.execute(
            "UPDATE submissions SET status=?2,updated_at=?3 WHERE id=?1 AND status='validating'",
            params![current.submission_id, next_status, now],
        )?;
        if changed != 1 {
            return Err(JobError::Stale);
        }
        tx.execute("INSERT INTO submission_events(submission_id,actor_account_id,from_status,to_status,artifact_digest,created_at) \
                    VALUES(?1,NULL,'validating',?2,?3,?4)",
            params![current.submission_id, next_status, current.artifact_digest, now])?;
        let result = tx.query_row(
            "SELECT id,app_id,upload_id,version,release_number,status FROM submissions WHERE id=?1",
            params![current.submission_id],
            record,
        )?;
        tx.commit()?;
        Ok(result)
    }
}
