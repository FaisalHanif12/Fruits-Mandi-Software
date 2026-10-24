//! R35 پارٹی کی روزانہ پرچی (the party day sheet) (PRD 19.2, FR-19.12, FR-19.10, E2E-43).
//!
//! The collector takes a slip for each party to the shops. For one day (the "as of" day, today
//! when none is given) the report has two layouts:
//!
//! * **All parties, one table** (no party picked). Two tables on one page, buyers first and
//!   sellers second, each with a heading row that carries its totals. One row for each party
//!   who has a balance or an entry that day, sorted by his Urdu name: the short code, the party
//!   (name with tag), سابقہ (the balance at the start of the day), بنام and جمع (everything
//!   debited and credited that day), بقایا (the balance at the end), and an empty column for
//!   the collector's note. Every row, every heading and the total row hold
//!   `سابقہ + بنام − جمع = بقایا` (signed: a debit balance is positive).
//! * **One party's slip** (a party picked). A buyer's slip lists the lines of the day (تعداد,
//!   قسم مال, ریٹ and the amount with commission and لاگا, C5) and, below, the lines of PRD
//!   FR-19.12: total نگ, previous balance, what he was charged that day, what he paid on the
//!   spot (the cash part of a نقدی line), what he paid that day (وصولی), and the balance that
//!   remains. A seller's slip lists the credit lines of his بکری that day with their lot's item
//!   and quantity, and below: previous, sold, نقد given on the spot, paid that day, remaining.
//!
//! The figures are the posted ledger, the same sums as R20, R13 and the trial balance: the
//! balance at the start of the day is every posted line dated before it, and the debits and
//! credits of the day are the posted lines dated that day. The two entries of a same-day cancel
//! or edit (the posting and its reversal) are both left out of the day's columns, because they
//! cancel each other and a cancelled bill is not what the party bought. The balances do not
//! change by this, so the identity above always holds. Where a slip names a charge or a payment
//! it is worked out from the stored sale lines and the ledger; nothing is calculated again with
//! today's rules (Code Standards B20). A month close writes no line on a buyer or a seller.
//!
//! Example (E2E-43, 10-10-26): اکرم previous 0, bought 12,880 (کیلا 20 پیٹی @ 600), paid that day
//! 10,000, remaining 2,880 بنام; the buyers' remaining total is 21,564, the total of R20.

use std::collections::HashMap;

use mandi_core::accounts::Balance;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_core::text::UrduCollator;
use mandi_db::rusqlite::{Connection, params};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{
    add, balance_cell, col, count_of, load_parties, names_cell, paisa, pukhta_party, text_cell,
};
use crate::reports::filters::Typed;
use crate::reports::grouped_sums::pukhta_names;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, PartyRef, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
    SummaryLine,
};

/// The control codes of the two kinds of party the sheet is for (PRD FR-10.2).
const BUYER_CONTROL: &str = "0304";
const SELLER_CONTROL: &str = "03011";

/// An entry that is one half of a same-day cancel or edit: the posting dated the same day as its
/// reversal, or the reversal itself. The two cancel each other.
const SAME_DAY_PAIR: &str = "(e.reverses_entry_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM journal_entries o
        WHERE o.id = e.reverses_entry_id AND o.entry_date = e.entry_date))
    OR EXISTS (SELECT 1 FROM journal_entries r
        WHERE r.reverses_entry_id = e.id AND r.status = 'posted' AND r.entry_date = e.entry_date)";

/// What one party (a buyer or seller account, or a person inside a پختہ کھاتہ account) has on the
/// day.
#[derive(Debug, Clone)]
struct Activity {
    account_id: i64,
    person_key: Option<String>,
    buyer: bool,
    prev_dr: Paisa,
    prev_cr: Paisa,
    day_dr: Paisa,
    day_cr: Paisa,
    /// Every posted line up to and including the day: the true ledger balance.
    end_dr: Paisa,
    end_cr: Paisa,
}

