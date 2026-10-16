//! Test helpers: fake parts for a [`Context`] and a ready test environment. Public because the
//! integration tests of this crate and of the app use them. Never used by the running app.

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use mandi_db::audit::{AuditKey, MemoryKeyStore};
use mandi_db::clock::ManualClock;
use mandi_db::disk::{DiskProbe, MIN_FREE_DISK_BYTES};
use mandi_db::{Db, DbOptions};

use crate::context::{Context, Parts};
use crate::error::ServiceError;
use crate::ports::{Argon2Hasher, Monotonic, RandomSource};

/// A monotonic clock a test moves by hand. It never goes back by itself.
#[derive(Debug, Default)]
pub struct ManualMonotonic(Mutex<Duration>);

impl ManualMonotonic {
    /// Moves the time on.
    pub fn advance(&self, by: Duration) {
        let mut now = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        *now = now.saturating_add(by);
    }

    /// Moves the time on by whole seconds.
    pub fn advance_seconds(&self, seconds: u64) {
        self.advance(Duration::from_secs(seconds));
    }
}

impl Monotonic for ManualMonotonic {
    fn elapsed(&self) -> Duration {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Random bytes that are the same in every run (a counter run through a simple mixer).
/// Not random at all: for tests only.
#[derive(Debug, Default)]
pub struct CountingRandom(Mutex<u64>);

impl RandomSource for CountingRandom {
    fn fill(&self, buffer: &mut [u8]) -> Result<(), ServiceError> {
        let mut state = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        for byte in buffer.iter_mut() {
            *state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            *byte = u8::try_from((*state >> 33) & 0xFF).unwrap_or(0);
        }
        Ok(())
    }
}

/// A random source that always gives the same byte. For tests only.
#[derive(Debug)]
pub struct ConstantRandom(pub u8);

impl RandomSource for ConstantRandom {
    fn fill(&self, buffer: &mut [u8]) -> Result<(), ServiceError> {
        buffer.fill(self.0);
        Ok(())
    }
}

/// Argon2id with the smallest allowed cost (8 KiB, 1 pass): real hashes, but fast.
/// `None` only if Argon2 refuses those numbers, which it does not.
pub fn fast_hasher() -> Option<Arc<Argon2Hasher<CountingRandom>>> {
    Argon2Hasher::with_cost(CountingRandom::default(), 8, 1)
        .ok()
        .map(Arc::new)
}

/// Free disk space a test can change while the app runs (NFR-REL-10). Starts with plenty.
#[derive(Debug)]
pub struct AdjustableDiskProbe(AtomicU64);

impl AdjustableDiskProbe {
    /// Plenty of space.
    pub const PLENTY: u64 = 10 * 1024 * 1024 * 1024;

    /// Pretends the disk is full: a save is refused until [`Self::make_room`].
    pub fn fill_up(&self) {
        self.0
            .store(MIN_FREE_DISK_BYTES.saturating_sub(1), Ordering::SeqCst);
    }

    /// Pretends there is plenty of space again.
    pub fn make_room(&self) {
        self.0.store(Self::PLENTY, Ordering::SeqCst);
    }
}

impl DiskProbe for AdjustableDiskProbe {
    fn available_bytes(&self, _path: &Path) -> io::Result<u64> {
        Ok(self.0.load(Ordering::SeqCst))
    }
}

/// A start time for the test clock: 10-10-2026, 09:00 UTC (the PRD worked day).
pub fn test_start() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0)
        .single()
        .unwrap_or_default()
}

/// A database in a folder, with a context whose clock and monotonic time a test can move.
pub struct TestEnv {
    /// The context under test.
    pub ctx: Arc<Context>,
    /// The wall clock (also used for audit rows).
    pub clock: Arc<ManualClock>,
    /// The monotonic clock.
    pub mono: Arc<ManualMonotonic>,
    /// The free disk space the database sees.
    pub disk: Arc<AdjustableDiskProbe>,
}

impl TestEnv {
    /// Opens a new database at `<dir>/data/mandi.db` and builds the context.
    ///
    /// # Errors
    /// Whatever opening the database returns, or `ServiceError::Hash` if the test hasher
    /// cannot be made.
    pub fn new(dir: &Path) -> Result<Self, ServiceError> {
        let clock = Arc::new(ManualClock::new(test_start()));
        let disk = Arc::new(AdjustableDiskProbe(AtomicU64::new(
            AdjustableDiskProbe::PLENTY,
        )));
        let options = DbOptions {
            backups_dir: dir.join("backups"),
            disk_probe: disk.clone(),
            clock: clock.clone(),
            key_store: Arc::new(MemoryKeyStore::new(AuditKey::from_bytes([7; 32]))),
            busy_timeout: Duration::from_millis(500),
            check_wait: Duration::from_millis(500),
        };
        let db = Db::open_checked(&dir.join("data").join("mandi.db"), options)?;
        let mono = Arc::new(ManualMonotonic::default());
        let hasher = fast_hasher().ok_or_else(|| ServiceError::Hash("test hasher".to_owned()))?;
        let ctx = Context::with_parts(
            Arc::new(db),
            Parts {
                monotonic: mono.clone(),
                hasher,
                random: Arc::new(CountingRandom::default()),
            },
        );
        Ok(Self {
            ctx: Arc::new(ctx),
            clock,
            mono,
            disk,
        })
    }
}
