//! R9 پختہ کھاتہ رپورٹ (the register of one-time parties, per person) (PRD 19.2, FR-13.5,
//! FR-13.6, FR-13.9, FR-19.10).
//!
//! One row for each line of a one-time buyer and each lot of a one-time seller, with its date,
//! بکری number, side, person, phone, item, نگ, amount, commission and whether it is paid or
//! still open. It reads the same lines as the register of the one-time screen
//! (`services::bikri::pukhta::register`, plan 13) and works out paid or open the same way, so the
//! screen and the report can never disagree; a test in `services/tests` compares them.
//!
//! - A buyer line: the open amount of its bill is its debit line less the active rows of
//!   `bill_allocations` (PRD FR-15.4). A نقدی line is pinned to its own cash, so it is paid at
//!   once. The bill lines of one posting and one person line up with his sale lines in the order
//!   of saving (the entry builder writes the buyer debits in lot and line order); if they do not
//!   line up the report fails loudly, it never shows a wrong open amount.
//! - A seller lot: what is still to pay him is worked out when shown (PRD FR-7.59, nothing is
//!   stored): each lot's صافی بکری when it is above 0 (the نقد is already in it), then what is set
//!   against his lots oldest first with the same `match_bills`: his later payments and the debit
//!   of a lot whose net is below 0.
//!
//! A cancelled line is listed as cancelled, is neither paid nor open, and is left out of the
//! totals. The totals (نگ, amount, commission, open) are checked sums over the whole filter.
//!
//! Example (PRD 4.5): شاہد buys 10 × 1,200 for cash: 12,800, commission 720, paid, nothing open.

use std::collections::{BTreeMap, HashMap};

use mandi_core::bikri::PukhtaSide;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_db::bikri::saved::{self as repo, BillLine, RegisterLine};
use mandi_db::matching as matching_repo;
use mandi_db::rusqlite::Connection;

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{
    add, col, count_of, load_parties, names_cell, paisa, party_of_line, pukhta_party, text_cell,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, PaidFilter, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};
use crate::reports::workings::{Open, match_bills};

/// A payment against a one-time seller's lots, with where it stands in the order of the key.
struct Payment {
    entry_date: String,
    entry_id: i64,
    open: Open,
}

/// What is open on each seller lot (see the module text), by lot id.
fn seller_open(
    conn: &Connection,
    lines: &[RegisterLine],
) -> Result<HashMap<i64, Paisa>, ReportsError> {
    let mut accounts: Vec<i64> = lines.iter().map(|l| l.account_id).collect();
    accounts.sort_unstable();
    accounts.dedup();
    let mut open_of: HashMap<i64, Paisa> = HashMap::new();
    for account_id in accounts {
        let live = matching_repo::live_lines(conn, account_id)?;
        // Each person's lots, and what is set against them: his later payments and the debit of
        // a lot whose net is below 0 (he owes the firm what the lot paid him too much).
        let mut people: BTreeMap<&str, (Vec<Open>, Vec<Payment>)> = BTreeMap::new();
        for line in lines.iter().filter(|l| l.account_id == account_id) {
            let net = paisa(line.seller_net_paisa)?;
            let entry = people.entry(line.person_key.as_str()).or_default();
            if line.cancelled {
                entry.0.push(Open {
                    line_id: line.row_id,
                    open: Paisa::ZERO,
                });
            } else if net.is_negative() {
                entry.0.push(Open {
                    line_id: line.row_id,
                    open: Paisa::ZERO,
                });
                entry.1.push(Payment {
                    entry_date: line.entry_date.clone(),
                    entry_id: line.journal_entry_id,
                    open: Open {
                        line_id: line.row_id,
                        open: net.checked_neg()?,
                    },
                });
            } else {
                entry.0.push(Open {
                    line_id: line.row_id,
                    open: net,
                });
            }
        }
        for line in &live {
            let Some(key) = line.person_key.as_deref() else {
                continue;
            };
            let paid_later = line.dr_paisa > 0
                && !line.is_advance
                && !matches!(line.posting_type.as_str(), "bikri_first" | "bikri_later");
            if paid_later && let Some(entry) = people.get_mut(key) {
                entry.1.push(Payment {
                    entry_date: line.entry_date.clone(),
                    entry_id: line.entry_id,
                    open: Open {
                        line_id: line.line_id,
                        open: paisa(line.dr_paisa)?,
                    },
                });
            }
        }
        for (lots, payments) in people.values_mut() {
            payments.sort_by(|a, b| {
                (a.entry_date.as_str(), a.entry_id, a.open.line_id).cmp(&(
                    b.entry_date.as_str(),
                    b.entry_id,
                    b.open.line_id,
                ))
            });
            let ordered: Vec<Open> = payments.iter().map(|p| p.open).collect();
            let rows = match_bills(lots, &ordered)?;
            for lot in lots.iter() {
                let used = rows
                    .iter()
                    .filter(|r| r.bill_line_id == lot.line_id)
                    .try_fold(Paisa::ZERO, |sum, r| sum.checked_add(r.amount))?;
                open_of.insert(lot.line_id, lot.open.checked_sub(used)?);
            }
        }
    }
    Ok(open_of)
}

