//! The audit writer and its keyed hash chain (plan 03 Section 4.5, PRD FR-21.3, NFR-SEC-5/6).
//!
//! Every audit row carries `chain_hash = blake3::keyed_hash(key, previous_hash | row)`. The key
//! is 32 random bytes kept in Windows Credential Manager (the macOS keychain in development),
//! never in the data file (Code Standards S3). Someone who edits or removes a row outside the
//! app cannot make the chain match again without the key, and [`verify_audit_chain`] finds the
//! first row that does not match.
//!
//! Known limit: taking rows off the END of the log leaves a valid chain. Anchoring the last
//! hash outside the data file is a later step (NFR-SEC-6 check, plan D5).

use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};

use rusqlite::{Connection, params};
use serde_json::Value;
use thiserror::Error;

use crate::clock::{Clock, format_utc};
use crate::error::DbError;

/// Credential Manager entry name of the audit chain key.
pub const KEYRING_ENTRY: &str = "audit-chain-key";
/// Bytes in the key.
pub const KEY_LEN: usize = 32;
/// The "previous hash" of the first audit row: 64 zeros.
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// The secret key of the chain. Its `Debug` text never shows the bytes (NFR-OBS-2).
#[derive(Clone, PartialEq, Eq)]
pub struct AuditKey([u8; KEY_LEN]);

impl AuditKey {
    /// A key from 32 bytes.
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self(bytes)
    }

    /// A new random key from the operating system.
    ///
    /// # Errors
    /// `DbError::KeyStore` if the system has no random source.
    pub fn random() -> Result<Self, DbError> {
        let mut bytes = [0_u8; KEY_LEN];
        getrandom::fill(&mut bytes)
            .map_err(|e| DbError::KeyStore(format!("no random source: {e}")))?;
        Ok(Self(bytes))
    }

    /// The bytes of the key, for the hash functions of this crate. Not public: the key never
    /// leaves the crate.
    pub(crate) fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    fn from_slice(bytes: &[u8]) -> Result<Self, DbError> {
        <[u8; KEY_LEN]>::try_from(bytes)
            .map(Self)
            .map_err(|_| DbError::KeyStore("the stored audit key has the wrong length".to_owned()))
    }

    /// The key as 64 lower-case hex characters. It exists for ONE use: the encrypted bundle of a
    /// USB or cloud copy carries the key, locked by the backup key, so a restore on a new
    /// computer can verify the chains (PRD FR-20.11, NFR-SEC-6). The text must never reach a
    /// log, the data file, a manifest or any plain file.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// Reads a key written by [`AuditKey::to_hex`].
    ///
    /// # Errors
    /// `DbError::KeyStore` when the text is not 64 hex characters.
    pub fn from_hex(text: &str) -> Result<Self, DbError> {
        let bad = || DbError::KeyStore("the audit key in the copy has the wrong form".to_owned());
        let bytes = text.as_bytes();
        if bytes.len() != KEY_LEN * 2 {
            return Err(bad());
        }
        let mut key = [0_u8; KEY_LEN];
        for (slot, pair) in key.iter_mut().zip(bytes.chunks(2)) {
            let digit = |c: u8| match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                b'A'..=b'F' => Some(c - b'A' + 10),
                _ => None,
            };
            let (Some(high), Some(low)) = (
                pair.first().copied().and_then(digit),
                pair.get(1).copied().and_then(digit),
            ) else {
                return Err(bad());
            };
            *slot = high << 4 | low;
        }
        Ok(Self(key))
    }

    /// True when both keys are the same bytes.
    pub fn same_as(&self, other: &AuditKey) -> bool {
        self.0 == other.0
    }
}

impl fmt::Debug for AuditKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuditKey([hidden])")
    }
}

impl Drop for AuditKey {
    /// Clears the bytes when the key goes away. Best effort: there is no `zeroize` crate in this
    /// workspace (no new dependency, plan 75 Section 7.8) and `unsafe` is forbidden, so the write is
    /// kept from being optimised away with `black_box`.
    fn drop(&mut self) {
        self.0 = [0; KEY_LEN];
        std::hint::black_box(&self.0);
    }
}

