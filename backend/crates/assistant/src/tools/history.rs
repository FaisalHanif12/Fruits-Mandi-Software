//! `report{id:"R13", layout:"bikri"}`: one party's history with one row for each بکری or entry, the
//! same list as the پارٹی کھاتے screen and its statement print (PRD FR-19.8 v1.43, FR-7.87 v1.60,
//! PRD 12.12).
//!
//! It calls `mandi_reports::reports::statement_rows`, the function of that screen (Code Standards
//! Q3). The rows come newest first, an edited entry once in its latest form and a cancelled one left
//! out, as on the screen. The totals are those of the whole period, never of the rows given. The
//! closing balance, when the period runs up to now, is read in two places like every party balance:
//! the party card and the closing of R13, and the statement's own closing must be the same (PRD
//! FR-7.76). Nothing is added or rounded here (Code Standards Q5).
//!
//! Example (worked day): اکرم, 31-08-26 to 10-10-26: two rows, newest first: وصولی 1 (credit
//! 10,000, balance 2,880 بنام) and بکری 1 (debit 12,880, balance 12,880 بنام); closing 2,880 بنام.

use mandi_reports::reports::{
    Cell, Column, ColumnKind, ReportId, Row, RowKind, StatementRowsInput, statement_rows,
};
use mandi_services::core::accounts::Balance;
use mandi_services::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_services::core::money::Paisa;

use super::cells::cell_node;
use super::records::named;
use super::report::{Dated, Input, add_dated, reports_stop};
use super::{Draft, Env, State, Stop, ToolError, date_arg, service_stop};
use crate::answer::{AnswerRow, AnswerSource, AnswerTable};
use crate::results::{Money, NameVal, Node, Shown, Val};
use crate::source::{SourceParty, SourceReportFilters};

/// The word of `layout` for this list.
pub const LAYOUT_BIKRI: &str = "bikri";

/// The columns, as (the key the model reads, the heading, the kind).
const COLUMNS: [(&str, &str, ColumnKind); 7] = [
    ("date", "report-col-date", ColumnKind::Date),
    ("entry", "report-col-doc", ColumnKind::Doc),
    ("detail", "report-col-detail", ColumnKind::Detail),
    ("debit", "report-col-debit", ColumnKind::Money),
    ("paid_on_spot", "report-col-paid-on-spot", ColumnKind::Money),
    ("credit", "report-col-credit", ColumnKind::Money),
    ("balance", "report-col-balance", ColumnKind::Balance),
];

fn refuse(field: &str, detail: &str) -> ToolError {
    ToolError::new("ERR_OUT_OF_RANGE", field, detail)
}

/// The inputs this layout does not take. A filter it does not have is an error, never left out
/// without a word (Code Standards Q1).
fn only_party_and_dates(parsed: &Input) -> Result<(), ToolError> {
    let taken = [
        ("item", parsed.item.is_some()),
        ("item_group", parsed.item_group.is_some()),
        ("grade", parsed.grade.is_some()),
        ("city", parsed.city.is_some()),
        ("adda", parsed.adda.is_some()),
        ("collector", parsed.collector.is_some()),
        ("partner", parsed.partner.is_some()),
        ("trip", parsed.trip.is_some()),
        ("garden", parsed.garden.is_some()),
        ("bikri", parsed.bikri.is_some()),
        ("seller", parsed.seller.is_some()),
        ("zamindar", parsed.zamindar.is_some()),
        ("status", parsed.status.is_some()),
        ("voucher_type", parsed.voucher_type.is_some()),
        ("group_by", parsed.group_by.is_some()),
        ("as_of", parsed.as_of.is_some()),
    ];
    if let Some((field, _)) = taken.iter().find(|(_, given)| *given) {
        return Err(refuse(
            field,
            "layout bikri takes party, from, to, period and rows only",
        ));
    }
    if parsed.where_.is_some() || parsed.sort.as_ref().is_some_and(|s| !s.is_empty()) {
        return Err(refuse(
            "sort",
            "layout bikri is read as it is, newest first; leave where and sort null",
        ));
    }
    Ok(())
}

fn money_cell(paisa: Paisa) -> Cell {
    if paisa == Paisa::ZERO {
        Cell::Empty
    } else {
        Cell::Money { paisa }
    }
}

fn balance_cell(balance: &Balance) -> Cell {
    Cell::Balance {
        paisa: balance.amount_paisa,
        side: balance.side,
    }
}

/// The closing of the list up to now, checked in the two places of a party's balance (the party card
/// and the closing of R13) and against the list's own closing.
fn checked_closing(env: &Env<'_>, account_id: i64, closing: &Balance) -> Result<Money, Stop> {
    let card = mandi_services::assist::party_card(env.ctx, account_id).map_err(service_stop)?;
    let mut money = super::party::checked_balance(env, &card)?;
    let side = super::party::side_of(closing.side);
    if closing.amount_paisa != money.paisa || Some(side) != money.side {
        money.checked = Some(false);
        money.places.push(("party_ledgers", closing.amount_paisa));
    }
    Ok(money)
}

