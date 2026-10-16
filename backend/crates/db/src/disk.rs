//! Free disk space check before every save (NFR-REL-10).
//!
//! The probe is injected, so a test can pretend the disk is full.

use std::io;
use std::path::Path;

/// A save needs at least this much free space (50 MB). A save, its journal and a backup copy
/// are all well under this.
pub const MIN_FREE_DISK_BYTES: u64 = 50 * 1024 * 1024;

/// Tells how much disk space is free where the data file lives.
pub trait DiskProbe: Send + Sync {
    /// Free bytes available to this program on the disk that holds `path`.
    fn available_bytes(&self, path: &Path) -> io::Result<u64>;
}

/// Asks the operating system.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealDiskProbe;

impl DiskProbe for RealDiskProbe {
    fn available_bytes(&self, path: &Path) -> io::Result<u64> {
        fs4::available_space(path)
    }
}

/// Always reports the same free space. For tests.
#[derive(Debug, Clone, Copy)]
pub struct FixedDiskProbe(pub u64);

impl DiskProbe for FixedDiskProbe {
    fn available_bytes(&self, _path: &Path) -> io::Result<u64> {
        Ok(self.0)
    }
}