/// Where the key lives. The real one is the system key store; tests use memory.
pub trait KeyStore: Send + Sync {
    /// The key. Creates and stores a new one the first time.
    ///
    /// # Errors
    /// `DbError::KeyStore` when the store cannot be reached.
    fn audit_key(&self) -> Result<AuditKey, DbError>;

    /// Stores `key` in place of the one that is there. A restore on a new computer calls this,
    /// because the restored chains were made with the key of the old computer (PRD FR-20.11).
    ///
    /// # Errors
    /// `DbError::KeyStore` when the store cannot be written, or cannot be changed at all (the
    /// default: a read-only store).
    fn set_audit_key(&self, _key: &AuditKey) -> Result<(), DbError> {
        Err(DbError::KeyStore(
            "this key store cannot be changed".to_owned(),
        ))
    }
}

/// The key lives in memory. For tests.
#[derive(Debug, Clone)]
pub struct MemoryKeyStore(Arc<Mutex<AuditKey>>);

impl MemoryKeyStore {
    /// A store that holds this key.
    pub fn new(key: AuditKey) -> Self {
        Self(Arc::new(Mutex::new(key)))
    }
}

impl KeyStore for MemoryKeyStore {
    fn audit_key(&self) -> Result<AuditKey, DbError> {
        Ok(self
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone())
    }

    fn set_audit_key(&self, key: &AuditKey) -> Result<(), DbError> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = key.clone();
        Ok(())
    }
}

/// The key lives in Windows Credential Manager (`keyring`), service `pk.mandisoftware.app`
/// (with a dot and the name added under a developer profile, `mandi_core::profile`), entry
/// `audit-chain-key`.
#[derive(Debug, Clone, Default)]
pub struct KeyringKeyStore;

impl KeyringKeyStore {
    /// The Credential Manager service name this store uses: the real app's, or the one of the
    /// developer profile (`MANDI_PROFILE`, plan 56 R5).
    pub fn service() -> String {
        mandi_core::profile::keyring_service()
    }
}

/// A key store error as the app sees it. A refusal of the store itself (a platform failure, such
/// as a dismissed or mistyped Keychain box, or no access to the store) is `KeyStoreDenied`, which
/// the user can fix by allowing the app. Any other failure is `KeyStore` (plan 75 Section 7.8).
fn store_error(error: keyring::Error) -> DbError {
    match &error {
        keyring::Error::PlatformFailure(_) | keyring::Error::NoStorageAccess(_) => {
            DbError::KeyStoreDenied(error.to_string())
        }
        _ => DbError::KeyStore(error.to_string()),
    }
}

impl KeyStore for KeyringKeyStore {
    fn audit_key(&self) -> Result<AuditKey, DbError> {
        let entry =
            keyring::Entry::new(&KeyringKeyStore::service(), KEYRING_ENTRY).map_err(store_error)?;
        match entry.get_secret() {
            Ok(bytes) => AuditKey::from_slice(&bytes),
            Err(keyring::Error::NoEntry) => {
                let key = AuditKey::random()?;
                entry.set_secret(&key.0).map_err(store_error)?;
                Ok(key)
            }
            Err(error) => Err(store_error(error)),
        }
    }

    fn set_audit_key(&self, key: &AuditKey) -> Result<(), DbError> {
        keyring::Entry::new(&KeyringKeyStore::service(), KEYRING_ENTRY)
            .map_err(store_error)?
            .set_secret(&key.0)
            .map_err(store_error)
    }
}

/// One thing that happened, to be written to the log (PRD FR-21.3).
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEntry {
    /// Who did it. `None` for the system, or a failed login of an unknown name.
    pub user_id: Option<i64>,
    /// What was done, for example `create`, `edit`, `login_failed`.
    pub action: String,
    /// The kind of record, for example `account`.
    pub entity: String,
    /// The record, if there is one.
    pub entity_id: Option<i64>,
    /// The value before.
    pub old_json: Option<Value>,
    /// The value after.
    pub new_json: Option<Value>,
    /// Why, when the action needs a reason (cancel, reopen).
    pub reason: Option<String>,
}

