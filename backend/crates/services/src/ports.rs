//! Things the services need from outside, behind small traits so tests can replace them:
//! a monotonic clock, a random source and the password hasher (plan 04 Section 4).

use std::time::{Duration, Instant};

use argon2::password_hash::{PasswordHasher as _, PasswordVerifier as _};
use argon2::{Algorithm, Argon2, Params, Version};

use crate::error::ServiceError;
use crate::secret::Secret;

/// Time that only moves forward, even if the computer's clock is changed. Used for lockouts
/// while the app runs (NFR-REL-11).
pub trait Monotonic: Send + Sync {
    /// Time since an arbitrary start. Never goes back.
    fn elapsed(&self) -> Duration;
}

/// The real monotonic clock.
#[derive(Debug)]
pub struct SystemMonotonic(Instant);

impl SystemMonotonic {
    /// Starts counting now.
    pub fn new() -> Self {
        Self(Instant::now())
    }
}

impl Default for SystemMonotonic {
    fn default() -> Self {
        Self::new()
    }
}

impl Monotonic for SystemMonotonic {
    fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}

/// Random bytes from the operating system (salts, recovery codes).
pub trait RandomSource: Send + Sync {
    /// Fills `buffer` with random bytes.
    ///
    /// # Errors
    /// `ServiceError::Random` if the system has no random source.
    fn fill(&self, buffer: &mut [u8]) -> Result<(), ServiceError>;
}

/// The operating system's random source.
#[derive(Debug, Clone, Copy, Default)]
pub struct OsRandom;

impl RandomSource for OsRandom {
    fn fill(&self, buffer: &mut [u8]) -> Result<(), ServiceError> {
        getrandom::fill(buffer).map_err(|e| ServiceError::Random(e.to_string()))
    }
}

/// Hashes and checks passwords and recovery codes.
pub trait PasswordHasher: Send + Sync {
    /// A hash in the PHC text form (`$argon2id$v=19$m=...$salt$hash`) with a new random salt.
    ///
    /// # Errors
    /// `ServiceError::Hash` or `ServiceError::Random`.
    fn hash(&self, password: &Secret) -> Result<String, ServiceError>;

    /// True if `password` matches `phc`. A hash that cannot be read is a mismatch.
    fn verify(&self, password: &Secret, phc: &str) -> bool;
}

/// Bytes of salt for every hash.
const SALT_BYTES: usize = 16;

/// Argon2id (NFR-SEC-1, Code Standards S2).
pub struct Argon2Hasher<R: RandomSource> {
    argon: Argon2<'static>,
    random: R,
}

impl<R: RandomSource> Argon2Hasher<R> {
    /// The recommended default of the `argon2` crate: Argon2id, version 19, 19 MiB of memory,
    /// 2 passes, 1 lane. About 60 ms on a normal laptop.
    pub fn recommended(random: R) -> Self {
        Self {
            argon: Argon2::default(),
            random,
        }
    }

    /// Argon2id with the given cost. Tests use small numbers to stay fast. Never use this in
    /// the app.
    ///
    /// # Errors
    /// `ServiceError::Hash` if the numbers are not allowed by Argon2.
    pub fn with_cost(random: R, memory_kib: u32, passes: u32) -> Result<Self, ServiceError> {
        let params = Params::new(memory_kib, passes, 1, None)
            .map_err(|e| ServiceError::Hash(e.to_string()))?;
        Ok(Self {
            argon: Argon2::new(Algorithm::Argon2id, Version::V0x13, params),
            random,
        })
    }
}

impl<R: RandomSource> PasswordHasher for Argon2Hasher<R> {
    fn hash(&self, password: &Secret) -> Result<String, ServiceError> {
        let mut salt = [0_u8; SALT_BYTES];
        self.random.fill(&mut salt)?;
        let hash = self
            .argon
            .hash_password_with_salt(password.expose().as_bytes(), &salt)
            .map_err(|e| ServiceError::Hash(e.to_string()))?;
        Ok(hash.to_string())
    }

    fn verify(&self, password: &Secret, phc: &str) -> bool {
        self.argon
            .verify_password(password.expose().as_bytes(), phc)
            .is_ok()
    }
}
