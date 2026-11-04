//! Robustness tests for "restore on a new computer" (plan 56, PRD FR-20.11, NFR-INS-6,
//! NFR-REL-4, NFR-REL-10, NFR-BAK-12, NFR-SEC-7). Attack tests, written by `robustness`.
//!
//! The question of every test is the same: after the attack, is the new computer still a fresh
//! install (no Owner, no stored key, no scratch file, the data file unchanged), and did the app
//! answer with a clear code instead of a crash, a hang or a lie?
//!
//! Groups:
//!  1. Hostile bundles that open with the RIGHT key (bad payload, lying manifest, bad lengths,
//!     bad chain key, a copy with no Owner, a newer schema).
//!  2. A real bundle cut or flipped at many places.
//!  3. A bundle whose header asks for a huge scrypt work factor (cost without the key).
//!  4. A decompression bomb and a nearly full disk.
//!  5. Path tricks: a symlink to a secret file, special files, junk in a folder, copy names,
//!     paths, keys and cloud fields.
//!  6. Parallel calls: the busy lock, six restores at once, a shop set up during a restore.
//!  7. The preview called many times.
//!  8. Time against NFR-BAK-12 (restore under 5 minutes) and NFR-REL-4 (whole flow under 30
//!     minutes). The big runs are `#[ignore]`: `cargo test -p mandi-backup --test
//!     robustness_restore_new_computer -- --ignored --nocapture`.
//!
//! No real key, no real Keychain, no real data folder and no network are used.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
mod new_computer;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use age::secrecy::SecretString;
use mandi_backup::api::{
    FirstRunListInput, FirstRunPreviewInput, FirstRunRestoreInput, RestoreProgress, RestoreSource,
};
use mandi_backup::cloud::{CloudError, CloudObject, CloudStore, FixedCloud};
use mandi_backup::daily_pack::PendingPack;
use mandi_backup::error::{BackupCode, BackupError};
use mandi_backup::io::{IoHook, NoHook};
use mandi_backup::model::{Embedded, InnerManifest, MANIFEST_FORMAT, SecretText};
use mandi_backup::secrets::SecretName;
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_backup::snapshot::{inspect, read_totals};
use mandi_services::auth;
use mandi_services::db::audit::{AuditKey, verify_audit_chain};
use mandi_services::db::rusqlite::{Connection, OpenFlags};
use mandi_services::ledger;
use new_computer::*;

type Snapshot = BTreeMap<String, (i64, String)>;

const MAGIC: &[u8; 8] = b"MANDIBK1";
/// The longest manifest the app reads (`pack.rs`).
const MANIFEST_LIMIT: usize = 1024 * 1024;

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

fn preview_input(source: RestoreSource, copy: Option<&str>, key: &str) -> FirstRunPreviewInput {
    FirstRunPreviewInput {
        source,
        copy: copy.map(str::to_owned),
        key: key.to_owned(),
    }
}

fn restore_input(source: RestoreSource, copy: &str, key: &str) -> FirstRunRestoreInput {
    FirstRunRestoreInput {
        source,
        copy: copy.to_owned(),
        key: key.to_owned(),
    }
}

fn file_source(path: &Path) -> RestoreSource {
    RestoreSource::File(path.display().to_string())
}

fn folder_source(path: &Path) -> RestoreSource {
    RestoreSource::Folder(path.display().to_string())
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_owned()
}

/// What a failed or refused call must leave exactly as it was: the computer is still a fresh
/// install. Returns what is wrong, empty when all is right.
fn fresh_problems(new: &New, before: &Snapshot) -> Vec<String> {
    let mut problems = Vec::new();
    if !new.is_fresh() {
        problems.push("an Owner exists".to_owned());
    }
    if &new.fingerprint() != before {
        problems.push("the data file is not the fresh one".to_owned());
    }
    for name in [
        SecretName::BackupKey,
        SecretName::CloudAccessKeyId,
        SecretName::CloudSecretAccessKey,
    ] {
        if new.secrets.peek(name).is_some() {
            problems.push(format!("{name:?} was kept"));
        }
    }
    if let Some(key) = new.keys.current()
        && key.same_as(&old_chain_key())
    {
        problems.push("the old chain key was kept".to_owned());
    }
    let tmp = new.root.path().join("backups").join("tmp");
    let left = fs::read_dir(tmp).map_or(0, Iterator::count);
    if left != 0 {
        problems.push(format!("{left} scratch entries left"));
    }
    problems
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Call {
    Preview,
    Restore,
}

fn call_name(call: Call) -> &'static str {
    match call {
        Call::Preview => "preview",
        Call::Restore => "restore",
    }
}

/// One call of the service, with a panic turned into a text.
fn run_call(
    new: &New,
    call: Call,
    source: &RestoreSource,
    copy: Option<&str>,
    key: &str,
) -> Result<Result<(), BackupError>, String> {
    panic::catch_unwind(AssertUnwindSafe(|| match call {
        Call::Preview => new
            .service
            .first_run_preview(&preview_input(source.clone(), copy, key), &|_| {})
            .map(|_| ()),
        Call::Restore => new
            .service
            .first_run_restore(
                &restore_input(source.clone(), copy.unwrap_or("x.mbk"), key),
                &|_| {},
            )
            .map(|_| ()),
    }))
    .map_err(|_| "PANICKED".to_owned())
}

/// Attacks with `calls`; every call must be refused with one of `allowed` (any code when empty),
/// must not panic, and must leave the computer a fresh install.
fn attack(
    (new, before): (&New, &Snapshot),
    label: &str,
    source: &RestoreSource,
    copy: Option<&str>,
    key: &str,
    calls: &[Call],
    allowed: &[BackupCode],
) -> Vec<String> {
    let mut problems = Vec::new();
    for call in calls {
        let what = call_name(*call);
        match run_call(new, *call, source, copy, key) {
            Err(text) => problems.push(format!("{label}: {what} {text}")),
            Ok(Ok(())) => problems.push(format!("{label}: {what} was ACCEPTED")),
            Ok(Err(error)) => {
                if !allowed.is_empty() && !allowed.contains(&error.code) {
                    problems.push(format!(
                        "{label}: {what} gave {:?} (field {:?}), expected one of {allowed:?}",
                        error.code, error.field
                    ));
                }
            }
        }
        problems.extend(
            fresh_problems(new, before)
                .into_iter()
                .map(|p| format!("{label}: after {what}: {p}")),
        );
    }
    problems
}

const BOTH: &[Call] = &[Call::Preview, Call::Restore];

/// One rival call made while the first restore is held at its download.
type Rival<'a> = Box<dyn Fn() -> Result<(), BackupError> + 'a>;

fn assert_no_problems(problems: &[String]) {
    assert!(
        problems.is_empty(),
        "{} problem(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

/// The old shop with its worked day, one cloud copy opened, and a NEW computer. `work` is a
/// folder for the hostile files.
struct Fixture {
    old: Old,
    new: New,
    before: Snapshot,
    work: tempfile::TempDir,
    _plain_dir: tempfile::TempDir,
    plain: PathBuf,
    base: Embedded,
    key: String,
    cloud_name: String,
}

fn fixture() -> Fixture {
    let old = old_computer();
    let cloud_name = old.back_up();
    let (plain_dir, plain) = old.unpack_cloud(&cloud_name);
    let base = embedded_for(&plain, Some(&old_chain_key()));
    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();
    let key = old.printed_key.clone();
    Fixture {
        old,
        new,
        before,
        work: tempfile::tempdir().unwrap(),
        _plain_dir: plain_dir,
        plain,
        base,
        key,
        cloud_name,
    }
}

fn embedded_for(plain: &Path, chain_key: Option<&AuditKey>) -> Embedded {
    let found = inspect(plain).unwrap();
    Embedded {
        format: MANIFEST_FORMAT,
        created_at: "2026-10-11T09:00:00.000Z".to_owned(),
        app_version: "0.0.0-test".to_owned(),
        schema_version: found.schema_version,
        inner: InnerManifest {
            plain_blake3: found.blake3,
            plain_size: found.size_bytes,
            counts: found.counts,
            totals: found.totals,
        },
        chain_key: chain_key.map(|k| SecretText::new(k.to_hex())),
    }
}

fn pass_of(printed: &str) -> SecretString {
    SecretString::from(printed.chars().filter(|c| *c != '-').collect::<String>())
}

/// Writes `age(scrypt) + zstd` of whatever `body` writes. The same envelope the app makes, with
/// a work factor of our choice, so hostile contents can be put inside a real envelope.
fn write_envelope(
    path: &Path,
    printed_key: &str,
    work_factor: u8,
    body: &mut dyn FnMut(&mut dyn Write) -> io::Result<()>,
) {
    let mut recipient = age::scrypt::Recipient::new(pass_of(printed_key));
    recipient.set_work_factor(work_factor);
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
            .unwrap();
    let file = File::create(path).unwrap();
    let locked = encryptor.wrap_output(file).unwrap();
    let mut packed = zstd::stream::write::Encoder::new(locked, 3).unwrap();
    body(&mut packed).unwrap();
    let locked = packed.finish().unwrap();
    locked.finish().unwrap();
}

fn len_field(length: usize) -> [u8; 4] {
    u32::try_from(length).unwrap().to_le_bytes()
}

impl Fixture {
    fn path(&self, label: &str) -> PathBuf {
        self.work.path().join(format!("mandi-hostile-{label}.mbk"))
    }

    /// A well formed bundle (work factor 10: quick) of `payload` with the manifest `embedded`.
    fn bundle(&self, label: &str, embedded: &Embedded, payload: &Path) -> PathBuf {
        let path = self.path(label);
        let manifest = serde_json::to_vec(embedded).unwrap();
        write_envelope(&path, &self.key, 10, &mut |w| {
            w.write_all(MAGIC)?;
            w.write_all(&len_field(manifest.len()))?;
            w.write_all(&manifest)?;
            io::copy(&mut File::open(payload)?, w)?;
            Ok(())
        });
        path
    }

    /// A well formed bundle of payload BYTES.
    fn bundle_bytes(&self, label: &str, embedded: &Embedded, payload: &[u8]) -> PathBuf {
        let path = self.path(label);
        let manifest = serde_json::to_vec(embedded).unwrap();
        write_envelope(&path, &self.key, 10, &mut |w| {
            w.write_all(MAGIC)?;
            w.write_all(&len_field(manifest.len()))?;
            w.write_all(&manifest)?;
            w.write_all(payload)?;
            Ok(())
        });
        path
    }

    /// A bundle whose stream is exactly `parts` joined (to put a wrong magic, a wrong length or
    /// a cut header in a real envelope).
    fn bundle_parts(&self, label: &str, parts: &[&[u8]]) -> PathBuf {
        let path = self.path(label);
        write_envelope(&path, &self.key, 10, &mut |w| {
            for part in parts {
                w.write_all(part)?;
            }
            Ok(())
        });
        path
    }

    /// A copy of the good plain database in a new file, changed by `change`.
    fn changed_plain(&self, label: &str, change: &str) -> PathBuf {
        let path = self.work.path().join(format!("plain-{label}.db"));
        fs::copy(&self.plain, &path).unwrap();
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA journal_mode = DELETE;").unwrap();
        conn.execute_batch(change).unwrap();
        drop(conn);
        path
    }

    fn embedded(&self) -> Embedded {
        self.base.clone()
    }
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// The total of the file sizes under a folder.
fn dir_bytes(dir: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| {
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => dir_bytes(&path),
                Ok(kind) if kind.is_file() => fs::metadata(&path).map_or(0, |m| m.len()),
                _ => 0,
            }
        })
        .sum()
}

/// Watches a folder in the background and remembers the most bytes it ever held.
struct Peak {
    stop: Arc<AtomicBool>,
    handle: thread::JoinHandle<u64>,
}

fn watch(dir: PathBuf) -> Peak {
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        let mut peak = 0_u64;
        while !flag.load(Ordering::SeqCst) {
            peak = peak.max(dir_bytes(&dir));
            thread::sleep(Duration::from_millis(1));
        }
        peak.max(dir_bytes(&dir))
    });
    Peak { stop, handle }
}