/// Writes audit rows. One per app: it holds the key store and the clock.
pub struct Auditor {
    keys: Arc<dyn KeyStore>,
    clock: Arc<dyn Clock>,
    key: Mutex<Option<AuditKey>>,
}

impl fmt::Debug for Auditor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Auditor")
    }
}

impl Auditor {
    /// An auditor with this key store and clock.
    pub fn new(keys: Arc<dyn KeyStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            keys,
            clock,
            key: Mutex::new(None),
        }
    }

    /// The key, read from the store once per app run and kept in memory (plan 75 Section 7.8: one
    /// Keychain approval covers the whole run, not every sign-in). It is never written to disk or
    /// to a log. A refusal is not kept, so the next call asks the store again and the user can
    /// allow the app then.
    ///
    /// # Errors
    /// `DbError::KeyStore`, or `DbError::KeyStoreDenied` when the store refused the app.
    pub fn key(&self) -> Result<AuditKey, DbError> {
        let mut cached = self.key.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(key) = cached.as_ref() {
            return Ok(key.clone());
        }
        let key = self.keys.audit_key()?;
        *cached = Some(key.clone());
        Ok(key)
    }

    /// Stores `key` in the key store and keeps it in memory instead of the one used so far.
    /// A restore calls this when the restored chains need the key of another computer (PRD
    /// FR-20.11). If the store refuses, nothing changes.
    ///
    /// # Errors
    /// `DbError::KeyStore`.
    pub fn install_key(&self, key: AuditKey) -> Result<(), DbError> {
        let mut cached = self.key.lock().unwrap_or_else(PoisonError::into_inner);
        self.keys.set_audit_key(&key)?;
        *cached = Some(key);
        Ok(())
    }

    /// Adds one audit row inside the caller's transaction (Code Standards S7) and returns its
    /// id. `tx` is the transaction of `Db::write` (it derefs to `Connection`).
    ///
    /// # Errors
    /// `DbError` when the key or the database fails.
    pub fn write(&self, tx: &Connection, entry: &AuditEntry) -> Result<i64, DbError> {
        let key = self.key()?;
        let at = format_utc(self.clock.now_utc());
        let old_text = entry.old_json.as_ref().map(Value::to_string);
        let new_text = entry.new_json.as_ref().map(Value::to_string);
        let previous: String = tx
            .query_row(
                "SELECT chain_hash FROM audit_log ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(GENESIS_HASH.to_owned()),
                other => Err(other),
            })?;
        let fields = Fields {
            at: &at,
            user_id: entry.user_id,
            action: &entry.action,
            entity: &entry.entity,
            entity_id: entry.entity_id,
            old_json: old_text.as_deref(),
            new_json: new_text.as_deref(),
            reason: entry.reason.as_deref(),
        };
        let chain_hash = chain_hash(&key, &previous, &fields)?;
        tx.execute(
            "INSERT INTO audit_log (created_by, updated_by, at, user_id, action, entity, entity_id,
                 old_json, new_json, reason, chain_hash)
             VALUES (?1, ?1, ?2, ?1, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry.user_id,
                at,
                entry.action,
                entry.entity,
                entry.entity_id,
                old_text,
                new_text,
                entry.reason,
                chain_hash
            ],
        )?;
        Ok(tx.last_insert_rowid())
    }
}

/// The audit row without its hash, as it goes into the hash.
struct Fields<'a> {
    at: &'a str,
    user_id: Option<i64>,
    action: &'a str,
    entity: &'a str,
    entity_id: Option<i64>,
    old_json: Option<&'a str>,
    new_json: Option<&'a str>,
    reason: Option<&'a str>,
}

