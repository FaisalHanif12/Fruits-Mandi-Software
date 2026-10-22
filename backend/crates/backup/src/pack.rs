//! The encrypted bundle of USB and cloud copies (PRD FR-20.10, NFR-BAK-6, Architecture 14).
//!
//! A bundle is one file: the plain bytes `MANDIBK1 | length of the manifest | manifest (JSON) |
//! the SQLite file`, compressed with zstd and encrypted with age and the backup key. Nobody
//! without the key can read the books or even the totals. Opening a bundle checks the
//! checksum of the database it holds.

use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::iter;
use std::path::Path;

use age::{DecryptError, Decryptor, Encryptor};

use crate::error::{BackupCode, BackupError};
use crate::io::{CHUNK, IoHook, TeeWriter, partial_path, remove_quietly, write_error};
use crate::key::BackupKey;
use crate::model::{BackupKind, Embedded};

/// The first bytes of what a bundle holds.
const MAGIC: &[u8; 8] = b"MANDIBK1";
/// The most a manifest may take. A bigger one is not ours.
const MANIFEST_LIMIT: usize = 1024 * 1024;
/// scrypt work factor (`N = 2^15`, 32 MiB). The key has 120 bits of chance, so a slow guess
/// check adds nothing, and a backup must not wait seconds for it on a small computer.
pub const WORK_FACTOR: u8 = 15;
/// zstd level 3: fast and about as small as the data allows.
const ZSTD_LEVEL: i32 = 3;

/// What `seal` made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sealed {
    /// Checksum of the bundle file.
    pub bundle_blake3: String,
    /// Size of the bundle file.
    pub bundle_size: u64,
}

fn corrupt(detail: impl std::fmt::Display) -> BackupError {
    BackupError::new(BackupCode::ErrBackupCorrupt, detail)
}

/// Compresses and encrypts `plain_db` with its manifest into `dest`, through a `.partial` file.
///
/// # Errors
/// `ERR_DISK_FULL` or the code of `target` (`ERR_BACKUP_USB_FAILED` ...) when a write fails;
/// `ERR_INTERNAL` otherwise. `dest` is whole or absent.
pub fn seal(
    plain_db: &Path,
    embedded: &Embedded,
    key: &BackupKey,
    dest: &Path,
    hook: &dyn IoHook,
    target: BackupKind,
) -> Result<Sealed, BackupError> {
    let partial = partial_path(dest);
    let result = seal_to(plain_db, embedded, key, &partial, hook, target).and_then(|sealed| {
        fs::rename(&partial, dest).map_err(|e| write_error(target, &e))?;
        Ok(sealed)
    });
    if result.is_err() {
        remove_quietly(&partial);
    }
    result
}

fn seal_to(
    plain_db: &Path,
    embedded: &Embedded,
    key: &BackupKey,
    partial: &Path,
    hook: &dyn IoHook,
    target: BackupKind,
) -> Result<Sealed, BackupError> {
    let fail = |e: std::io::Error| write_error(target, &e);
    if let Some(dir) = partial.parent() {
        fs::create_dir_all(dir).map_err(fail)?;
    }
    let manifest = serde_json::to_vec(embedded)?;
    let manifest_len =
        u32::try_from(manifest.len()).map_err(|_| BackupError::internal("manifest is too big"))?;
    let mut recipient = age::scrypt::Recipient::new(key.passphrase());
    recipient.set_work_factor(WORK_FACTOR);
    let encryptor = Encryptor::with_recipients(iter::once(&recipient as &dyn age::Recipient))
        .map_err(|e| BackupError::internal(format!("encryptor: {e}")))?;
    let file = File::create(partial).map_err(fail)?;
    let encrypted = encryptor
        .wrap_output(TeeWriter::new(file, hook, target))
        .map_err(fail)?;
    let mut compressed = zstd::stream::write::Encoder::new(encrypted, ZSTD_LEVEL).map_err(fail)?;
    compressed.write_all(MAGIC).map_err(fail)?;
    compressed
        .write_all(&manifest_len.to_le_bytes())
        .map_err(fail)?;
    compressed.write_all(&manifest).map_err(fail)?;
    let mut input = File::open(plain_db).map_err(|e| BackupError::io("snapshot", &e))?;
    let mut buffer = vec![0_u8; CHUNK];
    loop {
        let count = input
            .read(&mut buffer)
            .map_err(|e| BackupError::io("snapshot", &e))?;
        if count == 0 {
            break;
        }
        if let Some(chunk) = buffer.get(..count) {
            compressed.write_all(chunk).map_err(fail)?;
        }
    }
    let encrypted = compressed.finish().map_err(fail)?;
    let tee = encrypted.finish().map_err(fail)?;
    let (file, bundle_blake3, bundle_size) = tee.finish();
    file.sync_all().map_err(fail)?;
    Ok(Sealed {
        bundle_blake3,
        bundle_size,
    })
}