impl Activity {
    /// The balance at the end of the day, signed (a debit balance is positive): the posted lines
    /// up to the day, the same sums as the trial balance and R20.
    fn end_net(&self) -> Result<Paisa, ReportsError> {
        Ok(self.end_dr.checked_sub(self.end_cr)?)
    }

    /// `previous + debit − credit = remaining` must hold; if it does not, the reversal rules
    /// have changed under this report and its numbers cannot be trusted.
    fn check(&self) -> Result<(), ReportsError> {
        let by_day = self
            .prev_net()?
            .checked_add(self.day_dr)?
            .checked_sub(self.day_cr)?;
        if by_day != self.end_net()? {
            return Err(ReportsError::Data(
                "the day of a party does not add up to his ledger balance".to_owned(),
            ));
        }
        Ok(())
    }

    fn prev_net(&self) -> Result<Paisa, ReportsError> {
        Ok(self.prev_dr.checked_sub(self.prev_cr)?)
    }

    /// True when the party has a balance or an entry that day.
    fn worth_a_row(&self) -> Result<bool, ReportsError> {
        Ok(self.prev_net()? != Paisa::ZERO
            || self.day_dr != Paisa::ZERO
            || self.day_cr != Paisa::ZERO
            || self.end_net()? != Paisa::ZERO)
    }
}

