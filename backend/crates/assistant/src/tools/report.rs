//! The tool `report`: any report R1 to R38 with its own filters (PRD FR-7.87, Code Standards Q3).
//!
//! It calls `mandi_reports::reports::run`, the function of the report screen, so the rows and the
//! totals are the screen's own. The filters come from the report's catalogue entry: a filter the
//! report does not have is an error, never ignored. Rows are 20 unless `rows` is set (at most 100);
//! ledgers (R11, R12, R13) give their newest rows first. `where` and a second sort key read every
//! row of the filter (up to [`MAX_FETCH_ROWS`]); a single sort key is sorted by the report itself.
//! `matched` adds the count and the totals of the rows that pass `where`, worked out by
//! `services::figures` (Code Standards Q5).
//!
//! Example (worked day): `report{id:"R13", party:"r1.matches.0", from:"2026-08-31", to:"2026-10-10"}`
//! gives the rows of اکرم's ledger newest first, the totals and the summary `opening`, `closing`.

use mandi_reports::ReportsError;
use mandi_reports::reports::{
    Cell, Column, FilterKind, GroupBy, PartyFilter, ReportFilters, ReportId, ReportPage,
    ReportRunInput, Row, RowKind, catalog, run as engine_run, run_first_rows, statement_page,
    statement_request,
};
use mandi_services::core::accounts::{AccountType, BalanceSide};
use mandi_services::core::capital::{CapitalHistoryInput, CapitalRowKind};
use mandi_services::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::money::Paisa;
use mandi_services::core::paging::{SortDirection, SortInput};
use mandi_services::core::statement::PartnersStatementInput;
use mandi_services::domain::figures::{CalcOp, Dir, NumberUnit, Value};
use mandi_services::{bikri, figures, partners};
use serde::Deserialize;
use serde_json::Value as Json;

use super::calc::question_number;
use super::cells::{
    Order, cell_node, compare_cells, is_left_out, mixed_sides, order_of, row_key, shown_key,
    signed_paisa,
};
use super::cues;
use super::records::{named, require_type};
use super::report_extra;
use super::teach::{column_words, report_fields};
use super::{
    Draft, Env, PartnerRule, State, Stop, ToolError, date_arg, input, lookup, report_rule,
    service_stop,
};
use crate::answer::{AnswerRow, AnswerSource, AnswerTable};
use crate::results::{Money, NameVal, Node, Shown, Side, Val};
use crate::source::{AnswerRecord, SourceBikri, SourceParty, SourceReportFilters};

/// Rows of a result when `rows` is not set.
pub const DEFAULT_ROWS: i64 = 20;
/// Most rows of a result.
pub const MAX_ROWS: i64 = 100;
/// Most rows read to filter or sort in memory (NFR-SCL-1: 20,000 accounts; a list longer than
/// this is refused rather than cut, so `matched` is always over every row).
pub const MAX_FETCH_ROWS: i64 = 10_000;
/// A comparison of `where`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    /// More than.
    Gt,
    /// At least.
    Gte,
    /// Less than.
    Lt,
    /// At most.
    Lte,
    /// Equal.
    Eq,
}

/// `where`: keep the rows whose column compares with the value.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Where {
    /// The column key.
    pub column: String,
    /// The comparison.
    pub op: CompareOp,
    /// A number of the question (`q.n1`) or a figure already read (`r3.totals.balance`).
    pub value: String,
}

/// The direction of a sort key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderWord {
    /// Smallest first.
    Asc,
    /// Biggest first.
    Desc,
}

/// One sort key.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SortKey {
    /// The column key.
    pub column: String,
    /// The direction.
    pub order: OrderWord,
}

/// The input of `report` (D6-system-prompt Section 3).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// `R1` to `R38`.
    pub id: String,
    /// A period result (`r2.from`) or `YYYY-MM-DD`.
    pub from: Option<String>,
    /// The last day.
    pub to: Option<String>,
    /// The day of a balance list.
    pub as_of: Option<String>,
    /// A find match or a key.
    pub party: Option<String>,
    /// An item.
    pub item: Option<String>,
    /// `fruit` or `vegetable`: all the items of the group, never with `item` (R2, R3, R4, R7, R10,
    /// R27).
    #[serde(default)]
    pub item_group: Option<String>,
    /// `first`, `second` or `third`: only the sale lines of that quality (R2, R3, R4).
    #[serde(default)]
    pub grade: Option<String>,
    /// A city.
    pub city: Option<String>,
    /// An اڈا.
    pub adda: Option<String>,
    /// A collector, a staff member (R22, R23, R35).
    pub collector: Option<String>,
    /// A partner.
    pub partner: Option<String>,
    /// A trip (no report has this filter yet: it is an error).
    pub trip: Option<String>,
    /// A garden.
    pub garden: Option<String>,
    /// A بکری number, for R1.
    pub bikri: Option<i64>,
    /// A seller (R2 to R5, R6, R7, R10, R26, R27): plan 55 P1 to P4.
    #[serde(default)]
    pub seller: Option<String>,
    /// A زمیندار (R2 to R5, R6, R27).
    #[serde(default)]
    pub zamindar: Option<String>,
    /// A lot of the seller and the item named, as its key (`l7`). Only the app sets it, for the lot
    /// the Owner picked (plan 71 L9, PRD FR-12.45, FR-7.77); the schema the model reads has no such
    /// field. It narrows the report to that lot, and is left out when the report is not of that
    /// seller and item.
    #[serde(default)]
    pub lot: Option<String>,
    /// `paid` or `due` (R38).
    #[serde(default)]
    pub status: Option<String>,
    /// One kind of voucher, in its stored word (R15).
    #[serde(default)]
    pub voucher_type: Option<String>,
    /// How the report lays out its rows: `detail` (R14), `item` `day` `month` `partner` (R33),
    /// `partner` (R36), `item` (R10, one row for each item).
    #[serde(default)]
    pub layout: Option<String>,
    /// `day` or `month`, for R28.
    pub group_by: Option<String>,
    /// Keep the rows that pass.
    #[serde(rename = "where")]
    pub where_: Option<Where>,
    /// Up to 2 sort keys.
    pub sort: Option<Vec<SortKey>>,
    /// How many rows.
    pub rows: Option<i64>,
}

/// A report problem as a stop for the model.
pub(crate) fn reports_stop(error: ReportsError) -> Stop {
    match error {
        ReportsError::Rejected(items) => match items.into_iter().next() {
            Some(item) if item.code == ErrorCode::ErrPartnerLocked => Stop::Locked,
            Some(item) => Stop::Error(ToolError::from_code(item.code, Some(&item.field))),
            None => Stop::Error(ToolError::from_code(ErrorCode::ErrInternal, None)),
        },
        ReportsError::NotFound => Stop::Error(ToolError::from_code(ErrorCode::ErrNotFound, None)),
        _ => Stop::Error(ToolError::from_code(ErrorCode::ErrInternal, None)),
    }
}

fn bad(code: &str, field: &str, detail: &str) -> ToolError {
    ToolError::new(code, field, detail)
}

/// The filters of a run: the engine's own, and the same with whole records for the buttons.
struct Built {
    filters: ReportFilters,
    source: SourceReportFilters,
    party: Option<AnswerRecord>,
    /// What the tool did that the model did not ask for, given back as `note` (plan 52 item 2).
    notes: Vec<String>,
    /// True when the model asked for a sort: the heading and total lines are then given apart
    /// (`headings`), never among the ranked rows (Q194 of the live run of 03-11-2026).
    ranked: bool,
}

fn takes_or(takes: &[FilterKind], kind: FilterKind, field: &str) -> Result<(), ToolError> {
    if takes.contains(&kind) {
        Ok(())
    } else {
        Err(not_taken(takes, field))
    }
}

/// A filter the report does not have: what it does take is listed (a balance list takes `as_of`
/// where the others take `from` and `to`).
pub(super) fn not_taken(takes: &[FilterKind], field: &str) -> ToolError {
    bad(
        "ERR_OUT_OF_RANGE",
        field,
        "this report does not have that filter; the filters it takes are listed",
    )
    .allowed(report_fields(takes))
}

