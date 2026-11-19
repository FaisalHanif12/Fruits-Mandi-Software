//! The چکتا buyers (plan 64, PRD FR-7.29, FR-7.69, Section 13): the list the buyer picker of a
//! بکری line offers beside ordinary buyers, and the add on the spot. A چکتا buyer is not an
//! account and has no ledger (PRD FR-13.6).

use mandi_services::core::chukta::{ChuktaAddInput, ChuktaBuyer, ChuktaSearchInput};
use mandi_services::{auth, chukta};

use super::run;
use crate::error::UiError;

/// The چکتا buyers whose names or phone hold the typed text, in either script, newest first (by
/// the day of his last sale line), at most 10 (PRD FR-7.29). Empty text lists the newest ten.
/// Picking one reuses his exact names and phone. Read only (Code Standards B29). Errors:
/// `ERR_TEXT_TOO_LONG` on `text`.
#[tauri::command]
pub async fn chukta_search<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ChuktaSearchInput,
) -> Result<Vec<ChuktaBuyer>, UiError> {
    run(app, "chukta_search", move |ctx| {
        auth::require(ctx)?;
        chukta::search(ctx, &input)
    })
    .await
}

/// Adds a چکتا buyer from the short form of the picker (PRD FR-7.69, FR-13.3): both names and an
/// optional phone. A person with the same name and phone is returned as he is, not made twice. One
/// transaction with its audit row. Errors, together, on their own field: `ERR_CHUKTA_NAME_REQUIRED`,
/// `ERR_NAME_UR_SCRIPT` and `ERR_NAME_EN_SCRIPT` (`name_ur`, `name_en`), `ERR_PHONE_INVALID`
/// (`phone`).
#[tauri::command]
pub async fn chukta_add<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ChuktaAddInput,
) -> Result<ChuktaBuyer, UiError> {
    run(app, "chukta_add", move |ctx| {
        let actor = auth::require(ctx)?;
        chukta::add(ctx, &actor, &input)
    })
    .await
}