/// The parties' sums for the day, optionally for one account and one person. One grouped query
/// over the posted lines of the buyer and seller accounts up to the day (Code Standards B22).
fn activities(
    conn: &Connection,
    day: &str,
    account: Option<i64>,
    person_key: Option<&str>,
) -> Result<Vec<Activity>, ReportsError> {
    let sql = format!(
        "SELECT l.account_id, l.pukhta_person_key, k.code,
                COALESCE(SUM(CASE WHEN e.entry_date < ?1 THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date < ?1 THEN l.cr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date = ?1 AND NOT ({SAME_DAY_PAIR})
                                  THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date = ?1 AND NOT ({SAME_DAY_PAIR})
                                  THEN l.cr_paisa END), 0),
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ('{BUYER_CONTROL}', '{SELLER_CONTROL}') AND e.status = 'posted'
           AND e.entry_date <= ?1
           AND (?2 IS NULL OR l.account_id = ?2)
           AND (?3 IS NULL OR l.pukhta_person_key = ?3)
         GROUP BY l.account_id, l.pukhta_person_key, k.code
         ORDER BY l.account_id, l.pukhta_person_key"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params![day, account, person_key], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (account_id, person_key, control, prev_dr, prev_cr, day_dr, day_cr, end_dr, end_cr) in rows
    {
        let item = Activity {
            account_id,
            person_key,
            buyer: control == BUYER_CONTROL,
            prev_dr: paisa(prev_dr)?,
            prev_cr: paisa(prev_cr)?,
            day_dr: paisa(day_dr)?,
            day_cr: paisa(day_cr)?,
            end_dr: paisa(end_dr)?,
            end_cr: paisa(end_cr)?,
        };
        item.check()?;
        out.push(item);
    }
    Ok(out)
}

/// The parties of the activities as the report names them: the account's names with its tag, and
/// a one-time person with his own names and key.
fn parties_of(
    conn: &Connection,
    items: &[Activity],
) -> Result<HashMap<(i64, Option<String>), PartyRef>, ReportsError> {
    let ids: Vec<i64> = items.iter().map(|a| a.account_id).collect();
    let accounts = load_parties(conn, &ids)?;
    let pukhta_ids: Vec<i64> = items
        .iter()
        .filter(|a| a.person_key.is_some())
        .map(|a| a.account_id)
        .collect();
    let people = pukhta_names(conn, &pukhta_ids)?;
    let mut out = HashMap::new();
    for item in items {
        let base = accounts
            .get(&item.account_id)
            .cloned()
            .ok_or_else(|| ReportsError::Data(format!("account {} is missing", item.account_id)))?;
        let party = match &item.person_key {
            Some(key) => {
                let person = people.get(key);
                pukhta_party(
                    base,
                    Some(key.clone()),
                    person.map(|p| p.name_ur.as_str()),
                    person.map(|p| p.name_en.as_str()),
                )
            }
            None => base,
        };
        out.insert((item.account_id, item.person_key.clone()), party);
    }
    Ok(out)
}

/// The signed total of a side as a balance cell.
fn net_cell(net: Paisa) -> Result<Cell, ReportsError> {
    Ok(balance_cell(Balance::from_net(net)?))
}

/// The columns of the table of all parties.
fn table_columns() -> Vec<crate::reports::model::Column> {
    vec![
        col("code", ids::COL_CODE, ColumnKind::Text),
        col("party", ids::COL_ACCOUNT, ColumnKind::Party),
        col("previous", ids::COL_PREVIOUS, ColumnKind::Balance),
        col("debit", ids::COL_DEBIT, ColumnKind::Money),
        col("credit", ids::COL_CREDIT, ColumnKind::Money),
        col("remaining", ids::COL_BALANCE, ColumnKind::Balance),
        col("collector", ids::COL_COLLECTOR, ColumnKind::Text),
    ]
}

/// The sums of a section (or of the whole table).
#[derive(Debug, Clone, Copy, Default)]
struct Sums {
    prev_net: Paisa,
    day_dr: Paisa,
    day_cr: Paisa,
    end_net: Paisa,
}

impl Sums {
    fn add(&mut self, item: &Activity) -> Result<(), ReportsError> {
        self.prev_net = self.prev_net.checked_add(item.prev_net()?)?;
        add(&mut self.day_dr, item.day_dr)?;
        add(&mut self.day_cr, item.day_cr)?;
        self.end_net = self.end_net.checked_add(item.end_net()?)?;
        Ok(())
    }

    fn cells(&self, lead: Cell) -> Result<Vec<Cell>, ReportsError> {
        Ok(vec![
            lead,
            Cell::Empty,
            net_cell(self.prev_net)?,
            Cell::Money { paisa: self.day_dr },
            Cell::Money { paisa: self.day_cr },
            net_cell(self.end_net)?,
            Cell::Empty,
        ])
    }
}

/// One party row of the table.
fn party_row(item: &Activity, party: &PartyRef) -> Result<Row, ReportsError> {
    let code = party
        .short_code
        .clone()
        .or_else(|| Some(party.code.clone()));
    Ok(Row {
        cells: vec![
            text_cell(code),
            Cell::Party {
                party: party.clone(),
            },
            net_cell(item.prev_net()?)?,
            Cell::Money { paisa: item.day_dr },
            Cell::Money { paisa: item.day_cr },
            net_cell(item.end_net()?)?,
            Cell::Empty,
        ],
        source: Some(RowSource {
            kind: SourceKind::Account,
            id: item.account_id,
        }),
        kind: RowKind::Normal,
    })
}

/// The all-parties layout.
fn table(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    day: &str,
) -> Result<ReportPage, ReportsError> {
    let mut items = Vec::new();
    for item in activities(conn, day, None, None)? {
        if item.worth_a_row()? {
            items.push(item);
        }
    }
    let parties = parties_of(conn, &items)?;
    // Urdu alphabetical order of the names; on a tie the account and then the person.
    let collator = UrduCollator::new().ok();
    let sort_key = |name: &str| -> Vec<u8> {
        collator
            .as_ref()
            .map_or_else(|| name.as_bytes().to_vec(), |c| c.sort_key(name))
    };
    let mut keyed: Vec<(Vec<u8>, Activity)> = items
        .into_iter()
        .map(|item| {
            let name = parties
                .get(&(item.account_id, item.person_key.clone()))
                .map(|p| p.name_ur.clone())
                .unwrap_or_default();
            (sort_key(&name), item)
        })
        .collect();
    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.account_id.cmp(&b.1.account_id))
            .then(a.1.person_key.cmp(&b.1.person_key))
    });

    let mut rows = Vec::new();
    let mut grand = Sums::default();
    // The shop's total اگراہی (PRD FR-19.9): what the buyers owe, a buyer who is in credit not
    // taken off. R20 lists the same amounts, so this line always equals its میزان.
    let mut collection = Paisa::ZERO;
    for (buyers, label) in [(true, ids::R35_BUYERS), (false, ids::R35_SELLERS)] {
        let section: Vec<&Activity> = keyed
            .iter()
            .map(|(_, a)| a)
            .filter(|a| a.buyer == buyers)
            .collect();
        if section.is_empty() {
            continue;
        }
        let mut sums = Sums::default();
        let mut section_rows = Vec::with_capacity(section.len());
        for item in &section {
            sums.add(item)?;
            if buyers && item.end_net()? > Paisa::ZERO {
                add(&mut collection, item.end_net()?)?;
            }
            let party = parties
                .get(&(item.account_id, item.person_key.clone()))
                .ok_or_else(|| ReportsError::Data("a party of the sheet is missing".to_owned()))?;
            section_rows.push(party_row(item, party)?);
        }
        // The totals row is the buyers' table: what is owed to the shop. The sellers' total is
        // what the shop owes; the two are not added together (PRD E2E-43, money agent S1).
        if buyers {
            grand = sums;
        }
        rows.push(Row {
            cells: sums.cells(Cell::Label {
                id: label.to_owned(),
            })?,
            source: None,
            kind: RowKind::Group,
        });
        rows.extend(section_rows);
    }

    let mut resolved = typed.resolved.clone();
    resolved.as_of = Some(day.to_owned());
    Ok(ReportPage {
        id: ReportId::R35,
        title_id: ids::title_id("R35"),
        filters: resolved,
        columns: table_columns(),
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: grand.cells(Cell::Empty)?,
        summary: vec![line(ids::R35_COLLECTION, Cell::Money { paisa: collection })],
    })
}

