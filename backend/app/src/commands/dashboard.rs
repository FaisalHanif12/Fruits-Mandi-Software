//! The dashboard and the attention list (plan 19, PRD 9, FR-9.1, FR-7.44, NFR-SPD-2, NFR-SPD-22).
//! Both commands need the signed-in Owner (NFR-SEC-4) and only read (Code Standards B29).
//!
//! The work is in `mandi_reports::dashboard`; a command checks sign-in, tells it what the backup
//! service knows (the backup state is kept by that service, not in the database), and calls one
//! function.

use mandi_reports::dashboard::{
    self, AttentionPanel, BackupFacts, BuyerDuesInput, BuyerDuesPage, BuyerDuesPrintInput,
    CashSalesDay, CashSalesDayInput, CashSalesDayPrintInput, CashSalesMonth, CashSalesMonthInput,
    CashSalesMonthPrintInput, CommissionDay, CommissionDayInput, CommissionMonth,
    CommissionMonthInput, CommissionPrintInput, Dashboard, LentOpenInput, LentOpenPage,
    LentOpenPrintInput, SalesDay, SalesDayInput, SalesPeriod, SalesPeriodInput, SalesPrintInput,
    SellerDuesInput, SellerDuesPage, SellerDuesPrintInput, TotalDues, TotalDuesPrintInput,
    TotalWorthInput, TotalWorthPage, TotalWorthPrintInput,
};
use mandi_reports::reports::TablePrintData;
use mandi_services::auth;
use mandi_services::partner_code;

use mandi_services::assist;
use serde::Deserialize;
use ts_rs::TS;

use super::reports_engine::engine_error;
use super::run;
use crate::backup_runtime::BackupRuntime;
use crate::error::UiError;

/// Input of `assist_snooze`: the `item_key` of the panel item to hide.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AttentionSnoozeInput {
    /// For example `seller:12`, `buyer:7`, `payable:3` or `kind:drafts`.
    pub item_key: String,
}

/// What the backup service says about the cloud copy. If the service is not running, the cloud
/// copy counts as off: no old-backup item is made up for a copy that cannot run.
pub(super) fn backup_facts<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> BackupFacts {
    let status = BackupRuntime::of(app)
        .ok()
        .and_then(|runtime| runtime.service().status().ok());
    match status {
        Some(status) => BackupFacts {
            cloud_on: status.cloud_enabled,
            cloud_age_hours: status.cloud_age_hours,
        },
        None => BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    }
}

/// Everything the dashboard shows: the tiles, the partner strip, today's بکری, the oldest buyer
/// dues, the two charts and the attention list (آج کے کام). Read only. The last backup and the
/// daily health result come from `backup_status` and `health_last`.
#[tauri::command]
pub async fn dashboard_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Dashboard, UiError> {
    let backup = backup_facts(&app);
    run(app, "dashboard_get", move |ctx| {
        auth::require(ctx)?;
        let mut shown = dashboard::get(ctx.db(), backup).map_err(engine_error)?;
        if partner_code::is_locked(ctx)? {
            shown.hide_partners();
        }
        Ok(shown)
    })
    .await
}

/// The bell panel (PRD FR-7.44, v1.61, plan 100): the top 5 sellers to pay, the top 5 buyers to
/// collect from, the payables still to pay and the other reminders, without the items hidden by
/// `assist_snooze`. Read only. The bell asks again after every save.
#[tauri::command]
pub async fn assist_attention<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AttentionPanel, UiError> {
    let backup = backup_facts(&app);
    run(app, "assist_attention", move |ctx| {
        let actor = auth::require(ctx)?;
        let hidden = assist::snoozed_keys(ctx, &actor)?;
        let mut shown =
            dashboard::attention_panel(ctx.db(), backup, &hidden).map_err(engine_error)?;
        if partner_code::is_locked(ctx)? {
            shown.hide_partners();
        }
        Ok(shown)
    })
    .await
}

/// Hides one item of the bell panel until the next بکری save ("بعد میں", plan 100 J). It writes the
/// Owner's own preference `attention.snoozed` and nothing else. Errors: `ERR_REQUIRED` and
/// `ERR_FORMAT_INVALID` on `item_key` (not a key the panel gives).
#[tauri::command]
pub async fn assist_snooze<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AttentionSnoozeInput,
) -> Result<(), UiError> {
    run(app, "assist_snooze", move |ctx| {
        let actor = auth::require(ctx)?;
        assist::snooze(ctx, &actor, &input.item_key)
    })
    .await
}