/// What is open on each buyer line, by sale id: the debit line less its active matching rows.
fn buyer_open(
    bills: &[BillLine],
    lines: &[RegisterLine],
) -> Result<HashMap<i64, Paisa>, ReportsError> {
    // The bill lines of one posting and person, in line order, line up with its sale lines in
    // order of saving (the builder writes the buyer debits in lot and line order).
    let mut by_group: BTreeMap<(i64, i64, &str), Vec<&BillLine>> = BTreeMap::new();
    for bill in bills {
        by_group
            .entry((bill.entry_id, bill.account_id, bill.person_key.as_str()))
            .or_default()
            .push(bill);
    }
    let mut sales_by_group: BTreeMap<(i64, i64, &str), Vec<&RegisterLine>> = BTreeMap::new();
    for line in lines {
        sales_by_group
            .entry((
                line.journal_entry_id,
                line.account_id,
                line.person_key.as_str(),
            ))
            .or_default()
            .push(line);
    }
    let mut out = HashMap::new();
    for (group, sales) in sales_by_group {
        let bills = by_group.get(&group).map_or(&[][..], Vec::as_slice);
        if bills.len() != sales.len() {
            return Err(ReportsError::Data(
                "the sale lines of a one-time buyer and his bill lines do not line up".to_owned(),
            ));
        }
        for (sale, bill) in sales.iter().zip(bills) {
            let open = paisa(bill.amount_paisa)?.checked_sub(paisa(bill.matched_paisa)?)?;
            out.insert(sale.row_id, open);
        }
    }
    Ok(out)
}

/// One line of the register with its state.
struct Entry {
    line: RegisterLine,
    open: Paisa,
}

impl Entry {
    fn paid(&self) -> bool {
        !self.line.cancelled && self.open == Paisa::ZERO
    }
}

/// The lines of the register, oldest first, with what is open on each. Every line of a person is
/// read, not only those in the date range, because what is open on a lot depends on all of them.
fn load(conn: &Connection, typed: &Typed) -> Result<Vec<Entry>, ReportsError> {
    let from = typed.from.as_deref();
    let to = typed.to.as_deref();
    let mut rows: Vec<Entry> = Vec::new();

    let lines = repo::register_buyer_lines(conn, from, to)?;
    let bills = repo::pukhta_bill_lines(conn)?;
    let open = buyer_open(&bills, &lines)?;
    for line in lines {
        let left = open.get(&line.row_id).copied().unwrap_or(Paisa::ZERO);
        rows.push(Entry { line, open: left });
    }

    let lines = repo::register_seller_lines(conn, from, to)?;
    // The open amounts count every lot of the person, not only those in the range.
    let all = repo::register_seller_lines(conn, None, None)?;
    let open = seller_open(conn, &all)?;
    for line in lines {
        let left = open.get(&line.row_id).copied().unwrap_or(Paisa::ZERO);
        rows.push(Entry { line, open: left });
    }

    rows.sort_by(|a, b| {
        (
            a.line.entry_date.as_str(),
            a.line.bikri_number,
            a.line.row_id,
        )
            .cmp(&(
                b.line.entry_date.as_str(),
                b.line.bikri_number,
                b.line.row_id,
            ))
    });
    for entry in &mut rows {
        if entry.line.cancelled {
            entry.open = Paisa::ZERO;
        }
    }
    Ok(rows)
}

