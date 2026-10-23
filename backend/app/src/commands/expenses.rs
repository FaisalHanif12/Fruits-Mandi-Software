//! The خرچہ screen (PRD 16.2, 16.3, FR-7.67, FR-16.8, plan 17): shop expense, a partner's personal
//! spending and its return, with the effect shown before the save. Every command needs the
//! signed-in Owner. There is no mode, bank or cheque (PRD v1.6): everything posts روکڑ.

use mandi_services::core::expenses::{
    ExpenseHeads, ExpensePreview, ExpensePreviewInput, PartnerTakes, PartnerTakesInput,
    PersonalExpenseInput, PersonalReturnInput, RepeatLastMonth, RepeatLastMonthInput,
    ShopExpenseInput,
};
use mandi_services::core::vouchers::VoucherSaved;
use mandi_services::{auth, masters, partners, vouchers};

use super::run;
use crate::error::UiError;

/// Saves a دکان خرچہ (PRD 16.2): the head بنام, روکڑ جمع, split between the partners by the shares
/// of the date (E2E-20). One transaction. Errors: the date codes on `date`;
/// `ERR_EXPENSE_HEAD_REQUIRED` and `ERR_NOT_FOUND` on `expense_head_id`; the money codes on
/// `amount`; the text codes on `paid_to` and `narration`; `ERR_SHARES_NOT_100`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
#[tauri::command]
pub async fn vouchers_save_shop_expense<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ShopExpenseInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_shop_expense", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_shop_expense(ctx, &actor, &input)
    })
    .await
}

/// Saves a ذاتی خرچہ (PRD 16.3): the partner's drawings account بنام, روکڑ جمع. Never split.
/// Errors: as `vouchers_save_shop_expense`, with `ERR_PARTNER_REQUIRED` and `ERR_NOT_FOUND` on
/// `partner_id`, `ERR_REASON_REQUIRED` on `reason`, `ERR_RETURN_BY_BEFORE_ENTRY` on `return_by`
/// (PRD FR-16.9).
#[tauri::command]
pub async fn vouchers_save_personal_expense<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PersonalExpenseInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_personal_expense", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_personal_expense(ctx, &actor, &input)
    })
    .await
}

/// Saves a ذاتی خرچہ واپسی (PRD 16.3): روکڑ بنام, drawings جمع. Errors: the date and money codes;
/// `ERR_PARTNER_REQUIRED` and `ERR_NOT_FOUND` on `partner_id`; `ERR_RETURN_EXCEEDS_OPEN` (value
/// `open_paisa`) on `amount`.
#[tauri::command]
pub async fn vouchers_save_personal_return<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PersonalReturnInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_personal_return", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_personal_return(ctx, &actor, &input)
    })
    .await
}

/// Works out any of the three entries for the screen (PRD FR-16.8): each partner's part of a shop
/// expense (C11), or the chosen partner's کل رقم now and after. Nothing is saved. The same function
/// as the save (Code Standards B5).
#[tauri::command]
pub async fn vouchers_preview_expense<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ExpensePreviewInput,
) -> Result<ExpensePreview, UiError> {
    run(app, "vouchers_preview_expense", move |ctx| {
        auth::require(ctx)?;
        vouchers::preview_expense(ctx, &input)
    })
    .await
}

/// The money a partner took, oldest first, with what was returned against each, one page (PRD
/// FR-16.2, FR-19.11). Read only. Errors: `ERR_NOT_FOUND` on `partner_id`.
#[tauri::command]
pub async fn partner_takes<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerTakesInput,
) -> Result<PartnerTakes, UiError> {
    run(app, "partner_takes", move |ctx| {
        auth::require(ctx)?;
        partners::takes(ctx, &input)
    })
    .await
}

/// This month's regular expenses proposed from last month, for the user to check and save (PRD
/// FR-7.62). Nothing is saved. Errors: `ERR_FORMAT_INVALID` on `month`.
#[tauri::command]
pub async fn vouchers_repeat_last_month_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RepeatLastMonthInput,
) -> Result<RepeatLastMonth, UiError> {
    run(app, "vouchers_repeat_last_month_preview", move |ctx| {
        auth::require(ctx)?;
        vouchers::repeat_last_month_preview(ctx, &input)
    })
    .await
}

/// The active expense heads, most used first (PRD FR-7.62). Read only.
#[tauri::command]
pub async fn masters_expense_heads<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<ExpenseHeads, UiError> {
    run(app, "masters_expense_heads", move |ctx| {
        auth::require(ctx)?;
        masters::expense_heads(ctx)
    })
    .await
}
