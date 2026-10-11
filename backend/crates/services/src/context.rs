//! The context every service runs in: the database, the clock, the password hasher, the random
//! source, the one signed-in session and the lockout memory (CLAUDE.md rule 3, plan 04).
//!
//! The app builds one `Context` at start-up. Tests build their own with fake parts.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::Duration;

use chrono::{DateTime, Utc};
use mandi_db::Db;
use mandi_db::clock::{Clock, format_utc};
use mandi_domain::core::translit::LearnedPair;

use crate::error::ServiceError;
use crate::ports::{
    Argon2Hasher, Monotonic, OsRandom, PasswordHasher, RandomSource, SystemMonotonic,
};
use crate::secret::Secret;

/// The parts of a [`Context`] that tests replace. The clock comes from the `Db`.
pub struct Parts {
    /// Time that only moves forward.
    pub monotonic: Arc<dyn Monotonic>,
    /// Hashes passwords and recovery codes.
    pub hasher: Arc<dyn PasswordHasher>,
    /// Random bytes.
    pub random: Arc<dyn RandomSource>,
}

impl Parts {
    /// The real parts: system monotonic clock, Argon2id with the recommended cost, OS random.
    pub fn real() -> Self {
        Self {
            monotonic: Arc::new(SystemMonotonic::new()),
            hasher: Arc::new(Argon2Hasher::recommended(OsRandom)),
            random: Arc::new(OsRandom),
        }
    }
}

/// The signed-in user. At most one at a time (PRD FR-2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Session {
    pub user_id: i64,
    pub username: String,
    pub locked: bool,
}

/// A lock that began in this run of the app, kept in monotonic time (NFR-REL-11).
#[derive(Debug, Clone, Copy)]
pub(crate) struct MemoryLock {
    /// When the lock ends, in monotonic time.
    pub until: Duration,
}

/// What the app remembers about wrong tries while it runs.
#[derive(Debug, Default)]
pub(crate) struct Lockouts {
    /// Locks by user name. An entry that has run out stays, so a clock moved back later cannot
    /// bring the stored end time back to life.
    pub locks: HashMap<String, MemoryLock>,
    /// Wrong tries of names that are not users, so a wrong name behaves like a wrong password.
    pub unknown_tries: HashMap<String, u32>,
}

/// The most unknown names remembered. More than this are forgotten (memory is bounded).
pub(crate) const MAX_UNKNOWN_NAMES: usize = 10_000;

/// Everything a service needs.
pub struct Context {
    pub(crate) db: Arc<Db>,
    pub(crate) monotonic: Arc<dyn Monotonic>,
    pub(crate) hasher: Arc<dyn PasswordHasher>,
    pub(crate) random: Arc<dyn RandomSource>,
    pub(crate) session: Mutex<Option<Session>>,
    pub(crate) lockouts: Mutex<Lockouts>,
    /// The learned word pairs, kept in memory (Tech Stack 4.4). `None` until first needed.
    pub(crate) learned: Mutex<Option<Arc<Vec<LearnedPair>>>>,
    /// The search index (Tech Stack 4.7), kept in memory. `None` until first needed.
    pub(crate) search: Mutex<Option<Arc<crate::search::Index>>>,
    dummy_hash: OnceLock<String>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context").finish_non_exhaustive()
    }
}

impl Context {
    /// A context with the real parts.
    pub fn new(db: Arc<Db>) -> Self {
        Self::with_parts(db, Parts::real())
    }

    /// A context with chosen parts.
    pub fn with_parts(db: Arc<Db>, parts: Parts) -> Self {
        Self {
            db,
            monotonic: parts.monotonic,
            hasher: parts.hasher,
            random: parts.random,
            session: Mutex::new(None),
            lockouts: Mutex::new(Lockouts::default()),
            learned: Mutex::new(None),
            search: Mutex::new(None),
            dummy_hash: OnceLock::new(),
        }
    }

    /// The database.
    pub fn db(&self) -> &Arc<Db> {
        &self.db
    }

    /// The clock of the database (audit rows use the same one).
    pub fn clock(&self) -> &Arc<dyn Clock> {
        self.db.clock()
    }

    /// Now, in UTC.
    pub fn now_utc(&self) -> DateTime<Utc> {
        self.db.clock().now_utc()
    }

    /// Now, as stored in the database: `2026-10-10T07:30:12.345Z`.
    pub fn now_text(&self) -> String {
        format_utc(self.now_utc())
    }

    pub(crate) fn session_lock(&self) -> MutexGuard<'_, Option<Session>> {
        self.session.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn search_lock(&self) -> MutexGuard<'_, Option<Arc<crate::search::Index>>> {
        self.search.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn learned_lock(&self) -> MutexGuard<'_, Option<Arc<Vec<LearnedPair>>>> {
        self.learned.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn lockouts_lock(&self) -> MutexGuard<'_, Lockouts> {
        self.lockouts.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A copy of the session, if one is open.
    pub(crate) fn session_snapshot(&self) -> Option<Session> {
        self.session_lock().clone()
    }

    /// A hash nobody knows the password of. A sign-in with an unknown name is checked against
    /// it, so it takes as long as a wrong password and does not tell the name is unknown.
    pub(crate) fn dummy_hash(&self) -> Result<&str, ServiceError> {
        if let Some(hash) = self.dummy_hash.get() {
            return Ok(hash);
        }
        let mut noise = [0_u8; 24];
        self.random.fill(&mut noise)?;
        let secret = Secret::new(noise.iter().map(|b| format!("{b:02x}")).collect::<String>());
        let hash = self.hasher.hash(&secret)?;
        Ok(self.dummy_hash.get_or_init(|| hash))
    }
}