/// Builds R9.
///
/// # Errors
/// A database error; `ReportsError::Money`; `ReportsError::Data` if the bill lines of a person
/// do not line up with his sale lines; `ReportsError::Rejected` for a party that is not a
/// پختہ کھاتہ account.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    if let Some(account_id) = typed.party_id {
        let pukhta: i64 = conn.query_row(
            "SELECT is_pukhta_khata FROM accounts WHERE id = ?1",
            [account_id],
            |r| r.get(0),
        )?;
        if pukhta != 1 {
            return Err(ReportsError::Rejected(vec![FieldError::new(
                ErrorCode::ErrOutOfRange,
                "filters.party",
            )]));
        }
    }
    let entries = load(conn, typed)?;
    let kept: Vec<&Entry> = entries
        .iter()
        .filter(|e| typed.party_id.is_none_or(|id| e.line.account_id == id))
        .filter(|e| {
            typed
                .person_key
                .as_deref()
                .is_none_or(|key| e.line.person_key == key)
        })
        .filter(|e| match typed.status {
            None => true,
            Some(PaidFilter::Paid) => e.paid(),
            Some(PaidFilter::Due) => !e.line.cancelled && !e.paid(),
        })
        .collect();

    let (mut qty, mut amount, mut commission, mut open) =
        (0_i64, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    for entry in kept.iter().filter(|e| !e.line.cancelled) {
        qty = qty
            .checked_add(entry.line.qty)
            .ok_or_else(|| ReportsError::Data("the quantity total does not fit".to_owned()))?;
        add(&mut amount, entry.line.amount)?;
        add(&mut commission, entry.line.commission)?;
        add(&mut open, entry.open)?;
    }

    let page: Vec<&Entry> = spec.slice(&kept);
    let account_ids: Vec<i64> = page.iter().map(|e| e.line.account_id).collect();
    let parties = load_parties(conn, &account_ids)?;
    let mut rows = Vec::with_capacity(page.len());
    for entry in page {
        let line = &entry.line;
        let party = pukhta_party(
            party_of_line(&parties, line.account_id, None)?,
            Some(line.person_key.clone()),
            Some(&line.name_ur),
            Some(&line.name_en),
        );
        let state = if line.cancelled {
            ids::STATUS_CANCELLED
        } else if entry.paid() {
            ids::STATUS_PAID
        } else {
            ids::STATUS_DUE
        };
        rows.push(Row {
            cells: vec![
                Cell::Date {
                    iso: line.entry_date.clone(),
                },
                Cell::Doc {
                    type_id: ids::doc_id("bikri_first"),
                    number: Some(line.bikri_number),
                },
                Cell::Label {
                    id: match line.side {
                        PukhtaSide::Buyer => ids::SIDE_BUYER,
                        PukhtaSide::Seller => ids::SIDE_SELLER,
                    }
                    .to_owned(),
                },
                Cell::Party { party },
                text_cell(Some(line.phone.clone())),
                names_cell(
                    Some(line.item.name_ur.clone()),
                    Some(line.item.name_en.clone()),
                ),
                Cell::Number { value: line.qty },
                Cell::Money { paisa: line.amount },
                Cell::Money {
                    paisa: line.commission,
                },
                Cell::Label {
                    id: state.to_owned(),
                },
                if entry.open == Paisa::ZERO {
                    Cell::Empty
                } else {
                    Cell::Money { paisa: entry.open }
                },
            ],
            source: Some(RowSource {
                kind: SourceKind::Bikri,
                id: line.bikri_id,
            }),
            kind: RowKind::Normal,
        });
    }

    Ok(ReportPage {
        id: ReportId::R9,
        title_id: ids::title_id("R9"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("date", ids::COL_DATE, ColumnKind::Date),
            col("doc", ids::COL_DOC, ColumnKind::Doc),
            col("side", ids::COL_SIDE, ColumnKind::Label),
            col("person", ids::COL_PERSON, ColumnKind::Party),
            col("phone", ids::COL_PHONE, ColumnKind::Text),
            col("item", ids::COL_ITEM, ColumnKind::Names),
            col("qty", ids::COL_QTY, ColumnKind::Number),
            col("amount", ids::COL_AMOUNT, ColumnKind::Money),
            col("commission", ids::COL_COMMISSION, ColumnKind::Money),
            col("status", ids::COL_STATUS, ColumnKind::Label),
            col("open", ids::COL_OPEN, ColumnKind::Money),
        ],
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(kept.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Number { value: qty },
            Cell::Money { paisa: amount },
            Cell::Money { paisa: commission },
            Cell::Empty,
            Cell::Money { paisa: open },
        ],
        summary: Vec::new(),
    })
}
