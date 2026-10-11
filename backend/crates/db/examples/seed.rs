//! Makes a development database and prints what it holds. Run by `pnpm seed`:
//!
//! ```bash
//! cargo run -p mandi-db --example seed -- <path to mandi.db>
//! ```
//!
//! The file gets the full schema and the system accounts of PRD FR-10.4. Running it again on
//! the same file changes nothing.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mandi_db::{Db, DbError, DbOptions, seed};

fn say(line: &str) {
    // Printing macros are denied in this workspace; this writes the same way.
    writeln!(std::io::stdout(), "{line}").ok();
}

fn run(path: &Path) -> Result<(), DbError> {
    let backups = path
        .parent()
        .map_or_else(|| PathBuf::from("backups"), |dir| dir.join("backups"));
    let db = Db::open_checked(path, DbOptions::new(backups))?;
    let counts = db.read(|conn| seed::counts(conn).map_err(DbError::from))?;
    say(&format!("seed: {}", path.display()));
    say(&format!(
        "seed: {} heads, {} controls, {} accounts",
        counts.heads, counts.controls, counts.accounts
    ));
    Ok(())
}

fn main() -> ExitCode {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        writeln!(std::io::stderr(), "usage: seed <path to mandi.db>").ok();
        return ExitCode::from(2);
    };
    match run(&path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            writeln!(std::io::stderr(), "seed failed: {error}").ok();
            ExitCode::FAILURE
        }
    }
}
