//! The support information file (plan 22 early, NFR-OBS-4, NFR-OBS-2): one text file with the
//! app version, the system, the state of the books and the backups, and the last lines of the
//! log. It holds no password, key, recovery code or balance (see `support_info`).
//!
//! The Owner may choose where it goes (the save box of the dialog plugin gives a full path);
//! without a path it goes in the `support` folder of the app data, named by the time.

use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};
use mandi_backup::api::BackupStatus;
use mandi_services::core::error::ErrorCode;
use mandi_services::{ServiceError, auth};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::run;
use crate::backup_runtime::BackupRuntime;
use crate::error::UiError;
use crate::state::AppState;
use crate::support_info::{MAX_LOG_BYTES, MAX_LOG_LINES, Section, SupportParts, log_tail, render};

/// Pakistan time is UTC+5 all year (Code Standards G9).
const PAKISTAN_OFFSET_HOURS: i64 = 5;

/// The input of `support_info_save`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SupportInfoInput {
    /// A full path ending in `.txt`, chosen in the save box. Null puts the file in the `support`
    /// folder of the app data.
    pub path: Option<String>,
}

/// What `support_info_save` made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SupportInfoSaved {
    /// Where the file is.
    pub path: String,
    /// Its size in bytes.
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// The log lines in it.
    #[ts(type = "number")]
    pub log_lines: u64,
}

fn fact(name: &str, value: impl ToString) -> (String, String) {
    (name.to_owned(), value.to_string())
}

fn yes_no(flag: bool) -> &'static str {
    if flag { "yes" } else { "no" }
}

fn text_or_none(value: Option<&str>) -> String {
    value.unwrap_or("none").to_owned()
}

/// Checks a path chosen in the save box: full, a `.txt` file name, no control characters, and its
/// folder is there.
fn checked_path(text: &str) -> Result<PathBuf, ServiceError> {
    let bad = || ServiceError::reject_field(ErrorCode::ErrOutOfRange, "path");
    let path = Path::new(text.trim());
    let is_txt = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("txt"));
    if text.chars().any(char::is_control)
        || !path.is_absolute()
        || !is_txt
        || path.file_name().is_none()
        || !path.parent().is_some_and(Path::is_dir)
    {
        return Err(bad());
    }
    Ok(path.to_path_buf())
}

/// The numbers and dates of the database, never an amount.
fn data_facts(
    ctx: &mandi_services::Context,
    data_dir: &Path,
) -> Result<Vec<(String, String)>, ServiceError> {
    let size = std::fs::metadata(data_dir.join("data").join("mandi.db"))
        .map(|m| m.len())
        .unwrap_or(0);
    let count = |sql: &str| -> Result<i64, ServiceError> {
        Ok(ctx
            .db()
            .read(|conn| conn.query_row(sql, [], |r| r.get(0)))?)
    };
    Ok(vec![
        fact("database file bytes", size),
        fact("schema version", count("PRAGMA user_version")?),
        fact("accounts", count("SELECT COUNT(*) FROM accounts")?),
        fact(
            "posted journal entries",
            count("SELECT COUNT(*) FROM journal_entries WHERE status = 'posted'")?,
        ),
        fact(
            "bikri entries",
            count("SELECT COUNT(*) FROM bikri_entries")?,
        ),
        fact("vouchers", count("SELECT COUNT(*) FROM vouchers")?),
        fact("audit rows", count("SELECT COUNT(*) FROM audit_log")?),
        fact("drafts waiting", count("SELECT COUNT(*) FROM drafts")?),
    ])
}

fn backup_facts(status: Option<&BackupStatus>) -> Vec<(String, String)> {
    let Some(s) = status else {
        return vec![fact("backup service", "not running")];
    };
    vec![
        fact("backup key set", yes_no(s.key_set)),
        fact("backup key confirmed", yes_no(s.key_confirmed)),
        fact(
            "last good local copy",
            text_or_none(s.last_local_at.as_deref()),
        ),
        fact("last good usb copy", text_or_none(s.last_usb_at.as_deref())),
        fact(
            "last good cloud copy",
            text_or_none(s.last_cloud_at.as_deref()),
        ),
        fact("cloud level", format!("{:?}", s.cloud_level)),
        fact("cloud copies waiting", s.queue_len),
        fact("usb copy on", yes_no(s.usb_enabled)),
        fact("cloud copy on", yes_no(s.cloud_enabled)),
        fact("backup problems", s.errors.len()),
    ]
}

