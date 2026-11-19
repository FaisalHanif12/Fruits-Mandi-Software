//! Updates (plan 22 early, PRD NFR-INS-3, NFR-INS-4, NFR-SEC-11, NFR-REL-13; Release Pipeline 1).
//!
//! The app looks for a newer version only when the Owner asks (`updates_check`), shows what is
//! new, and installs only when he agrees (`updates_install`). It never installs by itself and
//! never during work. The steps of an install, in this order, each of which can stop it:
//!
//! 1. the Owner agreed (`confirmed`);
//! 2. no entry is waiting unsaved (`drafts` is empty), because the installer closes the app
//!    (NFR-INS-3: never during work);
//! 3. the update on offer is the version the Owner saw (`expected_version`);
//! 4. the package is downloaded and its signature is checked against the public key in
//!    `tauri.conf.json`; a package not signed by us, or changed on the way, is refused and
//!    nothing else happens (NFR-SEC-11);
//! 5. a full copy of the books is taken first and must be good (NFR-REL-13, NFR-INS-4). If it
//!    fails, nothing is installed;
//! 6. only now the installer runs. On Windows it closes the app and the new version starts.
//!
//! Example: the Owner presses "update" with version `0.1.13` on screen. The app finds `0.1.13`,
//! downloads it, the signature matches, the local copy is made and checked, and the installer
//! runs. With a bad signature the answer is `ERR_UPDATE_NOT_SIGNED` and no copy is taken.

use mandi_backup::error::{BackupCode, BackupError};
use mandi_services::Context;
use mandi_services::core::error::ErrorValue;
use serde::{Deserialize, Serialize};
use tauri_plugin_updater::{Update, UpdaterExt};
use ts_rs::TS;

/// The heading of the English part of `docs/release/whats-new.md` (the Urdu part comes first).
const ENGLISH_HEADING: &str = "## What's new";

/// What an update check found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct UpdateCheck {
    /// The version that is running.
    pub current_version: String,
    /// True when a newer version is on offer.
    pub available: bool,
    /// The version on offer.
    pub version: Option<String>,
    /// When it was published (text as the release says it).
    pub date: Option<String>,
    /// What is new, as the release carries it (`docs/release/whats-new.md`, Urdu then English).
    pub notes: String,
    /// The Urdu part of the notes.
    pub notes_ur: String,
    /// The English part of the notes.
    pub notes_en: String,
    /// Entries waiting unsaved. An install is refused while there are any.
    #[ts(type = "number")]
    pub drafts: i64,
}

/// The input of `updates_install`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct UpdateInstallInput {
    /// The version the Owner saw and agreed to.
    pub expected_version: String,
    /// True after the Owner said yes.
    pub confirmed: bool,
}

/// The answer of `updates_install`. On Windows the installer closes the app, so the screen may
/// never read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct UpdateInstalled {
    /// The version installed.
    pub version: String,
    /// The local copy taken before the install (its file name).
    pub backup_file: Option<String>,
}

/// Splits the notes of a release into the Urdu part and the English part (the English part
/// starts at the heading `## What's new`). Without that heading both parts are empty and the
/// screen shows `notes` whole.
///
/// ```
/// let (ur, en) = mandi_app_lib::updates::split_notes("## نیا کیا ہے\n\n- ایک\n\n## What's new\n\n- One\n");
/// assert_eq!(ur, "- ایک");
/// assert_eq!(en, "- One");
/// ```
pub fn split_notes(text: &str) -> (String, String) {
    let Some(at) = text.find(ENGLISH_HEADING) else {
        return (String::new(), String::new());
    };
    let (urdu, english) = text.split_at(at);
    let body = |part: &str| -> String {
        part.lines()
            .skip_while(|line| line.trim().is_empty() || line.trim_start().starts_with("## "))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_owned()
    };
    (body(urdu), body(english))
}

/// A plugin error as a code the screen has words for.
fn failure(error: &tauri_plugin_updater::Error) -> BackupError {
    use tauri_plugin_updater::Error;
    let code = match error {
        // Anything that says the package is not what we signed.
        Error::Minisign(_)
        | Error::Base64(_)
        | Error::SignatureUtf8(_)
        | Error::SignedVersionMismatch { .. }
        | Error::MissingSignedVersion => BackupCode::ErrUpdateNotSigned,
        Error::Io(_) => BackupCode::ErrInternal,
        _ => BackupCode::ErrUpdateCheckFailed,
    };
    BackupError::new(code, format!("updater: {error}"))
}

/// The updater of the app. If the update service is not part of this app (the plugin is not
/// there), that is a clean "cannot check" and not a crash: the plugin panics when asked without
/// being set up.
fn updater_of<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<tauri_plugin_updater::Updater, BackupError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.updater())) {
        Ok(built) => built.map_err(|e| failure(&e)),
        Err(_) => Err(BackupError::new(
            BackupCode::ErrUpdateCheckFailed,
            "the update service is not running",
        )),
    }
}