impl Peak {
    fn finish(self) -> u64 {
        self.stop.store(true, Ordering::SeqCst);
        self.handle.join().unwrap()
    }
}

/// A cloud that counts downloads and can hold one download until the test lets it go.
struct Gated {
    inner: Arc<dyn CloudStore>,
    downloads: AtomicUsize,
    gate_on: AtomicBool,
    entered: Mutex<Option<mpsc::Sender<()>>>,
    release: Mutex<mpsc::Receiver<()>>,
}

impl Gated {
    fn new(inner: Arc<dyn CloudStore>) -> (Arc<Self>, mpsc::Receiver<()>, mpsc::Sender<()>) {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let gated = Arc::new(Self {
            inner,
            downloads: AtomicUsize::new(0),
            gate_on: AtomicBool::new(false),
            entered: Mutex::new(Some(entered_tx)),
            release: Mutex::new(release_rx),
        });
        (gated, entered_rx, release_tx)
    }

    fn hold(&self) {
        if !self.gate_on.load(Ordering::SeqCst) {
            return;
        }
        if let Some(tx) = self
            .entered
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
        {
            let _ = tx.send(());
        }
        let _ = self
            .release
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .recv_timeout(Duration::from_secs(120));
    }
}

impl CloudStore for Gated {
    fn put_file(&self, key: &str, source: &Path, hook: &dyn IoHook) -> Result<(), CloudError> {
        self.inner.put_file(key, source, hook)
    }
    fn get_file(&self, key: &str, dest: &Path) -> Result<(), CloudError> {
        self.inner.get_file(key, dest)
    }
    fn get_file_progress(
        &self,
        key: &str,
        dest: &Path,
        progress: &mut dyn FnMut(u64, u64),
    ) -> Result<(), CloudError> {
        if key.ends_with(".mbk") {
            self.downloads.fetch_add(1, Ordering::SeqCst);
            self.hold();
        }
        self.inner.get_file_progress(key, dest, progress)
    }
    fn list(&self, prefix: &str) -> Result<Vec<CloudObject>, CloudError> {
        self.inner.list(prefix)
    }
    fn delete(&self, key: &str) -> Result<(), CloudError> {
        self.inner.delete(key)
    }
}

/// A new computer whose backup service sees the disk free space that the test sets.
fn new_computer_with_disk(cloud: Arc<dyn CloudStore>) -> (New, Arc<Plenty>) {
    let mut new = new_computer(Arc::clone(&cloud));
    let disk = Plenty::new();
    new.service = Arc::new(BackupService::new(
        Arc::clone(new.ctx.db()),
        BackupConfig {
            backups_dir: new.root.path().join("backups"),
            app_version: "0.0.0-test".to_owned(),
        },
        BackupParts {
            secrets: new.secrets.clone(),
            cloud: Arc::new(FixedCloud(cloud)),
            disk: disk.clone(),
            hook: Arc::new(NoHook),
            pack: Arc::new(PendingPack),
        },
    ));
    (new, disk)
}

/// Both hash chains and the books of the new computer are those of a healthy shop.
fn assert_books_and_chains(new: &New) {
    ledger::verify_journal_chain(&new.ctx).expect("the journal chain verifies");
    let key = new
        .keys
        .current()
        .expect("the new computer has a chain key");
    new.ctx
        .db()
        .read(|conn| verify_audit_chain(conn, &key).map_err(|e| e.to_string()))
        .expect("the audit chain verifies");
    let totals = new
        .ctx
        .db()
        .read(read_totals)
        .expect("the books can be read");
    assert_eq!(totals.debit_paisa, totals.credit_paisa, "the books balance");
}

// ---------------------------------------------------------------------------------------------
// 1. Hostile bundles that open with the right key
// ---------------------------------------------------------------------------------------------

#[test]
fn hostile_bundles_with_the_right_key_are_refused_and_change_nothing() {
    let f = fixture();
    let mut cases: Vec<(String, PathBuf, Vec<BackupCode>)> = Vec::new();
    let corrupt = vec![BackupCode::ErrBackupCorrupt];
    let any: Vec<BackupCode> = Vec::new();
    let mut add = |label: &str, path: PathBuf, allowed: &[BackupCode]| {
        cases.push((label.to_owned(), path, allowed.to_vec()));
    };

    // -- the payload is not the database the manifest talks about
    let junk = b"This is not a database. ".repeat(200);
    let mut e = f.embedded();
    e.inner.plain_blake3 = blake3_hex(&junk);
    e.inner.plain_size = junk.len() as u64;
    add(
        "not_sqlite",
        f.bundle_bytes("not_sqlite", &e, &junk),
        &corrupt,
    );

    let mut e = f.embedded();
    e.inner.plain_blake3 = blake3_hex(b"");
    e.inner.plain_size = 0;
    add(
        "empty_payload",
        f.bundle_bytes("empty_payload", &e, b""),
        &corrupt,
    );

    // The right database with 5 bytes of junk after it, the manifest unchanged.
    let mut good = fs::read(&f.plain).unwrap();
    good.extend_from_slice(b"JUNK!");
    add(
        "bytes_after_the_database",
        f.bundle_bytes("bytes_after", &f.embedded(), &good),
        &corrupt,
    );

    // The right database with one byte missing at the end.
    let mut short = fs::read(&f.plain).unwrap();
    short.pop();
    add(
        "one_byte_short",
        f.bundle_bytes("one_byte_short", &f.embedded(), &short),
        &corrupt,
    );

    // -- a manifest that lies
    let mut e = f.embedded();
    e.inner.plain_blake3 = "0".repeat(64);
    add(
        "lying_checksum",
        f.bundle("lying_checksum", &e, &f.plain),
        &corrupt,
    );
    for (label, size) in [
        ("size_zero", 0_u64),
        ("size_small", 1),
        ("size_max", u64::MAX),
    ] {
        let mut e = f.embedded();
        e.inner.plain_size = size;
        add(label, f.bundle(label, &e, &f.plain), &corrupt);
    }
    let mut e = f.embedded();
    e.inner.counts.insert("accounts".to_owned(), 99_999);
    add(
        "lying_counts",
        f.bundle("lying_counts", &e, &f.plain),
        &corrupt,
    );
    let mut e = f.embedded();
    e.inner.counts.insert("no_such_table".to_owned(), 1);
    add(
        "invented_table_in_counts",
        f.bundle("invented_table", &e, &f.plain),
        &corrupt,
    );
    let mut e = f.embedded();
    e.schema_version = 9999;
    add(
        "manifest_says_schema_9999",
        f.bundle("manifest_schema_up", &e, &f.plain),
        &any,
    );
    let mut e = f.embedded();
    e.schema_version = -1;
    add(
        "manifest_says_schema_minus_1",
        f.bundle("manifest_schema_neg", &e, &f.plain),
        &any,
    );

    // -- the database itself is newer than the app, or empty, or has no Owner
    let future = f.changed_plain("future", "PRAGMA user_version = 9999;");
    add(
        "database_schema_9999",
        f.bundle(
            "db_schema_up",
            &embedded_for(&future, Some(&old_chain_key())),
            &future,
        ),
        &[BackupCode::ErrBackupNewerThanApp],
    );
    let empty_db = f.work.path().join("plain-empty.db");
    {
        let conn = Connection::open(&empty_db).unwrap();
        conn.execute_batch("CREATE TABLE junk(x); DROP TABLE junk;")
            .unwrap();
    }
    let mut e = f.embedded();
    e.schema_version = 0;
    e.inner.counts = BTreeMap::new();
    e.inner.plain_size = fs::metadata(&empty_db).unwrap().len();
    e.inner.plain_blake3 = blake3_hex(&fs::read(&empty_db).unwrap());
    add(
        "database_with_no_tables",
        f.bundle("no_tables", &e, &empty_db),
        &any,
    );
    // -- a copy of another shop: its chain key is not ours, or its backup key is not ours
    let other_chain = AuditKey::random().unwrap();
    let mut e = f.embedded();
    e.chain_key = Some(SecretText::new(other_chain.to_hex()));
    add(
        "chain_key_of_another_shop",
        f.bundle("other_chain_key", &e, &f.plain),
        &corrupt,
    );
    let other_key = mandi_backup::key::BackupKey::generate().unwrap().display();
    let other_shop = f.path("other_backup_key");
    {
        let manifest = serde_json::to_vec(&f.embedded()).unwrap();
        write_envelope(&other_shop, &other_key, 10, &mut |w| {
            w.write_all(MAGIC)?;
            w.write_all(&len_field(manifest.len()))?;
            w.write_all(&manifest)?;
            io::copy(&mut File::open(&f.plain)?, w)?;
            Ok(())
        });
    }
    add(
        "sealed_with_the_key_of_another_shop",
        other_shop,
        &[BackupCode::ErrBackupKeyWrong],
    );

    // -- the chain key in the manifest is not a key
    for (label, text) in [
        ("chain_key_empty", String::new()),
        ("chain_key_not_hex", "z".repeat(64)),
        ("chain_key_63", "a".repeat(63)),
        ("chain_key_65", "a".repeat(65)),
        ("chain_key_multibyte", "é".repeat(32)),
        ("chain_key_huge", "a".repeat(500_000)),
        ("chain_key_nul", "\0".repeat(64)),
    ] {
        let mut e = f.embedded();
        e.chain_key = Some(SecretText::new(text));
        add(label, f.bundle(label, &e, &f.plain), &any);
    }

    // -- the manifest bytes themselves
    let manifest = serde_json::to_vec(&f.embedded()).unwrap();
    let db = fs::read(&f.plain).unwrap();
    let mut big = f.embedded();
    for i in 0..40_000 {
        big.inner
            .counts
            .insert(format!("padding_table_{i:08}_{}", "x".repeat(40)), 1);
    }
    assert!(serde_json::to_vec(&big).unwrap().len() > MANIFEST_LIMIT);
    add(
        "manifest_over_1_mib",
        f.bundle("manifest_big", &big, &f.plain),
        &corrupt,
    );
    add(
        "manifest_is_not_json",
        f.bundle_parts(
            "manifest_text",
            &[MAGIC, &len_field(11), b"hello world", &db],
        ),
        &corrupt,
    );
    let deep = vec![b'['; 400_000];
    add(
        "manifest_nested_400k_deep",
        f.bundle_parts(
            "manifest_deep",
            &[MAGIC, &len_field(deep.len()), &deep, &db],
        ),
        &corrupt,
    );
    add(
        "manifest_cut_in_half",
        f.bundle_parts(
            "manifest_half",
            &[
                MAGIC,
                &len_field(manifest.len() / 2),
                &manifest[..manifest.len() / 2],
                &db,
            ],
        ),
        &corrupt,
    );
    for (label, length) in [
        ("length_zero", 0_u32),
        ("length_u32_max", u32::MAX),
        ("length_one_more_than_limit", (MANIFEST_LIMIT + 1) as u32),
        ("length_longer_than_stream", 50_000_000),
        ("length_one_less", (manifest.len() - 1) as u32),
        ("length_one_more", (manifest.len() + 1) as u32),
    ] {
        add(
            label,
            f.bundle_parts(label, &[MAGIC, &length.to_le_bytes(), &manifest, &db]),
            &corrupt,
        );
    }

    // -- the first bytes
    add(
        "wrong_magic",
        f.bundle_parts(
            "wrong_magic",
            &[b"MANDIBK2", &len_field(manifest.len()), &manifest, &db],
        ),
        &corrupt,
    );
    add(
        "only_magic",
        f.bundle_parts("only_magic", &[MAGIC]),
        &corrupt,
    );
    add(
        "three_bytes",
        f.bundle_parts("three_bytes", &[b"MAN"]),
        &corrupt,
    );
    add(
        "empty_stream",
        f.bundle_parts("empty_stream", &[]),
        &corrupt,
    );
    add(
        "magic_and_half_a_length",
        f.bundle_parts("half_length", &[MAGIC, &[1, 0]]),
        &corrupt,
    );

    // -- inside the envelope there is no zstd stream at all
    let not_zstd = f.path("not_zstd");
    {
        let mut recipient = age::scrypt::Recipient::new(pass_of(&f.key));
        recipient.set_work_factor(10);
        let encryptor =
            age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
                .unwrap();
        let mut locked = encryptor
            .wrap_output(File::create(&not_zstd).unwrap())
            .unwrap();
        locked.write_all(b"plain bytes, never compressed").unwrap();
        locked.finish().unwrap();
    }
    add("age_but_no_zstd", not_zstd, &corrupt);

    let mut problems = Vec::new();
    for (label, path, allowed) in &cases {
        problems.extend(attack(
            (&f.new, &f.before),
            label,
            &file_source(path),
            Some(&name_of(path)),
            &f.key,
            BOTH,
            allowed,
        ));
    }

    // The attacks must not have poisoned anything: the genuine copy still restores.
    let genuine = f.bundle("genuine", &f.embedded(), &f.plain);
    let restored = f.new.service.first_run_restore(
        &restore_input(file_source(&genuine), &name_of(&genuine), &f.key),
        &|_| {},
    );
    if let Err(error) = restored {
        problems.push(format!(
            "after all attacks the genuine copy was refused: {error:?}"
        ));
    }
    assert_no_problems(&problems);
    assert_books_and_chains(&f.new);
}