fn build_filters(
    env: &Env<'_>,
    state: &State,
    input: &Input,
    takes: &[FilterKind],
) -> Result<Built, ToolError> {
    let mut filters = ReportFilters::default();
    let mut source = SourceReportFilters::default();
    let mut party_record = None;
    let date = |field: &str, text: &Option<String>| -> Result<Option<String>, ToolError> {
        // A period field with no date (the `from` of `up_to_now`) leaves the date open.
        Ok(text
            .as_deref()
            .map(|t| date_arg(env, state, t, field))
            .transpose()?
            .flatten())
    };
    if let Some(from) = date("from", &input.from)? {
        takes_or(takes, FilterKind::DateRange, "from")?;
        filters.from = from.clone();
        source.from = from;
    }
    if let Some(to) = date("to", &input.to)? {
        takes_or(takes, FilterKind::DateRange, "to")?;
        filters.to = to.clone();
        source.to = to;
    }
    if let Some(as_of) = date("as_of", &input.as_of)? {
        takes_or(takes, FilterKind::AsOf, "as_of")?;
        filters.as_of = as_of.clone();
        source.as_of = as_of;
    }
    if let Some(text) = &input.party {
        let (account_id, record) = named(env, state, text, "party", &["a"])?;
        if takes.contains(&FilterKind::Party) {
            filters.party = Some(PartyFilter {
                account_id,
                person_key: None,
            });
            source.party = Some(SourceParty {
                record: record.clone(),
                person_key: None,
            });
        } else if takes.contains(&FilterKind::Seller) {
            require_type(env, account_id, "party", AccountType::Seller)?;
            filters.seller_id = Some(account_id);
            source.seller = Some(record.clone());
        } else {
            return Err(not_taken(takes, "party"));
        }
        party_record = Some(record);
    }
    if let Some(text) = &input.item {
        takes_or(takes, FilterKind::Item, "item")?;
        let (item_id, record) = named(env, state, text, "item", &["i"])?;
        filters.item_id = Some(item_id);
        source.item = Some(record);
    }
    if let Some(text) = &input.city {
        takes_or(takes, FilterKind::City, "city")?;
        let (city_id, record) = named(env, state, text, "city", &["ct"])?;
        filters.city_id = Some(city_id);
        source.city = Some(record);
    }
    if let Some(text) = &input.adda {
        takes_or(takes, FilterKind::Adda, "adda")?;
        let (adda_id, record) = named(env, state, text, "adda", &["ad"])?;
        filters.adda_id = Some(adda_id);
        source.adda = Some(record);
    }
    if let Some(text) = &input.collector {
        // R22, R23 and R35 take a collector (a staff member, found with role `collector` or
        // `staff`); no other report does.
        takes_or(takes, FilterKind::Collector, "collector")?;
        let (collector_id, record) = named(env, state, text, "collector", &["st"])?;
        filters.collector_id = Some(collector_id);
        source.collector = Some(record);
    }
    if input.trip.is_some() {
        return Err(bad(
            "ERR_OUT_OF_RANGE",
            "trip",
            "no report has a trip filter; use analytics view J",
        ));
    }
    if let Some(text) = &input.partner {
        if report_extra::is_guests(text) {
            // The shop's guests are not a partner: R37 reads only their fruit (plan 55 P1).
            report_extra::guests(takes, &mut filters, &mut source)?;
        } else {
            takes_or(takes, FilterKind::Partner, "partner")?;
            let (partner_id, record) = named(env, state, text, "partner", &["s"])?;
            filters.partner_id = Some(partner_id);
            source.partner = Some(record);
        }
    }
    if let Some(text) = &input.garden {
        takes_or(takes, FilterKind::Garden, "garden")?;
        let (garden_id, record) = named(env, state, text, "garden", &["g"])?;
        filters.garden_id = Some(garden_id);
        source.garden = Some(record);
    }
    if let Some(number) = input.bikri {
        takes_or(takes, FilterKind::Bikri, "bikri")?;
        let rows = bikri::find(
            env.ctx,
            &mandi_services::core::bikri::BikriFindInput {
                number: Some(number),
                ..Default::default()
            },
        )
        .map_err(|_| bad("ERR_INTERNAL", "bikri", "the search failed"))?;
        let found = rows
            .into_iter()
            .find(|r| r.number == number)
            .ok_or_else(|| bad("ERR_NOT_FOUND", "bikri", "no such بکری"))?;
        filters.bikri_id = Some(found.id);
        source.bikri = Some(SourceBikri {
            id: found.id,
            number,
        });
    }
    report_extra::apply(
        env,
        state,
        &report_extra::Extra {
            seller: input.seller.as_deref(),
            zamindar: input.zamindar.as_deref(),
            status: input.status.as_deref(),
            voucher_type: input.voucher_type.as_deref(),
            layout: input.layout.as_deref(),
            item_group: input.item_group.as_deref(),
            grade: input.grade.as_deref(),
            lot: input.lot.as_deref(),
        },
        takes,
        &mut filters,
        &mut source,
    )?;
    if let Some(word) = &input.group_by {
        takes_or(takes, FilterKind::GroupBy, "group_by")?;
        let group = match word.as_str() {
            "day" => GroupBy::Day,
            "month" => GroupBy::Month,
            _ => return Err(bad("ERR_OUT_OF_RANGE", "group_by", "day or month")),
        };
        filters.group_by = Some(group);
        source.group_by = Some(group);
    }
    // The lot the Owner picked narrowed this report (plan 71 L9): the model is told, so it says
    // the figures are of that lot.
    let notes = source
        .lot
        .iter()
        .map(|lot| format!("only the lot the Owner chose is read: {}", lot.name_en))
        .collect();
    Ok(Built {
        filters,
        source,
        party: party_record,
        notes,
        ranked: false,
    })
}

/// True for the reports whose newest-first order is the exact reverse of their order (see
/// `mandi_reports::reports::newest_first_reverses`).
fn id_is_newest_first_register(id: ReportId) -> bool {
    mandi_reports::reports::newest_first_reverses(id)
}

/// The sort the report itself does: one key, or the newest first for a ledger.
fn report_sort(id: ReportId, keys: &[SortKey], env: &Env<'_>) -> Option<SortInput> {
    let language = Some(env.language);
    match keys.first() {
        Some(key) if keys.len() == 1 => Some(SortInput {
            column: engine_name(&key.column),
            direction: match key.order {
                OrderWord::Asc => SortDirection::Asc,
                OrderWord::Desc => SortDirection::Desc,
            },
            language,
        }),
        // R31: a partner's statement newest first too, so the first row's `running` is his total
        // now and the 20 rows shown are the latest, not the oldest (money review). The statement
        // names its day column `entry_date`.
        None if id == ReportId::R31 => Some(SortInput {
            column: "entry_date".to_owned(),
            direction: SortDirection::Desc,
            language,
        }),
        None if matches!(id, ReportId::R11 | ReportId::R12 | ReportId::R13) => Some(SortInput {
            column: "date".to_owned(),
            direction: SortDirection::Desc,
            language,
        }),
        _ => None,
    }
}

/// The engine's key for a key the model uses where it is known without the columns.
fn engine_name(shown: &str) -> String {
    match shown {
        "entry" => "doc".to_owned(),
        other => other.to_owned(),
    }
}

fn read_page(
    env: &Env<'_>,
    id: ReportId,
    filters: &ReportFilters,
    offset: i64,
    limit: i64,
    sort: Option<SortInput>,
) -> Result<ReportPage, Stop> {
    if id == ReportId::R31 {
        return read_statement(env, filters, offset, limit, sort);
    }
    engine_run(
        env.ctx.db(),
        &ReportRunInput {
            id,
            filters: filters.clone(),
            offset,
            limit,
            sort,
            group_by_month: false,
        },
    )
    .map_err(reports_stop)
}

/// R31 of one partner is built from his statement (the report engine never builds it).
fn read_statement(
    env: &Env<'_>,
    filters: &ReportFilters,
    offset: i64,
    limit: i64,
    sort: Option<SortInput>,
) -> Result<ReportPage, Stop> {
    let request = statement_request(env.ctx.db(), filters).map_err(reports_stop)?;
    // The request holds ISO dates; the statement service takes `DD-MM-YYYY` (Q202 of the live run
    // of 03-11-2026: every R31 with dates was refused with ERR_DATE_INVALID).
    let day = |iso: &Option<String>| {
        iso.as_deref()
            .and_then(BusinessDate::parse_iso)
            .map(format_ddmmyyyy)
    };
    let statement = partners::statement(
        env.ctx,
        &PartnersStatementInput {
            partner_id: request.partner_id,
            from: day(&request.from),
            to: day(&request.to),
            offset: u32::try_from(offset).unwrap_or(0),
            limit: u32::try_from(limit).unwrap_or(0),
            sort,
            group_by_month: false,
        },
    )
    .map_err(service_stop)?;
    statement_page(&request, &statement, offset, limit).map_err(reports_stop)
}

