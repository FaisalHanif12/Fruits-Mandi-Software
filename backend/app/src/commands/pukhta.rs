//! The one-time people and the register of the پختہ کھاتہ accounts (PRD FR-7.29, Section 13,
//! plan 13). Read only (Code Standards B29).

use mandi_services::core::bikri::{
    PukhtaPeopleInput, PukhtaPerson, PukhtaRegister, PukhtaRegisterInput,
};
use mandi_services::core::vouchers::{MadeRegular, MakeRegularInput};
use mandi_services::{auth, bikri, vouchers};

use super::run;
use crate::error::UiError;

/// One-time people already recorded for the seller or the buyer side, newest first (at most 10),
/// matched by the typed text in either name or the phone (PRD FR-7.29). The screen shows them
/// first in the one merged list, then the Urdu spellings of the typed word. Picking one reuses
/// his exact names and phone, so his lines stay under one person key. Read only.
#[tauri::command]
pub async fn search_pukhta_people<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PukhtaPeopleInput,
) -> Result<Vec<PukhtaPerson>, UiError> {
    run(app, "search_pukhta_people", move |ctx| {
        auth::require(ctx)?;
        bikri::search_people(ctx, &input)
    })
    .await
}

/// The پختہ کھاتہ register (PRD FR-13.6, R9): every line of a one-time buyer and every lot of a
/// one-time seller, with date, بکری number, side, name, phone, item, نگ, amount, commission and
/// paid or due with the open amount (from the matching rows). Read only. Errors: the date codes
/// on `from` and `to`.
#[tauri::command]
pub async fn pukhta_register<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PukhtaRegisterInput,
) -> Result<PukhtaRegister, UiError> {
    run(app, "pukhta_register", move |ctx| {
        auth::require(ctx)?;
        bikri::register(ctx, &input)
    })
    .await
}

/// Makes a one-time person a regular account (PRD FR-13.8): creates the account with the rules
/// of the account form and moves his open balance to it with a جنرل entry, in one transaction,
/// with an audit row. Errors: every error of the account form (at `account_input.<field>`),
/// `ERR_NOT_FOUND` (`person_key`), `ERR_FORMAT_INVALID` (`side` against the account type).
#[tauri::command]
pub async fn pukhta_make_regular<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MakeRegularInput,
) -> Result<MadeRegular, UiError> {
    run(app, "pukhta_make_regular", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::make_regular(ctx, &actor, &input)
    })
    .await
}
