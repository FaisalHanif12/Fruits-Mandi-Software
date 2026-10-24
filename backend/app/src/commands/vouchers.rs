//! Vouchers (PRD 15, plans 10 and 15): پیشگی, وصولی and ادائیگی on one screen with a type list
//! (PRD FR-7.67). The other kinds use the same list and numbering in later plans. A voucher is
//! never deleted; it is cancelled by a reversal.

use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesInput, AdvancesPreview, AdvancesReport, AdvancesReportInput,
    AdvancesSaved, PaymentInput, ReceiptInput, ReceiptsInput, ReceiptsSaved, VoucherCancelInput,
    VoucherCancelled, VoucherEditInput, VoucherGetInput, VoucherPreview, VoucherPreviewInput,
    VoucherRow, VoucherSaved, VouchersList, VouchersListInput,
};
use mandi_services::{auth, vouchers};

use super::run;
use crate::error::UiError;

/// Gives an advance to a seller (PRD 15.4): seller بنام (an advance line), روکڑ جمع.
/// One transaction: the voucher, its journal entry, the audit row. Errors: the validate codes
/// (`date`, `amount`), `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE` (`date`), `ERR_NOT_A_SELLER`,
/// `ERR_ACCOUNT_INACTIVE`, `ERR_NOT_FOUND` (`seller_account_id`), `ERR_FIELD_NOT_ALLOWED` (an
/// old client that still sends `mode`, `bank_account_id`, `cheque_no` or `cheque_date`; the field
/// is the name sent and nothing is posted).
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
) -> Result<VouchersList, UiError> {
    run(app, "vouchers_list", move |ctx| {
        auth::require(ctx)?;
        vouchers::list(ctx, &input)
    })
    .await
}

/// Receives money (PRD 15.2): روکڑ بنام, the account جمع. From a buyer it settles his
/// oldest open bills first and the extra stays as his credit (FR-15.4, FR-15.5); for a
/// پختہ کھاتہ buyer it names the one-time person (FR-13.7). From a seller it is an advance
/// returned (FR-15.12): it adjusts his open advances, oldest first, then settles what he owes.
/// One transaction: the voucher, its journal entry, the matching rows and the audit row. Errors:
/// the validate codes (`date`, `amount`), `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE` (`date`),
/// `ERR_NOT_FOUND`, `ERR_ACCOUNT_INACTIVE`, `ERR_NOT_A_BUYER` (`account_id`),
/// `ERR_PERSON_REQUIRED` (`person_key`), `ERR_FIELD_NOT_ALLOWED` (a removed field was sent),
/// `ERR_RETURN_EXCEEDS_OPEN` (value `open_paisa`, field `amount`, a seller only).
#[tauri::command]
pub async fn vouchers_save_receipt<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ReceiptInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_receipt", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_receipt(ctx, &actor, &input)
    })
    .await
}

/// Pays money out (PRD 15.3): the account بنام, روکڑ جمع. To a seller above his open
/// payable it saves with a warning and the extra shows as "بیوپاری کے ذمے" (FR-15.7). To a
/// buyer it is cash lent: a reason is required and it becomes one of his bills (FR-15.11). A
/// payment never adjusts advances. Errors: as `vouchers_save_receipt`, plus
/// `ERR_PAYMENT_TO_BUYER_NEEDS_REASON` (`reason`) and `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value
/// `cash_after_paisa`) when the payment takes cash below 0 and `confirm_cash_below_zero` is
/// false.
#[tauri::command]
pub async fn vouchers_save_payment<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PaymentInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_payment", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_payment(ctx, &actor, &input)
    })
    .await
}

/// Checks any of the three entries and works out what saving it would do: the balance before
/// and after, the bills it settles (a buyer's receipt), the advances it adjusts (a seller's
/// receipt), the payable and the warning of FR-15.7 (a seller's payment), the cash after it and
/// the right-type hint of FR-7.68. Nothing is saved. The same function as the save (Code
/// Standards B5). With `replacing_voucher_id` it shows an edit: the old posting is taken back
/// first.
#[tauri::command]
pub async fn vouchers_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: VoucherPreviewInput,
) -> Result<VoucherPreview, UiError> {
    run(app, "vouchers_preview", move |ctx| {
        auth::require(ctx)?;
        vouchers::preview(ctx, &input)
    })
    .await
}

/// Edits a voucher (PRD FR-15.3, Architecture 12): while its day is open, the posting is taken
/// back by a reversal on the same day and the new version is posted, in one transaction. The
/// matching rows of the old posting are switched off and the party is matched again, oldest
/// first. The voucher keeps its number and its type. Errors: every error of the type's save, plus
/// `ERR_NOT_FOUND` (`id`), `ERR_VERSION_CHANGED`, `ERR_VOUCHER_CANCELLED`, `ERR_DATE_CLOSED`
/// (either day), `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
#[tauri::command]
pub async fn vouchers_edit<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: VoucherEditInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_edit", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::edit(ctx, &actor, &input)
    })
    .await
}

/// Cancels a voucher (PRD FR-15.3): the posting is taken back by a reversal and the voucher is
/// marked منسوخ. It keeps its number. A buyer's receipt that was matched to bills becomes
/// credit again and is matched to his oldest bills (FR-12.22). Errors: `ERR_NOT_FOUND` (`id`),
/// `ERR_VERSION_CHANGED`, `ERR_VOUCHER_CANCELLED`, `ERR_REASON_REQUIRED` (`reason`),
/// `ERR_DATE_CLOSED`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
#[tauri::command]
pub async fn vouchers_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: VoucherCancelInput,
) -> Result<VoucherCancelled, UiError> {
    run(app, "vouchers_cancel", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::cancel(ctx, &actor, &input)
    })
    .await
}

/// One voucher with its stored values, its `version` for an edit or a cancel, and whether it can
/// still be edited. Read only. Errors: `ERR_NOT_FOUND` on `id`.
#[tauri::command]
pub async fn vouchers_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: VoucherGetInput,
) -> Result<VoucherRow, UiError> {
    run(app, "vouchers_get", move |ctx| {
        auth::require(ctx)?;
        vouchers::get(ctx, input.id)
    })
    .await
}

/// Saves the receipts of the collection round (PRD FR-7.61): each is its own posting event in
/// its own transaction, a row with amount 0 or empty is skipped, and the answer has one result
/// per row. A row that fails does not stop the others.
#[tauri::command]
pub async fn vouchers_save_receipts_batch<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ReceiptsInput,
) -> Result<ReceiptsSaved, UiError> {
    run(app, "vouchers_save_receipts_batch", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_receipts_batch(ctx, &actor, &input)
    })
    .await
}

/// The advances of sellers (PRD R26 data): given, returned (a seller's وصولی), adjusted (by his
/// sales) and open, with the last day, for one seller or all. Read only. Errors: the date codes
/// on `as_of`, `ERR_NOT_FOUND` and `ERR_NOT_A_SELLER` on `seller_id`.
#[tauri::command]
pub async fn advances_report<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AdvancesReportInput,
) -> Result<AdvancesReport, UiError> {
    run(app, "advances_report", move |ctx| {
        auth::require(ctx)?;
        vouchers::advances_report(ctx, &input)
    })
    .await
}
