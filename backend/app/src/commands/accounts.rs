//! Accounts and the chart of accounts (PRD 10.1, 10.2, plan 07).
//!
//! No delete command exists: an account with postings can only be made inactive (PRD FR-10.6).

use mandi_services::core::accounts::{
    AccountInput, AccountRow, AccountsGetInput, AccountsList, AccountsListInput,
    AccountsNextCodeInput, AccountsSetActiveInput, AccountsSimilarInput, CoaControl, CoaHead,
    CoaTree, OpeningsInput, OpeningsPreview, OpeningsSaved, SimilarAccount,
};
use mandi_services::{accounts, auth, coa};

use super::run;
use crate::error::UiError;

/// Adds an account (`id` null) or edits one (`id` and the `version` read). One transaction:
/// the account, its opening entry (PRD FR-10.9), the spelling check and the audit row.
/// Errors: the validate codes with field paths, `ERR_CODE_TAKEN`, `ERR_CODE_RESERVED`,
/// `ERR_SERIES_FULL` (`code`), `ERR_OPENING_BOTH_SIDES`, `ERR_ADVANCE_NOT_SELLER`,
/// `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE` (`opening_date`), `ERR_NOT_FOUND`,
/// `ERR_VERSION_CHANGED`, `ERR_SPELLING_UNCONFIRMED`.
#[tauri::command]
pub async fn accounts_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AccountInput,
) -> Result<AccountRow, UiError> {
    run(app, "accounts_save", move |ctx| {
        let actor = auth::require(ctx)?;
        accounts::save(ctx, &actor, &input)
    })
    .await
}

/// One account, for the edit form. Errors: `ERR_NOT_FOUND`.
#[tauri::command]
pub async fn accounts_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AccountsGetInput,
) -> Result<AccountRow, UiError> {
    run(app, "accounts_get", move |ctx| {
        auth::require(ctx)?;
        accounts::get(ctx, input.id)
    })
    .await
}

/// The account list with the columns of PRD FR-10.5 and its footer totals.
#[tauri::command]
pub async fn accounts_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AccountsListInput,
) -> Result<AccountsList, UiError> {
    run(app, "accounts_list", move |ctx| {
        auth::require(ctx)?;
        accounts::list(ctx, &input)
    })
    .await
}

/// Makes an account active or inactive. Errors: `ERR_NOT_FOUND`.
#[tauri::command]
pub async fn accounts_set_active<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AccountsSetActiveInput,
) -> Result<AccountRow, UiError> {
    run(app, "accounts_set_active", move |ctx| {
        let actor = auth::require(ctx)?;
        accounts::set_active(ctx, &actor, input.id, input.is_active)
    })
    .await
}

/// The next free code for a kind of account. Errors: `ERR_SERIES_FULL`, `ERR_NOT_FOUND`
/// (`control_id`), `ERR_REQUIRED` (`control_id` for type `other`).
#[tauri::command]
pub async fn accounts_next_code<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AccountsNextCodeInput,
) -> Result<String, UiError> {
    run(app, "accounts_next_code", move |ctx| {
        auth::require(ctx)?;
        accounts::next_code(ctx, input.account_type, input.control_id)
    })
    .await
}

/// Accounts that look like the one being typed, by name in either script and by phone
/// (PRD FR-10.7, FR-7.63). Read only.
#[tauri::command]
pub async fn accounts_similar<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AccountsSimilarInput,
) -> Result<Vec<SimilarAccount>, UiError> {
    run(app, "accounts_similar", move |ctx| {
        auth::require(ctx)?;
        accounts::similar(ctx, &input)
    })
    .await
}

/// Checks the openings grid and says what ابتدائی فرق would be. Nothing is saved.
#[tauri::command]
pub async fn accounts_openings_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: OpeningsInput,
) -> Result<OpeningsPreview, UiError> {
    run(app, "accounts_openings_preview", move |ctx| {
        auth::require(ctx)?;
        accounts::openings_preview(ctx, &input)
    })
    .await
}

/// Saves the openings grid in one transaction (all or nothing). Errors point to
/// `lines[i].<field>`.
#[tauri::command]
pub async fn accounts_openings_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: OpeningsInput,
) -> Result<OpeningsSaved, UiError> {
    run(app, "accounts_openings_save", move |ctx| {
        let actor = auth::require(ctx)?;
        accounts::openings_save(ctx, &actor, &input)
    })
    .await
}

/// The chart of accounts in three levels with live balances (PRD FR-10.1).
#[tauri::command]
pub async fn coa_tree<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<CoaTree, UiError> {
    run(app, "coa_tree", |ctx| {
        auth::require(ctx)?;
        coa::tree(ctx)
    })
    .await
}

/// The six heads.
#[tauri::command]
pub async fn coa_heads_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<CoaHead>, UiError> {
    run(app, "coa_heads_list", |ctx| {
        auth::require(ctx)?;
        coa::heads(ctx)
    })
    .await
}

/// The controls, each with the flag that says whether an account of type `other` may go there.
#[tauri::command]
pub async fn coa_controls_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<CoaControl>, UiError> {
    run(app, "coa_controls_list", |ctx| {
        auth::require(ctx)?;
        coa::controls(ctx)
    })
    .await
}
