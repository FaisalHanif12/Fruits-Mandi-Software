//! A rig for the tests of "restore on a new computer" (plan 56): an old computer with the PRD
//! worked day and its cloud, and a NEW computer with an empty data file, an empty key store and
//! an empty credential store. The cloud is the real `object_store` adapter on top of memory.
//!
//! Never touches the real Keychain, the real data folder or any real key.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use mandi_backup::api::{
    BackupListInput, BackupRow, BackupSettingsInput, CloudAccountInput, FirstRunListInput,
    RestoreProgress, RestoreSource,
};
use mandi_backup::cloud::{CloudError, CloudObject, CloudStore, FixedCloud, ObjectStoreCloud};
use mandi_backup::daily_pack::PendingPack;
use mandi_backup::error::BackupError;
use mandi_backup::io::{IoHook, NoHook};
use mandi_backup::key::BackupKey;
use mandi_backup::model::{
    BackupKind, Embedded, InnerManifest, MANIFEST_FORMAT, Manifest, SecretText,
};
use mandi_backup::names::{bundle_name, manifest_name};
use mandi_backup::pack::{open, seal};
use mandi_backup::secrets::{MemorySecretStore, SecretName, SecretStore};
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_backup::settings::BackupSettings;
use mandi_backup::snapshot::inspect;
use mandi_db::clock::ManualClock;
use mandi_db::disk::DiskProbe;
use mandi_db::{Db, DbOptions};
use mandi_services::auth;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::auth::PrefKey;
use mandi_services::core::translit::Kind;
use mandi_services::db::audit::{AuditKey, KeyStore};
use mandi_services::db::rusqlite::{Connection, OpenFlags, types::ValueRef};
use mandi_services::testing::{CountingRandom, ManualMonotonic, fast_hasher, test_start};
use mandi_services::translit::{self, NewPair};
use mandi_services::{Context, Parts, ServiceError, settings};
use tempfile::TempDir;

use crate::harness::{Rig, worked_day_with};
use crate::{common, support};

/// The key every old test computer uses for its hash chains (`TestEnv`).
pub fn old_chain_key() -> AuditKey {
    AuditKey::from_bytes([7; 32])
}

/// The cloud: the real adapter, on memory.
pub fn memory_cloud() -> Arc<ObjectStoreCloud> {
    Arc::new(
        ObjectStoreCloud::new(Arc::new(object_store::memory::InMemory::new()), "mandi/").unwrap(),
    )
}

/// A cloud that can be told to fail, to test the words for a wrong key, bucket or internet.
pub struct Flaky {
    pub inner: Arc<dyn CloudStore>,
    pub fail: Mutex<Option<CloudError>>,
}

impl Flaky {
    pub fn new(inner: Arc<dyn CloudStore>) -> Arc<Self> {
        Arc::new(Self {
            inner,
            fail: Mutex::new(None),
        })
    }

    pub fn fail_with(&self, error: Option<CloudError>) {
        *self.fail.lock().unwrap_or_else(PoisonError::into_inner) = error;
    }

