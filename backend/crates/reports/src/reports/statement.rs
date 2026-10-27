//! R31 شراکت دار کھاتہ (the partner statement) (PRD 19.2, 17.9, FR-17.10, FR-19.10, FR-19.11).
//!
//! The lines of a partner's statement come from `mandi_services::partners::statement`: the
//! partner's events are put together there from the share rows, the journal and the vouchers,
//! and the running کل رقم (C22) is worked out there. That working is more than 700 lines and
//! belongs to the services, and `mandi-reports` may not depend on them. So the report engine
//! does not read the lines itself. It does two things:
//!
//! 1. [`statement_request`] reads and checks the filters (the dates, the partner) the way every
//!    report does, and says which partner and which days the services must read;
//! 2. [`statement_page`] turns the statement the services gave back into the report page, with
//!    the same columns, totals and summary lines as every other report. A line's words are a
//!    Fluent message and its arguments ([`Cell::Message`]), so the screen, the print and the
//!    Excel sheet write the same sentence in the language they are in.
//!
//! The app joins the two (`reports_run`, `reports_print_data`, `reports_export_excel`), so the
//! statement on the reports screen can never differ from the statement on the partner screen: it
//! is the same statement. Nothing is worked out here. The totals row is the net of the amounts
//! (added less taken away) and the closing کل رقم, both from the services.
//!
//! Example (E2E-37, ب from 10-10-26 to 11-10-26): opening 0; "10-10-26 بکری 1: …" +1,184
//! (running 1,184); "… بکری 2 …" +462 (1,646); "دکان خرچہ بجلی" −900 (746); "11-10-26 سرمایہ
//! جمع" +50,000 (50,746); closing 50,746.

use std::collections::BTreeMap;

use mandi_core::error::ErrorValue;
use mandi_core::money::Paisa;
use mandi_core::statement::{PartnerStatement, StatementDetail, StatementHeading, StatementLine};
use mandi_db::Db;

use super::common::{col, paisa};
use super::filters::{Takes, resolve};
use super::model::{
    ArgValue, Cell, ColumnKind, MessageArg, Names, ReportFilters, ReportId, ReportPage,
    ResolvedFilters, Row, RowKind, RowSource, SourceKind, SummaryLine,
};
use super::{common, ids, page_spec, spec_of};
use crate::error::ReportsError;

/// Who and which days the statement is for, with the filters as read (PRD FR-19.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementRequest {
    /// The partner.
    pub partner_id: i64,
    /// First day, included, `YYYY-MM-DD`. `None` starts at his first line.
    pub from: Option<String>,
    /// Last day, included, `YYYY-MM-DD`. `None` runs to the last line.
    pub to: Option<String>,
    /// The filters as read, echoed back on the page.
    pub resolved: ResolvedFilters,
}

/// Reads and checks the filters of R31: the days (or a shortcut) and the partner, who must be
/// given. Every problem comes back with its field (Code Standards 4.3).
///
/// # Errors
/// `ReportsError::Rejected` with `ERR_REQUIRED` on `filters.partner_id`, `ERR_NOT_FOUND` on an
/// unknown partner, the date codes on `filters.from` and `filters.to`, `ERR_OUT_OF_RANGE` on a
/// filter R31 does not take; a database error.
pub fn statement_request(
    db: &Db,
    filters: &ReportFilters,
) -> Result<StatementRequest, ReportsError> {
    let spec = spec_of(ReportId::R31)?;
    db.read(|conn| {
        let today = common::today(db.clock().now_utc())?;
        let typed = resolve(
            conn,
            today,
            Takes {
                kinds: spec.filters,
                party_required: spec.party_required,
            },
            filters,
        )?;
        let partner_id = typed
            .partner_id
            .ok_or_else(|| ReportsError::Data("the statement has no partner".to_owned()))?;
        Ok(StatementRequest {
            partner_id,
            from: typed.from.clone(),
            to: typed.to.clone(),
            resolved: typed.resolved,
        })
    })
}