/// Reads every row of the filter, up to [`MAX_FETCH_ROWS`], in ONE build of the report (plan 48
/// P48-S2). A page call works the whole report out again, so reading R20 in pages of 1,000 took
/// nine builds, 64 seconds on the NFR-SCL-1 volume, and the question timed out. One row more than
/// the most is asked for, so `total_count` says the report is too big and the rows are all of the
/// filter otherwise.
fn read_all(env: &Env<'_>, id: ReportId, filters: &ReportFilters) -> Result<ReportPage, Stop> {
    read_all_with(|max_rows| {
        run_first_rows(
            env.ctx.db(),
            &ReportRunInput {
                id,
                filters: filters.clone(),
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: false,
            },
            max_rows,
        )
    })
}

/// [`read_all`] with its reader given. The reader is a `FnOnce`, so the report can only be built
/// once, and it is asked for `MAX_FETCH_ROWS + 1` rows.
fn read_all_with(
    read: impl FnOnce(i64) -> Result<ReportPage, ReportsError>,
) -> Result<ReportPage, Stop> {
    let mut page = read(MAX_FETCH_ROWS.saturating_add(1)).map_err(reports_stop)?;
    if page.total_count > MAX_FETCH_ROWS {
        return Err(bad(
            "ERR_OUT_OF_RANGE",
            "where",
            "too many rows to filter; narrow the dates first",
        )
        .into());
    }
    page.limit = 0;
    Ok(page)
}

/// What a column that `where` reads holds. A question number must have the unit that fits it
/// (money review R3): rupees for money, a plain or counted number for a count, a percent for a
/// percent column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColumnUnit {
    /// Money, signed.
    Money,
    /// A balance: compared by its size (see [`mixed_sides`]).
    Balance,
    /// A count or quantity.
    Count,
    /// A percent, in hundredths.
    Percent,
}

/// The unit of the first filled cell of a column; `None` for text, dates and the like.
fn column_unit(rows: &[&Row], index: usize) -> Option<ColumnUnit> {
    match rows
        .iter()
        .filter_map(|r| r.cells.get(index))
        .find(|c| !matches!(c, Cell::Empty))?
    {
        Cell::Money { .. } => Some(ColumnUnit::Money),
        Cell::Balance { .. } => Some(ColumnUnit::Balance),
        Cell::Number { .. } => Some(ColumnUnit::Count),
        Cell::Percent { .. } => Some(ColumnUnit::Percent),
        _ => None,
    }
}

/// The words of the columns that hold numbers, for the refusal of a `where` on a text column.
fn number_words(columns: &[Column], rows: &[&Row]) -> Vec<String> {
    columns
        .iter()
        .enumerate()
        .filter(|(i, c)| !is_left_out(c) && column_unit(rows, *i).is_some())
        .map(|(i, c)| shown_key(i, c, false))
        .collect()
}

/// True when a balance column holds both sides (see [`mixed_sides`](super::cells::mixed_sides)).
fn column_mixes_sides(rows: &[&Row], index: usize) -> bool {
    mixed_sides(rows.iter().filter_map(|r| r.cells.get(index)))
}

/// The value `where` compares with, in the unit of the column (paisa for money, hundredths for a
/// percent, the number itself for a count).
fn where_value(
    env: &Env<'_>,
    state: &State,
    text: &str,
    unit: ColumnUnit,
) -> Result<i128, ToolError> {
    let found = lookup(env, state, text, "where.value")?;
    let mismatch = || {
        bad(
            "ERR_FORMAT_INVALID",
            "where.value",
            "the value is not of the column's kind",
        )
    };
    match found.node() {
        Node::Val(Val::Question { number }) => {
            let typed = question_number(env, text).ok_or_else(mismatch)?;
            match (unit, typed.unit) {
                (
                    ColumnUnit::Money | ColumnUnit::Balance,
                    NumberUnit::Plain | NumberUnit::Amount,
                ) => {
                    let paisa = Paisa::from_rupees(*number).map_err(|_| mismatch())?;
                    Ok(i128::from(paisa.as_paisa()))
                }
                (
                    ColumnUnit::Count,
                    NumberUnit::Plain | NumberUnit::Months | NumberUnit::Days | NumberUnit::Years,
                ) => Ok(i128::from(*number)),
                (ColumnUnit::Percent, NumberUnit::Percent) => {
                    typed.bps.map(i128::from).ok_or_else(mismatch)
                }
                _ => Err(mismatch()),
            }
        }
        Node::Val(val) => figure_value(val, unit).ok_or_else(mismatch),
        _ => Err(mismatch()),
    }
}

/// The value of a figure already read, in the unit of the column it is compared with (money
/// review R4). A balance keeps its side against a signed money column (a credit is below zero)
/// and is a size against a balance column; a difference that went down is below zero against a
/// signed column and a size against a balance column.
fn figure_value(val: &Val, unit: ColumnUnit) -> Option<i128> {
    let money = matches!(unit, ColumnUnit::Money | ColumnUnit::Balance);
    let by_size = unit == ColumnUnit::Balance;
    match val {
        Val::Money(m) if money => {
            let paisa = i128::from(m.paisa.as_paisa());
            match (by_size, m.side) {
                (false, Some(Side::Cr)) => paisa.checked_neg(),
                _ => Some(paisa),
            }
        }
        Val::Diff { size, dir } if money => {
            let paisa = i128::from(size.as_paisa());
            if *dir == Dir::Down && !by_size {
                paisa.checked_neg()
            } else {
                Some(paisa)
            }
        }
        Val::Count(n) if unit == ColumnUnit::Count => Some(i128::from(*n)),
        Val::CountDiff { size, dir } if unit == ColumnUnit::Count => {
            let n = i128::from(*size);
            if *dir == Dir::Down {
                n.checked_neg()
            } else {
                Some(n)
            }
        }
        Val::Pct(h) if unit == ColumnUnit::Percent => Some(i128::from(*h)),
        _ => None,
    }
}

fn passes(op: CompareOp, left: i128, right: i128) -> bool {
    match op {
        CompareOp::Gt => left > right,
        CompareOp::Gte => left >= right,
        CompareOp::Lt => left < right,
        CompareOp::Lte => left <= right,
        CompareOp::Eq => left == right,
    }
}

fn column_index(columns: &[Column], key: &str, field: &str) -> Result<usize, ToolError> {
    columns
        .iter()
        .enumerate()
        .position(|(i, c)| (c.key == key || shown_key(i, c, false) == key) && !is_left_out(c))
        .ok_or_else(|| no_such_column(columns, field))
}

/// A column the report does not have: the words it does have are listed.
fn no_such_column(columns: &[Column], field: &str) -> ToolError {
    bad(
        "ERR_OUT_OF_RANGE",
        field,
        "no such column in this report; use one of the listed words",
    )
    .allowed(column_words(columns, false, None))
}