    fn check(&self) -> Result<(), CloudError> {
        match self
            .fail
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
        {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl CloudStore for Flaky {
    fn put_file(&self, key: &str, source: &Path, hook: &dyn IoHook) -> Result<(), CloudError> {
        self.check()?;
        self.inner.put_file(key, source, hook)
    }
    fn get_file(&self, key: &str, dest: &Path) -> Result<(), CloudError> {
        self.check()?;
        self.inner.get_file(key, dest)
    }
    fn get_file_progress(
        &self,
        key: &str,
        dest: &Path,
        progress: &mut dyn FnMut(u64, u64),
    ) -> Result<(), CloudError> {
        self.check()?;
        self.inner.get_file_progress(key, dest, progress)
    }
    fn list(&self, prefix: &str) -> Result<Vec<CloudObject>, CloudError> {
        self.check()?;
        self.inner.list(prefix)
    }
    fn delete(&self, key: &str) -> Result<(), CloudError> {
        self.check()?;
        self.inner.delete(key)
    }
}

/// The old shop: the worked day, a learned word, a preference, a cloud, a USB folder, the
/// backup key (printed once) and the recovery code (printed once).
pub struct Old {
    pub rig: Rig,
    pub service: Arc<BackupService>,
    pub cloud: Arc<ObjectStoreCloud>,
    pub secrets: Arc<MemorySecretStore>,
    pub printed_key: String,
    pub recovery_code: String,
    pub local_dir: PathBuf,
}

pub fn old_computer() -> Old {
    let (dir, env) = common::env();
    let recovery_code = common::set_up(&env);
    common::login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = support::add_account(&env, &owner, AccountType::Seller, "رحیم", "Rahim", true, "");
    let w = support::finish_world(dir, env, owner, rahim, true);
    support::set_cash(&w, 50_000);
    let rig = worked_day_with(w);
    let ctx = &rig.w.env.ctx;
    let owner = &rig.w.owner;
    translit::learn(
        ctx,
        owner,
        &NewPair {
            roman: "arshad".into(),
            urdu: "ارشد".into(),
            kind: Kind::Name,
        },
    )
    .unwrap();
    settings::set_pref(ctx, owner, PrefKey::TextSize, "120").unwrap();

    let cloud = memory_cloud();
    let secrets = Arc::new(MemorySecretStore::default());
    let service = Arc::new(BackupService::new(
        Arc::clone(ctx.db()),
        BackupConfig {
            backups_dir: rig.backups.clone(),
            app_version: "0.0.0-test".to_owned(),
        },
        BackupParts {
            secrets: secrets.clone(),
            cloud: Arc::new(FixedCloud(cloud.clone())),
            disk: rig.disk.clone(),
            hook: rig.fault.clone(),
            pack: Arc::new(PendingPack),
        },
    ));
    let local_dir = rig.root.path().join("old-laptop-local");
    service
        .settings_set(
            Some(owner.user_id),
            &BackupSettingsInput {
                settings: BackupSettings {
                    local_dir: local_dir.display().to_string(),
                    usb_enabled: true,
                    usb_folder: rig.usb.display().to_string(),
                    cloud_enabled: true,
                    cloud_endpoint: "https://old.example.invalid".into(),
                    cloud_bucket: "old-bucket".into(),
                    ..BackupSettings::default()
                },
                cloud_access_key_id: Some("AKIAOLDKEYID".into()),
                cloud_secret_access_key: Some("old-secret-value-1".into()),
            },
        )
        .unwrap();
    let printed_key = service.key_setup(Some(owner.user_id)).unwrap().key;
    Old {
        rig,
        service,
        cloud,
        secrets,
        printed_key,
        recovery_code,
        local_dir,
    }
}

impl Old {
    /// Makes a copy to every place and returns the name of the cloud copy.
    pub fn back_up(&self) -> String {
        let report = self
            .service
            .run(mandi_backup::run::Scope::ALL)
            .expect("backup run");
        assert!(report.results.iter().all(|r| r.ok), "{report:?}");
        self.latest(BackupKind::Cloud)
    }

    /// The newest copy of a place.
    pub fn latest(&self, kind: BackupKind) -> String {
        self.service
            .list(&BackupListInput {
                kind: Some(kind),
                offset: 0,
                limit: 1,
            })
            .unwrap()
            .rows
            .first()
            .unwrap_or_else(|| panic!("no {kind:?} copy"))
            .name
            .clone()
    }

    /// The plain database inside a cloud copy, in a folder the caller keeps alive.
    pub fn unpack_cloud(&self, name: &str) -> (TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join(name);
        self.cloud
            .get_file(&format!("backups/{name}"), &bundle)
            .unwrap();
        let plain = dir.path().join("copy.db");
        open(
            &bundle,
            &BackupKey::parse(&self.printed_key).unwrap(),
            &plain,
        )
        .unwrap();
        (dir, plain)
    }

    pub fn account(&self) -> CloudAccountInput {
        CloudAccountInput {
            endpoint: "https://new.example.invalid".into(),
            bucket: "new-bucket".into(),
            region: "auto".into(),
            prefix: "mandi".into(),
            key_id: "AKIANEWKEYID".into(),
            secret: "new-secret-value-2".into(),
        }
    }
}

/// Plenty of disk, or none, for the data file of the new computer.
#[derive(Debug)]
pub struct Plenty(AtomicU64);

impl Plenty {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(AtomicU64::new(10 * 1024 * 1024 * 1024)))
    }

    pub fn set(&self, bytes: u64) {
        self.0.store(bytes, Ordering::SeqCst);
    }
}

impl DiskProbe for Plenty {
    fn available_bytes(&self, _: &Path) -> io::Result<u64> {
        Ok(self.0.load(Ordering::SeqCst))
    }
}

/// A key store that starts EMPTY and makes a random key the first time one is asked for, like the
/// Credential Manager of a new computer does.
#[derive(Debug, Default)]
pub struct EmptyKeyStore(Mutex<Option<AuditKey>>);

impl EmptyKeyStore {
    pub fn current(&self) -> Option<AuditKey> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl KeyStore for EmptyKeyStore {
    fn audit_key(&self) -> Result<AuditKey, mandi_db::DbError> {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(key) = held.as_ref() {
            return Ok(key.clone());
        }
        let key = AuditKey::random()?;
        *held = Some(key.clone());
        Ok(key)
    }

    fn set_audit_key(&self, key: &AuditKey) -> Result<(), mandi_db::DbError> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(key.clone());
        Ok(())
    }
}

/// A credential store that refuses to store anything after `allow` writes (a locked Keychain).
#[derive(Debug)]
pub struct FlakySecrets {
    pub inner: MemorySecretStore,
    allow: AtomicUsize,
    sets: AtomicUsize,
}

impl FlakySecrets {
    pub fn new(allow: usize) -> Arc<Self> {
        Arc::new(Self {
            inner: MemorySecretStore::default(),
            allow: AtomicUsize::new(allow),
            sets: AtomicUsize::new(0),
        })
    }