/// The page size of a screen page: 100 when the screen sends 0, at most 1,000.
///
/// # Errors
/// `ReportsError::Rejected` with `ERR_OUT_OF_RANGE` on `offset` or `limit`.
pub fn statement_paging(offset: i64, limit: i64) -> Result<(i64, i64), ReportsError> {
    let spec = page_spec(offset, limit)?;
    Ok((spec.offset, spec.limit.unwrap_or(super::DEFAULT_LIMIT)))
}

/// The message ID of the group a line belongs to.
fn heading_id(heading: StatementHeading) -> &'static str {
    match heading {
        StatementHeading::Bikri => "partners-statement-heading-bikri",
        StatementHeading::Expense => "partners-statement-heading-expense",
        StatementHeading::Capital => "partners-statement-heading-capital",
        StatementHeading::Drawings => "partners-statement-heading-drawings",
        StatementHeading::Trip => "partners-statement-heading-trip",
        StatementHeading::Garden => "partners-statement-heading-garden",
        StatementHeading::Fruit => "partners-statement-heading-fruit",
    }
}

/// The message ID of the sentence of a line.
fn detail_id(detail: StatementDetail) -> &'static str {
    match detail {
        StatementDetail::IncomeShare => "partners-statement-detail-income-share",
        StatementDetail::ExpenseShare => "partners-statement-detail-expense-share",
        StatementDetail::CapitalOpening => "partners-statement-detail-capital-opening",
        StatementDetail::CapitalIn => "partners-statement-detail-capital-in",
        StatementDetail::CapitalOut => "partners-statement-detail-capital-out",
        StatementDetail::MonthClose => "partners-statement-detail-month-close",
        StatementDetail::Taken => "partners-statement-detail-taken",
        StatementDetail::Returned => "partners-statement-detail-returned",
        StatementDetail::DrawingsOpening => "partners-statement-detail-drawings-opening",
        StatementDetail::TripPaid => "partners-statement-detail-trip-paid",
        StatementDetail::TripCredit => "partners-statement-detail-trip-credit",
        StatementDetail::TripWriteOff => "partners-statement-detail-trip-write-off",
        StatementDetail::TripClosed => "partners-statement-detail-trip-closed",
        StatementDetail::GardenPaid => "partners-statement-detail-garden-paid",
        StatementDetail::GardenClosed => "partners-statement-detail-garden-closed",
        StatementDetail::FruitTaken => "partners-statement-detail-fruit-taken",
    }
}

fn text_arg(name: &str, text: impl Into<String>) -> MessageArg {
    MessageArg {
        name: name.to_owned(),
        value: ArgValue::Text { text: text.into() },
    }
}

/// A document number as plain text (no grouping), like the partner screen writes it.
fn number_text(number: Option<i64>) -> String {
    number.map_or_else(String::new, |n| n.to_string())
}

/// A word made of a message with the number in it, for example `E-5` from
/// `vouchers-number-shop-expense`.
fn series_arg(name: &str, id: &str, number: Option<i64>) -> MessageArg {
    MessageArg {
        name: name.to_owned(),
        value: ArgValue::Message {
            id: id.to_owned(),
            args: vec![text_arg("number", number_text(number))],
        },
    }
}

/// The number series a `values.kind` word names, whatever the detail is (PRD FR-17.10: a salary
/// deduction is an LR- voucher, a جنرل a G- voucher).
fn series_of_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "salary" => Some("money-lent-return"),
        "general" => Some("general"),
        "shop_expense" => Some("shop-expense"),
        "capital_in" => Some("capital-in"),
        "capital_out" => Some("capital-out"),
        "personal_expense" => Some("personal-expense"),
        "personal_return" => Some("personal-return"),
        _ => None,
    }
}

/// The usual number series of a detail.
fn series_of_detail(detail: StatementDetail) -> Option<&'static str> {
    match detail {
        StatementDetail::ExpenseShare => Some("shop-expense"),
        StatementDetail::CapitalIn => Some("capital-in"),
        StatementDetail::CapitalOut => Some("capital-out"),
        StatementDetail::Taken => Some("personal-expense"),
        StatementDetail::Returned => Some("personal-return"),
        _ => None,
    }
}

