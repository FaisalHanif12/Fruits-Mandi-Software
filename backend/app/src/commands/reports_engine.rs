//! The report engine commands (plan 20, step B2, PRD 19.1 to 19.5, 19.10, 19.11, NFR-SEC-13).
//! Every command needs the signed-in Owner. They only read (PRD FR-18.8, Code Standards B29).
//!
//! The work is in `mandi_reports::reports`; a command checks sign-in, then calls one function.
//! The filters arrive as the text the user typed and are read by the backend; every problem is
//! returned together with its field (Code Standards 4.3).

use mandi_reports::ReportsError;
use mandi_reports::reports::{
    BuyerBillsPrintData, CatalogEntry, ReportFilters, ReportId, ReportPage, ReportPrintData,
    ReportPrintInput, ReportRunInput, buyer_bills_print_data, catalog, print_data,
    print_data_for_page, run as run_report, statement_page, statement_paging, statement_request,
};
use mandi_services::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::statement::{PartnerStatement, PartnersStatementInput};
use mandi_services::{Context, ServiceError, auth, partners};

use super::run;
use crate::error::UiError;

/// A reports error as the error the screen sees. A problem with the filters keeps its field; an
/// unknown report is `ERR_NOT_FOUND` on `id`; the rest keep their cause for the log.
pub(crate) fn engine_error(error: ReportsError) -> ServiceError {
    match error {
        ReportsError::Rejected(items) => ServiceError::from(items),
        ReportsError::NotFound => ServiceError::reject_field(ErrorCode::ErrNotFound, "id"),
        ReportsError::Db(error) => ServiceError::Db(error),
        error @ (ReportsError::Money(_) | ReportsError::Words(_)) => {
            tracing::warn!(cause = %error, "a report total could not be made");
            ServiceError::reject(error.code())
        }
        ReportsError::File { full: true, detail } => {
            tracing::warn!(cause = %detail, "the export file could not be written: disk full");
            ServiceError::reject(ErrorCode::ErrDiskFull)
        }
        other => ServiceError::Data(other.to_string()),
    }
}

/// The most lines asked of the partner statement in one read when every row is wanted. Each read
/// looks up its lines with `IN (...)` lists, and SQLite allows 32,766 values in one statement.
const STATEMENT_CHUNK: u32 = 10_000;

/// How many times R31 is read again when a save lands between two chunks of one read.
const STATEMENT_READ_TRIES: u32 = 3;

/// A day `YYYY-MM-DD` as the `DD-MM-YYYY` text the services read.
fn typed_day(iso: Option<&str>) -> Option<String> {
    iso.and_then(BusinessDate::parse_iso).map(format_ddmmyyyy)
}

/// One read of the partner statement for R31, at an offset and a limit.
fn statement_read(
    ctx: &Context,
    request: &mandi_reports::reports::StatementRequest,
    offset: u32,
    limit: u32,
) -> Result<PartnerStatement, ServiceError> {
    partners::statement(
        ctx,
        &PartnersStatementInput {
            partner_id: request.partner_id,
            from: typed_day(request.from.as_deref()),
            to: typed_day(request.to.as_deref()),
            offset,
            limit,
        },
    )
}

/// One page of R31, the partner statement (PRD FR-17.9, FR-19.10). The lines are put together by
/// the services, the same statement as the partner screen shows; the report engine turns them
/// into a report page.
pub(crate) fn statement_run(
    ctx: &Context,
    filters: &ReportFilters,
    offset: i64,
    limit: i64,
) -> Result<ReportPage, ServiceError> {
    let (offset, limit) = statement_paging(offset, limit).map_err(engine_error)?;
    let request = statement_request(ctx.db(), filters).map_err(engine_error)?;
    let statement = statement_read(
        ctx,
        &request,
        u32::try_from(offset).unwrap_or(u32::MAX),
        u32::try_from(limit).unwrap_or(u32::MAX),
    )?;
    statement_page(&request, &statement, offset, limit).map_err(engine_error)
}

/// The chunks of one R31 read do not belong together: a save landed between two of them.
#[derive(Debug, PartialEq, Eq)]
struct TornRead;

