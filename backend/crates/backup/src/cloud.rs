//! The cloud side (PRD FR-20.3, FR-20.6, NFR-BAK-6, NFR-BAK-9): one small trait, a folder
//! stand-in for tests and for D5-Q2, and the `object_store` adapter for S3 (Cloudflare R2).
//!
//! Only encrypted bundles, their manifests and the readable pack go through it.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::error::{BackupCode, BackupError, CloudReason};
use crate::io::{CHUNK, IoHook, copy_atomic, partial_path, remove_quietly};
use crate::model::BackupKind;
use crate::secrets::{SecretName, SecretStore};
use crate::settings::BackupSettings;

/// Why a cloud call failed. An offline failure is retried by the queue; a refused one (wrong
/// keys, no such bucket) is retried too, but shown with its own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudError {
    /// No internet, a timeout, or a server error.
    Offline(String),
    /// The cloud answered and said no.
    Refused(String),
    /// The object is not there.
    NotFound,
}

impl CloudError {
    /// The error for the screen, with a short `reason` value.
    pub fn into_backup_error(self) -> BackupError {
        let (reason, detail) = match &self {
            CloudError::Offline(d) => (CloudReason::Offline, d.clone()),
            CloudError::Refused(d) => (CloudReason::Refused, d.clone()),
            CloudError::NotFound => (CloudReason::NotFound, "not found".to_owned()),
        };
        BackupError::new(BackupCode::ErrBackupCloudFailed, detail).with(
            "reason",
            mandi_core::error::ErrorValue::Text(reason.as_text().to_owned()),
        )
    }
}

/// One object in the cloud.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudObject {
    /// The full key, with the prefix.
    pub key: String,
    /// Size in bytes.
    pub size: u64,
    /// When it was stored.
    pub modified: DateTime<Utc>,
}

/// The cloud store. Keys use `/`. Methods block the calling worker thread.
pub trait CloudStore: Send + Sync {
    /// Uploads a file under `key`. The object is whole or absent.
    ///
    /// # Errors
    /// [`CloudError`].
    fn put_file(&self, key: &str, source: &Path, hook: &dyn IoHook) -> Result<(), CloudError>;

    /// Downloads an object to `dest`, through a `.partial` file.
    ///
    /// # Errors
    /// [`CloudError`].
    fn get_file(&self, key: &str, dest: &Path) -> Result<(), CloudError>;

    /// Every object whose key starts with `prefix` (a folder, ending in `/`).
    ///
    /// # Errors
    /// [`CloudError`].
    fn list(&self, prefix: &str) -> Result<Vec<CloudObject>, CloudError>;

    /// Removes an object. An object that is not there is not an error.
    ///
    /// # Errors
    /// [`CloudError`].
    fn delete(&self, key: &str) -> Result<(), CloudError>;
}

/// Makes the cloud store from the settings and the stored secrets. Tests give a factory that
/// returns a folder stand-in.
pub trait CloudFactory: Send + Sync {
    /// The store for these settings.
    ///
    /// # Errors
    /// `ERR_BACKUP_CLOUD_NOT_CONFIGURED` when the cloud copy is off or its details are missing.
    fn open(
        &self,
        settings: &BackupSettings,
        secrets: &dyn SecretStore,
    ) -> Result<Arc<dyn CloudStore>, BackupError>;

    /// True when [`CloudFactory::open`] would work: the cloud copy is on and its details and keys
    /// are all there. Cheap: it makes no connection.
    fn ready(&self, settings: &BackupSettings, secrets: &dyn SecretStore) -> bool;
}

fn not_configured(detail: &str) -> BackupError {
    BackupError::new(BackupCode::ErrBackupCloudNotConfigured, detail)
}

/// Builds the S3 store for Cloudflare R2 (or any S3-compatible service) with `object_store`.
#[derive(Debug, Clone, Copy, Default)]
pub struct S3Factory;