/// The values the services sent as arguments: a whole number is a grouped number, a name ending
/// in `_paisa` is money under its own name and under the name without the ending, and a name
/// ending in `_bps` is also the percent a person reads under the name without the ending. Text
/// passes as it is.
fn value_args(values: &BTreeMap<String, ErrorValue>) -> Result<Vec<MessageArg>, ReportsError> {
    let mut args = Vec::with_capacity(values.len());
    for (key, value) in values {
        match value {
            ErrorValue::Text(text) => args.push(text_arg(key, text.clone())),
            ErrorValue::Number(n) => {
                if let Some(stem) = key.strip_suffix("_paisa") {
                    let money = paisa(*n)?;
                    for name in [key.as_str(), stem] {
                        args.push(MessageArg {
                            name: name.to_owned(),
                            value: ArgValue::Money { paisa: money },
                        });
                    }
                    continue;
                }
                args.push(MessageArg {
                    name: key.clone(),
                    value: ArgValue::Number { value: *n },
                });
                if let Some(stem) = key.strip_suffix("_bps") {
                    args.push(MessageArg {
                        name: stem.to_owned(),
                        value: ArgValue::Percent { hundredths: *n },
                    });
                }
            }
        }
    }
    Ok(args)
}

/// The text of a value the line carries under `kind`, if it is text.
fn kind_of(line: &StatementLine) -> Option<&str> {
    match line.values.get("kind") {
        Some(ErrorValue::Text(kind)) => Some(kind.as_str()),
        _ => None,
    }
}

/// The sentence of one line as a message and its arguments.
fn detail_cell(line: &StatementLine) -> Result<Cell, ReportsError> {
    // A fruit line of an entry that was edited since: the services give the details (item, packing,
    // values) only for the posting in force, so an older posting and its reversal read as that,
    // never as a sentence with holes (PRD FR-17.10). A cancelled entry keeps its details.
    if line.detail == StatementDetail::FruitTaken && !line.values.contains_key("qty") {
        return Ok(Cell::Message {
            id: "partners-statement-detail-fruit-earlier".to_owned(),
            args: vec![series_arg("fruit", "fruit-number", line.number)],
        });
    }
    let id = detail_id(line.detail);
    let mut args = value_args(&line.values)?;
    args.push(text_arg("number", number_text(line.number)));
    args.push(text_arg("reason", line.reason.clone()));
    args.push(match &line.subject {
        Some(subject) => MessageArg {
            name: "subject".to_owned(),
            value: ArgValue::Names {
                names: Names {
                    ur: subject.name_ur.clone(),
                    en: subject.name_en.clone(),
                },
            },
        },
        None => text_arg("subject", ""),
    });
    // The voucher the line came from, in its own number series: the kind the services name
    // first, else the usual series of the detail.
    let series = if line.heading == StatementHeading::Trip {
        series_of_detail(line.detail)
    } else {
        kind_of(line)
            .and_then(series_of_kind)
            .or_else(|| series_of_detail(line.detail))
    };
    // The packing of a fruit line: "2 پیٹی مالٹا @ 1,400". Empty on every other line.
    args.push(match &line.packing {
        Some(packing) => MessageArg {
            name: "packing".to_owned(),
            value: ArgValue::Names {
                names: Names {
                    ur: packing.name_ur.clone(),
                    en: packing.name_en.clone(),
                },
            },
        },
        None => text_arg("packing", ""),
    });
    // A garden's number in its own series (PRD FR-12.11).
    if line.heading == StatementHeading::Garden {
        // The garden's code: 0080 for the first garden (plan 34).
        args.push(text_arg(
            "garden",
            line.venture_code.clone().unwrap_or_default(),
        ));
    }
    args.push(match (series, line.number) {
        (Some(series), Some(_)) => {
            series_arg("voucher", &format!("vouchers-number-{series}"), line.number)
        }
        _ => text_arg("voucher", number_text(line.number)),
    });
    if line.heading == StatementHeading::Trip {
        // The trip's code: 0070 for the first trip (plan 34).
        args.push(text_arg(
            "trip",
            line.venture_code.clone().unwrap_or_default(),
        ));
        args.push(match (kind_of(line), line.secondary_number) {
            (Some("purchase"), Some(_)) => {
                series_arg("secondary", "purchases-number", line.secondary_number)
            }
            (Some("cost"), Some(_)) => {
                series_arg("secondary", "trips-cost-number", line.secondary_number)
            }
            _ => text_arg("secondary", number_text(line.secondary_number)),
        });
        args.push(text_arg("kind", kind_of(line).unwrap_or("other")));
        // A sale in another shop has no بکری number: the sentence says so (PRD FR-12.32).
        let source = if line.secondary_number.is_some_and(|n| n > 0) {
            "bikri"
        } else {
            "other"
        };
        args.push(text_arg("source", source));
    }
    Ok(Cell::Message {
        id: id.to_owned(),
        args,
    })
}

