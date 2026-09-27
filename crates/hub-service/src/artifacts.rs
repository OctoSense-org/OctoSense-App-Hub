//! Private, content-addressed upload quarantine. Completing an upload does
//! not enqueue validation or make the artifact public.
use crate::{auth::AuthError, publishers::new_id, Service};
use http_body_util::BodyExt;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::Serialize;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const MAX_PACK_BYTES: u64 = 12 * 1024 * 1024;

pub struct BlobStore {
    objects: PathBuf,
    quarantine: PathBuf,
}
impl BlobStore {
    pub(crate) fn new(root: &Path) -> std::io::Result<Self> {
        let objects = root.join("objects");
        let quarantine = root.join("quarantine");
        for path in [root, objects.as_path(), quarantine.as_path()] {
            std::fs::create_dir_all(path)?;
            if !std::fs::symlink_metadata(path)?.file_type().is_dir() {
                return Err(std::io::Error::other(
                    "blob storage must be real directories",
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        Ok(Self {
            objects,
            quarantine,
        })
    }

    fn stage(&self, bytes: &[u8]) -> Result<tempfile::NamedTempFile, UploadError> {
        if bytes.is_empty() || bytes.len() as u64 > MAX_PACK_BYTES {
            return Err(UploadError::Invalid);
        }
        let mut file = tempfile::NamedTempFile::new_in(&self.quarantine)?;
        file.write_all(bytes)?;
        Ok(file)
    }

    pub(crate) async fn stage_body(
        &self,
        mut body: axum::body::Body,
        expected_bytes: u64,
    ) -> Result<tempfile::NamedTempFile, UploadError> {
        use tokio::io::AsyncWriteExt;
        let file = tempfile::NamedTempFile::new_in(&self.quarantine)?;
        let mut output = tokio::fs::File::from_std(file.reopen()?);
        let mut total = 0u64;
        while let Some(frame) = body.frame().await {
            let frame = frame.map_err(|_| UploadError::Invalid)?;
            if let Ok(chunk) = frame.into_data() {
                total = total
                    .checked_add(chunk.len() as u64)
                    .ok_or(UploadError::Invalid)?;
                if total > MAX_PACK_BYTES || total > expected_bytes {
                    return Err(UploadError::Invalid);
                }
                output.write_all(&chunk).await?;
            }
        }
        if total != expected_bytes {
            return Err(UploadError::Invalid);
        }
        output.flush().await?;
        output.sync_all().await?;
        drop(output);
        Ok(file)
    }

    fn finalize(
        &self,
        mut staged: tempfile::NamedTempFile,
        digest: &str,
        bytes: u64,
    ) -> Result<(), UploadError> {
        staged.flush()?;
        staged.as_file().sync_all()?;
        if staged.as_file().metadata()?.len() != bytes {
            return Err(UploadError::Invalid);
        }
        let mut input = staged.reopen()?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = input.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        if hasher.finalize().to_hex().as_str() != digest {
            return Err(UploadError::Invalid);
        }
        let pack: octosense_app_hub::pack::Pack =
            serde_json::from_reader(staged.reopen()?).map_err(|_| UploadError::Invalid)?;
        let inspection = tempfile::tempdir_in(&self.quarantine)?;
        octosense_app_hub::pack::unpack(&pack, inspection.path())
            .map_err(|_| UploadError::Invalid)?;
        if !inspection.path().join("manifest.json").is_file() {
            return Err(UploadError::Invalid);
        }
        drop(inspection);
        let target = self.objects.join(digest);
        match staged.persist_noclobber(&target) {
            Ok(file) => {
                let mut permissions = file.metadata()?.permissions();
                permissions.set_readonly(true);
                file.set_permissions(permissions)?;
                file.sync_all()?;
                std::fs::File::open(&self.objects)?.sync_all()?;
            }
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                if !std::fs::symlink_metadata(&target)?.file_type().is_file()
                    || std::fs::metadata(&target)?.len() != bytes
                {
                    return Err(UploadError::Conflict);
                }
                let existing = std::fs::read(&target)?;
                if blake3::hash(&existing).to_hex().as_str() != digest {
                    return Err(UploadError::Conflict);
                }
            }
            Err(error) => return Err(UploadError::Io(error.error)),
        }
        Ok(())
    }

    pub(crate) fn inspect(
        &self,
        digest: &str,
    ) -> Result<octosense_app_policy::AppManifest, UploadError> {
        let path = self.objects.join(digest);
        if !std::fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err(UploadError::Invalid);
        }
        let file = std::fs::File::open(&path)?;
        if file.metadata()?.len() > MAX_PACK_BYTES {
            return Err(UploadError::Invalid);
        }
        let mut bytes = Vec::new();
        file.take(MAX_PACK_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_PACK_BYTES || blake3::hash(&bytes).to_hex().as_str() != digest {
            return Err(UploadError::Invalid);
        }
        let pack: octosense_app_hub::pack::Pack =
            serde_json::from_slice(&bytes).map_err(|_| UploadError::Invalid)?;
        let inspection = tempfile::tempdir_in(&self.quarantine)?;
        octosense_app_hub::pack::unpack(&pack, inspection.path())
            .map_err(|_| UploadError::Invalid)?;
        let text = std::fs::read_to_string(inspection.path().join("manifest.json"))?;
        let manifest =
            octosense_app_policy::AppManifest::parse(&text).map_err(|_| UploadError::Invalid)?;
        let actual = octosense_app_policy::bundle::digest_dir_limited(
            inspection.path(),
            octosense_app_hub::gate::MAX_BUNDLE_BYTES,
            octosense_app_hub::admission::MAX_ENTRIES,
            octosense_app_hub::admission::MAX_DEPTH,
        )
        .map_err(|_| UploadError::Invalid)?;
        if !actual.eq_ignore_ascii_case(&manifest.integrity.bundle_blake3) {
            return Err(UploadError::Invalid);
        }
        Ok(manifest)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Upload {
    pub id: String,
    pub app_id: String,
    pub expected_digest: String,
    pub expected_bytes: u64,
    pub status: String,
}

#[derive(Debug)]
pub enum UploadError {
    Invalid,
    Unauthorized,
    Conflict,
    Database(rusqlite::Error),
    Io(std::io::Error),
}
impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid upload or artifact",
            Self::Unauthorized => "upload authorization required",
            Self::Conflict => "upload conflicts with an existing artifact",
            Self::Database(_) | Self::Io(_) => "upload storage unavailable",
        })
    }
}
impl std::error::Error for UploadError {}
impl From<rusqlite::Error> for UploadError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}
impl From<std::io::Error> for UploadError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<AuthError> for UploadError {
    fn from(_: AuthError) -> Self {
        Self::Unauthorized
    }
}