    pub fn peek(&self, name: SecretName) -> Option<String> {
        self.inner.get(name).unwrap()
    }
}

impl SecretStore for FlakySecrets {
    fn get(&self, name: SecretName) -> Result<Option<String>, BackupError> {
        self.inner.get(name)
    }

    fn set(&self, name: SecretName, value: &str) -> Result<(), BackupError> {
        if self.sets.fetch_add(1, Ordering::SeqCst) >= self.allow.load(Ordering::SeqCst) {
            return Err(BackupError::internal("the credential store is locked"));
        }
        self.inner.set(name, value)
    }

    fn delete(&self, name: SecretName) -> Result<(), BackupError> {
        self.inner.delete(name)
    }
}

/// The new computer: an empty data file, an empty key store, an empty credential store.
pub struct New {
    pub root: TempDir,
    pub ctx: Arc<Context>,
    pub service: Arc<BackupService>,
    pub secrets: Arc<FlakySecrets>,
    pub keys: Arc<EmptyKeyStore>,
    pub data_disk: Arc<Plenty>,
    /// The free space the backup service sees (scratch files).
    pub service_disk: Arc<Plenty>,
}

pub fn new_computer(cloud: Arc<dyn CloudStore>) -> New {
    new_computer_with(cloud, FlakySecrets::new(usize::MAX))
}

pub fn new_computer_with(cloud: Arc<dyn CloudStore>, secrets: Arc<FlakySecrets>) -> New {
    let root = tempfile::tempdir().unwrap();
    let keys = Arc::new(EmptyKeyStore::default());
    let data_disk = Plenty::new();
    let clock = Arc::new(ManualClock::new(test_start() + chrono::Duration::days(30)));
    let options = DbOptions {
        backups_dir: root.path().join("backups"),
        disk_probe: data_disk.clone(),
        clock: clock.clone(),
        key_store: keys.clone(),
        busy_timeout: Duration::from_millis(500),
        check_wait: Duration::from_millis(500),
    };
    let db = Db::open_checked(&root.path().join("data").join("mandi.db"), options).unwrap();
    let ctx = Arc::new(Context::with_parts(
        Arc::new(db),
        Parts {
            monotonic: Arc::new(ManualMonotonic::default()),
            hasher: fast_hasher().unwrap(),
            random: Arc::new(CountingRandom::default()),
        },
    ));
    let service_disk = Plenty::new();
    let service = Arc::new(BackupService::new(
        Arc::clone(ctx.db()),
        BackupConfig {
            backups_dir: root.path().join("backups"),
            app_version: "0.0.0-test".to_owned(),
        },
        BackupParts {
            secrets: secrets.clone(),
            cloud: Arc::new(FixedCloud(cloud)),
            disk: service_disk.clone(),
            hook: Arc::new(NoHook),
            pack: Arc::new(PendingPack),
        },
    ));
    New {
        root,
        ctx,
        service,
        secrets,
        keys,
        data_disk,
        service_disk,
    }
}

impl New {
    pub fn db_path(&self) -> PathBuf {
        self.root.path().join("data").join("mandi.db")
    }