/// Decrypts and unpacks a bundle: writes the SQLite file to `out_db` (through a `.partial`
/// file) and returns the manifest inside. The checksum of the database is checked against the
/// manifest.
///
/// # Errors
/// `ERR_BACKUP_KEY_WRONG` (field `key`) when the key does not open the bundle;
/// `ERR_BACKUP_CORRUPT` when the bundle is damaged, truncated, not ours, or its database does
/// not match its checksum; `ERR_BACKUP_NOT_FOUND` when the file is not there. Nothing is
/// written to `out_db` unless everything is right.
pub fn open(bundle: &Path, key: &BackupKey, out_db: &Path) -> Result<Embedded, BackupError> {
    let partial = partial_path(out_db);
    let result = open_to(bundle, key, &partial).and_then(|embedded| {
        fs::rename(&partial, out_db).map_err(|e| BackupError::io("unpack", &e))?;
        Ok(embedded)
    });
    if result.is_err() {
        remove_quietly(&partial);
    }
    result
}

fn map_decrypt(error: DecryptError) -> BackupError {
    match error {
        DecryptError::DecryptionFailed
        | DecryptError::NoMatchingKeys
        | DecryptError::KeyDecryptionFailed => BackupError::new(
            BackupCode::ErrBackupKeyWrong,
            "the key does not open the backup",
        )
        .on_field("key"),
        other => corrupt(format!("age: {other}")),
    }
}