/// One page of the rows of one side of کل مالیت, with the totals of every source and the net
/// (plan 100 B, PRD FR-9.10). Read only. Errors: `ERR_OUT_OF_RANGE` on `limit` above 100.
#[tauri::command]
pub async fn dashboard_total_worth<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TotalWorthInput,
) -> Result<TotalWorthPage, UiError> {
    run(app, "dashboard_total_worth", move |ctx| {
        auth::require(ctx)?;
        dashboard::total_worth(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of کل مالیت: both sides with every row, a total for each source and side, and the net
/// (plan 100 B, PRD FR-9.10, FR-19.10). Read only.
#[tauri::command]
pub async fn dashboard_total_worth_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TotalWorthPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_total_worth_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::total_worth_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// One month of the commission page: a row for each day with a figure and the month totals
/// (plan 76 point 7). The partner columns are hidden while the partner section is locked (PRD
/// FR-17.11). Read only.
#[tauri::command]
pub async fn dashboard_commission_month<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CommissionMonthInput,
) -> Result<CommissionMonth, UiError> {
    run(app, "dashboard_commission_month", move |ctx| {
        auth::require(ctx)?;
        let mut shown = dashboard::commission_month(ctx.db(), &input).map_err(engine_error)?;
        if partner_code::is_locked(ctx)? {
            shown.hide_partners();
        }
        Ok(shown)
    })
    .await
}

/// The postings behind one day's commission (plan 76 point 7). Read only.
#[tauri::command]
pub async fn dashboard_commission_day<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CommissionDayInput,
) -> Result<CommissionDay, UiError> {
    run(app, "dashboard_commission_day", move |ctx| {
        auth::require(ctx)?;
        let mut shown = dashboard::commission_day(ctx.db(), &input).map_err(engine_error)?;
        if partner_code::is_locked(ctx)? {
            shown.hide_partners();
        }
        Ok(shown)
    })
    .await
}

/// The print of one commission month, with every row, in the print language (plan 76 point 7).
#[tauri::command]
pub async fn dashboard_commission_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CommissionPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_commission_print_data", move |ctx| {
        auth::require(ctx)?;
        let show_partners = !partner_code::is_locked(ctx)?;
        dashboard::commission_month_print_data(ctx.db(), &input, show_partners)
            .map_err(engine_error)
    })
    .await
}

