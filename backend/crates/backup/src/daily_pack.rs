//! The readable daily pack (PRD FR-20.5, NFR-BAK-13): the روزنامچہ PDF and spreadsheets that
//! are kept with each cloud copy, so the books can be read without the app.
//!
//! Plan 18 builds the place for it and the upload. The files themselves come from a
//! [`PackSource`]:
//! - the روزنامچہ PDF needs the report engine (plan 20, R11) and WebView2 printing, which live in
//!   the app crate, so the app gives a source that hands over a PDF;
//! - the spreadsheets come from the report engine too. They are built from the snapshot of the
//!   very copy the pack travels with (see [`PackSource::build`]), so the pack and the backup
//!   always hold the same numbers.
//!
//! Until a source exists, [`PendingPack`] hands over nothing and the status says
//! `pack_pending`. Pack files are encrypted with the backup key like every cloud object (NFR-BAK-6);
//! they open with any age tool and the printed key, so they are readable without this app.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::iter;
use std::path::{Path, PathBuf};

use age::Encryptor;

use crate::error::BackupError;
use crate::io::{CHUNK, IoHook, TeeWriter, partial_path, remove_quietly, write_error};
use crate::key::BackupKey;
use crate::model::BackupKind;
use crate::pack::WORK_FACTOR;

/// What kind of file a pack file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackPart {
    /// The روزنامچہ PDF of the day (R11).
    JournalPdf,
    /// A spreadsheet (buyers, sellers, دکان خرچہ, ذاتی خرچہ, partner shares).
    Sheet,
}

/// One file of the pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFile {
    /// Which part it is.
    pub part: PackPart,
    /// The file name inside the pack folder, for example `journal-2026-10-10.pdf`.
    pub name: String,
    /// Where the file is now.
    pub path: PathBuf,
}

/// Makes the files of the pack for one day.
pub trait PackSource: Send + Sync {
    /// Writes the files for `day` (`YYYY-MM-DD`) into `out_dir` and lists them. `snapshot` is
    /// the SQLite file of the backup copy the pack travels with, unpacked for this call and
    /// removed after it: build every number from it, never from the live books, which may have
    /// changed since the copy was taken. Open it read-only.
    ///
    /// # Errors
    /// Any failure. A pack that cannot be built never stops the backup itself.
    fn build(
        &self,
        day: &str,
        snapshot: &Path,
        out_dir: &Path,
    ) -> Result<Vec<PackFile>, BackupError>;

    /// True while the source cannot build the whole pack yet.
    fn pending(&self) -> bool;
}

/// No pack yet: the PDF waits for the report engine, the spreadsheets for D5-Q1.
#[derive(Debug, Clone, Copy, Default)]
pub struct PendingPack;

impl PackSource for PendingPack {
    fn build(
        &self,
        _day: &str,
        _snapshot: &Path,
        _out_dir: &Path,
    ) -> Result<Vec<PackFile>, BackupError> {
        Ok(Vec::new())
    }

    fn pending(&self) -> bool {
        true
    }
}

/// Encrypts one file with the backup key as a plain age file (no compression, no wrapper), so
/// `age -d` opens it.
///
/// # Errors
/// The codes of [`crate::pack::seal`].
pub fn seal_file(
    source: &Path,
    key: &BackupKey,
    dest: &Path,
    hook: &dyn IoHook,
) -> Result<(), BackupError> {
    let partial = partial_path(dest);
    let fail = |e: std::io::Error| write_error(BackupKind::Cloud, &e);
    let result = (|| {
        if let Some(dir) = partial.parent() {
            fs::create_dir_all(dir).map_err(fail)?;
        }
        let mut recipient = age::scrypt::Recipient::new(key.passphrase());
        recipient.set_work_factor(WORK_FACTOR);
        let encryptor = Encryptor::with_recipients(iter::once(&recipient as &dyn age::Recipient))
            .map_err(|e| BackupError::internal(format!("encryptor: {e}")))?;
        let mut writer = encryptor
            .wrap_output(TeeWriter::new(
                File::create(&partial).map_err(fail)?,
                hook,
                BackupKind::Cloud,
            ))
            .map_err(fail)?;
        let mut input = File::open(source).map_err(|e| BackupError::io("pack file", &e))?;
        let mut buffer = vec![0_u8; CHUNK];
        loop {
            let count = input
                .read(&mut buffer)
                .map_err(|e| BackupError::io("pack file", &e))?;
            if count == 0 {
                break;
            }
            if let Some(chunk) = buffer.get(..count) {
                writer.write_all(chunk).map_err(fail)?;
            }
        }
        let (file, _, _) = writer.finish().map_err(fail)?.finish();
        file.sync_all().map_err(fail)?;
        fs::rename(&partial, dest).map_err(fail)
    })();
    if result.is_err() {
        remove_quietly(&partial);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::Decryptor;
    use std::io::BufReader;

    #[test]
    fn a_pack_file_opens_with_any_age_tool_and_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("journal.pdf");
        fs::write(&source, b"%PDF-1.7 pretend").unwrap();
        let key = BackupKey::generate().unwrap();
        let dest = dir.path().join("journal.pdf.age");
        seal_file(&source, &key, &dest, &crate::io::NoHook).unwrap();
        let decryptor = Decryptor::new(BufReader::new(File::open(&dest).unwrap())).unwrap();
        let identity = age::scrypt::Identity::new(key.passphrase());
        let mut reader = decryptor
            .decrypt(iter::once(&identity as &dyn age::Identity))
            .unwrap();
        let mut text = Vec::new();
        reader.read_to_end(&mut text).unwrap();
        assert_eq!(text, b"%PDF-1.7 pretend");
    }

    #[test]
    fn the_pending_source_builds_nothing_and_says_so() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            PendingPack
                .build("2026-10-10", &dir.path().join("snapshot.db"), dir.path())
                .unwrap()
                .is_empty()
        );
        assert!(PendingPack.pending());
    }
}