impl CloudFactory for S3Factory {
    fn open(
        &self,
        settings: &BackupSettings,
        secrets: &dyn SecretStore,
    ) -> Result<Arc<dyn CloudStore>, BackupError> {
        if !settings.cloud_enabled {
            return Err(not_configured("the cloud copy is off"));
        }
        let access = secrets.get(SecretName::CloudAccessKeyId)?;
        let secret = secrets.get(SecretName::CloudSecretAccessKey)?;
        let (Some(access), Some(secret)) = (access, secret) else {
            return Err(not_configured("no cloud keys"));
        };
        // A request that hangs must not hold the app up at exit: short timeouts and a few tries.
        let options = object_store::ClientOptions::new()
            .with_connect_timeout(Duration::from_secs(10))
            .with_timeout(Duration::from_secs(60));
        let retry = object_store::RetryConfig {
            backoff: object_store::BackoffConfig::default(),
            max_retries: 3,
            retry_timeout: Duration::from_secs(90),
        };
        let store = object_store::aws::AmazonS3Builder::new()
            .with_client_options(options)
            .with_retry(retry)
            .with_endpoint(settings.cloud_endpoint.clone())
            .with_bucket_name(settings.cloud_bucket.clone())
            .with_region(settings.cloud_region.clone())
            .with_access_key_id(access)
            .with_secret_access_key(secret)
            .build()
            .map_err(|e| not_configured(&format!("cloud account: {e}")))?;
        Ok(Arc::new(ObjectStoreCloud::new(
            Arc::new(store),
            &settings.cloud_prefix,
        )?))
    }

    fn ready(&self, settings: &BackupSettings, secrets: &dyn SecretStore) -> bool {
        let has = |name| matches!(secrets.get(name), Ok(Some(value)) if !value.is_empty());
        settings.cloud_enabled
            && !settings.cloud_endpoint.trim().is_empty()
            && !settings.cloud_bucket.trim().is_empty()
            && has(SecretName::CloudAccessKeyId)
            && has(SecretName::CloudSecretAccessKey)
    }
}

/// A cloud store on top of any `object_store` backend: S3 in the app, memory in tests.
pub struct ObjectStoreCloud {
    store: Arc<dyn object_store::ObjectStore>,
    prefix: String,
    runtime: tokio::runtime::Runtime,
}

impl ObjectStoreCloud {
    /// Wraps `store`. Every key is stored under `prefix` (for example `mandi/`).
    ///
    /// # Errors
    /// `ERR_INTERNAL` if the small runtime cannot start.
    pub fn new(
        store: Arc<dyn object_store::ObjectStore>,
        prefix: &str,
    ) -> Result<Self, BackupError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| BackupError::internal(format!("cloud runtime: {e}")))?;
        let prefix = prefix.trim_matches('/');
        Ok(Self {
            store,
            prefix: if prefix.is_empty() {
                String::new()
            } else {
                format!("{prefix}/")
            },
            runtime,
        })
    }

    fn path(&self, key: &str) -> object_store::path::Path {
        object_store::path::Path::from(format!("{}{key}", self.prefix))
    }
}

fn classify(error: &object_store::Error) -> CloudError {
    match error {
        object_store::Error::NotFound { .. } => CloudError::NotFound,
        object_store::Error::Unauthenticated { .. }
        | object_store::Error::PermissionDenied { .. } => CloudError::Refused(error.to_string()),
        _ => CloudError::Offline(error.to_string()),
    }
}

impl CloudStore for ObjectStoreCloud {
    fn put_file(&self, key: &str, source: &Path, hook: &dyn IoHook) -> Result<(), CloudError> {
        use tokio::io::AsyncWriteExt;
        let location = self.path(key);
        let mut file =
            File::open(source).map_err(|e| CloudError::Refused(format!("source: {e}")))?;
        self.runtime.block_on(async {
            let mut writer =
                object_store::buffered::BufWriter::new(Arc::clone(&self.store), location);
            let mut buffer = vec![0_u8; CHUNK];
            let mut sent: u64 = 0;
            loop {
                let count = file
                    .read(&mut buffer)
                    .map_err(|e| CloudError::Refused(format!("source: {e}")))?;
                if count == 0 {
                    break;
                }
                hook.before_write(BackupKind::Cloud, sent)
                    .map_err(|e| CloudError::Offline(e.to_string()))?;
                let Some(chunk) = buffer.get(..count) else {
                    break;
                };
                writer
                    .write_all(chunk)
                    .await
                    .map_err(|e| CloudError::Offline(e.to_string()))?;
                sent = sent.saturating_add(u64::try_from(count).unwrap_or(0));
            }
            // The object appears only when shutdown succeeds.
            writer
                .shutdown()
                .await
                .map_err(|e| CloudError::Offline(e.to_string()))
        })
    }