/// A copy made on a computer that was never set up holds a valid, current, EMPTY shop: no Owner.
/// Restoring it would leave this computer with no Owner but with the old backup key stored as
/// confirmed, so the new shop that the Owner then sets up could not make its own key.
#[test]
fn a_copy_that_holds_no_owner_is_refused_and_changes_nothing() {
    let f = fixture();
    let no_owner = f.work.path().join("plain-no-owner.db");
    {
        let conn =
            Connection::open_with_flags(f.new.db_path(), OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        conn.execute("VACUUM INTO ?1", [no_owner.to_str().unwrap()])
            .unwrap();
    }
    let path = f.bundle(
        "no_owner",
        &embedded_for(&no_owner, Some(&old_chain_key())),
        &no_owner,
    );
    let problems = attack(
        (&f.new, &f.before),
        "copy with no Owner",
        &file_source(&path),
        Some(&name_of(&path)),
        &f.key,
        BOTH,
        &[],
    );
    assert_no_problems(&problems);
}

// ---------------------------------------------------------------------------------------------
// 2. A real bundle cut or flipped
// ---------------------------------------------------------------------------------------------

#[test]
fn a_real_bundle_cut_or_flipped_anywhere_is_refused_and_changes_nothing() {
    let f = fixture();
    let real = f.work.path().join("mandi-real.mbk");
    f.old
        .cloud
        .get_file(&format!("backups/{}", f.cloud_name), &real)
        .unwrap();
    let good = fs::read(&real).unwrap();
    assert!(good.len() > 2_000, "the bundle is big enough to cut");

    let mut offsets: Vec<usize> = vec![
        0, 1, 2, 7, 8, 21, 22, 40, 80, 120, 160, 200, 250, 300, 400, 600, 1000,
    ];
    let step = good.len() / 20;
    offsets.extend((1..20).map(|i| i * step));
    offsets.push(good.len() - 1);
    offsets.push(good.len() - 17);
    offsets.sort_unstable();
    offsets.dedup();
    offsets.retain(|o| *o < good.len());

    let mut problems = Vec::new();
    let mut tried = 0_usize;
    let allowed = [BackupCode::ErrBackupCorrupt, BackupCode::ErrBackupKeyWrong];
    for (index, offset) in offsets.iter().enumerate() {
        // Both calls on a few, the preview only on the rest (they share the same first steps).
        let calls: &[Call] = if index % 5 == 0 {
            BOTH
        } else {
            &[Call::Preview]
        };
        let cut = f.work.path().join(format!("mandi-cut-{offset}.mbk"));
        fs::write(&cut, &good[..*offset]).unwrap();
        problems.extend(attack(
            (&f.new, &f.before),
            &format!("cut at {offset}"),
            &file_source(&cut),
            None,
            &f.key,
            calls,
            &allowed,
        ));
        let flipped = f.work.path().join(format!("mandi-flip-{offset}.mbk"));
        let mut bytes = good.clone();
        bytes[*offset] ^= 0x01;
        fs::write(&flipped, &bytes).unwrap();
        problems.extend(attack(
            (&f.new, &f.before),
            &format!("one bit flipped at {offset}"),
            &file_source(&flipped),
            None,
            &f.key,
            calls,
            &allowed,
        ));
        tried += 2;
    }
    let mut longer = good.clone();
    longer.push(0);
    let longer_path = f.work.path().join("mandi-longer.mbk");
    fs::write(&longer_path, &longer).unwrap();
    problems.extend(attack(
        (&f.new, &f.before),
        "one byte added at the end",
        &file_source(&longer_path),
        None,
        &f.key,
        BOTH,
        &allowed,
    ));
    eprintln!("cut and flipped bundles tried: {tried}");
    assert_no_problems(&problems);

    // The untouched file still restores, so no attack left the service stuck.
    let done = f
        .new
        .service
        .first_run_restore(
            &restore_input(file_source(&real), &name_of(&real), &f.key),
            &|_| {},
        )
        .expect("the genuine bundle restores after all the attacks");
    assert!(done.totals.journal_entries > 0);
    assert_books_and_chains(&f.new);
}

// ---------------------------------------------------------------------------------------------
// 3. A header that asks for a huge amount of work, with no key needed
// ---------------------------------------------------------------------------------------------

/// The age header names the scrypt work factor. The decryptor spends that work BEFORE it can
/// tell the key is wrong. A header that asks for 2^22 costs about 4 GiB of memory and many
/// seconds. The app made the bundle itself with 2^15, so it can refuse anything bigger at once.
#[test]
fn a_header_asking_for_a_huge_scrypt_work_factor_is_refused_fast_and_not_as_a_wrong_key() {
    let f = fixture();
    let path = f.path("work_factor");
    // A real envelope at the app's own work factor.
    let manifest = serde_json::to_vec(&f.embedded()).unwrap();
    write_envelope(&path, &f.key, 15, &mut |w| {
        w.write_all(MAGIC)?;
        w.write_all(&len_field(manifest.len()))?;
        w.write_all(&manifest)?;
        io::copy(&mut File::open(&f.plain)?, w)?;
        Ok(())
    });
    // How long the genuine one takes to open (the baseline of this machine).
    let started = Instant::now();
    f.new
        .service
        .first_run_preview(&preview_input(file_source(&path), None, &f.key), &|_| {})
        .expect("the genuine bundle opens");
    let baseline = started.elapsed();

    // The same file with the number in its header made bigger: 2^18 is 8 times the work.
    let mut bytes = fs::read(&path).unwrap();
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(300)]).into_owned();
    let marker = head
        .find("-> scrypt ")
        .expect("the header has a scrypt line");
    let line_end = head[marker..].find('\n').unwrap() + marker;
    assert_eq!(&head[line_end - 3..line_end], " 15", "the header says 2^15");
    bytes[line_end - 2] = b'1';
    bytes[line_end - 1] = b'8';
    fs::write(&path, &bytes).unwrap();

    let started = Instant::now();
    let error = f
        .new
        .service
        .first_run_preview(&preview_input(file_source(&path), None, &f.key), &|_| {})
        .unwrap_err();
    let took = started.elapsed();
    eprintln!(
        "genuine open {baseline:?}; a header asking for 2^18 took {took:?} and gave {:?}",
        error.code
    );
    assert_ne!(
        error.code,
        BackupCode::ErrBackupKeyWrong,
        "the key is right: a damaged or hostile header must not say 'wrong key'"
    );
    assert_eq!(error.code, BackupCode::ErrBackupCorrupt);
    assert!(
        took < baseline * 2,
        "a header that asks for 8 times the work the app ever used took {took:?} against \
         {baseline:?} for a genuine copy: the work is done before it is refused. At 2^22 it is \
         about 4 GiB of memory with no key needed. Fix: set_max_work_factor(WORK_FACTOR) on the \
         scrypt identity in pack.rs open_to."
    );
    assert!(fresh_problems(&f.new, &f.before).is_empty());
}