/// The row of one line.
fn row_of(line: &StatementLine) -> Result<Row, ReportsError> {
    Ok(Row {
        cells: vec![
            Cell::Date {
                iso: line.entry_date.clone(),
            },
            Cell::Label {
                id: heading_id(line.heading).to_owned(),
            },
            detail_cell(line)?,
            line.amount_paisa
                .map_or(Cell::Empty, |paisa| Cell::Money { paisa }),
            Cell::Money {
                paisa: line.running_paisa,
            },
        ],
        source: line.entry_id.map(|id| RowSource {
            kind: SourceKind::Entry,
            id,
        }),
        kind: RowKind::Normal,
    })
}

/// The report page of a statement. `statement.lines` is the page to show (the services slice it);
/// `offset` and `limit` are echoed back. `limit` 0 means a page that holds every row (a print).
///
/// # Errors
/// `ReportsError::Money` if a stored amount is not whole rupees or a total does not fit.
pub fn statement_page(
    request: &StatementRequest,
    statement: &PartnerStatement,
    offset: i64,
    limit: i64,
) -> Result<ReportPage, ReportsError> {
    let mut filters = request.resolved.clone();
    filters.partner = Some(Names {
        ur: statement.partner.name_ur.clone(),
        en: statement.partner.name_en.clone(),
    });
    let rows = statement
        .lines
        .iter()
        .map(row_of)
        .collect::<Result<Vec<_>, _>>()?;
    // The amounts of the range added up: added less taken away. The services worked out both.
    let net = statement
        .totals
        .plus_paisa
        .checked_sub(statement.totals.minus_paisa)?;
    // The opening plus the net must be the closing. A mismatch is a bug in the working.
    if statement.opening_total_paisa.checked_add(net)? != statement.closing_total_paisa {
        return Err(ReportsError::Data(
            "the statement does not add up to its closing total".to_owned(),
        ));
    }
    let money = |paisa: Paisa| Cell::Money { paisa };
    Ok(ReportPage {
        id: ReportId::R31,
        title_id: ids::title_id("R31"),
        filters,
        columns: vec![
            col("date", ids::COL_DATE, ColumnKind::Date),
            col(
                "heading",
                "partners-statement-col-heading",
                ColumnKind::Label,
            ),
            col(
                "detail",
                "partners-statement-col-detail",
                ColumnKind::Message,
            ),
            col("amount", ids::COL_AMOUNT, ColumnKind::Money),
            col(
                "running",
                "partners-statement-col-running",
                ColumnKind::Money,
            ),
        ],
        rows,
        offset,
        limit,
        total_count: statement.total_count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            money(net),
            money(statement.closing_total_paisa),
        ],
        summary: vec![
            SummaryLine {
                label_id: "partners-statement-opening".to_owned(),
                cell: money(statement.opening_total_paisa),
            },
            SummaryLine {
                label_id: "partners-statement-plus".to_owned(),
                cell: money(statement.totals.plus_paisa),
            },
            SummaryLine {
                label_id: "partners-statement-minus".to_owned(),
                cell: money(statement.totals.minus_paisa),
            },
            SummaryLine {
                label_id: "partners-statement-closing".to_owned(),
                cell: money(statement.closing_total_paisa),
            },
        ],
    })
}