/// Reads the list. `dated` holds the period of the `period` word and the input as read.
pub(super) fn run(
    env: &Env<'_>,
    state: &State,
    parsed: &Input,
    rows_wanted: i64,
    title_id: &str,
    dated: Dated,
) -> Result<Draft, Stop> {
    only_party_and_dates(parsed)?;
    let text = parsed
        .party
        .as_deref()
        .ok_or_else(|| ToolError::new("ERR_REQUIRED", "party", "this report is one party's"))?;
    let (account_id, record) = named(env, state, text, "party", &["a"])?;
    let day = |field: &str, value: &Option<String>| -> Result<Option<String>, ToolError> {
        Ok(value
            .as_deref()
            .map(|t| date_arg(env, state, t, field))
            .transpose()?
            .flatten())
    };
    let from = day("from", &parsed.from)?;
    let to = day("to", &parsed.to)?;
    let typed = |iso: &Option<String>| {
        iso.as_deref()
            .and_then(BusinessDate::parse_iso)
            .map(format_ddmmyyyy)
    };
    let statement = statement_rows(
        env.ctx.db(),
        &StatementRowsInput {
            account_id,
            from: typed(&from),
            to: typed(&to),
            include_history: false,
            newest_first: true,
            offset: 0,
            limit: u32::try_from(rows_wanted).unwrap_or(0),
        },
    )
    .map_err(|e| match reports_stop(e) {
        Stop::Error(inner) if inner.code != "ERR_DATE_RANGE" => Stop::Error(refuse(
            "layout",
            "layout bikri is for one buyer or seller; for another account leave layout null",
        )),
        other => other,
    })?;

    let up_to_now = to
        .as_deref()
        .is_none_or(|t| t >= env.today.to_iso().as_str());
    let closing_value = if up_to_now {
        Val::Money(checked_closing(env, account_id, &statement.closing)?)
    } else {
        Val::Money(Money::balance(
            statement.closing.amount_paisa,
            super::party::side_of(statement.closing.side),
        ))
    };

    let mut keyed = Vec::new();
    let mut shown_rows = Vec::new();
    for row in &statement.rows {
        let cells = vec![
            Cell::Date {
                iso: row.date.clone(),
            },
            Cell::Doc {
                type_id: row.type_id.clone(),
                number: row.number,
            },
            Cell::Detail {
                parts: row.parts.clone(),
            },
            money_cell(row.dr_paisa),
            money_cell(row.on_spot_paisa),
            money_cell(row.cr_paisa),
            balance_cell(&row.balance),
        ];
        let fields: Vec<(String, Node)> = COLUMNS
            .iter()
            .zip(&cells)
            .map(|((key, _, _), cell)| ((*key).to_owned(), cell_node(env, cell, false)))
            .collect();
        keyed.push((format!("je{}", row.entry_id), Node::Obj(fields)));
        shown_rows.push(AnswerRow {
            row: Row {
                cells,
                source: Some(row.source),
                kind: RowKind::Normal,
            },
            analytics_key: None,
        });
    }
    let total = |paisa: Paisa| Node::Val(Val::Money(Money::plain(paisa)));
    let columns_node = COLUMNS
        .iter()
        .map(|(key, _, _)| Node::Str((*key).to_owned()))
        .collect();
    let filters_node: Vec<(&str, Node)> = [("from", &from), ("to", &to)]
        .into_iter()
        .filter_map(|(k, v)| v.clone().map(|v| (k, Node::Str(v))))
        .collect();
    let mut notes = vec![
        "one row for each بکری or entry, newest first, as on پارٹی کھاتے; debit and credit are \
         that entry's; paid_on_spot is part of credit, so never add them"
            .to_owned(),
    ];
    notes.extend(dated.note.clone());
    let count = statement.count;
    let node = Node::obj(vec![
        ("source", Node::Str(ReportId::R13.code().to_owned())),
        ("layout", Node::Str(LAYOUT_BIKRI.to_owned())),
        ("title", Node::Str(env.words.text(env.language, title_id))),
        ("filters", Node::obj(filters_node)),
        (
            "party",
            Node::obj(vec![
                ("key", Node::Str(format!("a{}", record.id))),
                (
                    "name",
                    Node::Val(Val::Name(NameVal {
                        name_ur: record.name_ur.clone(),
                        name_en: record.name_en.clone(),
                        tag: record.tag.clone(),
                        code: record.code.clone(),
                    })),
                ),
            ]),
        ),
        ("note", Node::Str(notes.join("; "))),
        ("columns", Node::List(columns_node)),
        ("rows", Node::Keyed(keyed)),
        ("count", Node::Val(Val::Count(count))),
        ("rows_cut", Node::Bool(false)),
        (
            "totals",
            Node::obj(vec![
                ("debit", total(statement.dr_total_paisa)),
                ("paid_on_spot", total(statement.on_spot_total_paisa)),
                ("credit", total(statement.cr_total_paisa)),
            ]),
        ),
        (
            "summary",
            Node::obj(vec![
                (
                    "opening",
                    cell_node(env, &balance_cell(&statement.brought_forward), false),
                ),
                ("closing", Node::Val(closing_value)),
            ]),
        ),
        ("matched", Node::Null),
    ]);
    let mut draft = Draft::new("report", dated.stored.clone(), node);
    add_dated(&mut draft.node, dated.period, None);
    let columns: Vec<Column> = COLUMNS
        .iter()
        .map(|(key, title, kind)| Column {
            key: (*key).to_owned(),
            title_id: (*title).to_owned(),
            kind: *kind,
        })
        .collect();
    draft.shown = Some(Shown::Table(AnswerTable {
        result_id: String::new(),
        title_id: title_id.to_owned(),
        columns,
        rows: shown_rows,
        count,
        totals_label_id: "report-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: statement.dr_total_paisa,
            },
            Cell::Money {
                paisa: statement.on_spot_total_paisa,
            },
            Cell::Money {
                paisa: statement.cr_total_paisa,
            },
            balance_cell(&statement.closing),
        ],
    }));
    draft.source = Some(AnswerSource::Report {
        result_id: String::new(),
        label_id: title_id.to_owned(),
        id: ReportId::R13,
        filters: SourceReportFilters {
            from: from.unwrap_or_default(),
            to: to.unwrap_or_default(),
            party: Some(SourceParty {
                record,
                person_key: None,
            }),
            ..SourceReportFilters::default()
        },
    });
    Ok(draft)
}