    pub fn is_fresh(&self) -> bool {
        auth::status(&self.ctx).unwrap().first_run
    }

    /// Everything a failed restore must leave exactly as it was.
    pub fn fingerprint(&self) -> BTreeMap<String, (i64, String)> {
        table_hashes(&self.db_path(), &[])
    }

    /// Every file under the root, with its bytes.
    pub fn all_files(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut found = Vec::new();
        files_under(self.root.path(), &mut found);
        found
    }
}

pub fn files_under(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_under(&path, out);
        } else if let Ok(bytes) = fs::read(&path) {
            out.push((path, bytes));
        }
    }
}

pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// The cloud source of a typed account.
pub fn cloud_source(old: &Old) -> RestoreSource {
    RestoreSource::Cloud(old.account())
}

pub fn list_input(source: RestoreSource) -> FirstRunListInput {
    FirstRunListInput { source }
}

/// Collects the progress events of a call.
#[derive(Default)]
pub struct Events(Mutex<Vec<RestoreProgress>>);

impl Events {
    pub fn sink(&self) -> impl Fn(RestoreProgress) + '_ {
        |p| {
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(p)
        }
    }

    pub fn all(&self) -> Vec<RestoreProgress> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Count and a checksum of the rows of every table, in id order. `skip` lists tables to leave
/// out. Two files with the same answer hold the same data.
pub fn table_hashes(path: &Path, skip: &[&str]) -> BTreeMap<String, (i64, String)> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    names
        .into_iter()
        .filter(|n| !skip.contains(&n.as_str()))
        .map(|name| {
            let hashed = hash_rows(&conn, &name, "");
            (name, hashed)
        })
        .collect()
}

/// The hash of one table, optionally with a WHERE clause (written by the test, never by input).
pub fn hash_rows(conn: &Connection, table: &str, where_clause: &str) -> (i64, String) {
    let sql = format!("SELECT * FROM \"{table}\" {where_clause} ORDER BY rowid");
    let mut statement = conn
        .prepare(&sql)
        .or_else(|_| conn.prepare(&format!("SELECT * FROM \"{table}\" {where_clause}")))
        .unwrap();
    let columns = statement.column_count();
    let mut rows = statement.query([]).unwrap();
    let mut hasher = blake3::Hasher::new();
    let mut count = 0_i64;
    while let Some(row) = rows.next().unwrap() {
        count += 1;
        for index in 0..columns {
            match row.get_ref(index).unwrap() {
                ValueRef::Null => hasher.update(b"N;"),
                ValueRef::Integer(i) => hasher.update(format!("I{i};").as_bytes()),
                ValueRef::Real(f) => hasher.update(format!("R{f};").as_bytes()),
                ValueRef::Text(t) => hasher.update(&[b"T", t, b";"].concat()),
                ValueRef::Blob(b) => hasher.update(&[b"B", b, b";"].concat()),
            };
        }
        hasher.update(b"\n");
    }
    (count, hasher.finalize().to_hex().to_string())
}