/// The row as one JSON array in a fixed order. An array, not an object, so the text never
/// depends on how a JSON library orders keys.
fn canonical_json(fields: &Fields<'_>) -> Result<String, DbError> {
    let parts = [
        serde_json::to_string(fields.at)?,
        serde_json::to_string(&fields.user_id)?,
        serde_json::to_string(fields.action)?,
        serde_json::to_string(fields.entity)?,
        serde_json::to_string(&fields.entity_id)?,
        serde_json::to_string(&fields.old_json)?,
        serde_json::to_string(&fields.new_json)?,
        serde_json::to_string(&fields.reason)?,
    ];
    Ok(format!("[{}]", parts.join(",")))
}

/// `blake3::keyed_hash(key, previous_hash | canonical_json(row))` as 64 hex characters.
fn chain_hash(key: &AuditKey, previous: &str, fields: &Fields<'_>) -> Result<String, DbError> {
    let mut input = Vec::new();
    input.extend_from_slice(previous.as_bytes());
    input.push(b'|');
    input.extend_from_slice(canonical_json(fields)?.as_bytes());
    Ok(blake3::keyed_hash(&key.0, &input).to_hex().to_string())
}

/// The first audit row that does not match its hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("audit chain broken at row {at_id}")]
pub struct ChainBreak {
    /// The id of the first row whose hash is wrong. If a row was removed, this is the row
    /// after the gap.
    pub at_id: i64,
}