    fn get_file(&self, key: &str, dest: &Path) -> Result<(), CloudError> {
        use object_store::ObjectStoreExt;
        let location = self.path(key);
        let partial = partial_path(dest);
        let result = self.runtime.block_on(async {
            let meta = self.store.head(&location).await.map_err(|e| classify(&e))?;
            if let Some(dir) = partial.parent() {
                fs::create_dir_all(dir).map_err(|e| CloudError::Refused(format!("folder: {e}")))?;
            }
            let mut out =
                File::create(&partial).map_err(|e| CloudError::Refused(format!("file: {e}")))?;
            let step = u64::try_from(CHUNK).unwrap_or(1);
            let mut at: u64 = 0;
            while at < meta.size {
                let end = at.saturating_add(step).min(meta.size);
                let bytes = self
                    .store
                    .get_range(&location, at..end)
                    .await
                    .map_err(|e| classify(&e))?;
                out.write_all(&bytes)
                    .map_err(|e| CloudError::Refused(format!("file: {e}")))?;
                at = end;
            }
            out.sync_all()
                .map_err(|e| CloudError::Refused(format!("file: {e}")))?;
            fs::rename(&partial, dest).map_err(|e| CloudError::Refused(format!("file: {e}")))
        });
        if result.is_err() {
            remove_quietly(&partial);
        }
        result
    }

    fn list(&self, prefix: &str) -> Result<Vec<CloudObject>, CloudError> {
        let location = self.path(prefix.trim_end_matches('/'));
        let strip = self.prefix.clone();
        self.runtime.block_on(async {
            let listing = self
                .store
                .list_with_delimiter(Some(&location))
                .await
                .map_err(|e| classify(&e))?;
            Ok(listing
                .objects
                .into_iter()
                .map(|meta| {
                    let full = meta.location.to_string();
                    CloudObject {
                        key: full.strip_prefix(&strip).unwrap_or(&full).to_owned(),
                        size: meta.size,
                        modified: meta.last_modified,
                    }
                })
                .collect())
        })
    }

    fn delete(&self, key: &str) -> Result<(), CloudError> {
        use object_store::ObjectStoreExt;
        let location = self.path(key);
        self.runtime.block_on(async {
            match self.store.delete(&location).await {
                Ok(()) | Err(object_store::Error::NotFound { .. }) => Ok(()),
                Err(error) => Err(classify(&error)),
            }
        })
    }
}

/// A folder that acts as the cloud (D5-Q2: the real account is not available yet). Tests and the
/// first runs of the app use it. It can be switched "offline" to test the retry queue.
#[derive(Debug, Clone)]
pub struct LocalFolderCloud {
    root: PathBuf,
    offline: Arc<AtomicBool>,
}

impl LocalFolderCloud {
    /// A stand-in whose objects are files under `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            offline: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Makes every call fail like a missing internet connection, or works again.
    pub fn set_offline(&self, offline: bool) {
        self.offline.store(offline, Ordering::SeqCst);
    }

    fn check_online(&self) -> Result<(), CloudError> {
        if self.offline.load(Ordering::SeqCst) {
            Err(CloudError::Offline("no internet".to_owned()))
        } else {
            Ok(())
        }
    }

    fn file(&self, key: &str) -> Result<PathBuf, CloudError> {
        if key.is_empty()
            || key.starts_with('/')
            || key.split('/').any(|part| part == ".." || part.is_empty())
        {
            return Err(CloudError::Refused("bad key".to_owned()));
        }
        Ok(self.root.join(key))
    }
}

impl CloudStore for LocalFolderCloud {
    fn put_file(&self, key: &str, source: &Path, hook: &dyn IoHook) -> Result<(), CloudError> {
        self.check_online()?;
        let dest = self.file(key)?;
        copy_atomic(source, &dest, hook, BackupKind::Cloud)
            .map(|_| ())
            .map_err(|e| CloudError::Offline(e.to_string()))
    }

    fn get_file(&self, key: &str, dest: &Path) -> Result<(), CloudError> {
        self.check_online()?;
        let source = self.file(key)?;
        if !source.exists() {
            return Err(CloudError::NotFound);
        }
        copy_atomic(&source, dest, &crate::io::NoHook, BackupKind::Local)
            .map(|_| ())
            .map_err(|e| CloudError::Offline(e.to_string()))
    }

    fn list(&self, prefix: &str) -> Result<Vec<CloudObject>, CloudError> {
        self.check_online()?;
        let dir = self.root.join(prefix.trim_end_matches('/'));
        let Ok(entries) = fs::read_dir(&dir) else {
            return Ok(Vec::new());
        };
        let mut objects = Vec::new();
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            let name = entry.file_name().to_string_lossy().into_owned();
            if !meta.is_file() || name.ends_with(".partial") {
                continue;
            }
            let modified = meta
                .modified()
                .map(DateTime::<Utc>::from)
                .unwrap_or_default();
            objects.push(CloudObject {
                key: format!("{}/{name}", prefix.trim_end_matches('/')),
                size: meta.len(),
                modified,
            });
        }
        objects.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(objects)
    }

    fn delete(&self, key: &str) -> Result<(), CloudError> {
        self.check_online()?;
        match fs::remove_file(self.file(key)?) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(CloudError::Offline(e.to_string())),
        }
    }
}