// ---------------------------------------------------------------------------------------------
// 4. A decompression bomb and a nearly full disk
// ---------------------------------------------------------------------------------------------

/// A bundle that unpacks to far more than its manifest says. The unpack must stop as soon as
/// it has written more than `plain_size`, not after the last byte.
#[test]
fn a_bundle_that_unpacks_to_far_more_than_its_manifest_says_stops_early() {
    let f = fixture();
    let big = 128 * 1024 * 1024_u64;
    let mut e = f.embedded();
    e.inner.plain_size = 4096;
    e.inner.plain_blake3 = "0".repeat(64);
    let path = f.path("bomb");
    let manifest = serde_json::to_vec(&e).unwrap();
    write_envelope(&path, &f.key, 10, &mut |w| {
        w.write_all(MAGIC)?;
        w.write_all(&len_field(manifest.len()))?;
        w.write_all(&manifest)?;
        io::copy(&mut io::repeat(0).take(big), w)?;
        Ok(())
    });
    let packed = fs::metadata(&path).unwrap().len();
    eprintln!("bomb: {packed} bytes on disk unpack to {big} bytes");

    let peak = watch(f.new.root.path().join("backups").join("tmp"));
    let error = f
        .new
        .service
        .first_run_preview(&preview_input(file_source(&path), None, &f.key), &|_| {})
        .unwrap_err();
    let peak_bytes = peak.finish();
    eprintln!(
        "bomb: refused with {:?}, scratch peak {peak_bytes} bytes",
        error.code
    );
    assert_eq!(error.code, BackupCode::ErrBackupCorrupt);
    assert!(
        peak_bytes <= 16 * 1024 * 1024,
        "the manifest says 4096 bytes but the unpack wrote {peak_bytes} bytes before it noticed. \
         Fix: stop writing in pack.rs open_to when written bytes pass plain_size."
    );
    assert!(fresh_problems(&f.new, &f.before).is_empty());
}

/// Free disk space small, a big copy: the app must say "disk full" at the start (NFR-REL-10),
/// and must not fill the disk first.
#[test]
fn a_big_copy_on_a_nearly_full_disk_is_refused_before_it_is_unpacked() {
    let f = fixture();
    let (new, disk) = new_computer_with_disk(f.old.cloud.clone());
    let before = new.fingerprint();
    let big = 96 * 1024 * 1024_u64;

    // An honest big payload: the manifest tells its size, so the room can be checked at once.
    let path = f.path("honest_big");
    let mut e = f.embedded();
    e.inner.plain_size = big;
    e.inner.plain_blake3 = "0".repeat(64);
    let manifest = serde_json::to_vec(&e).unwrap();
    write_envelope(&path, &f.key, 10, &mut |w| {
        w.write_all(MAGIC)?;
        w.write_all(&len_field(manifest.len()))?;
        w.write_all(&manifest)?;
        io::copy(&mut io::repeat(0).take(big), w)?;
        Ok(())
    });
    disk.set(8 * 1024 * 1024);
    let peak = watch(new.root.path().join("backups").join("tmp"));
    let error = new
        .service
        .first_run_preview(&preview_input(file_source(&path), None, &f.key), &|_| {})
        .unwrap_err();
    let peak_bytes = peak.finish();
    eprintln!(
        "honest big copy: {:?}, scratch peak {peak_bytes}",
        error.code
    );
    assert_eq!(
        error.code,
        BackupCode::ErrDiskFull,
        "8 MiB free and a copy that unpacks to 96 MiB must say ERR_DISK_FULL (scratch peak \
         {peak_bytes} bytes). Fix: after the manifest is read, check plain_size against the free \
         space before the first byte is written (pack.rs open_to / first_run.rs prepare)."
    );
    assert!(
        peak_bytes <= 16 * 1024 * 1024,
        "scratch grew to {peak_bytes} bytes on a disk that said 8 MiB free"
    );
    assert!(fresh_problems(&new, &before).is_empty());
}

/// A big `.mbk` file picked by hand is copied into the scratch folder first. With no room it must
/// be refused before the copy.
#[test]
fn a_big_file_picked_on_a_nearly_full_disk_is_refused_before_it_is_copied() {
    let f = fixture();
    let (new, disk) = new_computer_with_disk(f.old.cloud.clone());
    let before = new.fingerprint();
    let sparse = f.work.path().join("mandi-sparse.mbk");
    File::create(&sparse)
        .unwrap()
        .set_len(256 * 1024 * 1024)
        .unwrap();
    disk.set(8 * 1024 * 1024);
    let peak = watch(new.root.path().join("backups").join("tmp"));
    let error = new
        .service
        .first_run_preview(&preview_input(file_source(&sparse), None, &f.key), &|_| {})
        .unwrap_err();
    let peak_bytes = peak.finish();
    eprintln!(
        "sparse 256 MiB file: {:?}, scratch peak {peak_bytes}",
        error.code
    );
    assert_eq!(
        error.code,
        BackupCode::ErrDiskFull,
        "a 256 MiB file on a disk with 8 MiB free must say ERR_DISK_FULL before it is copied \
         (scratch peak {peak_bytes} bytes). Fix: require_room(file size + margin) in \
         first_run.rs copy_local_source."
    );
    assert!(
        peak_bytes <= 16 * 1024 * 1024,
        "the whole file was copied into scratch ({peak_bytes} bytes) before any room check"
    );
    assert!(fresh_problems(&new, &before).is_empty());
}

// ---------------------------------------------------------------------------------------------
// 5. Path tricks, special files, names, keys and cloud fields
// ---------------------------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn a_symlink_to_a_secret_file_leaks_none_of_it() {
    use std::os::unix::fs::symlink;
    const MARKER: &[u8] = b"TOP-SECRET-MARKER-5c1d9e7f-do-not-print";
    let f = fixture();
    let elsewhere = tempfile::tempdir().unwrap();
    let secret = elsewhere.path().join("passwords.txt");
    let mut body = b"line one\n".to_vec();
    body.extend_from_slice(MARKER);
    body.extend_from_slice(b"\nline three\n".repeat(50).as_slice());
    fs::write(&secret, &body).unwrap();

    let folder = f.work.path().join("usb");
    fs::create_dir_all(&folder).unwrap();
    let link = folder.join("mandi-20261011T090000000Z.mbk");
    symlink(&secret, &link).unwrap();

    let mut problems = Vec::new();
    let sources = [
        ("folder", folder_source(&folder), Some(name_of(&link))),
        ("file", file_source(&link), None),
    ];
    for (label, source, copy) in &sources {
        for call in BOTH {
            let result = run_call(&f.new, *call, source, copy.as_deref(), &f.key);
            match result {
                Err(text) => problems.push(format!("{label}: {text}")),
                Ok(Ok(())) => problems.push(format!("{label}: a password file was ACCEPTED")),
                Ok(Err(error)) => {
                    let shown = format!("{error:?} {} {:?}", error.detail, error.values);
                    if shown.contains("TOP-SECRET") {
                        problems.push(format!("{label}: the error shows the file: {shown}"));
                    }
                }
            }
        }
        problems.extend(
            fresh_problems(&f.new, &f.before)
                .into_iter()
                .map(|p| format!("{label}: {p}")),
        );
    }
    for (path, bytes) in f.new.all_files() {
        if contains(&bytes, MARKER) {
            problems.push(format!("the secret was copied into {}", path.display()));
        }
    }
    assert_no_problems(&problems);
}

#[cfg(unix)]
#[test]
fn directories_and_broken_links_named_like_copies_are_not_listed_as_copies() {
    use std::os::unix::fs::symlink;
    let f = fixture();
    let folder = f.work.path().join("usb");
    fs::create_dir_all(&folder).unwrap();
    let good_name = "mandi-20261010T090000000Z.mbk";
    let good = folder.join(good_name);
    fs::copy(
        {
            let real = f.work.path().join("real.mbk");
            f.old
                .cloud
                .get_file(&format!("backups/{}", f.cloud_name), &real)
                .unwrap();
            real
        },
        &good,
    )
    .unwrap();
    // Newer than the good copy, so "the newest copy" would pick them first.
    fs::create_dir_all(folder.join("mandi-20261012T090000000Z.mbk")).unwrap();
    symlink(
        "mandi-20261013T090000000Z.mbk",
        folder.join("mandi-20261013T090000000Z.mbk"),
    )
    .unwrap();
    symlink(
        "/mandi-no-such-target",
        folder.join("mandi-20261014T090000000Z.mbk"),
    )
    .unwrap();
    symlink("/dev/null", folder.join("mandi-20261015T090000000Z.mbk")).unwrap();

    let listed = f
        .new
        .service
        .first_run_list(&FirstRunListInput {
            source: folder_source(&folder),
        })
        .expect("a folder with odd entries still lists");
    let names: Vec<&str> = listed.rows.iter().map(|r| r.name.as_str()).collect();
    assert!(
        names.contains(&good_name),
        "the good copy is listed: {names:?}"
    );
    assert_eq!(
        names,
        vec![good_name],
        "a directory, a looping link, a broken link and a link to /dev/null are not copies"
    );
    // No name given: the newest real copy is picked, not a directory.
    let preview = f
        .new
        .service
        .first_run_preview(
            &preview_input(folder_source(&folder), None, &f.key),
            &|_| {},
        )
        .expect("the newest REAL copy is previewed");
    assert_eq!(preview.copy.name, good_name);
}