/// Joins the chunks of one read of the statement into the whole statement. Every chunk must be
/// about the same range with the same opening, closing, totals and line count, the lines must be
/// all of them, and the running total of the last line must be the closing total. Otherwise a
/// save landed between two chunks (offsets moved, a line could print twice or be missed) and the
/// read must be done again (money agent review, round 7).
fn join_chunks(
    first: PartnerStatement,
    rest: Vec<PartnerStatement>,
) -> Result<PartnerStatement, TornRead> {
    let mut whole = first;
    for next in rest {
        let same = next.partner == whole.partner
            && next.opening_total_paisa == whole.opening_total_paisa
            && next.closing_total_paisa == whole.closing_total_paisa
            && next.total_count == whole.total_count
            && next.totals == whole.totals;
        if !same {
            return Err(TornRead);
        }
        whole.lines.extend(next.lines);
    }
    if i64::try_from(whole.lines.len()).ok() != Some(whole.total_count) {
        return Err(TornRead);
    }
    let last = whole
        .lines
        .last()
        .map_or(whole.opening_total_paisa, |line| line.running_paisa);
    if last != whole.closing_total_paisa {
        return Err(TornRead);
    }
    Ok(whole)
}

/// R31 with every line of the filter, for the print and the Excel export (PRD FR-19.11). The
/// statement is read in chunks; the chunks are joined and checked, and a read that was torn by a
/// save in between is done again.
pub(crate) fn statement_all(
    ctx: &Context,
    filters: &ReportFilters,
) -> Result<ReportPage, ServiceError> {
    let request = statement_request(ctx.db(), filters).map_err(engine_error)?;
    for _ in 0..STATEMENT_READ_TRIES {
        let first = statement_read(ctx, &request, 0, STATEMENT_CHUNK)?;
        let mut have = i64::try_from(first.lines.len()).unwrap_or(i64::MAX);
        let total = first.total_count;
        let mut rest = Vec::new();
        while have < total {
            let offset = u32::try_from(have).unwrap_or(u32::MAX);
            let next = statement_read(ctx, &request, offset, STATEMENT_CHUNK)?;
            if next.lines.is_empty() {
                // The statement got shorter while it was read: the join below refuses it.
                break;
            }
            have = have.saturating_add(i64::try_from(next.lines.len()).unwrap_or(i64::MAX));
            rest.push(next);
        }
        if let Ok(whole) = join_chunks(first, rest) {
            return statement_page(&request, &whole, 0, 0).map_err(engine_error);
        }
    }
    Err(ServiceError::Data(
        "the partner statement kept changing while it was read".to_owned(),
    ))
}

/// The catalogue of reports: for each its PRD ID, group, title ID and the filters it takes
/// (`date_range`, `as_of`, `party`, `seller`, `zamindar`, `item`, `city`, `adda`, `detail`), and
/// whether the party must be filled. Read only.
#[tauri::command]
pub async fn reports_catalog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<CatalogEntry>, UiError> {
    run(app, "reports_catalog", |ctx| {
        auth::require(ctx)?;
        Ok(catalog())
    })
    .await
}