/// Why [`verify_audit_chain`] did not say "good".
#[derive(Debug, Error)]
pub enum VerifyError {
    /// A row was changed or removed.
    #[error(transparent)]
    Broken(#[from] ChainBreak),
    /// The log could not be read.
    #[error(transparent)]
    Db(#[from] DbError),
}

/// Walks the whole audit log and checks every hash (NFR-SEC-6).
///
/// # Errors
/// `VerifyError::Broken` with the first bad row, or `VerifyError::Db` when the log cannot be read.
pub fn verify_audit_chain(conn: &Connection, key: &AuditKey) -> Result<(), VerifyError> {
    let mut statement = conn
        .prepare(
            "SELECT id, at, user_id, action, entity, entity_id, old_json, new_json, reason, chain_hash
             FROM audit_log ORDER BY id",
        )
        .map_err(DbError::from)?;
    let mut rows = statement.query([]).map_err(DbError::from)?;
    let mut previous = GENESIS_HASH.to_owned();
    while let Some(row) = rows.next().map_err(DbError::from)? {
        let id: i64 = row.get(0).map_err(DbError::from)?;
        let at: String = row.get(1).map_err(DbError::from)?;
        let user_id: Option<i64> = row.get(2).map_err(DbError::from)?;
        let action: String = row.get(3).map_err(DbError::from)?;
        let entity: String = row.get(4).map_err(DbError::from)?;
        let entity_id: Option<i64> = row.get(5).map_err(DbError::from)?;
        let old_json: Option<String> = row.get(6).map_err(DbError::from)?;
        let new_json: Option<String> = row.get(7).map_err(DbError::from)?;
        let reason: Option<String> = row.get(8).map_err(DbError::from)?;
        let stored: String = row.get(9).map_err(DbError::from)?;
        let fields = Fields {
            at: &at,
            user_id,
            action: &action,
            entity: &entity,
            entity_id,
            old_json: old_json.as_deref(),
            new_json: new_json.as_deref(),
            reason: reason.as_deref(),
        };
        if chain_hash(key, &previous, &fields)? != stored {
            return Err(ChainBreak { at_id: id }.into());
        }
        previous = stored;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_round_trips_through_hex() {
        let key = AuditKey::from_bytes([
            0, 1, 2, 3, 250, 251, 252, 253, 254, 255, 16, 32, 64, 128, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        ]);
        let text = key.to_hex();
        assert_eq!(text.len(), 64);
        assert!(text.starts_with("00010203fafbfcfd"));
        assert!(AuditKey::from_hex(&text).unwrap().same_as(&key));
        assert!(
            AuditKey::from_hex(&text.to_uppercase())
                .unwrap()
                .same_as(&key)
        );
    }

    #[test]
    fn a_bad_hex_text_is_refused() {
        for bad in ["", "00", &"0".repeat(63), &"0".repeat(65), &"g".repeat(64)] {
            assert!(AuditKey::from_hex(bad).is_err(), "{bad}");
        }
        let text = format!("{:?}", AuditKey::from_bytes([1; 32]));
        assert!(!text.contains("01"), "Debug never shows the key");
    }

    /// A store that counts its reads and can refuse them.
    struct CountingStore {
        reads: std::sync::atomic::AtomicUsize,
        refuse: std::sync::atomic::AtomicBool,
    }

    impl CountingStore {
        fn new(refuse: bool) -> Arc<Self> {
            Arc::new(Self {
                reads: std::sync::atomic::AtomicUsize::new(0),
                refuse: std::sync::atomic::AtomicBool::new(refuse),
            })
        }

        fn reads(&self) -> usize {
            self.reads.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl KeyStore for CountingStore {
        fn audit_key(&self) -> Result<AuditKey, DbError> {
            self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if self.refuse.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(DbError::KeyStoreDenied("refused".to_owned()));
            }
            Ok(AuditKey::from_bytes([5; 32]))
        }
    }

    fn auditor_with(store: Arc<CountingStore>) -> Auditor {
        Auditor::new(
            store,
            Arc::new(crate::clock::ManualClock::new(chrono::Utc::now())),
        )
    }

    #[test]
    fn the_key_is_read_from_the_store_once_per_run() {
        let store = CountingStore::new(false);
        let auditor = auditor_with(store.clone());
        for _ in 0..5 {
            assert!(
                auditor
                    .key()
                    .unwrap()
                    .same_as(&AuditKey::from_bytes([5; 32]))
            );
        }
        assert_eq!(store.reads(), 1, "one read covers the whole run");
    }

    #[test]
    fn a_refusal_gives_its_own_code_and_is_not_kept() {
        use std::sync::atomic::Ordering;
        let store = CountingStore::new(true);
        let auditor = auditor_with(store.clone());
        let error = auditor.key().unwrap_err();
        assert_eq!(
            error.code(),
            mandi_core::error::ErrorCode::ErrKeystoreDenied,
            "{error}"
        );
        // The refusal is not cached: once the user allows the app, the next call works.
        store.refuse.store(false, Ordering::SeqCst);
        assert!(auditor.key().is_ok());
        assert_eq!(store.reads(), 2);
        assert!(auditor.key().is_ok());
        assert_eq!(store.reads(), 2, "and then it is kept");
    }

    #[test]
    fn a_platform_failure_or_no_access_is_a_refusal_and_other_failures_are_not() {
        let platform = |text: &str| {
            keyring::Error::PlatformFailure(Box::new(std::io::Error::other(text.to_owned())))
        };
        let access = keyring::Error::NoStorageAccess(Box::new(std::io::Error::other("locked")));
        for refused in [
            store_error(platform(
                "The user name or passphrase you entered is not correct.",
            )),
            store_error(access),
        ] {
            assert!(matches!(refused, DbError::KeyStoreDenied(_)), "{refused}");
            assert_eq!(
                refused.code(),
                mandi_core::error::ErrorCode::ErrKeystoreDenied
            );
        }
        let other = store_error(keyring::Error::BadStoreFormat("x".to_owned()));
        assert!(matches!(other, DbError::KeyStore(_)), "{other}");
        assert_eq!(other.code(), mandi_core::error::ErrorCode::ErrInternal);
    }

    #[test]
    fn installing_a_key_changes_the_store_and_the_key_used() {
        let old = AuditKey::from_bytes([1; 32]);
        let new = AuditKey::from_bytes([2; 32]);
        let store = Arc::new(MemoryKeyStore::new(old.clone()));
        let auditor = Auditor::new(
            store.clone(),
            Arc::new(crate::clock::ManualClock::new(chrono::Utc::now())),
        );
        assert!(auditor.key().unwrap().same_as(&old));
        auditor.install_key(new.clone()).unwrap();
        assert!(auditor.key().unwrap().same_as(&new));
        assert!(store.audit_key().unwrap().same_as(&new));
    }
}
