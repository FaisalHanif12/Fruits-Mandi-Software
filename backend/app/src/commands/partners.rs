//! Partners and their shares (PRD 17.1 to 17.3, plan 11). Every command needs the signed-in
//! Owner. Partners do not log in (PRD FR-2.4), so no user is made for a partner.
//!
//! Once a partner code is set (PRD FR-17.11), every command here except `partners_list` also needs
//! the partner section to be open and answers `ERR_PARTNER_LOCKED` when it is not. `partners_list`
//! gives the names only while it is locked (`partners_locked`, every `share_bps` 0).

use mandi_reports::reports::{TablePrintData, partner_statement_month_print_data};
use mandi_services::core::partners::{
    PartnerCreateInput, PartnerDetail, PartnerOverviewRow, PartnerRow, PartnerUpdateInput,
    PartnersCheckSharesInput, PartnersGetInput, PartnersList, PartnersSetSharesInput, SharesCheck,
};
use mandi_services::core::partners::{PartnerOpenVentures, PartnersOpenVenturesInput};
use mandi_services::core::statement::{
    PartnerStatement, PartnerStatementDay, PartnerStatementMonth, PartnersStatementDayInput,
    PartnersStatementInput, PartnersStatementMonthInput, PartnersStatementMonthPrintInput,
};
use mandi_services::partner_code;
use mandi_services::{auth, partners};

use super::reports_engine::engine_error;
use super::run;
use crate::error::UiError;

/// Every partner, active ones first, then by id. `share_bps` is the share on today's date. While
/// the partner section is locked (PRD FR-17.11) it names them only: every `share_bps` is 0 and
/// `partners_locked` is true.
#[tauri::command]
pub async fn partners_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<PartnersList, UiError> {
    run(app, "partners_list", move |ctx| {
        auth::require(ctx)?;
        partners::list_view(ctx)
    })
    .await
}

/// One partner with his share history. Errors: `ERR_NOT_FOUND` (`id`).
#[tauri::command]
pub async fn partners_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersGetInput,
) -> Result<PartnerDetail, UiError> {
    run(app, "partners_get", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::get(ctx, input.id)
    })
    .await
}

/// Adds a partner with his capital and drawings accounts. He has no share until
/// `partners_set_shares`. Errors: the name codes (`name_ur`, `name_en`),
/// `ERR_SPELLING_UNCONFIRMED`, the money codes (`opening_capital`), `ERR_DATE_CLOSED` for the
/// opening, `ERR_TOO_MANY_PARTNERS` (`max`).
#[tauri::command]
pub async fn partners_create<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerCreateInput,
) -> Result<PartnerRow, UiError> {
    run(app, "partners_create", move |ctx| {
        let actor = auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::create(ctx, &actor, &input)
    })
    .await
}

/// Changes a partner's names or makes him inactive. Errors: `ERR_VERSION_CHANGED`,
/// `ERR_NOT_FOUND` (`id`), the name codes, `ERR_PARTNER_HAS_SHARE`.
#[tauri::command]
pub async fn partners_update<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerUpdateInput,
) -> Result<PartnerRow, UiError> {
    run(app, "partners_update", move |ctx| {
        let actor = auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::update(ctx, &actor, &input)
    })
    .await
}

/// Sets the shares of all active partners from a day. Errors: `ERR_DATE_INVALID` and
/// `ERR_SHARE_BACKDATED` (`effective_from`), `ERR_PARTNER_NOT_LISTED` (`shares`), the percent
/// codes (`shares[i].share`), `ERR_SHARES_NOT_100` (`shares`, value `total_bps`).
#[tauri::command]
pub async fn partners_set_shares<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersSetSharesInput,
) -> Result<Vec<PartnerRow>, UiError> {
    run(app, "partners_set_shares", move |ctx| {
        let actor = auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::set_shares(ctx, &actor, &input)
    })
    .await
}

/// The running total of typed shares. Read only; nothing is saved.
///
/// This one command is not behind the partner code (plan 81, PRD FR-17.11). It reads only the
/// percent text the screen sends and the ids of the active partners, whose names are open in every
/// picker, and it answers with the total of what was typed. It holds no stored partner figure. The
/// new garden and new trip forms are outside the partner section and ask it for the share total, so
/// a locked section must not refuse it. Setting the shares (`partners_set_shares`) stays behind the
/// code.
#[tauri::command]
pub async fn partners_check_shares<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersCheckSharesInput,
) -> Result<SharesCheck, UiError> {
    run(app, "partners_check_shares", move |ctx| {
        auth::require(ctx)?;
        partners::check(ctx, &input)
    })
    .await
}

/// One row per active partner: today, month and year income share, month expense share, profit,
/// money taken and capital balance.
#[tauri::command]
pub async fn partners_overview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<PartnerOverviewRow>, UiError> {
    run(app, "partners_overview", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::overview(ctx)
    })
    .await
}

/// A partner's statement (PRD FR-17.9, FR-17.10): one dated line for every amount that reaches him,
/// with the running کل رقم, one page. Read only. Errors: `ERR_NOT_FOUND` on `partner_id`; the date
/// codes on `from` and `to`.
#[tauri::command]
pub async fn partners_statement<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersStatementInput,
) -> Result<PartnerStatement, UiError> {
    run(app, "partners_statement", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::statement(ctx, &input)
    })
    .await
}

/// A partner's statement for one month: his کل رقم at the start, a row for each day with a
/// movement by what it was, the month's totals and his کل رقم at the end (plan 76 point 11). Read
/// only, behind the partner code (PRD FR-17.11).
#[tauri::command]
pub async fn partners_statement_month<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersStatementMonthInput,
) -> Result<PartnerStatementMonth, UiError> {
    run(app, "partners_statement_month", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::statement_month(ctx, &input)
    })
    .await
}

/// Every movement of one day of a partner's statement, with its source and his کل رقم after the
/// day (plan 76 point 11). Behind the partner code.
#[tauri::command]
pub async fn partners_statement_day<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersStatementDayInput,
) -> Result<PartnerStatementDay, UiError> {
    run(app, "partners_statement_day", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::statement_day(ctx, &input)
    })
    .await
}

/// The print of one month of a partner's statement in the print language, with the closing in
/// words (plan 76 point 11). Behind the partner code.
#[tauri::command]
pub async fn partners_statement_month_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersStatementMonthPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "partners_statement_month_print_data", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        let month = partners::statement_month(
            ctx,
            &PartnersStatementMonthInput {
                partner_id: input.partner_id,
                month: input.month.clone(),
            },
        )?;
        partner_statement_month_print_data(ctx.db(), &month, input.language, input.paper)
            .map_err(engine_error)
    })
    .await
}

/// The open gardens and trips a partner is in, listed on demand (plan 76 point 11). Behind the
/// partner code.
#[tauri::command]
pub async fn partners_open_ventures<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnersOpenVenturesInput,
) -> Result<PartnerOpenVentures, UiError> {
    run(app, "partners_open_ventures", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        partners::open_ventures(ctx, &input)
    })
    .await
}