#[test]
fn a_folder_with_thousands_of_copies_lists_quickly() {
    let f = fixture();
    let folder = f.work.path().join("usb");
    fs::create_dir_all(&folder).unwrap();
    for i in 0..5_000_u32 {
        let secs = i % 60;
        let mins = (i / 60) % 60;
        let hours = (i / 3600) % 24;
        let day = 1 + i / 86_400 % 28;
        File::create(folder.join(format!(
            "mandi-202601{day:02}T{hours:02}{mins:02}{secs:02}000Z.mbk"
        )))
        .unwrap();
    }
    let started = Instant::now();
    let listed = f
        .new
        .service
        .first_run_list(&FirstRunListInput {
            source: folder_source(&folder),
        })
        .unwrap();
    let took = started.elapsed();
    eprintln!("{} rows listed in {took:?}", listed.rows.len());
    assert_eq!(listed.rows.len(), 5_000);
    assert_eq!(listed.total_count, 5_000);
    assert!(
        took < Duration::from_secs(3),
        "listing 5,000 copies took {took:?}"
    );
}

#[test]
fn copy_names_that_try_to_leave_the_folder_or_the_bucket_are_refused_before_any_io() {
    let f = fixture();
    let (counting, _entered, _release) = Gated::new(f.old.cloud.clone());
    let new = new_computer(counting.clone());
    let before = new.fingerprint();
    let usb = f.work.path().join("usb");
    fs::create_dir_all(&usb).unwrap();
    let out_of_range = [BackupCode::ErrOutOfRange];
    let bad = [
        "../mandi-20200101T000000000Z.mbk",
        "..\\mandi-20200101T000000000Z.mbk",
        "sub/mandi-20200101T000000000Z.mbk",
        "sub\\mandi-20200101T000000000Z.mbk",
        "/etc/passwd",
        "C:\\Windows\\win.ini",
        "\\\\server\\share\\mandi-20200101T000000000Z.mbk",
        "mandi-20200101T000000000Z.mbk\0.txt",
        "mandi-20200101T000000000Z.mbk:evil",
        "mandi-20200101T000000000Z.MBK",
        "mandi-20200101T000000000Z.mbk.json",
        "mandi-local-20200101T000000000Z.db",
        "mandi-%2e%2e%2fx.mbk",
        "mandi-\u{ff12}\u{ff10}.mbk",
        "mandi-\u{202e}kbm.exe",
        "mandi-20200101T000000000Z.mbk/..",
        &format!("mandi-{}.mbk", "a".repeat(200)),
        "mandi-..mbk",
        "mandi-x/../../mandi-y.mbk",
    ];
    let mut problems = Vec::new();
    for source in [folder_source(&usb), cloud_source(&f.old)] {
        for name in bad {
            problems.extend(attack(
                (&new, &before),
                &format!("name {name:?}"),
                &source,
                Some(name),
                &f.key,
                BOTH,
                &out_of_range,
            ));
        }
    }
    assert_eq!(
        counting.downloads.load(Ordering::SeqCst),
        0,
        "a bad name must be refused before the cloud is asked for anything"
    );
    // A clean name that is not there is "not found", not an error of the name. The screen trims
    // spaces and a new line around a name, so those are the same clean name.
    for name in [
        "mandi-20200101T000000000Z.mbk",
        "  mandi-20200101T000000000Z.mbk  ",
        "mandi-20200101T000000000Z.mbk\n",
    ] {
        problems.extend(attack(
            (&new, &before),
            &format!("missing copy {name:?}"),
            &folder_source(&usb),
            Some(name),
            &f.key,
            BOTH,
            &[BackupCode::ErrBackupNotFound],
        ));
    }
    assert_no_problems(&problems);
}

#[test]
fn hostile_paths_keys_and_cloud_fields_are_refused_with_a_code_and_no_crash() {
    let f = fixture();
    let usb = f.work.path().join("usb");
    fs::create_dir_all(&usb).unwrap();
    let mut problems = Vec::new();
    let path_codes = [
        BackupCode::ErrRequired,
        BackupCode::ErrOutOfRange,
        BackupCode::ErrBackupUsbNotPresent,
    ];

    let long = format!("/{}", "x".repeat(2_000));
    let folders = [
        "",
        "   ",
        "relative",
        "./here",
        "../up",
        "~/backups",
        "file:///tmp",
        "/mandi\0nul",
        "/mandi\nnewline",
        "/mandi\ttab",
        "/mandi-no-such-place",
        "\\\\server\\share\\x",
        long.as_str(),
    ];
    for text in folders {
        problems.extend(attack(
            (&f.new, &f.before),
            &format!("folder {:?}", &text[..text.len().min(30)]),
            &RestoreSource::Folder(text.to_owned()),
            None,
            &f.key,
            BOTH,
            &path_codes,
        ));
    }
    let files = [
        "",
        "relative.mbk",
        "/tmp/notes.txt",
        "/tmp/.mbk",
        "/tmp/x.mbk/",
        "/mandi-no-such-file.mbk",
        "/mandi\0.mbk",
        "/dev/null.mbk",
    ];
    for text in files {
        problems.extend(attack(
            (&f.new, &f.before),
            &format!("file {text:?}"),
            &RestoreSource::File(text.to_owned()),
            None,
            &f.key,
            BOTH,
            &path_codes,
        ));
    }

    // Keys: all of them are the wrong form, and none costs time or memory.
    let huge = "A".repeat(2_000_000);
    let keys = [
        "",
        " ",
        "ABCD",
        "ABCD-EFGH-JKLM-NPQR-STUV-WXY",
        "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ-ABCD",
        "0000-0000-0000-0000-0000-0000",
        "ABCD-EFGH-JKLM-NPQR-STUV-WXY\0",
        "\u{ff21}\u{ff22}\u{ff23}\u{ff24}-EFGH-JKLM-NPQR-STUV-WXYZ",
        "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ'; DROP TABLE users; --",
        huge.as_str(),
    ];
    let usb_source = folder_source(&usb);
    for text in keys {
        let started = Instant::now();
        problems.extend(attack(
            (&f.new, &f.before),
            &format!("key {:?}", &text[..text.len().min(30)]),
            &usb_source,
            None,
            text,
            BOTH,
            &[BackupCode::ErrBackupKeyFormat],
        ));
        if started.elapsed() > Duration::from_secs(1) {
            problems.push(format!("a bad key took {:?} to refuse", started.elapsed()));
        }
    }

    // The cloud fields: the app must refuse them before it connects anywhere.
    let account = |change: &dyn Fn(&mut mandi_backup::api::CloudAccountInput)| {
        let mut a = f.old.account();
        change(&mut a);
        RestoreSource::Cloud(a)
    };
    let long_text = "h".repeat(10_000);
    let hostile_accounts: Vec<(&str, RestoreSource)> = vec![
        (
            "http endpoint",
            account(&|a| a.endpoint = "http://x.example".into()),
        ),
        (
            "ftp endpoint",
            account(&|a| a.endpoint = "ftp://x.example".into()),
        ),
        ("empty endpoint", account(&|a| a.endpoint = String::new())),
        ("space in bucket", account(&|a| a.bucket = "a b".into())),
        ("dots in bucket", account(&|a| a.bucket = "../x".into())),
        (
            "dots in prefix",
            account(&|a| a.prefix = "../../etc".into()),
        ),
        (
            "newline in key id",
            account(&|a| a.key_id = "AKIA\nX".into()),
        ),
        ("10k secret", account(&|a| a.secret = long_text.clone())),
    ];
    for (label, source) in &hostile_accounts {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            f.new.service.first_run_list(&FirstRunListInput {
                source: source.clone(),
            })
        }));
        match result {
            Err(_) => problems.push(format!("cloud {label}: PANICKED")),
            Ok(Ok(_)) => problems.push(format!("cloud {label}: was ACCEPTED")),
            Ok(Err(error)) => {
                if !matches!(
                    error.code,
                    BackupCode::ErrRequired | BackupCode::ErrOutOfRange
                ) {
                    problems.push(format!("cloud {label}: gave {:?}", error.code));
                }
            }
        }
        problems.extend(
            fresh_problems(&f.new, &f.before)
                .into_iter()
                .map(|p| format!("cloud {label}: {p}")),
        );
    }

    // A key typed any way the Owner might type it from the printed sheet is the same key.
    let lower = f.key.to_lowercase();
    let spaced = f.key.replace('-', " ");
    let squashed = f.key.replace('-', "");
    let padded = format!("  {}\n", f.key);
    for typed in [lower, spaced, squashed, padded] {
        if let Err(error) = f
            .new
            .service
            .first_run_preview(&preview_input(cloud_source(&f.old), None, &typed), &|_| {})
        {
            problems.push(format!("the key typed as {typed:?} was refused: {error:?}"));
        }
    }
    assert_no_problems(&problems);
}

// ---------------------------------------------------------------------------------------------
// 5b. A tampered copy, a first run that is not a first run, secrets
// ---------------------------------------------------------------------------------------------

/// Drops every trigger of a copy (a man editing the file by hand does), then runs `edit`.
fn edit_outside_the_app(plain: &Path, edit: &str) {
    let conn = Connection::open(plain).unwrap();
    conn.execute_batch("PRAGMA journal_mode = DELETE; PRAGMA foreign_keys = OFF;")
        .unwrap();
    let triggers: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for name in triggers {
        conn.execute_batch(&format!("DROP TRIGGER \"{name}\";"))
            .unwrap();
    }
    conn.execute_batch(edit).unwrap();
}