impl Service {
    pub fn create_upload(
        &self,
        token: &str,
        app_id: &str,
        expected_digest: &str,
        expected_bytes: u64,
    ) -> Result<Upload, UploadError> {
        let account = self.authorize(token, "apps.submit")?;
        if expected_bytes == 0
            || expected_bytes > MAX_PACK_BYTES
            || expected_digest.len() != 64
            || !expected_digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(UploadError::Invalid);
        }
        let digest = expected_digest.to_ascii_lowercase();
        let id = new_id()
            .map_err(|_| UploadError::Io(std::io::Error::other("random ID unavailable")))?;
        let now = self.clock.now();
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let publisher_id: String = tx.query_row(
            "SELECT c.publisher_id FROM app_claims c \
             JOIN publisher_memberships m ON m.publisher_id=c.publisher_id \
             JOIN publishers p ON p.id=c.publisher_id JOIN accounts a ON a.id=m.account_id \
             WHERE c.app_id=?1 AND c.status='active' AND m.account_id=?2 \
             AND m.role IN ('owner','submitter') AND p.disabled_at IS NULL AND a.disabled_at IS NULL",
            params![app_id, account.id], |row| row.get(0),
        ).optional()?.ok_or(UploadError::Unauthorized)?;
        tx.execute("INSERT INTO uploads(id,app_id,publisher_id,initiated_by,expected_digest,expected_bytes,status,created_at) \
                    VALUES(?1,?2,?3,?4,?5,?6,'pending',?7) ON CONFLICT(app_id,expected_digest) DO NOTHING",
            params![id, app_id, publisher_id, account.id, digest, expected_bytes as i64, now])?;
        let upload = tx.query_row(
            "SELECT id,app_id,expected_digest,expected_bytes,status FROM uploads \
            WHERE app_id=?1 AND expected_digest=?2",
            params![app_id, digest],
            |row| {
                Ok(Upload {
                    id: row.get(0)?,
                    app_id: row.get(1)?,
                    expected_digest: row.get(2)?,
                    expected_bytes: row.get(3)?,
                    status: row.get(4)?,
                })
            },
        )?;
        if upload.expected_bytes != expected_bytes {
            return Err(UploadError::Conflict);
        }
        tx.commit()?;
        Ok(upload)
    }

    pub fn upload_status(&self, token: &str, upload_id: &str) -> Result<Upload, UploadError> {
        let account = self.authorize(token, "publisher.read")?;
        let db = self.db.lock().unwrap();
        db.query_row("SELECT u.id,u.app_id,u.expected_digest,u.expected_bytes,u.status FROM uploads u \
            JOIN publisher_memberships m ON m.publisher_id=u.publisher_id JOIN publishers p ON p.id=u.publisher_id \
            JOIN accounts a ON a.id=m.account_id JOIN app_claims c ON c.app_id=u.app_id \
            WHERE u.id=?1 AND m.account_id=?2 AND m.role IN ('owner','submitter') \
            AND p.disabled_at IS NULL AND a.disabled_at IS NULL AND c.status='active'",
            params![upload_id, account.id], |row| Ok(Upload { id: row.get(0)?, app_id: row.get(1)?,
                expected_digest: row.get(2)?, expected_bytes: row.get(3)?, status: row.get(4)? }))
            .optional()?.ok_or(UploadError::Unauthorized)
    }

    pub fn put_upload(
        &self,
        token: &str,
        upload_id: &str,
        bytes: &[u8],
    ) -> Result<Upload, UploadError> {
        self.authorize(token, "apps.submit")?;
        let upload = self.upload_status(token, upload_id)?;
        if bytes.len() as u64 != upload.expected_bytes {
            return Err(UploadError::Invalid);
        }
        let staged = self.blobs.stage(bytes)?;
        self.put_upload_staged(token, upload_id, staged)
    }

    pub(crate) fn put_upload_staged(
        &self,
        token: &str,
        upload_id: &str,
        staged: tempfile::NamedTempFile,
    ) -> Result<Upload, UploadError> {
        let account = self.authorize(token, "apps.submit")?;
        let upload = self.upload_status(token, upload_id)?;
        self.blobs
            .finalize(staged, &upload.expected_digest, upload.expected_bytes)?;
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("UPDATE uploads SET status='complete',completed_at=?2 WHERE id=?1 AND status='pending' \
            AND EXISTS(SELECT 1 FROM publisher_memberships m JOIN accounts a ON a.id=m.account_id \
                JOIN app_claims c ON c.publisher_id=m.publisher_id JOIN publishers p ON p.id=c.publisher_id \
                WHERE c.app_id=uploads.app_id AND c.status='active' AND m.account_id=?3 \
                AND m.role IN ('owner','submitter') AND a.disabled_at IS NULL AND p.disabled_at IS NULL)",
            params![upload_id, self.clock.now(), account.id])?;
        tx.commit()?;
        drop(db);
        self.upload_status(token, upload_id)
    }
}
