//! File helpers: a hook for faults, a writer that hashes what passes through, atomic file
//! writes, and a streaming checksum (NFR-BAK-4, NFR-BAK-5).
//!
//! A copy is always written under a `.partial` name and renamed when it is whole and checked.
//! So a power cut, a full disk or a pulled USB drive leaves either the old state or a whole new
//! copy, never a half-written file under a real name.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{BackupCode, BackupError};
use crate::model::BackupKind;

/// Size of one chunk when a file is copied or hashed (1 MiB).
pub const CHUNK: usize = 1024 * 1024;
/// The suffix of a file that is not finished.
pub const PARTIAL: &str = "partial";

/// Called before each chunk is written to a destination. A test uses it to break a copy half
/// way, the way a full disk or a pulled USB drive would. The real app uses [`NoHook`].
pub trait IoHook: Send + Sync {
    /// `written` is how many bytes of this file are already written. An `Err` stops the write
    /// like a real device error.
    fn before_write(&self, target: BackupKind, written: u64) -> io::Result<()> {
        let _ = (target, written);
        Ok(())
    }
}

/// Never breaks anything.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoHook;

impl IoHook for NoHook {}

/// A writer that counts and hashes everything written, and asks the hook first.
pub struct TeeWriter<'a, W: Write> {
    inner: W,
    hasher: blake3::Hasher,
    written: u64,
    hook: &'a dyn IoHook,
    target: BackupKind,
}

impl<'a, W: Write> TeeWriter<'a, W> {
    /// Wraps `inner`.
    pub fn new(inner: W, hook: &'a dyn IoHook, target: BackupKind) -> Self {
        Self {
            inner,
            hasher: blake3::Hasher::new(),
            written: 0,
            hook,
            target,
        }
    }

    /// The writer back, the hex checksum of what was written, and the byte count.
    pub fn finish(self) -> (W, String, u64) {
        (
            self.inner,
            self.hasher.finalize().to_hex().to_string(),
            self.written,
        )
    }
}

impl<W: Write> Write for TeeWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hook.before_write(self.target, self.written)?;
        let count = self.inner.write(buf)?;
        if let Some(done) = buf.get(..count) {
            self.hasher.update(done);
        }
        self.written = self
            .written
            .saturating_add(u64::try_from(count).unwrap_or(0));
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// The `.partial` name next to a final path.
pub fn partial_path(final_path: &Path) -> PathBuf {
    let mut name = final_path.as_os_str().to_owned();
    name.push(".");
    name.push(PARTIAL);
    PathBuf::from(name)
}

/// Removes a file if it is there. A missing file is not an error.
pub fn remove_quietly(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != io::ErrorKind::NotFound
    {
        tracing::warn!(path = %path.display(), "cannot remove file: {error}");
    }
}

/// The blake3 checksum of a file, as hex, read in chunks.
///
/// # Errors
/// The I/O error of reading the file.
pub fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; CHUNK];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if let Some(read) = buffer.get(..count) {
            hasher.update(read);
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Copies `source` to `dest` through a `.partial` file: write, flush to disk, rename. On any
/// error the `.partial` file is removed and `dest` is untouched. Returns the checksum of what
/// was written.
///
/// # Errors
/// The I/O error that stopped the copy (a full disk, a removed drive, ...).
pub fn copy_atomic(
    source: &Path,
    dest: &Path,
    hook: &dyn IoHook,
    target: BackupKind,
) -> io::Result<String> {
    let partial = partial_path(dest);
    let result = copy_to(source, &partial, hook, target).and_then(|checksum| {
        fs::rename(&partial, dest)?;
        Ok(checksum)
    });
    if result.is_err() {
        remove_quietly(&partial);
    }
    result
}

fn copy_to(
    source: &Path,
    partial: &Path,
    hook: &dyn IoHook,
    target: BackupKind,
) -> io::Result<String> {
    if let Some(dir) = partial.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut input = File::open(source)?;
    let mut out = TeeWriter::new(File::create(partial)?, hook, target);
    let mut buffer = vec![0_u8; CHUNK];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if let Some(chunk) = buffer.get(..count) {
            out.write_all(chunk)?;
        }
    }
    out.flush()?;
    let (file, checksum, _) = out.finish();
    file.sync_all()?;
    Ok(checksum)
}