/// A copy edited by hand, with the manifest made again to fit (counts, size, checksum): every
/// check but the hash chains passes. The chains must refuse it.
#[test]
fn a_copy_edited_outside_the_app_is_refused_by_the_chains_and_changes_nothing() {
    let f = fixture();
    let edits: [(&str, &str); 7] = [
        (
            "a journal amount raised on one side and on the other, so the books still balance",
            "UPDATE journal_lines SET dr_paisa = dr_paisa + 10000 \
                 WHERE id = (SELECT MIN(id) FROM journal_lines WHERE dr_paisa > 0); \
             UPDATE journal_lines SET cr_paisa = cr_paisa + 10000 \
                 WHERE id = (SELECT MAX(id) FROM journal_lines WHERE cr_paisa > 0);",
        ),
        (
            "a journal line moved to another account",
            "UPDATE journal_lines SET account_id = (SELECT MAX(id) FROM accounts) \
                 WHERE id = (SELECT MIN(id) FROM journal_lines WHERE dr_paisa > 0);",
        ),
        (
            "a journal entry dated another day",
            "UPDATE journal_entries SET entry_date = '2026-09-01' \
                 WHERE id = (SELECT MIN(id) FROM journal_entries WHERE status = 'posted');",
        ),
        (
            "two journal hashes swapped",
            "UPDATE journal_entries SET chain_hash = CASE id \
                 WHEN (SELECT MIN(id) FROM journal_entries WHERE status = 'posted') \
                   THEN (SELECT chain_hash FROM journal_entries WHERE id = \
                         (SELECT MIN(id) + 1 FROM journal_entries WHERE status = 'posted')) \
                 ELSE (SELECT chain_hash FROM journal_entries WHERE id = \
                         (SELECT MIN(id) FROM journal_entries WHERE status = 'posted')) END \
             WHERE id IN ((SELECT MIN(id) FROM journal_entries WHERE status = 'posted'), \
                          (SELECT MIN(id) + 1 FROM journal_entries WHERE status = 'posted'));",
        ),
        (
            "an audit row changed",
            "UPDATE audit_log SET new_json = '{\"edited\":true}' \
                 WHERE id = (SELECT MAX(id) / 2 FROM audit_log);",
        ),
        (
            "an audit row taken out of the middle",
            "DELETE FROM audit_log WHERE id = (SELECT MAX(id) / 2 FROM audit_log);",
        ),
        (
            "a posted entry taken out of the middle with its lines",
            "DELETE FROM journal_lines WHERE entry_id = \
                 (SELECT id FROM journal_entries WHERE status = 'posted' ORDER BY id LIMIT 1 OFFSET 2); \
             DELETE FROM journal_entries WHERE id = \
                 (SELECT id FROM journal_entries WHERE status = 'posted' ORDER BY id LIMIT 1 OFFSET 2);",
        ),
    ];
    let mut problems = Vec::new();
    for (index, (what, sql)) in edits.iter().enumerate() {
        let plain = f.work.path().join(format!("tampered-{index}.db"));
        fs::copy(&f.plain, &plain).unwrap();
        edit_outside_the_app(&plain, sql);
        let path = f.bundle(
            &format!("tampered_{index}"),
            &embedded_for(&plain, Some(&old_chain_key())),
            &plain,
        );
        for call in BOTH {
            match run_call(
                &f.new,
                *call,
                &file_source(&path),
                Some(&name_of(&path)),
                &f.key,
            ) {
                Err(text) => problems.push(format!("{what}: {text}")),
                Ok(Ok(())) => problems.push(format!("{what}: ACCEPTED by {}", call_name(*call))),
                Ok(Err(error)) => {
                    let check = value_of(&error, "check");
                    if error.code != BackupCode::ErrBackupCorrupt
                        || check.as_deref() != Some("chains")
                    {
                        problems.push(format!(
                            "{what}: {} gave {:?} check {check:?}, expected corrupt / chains",
                            call_name(*call),
                            error.code
                        ));
                    }
                }
            }
        }
        problems.extend(
            fresh_problems(&f.new, &f.before)
                .into_iter()
                .map(|p| format!("{what}: {p}")),
        );
    }
    assert_no_problems(&problems);
}

/// The chains do not cover the partner shares. A copy whose partner share rows were changed by
/// hand (the sum no longer matches the income and expense of the books, PRD FR-6.5) passes every
/// check today. NFR-SEC-6 says the other money tables "can be rebuilt and compared from the
/// journal"; the restore does not do it.
#[test]
#[ignore = "FINDING R4-9 (MINOR): partner share rows are in no chain and no restore check; needs the backup key to exploit"]
fn a_copy_whose_partner_shares_no_longer_add_up_to_the_books_is_refused() {
    let f = fixture();
    let plain = f.work.path().join("tampered-shares.db");
    fs::copy(&f.plain, &plain).unwrap();
    edit_outside_the_app(
        &plain,
        "UPDATE partner_share_rows SET amount_paisa = amount_paisa + 100000 \
             WHERE id = (SELECT MIN(id) FROM partner_share_rows WHERE kind = 'income');",
    );
    let path = f.bundle(
        "tampered_shares",
        &embedded_for(&plain, Some(&old_chain_key())),
        &plain,
    );
    let problems = attack(
        (&f.new, &f.before),
        "partner share raised by 1,000 rupees",
        &file_source(&path),
        Some(&name_of(&path)),
        &f.key,
        BOTH,
        &[],
    );
    assert_no_problems(&problems);
}

/// Once a shop exists, the three first-run calls are refused before they look at anything, for
/// every kind of source, and they touch no network and no file.
#[test]
fn once_a_shop_exists_every_first_run_call_is_refused_before_any_input_is_read() {
    let f = fixture();
    let (counting, _entered, _release) = Gated::new(f.old.cloud.clone());
    // The old computer HAS an Owner. Its own service, with a counting cloud.
    let service = BackupService::new(
        Arc::clone(f.old.rig.w.env.ctx.db()),
        BackupConfig {
            backups_dir: f.old.rig.backups.clone(),
            app_version: "0.0.0-test".to_owned(),
        },
        BackupParts {
            secrets: f.old.secrets.clone(),
            cloud: Arc::new(FixedCloud(counting.clone())),
            disk: f.old.rig.disk.clone(),
            hook: Arc::new(NoHook),
            pack: Arc::new(PendingPack),
        },
    );
    let before = table_hashes(&f.old.rig.w.dir.path().join("data").join("mandi.db"), &[]);
    let usb = f.old.rig.usb.clone();
    let sources = [
        cloud_source(&f.old),
        folder_source(&usb),
        folder_source(Path::new("/mandi-no-such-folder")),
        folder_source(Path::new("relative/nonsense")),
        RestoreSource::Folder(String::new()),
        RestoreSource::Folder("x\0y".to_owned()),
        file_source(&f.plain),
        RestoreSource::File("/etc/passwd".to_owned()),
        RestoreSource::File(String::new()),
    ];
    let keys = [
        f.key.as_str(),
        "",
        "not a key",
        "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ",
    ];
    let mut problems = Vec::new();
    let mut calls = 0;
    for source in &sources {
        for key in keys {
            let list = service.first_run_list(&FirstRunListInput {
                source: source.clone(),
            });
            let preview =
                service.first_run_preview(&preview_input(source.clone(), None, key), &|_| {});
            let restore = service.first_run_restore(
                &restore_input(source.clone(), "mandi-20200101T000000000Z.mbk", key),
                &|_| {},
            );
            for (what, code) in [
                ("list", list.err().map(|e| e.code)),
                ("preview", preview.err().map(|e| e.code)),
                ("restore", restore.err().map(|e| e.code)),
            ] {
                calls += 1;
                if code != Some(BackupCode::ErrAlreadySetUp) {
                    problems.push(format!("{what} {source:?} key {key:?} gave {code:?}"));
                }
            }
        }
    }
    assert!(calls >= 100);
    assert_eq!(
        counting.downloads.load(Ordering::SeqCst),
        0,
        "nothing was downloaded"
    );
    let after = table_hashes(&f.old.rig.w.dir.path().join("data").join("mandi.db"), &[]);
    assert_eq!(after, before, "the shop's data file is as it was");
    assert_no_problems(&problems);
}

/// Collects every event a thread logs, at any level, as text.
struct Capture(Arc<Mutex<String>>);

struct Text<'a>(&'a mut String);

impl tracing::field::Visit for Text<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write as _;
        let _ = write!(self.0, "{}={:?} ", field.name(), value);
    }
}

impl tracing::Subscriber for Capture {
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let mut text = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        attrs.record(&mut Text(&mut text));
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, values: &tracing::span::Record<'_>) {
        let mut text = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        values.record(&mut Text(&mut text));
    }
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        use std::fmt::Write as _;
        let mut text = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = write!(text, "{} ", event.metadata().target());
        event.record(&mut Text(&mut text));
        text.push('\n');
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

