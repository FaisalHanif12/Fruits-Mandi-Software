//! The روکڑ بک (PRD 16.1, FR-16.14 v1.60, FR-19.11, plan 17, plan 99): every cash in and out from all
//! modules, one page at a time, with the cards (کل سرمایہ, شروع میں روکڑ, آخر میں روکڑ), the totals
//! and the in and out of each type over the whole range. There is no cash count (PRD FR-16.1): the
//! book only shows what was posted.
//!
//! A row is one journal entry that moves cash, or, for a بکری posting, what cash received from the
//! buyers or what it gave the sellers. By default only the entries in force are rows (an edited entry
//! shows once; a cancelled one not at all); `show_reversed` adds the reversed ones and their الٹا
//! اندراج. Totals, summary and cards are over the entries in force, so the switch never changes them.
//!
//! The running cash is the opening cash plus the sums of the rows up to it, over the whole range, so a
//! page far down has the right figure. The sums are `SUM()` on INTEGER columns; the differences are
//! worked out here with checked maths.

use std::collections::HashMap;

use mandi_db::accounts as account_repo;
use mandi_db::cashbook as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::cashbook::{
    CashGroup, CashGroupSummary, CashKind, CashRefs, CashRowStatus, Cashbook, CashbookInput,
    CashbookRow, CashbookTotals, MAX_ROW_PARTIES,
};
use mandi_domain::core::codes::{CodeType, number_code};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::paging::SortDirection;
use mandi_domain::core::row_source::RowSource;
use mandi_domain::core::validate::{self, FieldErrors};

use crate::cash::CASH_ACCOUNT_CODE;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::page;

/// The controls whose lines are the partners' side of linked money (PRD FR-16.6, FR-16.13): never
/// the party of a cash row.
const LINKED_CONTROLS: [&str; 2] = ["0605", "0307"];

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// In less out, signed.
fn net(in_paisa: i64, out_paisa: i64) -> Result<Paisa, ServiceError> {
    paisa(in_paisa)?
        .checked_sub(paisa(out_paisa)?)
        .map_err(money_error)
}

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// A date bound of the range: empty is open.
fn bound(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&str>,
    today: BusinessDate,
) -> Option<BusinessDate> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors.take(validate::date(field, text, today))
}

/// The order of the sort asked for.
fn order_of(input: &CashbookInput) -> Result<repo::Order, ServiceError> {
    let Some(sort) = &input.sort else {
        return Ok(repo::Order::Date { desc: false });
    };
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    let chosen = sort
        .order_by(&repo::SORT_COLUMNS)
        .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    Ok(match sort.column.as_str() {
        "entry_date" => repo::Order::Date {
            desc: sort.direction == SortDirection::Desc,
        },
        "in_paisa" | "out_paisa" => repo::Order::Amount {
            out: sort.column == "out_paisa",
            desc: sort.direction == SortDirection::Desc,
        },
        _ => repo::Order::Other(chosen),
    })
}

/// One page of the cash book with the cards, the totals and the summary over the whole range (PRD
/// FR-16.14, FR-19.11). Without `from` the book starts at the first entry and the opening is 0. Read
/// only.
///
/// # Errors
/// The date codes on `from` and `to`; the sort codes; `ServiceError::Db`.
pub fn get(ctx: &Context, input: &CashbookInput) -> Result<Cashbook, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let from = bound(&mut errors, "from", input.from.as_deref(), today);
    let to = bound(&mut errors, "to", input.to.as_deref(), today);
    errors.finish()?;
    let (from_iso, to_iso) = (from.map(BusinessDate::to_iso), to.map(BusinessDate::to_iso));
    let order = order_of(input)?;
    let in_date_order = matches!(order, repo::Order::Date { .. });
    let kinds: Vec<&str> = input.kinds.iter().flatten().map(|k| k.as_str()).collect();
    ctx.db.read_snapshot(|conn| {
        let cash = account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
            .ok_or_else(|| data("the cash account is missing"))?;
        let (from, to) = (from_iso.as_deref(), to_iso.as_deref());
        let filter = repo::Filter {
            cash_id: cash,
            from,
            to,
            show_reversed: input.show_reversed,
            kinds: &kinds,
            split: true,
        };
        let live_filter = repo::Filter {
            show_reversed: false,
            ..filter
        };
        let sums = repo::range_sums(conn, cash, from, to)?;
        let opening = opening_cash(conn, cash, from)?;
        // A reversed pair adds the same to cash in and out, so the net of every entry is the net of
        // the entries in force.
        let closing = opening
            .checked_add(net(sums.live.in_paisa, sums.live.out_paisa)?)
            .map_err(money_error)?;
        // The rows shown, and what the totals are over: the entries in force of the same filter.
        let (set, totals) = if kinds.is_empty() {
            let set = if input.show_reversed {
                sums.all
            } else {
                sums.live
            };
            (set, sums.live)
        } else {
            let set = repo::filtered_sums(conn, &filter)?;
            let totals = if input.show_reversed {
                repo::filtered_sums(conn, &live_filter)?
            } else {
                set
            };
            (set, totals)
        };
        let keys = repo::page(
            conn,
            &filter,
            set,
            &order,
            i64::from(input.offset),
            page::limit_of(input.limit),
        )?;
        let rows = rows_of(conn, cash, &keys, opening)?;
        // Month headings with their subtotals, for the months this page touches (plan 38 B2).
        let months = match (
            input.group_by_month && in_date_order,
            rows.first(),
            rows.last(),
        ) {
            (true, Some(first), Some(last)) => {
                let newest_first = matches!(order, repo::Order::Date { desc: true });
                page::month_subtotals(
                    repo::months_between(conn, &filter, (&first.entry_date, &last.entry_date))?,
                    &[("in_paisa", true), ("out_paisa", true)],
                    newest_first,
                )
            }
            _ => Vec::new(),
        };
        let (capital_dr, capital_cr) = repo::capital_sums(conn, to)?;
        Ok(Cashbook {
            total_capital_paisa: net(capital_cr, capital_dr)?,
            opening_paisa: opening,
            closing_paisa: closing,
            first_entry_date: repo::first_entry_date(conn, cash)?,
            rows,
            total_count: set.rows,
            totals: CashbookTotals {
                in_paisa: paisa(totals.in_paisa)?,
                out_paisa: paisa(totals.out_paisa)?,
            },
            summary: summary_of(&repo::sums_by_kind(
                conn,
                &repo::Filter {
                    kinds: &[],
                    ..live_filter
                },
            )?)?,
            months,
        })
    })
}