/// A line of the footer.
fn line(label: &str, cell: Cell) -> SummaryLine {
    SummaryLine {
        label_id: label.to_owned(),
        cell,
    }
}

/// A money line that is left blank when it is 0 (the collector fills it in by hand).
fn blank_if_zero(amount: Paisa) -> Cell {
    if amount == Paisa::ZERO {
        Cell::Empty
    } else {
        Cell::Money { paisa: amount }
    }
}

/// The columns of one party's slip.
fn slip_columns() -> Vec<crate::reports::model::Column> {
    vec![
        col("qty", ids::COL_QTY, ColumnKind::Number),
        col("item", ids::COL_ITEM, ColumnKind::Names),
        col("rate", ids::COL_RATE, ColumnKind::Money),
        col("amount", ids::COL_AMOUNT, ColumnKind::Money),
    ]
}

/// The item of a line as the slip writes it: the packing and the item (کیلا in a پیٹی reads
/// "پیٹی کیلا" in Urdu), in both languages.
fn item_cell(packing_ur: &str, item_ur: &str, packing_en: &str, item_en: &str) -> Cell {
    names_cell(
        Some(format!("{packing_ur} {item_ur}")),
        Some(format!("{packing_en} {item_en}")),
    )
}

/// One line of a buyer's slip.
struct SaleLine {
    qty: i64,
    packing_ur: String,
    packing_en: String,
    item_ur: String,
    item_en: String,
    rate: Paisa,
    total: Paisa,
    cash: bool,
    bikri_id: i64,
}

