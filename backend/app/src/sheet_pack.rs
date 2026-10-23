//! The spreadsheets of the readable daily pack for the backup service (PRD FR-20.5, NFR-BAK-13).
//!
//! `mandi-backup` cannot depend on the report engine (Architecture 3), so it asks a
//! [`PackSource`] for the pack files; this one makes the five `.xlsx` sheets with
//! `mandi_reports::pack` and hands them over. The sheets are built from the SNAPSHOT of the copy
//! the pack travels with, opened read-only, never from the live books: a pack must hold the
//! numbers of the backup next to it. The روزنامچہ PDF needs WebView2 printing and comes
//! with the Windows phase, so [`PackSource::pending`] stays true: the pack is not whole until
//! the PDF is part of it. A pack that cannot be built never stops the backup itself.

use std::path::Path;
use std::sync::Arc;

use mandi_backup::BackupError;
use mandi_backup::daily_pack::{PackFile, PackPart, PackSource};
use mandi_reports::ReportsError;
use mandi_services::db::clock::Clock;

/// Makes the five spreadsheets of the pack from the snapshot of the copy it travels with.
pub struct SheetPack {
    clock: Arc<dyn Clock>,
}

impl SheetPack {
    /// The source. `clock` gives the time written on every sheet.
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self { clock }
    }
}

fn pack_error(error: &ReportsError) -> BackupError {
    // The cause goes to the log by the caller; no amount is in it.
    BackupError::internal(format!("readable pack: {error}"))
}

impl PackSource for SheetPack {
    fn build(
        &self,
        day: &str,
        snapshot: &Path,
        out_dir: &Path,
    ) -> Result<Vec<PackFile>, BackupError> {
        let sheets = mandi_reports::pack::build_sheets_from_snapshot(
            snapshot,
            Arc::clone(&self.clock),
            day,
            out_dir,
        )
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
