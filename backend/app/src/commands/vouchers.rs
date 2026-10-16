//! Vouchers (PRD 15, plan 10). Plan 10 has the پیشگی voucher; the other kinds use the same list
//! and numbering in later plans. A voucher is never deleted; it is cancelled by a reversal.

use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesInput, AdvancesPreview, AdvancesSaved, VoucherRow, VoucherSaved,
    VouchersListInput,
};
use mandi_services::{auth, vouchers};

use super::run;
use crate::error::UiError;

/// Gives an advance to a seller (PRD 15.4): seller بنام (an advance line), cash or bank جمع.
/// One transaction: the voucher, its journal entry, the audit row. Errors: the validate codes
/// (`date`, `amount`, `cheque_no`, `cheque_date`), `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE`
/// (`date`), `ERR_NOT_A_SELLER`, `ERR_ACCOUNT_INACTIVE`, `ERR_NOT_FOUND` (`seller_account_id`,
/// `bank_account_id`), `ERR_REQUIRED` (`bank_account_id` for bank and cheque).
#[tauri::command]
pub async fn vouchers_save_advance<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AdvanceInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_advance", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_advance(ctx, &actor, &input)
    })
    .await
}

/// Checks the running list of advances (PRD FR-15.1) and gives each line's current and new
/// balance and the total. Nothing is saved. The same function as the save (Code Standards B5).
#[tauri::command]
pub async fn vouchers_preview_advances<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AdvancesInput,
) -> Result<AdvancesPreview, UiError> {
    run(app, "vouchers_preview_advances", move |ctx| {
        auth::require(ctx)?;
        vouchers::preview_advances(ctx, &input)
    })
    .await
}

/// Saves the running list (PRD FR-15.1): each voucher is its own posting event in its own
/// transaction, and the answer has one result per line. A line that fails does not stop the
/// others.
#[tauri::command]
pub async fn vouchers_save_advances<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AdvancesInput,
) -> Result<AdvancesSaved, UiError> {
    run(app, "vouchers_save_advances", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_advances(ctx, &actor, &input)
    })
    .await
}

/// The vouchers of a kind in a date range, by day and number.
#[tauri::command]
pub async fn vouchers_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: VouchersListInput,
) -> Result<Vec<VoucherRow>, UiError> {
    run(app, "vouchers_list", move |ctx| {
        auth::require(ctx)?;
        vouchers::list(ctx, &input)
    })
    .await
}