fn open_to(bundle: &Path, key: &BackupKey, partial: &Path) -> Result<Embedded, BackupError> {
    let file = File::open(bundle).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            BackupError::new(BackupCode::ErrBackupNotFound, "bundle is not there")
        } else {
            BackupError::io("bundle", &e)
        }
    })?;
    let decryptor = Decryptor::new(BufReader::new(file)).map_err(map_decrypt)?;
    if !decryptor.is_scrypt() {
        return Err(corrupt("the bundle is not locked with a key"));
    }
    let identity = age::scrypt::Identity::new(key.passphrase());
    let reader = decryptor
        .decrypt(iter::once(&identity as &dyn age::Identity))
        .map_err(map_decrypt)?;
    let mut plain =
        zstd::stream::read::Decoder::new(reader).map_err(|e| corrupt(format!("zstd: {e}")))?;

    let mut magic = [0_u8; 8];
    plain
        .read_exact(&mut magic)
        .map_err(|e| corrupt(format!("header: {e}")))?;
    if &magic != MAGIC {
        return Err(corrupt("not a Mandi bundle"));
    }
    let mut len_bytes = [0_u8; 4];
    plain
        .read_exact(&mut len_bytes)
        .map_err(|e| corrupt(format!("header: {e}")))?;
    let len = usize::try_from(u32::from_le_bytes(len_bytes)).unwrap_or(usize::MAX);
    if len > MANIFEST_LIMIT {
        return Err(corrupt("the manifest is too big"));
    }
    let mut manifest = vec![0_u8; len];
    plain
        .read_exact(&mut manifest)
        .map_err(|e| corrupt(format!("manifest: {e}")))?;
    let embedded: Embedded =
        serde_json::from_slice(&manifest).map_err(|e| corrupt(format!("manifest: {e}")))?;

    if let Some(dir) = partial.parent() {
        fs::create_dir_all(dir).map_err(|e| BackupError::io("unpack", &e))?;
    }
    let hook = crate::io::NoHook;
    let mut out = TeeWriter::new(
        File::create(partial).map_err(|e| BackupError::io("unpack", &e))?,
        &hook,
        BackupKind::Local,
    );
    let mut buffer = vec![0_u8; CHUNK];
    loop {
        let count = plain
            .read(&mut buffer)
            .map_err(|e| corrupt(format!("data: {e}")))?;
        if count == 0 {
            break;
        }
        if let Some(chunk) = buffer.get(..count) {
            out.write_all(chunk)
                .map_err(|e| write_error(BackupKind::Local, &e))?;
        }
    }
    out.flush()
        .map_err(|e| write_error(BackupKind::Local, &e))?;
    let (file, checksum, size) = out.finish();
    file.sync_all()
        .map_err(|e| write_error(BackupKind::Local, &e))?;
    if checksum != embedded.inner.plain_blake3 || size != embedded.inner.plain_size {
        return Err(corrupt("the database inside does not match its checksum"));
    }
    Ok(embedded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BackupTotals, InnerManifest, MANIFEST_FORMAT};
    use mandi_core::money::Paisa;
    use std::collections::BTreeMap;

    fn embedded_for(plain: &Path) -> Embedded {
        Embedded {
            format: MANIFEST_FORMAT,
            created_at: "2026-10-10T09:00:00.000Z".into(),
            app_version: "test".into(),
            schema_version: 6,
            inner: InnerManifest {
                plain_blake3: crate::io::hash_file(plain).unwrap(),
                plain_size: fs::metadata(plain).unwrap().len(),
                counts: BTreeMap::new(),
                totals: BackupTotals {
                    accounts: 1,
                    bikri_entries: 0,
                    vouchers: 0,
                    journal_entries: 0,
                    audit_rows: 0,
                    last_entry_date: None,
                    debit_paisa: Paisa::ZERO,
                    credit_paisa: Paisa::ZERO,
                    cash_paisa: Paisa::ZERO,
                },
            },
        }
    }

    fn sample(dir: &Path) -> std::path::PathBuf {
        let plain = dir.join("plain.db");
        let data: Vec<u8> = (0..3 * CHUNK + 17)
            .map(|i| u8::try_from(i % 251).unwrap_or(0))
            .collect();
        fs::write(&plain, data).unwrap();
        plain
    }

    #[test]
    fn a_bundle_round_trips_and_is_not_readable_without_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let plain = sample(dir.path());
        let key = BackupKey::generate().unwrap();
        let bundle = dir.path().join("b.mbk");
        let embedded = embedded_for(&plain);
        let sealed = seal(
            &plain,
            &embedded,
            &key,
            &bundle,
            &crate::io::NoHook,
            BackupKind::Usb,
        )
        .unwrap();
        assert_eq!(sealed.bundle_blake3, crate::io::hash_file(&bundle).unwrap());
        let raw = fs::read(&bundle).unwrap();
        assert!(
            !raw.windows(8).any(|w| w == MAGIC),
            "the manifest marker is not visible"
        );
        let out = dir.path().join("out.db");
        let back = open(&bundle, &key, &out).unwrap();
        assert_eq!(back, embedded);
        assert_eq!(fs::read(&out).unwrap(), fs::read(&plain).unwrap());
    }

    #[test]
    fn the_wrong_key_changes_nothing_and_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let plain = sample(dir.path());
        let key = BackupKey::generate().unwrap();
        let bundle = dir.path().join("b.mbk");
        seal(
            &plain,
            &embedded_for(&plain),
            &key,
            &bundle,
            &crate::io::NoHook,
            BackupKind::Cloud,
        )
        .unwrap();
        let out = dir.path().join("out.db");
        let error = open(&bundle, &BackupKey::generate().unwrap(), &out).unwrap_err();
        assert_eq!(error.code, BackupCode::ErrBackupKeyWrong);
        assert_eq!(error.field.as_deref(), Some("key"));
        assert!(!out.exists() && !partial_path(&out).exists());
    }

    #[test]
    fn a_damaged_or_cut_bundle_is_corrupt_and_writes_no_database() {
        let dir = tempfile::tempdir().unwrap();
        let plain = sample(dir.path());
        let key = BackupKey::generate().unwrap();
        let bundle = dir.path().join("b.mbk");
        seal(
            &plain,
            &embedded_for(&plain),
            &key,
            &bundle,
            &crate::io::NoHook,
            BackupKind::Cloud,
        )
        .unwrap();
        let good = fs::read(&bundle).unwrap();
        let out = dir.path().join("out.db");

        let mut flipped = good.clone();
        let middle = flipped.len() / 2;
        if let Some(byte) = flipped.get_mut(middle) {
            *byte ^= 0xFF;
        }
        fs::write(&bundle, &flipped).unwrap();
        assert_eq!(
            open(&bundle, &key, &out).unwrap_err().code,
            BackupCode::ErrBackupCorrupt
        );

        fs::write(&bundle, good.get(..good.len() - 40).unwrap()).unwrap();
        assert_eq!(
            open(&bundle, &key, &out).unwrap_err().code,
            BackupCode::ErrBackupCorrupt
        );

        fs::write(&bundle, b"not a bundle at all").unwrap();
        assert_eq!(
            open(&bundle, &key, &out).unwrap_err().code,
            BackupCode::ErrBackupCorrupt
        );
        assert!(!out.exists() && !partial_path(&out).exists());
    }

    #[test]
    fn a_failed_write_leaves_no_bundle() {
        struct Break;
        impl IoHook for Break {
            fn before_write(&self, _: BackupKind, written: u64) -> std::io::Result<()> {
                if written > 0 {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "usb removed",
                    ))
                } else {
                    Ok(())
                }
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let plain = sample(dir.path());
        let key = BackupKey::generate().unwrap();
        let bundle = dir.path().join("b.mbk");
        let error = seal(
            &plain,
            &embedded_for(&plain),
            &key,
            &bundle,
            &Break,
            BackupKind::Usb,
        )
        .unwrap_err();
        assert_eq!(error.code, BackupCode::ErrBackupUsbFailed);
        assert!(!bundle.exists() && !partial_path(&bundle).exists());
    }
}