/// Writes `text` to `dest` through a `.partial` file and a rename.
///
/// # Errors
/// The I/O error that stopped the write.
pub fn write_atomic(dest: &Path, bytes: &[u8]) -> io::Result<()> {
    let partial = partial_path(dest);
    let result = (|| {
        if let Some(dir) = partial.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut file = File::create(&partial)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&partial, dest)
    })();
    if result.is_err() {
        remove_quietly(&partial);
    }
    result
}

/// Checks that `dir` has room for `needed` more bytes (NFR-REL-10).
///
/// # Errors
/// `ERR_DISK_FULL` when it does not. A probe that cannot answer lets the copy go on: a full
/// disk is found by the write itself.
pub fn require_room(
    probe: &dyn mandi_db::disk::DiskProbe,
    dir: &Path,
    needed: u64,
) -> Result<(), BackupError> {
    // The nearest folder that exists answers for the disk.
    let mut probe_dir = dir;
    while !probe_dir.exists() {
        match probe_dir.parent() {
            Some(parent) => probe_dir = parent,
            None => break,
        }
    }
    match probe.available_bytes(probe_dir) {
        Ok(available) if available < needed => Err(BackupError::new(
            BackupCode::ErrDiskFull,
            format!("{available} bytes free, {needed} needed"),
        )),
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::warn!("cannot read free disk space: {error}");
            Ok(())
        }
    }
}

/// Maps a write error to the code of its target. A full disk is a full disk anywhere.
pub fn write_error(target: BackupKind, error: &io::Error) -> BackupError {
    let full = error.kind() == io::ErrorKind::StorageFull;
    let code = if full {
        BackupCode::ErrDiskFull
    } else {
        match target {
            BackupKind::Usb => BackupCode::ErrBackupUsbFailed,
            BackupKind::Local => BackupCode::ErrBackupLocalFailed,
            BackupKind::Cloud => BackupCode::ErrBackupCloudFailed,
        }
    };
    BackupError::new(code, format!("{target:?} write: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailAfter(u64);

    impl IoHook for FailAfter {
        fn before_write(&self, _: BackupKind, written: u64) -> io::Result<()> {
            if written >= self.0 {
                Err(io::Error::new(io::ErrorKind::NotFound, "drive removed"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn copy_atomic_copies_and_returns_the_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.bin");
        fs::write(&src, vec![7_u8; 3 * CHUNK + 5]).unwrap();
        let dst = dir.path().join("sub").join("b.bin");
        let sum = copy_atomic(&src, &dst, &NoHook, BackupKind::Usb).unwrap();
        assert_eq!(sum, hash_file(&src).unwrap());
        assert_eq!(hash_file(&dst).unwrap(), sum);
        assert!(!partial_path(&dst).exists());
    }

    #[test]
    fn a_broken_copy_leaves_no_file_under_the_real_name() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.bin");
        fs::write(&src, vec![1_u8; 4 * CHUNK]).unwrap();
        let dst = dir.path().join("b.bin");
        fs::write(&dst, b"old").unwrap();
        let error = copy_atomic(&src, &dst, &FailAfter(CHUNK as u64), BackupKind::Usb).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert_eq!(fs::read(&dst).unwrap(), b"old", "the old file is untouched");
        assert!(!partial_path(&dst).exists(), "no half copy is left");
    }

    #[test]
    fn room_check_refuses_when_the_disk_is_too_small() {
        let dir = tempfile::tempdir().unwrap();
        let probe = mandi_db::disk::FixedDiskProbe(10);
        let error = require_room(&probe, &dir.path().join("not-yet"), 11).unwrap_err();
        assert_eq!(error.code, BackupCode::ErrDiskFull);
        require_room(&probe, dir.path(), 10).unwrap();
    }
}