fn health_facts(ctx: &mandi_services::Context) -> Result<Vec<(String, String)>, ServiceError> {
    let Some(last) = mandi_services::health::last(ctx)? else {
        return Ok(vec![fact("health check", "never ran")]);
    };
    let flag = |path: &[&str]| -> String {
        let mut value = &last;
        for key in path {
            match value.get(key) {
                Some(next) => value = next,
                None => return "unknown".to_owned(),
            }
        }
        value
            .as_bool()
            .map_or_else(|| "unknown".to_owned(), |b| yes_no(b).to_owned())
    };
    Ok(vec![
        fact(
            "health checked at",
            last.get("checked_at")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown"),
        ),
        fact("health ok", flag(&["ok"])),
        fact("books balanced", flag(&["books", "balanced"])),
        fact("journal chain ok", flag(&["books", "journal_chain", "ok"])),
        fact("audit chain ok", flag(&["books", "audit_chain", "ok"])),
    ])
}

/// Makes the support information file and says where it is: the app version, the system, the
/// state of the data file, the books and the backups (counts, dates and yes/no only) and the
/// last lines of the log with every secret hidden. Errors: `ERR_OUT_OF_RANGE` on `path` (not a
/// full path to a `.txt` file in a folder that exists), `ERR_DISK_FULL`, `ERR_INTERNAL`.
#[tauri::command]
pub async fn support_info_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SupportInfoInput,
) -> Result<SupportInfoSaved, UiError> {
    let state = AppState::of(&app)?;
    let backup = BackupRuntime::of(&app)
        .ok()
        .and_then(|runtime| runtime.service().status().ok());
    let package = app.package_info().clone();
    let identifier = app.config().identifier.clone();
    let webview = tauri::webview_version().ok();
    run(app, "support_info_save", move |ctx| {
        auth::require(ctx)?;
        // The app data folder is the one the data file is in (`<app data>/data/mandi.db`), so the
        // logs and the `support` folder are next to it.
        let data_dir = ctx
            .db()
            .path()
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .ok_or_else(|| ServiceError::Data("the data file has no folder".to_owned()))?;
        let now = Utc::now();
        let local = now + Duration::hours(PAKISTAN_OFFSET_HOURS);
        let target = match input.path.as_deref() {
            Some(text) => checked_path(text)?,
            None => data_dir.join("support").join(format!(
                "mandi-support-{}.txt",
                local.format("%Y%m%d-%H%M%S")
            )),
        };
        let data = state.data_status();
        let mut sections = vec![
            Section {
                title: "app".to_owned(),
                facts: vec![
                    fact("name", &package.name),
                    fact("version", &package.version),
                    fact("identifier", identifier),
                    fact("made at (UTC)", now.format("%Y-%m-%d %H:%M:%S")),
                    fact("made at (Pakistan)", local.format("%Y-%m-%d %H:%M:%S")),
                ],
            },
            Section {
                title: "system".to_owned(),
                facts: vec![
                    fact("os", std::env::consts::OS),
                    fact("family", std::env::consts::FAMILY),
                    fact("architecture", std::env::consts::ARCH),
                    fact("webview", webview.unwrap_or_else(|| "unknown".to_owned())),
                ],
            },
            Section {
                title: "data".to_owned(),
                facts: {
                    let mut facts = vec![
                        fact("data check", format!("{:?}", data.state)),
                        fact(
                            "data check reference",
                            text_or_none(data.reference.as_deref()),
                        ),
                    ];
                    facts.extend(data_facts(ctx, &data_dir)?);
                    facts
                },
            },
            Section {
                title: "health".to_owned(),
                facts: health_facts(ctx)?,
            },
            Section {
                title: "backup".to_owned(),
                facts: backup_facts(backup.as_ref()),
            },
        ];
        sections.push(Section {
            title: "settings".to_owned(),
            facts: {
                let settings = mandi_services::settings::get(ctx)?;
                let mut facts = vec![
                    fact("default language", settings.default_language.as_str()),
                    fact("number grouping", format!("{:?}", settings.number_grouping)),
                    fact("idle lock minutes", settings.idle_lock_minutes),
                ];
                if let Ok(day) = ctx.db().read(mandi_backup::day_settings::load) {
                    facts.push(fact("closing time", day.closing_time));
                    facts.push(fact("auto day close", yes_no(day.auto_day_close)));
                }
                facts
            },
        });
        let log_lines = log_tail(&data_dir.join("logs"), MAX_LOG_LINES, MAX_LOG_BYTES)
            .map_err(|e| ServiceError::Data(format!("the log folder: {e}")))?;
        let line_count = u64::try_from(log_lines.len()).unwrap_or(u64::MAX);
        let text = render(&SupportParts {
            sections,
            log_lines,
        });
        if let Some(folder) = target.parent() {
            std::fs::create_dir_all(folder)
                .map_err(|e| ServiceError::Data(format!("the support folder: {e}")))?;
        }
        std::fs::write(&target, text.as_bytes()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::StorageFull {
                ServiceError::reject(ErrorCode::ErrDiskFull)
            } else {
                ServiceError::Data(format!("the support file: {e}"))
            }
        })?;
        Ok(SupportInfoSaved {
            path: target.to_string_lossy().into_owned(),
            size_bytes: u64::try_from(text.len()).unwrap_or(u64::MAX),
            log_lines: line_count,
        })
    })
    .await
}