/// The sale lines of the day of a buyer (and one-time person), in the order they were saved.
fn sale_lines(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
) -> Result<Vec<SaleLine>, ReportsError> {
    let mut statement = conn.prepare(
        "SELECT s.qty, pk.name_ur, pk.name_en, i.name_ur, i.name_en, s.buyer_rate_paisa,
                s.buyer_total_paisa, s.payment_type = 'cash', s.bikri_id
         FROM bikri_sales s
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN items i ON i.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         WHERE s.is_active = 1 AND s.entry_date = ?1 AND s.buyer_account_id = ?2
           AND (?3 IS NULL OR s.pukhta_person_key = ?3)
         ORDER BY s.bikri_id, s.line_no",
    )?;
    let rows = statement
        .query_map(params![day, account, person_key], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, bool>(7)?,
                r.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (qty, packing_ur, packing_en, item_ur, item_en, rate, total, cash, bikri_id) in rows {
        out.push(SaleLine {
            qty,
            packing_ur,
            packing_en,
            item_ur,
            item_en,
            rate: paisa(rate)?,
            total: paisa(total)?,
            cash,
            bikri_id,
        });
    }
    Ok(out)
}

/// A lot of a posting: the quantity sold, the names of its packing and item, and the sums that
/// make its credit (sales, their commission, the lot charges of the first posting).
type LotRow = (i64, (String, String, String, String), [i64; 3]);

/// One credit line of a seller's slip: the posting's line with the lot it belongs to, when the
/// lots of the posting can be matched to the lines one for one.
struct CreditLine {
    bikri_id: i64,
    amount: Paisa,
    qty: Option<i64>,
    names: Option<(String, String, String, String)>,
}

/// The seller's credit lines of بکری postings dated the day, with each lot's item and the quantity
/// sold in that posting. The lots of a posting that belong to the seller, in lot order, are
/// matched to his credit lines of the posting, in line order, only when there are as many lots
/// as lines; otherwise the item and quantity are left empty (nothing is guessed).
fn credit_lines(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
) -> Result<Vec<CreditLine>, ReportsError> {
    let sql = format!(
        "SELECT e.id, e.source_id, l.cr_paisa
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?2 AND (?3 IS NULL OR l.pukhta_person_key = ?3)
           AND e.entry_date = ?1 AND e.status = 'posted' AND l.cr_paisa > 0
           AND e.source_type = 'bikri' AND e.posting_type IN ('bikri_first', 'bikri_later')
           AND NOT ({SAME_DAY_PAIR})
         ORDER BY e.id, l.line_no"
    );
    let mut statement = conn.prepare(&sql)?;
    let lines = statement
        .query_map(params![day, account, person_key], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    // For each lot of the posting: the quantity sold in it, its item, and whether its credit is
    // more than 0 (a lot with a net of 0 or less has no credit line, PRD FR-12.7). The credit is
    // the sales less their commission, and in the first posting less the lot charges (C8).
    let mut lots_stmt = conn.prepare(
        "SELECT SUM(s.qty), pk.name_ur, pk.name_en, i.name_ur, i.name_en,
                SUM(s.seller_amount_paisa), SUM(s.seller_comm_paisa),
                CASE WHEN l.first_entry_id = ?1
                     THEN l.labour_paisa + l.munshiana_paisa + l.phone_paisa + l.store_rent_paisa
                          + CASE WHEN l.freight_paid_by = 'firm' THEN l.freight_paisa ELSE 0 END
                     ELSE 0 END
         FROM bikri_lots l
         JOIN bikri_sales s ON s.lot_id = l.id AND s.is_active = 1 AND s.journal_entry_id = ?1
         JOIN items i ON i.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         WHERE l.bikri_id = ?2 AND l.seller_account_id = ?3
           AND (?4 IS NULL OR l.pukhta_person_key = ?4)
         GROUP BY l.id ORDER BY l.lot_no",
    )?;
    let mut out = Vec::with_capacity(lines.len());
    let mut at = 0_usize;
    while at < lines.len() {
        let Some(&(entry_id, bikri_id, _)) = lines.get(at) else {
            break;
        };
        let mut end = at;
        while lines.get(end).is_some_and(|l| l.0 == entry_id) {
            end = end.saturating_add(1);
        }
        let group = lines.get(at..end).unwrap_or_default();
        let lots = lots_stmt
            .query_map(params![entry_id, bikri_id, account, person_key], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    (
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                    ),
                    [
                        r.get::<_, i64>(5)?,
                        r.get::<_, i64>(6)?,
                        r.get::<_, i64>(7)?,
                    ],
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        // One credit line for each lot with a credit, in lot order. When the lots and the lines
        // do not match one for one, only the lots with a credit are tried; if that does not match
        // either, the item and quantity are left empty (nothing is guessed).
        let mut with_credit = Vec::new();
        for lot in &lots {
            let credit = paisa(lot.2[0])?
                .checked_sub(paisa(lot.2[1])?)?
                .checked_sub(paisa(lot.2[2])?)?;
            if credit > Paisa::ZERO {
                with_credit.push(lot);
            }
        }
        let candidates: Vec<&LotRow> = if lots.len() == group.len() {
            lots.iter().collect()
        } else if with_credit.len() == group.len() {
            with_credit
        } else {
            Vec::new()
        };
        for (index, (_, _, credit)) in group.iter().enumerate() {
            let lot = candidates.get(index).copied();
            out.push(CreditLine {
                bikri_id,
                amount: paisa(*credit)?,
                qty: lot.map(|l| l.0),
                names: lot.map(|l| l.1.clone()),
            });
        }
        at = end;
    }
    Ok(out)
}

/// The نقد given to a seller on the day: the cash of the lots whose first posting is dated the
/// day (stored on the lot, PRD FR-12.8), leaving out a posting that was cancelled or edited the
/// same day. The debit lines of a بکری posting are this cash and, for a lot whose net is
/// negative (PRD FR-12.7), the seller's debit; only the first is cash he got.
fn cash_given(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
) -> Result<Paisa, ReportsError> {
    let sql = format!(
        "SELECT COALESCE(SUM(l.cash_paid_paisa), 0)
         FROM bikri_lots l JOIN journal_entries e ON e.id = l.first_entry_id
         WHERE l.seller_account_id = ?2 AND (?3 IS NULL OR l.pukhta_person_key = ?3)
           AND e.entry_date = ?1 AND e.status = 'posted' AND NOT ({SAME_DAY_PAIR})"
    );
    let total: i64 = conn.query_row(&sql, params![day, account, person_key], |r| r.get(0))?;
    paisa(total)
}

/// The sum of the debits (or credits) of the day on the party that came from بکری postings: the
/// cash part of a نقدی line (a credit on the buyer) and the debits of a seller.
fn bikri_side(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
    credit: bool,
) -> Result<Paisa, ReportsError> {
    let column = if credit { "l.cr_paisa" } else { "l.dr_paisa" };
    let sql = format!(
        "SELECT COALESCE(SUM({column}), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?2 AND (?3 IS NULL OR l.pukhta_person_key = ?3)
           AND e.entry_date = ?1 AND e.status = 'posted' AND e.source_type = 'bikri'
           AND e.posting_type IN ('bikri_first', 'bikri_later') AND NOT ({SAME_DAY_PAIR})"
    );
    let total: i64 = conn.query_row(&sql, params![day, account, person_key], |r| r.get(0))?;
    paisa(total)
}

/// The sum of the debits (or credits) of the day on the party that came from postings of one
/// kind: the receipts a buyer paid (credits of `receipt`), the payments made to a seller (debits
/// of `payment`). A posting cancelled or edited the same day is left out with its reversal.
fn posting_sum(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
    posting_type: &str,
    credit: bool,
) -> Result<Paisa, ReportsError> {
    let column = if credit { "l.cr_paisa" } else { "l.dr_paisa" };
    let sql = format!(
        "SELECT COALESCE(SUM({column}), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?2 AND (?3 IS NULL OR l.pukhta_person_key = ?3)
           AND e.entry_date = ?1 AND e.status = 'posted' AND e.posting_type = ?4
           AND NOT ({SAME_DAY_PAIR})"
    );
    let total: i64 =
        conn.query_row(&sql, params![day, account, person_key, posting_type], |r| {
            r.get(0)
        })?;
    paisa(total)
}

/// The slip of one party.
fn slip(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    day: &str,
    account: i64,
) -> Result<ReportPage, ReportsError> {
    let control: Option<(String, bool)> = conn
        .query_row(
            "SELECT k.code, a.is_pukhta_khata FROM accounts a
             JOIN account_controls k ON k.id = a.control_id WHERE a.id = ?1",
            [account],
            |r| Ok((r.get(0)?, r.get::<_, i64>(1)? == 1)),
        )
        .ok();
    let Some((control, pukhta)) = control else {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrNotFound,
            "filters.party.account_id",
        )]));
    };
    let buyer = control == BUYER_CONTROL;
    if !buyer && control != SELLER_CONTROL {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.party.account_id",
        )]));
    }
    let key = typed.person_key.as_deref();
    if !pukhta && key.is_some() {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.party.person_key",
        )]));
    }
    if pukhta && key.is_none() {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrRequired,
            "filters.party.person_key",
        )]));
    }

    // The party's sums; a party with no posted line has none.
    let found = activities(conn, day, Some(account), key)?;
    let me = found.into_iter().next().unwrap_or(Activity {
        account_id: account,
        person_key: key.map(str::to_owned),
        buyer,
        prev_dr: Paisa::ZERO,
        prev_cr: Paisa::ZERO,
        day_dr: Paisa::ZERO,
        day_cr: Paisa::ZERO,
        end_dr: Paisa::ZERO,
        end_cr: Paisa::ZERO,
    });
    let previous = net_cell(me.prev_net()?)?;
    let remaining = net_cell(me.end_net()?)?;

    let mut rows = Vec::new();
    let mut total_qty = 0_i64;
    let total_amount: Paisa;
    let mut summary = Vec::new();
    if buyer {
        let lines = sale_lines(conn, day, account, key)?;
        let (mut bought, mut on_spot) = (Paisa::ZERO, Paisa::ZERO);
        for l in &lines {
            total_qty = total_qty
                .checked_add(l.qty)
                .ok_or_else(|| ReportsError::Data("the quantities do not fit".to_owned()))?;
            add(&mut bought, l.total)?;
            if l.cash {
                add(&mut on_spot, l.total)?;
            }
            rows.push(Row {
                cells: vec![
                    Cell::Number { value: l.qty },
                    item_cell(&l.packing_ur, &l.item_ur, &l.packing_en, &l.item_en),
                    Cell::Money { paisa: l.rate },
                    Cell::Money { paisa: l.total },
                ],
                source: Some(RowSource {
                    kind: SourceKind::Bikri,
                    id: l.bikri_id,
                }),
                kind: RowKind::Normal,
            });
        }
        total_amount = bought;
        // What the ledger shows beyond the sale lines: cash lent, a جنرل debit.
        let other_debits = me.day_dr.checked_sub(bought)?;
        // "Paid that day" is the receipts (وصولی) only; any other credit (a جنرل credit, an
        // opening credit, a sale taken back) is its own line (money agent S2).
        let paid = posting_sum(conn, day, account, key, "receipt", true)?;
        let other_credits = me.day_cr.checked_sub(on_spot)?.checked_sub(paid)?;
        if other_debits.is_negative() || paid.is_negative() || other_credits.is_negative() {
            return Err(ReportsError::Data(
                "the sale lines of the day are more than the ledger of the party".to_owned(),
            ));
        }
        summary.push(line(ids::R35_TOTAL_QTY, Cell::Number { value: total_qty }));
        summary.push(line(ids::R35_PREVIOUS, previous));
        summary.push(line(ids::R35_BOUGHT, Cell::Money { paisa: bought }));
        if other_debits != Paisa::ZERO {
            summary.push(line(
                ids::R35_OTHER_DEBITS,
                Cell::Money {
                    paisa: other_debits,
                },
            ));
        }
        if on_spot != Paisa::ZERO {
            summary.push(line(ids::R35_ON_SPOT, Cell::Money { paisa: on_spot }));
        }
        summary.push(line(ids::R35_PAID, blank_if_zero(paid)));
        if other_credits != Paisa::ZERO {
            summary.push(line(
                ids::R35_OTHER_CREDITS,
                Cell::Money {
                    paisa: other_credits,
                },
            ));
        }
        summary.push(line(ids::R35_REMAINING, remaining));
    } else {
        let lines = credit_lines(conn, day, account, key)?;
        let mut sold = Paisa::ZERO;
        for l in &lines {
            add(&mut sold, l.amount)?;
            if let Some(qty) = l.qty {
                total_qty = total_qty
                    .checked_add(qty)
                    .ok_or_else(|| ReportsError::Data("the quantities do not fit".to_owned()))?;
            }
            rows.push(Row {
                cells: vec![
                    l.qty.map_or(Cell::Empty, |value| Cell::Number { value }),
                    l.names
                        .as_ref()
                        .map_or(Cell::Empty, |(pu, pe, iu, ie)| item_cell(pu, iu, pe, ie)),
                    Cell::Empty,
                    Cell::Money { paisa: l.amount },
                ],
                source: Some(RowSource {
                    kind: SourceKind::Bikri,
                    id: l.bikri_id,
                }),
                kind: RowKind::Normal,
            });
        }
        let cash_given = cash_given(conn, day, account, key)?;
        let debited_by_bikri = bikri_side(conn, day, account, key, false)?;
        let credited_by_bikri = bikri_side(conn, day, account, key, true)?;
        // A lot whose net is negative debits the seller; that is not cash he got.
        let negative_nets = debited_by_bikri.checked_sub(cash_given)?;
        let other_credits = me.day_cr.checked_sub(credited_by_bikri)?;
        // "Paid that day" is the payments (ادائیگی) only; an advance given or a جنرل debit is
        // its own line (money agent S3).
        let paid = posting_sum(conn, day, account, key, "payment", false)?;
        let other_debits = me.day_dr.checked_sub(debited_by_bikri)?.checked_sub(paid)?;
        if negative_nets.is_negative()
            || other_credits.is_negative()
            || other_debits.is_negative()
            || paid.is_negative()
            || credited_by_bikri != sold
        {
            return Err(ReportsError::Data(
                "the credit lines of the day do not match the ledger of the seller".to_owned(),
            ));
        }
        if negative_nets != Paisa::ZERO {
            // One row for them, so the rows still add up to what was sold.
            rows.push(Row {
                cells: vec![
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Money {
                        paisa: Paisa::ZERO.checked_sub(negative_nets)?,
                    },
                ],
                source: None,
                kind: RowKind::Normal,
            });
        }
        let sold = sold.checked_sub(negative_nets)?;
        total_amount = sold;
        summary.push(line(ids::R35_TOTAL_QTY, Cell::Number { value: total_qty }));
        summary.push(line(ids::R35_PREVIOUS, previous));
        summary.push(line(ids::R35_SOLD, Cell::Money { paisa: sold }));
        if other_credits != Paisa::ZERO {
            summary.push(line(
                ids::R35_OTHER_CREDITS,
                Cell::Money {
                    paisa: other_credits,
                },
            ));
        }
        if cash_given != Paisa::ZERO {
            summary.push(line(ids::R35_CASH_GIVEN, Cell::Money { paisa: cash_given }));
        }
        summary.push(line(ids::R35_PAID, blank_if_zero(paid)));
        if other_debits != Paisa::ZERO {
            summary.push(line(
                ids::R35_OTHER_DEBITS,
                Cell::Money {
                    paisa: other_debits,
                },
            ));
        }
        summary.push(line(ids::R35_REMAINING, remaining));
    }

    let mut resolved = typed.resolved.clone();
    resolved.as_of = Some(day.to_owned());
    Ok(ReportPage {
        id: ReportId::R35,
        title_id: ids::title_id("R35"),
        filters: resolved,
        columns: slip_columns(),
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Number { value: total_qty },
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: total_amount,
            },
        ],
        summary,
    })
}

/// Builds R35.
///
/// # Errors
/// `ReportsError::Rejected` for a party that is neither a buyer nor a seller, or a one-time
/// account without his person; a database error; `ReportsError::Money` if a stored amount is not
/// whole rupees; `ReportsError::Data` if the sale lines or credit lines of the day do not agree
/// with the ledger of the party.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let today = typed
        .today
        .ok_or_else(|| ReportsError::Data("the report has no today".to_owned()))?;
    let day = typed.as_of.clone().unwrap_or_else(|| today.to_iso());
    match typed.party_id {
        Some(account) => slip(conn, typed, spec, &day, account),
        None => table(conn, typed, spec, &day),
    }
}