/// Cash in hand at the end of the day before the first day of the range: the whole book less what
/// is dated from the first day on (both are kept sums, so a page after the first does not read the
/// book again). 0 without a first day.
fn opening_cash(conn: &Connection, cash: i64, from: Option<&str>) -> Result<Paisa, ServiceError> {
    let Some(first) = from else {
        return Ok(Paisa::ZERO);
    };
    let whole = repo::range(conn, cash, None, None)?;
    let from_first = repo::range(conn, cash, Some(first), None)?;
    net(whole.in_paisa, whole.out_paisa)?
        .checked_sub(net(from_first.in_paisa, from_first.out_paisa)?)
        .map_err(money_error)
}

/// In and out of every group of the type filter, in filter order (PRD FR-16.14).
fn summary_of(by_kind: &[(String, repo::RowSums)]) -> Result<Vec<CashGroupSummary>, ServiceError> {
    CashGroup::ALL
        .into_iter()
        .map(|group| {
            let (mut count, mut cash_in, mut cash_out) = (0_i64, Paisa::ZERO, Paisa::ZERO);
            for (kind, sums) in by_kind {
                let kind =
                    CashKind::parse(kind).ok_or_else(|| data("an unknown kind of cash row"))?;
                if kind.group() != group {
                    continue;
                }
                count = count
                    .checked_add(sums.rows)
                    .ok_or_else(|| data("the rows of the cash book do not fit"))?;
                cash_in = cash_in
                    .checked_add(paisa(sums.in_paisa)?)
                    .map_err(money_error)?;
                cash_out = cash_out
                    .checked_add(paisa(sums.out_paisa)?)
                    .map_err(money_error)?;
            }
            Ok(CashGroupSummary {
                group,
                kinds: CashKind::ALL
                    .into_iter()
                    .filter(|k| k.group() == group)
                    .collect(),
                count,
                in_paisa: cash_in,
                out_paisa: cash_out,
            })
        })
        .collect()
}

/// The rows of a page, in the order of its keys, with their names and running cash.
fn rows_of(
    conn: &Connection,
    cash: i64,
    keys: &[repo::PageKey],
    opening: Paisa,
) -> Result<Vec<CashbookRow>, ServiceError> {
    let row_keys: Vec<repo::RowKey> = keys.iter().map(|k| k.key).collect();
    let facts = repo::rows(conn, cash, &row_keys)?;
    let mut entry_ids: Vec<i64> = row_keys.iter().map(|k| k.entry_id).collect();
    entry_ids.sort_unstable();
    entry_ids.dedup();
    let lines = repo::entry_lines(conn, &entry_ids)?;
    let none = Vec::new();
    keys.iter()
        .map(|key| {
            let row = facts
                .get(&key.key)
                .ok_or_else(|| data("a row of the cash book is missing"))?;
            let kind =
                CashKind::parse(&row.kind).ok_or_else(|| data("an unknown kind of cash row"))?;
            let running = match (key.run_in_paisa, key.run_out_paisa) {
                (Some(run_in), Some(run_out)) => Some(
                    opening
                        .checked_add(net(run_in, run_out)?)
                        .map_err(money_error)?,
                ),
                _ => None,
            };
            Ok(CashbookRow {
                entry_id: key.key.entry_id,
                entry_date: row.entry_date.clone(),
                // The پیشگی typed on a lot is its own entry with a بکری as its source (PRD FR-12.52
                // v1.57): its row says `bikri_advance` and carries the number of the SALE, so it is
                // never read as the number of a پیشگی voucher.
                posting_type: if row.posting_type == "advance" && row.source_type == "bikri" {
                    "bikri_advance".to_owned()
                } else {
                    row.posting_type.clone()
                },
                kind,
                group: kind.group(),
                status: match row.status.as_str() {
                    "reversed" => CashRowStatus::Reversed,
                    "reversal" => CashRowStatus::Reversal,
                    _ => CashRowStatus::Live,
                },
                number: row.number,
                refs: refs_of(
                    cash,
                    kind,
                    row,
                    lines.get(&key.key.entry_id).unwrap_or(&none),
                ),
                narration: if row.narration.is_empty() {
                    row.reason.clone()
                } else {
                    row.narration.clone()
                },
                in_paisa: paisa(row.in_paisa)?,
                out_paisa: paisa(row.out_paisa)?,
                running_paisa: running,
                source: Some(source_of(row)),
            })
        })
        .collect()
}