/// Sums a column of the matching rows with the calculator. `None` when a cell is not of one
/// summable kind.
fn sum_column(rows: &[&Row], index: usize) -> Option<Cell> {
    enum Kind {
        Money,
        Balance,
        Count,
    }
    let first = rows
        .iter()
        .filter_map(|r| r.cells.get(index))
        .find(|c| !matches!(c, Cell::Empty))?;
    let kind = match first {
        Cell::Money { .. } => Kind::Money,
        Cell::Balance { .. } => Kind::Balance,
        Cell::Number { .. } => Kind::Count,
        _ => return None,
    };
    let mut values = Vec::new();
    for row in rows {
        let cell = row.cells.get(index)?;
        let value = match (&kind, cell) {
            (_, Cell::Empty) => continue,
            (Kind::Money, Cell::Money { paisa }) => Value::Money(*paisa),
            (Kind::Balance, Cell::Balance { .. }) => {
                Value::Balance(Paisa::try_from_paisa(signed_paisa(cell)?).ok()?)
            }
            (Kind::Count, Cell::Number { value }) => Value::Count(*value),
            _ => return None,
        };
        values.push(value);
    }
    if values.is_empty() {
        return None;
    }
    // The calculator takes up to 1,000 inputs: sum in chunks, then the sums.
    let mut partial = Vec::new();
    for chunk in values.chunks(1_000) {
        partial.push(figures::calculate(CalcOp::Sum, chunk).ok()?.result);
    }
    let total = if partial.len() == 1 {
        partial.first().copied()?
    } else {
        figures::calculate(CalcOp::Sum, &partial).ok()?.result
    };
    Some(match total {
        Value::Money(paisa) => Cell::Money { paisa },
        Value::Balance(p) => {
            let side = match p.as_paisa().cmp(&0) {
                std::cmp::Ordering::Greater => BalanceSide::Dr,
                std::cmp::Ordering::Less => BalanceSide::Cr,
                std::cmp::Ordering::Equal => BalanceSide::Zero,
            };
            Cell::Balance {
                paisa: super::cells::magnitude(p),
                side,
            }
        }
        Value::Count(value) => Cell::Number { value },
        Value::Percent(hundredths) => Cell::Percent { hundredths },
    })
}

/// The label of a summary line as a short key: `report-closing-balance` is `closing`.
fn summary_key(label_id: &str) -> String {
    let short = label_id.strip_prefix("report-").unwrap_or(label_id);
    let short = short.strip_suffix("-balance").unwrap_or(short);
    short.replace('-', "_")
}

/// The tool.
pub fn run_report(env: &Env<'_>, state: &State, args: &Json) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let id: ReportId = serde_json::from_value(Json::String(parsed.id.clone()))
        .map_err(|_| bad("ERR_OUT_OF_RANGE", "id", "R1 to R38"))?;
    let entry = catalog()
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| bad("ERR_OUT_OF_RANGE", "id", "R1 to R38"))?;
    let mut rows_wanted = parsed.rows.unwrap_or(DEFAULT_ROWS);
    if !(1..=MAX_ROWS).contains(&rows_wanted) {
        return Err(bad("ERR_OUT_OF_RANGE", "rows", "1 to 100").into());
    }
    let mut keys = parsed.sort.clone().unwrap_or_default();
    if keys.len() > 2 {
        return Err(bad("ERR_OUT_OF_RANGE", "sort", "up to 2 keys").into());
    }
    // "Newest first by date, then by entry" in a register of the entries in the order they were made:
    // the register's own newest-first order already breaks a tie by entry and line, newest first, in
    // one SQL page (D7, plan 59 S-B1). The second key would only force the whole report to be read.
    if id_is_newest_first_register(id)
        && let [first, second] = keys.as_slice()
        && first.column == "date"
        && matches!(first.order, OrderWord::Desc)
        && matches!(second.column.as_str(), "entry" | "doc")
        && matches!(second.order, OrderWord::Desc)
    {
        keys.truncate(1);
    }
    let mut built = build_filters(env, state, &parsed, &entry.filters)?;
    if entry.party_required && id != ReportId::R1 && id != ReportId::R31 && built.party.is_none() {
        return Err(bad("ERR_REQUIRED", "party", "this report is one party's").into());
    }
    let mut hidden_parts = false;
    let rule = report_rule(id, &built.filters);
    let partner_data = rule != PartnerRule::Free && !env.partner_locked;
    match rule {
        PartnerRule::Pause if env.partner_locked => return Err(Stop::Locked),
        PartnerRule::HideParts if env.partner_locked => {
            built.filters.hide_partners = true;
            hidden_parts = true;
        }
        _ => {}
    }
    if id == ReportId::R31 && built.filters.partner_id.is_none() {
        let mut draft = partner_overview(env, args, &entry.title_id, &built.source)?;
        draft.partner_data = partner_data;
        return Ok(draft);
    }
    // A sort word the report has no column for is left out, and the note says so (plan 52 item
    // 2). The rows then come in the report's own order, so every row is given (up to 100) and
    // never a cut that looks like a top list; a longer report still needs a real column.
    if !keys.is_empty() && id != ReportId::R31 {
        let probe = read_page(env, id, &built.filters, 0, 1, None)?;
        let (known, dropped): (Vec<SortKey>, Vec<SortKey>) = keys
            .into_iter()
            .partition(|k| column_index(&probe.columns, &k.column, "sort.column").is_ok());
        if !dropped.is_empty() {
            if known.is_empty() && probe.total_count > MAX_ROWS {
                return Err(no_such_column(&probe.columns, "sort.column").into());
            }
            let words: Vec<&str> = dropped.iter().map(|k| k.column.as_str()).collect();
            built.notes.push(format!(
                "the sort word {} is not a column of this report, so it was left out{}; the sort \
                 words of this report are {}",
                words.join(", "),
                if known.is_empty() {
                    ", every row is given in the report's own order"
                } else {
                    ""
                },
                column_words(&probe.columns, false, None).join(", ")
            ));
            if known.is_empty() {
                rows_wanted = MAX_ROWS;
            }
        }
        keys = known;
    }
    built.ranked = !keys.is_empty();
    let in_memory = parsed.where_.is_some() || keys.len() > 1;
    let (page, rows, matched) = if in_memory {
        if id == ReportId::R31 {
            return Err(bad("ERR_OUT_OF_RANGE", "where", "a statement is read as it is").into());
        }
        let page = read_all(env, id, &built.filters)?;
        let (rows, matched) = filter_and_sort(env, state, &page, &parsed, &keys, rows_wanted)?;
        (page, rows, matched)
    } else {
        let sort = report_sort(id, &keys, env);
        match read_page(env, id, &built.filters, 0, rows_wanted, sort) {
            Ok(page) => {
                let rows = page.rows.clone();
                (page, rows, None)
            }
            // The report engine knows its own column keys; the model uses the words it was shown
            // (`name`, `entry`). A shown word the engine did not know is sorted in memory, like a
            // second key; a word the report has no column for is refused with the list.
            Err(Stop::Error(e)) if e.field.as_deref() == Some("sort.column") => {
                let probe = read_page(env, id, &built.filters, 0, 1, None)?;
                let known = keys
                    .iter()
                    .all(|k| column_index(&probe.columns, &k.column, "sort.column").is_ok());
                if !known {
                    return Err(Stop::Error(no_such_column(&probe.columns, "sort.column")));
                }
                if id == ReportId::R31 {
                    return Err(
                        bad("ERR_OUT_OF_RANGE", "sort", "a statement is read as it is").into(),
                    );
                }
                let page = read_all(env, id, &built.filters)?;
                let (rows, matched) =
                    filter_and_sort(env, state, &page, &parsed, &keys, rows_wanted)?;
                (page, rows, matched)
            }
            Err(other) => return Err(other),
        }
    };
    let closing = closing_check(env, id, &built, &page)?;
    let mut draft = assemble(
        env,
        args,
        id,
        &built,
        &page,
        &rows,
        matched,
        hidden_parts,
        closing,
    );
    draft.partner_data = partner_data;
    // A partner's capital, next to his statement, when the statement runs up to now (his capital
    // balance is today's): Q203 and Q204 of the live run 2 found only his کل رقم. Only when the
    // question asks for the capital: its `balance` is a figure of its own, and beside the
    // statement's closing it was cited for "kul raqam" (live run 1, Q198 and Q199: 0, with a tick;
    // Q546 to Q548: Usman's capital for his کل رقم).
    if id == ReportId::R31
        && cues::capital_asked(env.question)
        && built.filters.partner_id.is_some()
        && page
            .filters
            .to
            .as_deref()
            .is_some_and(|to| to < env.today.to_iso().as_str())
        && let Node::Obj(fields) = &mut draft.node
    {
        // The capital is worked out for now only: an earlier day has none, and the closing of the
        // statement is his کل رقم, not his capital (money review).
        fields.push((
            "capital".to_owned(),
            Node::obj(vec![(
                "note",
                Node::Str(
                    "his capital is given as of now only; for an earlier day it is not worked \
                     out, and the closing of this statement is his کل رقم, not his capital"
                        .to_owned(),
                ),
            )]),
        ));
    } else if id == ReportId::R31
        && cues::capital_asked(env.question)
        && let Some(partner_id) = built.filters.partner_id
        && page
            .filters
            .to
            .as_deref()
            .is_none_or(|to| to >= env.today.to_iso().as_str())
    {
        let overview = partners::overview(env.ctx).map_err(service_stop)?;
        let second_place = overview
            .iter()
            .find(|row| row.partner_id == partner_id)
            .map_or(Paisa::ZERO, |row| row.capital_balance_paisa);
        let capital = capital_node(env, partner_id, second_place)?;
        if let Node::Obj(fields) = &mut draft.node {
            fields.push(("capital".to_owned(), capital));
        }
    }
    // The fruit taken by each partner, when the register is read for everyone.
    if id == ReportId::R37 && !env.partner_locked && parsed.where_.is_none() {
        match fruit_by_for(env, &built, &page)? {
            Some(by_for) => {
                if let Node::Obj(fields) = &mut draft.node {
                    fields.push(("by_for".to_owned(), by_for));
                }
                super::typed::add_note(
                    &mut draft.node,
                    "by_for gives each partner's own fruit taken for his home (home_qty, \
                     home_value) and the fruit for the shop's guests (guests): guest fruit is \
                     shared equally by the partners and is not in a partner's figure",
                );
                draft.partner_data = true;
            }
            None => super::typed::add_note(
                &mut draft.node,
                "the totals of each partner did not add up to the total of the register, so they \
                 are not given; read R37 with `partner` set for one partner's total",
            ),
        }
    }
    Ok(draft)
}

