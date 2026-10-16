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

/// Credential Manager service name (Tech Stack 7: the bundle identifier).
pub const KEYRING_SERVICE: &str = "pk.mandisoftware.app";
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
}

impl fmt::Debug for AuditKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuditKey([hidden])")
    }
}

/// Where the key lives. The real one is the system key store; tests use memory.
pub trait KeyStore: Send + Sync {
    /// The key. Creates and stores a new one the first time.
    ///
    /// # Errors
    /// `DbError::KeyStore` when the store cannot be reached.
    fn audit_key(&self) -> Result<AuditKey, DbError>;
}

/// The key lives in memory. For tests.
#[derive(Debug, Clone)]
pub struct MemoryKeyStore(AuditKey);

impl MemoryKeyStore {
    /// A store that holds this key.
    pub fn new(key: AuditKey) -> Self {
        Self(key)
    }
}

impl KeyStore for MemoryKeyStore {
    fn audit_key(&self) -> Result<AuditKey, DbError> {
        Ok(self.0.clone())
    }
}

/// The key lives in Windows Credential Manager (`keyring`), service `pk.mandisoftware.app`,
/// entry `audit-chain-key`.
#[derive(Debug, Clone, Default)]
pub struct KeyringKeyStore;

impl KeyStore for KeyringKeyStore {
    fn audit_key(&self) -> Result<AuditKey, DbError> {
        let store_error = |e: keyring::Error| DbError::KeyStore(e.to_string());
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ENTRY).map_err(store_error)?;
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

    /// The key, read from the store once and kept in memory.
    ///
    /// # Errors
    /// `DbError::KeyStore`.
    pub fn key(&self) -> Result<AuditKey, DbError> {
        let mut cached = self.key.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(key) = cached.as_ref() {
            return Ok(key.clone());
        }
        let key = self.keys.audit_key()?;
        *cached = Some(key.clone());
        Ok(key)
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