/// My own scan, over more paths than the happy one: wrong key, damaged and hostile copies, a cloud
/// that refuses with the keys in its words, the symlink to a password file, and then the good
/// restore. Nothing secret may be in an error, a log line, or any file the new computer holds.
#[test]
fn no_secret_is_in_any_error_log_or_file_after_good_and_bad_restores() {
    let f = fixture();
    let flaky = Flaky::new(f.old.cloud.clone());
    let new = new_computer(flaky.clone());
    let before = new.fingerprint();
    let raw_key = f.key.replace('-', "");
    let wrong_key = mandi_backup::key::BackupKey::generate().unwrap().display();
    let account = f.old.account();
    let needles: Vec<(String, Vec<u8>)> = vec![
        ("the printed key".into(), f.key.clone().into_bytes()),
        (
            "the key without dashes".into(),
            raw_key.clone().into_bytes(),
        ),
        (
            "the key in lower case".into(),
            raw_key.to_lowercase().into_bytes(),
        ),
        ("the typed wrong key".into(), wrong_key.clone().into_bytes()),
        (
            "the chain key".into(),
            old_chain_key().to_hex().into_bytes(),
        ),
        (
            "the cloud secret".into(),
            account.secret.clone().into_bytes(),
        ),
        (
            "the cloud key id".into(),
            account.key_id.clone().into_bytes(),
        ),
        (
            "the old cloud secret".into(),
            b"old-secret-value-1".to_vec(),
        ),
        ("the old cloud key id".into(), b"AKIAOLDKEYID".to_vec()),
        (
            "the recovery code".into(),
            f.old.recovery_code.clone().into_bytes(),
        ),
    ];

    let log = Arc::new(Mutex::new(String::new()));
    let mut printed = String::new();
    let outcome = tracing::subscriber::with_default(Capture(log.clone()), || {
        tracing::warn!("capture probe");
        let source = cloud_source(&f.old);
        // The cloud says no, and puts the keys in its words.
        for error in [
            CloudError::Refused(format!("denied {} {}", account.key_id, account.secret)),
            CloudError::Offline(format!("no route for {}", account.secret)),
            CloudError::NotFound,
        ] {
            flaky.fail_with(Some(error));
            let listed = new.service.first_run_list(&list_input(source.clone()));
            printed.push_str(&format!("{listed:?}"));
            if let Err(e) = listed {
                printed.push_str(&format!(" {} {:?}", e.detail, e.values));
            }
        }
        flaky.fail_with(None);
        // A wrong key, then a hostile copy, then a symlink to a password file.
        let wrong = new.service.first_run_preview(
            &preview_input(source.clone(), Some(&f.cloud_name), &wrong_key),
            &|_| {},
        );
        printed.push_str(&format!("{wrong:?}"));
        for label in ["lying_checksum", "length_u32_max", "not_sqlite"] {
            let path = match label {
                "lying_checksum" => {
                    let mut e = f.embedded();
                    e.inner.plain_blake3 = "0".repeat(64);
                    f.bundle(label, &e, &f.plain)
                }
                "length_u32_max" => {
                    let manifest = serde_json::to_vec(&f.embedded()).unwrap();
                    f.bundle_parts(label, &[MAGIC, &u32::MAX.to_le_bytes(), &manifest])
                }
                _ => f.bundle_bytes(label, &f.embedded(), b"not a database"),
            };
            let result = new
                .service
                .first_run_preview(&preview_input(file_source(&path), None, &f.key), &|_| {});
            printed.push_str(&format!("{result:?}"));
            if let Err(e) = result {
                printed.push_str(&format!(" {} {:?}", e.detail, e.values));
            }
        }
        // The good restore.
        let done = new
            .service
            .first_run_restore(&restore_input(source, &f.cloud_name, &f.key), &|_| {});
        printed.push_str(&format!("{done:?}"));
        done
    });
    outcome.expect("the good restore went through");
    assert!(
        fresh_problems(&new, &before)
            .iter()
            .all(|p| !p.contains("scratch"))
    );

    let logged = log.lock().unwrap_or_else(PoisonError::into_inner).clone();
    assert!(logged.contains("capture probe"), "the capture works");
    let mut problems = Vec::new();
    for (label, needle) in &needles {
        let text = String::from_utf8_lossy(needle).into_owned();
        if printed.contains(&text) {
            problems.push(format!("{label} is in an error or a Debug text"));
        }
        if logged.contains(&text) {
            problems.push(format!("{label} is in the log"));
        }
        for (path, bytes) in new.all_files() {
            // The good restore stores the backup key in the credential store, never in a file.
            if contains(&bytes, needle) {
                problems.push(format!("{label} is in the file {}", path.display()));
            }
        }
    }
    assert_no_problems(&problems);
}

/// The cloud address is checked like the one on the Settings screen (the same code). A line
/// break inside it, or ten thousand letters, get through today.
#[test]
fn a_cloud_address_with_a_line_break_or_ten_thousand_letters_is_refused() {
    let f = fixture();
    let long_text = "h".repeat(10_000);
    let mut problems = Vec::new();
    for (label, endpoint) in [
        ("line break", "https://x.example\r\nX-Evil: 1".to_owned()),
        ("tab", "https://x.example\t/path".to_owned()),
        ("10k letters", format!("https://{long_text}.example")),
    ] {
        let mut account = f.old.account();
        account.endpoint = endpoint;
        match f.new.service.first_run_list(&FirstRunListInput {
            source: RestoreSource::Cloud(account),
        }) {
            Ok(_) => problems.push(format!("{label}: was ACCEPTED")),
            Err(error) => {
                if error.field.as_deref() != Some("source.cloud.endpoint") {
                    problems.push(format!(
                        "{label}: gave {:?} on {:?}",
                        error.code, error.field
                    ));
                }
            }
        }
    }
    assert_no_problems(&problems);
}

// ---------------------------------------------------------------------------------------------
// 6. Parallel calls
// ---------------------------------------------------------------------------------------------