/// The rows that pass `where`, sorted by up to two keys, cut to the rows wanted; and `matched`.
fn filter_and_sort(
    env: &Env<'_>,
    state: &State,
    page: &ReportPage,
    parsed: &Input,
    keys: &[SortKey],
    rows_wanted: i64,
) -> Result<(Vec<Row>, Option<Matched>), ToolError> {
    // Only the report's own lines: a heading or a total line holds the sums of the lines under it
    // and would be counted twice (money review R1).
    let mut kept: Vec<&Row> = page
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .collect();
    let mut matched = None;
    if let Some(clause) = &parsed.where_ {
        let index = column_index(&page.columns, &clause.column, "where.column")?;
        // A column with no filled cell (an empty report, or an aging column nobody is in) has
        // nothing that can pass: the count is 0, not a refusal (plan 51 B2, "kaun sa maal 2 din se
        // para hai" when no goods are waiting).
        let filled = kept
            .iter()
            .filter_map(|r| r.cells.get(index))
            .any(|c| !matches!(c, Cell::Empty));
        if filled {
            let unit = column_unit(&kept, index).ok_or_else(|| {
                bad(
                    "ERR_FORMAT_INVALID",
                    "where.column",
                    "this column is not a number; use one of the listed number columns",
                )
                .allowed(number_words(&page.columns, &kept))
            })?;
            if unit == ColumnUnit::Balance && column_mixes_sides(&kept, index) {
                return Err(mixed_sides_error("where.column"));
            }
            let right = where_value(env, state, &clause.value, unit)?;
            kept.retain(|row| {
                row.cells
                    .get(index)
                    .is_some_and(|cell| match order_of(cell, true) {
                        Order::Num(left) => passes(clause.op, left, right),
                        _ => false,
                    })
            });
        } else {
            // The value is still checked: it must be a number of the question or a figure read.
            lookup(env, state, &clause.value, "where.value")?;
            kept.clear();
        }
        let mut totals = Vec::new();
        for (i, column) in page.columns.iter().enumerate() {
            if is_left_out(column) {
                continue;
            }
            if let Some(total) = sum_column(&kept, i) {
                totals.push((i, total));
            }
        }
        matched = Some(Matched {
            count: i64::try_from(kept.len()).unwrap_or(i64::MAX),
            totals,
        });
    }
    if !keys.is_empty() {
        let ur = matches!(env.language, mandi_services::core::auth::Language::Ur);
        let mut spec = Vec::new();
        for key in keys {
            let index = column_index(&page.columns, &key.column, "sort.column")?;
            if column_unit(&kept, index) == Some(ColumnUnit::Balance)
                && column_mixes_sides(&kept, index)
            {
                return Err(mixed_sides_error("sort.column"));
            }
            spec.push((index, key.order));
        }
        // "Newest first" by date: in a register of the entries in the order they were made, equal days
        // run newest line first, so the first row is the last line entered (D7, plan 59 F1, Q2436). The
        // sorts below are stable, so reversing the rows first reverses every tie.
        if let Some((index, OrderWord::Desc)) = spec.first()
            && page
                .columns
                .get(*index)
                .is_some_and(|c| c.kind == mandi_reports::reports::ColumnKind::Date)
            && mandi_reports::reports::newest_first_reverses(page.id)
        {
            kept.reverse();
        }
        // Stable: the second key first, then the first key decides.
        for (index, order) in spec.iter().rev() {
            kept.sort_by(|a, b| {
                let left = a.cells.get(*index);
                let right = b.cells.get(*index);
                let ordering = match (left, right) {
                    (Some(l), Some(r)) => compare_cells(l, r, ur),
                    _ => std::cmp::Ordering::Equal,
                };
                match order {
                    OrderWord::Asc => ordering,
                    OrderWord::Desc => ordering.reverse(),
                }
            });
        }
    }
    let take = usize::try_from(rows_wanted).unwrap_or(0);
    let rows = kept.into_iter().take(take).cloned().collect();
    Ok((rows, matched))
}

fn mixed_sides_error(field: &str) -> ToolError {
    bad(
        "ERR_OUT_OF_RANGE",
        field,
        "this column holds both sides; use the column of what is owed to him or by him (payable, owes_firm)",
    )
}

/// The count and totals of the rows that passed `where`.
struct Matched {
    count: i64,
    totals: Vec<(usize, Cell)>,
}