fn record(named: &repo::Named) -> RecordRef {
    RecordRef {
        id: named.id,
        code: named.code.clone(),
        name_ur: named.name_ur.clone(),
        name_en: named.name_en.clone(),
        tag: named.tag.clone(),
    }
}

/// The records a row names (PRD FR-16.14): read from the lines of its entry, so an old version of an
/// edited entry names its own parties.
fn refs_of(cash: i64, kind: CashKind, row: &repo::CashRow, lines: &[repo::EntryLine]) -> CashRefs {
    let mut refs = CashRefs::default();
    if let Some(venture) = &row.venture {
        let (code_type, slot) = if venture.kind == "garden" {
            (CodeType::Garden, &mut refs.garden)
        } else {
            (CodeType::Trip, &mut refs.trip)
        };
        *slot = Some(RecordRef {
            id: venture.id,
            code: number_code(code_type, venture.number),
            name_ur: venture.name_ur.clone(),
            name_en: venture.name_en.clone(),
            tag: None,
        });
        return refs;
    }
    let others = || {
        lines.iter().filter(move |l| {
            l.account.id != cash && !LINKED_CONTROLS.contains(&l.control_code.as_str())
        })
    };
    match kind {
        CashKind::OpeningCash => {}
        CashKind::OpeningCapital
        | CashKind::CapitalIn
        | CashKind::CapitalOut
        | CashKind::PersonalExpense
        | CashKind::PersonalReturn => {
            refs.partner = others().find_map(|l| l.partner.as_ref().map(record));
        }
        CashKind::ShopExpense => {
            refs.expense_head = others().find_map(|l| l.expense_head.as_ref().map(record));
        }
        CashKind::BikriPaid | CashKind::BikriSellerCash => {
            // The cash part of a بکری posts each نقد as seller then cash, and each payment on the
            // spot as cash then buyer (PRD FR-12.9); a reversal keeps the order of the lines.
            let buyers = kind == CashKind::BikriPaid;
            let by_line: HashMap<i64, &repo::EntryLine> =
                lines.iter().map(|l| (l.line_no, l)).collect();
            let found = lines
                .iter()
                .filter(|l| l.account.id == cash)
                // Part 1 is cash in, part 2 cash out: only the cash lines of this row's side.
                .filter(|l| (l.dr_paisa > 0) == (row.key.part == 1))
                .filter_map(|l| {
                    let beside = if buyers {
                        l.line_no.checked_add(1)
                    } else {
                        l.line_no.checked_sub(1)
                    };
                    beside.and_then(|n| by_line.get(&n).copied())
                });
            set_parties(&mut refs, found);
        }
        _ => set_parties(&mut refs, others()),
    }
    refs
}

/// The distinct parties of some lines, in their order: at most [`MAX_ROW_PARTIES`] names and the
/// count of all. Two چکتا buyers on the account 0030 are two parties.
fn set_parties<'a>(refs: &mut CashRefs, lines: impl Iterator<Item = &'a repo::EntryLine>) {
    let mut seen: Vec<(i64, &str)> = Vec::new();
    for line in lines {
        let key = (line.account.id, line.account.name_ur.as_str());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        if refs.parties.len() < MAX_ROW_PARTIES {
            refs.parties.push(record(&line.account));
        }
    }
    refs.party_count = i64::try_from(seen.len()).unwrap_or(i64::MAX);
}

/// The record a row opens (PRD FR-19.4). An entry of a trip or a garden opens the trip or the
/// garden itself: its stored source id is the id of one of its entries.
fn source_of(row: &repo::CashRow) -> RowSource {
    RowSource::of_stored(
        &row.source_type,
        row.source_id,
        row.key.entry_id,
        row.venture_kind.as_deref(),
        row.venture_id,
    )
}
