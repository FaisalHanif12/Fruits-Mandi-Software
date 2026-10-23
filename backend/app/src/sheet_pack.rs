//! The spreadsheets of the readable daily pack for the backup service (PRD FR-20.5, NFR-BAK-13).
//!
//! `mandi-backup` cannot depend on the report engine (Architecture 3), so it asks a
//! [`PackSource`] for the pack files; this one makes the five `.xlsx` sheets with
//! `mandi_reports::pack` and hands them over. The روزنامچہ PDF needs WebView2 printing and comes
//! with the Windows phase, so [`PackSource::pending`] stays true: the pack is not whole until
//! the PDF is part of it. A pack that cannot be built never stops the backup itself.

use std::path::Path;
use std::sync::Arc;

use mandi_backup::BackupError;
use mandi_backup::daily_pack::{PackFile, PackPart, PackSource};
use mandi_reports::ReportsError;
use mandi_services::db::Db;

/// Makes the five spreadsheets of the pack from the open database.
pub struct SheetPack {
    db: Arc<Db>,
}

impl SheetPack {
    /// The source for this database.
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

fn pack_error(error: &ReportsError) -> BackupError {
    // The cause goes to the log by the caller; no amount is in it.
    BackupError::internal(format!("readable pack: {error}"))
}

impl PackSource for SheetPack {
    fn build(&self, day: &str, out_dir: &Path) -> Result<Vec<PackFile>, BackupError> {
        let sheets = mandi_reports::pack::build_sheets(&self.db, day, out_dir)
            .map_err(|e| pack_error(&e))?;
        Ok(sheets
            .into_iter()
            .map(|sheet| PackFile {
                part: PackPart::Sheet,
                name: sheet.name,
                path: sheet.path,
            })
            .collect())
    }

    fn pending(&self) -> bool {
        true
    }
}