#[allow(clippy::too_many_arguments)]
fn assemble(
    env: &Env<'_>,
    args: &Json,
    id: ReportId,
    built: &Built,
    page: &ReportPage,
    rows: &[Row],
    matched: Option<Matched>,
    hidden_parts: bool,
    closing: Option<Money>,
) -> Draft {
    let kept: Vec<usize> = page
        .columns
        .iter()
        .enumerate()
        .filter(|(_, c)| !is_left_out(c))
        .map(|(i, _)| i)
        .collect();
    let keys_of_columns: Vec<Node> = kept
        .iter()
        .filter_map(|i| Some(shown_key(*i, page.columns.get(*i)?, false)))
        .map(Node::Str)
        .collect();
    let mut used = Vec::new();
    let mut keyed = Vec::new();
    // A list of balances "as of" an earlier day is not today's party card: nothing to compare.
    let balances_are_past = page
        .filters
        .as_of
        .as_deref()
        .is_some_and(|day| day < env.today.to_iso().as_str());
    let mut headings = Vec::new();
    for (place, row) in rows.iter().enumerate() {
        let absolute = usize::try_from(page.offset)
            .unwrap_or(0)
            .saturating_add(place);
        let key = row_key(row, absolute, id == ReportId::R10, &mut used);
        let mut fields: Vec<(String, Node)> = kept
            .iter()
            .filter_map(|i| {
                Some((
                    shown_key(*i, page.columns.get(*i)?, false),
                    row.cells.get(*i)?,
                ))
            })
            .map(|(k, cell)| (k, cell_node(env, cell, false)))
            .collect();
        if let Some(node) = row_balance(env, id, &page.columns, row, balances_are_past)
            && let Some(slot) = fields.iter_mut().find(|(k, _)| k == "balance")
        {
            slot.1 = node;
        }
        // A heading or total line holds the sums of the lines under it. In a sorted list it is
        // given apart, so it is never read as the biggest or the smallest row (Q194: the total of
        // all shop expenses was cited as the biggest head).
        if built.ranked && row.kind != RowKind::Normal {
            headings.push((key, Node::Obj(fields)));
        } else {
            keyed.push((key, Node::Obj(fields)));
        }
    }
    let totals: Vec<(String, Node)> = kept
        .iter()
        .filter_map(|i| {
            Some((
                shown_key(*i, page.columns.get(*i)?, false),
                page.totals.get(*i)?,
            ))
        })
        .filter(|(_, cell)| !matches!(cell, Cell::Empty))
        .map(|(k, cell)| match (&closing, id, k.as_str()) {
            // A partner's running total at the end of his statement is the closing, with its ticks.
            (Some(m), ReportId::R31, "running") => (k, Node::Val(Val::Money(m.clone()))),
            _ => (k, cell_node(env, cell, false)),
        })
        .collect();
    let summary: Vec<(String, Node)> = page
        .summary
        .iter()
        .map(|line| {
            let key = summary_key(&line.label_id);
            // A party's closing balance carries its two places when the ledger runs up to now.
            let node = match (&closing, key.as_str()) {
                (Some(m), "closing" | "partners_statement_closing") => {
                    Node::Val(Val::Money(m.clone()))
                }
                _ => cell_node(env, &line.cell, false),
            };
            (key, node)
        })
        .collect();
    let count = matched.as_ref().map_or(page.total_count, |m| m.count);
    let matched_node = matched.as_ref().map_or(Node::Null, |m| {
        let totals = m
            .totals
            .iter()
            .filter_map(|(i, cell)| {
                Some((
                    shown_key(*i, page.columns.get(*i)?, false),
                    cell_node(env, cell, false),
                ))
            })
            .collect();
        Node::obj(vec![
            ("count", Node::Val(Val::Count(m.count))),
            ("totals", Node::Obj(totals)),
        ])
    });
    let mut echo = Vec::new();
    for (name, value) in [
        ("from", page.filters.from.clone()),
        ("to", page.filters.to.clone()),
        ("as_of", page.filters.as_of.clone()),
    ] {
        if let Some(value) = value {
            echo.push((name, Node::Str(value)));
        }
    }
    // A read for a group says so, so a zero for a group with no sales is not taken for the whole.
    if let Some(group) = page.filters.item_group {
        echo.push(("item_group", Node::Str(group.as_str().to_owned())));
    }
    // So the model can see the figure is for one quality, not for the whole item.
    if let Some(grade) = page.filters.grade {
        echo.push(("grade", Node::Str(grade.as_str().to_owned())));
    }
    let mut fields = vec![
        ("source", Node::Str(id.code().to_owned())),
        (
            "title",
            Node::Str(env.words.text(env.language, &page.title_id)),
        ),
        ("filters", Node::obj(echo)),
    ];
    if let Some(party) = &built.party {
        fields.push((
            "party",
            Node::obj(vec![
                ("key", Node::Str(format!("a{}", party.id))),
                (
                    "name",
                    Node::Val(Val::Name(crate::results::NameVal {
                        name_ur: party.name_ur.clone(),
                        name_en: party.name_en.clone(),
                        tag: party.tag.clone(),
                        code: party.code.clone(),
                    })),
                ),
            ]),
        ));
    }
    let mut notes = built.notes.clone();
    // A line paid in part is in R3 and in R4: the note is for the model that adds or subtracts them.
    if matches!(id, ReportId::R3 | ReportId::R4) {
        notes.push(
            "a line paid in part is in both R3 and R4, so no figure of R3 may be added to or taken \
             from a figure of R4, except the paid of R3 with the left of R4 read for the same \
             dates; paid on the spot is the paid total (R3) and left on account is the left \
             total (R4), and both are on R5 (buyer bills) totals"
                .to_owned(),
        );
    }
    if !headings.is_empty() {
        notes.push(
            "the heading and total lines are given apart in `headings`: each holds the sums of \
             the lines under it, so it is not a row to rank"
                .to_owned(),
        );
    }
    if !notes.is_empty() {
        fields.push(("note", Node::Str(notes.join("; "))));
    }
    fields.extend([
        ("columns", Node::List(keys_of_columns)),
        ("rows", Node::Keyed(keyed)),
        // `count` is over the whole filter; with `where`, `matched.count` is the matching rows.
        ("count", Node::Val(Val::Count(page.total_count))),
        ("rows_cut", Node::Bool(false)),
        ("totals", Node::Obj(totals)),
        ("summary", Node::Obj(summary)),
        ("matched", matched_node),
    ]);
    if !headings.is_empty() {
        fields.push(("headings", Node::Keyed(headings)));
    }
    let mut draft = Draft::new("report", args.clone(), Node::obj(fields));
    // With `where` the totals row of the table is the matching rows' totals, not the whole filter's.
    let shown_totals: Vec<Cell> = match &matched {
        Some(m) => (0..page.columns.len())
            .map(|i| {
                m.totals
                    .iter()
                    .find(|(at, _)| *at == i)
                    .map_or(Cell::Empty, |(_, cell)| cell.clone())
            })
            .collect(),
        None => page.totals.clone(),
    };
    draft.shown = Some(Shown::Table(AnswerTable {
        result_id: String::new(),
        title_id: page.title_id.clone(),
        columns: page.columns.clone(),
        rows: rows
            .iter()
            .map(|row| AnswerRow {
                row: row.clone(),
                analytics_key: None,
            })
            .collect(),
        count,
        totals_label_id: page.totals_label_id.clone(),
        totals: shown_totals,
    }));
    draft.source = Some(AnswerSource::Report {
        result_id: String::new(),
        label_id: page.title_id.clone(),
        id,
        filters: built.source.clone(),
    });
    draft.partner_parts_locked = hidden_parts;
    draft
}

/// The closing balance of a party's ledger (R13, R14) read against his party card, when the ledger
/// runs up to now (PRD FR-7.76: the two places of a party's balance).
fn closing_check(
    env: &Env<'_>,
    id: ReportId,
    built: &Built,
    page: &ReportPage,
) -> Result<Option<Money>, Stop> {
    let up_to_now = page
        .filters
        .to
        .as_deref()
        .is_none_or(|to| to >= env.today.to_iso().as_str());
    if id == ReportId::R31 {
        return statement_closing_check(env, built, page, up_to_now);
    }
    let Some(party) = &built.filters.party else {
        return Ok(None);
    };
    if !matches!(id, ReportId::R13 | ReportId::R14) {
        return Ok(None);
    }
    if !up_to_now {
        return Ok(None);
    }
    let card =
        mandi_services::assist::party_card(env.ctx, party.account_id).map_err(service_stop)?;
    Ok(Some(super::party::checked_balance(env, &card)?))
}

/// The closing of a partner's statement (his کل رقم) read in two places when the statement runs up
/// to now: the statement's own closing, and the same partner's `total` in the partner overview,
/// which the services work out apart (`money::total_of`). The ticked figure is then the کل رقم, as
/// the capital is ticked: without it the capital was the figure that looked checked, and it was
/// cited for a کل رقم question (live run 1, Q198, Q199, Q546 to Q548).
///
/// `None` for a statement that ends before today (there is no second place for an earlier day).
fn statement_closing_check(
    env: &Env<'_>,
    built: &Built,
    page: &ReportPage,
    up_to_now: bool,
) -> Result<Option<Money>, Stop> {
    let Some(partner_id) = built.filters.partner_id else {
        return Ok(None);
    };
    if !up_to_now {
        return Ok(None);
    }
    let closing = page
        .summary
        .iter()
        .find(|line| summary_key(&line.label_id) == "partners_statement_closing")
        .and_then(|line| match line.cell {
            Cell::Money { paisa } => Some(paisa),
            _ => None,
        });
    let Some(closing) = closing else {
        return Ok(None);
    };
    let overview = partners::overview(env.ctx).map_err(service_stop)?;
    let Some(row) = overview.iter().find(|row| row.partner_id == partner_id) else {
        return Ok(None);
    };
    Ok(Some(Money {
        paisa: closing,
        side: None,
        checked: Some(row.total_paisa == closing),
        places: vec![("R31", closing), ("partner_overview", row.total_paisa)],
    }))
}

/// The most capital lines read for one partner (the page limit of the history).
const CAPITAL_LINES: u32 = 1_000;