/// The sales of a period: a row for each day (a row for each month in a year) with a sale, and the
/// totals over the whole period (plan 83, PRD FR-9.9). The tile آج کی بکری opens it. Narrowed to one
/// party, one item or some goods when asked. Read only. Errors: `ERR_REQUIRED`, `ERR_FORMAT_INVALID`,
/// `ERR_DATE_INVALID`, `ERR_DATE_IN_FUTURE` and `ERR_DATE_RANGE` on `period.*`; `ERR_NOT_FOUND`,
/// `ERR_WRONG_PARTY_TYPE` and `ERR_OUT_OF_RANGE` on `party.*`; `ERR_NOT_FOUND` on `item_id`;
/// `ERR_OUT_OF_RANGE` on `offset` and `limit`.
#[tauri::command]
pub async fn dashboard_sales_period<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SalesPeriodInput,
) -> Result<SalesPeriod, UiError> {
    run(app, "dashboard_sales_period", move |ctx| {
        auth::require(ctx)?;
        dashboard::sales_period(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The بکری sold on one day, with their figures of that day (plan 83, PRD FR-9.9). Read only.
/// Errors: `ERR_REQUIRED`, `ERR_DATE_INVALID` and `ERR_DATE_IN_FUTURE` on `date`; the party and item
/// errors of `dashboard_sales_period`; `ERR_OUT_OF_RANGE` on `offset` and `limit`.
#[tauri::command]
pub async fn dashboard_sales_day<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SalesDayInput,
) -> Result<SalesDay, UiError> {
    run(app, "dashboard_sales_day", move |ctx| {
        auth::require(ctx)?;
        dashboard::sales_day(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of the sales page of a period, with every row, in the print language (plan 83).
#[tauri::command]
pub async fn dashboard_sales_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SalesPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_sales_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::sales_period_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// A page of the buyers who owe, with the total of all that match (plan 76 point 9). Read only.
#[tauri::command]
pub async fn dashboard_buyer_dues<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BuyerDuesInput,
) -> Result<BuyerDuesPage, UiError> {
    run(app, "dashboard_buyer_dues", move |ctx| {
        auth::require(ctx)?;
        dashboard::buyer_dues(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of the buyer dues, every buyer of the filter (plan 76 point 9).
#[tauri::command]
pub async fn dashboard_buyer_dues_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BuyerDuesPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_buyer_dues_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::buyer_dues_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// A page of the sellers, with the totals of both sides (plan 79 item 2a, PRD FR-9.6). Both seller
/// tiles open it: `side` `shop_owes` for بیوپاریوں کو ادائیگی باقی, `seller_owes` for بیوپاریوں کے ذمے.
/// A seller's history opens with `party_ledgers_statement_rows`. Read only. Errors:
/// `ERR_OUT_OF_RANGE` on `sort.column`.
#[tauri::command]
pub async fn dashboard_seller_dues<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SellerDuesInput,
) -> Result<SellerDuesPage, UiError> {
    run(app, "dashboard_seller_dues", move |ctx| {
        auth::require(ctx)?;
        dashboard::seller_dues(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of the seller dues, every seller of the filter (plan 79 item 2a).
#[tauri::command]
pub async fn dashboard_seller_dues_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SellerDuesPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_seller_dues_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::seller_dues_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The four parts of کل اگراہی (buyers, sellers, dead amount, others) with their totals and the
/// grand total, which is the tile (plan 79 item 2b, PRD FR-9.7). The rows of each part come from
/// that part's page. Read only.
#[tauri::command]
pub async fn dashboard_total_dues<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<TotalDues, UiError> {
    run(app, "dashboard_total_dues", move |ctx| {
        auth::require(ctx)?;
        dashboard::total_dues_parts(ctx.db()).map_err(engine_error)
    })
    .await
}

/// A page of the people who borrowed and still owe (ادھار رقم باقی, the others part of کل اگراہی),
/// with the total of all that match (plan 79 items 2b and 3). A person's loans open with `lent_list`.
/// Read only. Errors: `ERR_OUT_OF_RANGE` on `sort.column`.
#[tauri::command]
pub async fn dashboard_lent_open<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LentOpenInput,
) -> Result<LentOpenPage, UiError> {
    run(app, "dashboard_lent_open", move |ctx| {
        auth::require(ctx)?;
        dashboard::lent_open(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// One month of days with money paid on the spot, the tile آج کی نقدی page (plan 79 item 2c, PRD
/// FR-9.8). Read only. Errors: `ERR_FORMAT_INVALID` and `ERR_DATE_IN_FUTURE` on `month`.
#[tauri::command]
pub async fn dashboard_cash_sales_month<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CashSalesMonthInput,
) -> Result<CashSalesMonth, UiError> {
    run(app, "dashboard_cash_sales_month", move |ctx| {
        auth::require(ctx)?;
        dashboard::cash_sales_month(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// One day's buyers who paid on the spot (plan 79 item 2c). Read only. The date codes on `date`.
#[tauri::command]
pub async fn dashboard_cash_sales_day<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CashSalesDayInput,
) -> Result<CashSalesDay, UiError> {
    run(app, "dashboard_cash_sales_day", move |ctx| {
        auth::require(ctx)?;
        dashboard::cash_sales_day(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of the four parts of کل اگراہی with the grand total (plan 79 item 2b, FR-19.10).
#[tauri::command]
pub async fn dashboard_total_dues_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TotalDuesPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_total_dues_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::total_dues_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of the people who borrowed and still owe, every person of the filter (plan 79 item 3).
#[tauri::command]
pub async fn dashboard_lent_open_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LentOpenPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_lent_open_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::lent_open_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of one month of cash sales (plan 79 item 2c). Errors as `dashboard_cash_sales_month`.
#[tauri::command]
pub async fn dashboard_cash_sales_month_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CashSalesMonthPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_cash_sales_month_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::cash_sales_month_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of one day's cash sales, every buyer who paid (plan 79 item 2c). Errors as
/// `dashboard_cash_sales_day`.
#[tauri::command]
pub async fn dashboard_cash_sales_day_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CashSalesDayPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_cash_sales_day_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::cash_sales_day_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}