/// Puts a copy made by hand into the cloud: the bundle, and a manifest next to it. `chain_key`
/// None makes a copy like the ones made before the key travelled.
pub fn put_crafted_copy(
    cloud: &ObjectStoreCloud,
    key: &BackupKey,
    plain: &Path,
    stamp_text: &str,
    chain_key: Option<&AuditKey>,
    with_manifest: bool,
) -> String {
    let inspected = inspect(plain).unwrap();
    let created_at = "2026-10-11T09:00:00.000Z".to_owned();
    let embedded = Embedded {
        format: MANIFEST_FORMAT,
        created_at: created_at.clone(),
        app_version: "0.0.0-test".into(),
        schema_version: inspected.schema_version,
        inner: InnerManifest {
            plain_blake3: inspected.blake3.clone(),
            plain_size: inspected.size_bytes,
            counts: inspected.counts.clone(),
            totals: inspected.totals.clone(),
        },
        chain_key: chain_key.map(|k| SecretText::new(k.to_hex())),
    };
    let dir = tempfile::tempdir().unwrap();
    let name = bundle_name(stamp_text);
    let bundle = dir.path().join(&name);
    let sealed = seal(plain, &embedded, key, &bundle, &NoHook, BackupKind::Usb).unwrap();
    cloud
        .put_file(&format!("backups/{name}"), &bundle, &NoHook)
        .unwrap();
    if with_manifest {
        let manifest = Manifest {
            format: MANIFEST_FORMAT,
            kind: BackupKind::Cloud,
            created_at,
            app_version: "0.0.0-test".into(),
            schema_version: inspected.schema_version,
            encrypted: true,
            file: name.clone(),
            size_bytes: sealed.bundle_size,
            file_blake3: sealed.bundle_blake3,
            integrity_ok: true,
            inner: None,
        };
        let side = dir.path().join(manifest_name(&name));
        fs::write(&side, serde_json::to_vec(&manifest).unwrap()).unwrap();
        cloud
            .put_file(&format!("backups/{}", manifest_name(&name)), &side, &NoHook)
            .unwrap();
    }
    name
}

/// The codes and fields of an error, all together, for an easy comparison.
pub fn codes(error: &BackupError) -> Vec<(String, String)> {
    let mut items: Vec<(String, String)> = clone_all(error)
        .into_iter()
        .map(|e| {
            (
                serde_json::to_string(&e.code).unwrap().replace('"', ""),
                e.field.unwrap_or_default(),
            )
        })
        .collect();
    items.sort();
    items
}

fn clone_all(error: &BackupError) -> Vec<BackupError> {
    let mut items = vec![BackupError {
        code: error.code,
        field: error.field.clone(),
        values: error.values.clone(),
        detail: error.detail.clone(),
        also: Vec::new(),
    }];
    items.extend(error.also.iter().flat_map(clone_all));
    items
}

/// The value `reason` or `check` of an error.
pub fn value_of(error: &BackupError, name: &str) -> Option<String> {
    match error.values.get(name) {
        Some(mandi_core::error::ErrorValue::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

pub fn row_names(rows: &[BackupRow]) -> Vec<String> {
    rows.iter().map(|r| r.name.clone()).collect()
}

pub fn service_error(error: &ServiceError) -> String {
    format!("{error:?}")
}