/// One page of a report: typed columns and rows, the number of rows of the whole filter, the
/// totals over the whole filter (never a sum of the page), the filters as read, and for each row
/// the record it opens. `limit` 0 means 100; at most 1,000. Read only. Errors: `ERR_NOT_FOUND`
/// on `id`; `ERR_REQUIRED` on `filters.party` (R13, R14); `ERR_OUT_OF_RANGE` on a filter the
/// report does not take, on `offset` and on `limit`; `ERR_DATE_INVALID` on `filters.from`,
/// `filters.to` or `filters.as_of`; `ERR_DATE_RANGE` on `filters.to`; `ERR_NOT_FOUND` on a
/// picked record (`filters.party.account_id`, `filters.item_id`, ...).
#[tauri::command]
pub async fn reports_run<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ReportRunInput,
) -> Result<ReportPage, UiError> {
    run(app, "reports_run", move |ctx| {
        auth::require(ctx)?;
        if input.id == ReportId::R31 {
            return statement_run(ctx, &input.filters, input.offset, input.limit);
        }
        run_report(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print data of a report with every row: the firm header and the filters in the print
/// language, the columns, the rows and the totals row formatted for print, the closing balance
/// in words for R13 and R14, the paper class, `dir` and the message ID of the page footer. Read
/// only. Errors: as `reports_run`.
#[tauri::command]
pub async fn reports_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ReportPrintInput,
) -> Result<ReportPrintData, UiError> {
    run(app, "reports_print_data", move |ctx| {
        auth::require(ctx)?;
        if input.id == ReportId::R31 {
            let page = statement_all(ctx, &input.filters)?;
            return print_data_for_page(ctx.db(), &page, &input).map_err(engine_error);
        }
        print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print data of R5 "Print All" (PRD 19.2 R5, FR-19.10, FR-19.11): one slip for each credit
/// buyer of the filter, with every line of his, his sums, and the sums of all slips, in the print
/// language. A one-time person has his own slip; a buyer who paid cash on the spot has none. Read
/// only. The input is that of `reports_print_data` with `id` R5. Errors: as `reports_run`;
/// `ERR_OUT_OF_RANGE` on `id` if the report is not R5.
#[tauri::command]
pub async fn reports_buyer_bills_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ReportPrintInput,
) -> Result<BuyerBillsPrintData, UiError> {
    run(app, "reports_buyer_bills_print_data", move |ctx| {
        auth::require(ctx)?;
        buyer_bills_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandi_services::core::accounts::RecordRef;
    use mandi_services::core::money::Paisa;
    use mandi_services::core::statement::{
        StatementDetail, StatementHeading, StatementLine, StatementTotals,
    };
    use std::collections::BTreeMap;

    fn rupees(n: i64) -> Paisa {
        Paisa::from_rupees(n).unwrap()
    }

    fn line(amount: i64, running: i64) -> StatementLine {
        StatementLine {
            entry_date: "2026-10-10".to_owned(),
            heading: StatementHeading::Bikri,
            detail: StatementDetail::IncomeShare,
            entry_id: Some(1),
            number: Some(1),
            secondary_number: None,
            subject: None,
            reason: String::new(),
            amount_paisa: Some(rupees(amount)),
            running_paisa: rupees(running),
            values: BTreeMap::new(),
        }
    }

    /// A statement of three lines (+100, +200, -50) from an opening of 10, cut as asked.
    fn chunk(from: usize, to: usize, closing: i64) -> PartnerStatement {
        let all = [line(100, 110), line(200, 310), line(-50, 260)];
        PartnerStatement {
            partner: RecordRef {
                id: 1,
                code: String::new(),
                name_ur: "الف".to_owned(),
                name_en: "Partner A".to_owned(),
            },
            opening_total_paisa: rupees(10),
            closing_total_paisa: rupees(closing),
            lines: all[from..to].to_vec(),
            total_count: 3,
            totals: StatementTotals {
                plus_paisa: rupees(300),
                minus_paisa: rupees(50),
            },
        }
    }

    #[test]
    fn chunks_of_one_read_join_in_order_into_the_whole_statement() {
        let whole = join_chunks(chunk(0, 2, 260), vec![chunk(2, 3, 260)]).unwrap();
        assert_eq!(whole, chunk(0, 3, 260));
        // One chunk that holds everything joins to itself.
        assert_eq!(join_chunks(chunk(0, 3, 260), Vec::new()).unwrap(), whole);
    }

    #[test]
    fn a_save_between_two_chunks_is_caught() {
        // The second chunk saw a different closing total: a posting landed in between.
        assert_eq!(
            join_chunks(chunk(0, 2, 260), vec![chunk(2, 3, 300)]),
            Err(TornRead)
        );
        // A chunk with another line count.
        let mut longer = chunk(2, 3, 260);
        longer.total_count = 4;
        assert_eq!(join_chunks(chunk(0, 2, 260), vec![longer]), Err(TornRead));
    }

    #[test]
    fn missing_lines_and_a_wrong_last_running_total_are_caught() {
        // A line is missing (the statement got shorter while it was read).
        assert_eq!(join_chunks(chunk(0, 2, 260), Vec::new()), Err(TornRead));
        // The last running total is not the closing total.
        assert_eq!(join_chunks(chunk(0, 3, 999), Vec::new()), Err(TornRead));
    }
}