/// A factory that always returns the same store. For tests.
pub struct FixedCloud(pub Arc<dyn CloudStore>);

impl CloudFactory for FixedCloud {
    fn open(
        &self,
        settings: &BackupSettings,
        _secrets: &dyn SecretStore,
    ) -> Result<Arc<dyn CloudStore>, BackupError> {
        if settings.cloud_enabled {
            Ok(Arc::clone(&self.0))
        } else {
            Err(not_configured("the cloud copy is off"))
        }
    }

    fn ready(&self, settings: &BackupSettings, _secrets: &dyn SecretStore) -> bool {
        settings.cloud_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::NoHook;

    fn put_get_list_delete(cloud: &dyn CloudStore) {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.bin");
        fs::write(&src, vec![9_u8; 2 * CHUNK + 3]).unwrap();
        cloud.put_file("backups/one.mbk", &src, &NoHook).unwrap();
        cloud.put_file("backups/two.mbk", &src, &NoHook).unwrap();
        let listed = cloud.list("backups/").unwrap();
        assert_eq!(
            listed.iter().map(|o| o.key.as_str()).collect::<Vec<_>>(),
            ["backups/one.mbk", "backups/two.mbk"]
        );
        assert!(listed.iter().all(|o| o.size == 2 * CHUNK as u64 + 3));
        let back = dir.path().join("back.bin");
        cloud.get_file("backups/one.mbk", &back).unwrap();
        assert_eq!(fs::read(&back).unwrap(), fs::read(&src).unwrap());
        assert_eq!(
            cloud.get_file("backups/none.mbk", &back).unwrap_err(),
            CloudError::NotFound
        );
        cloud.delete("backups/one.mbk").unwrap();
        cloud.delete("backups/one.mbk").unwrap();
        assert_eq!(cloud.list("backups/").unwrap().len(), 1);
    }

    #[test]
    fn the_folder_stand_in_works_like_a_store() {
        let dir = tempfile::tempdir().unwrap();
        put_get_list_delete(&LocalFolderCloud::new(dir.path().join("cloud")));
    }

    #[test]
    fn the_object_store_adapter_works_like_a_store() {
        let cloud =
            ObjectStoreCloud::new(Arc::new(object_store::memory::InMemory::new()), "mandi/")
                .unwrap();
        put_get_list_delete(&cloud);
    }

    #[test]
    fn offline_fails_every_call_until_it_works_again() {
        let dir = tempfile::tempdir().unwrap();
        let cloud = LocalFolderCloud::new(dir.path().join("cloud"));
        let src = dir.path().join("a.bin");
        fs::write(&src, b"x").unwrap();
        cloud.set_offline(true);
        assert!(matches!(
            cloud.put_file("a/b.mbk", &src, &NoHook),
            Err(CloudError::Offline(_))
        ));
        assert!(matches!(cloud.list("a/"), Err(CloudError::Offline(_))));
        cloud.set_offline(false);
        cloud.put_file("a/b.mbk", &src, &NoHook).unwrap();
    }

    #[test]
    fn a_key_cannot_leave_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let cloud = LocalFolderCloud::new(dir.path().join("cloud"));
        let src = dir.path().join("a.bin");
        fs::write(&src, b"x").unwrap();
        for bad in ["../x", "/abs", "a//b", ""] {
            assert!(
                matches!(
                    cloud.put_file(bad, &src, &NoHook),
                    Err(CloudError::Refused(_))
                ),
                "{bad}"
            );
        }
    }

    #[test]
    fn s3_needs_keys_and_the_setting_on() {
        let secrets = crate::secrets::MemorySecretStore::default();
        let mut settings = BackupSettings::default();
        let error = S3Factory.open(&settings, &secrets).err().unwrap();
        assert_eq!(error.code, BackupCode::ErrBackupCloudNotConfigured);
        settings.cloud_enabled = true;
        let error = S3Factory.open(&settings, &secrets).err().unwrap();
        assert_eq!(error.code, BackupCode::ErrBackupCloudNotConfigured);
    }
}
