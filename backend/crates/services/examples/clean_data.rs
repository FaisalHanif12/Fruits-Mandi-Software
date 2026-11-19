//! Makes a clean data file for the Owner's app (plan 32 R9). Run by `pnpm seed -- --clean <live file>`:
//!
//! ```bash
//! cargo run -p mandi-services --example clean_data -- <live mandi.db> <new mandi.db> [--drop-partners]
//! ```
//!
//! The work is `mandi_services::clean_data::make_clean_file` (see its header: what is kept and what
//! is left behind). The live file is never opened, changed or deleted; a copy is read. The new file
//! is made with the real audit key store, like the app's own, so the app can verify its audit chain
//! (on a Mac a Keychain prompt may appear; `MANDI_SEED_TEST_KEY=1` uses a throwaway dev key
//! instead, and a file made with it cannot be verified by the real app).
//!
//! If the live file has partners they are NOT copied, so the tool stops unless `--drop-partners`
//! is given. At the end it prints the steps to put the file in place: the old folder is kept beside
//! the new one as `<folder>.old-<time>` and is never deleted.

use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use mandi_services::clean_data::{CleanReport, make_clean_file};
use mandi_services::db::DbOptions;
use mandi_services::db::audit::{AuditKey, MemoryKeyStore};
use mandi_services::db::clock::{Clock, SystemClock};

fn say(line: &str) {
    // Printing macros are denied in this workspace; this writes the same way.
    writeln!(std::io::stdout(), "{line}").ok();
}

fn print_report(live_path: &Path, new_path: &Path, report: &CleanReport) {
    say(&format!(
        "clean_data: read a copy of {}",
        live_path.display()
    ));
    for (name, number) in &report.left_behind {
        say(&format!(
            "  live file has {number} {name} (none of them is copied)"
        ));
    }
    say(&format!(
        "clean_data: made {} for owner \"{}\" (same password, {} settings kept)",
        new_path.display(),
        report.username,
        report.kept_settings.len()
    ));
    say(&format!(
        "  it has {} system accounts, {} starting rules, {} sellers, {} buyers, {} items, {} entries; the books balance and the audit chain verifies",
        report.system_accounts,
        report.rules,
        report.sellers,
        report.buyers,
        report.items,
        report.entries
    ));
}

/// Tells the user how to put the file in place. The old folder is renamed, never deleted.
fn print_steps(live_path: &Path, new_path: &Path) {
    let data_dir = live_path.parent().unwrap_or_else(|| Path::new("."));
    // The app's folder holds `data/` and `backups/`; a bare file sits in a folder of its own.
    let app_dir = if data_dir.file_name().is_some_and(|n| n == "data") {
        data_dir.parent().unwrap_or(data_dir)
    } else {
        data_dir
    };
    let stamp = SystemClock.now_utc().format("%Y%m%d-%H%M%S");
    let old = format!("{}.old-{stamp}", app_dir.display());
    // If the new file sits inside the folder that is moved (a dev run), it is found in the old
    // folder after step 1.
    let source = match new_path.strip_prefix(app_dir) {
        Ok(inside) => Path::new(&old).join(inside),
        Err(_) => new_path.to_path_buf(),
    };
    say("To use it (close the app first; nothing below deletes anything):");
    say(&format!("  1. mv \"{}\" \"{old}\"", app_dir.display()));
    say(&format!("  2. mkdir -p \"{}/data\"", app_dir.display()));
    say(&format!(
        "  3. cp \"{}\" \"{}/data/mandi.db\"",
        source.display(),
        app_dir.display()
    ));
    say(
        "  4. Start the app and sign in with the same user name and password. The first seller you add gets 0010.",
    );
    say(&format!(
        "  To go back: close the app, remove the new folder and move \"{old}\" back to \"{}\".",
        app_dir.display()
    ));
    say(&format!(
        "The live file {} was not changed.",
        live_path.display()
    ));
}

fn main() -> ExitCode {
    let mut paths: Vec<String> = Vec::new();
    let mut drop_partners = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--drop-partners" => drop_partners = true,
            _ => paths.push(arg),
        }
    }
    let [live, new] = paths.as_slice() else {
        say("usage: clean_data <live mandi.db> <new mandi.db> [--drop-partners]");
        return ExitCode::FAILURE;
    };
    let (live_path, new_path) = (Path::new(live), Path::new(new));
    let backups = new_path.parent().map_or_else(
        || Path::new("backups").to_path_buf(),
        |dir| dir.join("backups"),
    );
    let mut options = DbOptions::new(backups);
    if std::env::var_os("MANDI_SEED_TEST_KEY").is_some() {
        options.key_store = Arc::new(MemoryKeyStore::new(AuditKey::from_bytes([7; 32])));
    }
    match make_clean_file(live_path, new_path, options, drop_partners) {
        Ok(report) => {
            print_report(live_path, new_path, &report);
            print_steps(live_path, new_path);
            ExitCode::SUCCESS
        }
        Err(error) => {
            say(&format!("clean_data: {error}"));
            ExitCode::FAILURE
        }
    }
}