/// The number of entries waiting unsaved.
fn drafts_waiting(ctx: &Context) -> Result<i64, BackupError> {
    ctx.db()
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM drafts", [], |r| r.get(0)))
        .map_err(|e| BackupError::internal(format!("drafts: {e}")))
}

/// Looks for a newer version.
///
/// # Errors
/// `ERR_UPDATE_CHECK_FAILED` when the server cannot be asked or answers with something that is
/// not an update; `ERR_INTERNAL`.
pub async fn check<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    ctx: &Context,
) -> Result<UpdateCheck, BackupError> {
    let drafts = drafts_waiting(ctx)?;
    let updater = updater_of(app)?;
    let current_version = app.package_info().version.to_string();
    let found = updater.check().await.map_err(|e| failure(&e))?;
    Ok(match found {
        None => UpdateCheck {
            current_version,
            available: false,
            version: None,
            date: None,
            notes: String::new(),
            notes_ur: String::new(),
            notes_en: String::new(),
            drafts,
        },
        Some(update) => {
            let notes = update.body.clone().unwrap_or_default();
            let (notes_ur, notes_en) = split_notes(&notes);
            UpdateCheck {
                current_version,
                available: true,
                date: update
                    .raw_json
                    .get("pub_date")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                version: Some(update.version),
                notes,
                notes_ur,
                notes_en,
                drafts,
            }
        }
    })
}

/// Installs the update the Owner agreed to, in the order of the module comment. The copy before
/// the install and the install itself are given as functions, so a test can watch the order
/// without installing anything.
///
/// # Errors
/// `ERR_UPDATE_NOT_CONFIRMED` (field `confirmed`), `ERR_UPDATE_DURING_WORK` (value `drafts`),
/// `ERR_UPDATE_NONE`, `ERR_UPDATE_CHANGED` (field `expected_version`), `ERR_UPDATE_NOT_SIGNED`,
/// `ERR_UPDATE_CHECK_FAILED`, `ERR_UPDATE_BACKUP_FAILED`, `ERR_INTERNAL`.
pub async fn install_flow<R, B, I>(
    app: &tauri::AppHandle<R>,
    ctx: &Context,
    input: &UpdateInstallInput,
    take_backup: B,
    install: I,
) -> Result<UpdateInstalled, BackupError>
where
    R: tauri::Runtime,
    B: FnOnce() -> Result<Option<String>, BackupError> + Send + 'static,
    I: FnOnce(&Update, Vec<u8>) -> Result<(), tauri_plugin_updater::Error> + Send + 'static,
{
    if !input.confirmed {
        return Err(
            BackupError::new(BackupCode::ErrUpdateNotConfirmed, "the Owner did not agree")
                .on_field("confirmed"),
        );
    }
    let drafts = drafts_waiting(ctx)?;
    if drafts > 0 {
        return Err(BackupError::new(
            BackupCode::ErrUpdateDuringWork,
            "entries are waiting unsaved",
        )
        .with("drafts", ErrorValue::Number(drafts)));
    }
    let updater = updater_of(app)?;
    let Some(update) = updater.check().await.map_err(|e| failure(&e))? else {
        return Err(BackupError::new(
            BackupCode::ErrUpdateNone,
            "no newer version",
        ));
    };
    if update.version != input.expected_version.trim() {
        return Err(BackupError::new(
            BackupCode::ErrUpdateChanged,
            format!(
                "the offer is {}, the Owner saw {}",
                update.version, input.expected_version
            ),
        )
        .on_field("expected_version"));
    }
    // The package is checked against our public key as it is downloaded. A bad signature stops
    // here: no copy is taken and nothing is installed.
    let bytes = update
        .download(|_, _| {}, || {})
        .await
        .map_err(|e| failure(&e))?;
    // The books are copied and checked before the installer can change anything.
    let backup_file = tauri::async_runtime::spawn_blocking(take_backup)
        .await
        .map_err(|e| BackupError::internal(format!("the copy did not finish: {e}")))??;
    let version = update.version.clone();
    tauri::async_runtime::spawn_blocking(move || install(&update, bytes))
        .await
        .map_err(|e| BackupError::internal(format!("the install did not finish: {e}")))?
        .map_err(|e| failure(&e))?;
    Ok(UpdateInstalled {
        version,
        backup_file,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notes_split_at_the_english_heading() {
        let text = "## نیا کیا ہے\n\n- ایک\n- دو\n\n## What's new\n\n- One\n- Two\n";
        let (ur, en) = split_notes(text);
        assert_eq!(ur, "- ایک\n- دو");
        assert_eq!(en, "- One\n- Two");
    }

    #[test]
    fn notes_without_the_english_heading_are_not_split() {
        assert_eq!(split_notes("only text"), (String::new(), String::new()));
        assert_eq!(split_notes(""), (String::new(), String::new()));
    }
}
