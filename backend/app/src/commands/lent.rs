//! ادھار رقم: money lent to staff and others (PRD 16.5, plan 17). Every command needs the
//! signed-in Owner. Reads only read (Code Standards B29).

use mandi_services::core::lent::{
    AttentionOverdue, LentList, LentListInput, LentPerson, LentPersonCreateInput, LentReport,
    LentReportInput, MoneyLentInput, MoneyLentReturnInput,
};
use mandi_services::core::vouchers::VoucherSaved;
use mandi_services::{auth, lent, vouchers};

use super::run;
use crate::error::UiError;

/// Saves money lent (PRD 16.5): the person's account بنام, روکڑ جمع. One transaction. Errors: the
/// date and money codes; `ERR_LENT_PERSON_REQUIRED` and `ERR_NOT_FOUND` on `person_account_id`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_RETURN_BY_BEFORE_ENTRY` on `return_by` (PRD FR-16.9);
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
#[tauri::command]
pub async fn vouchers_save_money_lent<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MoneyLentInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_money_lent", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_money_lent(ctx, &actor, &input)
    })
    .await
}

/// Saves a return of money lent (PRD FR-16.4): in cash (روکڑ بنام, person جمع) or from salary (the
/// salary head بنام, person جمع, split as a shop expense). Errors: as `vouchers_save_money_lent`
/// without `reason` and `return_by`; `ERR_RETURN_EXCEEDS_OPEN` (value `open_paisa`) on `amount`;
/// `ERR_NOT_FOUND` on `salary_head_id`.
#[tauri::command]
pub async fn vouchers_save_money_lent_return<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MoneyLentReturnInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_money_lent_return", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_money_lent_return(ctx, &actor, &input)
    })
    .await
}

/// Makes the account of a person who can be lent money (control 0103; the `F4` of the screen).
/// Errors: the name codes on `name_ur` and `name_en`, `ERR_SPELLING_UNCONFIRMED`, the phone code
/// on `phone`, `ERR_NOT_FOUND` on `staff_id`.
#[tauri::command]
pub async fn lent_person_create<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LentPersonCreateInput,
) -> Result<LentPerson, UiError> {
    run(app, "lent_person_create", move |ctx| {
        let actor = auth::require(ctx)?;
        lent::person_create(ctx, &actor, &input)
    })
    .await
}

/// One person's loans, oldest first, with what was returned against each, one page (PRD FR-16.5).
/// Read only. Errors: `ERR_NOT_FOUND` on `person_id`.
#[tauri::command]
pub async fn lent_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LentListInput,
) -> Result<LentList, UiError> {
    run(app, "lent_list", move |ctx| {
        auth::require(ctx)?;
        lent::list(ctx, &input)
    })
    .await
}

/// R32 data: everyone who was lent money as of a day, one page, with the totals over all people.
/// Read only. Errors: the date codes on `as_of`.
#[tauri::command]
pub async fn lent_report<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LentReportInput,
) -> Result<LentReport, UiError> {
    run(app, "lent_report", move |ctx| {
        auth::require(ctx)?;
        lent::report(ctx, &input)
    })
    .await
}

/// The overdue loans and partner takes past their return-by day (PRD FR-7.44). Read only; nothing
/// is written (Code Standards B29).
#[tauri::command]
pub async fn assist_attention_overdue<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AttentionOverdue, UiError> {
    run(app, "assist_attention_overdue", move |ctx| {
        auth::require(ctx)?;
        lent::attention_overdue(ctx)
    })
    .await
}
