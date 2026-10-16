//! Log files (plan 03 Section 4.7, NFR-OBS-1, NFR-OBS-2).
//!
//! - Daily files `mandi.log.YYYY-MM-DD` in `<app_data_dir>/logs/`.
//! - At start-up our own clean-up keeps 30 days and at most 50 MB in total (oldest first).
//! - A field whose name holds `password`, `key`, `secret`, `token`, `recovery_code` or
//!   `balance` is written as `[hidden]`, never its value.
//! - A panic anywhere is written to the log before the thread ends.
//!
//! A secret must be passed as a field (`tracing::info!(password = %p, "login")`), never put
//! inside the message text, because only fields can be hidden by name.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use chrono::NaiveDate;
use tracing::Subscriber;
use tracing::field::Field;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::format::{Writer, debug_fn};

/// Start of every log file name.
pub const LOG_FILE_PREFIX: &str = "mandi.log";
/// Log files this many days old or older are deleted (NFR-OBS-1: 30 days kept).
pub const LOG_KEEP_DAYS: i64 = 30;
/// All log files together stay under this size (NFR-OBS-1: 50 MB).
pub const LOG_MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024;
/// What replaces the value of a secret field.
pub const HIDDEN: &str = "[hidden]";

/// Words that make a field secret. A field is hidden if its name CONTAINS one of them, so
/// `password_hash`, `new_password`, `audit_key` and `balance_paisa` are hidden too.
const SECRET_WORDS: [&str; 6] = [
    "password",
    "key",
    "secret",
    "token",
    "recovery_code",
    "balance",
];

/// True if the value of the field must never be written to the log.
pub fn is_secret_field(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SECRET_WORDS.iter().any(|word| lower.contains(word))
}

fn write_field(writer: &mut Writer<'_>, field: &Field, value: &dyn fmt::Debug) -> fmt::Result {
    let name = field.name();
    if is_secret_field(name) {
        write!(writer, "{name}={HIDDEN}")
    } else if name == "message" {
        write!(writer, "{value:?}")
    } else {
        write!(writer, "{name}={value:?}")
    }
}

/// The subscriber the app uses, writing to `writer`. Tests use it with a file or a buffer, so
/// they check the same redaction the real log gets.
pub fn subscriber<W>(writer: W) -> impl Subscriber + Send + Sync
where
    W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
{
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .fmt_fields(debug_fn(write_field))
        .finish()
}

/// What a clean-up removed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CleanupReport {
    /// Names of the deleted files.
    pub deleted: Vec<String>,
    /// Bytes left in the log folder.
    pub kept_bytes: u64,
}

struct LogFile {
    name: String,
    date: NaiveDate,
    bytes: u64,
}

/// The date in `mandi.log.2026-10-10`, else the file's modified date.
fn file_date(name: &str, metadata: &fs::Metadata) -> Option<NaiveDate> {
    let suffix = name.strip_prefix(LOG_FILE_PREFIX)?.strip_prefix('.')?;
    NaiveDate::parse_from_str(suffix, "%Y-%m-%d")
        .ok()
        .or_else(|| {
            let modified = metadata.modified().ok()?;
            Some(chrono::DateTime::<chrono::Utc>::from(modified).date_naive())
        })
}

/// Deletes log files that are `keep_days` old or older, then the oldest ones until the rest
/// fit in `max_bytes`. Today's file is never deleted (it is open). Other files in the folder
/// are not touched.
///
/// # Errors
/// An I/O error when the folder cannot be read. A file that cannot be deleted is skipped.
pub fn clean_up(
    dir: &Path,
    today: NaiveDate,
    keep_days: i64,
    max_bytes: u64,
) -> io::Result<CleanupReport> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata = entry.metadata()?;
        if !metadata.is_file() || !name.starts_with(LOG_FILE_PREFIX) {
            continue;
        }
        if let Some(date) = file_date(&name, &metadata) {
            files.push(LogFile {
                name,
                date,
                bytes: metadata.len(),
            });
        }
    }
    files.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.name.cmp(&b.name)));

    let mut report = CleanupReport::default();
    let mut total: u64 = files.iter().map(|f| f.bytes).sum();
    for file in &files {
        let too_old = (today - file.date).num_days() >= keep_days;
        let too_big = total > max_bytes;
        if file.date >= today || !(too_old || too_big) {
            continue;
        }
        if fs::remove_file(dir.join(&file.name)).is_ok() {
            total = total.saturating_sub(file.bytes);
            report.deleted.push(file.name.clone());
        }
    }
    report.kept_bytes = total;
    Ok(report)
}

/// Starts logging to `<log_dir>/mandi.log.<date>`: clean-up, daily file, redaction and the
/// panic hook. Keep the returned guard alive until the app ends, or the last lines are lost.
///
/// # Errors
/// An I/O error when the folder cannot be made, or `AlreadySet` when logging was started before.
pub fn init(log_dir: &Path) -> io::Result<WorkerGuard> {
    fs::create_dir_all(log_dir)?;
    let today = chrono::Utc::now().date_naive();
    // A failed clean-up must not stop the app: old logs just stay a little longer.
    if let Err(error) = clean_up(log_dir, today, LOG_KEEP_DAYS, LOG_MAX_TOTAL_BYTES) {
        tracing::warn!("log clean-up failed: {error}");
    }
    let appender = tracing_appender::rolling::daily(log_dir, LOG_FILE_PREFIX);
    let (writer, guard) = tracing_appender::non_blocking(appender);
    tracing::subscriber::set_global_default(subscriber(writer))
        .map_err(|e| io::Error::new(io::ErrorKind::AlreadyExists, e.to_string()))?;
    install_panic_hook();
    Ok(guard)
}

/// Writes every panic to the log, then runs the previous hook.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-text panic".to_owned());
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        tracing::error!(location = %location, "panic: {message}");
        previous(info);
    }));
}
