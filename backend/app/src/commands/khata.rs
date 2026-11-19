//! الگ کھاتہ (PRD 16.6, plan 17): paid out and received on one screen, with the khata's statement.
//! Every command needs the signed-in Owner. A khata is never income or expense, never split and
//! never in a partner's total.

use mandi_services::core::khata::{
    KhataAccount, KhataCreateRequest, KhataInInput, KhataOutInput, KhataSetLinkInput,
    KhataStatement, KhataStatementInput,
};
use mandi_services::core::vouchers::VoucherSaved;
use mandi_services::{auth, khata, vouchers};

use super::run;
use crate::error::UiError;

/// Makes a separate khata (an account under control 0305; the `F4` of the screen), with its link
/// (plan 76 point 5; no link is the firm's, as before). Errors: the name codes on `name_ur` and
/// `name_en`, `ERR_SPELLING_UNCONFIRMED`, the text code on `note`; `ERR_LINK_PARTNER_REQUIRED`,
/// `ERR_NOT_FOUND` and `ERR_PARTNER_INACTIVE` on `link.partner_id`; `ERR_SHARES_NOT_100` on `link`.
#[tauri::command]
pub async fn khata_create<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: KhataCreateRequest,
) -> Result<KhataAccount, UiError> {
    run(app, "khata_create", move |ctx| {
        let actor = auth::require(ctx)?;
        khata::create_linked(ctx, &actor, &input)
    })
    .await
}

/// Changes the link of a khata (plan 76 point 5, PRD FR-16.6). Only while its balance is 0.
/// Errors: `ERR_NOT_FOUND` on `khata_account_id`; `ERR_LINK_LOCKED` on `link`;
/// `ERR_LINK_PARTNER_REQUIRED`, `ERR_NOT_FOUND` and `ERR_PARTNER_INACTIVE` on `link.partner_id`;
/// `ERR_SHARES_NOT_100` on `link`.
#[tauri::command]
pub async fn khata_set_link<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: KhataSetLinkInput,
) -> Result<KhataAccount, UiError> {
    run(app, "khata_set_link", move |ctx| {
        let actor = auth::require(ctx)?;
        khata::set_link(ctx, &actor, &input)
    })
    .await
}

/// Saves money paid out (PRD FR-16.6): the khata بنام (minus), روکڑ جمع. Errors: the date and money
/// codes; `ERR_KHATA_REQUIRED` and `ERR_NOT_FOUND` on `khata_account_id`; `ERR_REASON_REQUIRED` on
/// `reason`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
#[tauri::command]
pub async fn vouchers_save_khata_out<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: KhataOutInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_khata_out", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_khata_out(ctx, &actor, &input)
    })
    .await
}

/// Saves money received (PRD FR-16.6): روکڑ بنام, the khata جمع (plus). Errors: as
/// `vouchers_save_khata_out` without the cash confirm.
#[tauri::command]
pub async fn vouchers_save_khata_in<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: KhataInInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_khata_in", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_khata_in(ctx, &actor, &input)
    })
    .await
}

/// A khata's entries in date order with the signed running balance, one page (PRD FR-16.6,
/// FR-19.11). Read only. Errors: `ERR_NOT_FOUND` on `khata_account_id`; the date codes.
#[tauri::command]
pub async fn khata_statement<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: KhataStatementInput,
) -> Result<KhataStatement, UiError> {
    run(app, "khata_statement", move |ctx| {
        auth::require(ctx)?;
        khata::statement(ctx, &input)
    })
    .await
}