/// A partner's capital (his account 0602) as the model reads it: the balance now (credit is plus,
/// a minus is a debit balance), what he put in (the opening included) and took back, and the profit
/// that month close moved into capital. The balance is read in two places, the capital history and
/// the partner overview (`overview`), and it is checked only when they agree AND the history adds up
/// (`balance = invested − taken back + month close`).
///
/// An opening debit and a جنرل line on the account count as invested or taken back by their side.
///
/// Example (PRD E2E-35): ب invests 50,000: balance 50,000, invested 50,000, taken back 0.
fn capital_node(env: &Env<'_>, partner_id: i64, overview: Paisa) -> Result<Node, Stop> {
    let history = partners::capital_history(
        env.ctx,
        &CapitalHistoryInput {
            partner_id,
            offset: 0,
            limit: CAPITAL_LINES,
            sort: None,
            group_by_month: false,
        },
    )
    .map_err(service_stop)?;
    let balance = history.balance_paisa;
    let (invested, taken) = (
        history.totals.invested_paisa,
        history.totals.taken_back_paisa,
    );
    let overflow = |_| bad("ERR_OVERFLOW", "capital", "the figure does not fit");
    let mut month_close = Paisa::ZERO;
    for row in history
        .rows
        .iter()
        .filter(|r| r.kind == CapitalRowKind::MonthClose)
    {
        month_close = month_close
            .checked_add(row.amount_paisa)
            .map_err(overflow)?;
    }
    // Every line was read, and the three parts make the balance.
    let complete = i64::try_from(history.rows.len()).ok() == Some(history.total_count);
    let adds_up = complete
        && invested
            .checked_sub(taken)
            .and_then(|net| net.checked_add(month_close))
            .is_ok_and(|expected| expected == balance);
    let agree = overview == balance;
    // With more lines than were read, the parts cannot be checked, but the two places still can:
    // the balance is then unchecked only if they differ, and the month close is not given.
    let money = Money {
        paisa: balance,
        side: None,
        checked: Some(if complete { agree && adds_up } else { agree }),
        places: vec![("capital_history", balance), ("partner_overview", overview)],
    };
    let month_close_node = if complete {
        Node::Val(Val::Money(Money::plain(month_close)))
    } else {
        Node::Null
    };
    Ok(Node::obj(vec![
        ("as_of", Node::Str(env.today.to_iso())),
        ("balance", Node::Val(Val::Money(money))),
        ("invested", Node::Val(Val::Money(Money::plain(invested)))),
        ("taken_back", Node::Val(Val::Money(Money::plain(taken)))),
        ("month_close", month_close_node),
        (
            "note",
            Node::Str(
                "balance is his capital now: invested, less taken back, plus the profit month \
                 close moved in (month_close); a minus balance is a debit balance"
                    .to_owned(),
            ),
        ),
    ]))
}

/// The fruit taken for each partner's home and for the shop's guests, from R37 itself: the same
/// register read once for each partner and once for the guests, so every figure is the engine's
/// SUM over the whole filter (never a page). Every partner counts, active or not. `None` when the
/// parts do not add up to the register's own total (a save landed between the reads, or an entry
/// is in no group): the node is then left out, never given unchecked.
fn fruit_by_for(env: &Env<'_>, built: &Built, page: &ReportPage) -> Result<Option<Node>, Stop> {
    if built.filters.partner_id.is_some() || built.filters.guests_only == Some(true) {
        return Ok(None);
    }
    let at = |key: &str| page.columns.iter().position(|c| c.key == key);
    let (Some(qty_at), Some(value_at)) = (at("qty"), at("value")) else {
        return Ok(None);
    };
    let totals_of = |read: &ReportPage| -> Option<(i64, Paisa)> {
        match (read.totals.get(qty_at)?, read.totals.get(value_at)?) {
            (Cell::Number { value }, Cell::Money { paisa }) => Some((*value, *paisa)),
            _ => None,
        }
    };
    let Some((grand_qty, grand_value)) = totals_of(page) else {
        return Ok(None);
    };
    let mut parts: Vec<(
        Option<mandi_services::core::partners::PartnerRow>,
        i64,
        Paisa,
    )> = Vec::new();
    for partner in partners::list(env.ctx).map_err(service_stop)? {
        let mut filters = built.filters.clone();
        filters.partner_id = Some(partner.id);
        let read = read_page(env, ReportId::R37, &filters, 0, 1, None)?;
        let Some((qty, value)) = totals_of(&read) else {
            return Ok(None);
        };
        parts.push((Some(partner), qty, value));
    }
    let mut guests = built.filters.clone();
    guests.partner_id = None;
    guests.guests_only = Some(true);
    let read = read_page(env, ReportId::R37, &guests, 0, 1, None)?;
    let Some((guest_qty, guest_value)) = totals_of(&read) else {
        return Ok(None);
    };
    parts.push((None, guest_qty, guest_value));
    let mut qty_sum: i64 = 0;
    let mut value_sum = Paisa::ZERO;
    for (_, qty, value) in &parts {
        let (Some(q), Ok(v)) = (qty_sum.checked_add(*qty), value_sum.checked_add(*value)) else {
            return Ok(None);
        };
        (qty_sum, value_sum) = (q, v);
    }
    if qty_sum != grand_qty || value_sum != grand_value {
        return Ok(None);
    }
    let money = |p: Paisa| Node::Val(Val::Money(Money::plain(p)));
    let rows: Vec<(String, Node)> = parts
        .into_iter()
        .filter(|(who, qty, value)| who.is_none() || *qty != 0 || *value != Paisa::ZERO)
        .map(|(who, qty, value)| match who {
            Some(p) => (
                format!("s{}", p.id),
                Node::obj(vec![
                    (
                        "name",
                        Node::Val(Val::Name(NameVal {
                            name_ur: p.name_ur,
                            name_en: p.name_en,
                            tag: None,
                            code: None,
                        })),
                    ),
                    ("home_qty", Node::Val(Val::Count(qty))),
                    ("home_value", money(value)),
                ]),
            ),
            None => (
                "guests".to_owned(),
                Node::obj(vec![
                    ("qty", Node::Val(Val::Count(qty))),
                    ("value", money(value)),
                ]),
            ),
        })
        .collect();
    Ok(Some(Node::Keyed(rows)))
}

/// R31 with no partner: one row for each partner with his کل رقم, read in two places (the partner
/// overview and the closing total of his statement, PRD FR-7.76).
fn partner_overview(
    env: &Env<'_>,
    args: &Json,
    title_id: &str,
    source: &SourceReportFilters,
) -> Result<Draft, Stop> {
    let rows = partners::overview(env.ctx).map_err(service_stop)?;
    let capital_asked = cues::capital_asked(env.question);
    // The days asked for. The overview itself is today's; with a day the figure is each partner's
    // statement closing at the end of `to` (money review: a dated read showed today's total, with
    // a tick). There is no second place for an earlier day, so it has no tick.
    let day = |iso: &str| {
        (!iso.is_empty())
            .then(|| BusinessDate::parse_iso(iso).map(format_ddmmyyyy))
            .flatten()
    };
    let (from, to) = (day(&source.from), day(&source.to));
    let up_to_now = source.to.is_empty() || source.to.as_str() >= env.today.to_iso().as_str();
    let mut keyed = Vec::new();
    let mut table_rows = Vec::new();
    for row in &rows {
        let statement = partners::statement(
            env.ctx,
            &PartnersStatementInput {
                partner_id: row.partner_id,
                from: from.clone(),
                to: to.clone(),
                offset: 0,
                limit: 1,
                sort: None,
                group_by_month: false,
            },
        )
        .map_err(service_stop)?;
        let closing = statement.closing_total_paisa;
        let (overview, money) = if up_to_now {
            let overview = row.total_paisa;
            (
                overview,
                Money {
                    paisa: overview,
                    side: None,
                    checked: Some(overview == closing),
                    places: vec![("partner_overview", overview), ("R31", closing)],
                },
            )
        } else {
            (closing, Money::plain(closing))
        };
        let mut fields = vec![
            (
                "name",
                Node::Val(Val::Name(NameVal {
                    name_ur: row.name_ur.clone(),
                    name_en: row.name_en.clone(),
                    tag: None,
                    code: None,
                })),
            ),
            ("total", Node::Val(Val::Money(money))),
        ];
        // The capital only when the question asks for it: its `balance` beside `total` was cited
        // for a کل رقم question (live run 1, Q198 to Q199, Q546 to Q548).
        if capital_asked {
            fields.push((
                "capital",
                capital_node(env, row.partner_id, row.capital_balance_paisa)?,
            ));
        }
        keyed.push((format!("s{}", row.partner_id), Node::obj(fields)));
        table_rows.push(AnswerRow {
            row: Row {
                cells: vec![
                    Cell::Names {
                        names: mandi_reports::reports::Names {
                            ur: row.name_ur.clone(),
                            en: row.name_en.clone(),
                        },
                    },
                    Cell::Money { paisa: overview },
                ],
                source: None,
                kind: mandi_reports::reports::RowKind::Normal,
            },
            analytics_key: None,
        });
    }
    let count = i64::try_from(keyed.len()).unwrap_or(i64::MAX);
    let mut echo = Vec::new();
    if !source.from.is_empty() {
        echo.push(("from", Node::Str(source.from.clone())));
    }
    if !source.to.is_empty() {
        echo.push(("to", Node::Str(source.to.clone())));
    }
    let mut fields = vec![
        ("source", Node::Str("R31".to_owned())),
        ("title", Node::Str(env.words.text(env.language, title_id))),
        ("filters", Node::obj(echo)),
    ];
    if !up_to_now {
        fields.push((
            "note",
            Node::Str(
                "each total is his کل رقم at the end of `to`, worked from his statement; an \
                 earlier day has no second place, so it is not checked"
                    .to_owned(),
            ),
        ));
    }
    fields.extend([
        (
            "columns",
            Node::List(vec![
                Node::Str("name".to_owned()),
                Node::Str("total".to_owned()),
            ]),
        ),
        ("rows", Node::Keyed(keyed)),
        ("count", Node::Val(Val::Count(count))),
        ("rows_cut", Node::Bool(false)),
        ("totals", Node::Obj(Vec::new())),
        ("summary", Node::Obj(Vec::new())),
        ("matched", Node::Null),
    ]);
    let node = Node::obj(fields);
    let mut draft = Draft::new("report", args.clone(), node);
    draft.shown = Some(Shown::Table(AnswerTable {
        result_id: String::new(),
        title_id: title_id.to_owned(),
        columns: vec![
            Column {
                key: "name".to_owned(),
                title_id: "report-col-partner".to_owned(),
                kind: mandi_reports::reports::ColumnKind::Names,
            },
            Column {
                key: "total".to_owned(),
                title_id: "report-col-total".to_owned(),
                kind: mandi_reports::reports::ColumnKind::Money,
            },
        ],
        rows: table_rows,
        count,
        totals_label_id: "report-total".to_owned(),
        totals: vec![Cell::Empty, Cell::Empty],
    }));
    draft.source = Some(AnswerSource::Report {
        result_id: String::new(),
        label_id: title_id.to_owned(),
        id: ReportId::R31,
        filters: source.clone(),
    });
    Ok(draft)
}