/// While one restore is downloading, every other call says "busy" at once and none waits.
#[test]
fn while_one_restore_runs_every_other_call_says_busy_at_once() {
    let f = fixture();
    let (gated, entered, release) = Gated::new(f.old.cloud.clone());
    gated.gate_on.store(true, Ordering::SeqCst);
    let new = new_computer(gated.clone());
    let before = new.fingerprint();
    let source = cloud_source(&f.old);

    let service = Arc::clone(&new.service);
    let input = restore_input(source.clone(), &f.cloud_name, &f.key);
    let runner = thread::spawn(move || service.first_run_restore(&input, &|_| {}));
    entered
        .recv_timeout(Duration::from_secs(60))
        .expect("the first restore reached the download");

    let mut problems = Vec::new();
    let rivals: [(&str, Rival<'_>); 3] = [
        (
            "list",
            Box::new(|| {
                new.service
                    .first_run_list(&FirstRunListInput {
                        source: source.clone(),
                    })
                    .map(|_| ())
            }),
        ),
        (
            "preview",
            Box::new(|| {
                new.service
                    .first_run_preview(&preview_input(source.clone(), None, &f.key), &|_| {})
                    .map(|_| ())
            }),
        ),
        (
            "restore",
            Box::new(|| {
                new.service
                    .first_run_restore(
                        &restore_input(source.clone(), &f.cloud_name, &f.key),
                        &|_| {},
                    )
                    .map(|_| ())
            }),
        ),
    ];
    for (label, call) in &rivals {
        let started = Instant::now();
        match call() {
            Err(error) if error.code == BackupCode::ErrBackupBusy => {}
            other => problems.push(format!("{label} while busy gave {other:?}")),
        }
        if started.elapsed() > Duration::from_secs(2) {
            problems.push(format!("{label} while busy waited {:?}", started.elapsed()));
        }
    }
    // The three rivals did not start a second download.
    if gated.downloads.load(Ordering::SeqCst) != 1 {
        problems.push(format!(
            "{} downloads started, expected 1",
            gated.downloads.load(Ordering::SeqCst)
        ));
    }
    release.send(()).unwrap();
    let done = runner.join().expect("the first restore did not panic");
    if let Err(error) = &done {
        problems.push(format!("the first restore failed: {error:?}"));
    }
    assert_no_problems(&problems);
    assert!(!new.is_fresh());
    assert_books_and_chains(&new);

    // After it, every call says "already set up", not "busy" (the lock was let go).
    for call in [Call::Preview, Call::Restore] {
        match run_call(&new, call, &source, Some(&f.cloud_name), &f.key) {
            Ok(Err(error)) => assert_eq!(error.code, BackupCode::ErrAlreadySetUp),
            other => panic!(
                "after the restore, {call:?} gave {other:?}",
                call = call_name(call)
            ),
        }
    }
    drop(before);
}

/// Six restores of the same copy start at the same moment. Exactly one wins, the rest are told
/// "busy" or "already set up", and the books and both chains are whole.
#[test]
fn six_restores_at_the_same_moment_give_one_restore_and_whole_books() {
    let f = fixture();
    let new = Arc::new(new_computer(f.old.cloud.clone()));
    let source = cloud_source(&f.old);
    let barrier = Arc::new(Barrier::new(6));
    let handles: Vec<_> = (0..6)
        .map(|_| {
            let new = Arc::clone(&new);
            let barrier = Arc::clone(&barrier);
            let input = restore_input(source.clone(), &f.cloud_name, &f.key);
            thread::spawn(move || {
                barrier.wait();
                new.service.first_run_restore(&input, &|_| {})
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("no thread panicked"))
        .collect();
    let wins = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(wins, 1, "exactly one restore wins: {results:?}");
    for result in &results {
        if let Err(error) = result {
            assert!(
                matches!(
                    error.code,
                    BackupCode::ErrBackupBusy | BackupCode::ErrAlreadySetUp
                ),
                "a losing restore gave {:?}",
                error.code
            );
        }
    }
    assert_books_and_chains(&new);
    let users: i64 = new
        .ctx
        .db()
        .read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
                .map_err(BackupError::from)
        })
        .unwrap();
    assert_eq!(users, 1, "one Owner, the one of the copy");
    assert_eq!(
        new.secrets.peek(SecretName::BackupKey).as_deref(),
        Some(f.key.replace('-', "").as_str()),
        "the stored backup key is the key of the copy"
    );
    let tmp = new.root.path().join("backups").join("tmp");
    assert_eq!(fs::read_dir(tmp).map_or(0, Iterator::count), 0);
}

/// The Owner of a new shop is set up in the middle of a restore. The shop that was reported as
/// set up must not be erased without a word: either the setup or the restore must say no.
#[test]
fn a_shop_set_up_in_the_middle_of_a_restore_is_not_silently_erased() {
    let f = fixture();
    let (gated, entered, release) = Gated::new(f.old.cloud.clone());
    gated.gate_on.store(true, Ordering::SeqCst);
    let new = new_computer(gated.clone());
    let source = cloud_source(&f.old);
    let service = Arc::clone(&new.service);
    let input = restore_input(source, &f.cloud_name, &f.key);
    let runner = thread::spawn(move || service.first_run_restore(&input, &|_| {}));
    entered.recv_timeout(Duration::from_secs(60)).unwrap();

    let setup = auth::first_run(&new.ctx, &common::first_run_input());
    release.send(()).unwrap();
    let restored = runner.join().unwrap();
    eprintln!(
        "setup during restore: setup ok = {}, restore ok = {}",
        setup.is_ok(),
        restored.is_ok()
    );
    assert!(
        !(setup.is_ok() && restored.is_ok()),
        "the setup of a new shop was reported as done, and then the restore replaced the data \
         file: the new shop (its Owner, its recovery code) is gone. The first-run setup and the \
         restore must exclude each other (the restore holds the busy lock, setup does not look at it)."
    );
}

// ---------------------------------------------------------------------------------------------
// 7. The preview called many times
// ---------------------------------------------------------------------------------------------

fn many_previews(count: usize) {
    let f = fixture();
    let (counting, _entered, _release) = Gated::new(f.old.cloud.clone());
    let new = new_computer(counting.clone());
    let before = new.fingerprint();
    let source = cloud_source(&f.old);
    let size_before = dir_bytes(new.root.path());
    let mut times = Vec::with_capacity(count);
    for i in 0..count {
        let started = Instant::now();
        new.service
            .first_run_preview(
                &preview_input(source.clone(), Some(&f.cloud_name), &f.key),
                &|_| {},
            )
            .unwrap_or_else(|e| panic!("preview {i} failed: {e:?}"));
        times.push(started.elapsed());
        if i % 25 == 0 {
            let leftovers = fresh_problems(&new, &before);
            assert!(leftovers.is_empty(), "after preview {i}: {leftovers:?}");
        }
    }
    let size_after = dir_bytes(new.root.path());
    let head: Duration = times.iter().take(5).sum::<Duration>() / 5;
    let tail: Duration = times.iter().rev().take(5).sum::<Duration>() / 5;
    eprintln!(
        "{count} previews: first five {head:?} each, last five {tail:?} each, files grew by {} bytes, \
         downloads {}",
        size_after.saturating_sub(size_before),
        counting.downloads.load(Ordering::SeqCst)
    );
    assert!(fresh_problems(&new, &before).is_empty());
    assert!(
        size_after <= size_before + 64 * 1024,
        "{count} previews left {} bytes of files behind",
        size_after.saturating_sub(size_before)
    );
    assert!(
        tail <= head * 3 + Duration::from_millis(100),
        "previews slow down: first five {head:?}, last five {tail:?}"
    );
    assert_eq!(counting.downloads.load(Ordering::SeqCst), count);
    // After all of them the restore still works.
    new.service
        .first_run_restore(&restore_input(source, &f.cloud_name, &f.key), &|_| {})
        .expect("the restore works after many previews");
    assert_books_and_chains(&new);
}

#[test]
fn forty_previews_in_a_row_leave_nothing_behind_and_do_not_slow_down() {
    many_previews(40);
}

#[test]
#[ignore = "slow (a thousand scrypt runs and unpacks); run with --ignored"]
fn a_thousand_previews_in_a_row_leave_nothing_behind_and_do_not_slow_down() {
    many_previews(1000);
}

// ---------------------------------------------------------------------------------------------
// 8. Time: NFR-BAK-12 (restore of 10 years of data under 5 minutes), NFR-REL-4 (30 minutes)
// ---------------------------------------------------------------------------------------------

fn bloat(plain: &Path, megabytes: u64) {
    let conn = Connection::open(plain).unwrap();
    conn.execute_batch("PRAGMA journal_mode = DELETE; PRAGMA synchronous = OFF;")
        .unwrap();
    // Learned word pairs: a table no hash chain and no total looks at, so the copy stays valid.
    let target = megabytes * 1024 * 1024;
    let mut word = 0_u64;
    loop {
        conn.execute_batch("BEGIN").unwrap();
        {
            let mut insert = conn
                .prepare(
                    "INSERT INTO translit_pairs (roman, urdu, kind, use_count, last_used_at, \
                     created_at, created_by, updated_at, updated_by, version) \
                     VALUES (?1, ?2, 'word', 1, '2026-10-10T09:00:00.000Z', \
                     '2026-10-10T09:00:00.000Z', 1, '2026-10-10T09:00:00.000Z', 1, 1)",
                )
                .unwrap();
            for _ in 0..20_000 {
                word += 1;
                let roman = format!("w{:x}{:x}", word.wrapping_mul(0x9E37_79B9_7F4A_7C15), word);
                insert.execute((&roman, format!("لفظ {roman}"))).unwrap();
            }
        }
        conn.execute_batch("COMMIT").unwrap();
        if fs::metadata(plain).map_or(0, |m| m.len()) >= target {
            break;
        }
    }
    drop(conn);
}

/// The restore of a big copy, stage by stage. The copy is the worked day plus learned word pairs
/// up to the size asked for. Chains, counts and balance stay true, so every check really runs.
/// Default size 64 MB; set `MANDI_ROBUST_RESTORE_MB` for more (300 is ten years at normal
/// volume, NFR-SPD-15; 2000 is the NFR-SCL-4 ceiling).
#[test]
#[ignore = "heavy: builds a big copy; run with --ignored --nocapture"]
fn restore_time_for_a_big_copy_against_nfr_bak_12_and_nfr_rel_4() {
    let megabytes: u64 = std::env::var("MANDI_ROBUST_RESTORE_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(64);
    let old = old_computer();
    let name = old.back_up();
    let (_dir, plain) = old.unpack_cloud(&name);
    bloat(&plain, megabytes);
    let plain_bytes = fs::metadata(&plain).unwrap().len();

    let cloud = memory_cloud();
    let made = Instant::now();
    let copy = put_crafted_copy(
        &cloud,
        &mandi_backup::key::BackupKey::parse(&old.printed_key).unwrap(),
        &plain,
        "20261011T090000000Z",
        Some(&old_chain_key()),
        true,
    );
    let bundle_bytes = cloud
        .list("backups/")
        .unwrap()
        .iter()
        .find(|o| o.key.ends_with(".mbk"))
        .map_or(0, |o| o.size);
    eprintln!(
        "copy: {} MB plain, {} MB packed, built in {:?}",
        plain_bytes / 1_000_000,
        bundle_bytes / 1_000_000,
        made.elapsed()
    );
    time_the_flow(&old, cloud, &copy, plain_bytes);
}

/// Lists, previews and restores `copy` on a new computer and reports every stage. NFR-BAK-12: ten
/// years of data restores in under 5 minutes. NFR-REL-4: the whole job, with the install and the
/// typing of the key, in under 30 minutes; the program part is a small slice of it.
fn time_the_flow(old: &Old, cloud: Arc<dyn CloudStore>, copy: &str, plain_bytes: u64) {
    let new = new_computer(cloud);
    let source = RestoreSource::Cloud(old.account());
    let flow = Instant::now();

    let started = Instant::now();
    let listed = new
        .service
        .first_run_list(&FirstRunListInput {
            source: source.clone(),
        })
        .unwrap();
    assert!(!listed.rows.is_empty());
    let t_list = started.elapsed();

    let started = Instant::now();
    new.service
        .first_run_preview(
            &preview_input(source.clone(), Some(copy), &old.printed_key),
            &|_| {},
        )
        .unwrap();
    let t_preview = started.elapsed();

    let stamps: Mutex<Vec<(String, Instant)>> = Mutex::new(Vec::new());
    let started = Instant::now();
    new.service
        .first_run_restore(
            &restore_input(source, copy, &old.printed_key),
            &|p: RestoreProgress| {
                stamps
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push((format!("{:?}", p.stage), Instant::now()));
            },
        )
        .unwrap();
    let t_restore = started.elapsed();
    let mut last_stage = String::new();
    for (stage, at) in stamps.lock().unwrap().iter() {
        if *stage != last_stage {
            eprintln!(
                "  stage {stage:<16} starts {:>8.1?} after the start",
                at.duration_since(started)
            );
            last_stage.clone_from(stage);
        }
    }

    let started = Instant::now();
    assert_books_and_chains(&new);
    let t_verify = started.elapsed();
    let t_flow = flow.elapsed();
    eprintln!(
        "list {t_list:?}, preview {t_preview:?}, restore {t_restore:?}, check the books and both \
         chains again {t_verify:?}, whole flow {t_flow:?} for a copy of {} MB",
        plain_bytes / 1_000_000
    );
    assert!(
        t_restore < Duration::from_secs(300),
        "NFR-BAK-12: restore of {} MB took {t_restore:?}",
        plain_bytes / 1_000_000
    );
    assert!(
        t_flow < Duration::from_secs(25 * 60),
        "NFR-REL-4: list + preview + restore took {t_flow:?}"
    );
}

/// Real postings through the real services (so the journal and the audit log hold real keyed
/// chains of the size asked for), then the restore of that copy. `MANDI_ROBUST_POSTINGS` is the
/// number of receipts (default 2,000; two years at the client's pace is about 20,000 entries and
/// 58,000 lines, NFR-SCL-1 is 3,500,000 lines). The chain checks read every entry, so they are
/// timed apart as well, on the old computer, and the cost per entry is reported.
#[test]
#[ignore = "heavy: thousands of real postings; run with --ignored --nocapture"]
fn restore_time_for_many_real_postings_with_both_chains_against_nfr_bak_12() {
    let postings: usize = std::env::var("MANDI_ROBUST_POSTINGS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2_000);
    let old = old_computer();
    let w = &old.rig.w;
    let building = Instant::now();
    for i in 0..postings {
        let buyer = if i % 2 == 0 { w.akram } else { w.arshad };
        mandi_services::vouchers::save_receipt(
            &w.env.ctx,
            &w.owner,
            &harness::rupees_input(buyer, "100"),
        )
        .unwrap();
        if (i + 1) % 1_000 == 0 {
            eprintln!("  {} postings in {:?}", i + 1, building.elapsed());
        }
    }
    let t_build = building.elapsed();

    // The chain checks on the old computer's own data, apart from any restore.
    let started = Instant::now();
    ledger::verify_journal_chain(&w.env.ctx).expect("the journal chain verifies");
    let t_journal = started.elapsed();
    let started = Instant::now();
    w.env
        .ctx
        .db()
        .read(|conn| verify_audit_chain(conn, &old_chain_key()).map_err(|e| e.to_string()))
        .expect("the audit chain verifies");
    let t_audit = started.elapsed();
    let entries: i64 = w
        .env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |r| r.get(0))
                .map_err(BackupError::from)
        })
        .unwrap();
    let lines: i64 = w
        .env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM journal_lines", [], |r| r.get(0))
                .map_err(BackupError::from)
        })
        .unwrap();
    eprintln!(
        "{entries} journal entries, {lines} lines built in {t_build:?}; journal chain {t_journal:?} \
         ({:.1} us an entry), audit chain {t_audit:?}",
        t_journal.as_micros() as f64 / entries.max(1) as f64
    );

    let made = Instant::now();
    let copy = old.back_up();
    let t_backup = made.elapsed();
    let bundle_bytes = old
        .cloud
        .list("backups/")
        .unwrap()
        .iter()
        .filter(|o| o.key.ends_with(".mbk"))
        .map(|o| o.size)
        .max()
        .unwrap_or(0);
    eprintln!(
        "backup of {entries} entries to every place took {t_backup:?}; the cloud bundle is {} MB",
        bundle_bytes / 1_000_000
    );
    time_the_flow(&old, old.cloud.clone(), &copy, bundle_bytes);
}