/// A party's balance on a row of the dues and payables lists (R20 to R22, R24): with its side and
/// read against his party card and the closing of his ledger (PRD FR-7.76). A چکتا buyer
/// shares one account with others, so his row is not checked against the account's card.
fn row_balance(
    env: &Env<'_>,
    id: ReportId,
    columns: &[Column],
    row: &Row,
    past: bool,
) -> Option<Node> {
    if !matches!(
        id,
        ReportId::R20 | ReportId::R21 | ReportId::R22 | ReportId::R24
    ) {
        return None;
    }
    let party = row.cells.iter().find_map(|c| match c {
        Cell::Party { party } => Some(party),
        _ => None,
    })?;
    let index = columns.iter().position(|c| c.key == "balance")?;
    let (paisa, side) = match row.cells.get(index)? {
        Cell::Money { paisa } => (*paisa, Side::Dr),
        Cell::Balance { paisa, side } => (*paisa, super::party::side_of(*side)),
        _ => return None,
    };
    // A چکتا buyer, or a list as of an earlier day: no second place (money review R2).
    if party.person_key.is_some() || past {
        return Some(Node::Val(Val::Money(Money::balance(paisa, side))));
    }
    let card = mandi_services::assist::party_card(env.ctx, party.account_id).ok()?;
    let seen = super::party::checked_balance(env, &card).ok()?;
    let agree = seen.checked == Some(true) && seen.paisa == paisa && seen.side == Some(side);
    Some(Node::Val(Val::Money(Money {
        paisa,
        side: Some(side),
        checked: Some(agree),
        places: seen.places,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rupees(n: i64) -> Paisa {
        Paisa::from_rupees(n).unwrap()
    }

    fn money(n: i64, side: Option<Side>) -> Val {
        Val::Money(Money {
            paisa: rupees(n),
            side,
            checked: None,
            places: Vec::new(),
        })
    }

    /// A page of R20 with no rows that says its filter has `total_count` rows.
    fn page_of_count(total_count: i64) -> ReportPage {
        ReportPage {
            id: ReportId::R20,
            title_id: String::new(),
            filters: Default::default(),
            columns: Vec::new(),
            rows: Vec::new(),
            offset: 0,
            limit: PAGE_OF_THE_SCREEN,
            total_count,
            totals_label_id: String::new(),
            totals: Vec::new(),
            summary: Vec::new(),
            tables: Vec::new(),
            group_above: None,
        }
    }

    /// The limit a page of the screen would report (not 0, which is the sign of every row).
    const PAGE_OF_THE_SCREEN: i64 = 1_000;

    #[test]
    fn every_row_is_read_in_one_build_asking_for_one_row_more_than_the_most() {
        // Plan 48 P48-S2: nine page reads of R20 took 64 seconds on the NFR-SCL-1 volume. The
        // reader is called once, with the most plus one, and its page is given back with the
        // sign of "every row" (limit 0).
        let mut asked = Vec::new();
        let page = read_all_with(|max_rows| {
            asked.push(max_rows);
            Ok(page_of_count(8_998))
        });
        assert_eq!(asked, vec![MAX_FETCH_ROWS + 1]);
        match page {
            Ok(page) => assert_eq!((page.total_count, page.limit), (8_998, 0)),
            Err(_) => panic!("8,998 rows are under the most and must be read"),
        }
    }

    #[test]
    fn a_report_over_the_most_is_refused_after_that_one_build() {
        let mut calls = 0;
        let page = read_all_with(|_| {
            calls += 1;
            Ok(page_of_count(MAX_FETCH_ROWS + 1))
        });
        assert_eq!(calls, 1);
        assert!(page.is_err(), "10,001 rows are over the most");
        // Exactly the most is read.
        assert!(read_all_with(|_| Ok(page_of_count(MAX_FETCH_ROWS))).is_ok());
    }

    #[test]
    fn a_figure_keeps_its_sign_against_a_signed_column_and_is_a_size_against_a_balance_column() {
        let p = |n: i64| i128::from(rupees(n).as_paisa());
        // A credit balance of 10,160 is below zero next to a signed money column ...
        let credit = money(10_160, Some(Side::Cr));
        assert_eq!(figure_value(&credit, ColumnUnit::Money), Some(-p(10_160)));
        // ... and a size next to a balance column, which compares sizes.
        assert_eq!(figure_value(&credit, ColumnUnit::Balance), Some(p(10_160)));
        // A debit balance and a money figure with no side are as they are.
        assert_eq!(
            figure_value(&money(2_880, Some(Side::Dr)), ColumnUnit::Money),
            Some(p(2_880))
        );
        assert_eq!(
            figure_value(&money(1_940, None), ColumnUnit::Money),
            Some(p(1_940))
        );
        // A difference of 500 going down is -500 next to a signed column, 500 next to a balance.
        let down = Val::Diff {
            size: rupees(500),
            dir: Dir::Down,
        };
        let up = Val::Diff {
            size: rupees(500),
            dir: Dir::Up,
        };
        assert_eq!(figure_value(&down, ColumnUnit::Money), Some(-p(500)));
        assert_eq!(figure_value(&down, ColumnUnit::Balance), Some(p(500)));
        assert_eq!(figure_value(&up, ColumnUnit::Money), Some(p(500)));
        // A count goes down the same way; a money figure is never a count.
        let fewer = Val::CountDiff {
            size: 3,
            dir: Dir::Down,
        };
        assert_eq!(figure_value(&fewer, ColumnUnit::Count), Some(-3));
        assert_eq!(figure_value(&credit, ColumnUnit::Count), None);
        assert_eq!(figure_value(&Val::Count(7), ColumnUnit::Money), None);
        // A percent (hundredths) only meets a percent column.
        assert_eq!(
            figure_value(&Val::Pct(7_210), ColumnUnit::Percent),
            Some(7_210)
        );
        assert_eq!(figure_value(&Val::Pct(7_210), ColumnUnit::Money), None);
    }
}
