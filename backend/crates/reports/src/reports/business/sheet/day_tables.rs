//! The additions of R34, the day total sheet (plan 42, PRD R34 v1.12, E2E-36): the lines for the
//! نگ and the commission, the buyers' and sellers' lines, and the buyers' and sellers' table of each
//! day with a total row.
//!
//! Every figure is the one of the day slips ([`figures`](super::figures)): a table row is one
//! party's slip in a line, so a row and a slip can never differ. Before a day is returned, the
//! sums of its parties are tied to the sales facts the rest of R34 reads (agent rule 6: one value,
//! one source): the buyers' goods, commission, لاگا and نگ must equal the facts', and the seller
//! amount and commission of every active sale line must equal them too.
//!
//! * The summary lines (billed, received, left; net sale, paid out) count every party that has an
//!   entry that day, the people in the dead amount included, so they tie to the cash book. A
//!   table follows the rule of the all-parties table of R35: the row of a person in the dead
//!   amount (FR-10.10) is marked, kept out of the total, and a second total row adds him.
//! * A table holds the parties that have an entry that day, not every party with a balance.
//! * Sellers are the normal sellers (control 03011). A garden's lots credit the garden's account
//!   and a trip's own lots credit the partners, so they have no seller ledger to reconcile.

use std::collections::{BTreeMap, HashMap};

use super::activity::{net_cell, parties_for, sort_by_party};
use super::figures::{BuyerDay, Figures, SellerDay, load};
use super::slip::signed;
use super::*;
use crate::reports::business::day::Sales;
use crate::reports::model::{Column, ReportTable};

/// One line of the flat part of R34: label, optional count, optional amount cell.
pub(crate) type Line = (&'static str, Option<i64>, Option<Cell>);

/// The day figures of R34 for the days asked.
pub(crate) struct DaySheet {
    figures: Figures,
}

fn money(paisa: Paisa) -> Option<Cell> {
    Some(Cell::Money { paisa })
}

fn data_error(text: &str) -> ReportsError {
    ReportsError::Data(text.to_owned())
}

/// Loads the figures of `days` (sorted, oldest first) and ties them to the sales facts.
///
/// # Errors
/// As [`load`], and `ReportsError::Data` when the parties' sums differ from the facts.
pub(crate) fn load_days(
    conn: &Connection,
    days: &[String],
    facts: &BTreeMap<String, Sales>,
) -> Result<DaySheet, ReportsError> {
    let figures = load(conn, days, None)?;
    let sheet = DaySheet { figures };
    for day in days {
        sheet.tie_to_facts(day, facts.get(day).copied().unwrap_or_default())?;
    }
    if let (Some(from), Some(to)) = (days.first(), days.last()) {
        tie_sellers(conn, from, to, facts)?;
    }
    Ok(sheet)
}

/// The seller amount and commission of every active sale line (any lot) against the facts.
fn tie_sellers(
    conn: &Connection,
    from: &str,
    to: &str,
    facts: &BTreeMap<String, Sales>,
) -> Result<(), ReportsError> {
    let mut statement = conn.prepare(
        "SELECT e.entry_date, COALESCE(SUM(s.seller_amount_paisa), 0),
                COALESCE(SUM(s.seller_comm_paisa), 0)
         FROM bikri_sales s JOIN journal_entries e ON e.id = s.journal_entry_id
         WHERE s.is_active = 1 AND e.entry_date >= ?1 AND e.entry_date <= ?2
         GROUP BY e.entry_date",
    )?;
    let rows = statement
        .query_map([from, to], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    for (day, gross, commission) in rows {
        seen.insert(day, (gross, commission));
    }
    let days: Vec<&String> = facts.keys().chain(seen.keys()).collect();
    for day in days {
        let fact = facts.get(day).copied().unwrap_or_default();
        let (gross, commission) = seen.get(day).copied().unwrap_or((0, 0));
        if fact.gross != gross || fact.commission_parts[0] != commission {
            return Err(data_error(
                "the sale lines of the day do not match the sales facts",
            ));
        }
    }
    Ok(())
}

impl DaySheet {
    fn buyers_of<'a>(&'a self, day: &str) -> impl Iterator<Item = &'a BuyerDay> + 'a {
        let day = day.to_owned();
        self.figures
            .buyers
            .range((day.clone(), i64::MIN, String::new())..)
            .take_while(move |(k, _)| k.0 == day)
            .map(|(_, v)| v)
    }

    fn sellers_of<'a>(&'a self, day: &str) -> impl Iterator<Item = &'a SellerDay> + 'a {
        let day = day.to_owned();
        self.figures
            .sellers
            .range((day.clone(), i64::MIN, String::new())..)
            .take_while(move |(k, _)| k.0 == day)
            .map(|(_, v)| v)
    }

    /// The buyers' sums of the day must be the facts of the day.
    fn tie_to_facts(&self, day: &str, fact: Sales) -> Result<(), ReportsError> {
        let (mut goods, mut commission, mut laga) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        let mut qty = 0_i64;
        for b in self.buyers_of(day) {
            add(&mut goods, b.goods)?;
            add(&mut commission, b.commission)?;
            add(&mut laga, b.laga)?;
            qty = qty
                .checked_add(b.qty)
                .ok_or_else(|| data_error("the quantities do not fit"))?;
        }
        if goods != paisa(fact.buyer_value)?
            || commission != paisa(fact.commission_parts[1])?
            || laga != paisa(fact.commission_parts[2])?
            || qty != fact.qty
        {
            return Err(data_error(
                "the buyers' sums of the day do not match the sales facts",
            ));
        }
        Ok(())
    }

    /// The lines after the بکری line: the نگ and the three parts of the commission (seller,
    /// buyer, لاگا). They come from the facts, as the commission total that follows them.
    pub(crate) fn commission_lines(sales: &Sales) -> Result<Vec<Line>, ReportsError> {
        let [seller, buyer, laga] = sales.commission_parts;
        Ok(vec![
            (ids::R34_QTY, Some(sales.qty), None),
            (ids::R34_COMMISSION_SELLER, None, money(paisa(seller)?)),
            (ids::R34_COMMISSION_BUYER, None, money(paisa(buyer)?)),
            (ids::R34_COMMISSION_LAGA, None, money(paisa(laga)?)),
        ])
    }

    /// The buyers' and the sellers' lines of the day, over every party with an entry: billed,
    /// other entries (when there are any), received and left; net sale, other entries (when there
    /// are any) and paid out. `billed + other − received = left` is checked on the cells.
    pub(crate) fn party_lines(&self, day: &str) -> Result<Vec<Line>, ReportsError> {
        let (mut billed, mut b_other, mut received, mut b_left) =
            (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        let mut b_has_other = false;
        for b in self.buyers_of(day) {
            add(&mut billed, b.total)?;
            add(&mut b_other, b.other)?;
            add(&mut received, b.paid)?;
            add(&mut b_left, b.left)?;
            b_has_other |= b.has_other;
        }
        let want_left = billed.checked_add(b_other)?.checked_sub(received)?;
        if want_left != b_left {
            return Err(data_error("the buyers' lines of the day do not add up"));
        }
        let (mut net, mut s_other, mut s_paid, mut s_left) =
            (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        let mut s_has_other = false;
        for s in self.sellers_of(day) {
            add(&mut net, s.sale)?;
            add(&mut s_other, s.other)?;
            add(&mut s_paid, s.paid)?;
            add(&mut s_left, s.left)?;
            s_has_other |= s.has_other;
        }
        // The sellers' lines add up too: net sale + other + paid (a payment is a debit) is what
        // the sellers' accounts moved that day.
        if net.checked_add(s_other)?.checked_add(s_paid)? != s_left {
            return Err(data_error("the sellers' lines of the day do not add up"));
        }
        let mut out: Vec<Line> = vec![(ids::R34_BUYERS_BILLED, None, money(billed))];
        if b_has_other {
            out.push((ids::R34_BUYERS_OTHER, None, Some(net_cell(b_other)?)));
        }
        out.push((ids::R34_BUYERS_RECEIVED, None, money(received)));
        out.push((ids::R34_BUYERS_LEFT, None, Some(net_cell(b_left)?)));
        out.push((ids::R34_SELLERS_NET, None, Some(net_cell(net)?)));
        if s_has_other {
            out.push((ids::R34_SELLERS_OTHER, None, Some(net_cell(s_other)?)));
        }
        out.push((ids::R34_SELLERS_PAID, None, money(s_paid)));
        Ok(out)
    }

    /// The buyers' and the sellers' table of each of `days`.
    pub(crate) fn tables(
        &self,
        conn: &Connection,
        days: &[String],
    ) -> Result<Vec<ReportTable>, ReportsError> {
        let mut per_day: Vec<(&String, Vec<TableRow>, Vec<TableRow>)> = Vec::new();
        let mut keys: Vec<(i64, Option<String>)> = Vec::new();
        for day in days {
            let buyers: Vec<TableRow> = self.buyers_of(day).map(TableRow::of_buyer).collect();
            let sellers: Vec<TableRow> = self.sellers_of(day).map(TableRow::of_seller).collect();
            keys.extend(buyers.iter().chain(&sellers).map(TableRow::key));
            per_day.push((day, buyers, sellers));
        }
        // The names of every party of the page, read once.
        keys.sort();
        keys.dedup();
        let parties = parties_for(conn, &keys)?;
        let mut out = Vec::new();
        for (day, buyers, sellers) in per_day {
            // The people in the dead amount on that day (they can differ from day to day).
            let marks = mandi_db::dead::marks_on(conn, Some(day))?;
            for (kind, rows) in [(Kind::Buyers, buyers), (Kind::Sellers, sellers)] {
                if rows.is_empty() {
                    continue;
                }
                out.push(build_table(kind, day, rows, &parties, &marks)?);
            }
        }
        Ok(out)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Buyers,
    Sellers,
}

/// One party of a table, the same for a buyer and a seller. `today` is the day's total of a
/// buyer (money) or the net sale of a seller with its sign (a balance). The payment takes a
/// buyer's balance down and puts a seller's up (it is a debit on his account).
struct TableRow {
    account_id: i64,
    person: String,
    previous: Paisa,
    today: Paisa,
    other: Paisa,
    has_other: bool,
    paid: Paisa,
    new: Paisa,
}

impl TableRow {
    fn of_buyer(b: &BuyerDay) -> Self {
        Self {
            account_id: b.account_id,
            person: b.person.clone(),
            previous: b.previous,
            today: b.total,
            other: b.other,
            has_other: b.has_other,
            paid: b.paid,
            new: b.new,
        }
    }

    fn of_seller(s: &SellerDay) -> Self {
        Self {
            account_id: s.account_id,
            person: s.person.clone(),
            previous: s.previous,
            today: s.sale,
            other: s.other,
            has_other: s.has_other,
            paid: s.paid,
            new: s.new,
        }
    }

    fn key(&self) -> (i64, Option<String>) {
        (
            self.account_id,
            Some(self.person.clone()).filter(|p| !p.is_empty()),
        )
    }
}

/// The sums of the rows of a table.
#[derive(Default, Clone, Copy)]
struct Sums {
    previous: Paisa,
    today: Paisa,
    other: Paisa,
    paid: Paisa,
    new: Paisa,
}

impl Sums {
    fn add(&mut self, row: &TableRow) -> Result<(), ReportsError> {
        add(&mut self.previous, row.previous)?;
        add(&mut self.today, row.today)?;
        add(&mut self.other, row.other)?;
        add(&mut self.paid, row.paid)?;
        add(&mut self.new, row.new)?;
        Ok(())
    }
}

/// The cells of a row or of a total row after the party columns: previous, today, [other], paid,
/// new. `with_other` says whether the table has the other-entries column. The identity
/// `new = previous + today + other + effect of the payment` is checked on the printed cells.
fn figure_cells(kind: Kind, sums: &Sums, with_other: bool) -> Result<Vec<Cell>, ReportsError> {
    let previous = net_cell(sums.previous)?;
    let today = match kind {
        Kind::Buyers => Cell::Money { paisa: sums.today },
        Kind::Sellers => net_cell(sums.today)?,
    };
    let other = net_cell(sums.other)?;
    let paid = Cell::Money { paisa: sums.paid };
    let new = net_cell(sums.new)?;
    let effect = match kind {
        Kind::Buyers => Paisa::ZERO.checked_sub(signed(&paid)?)?,
        Kind::Sellers => signed(&paid)?,
    };
    let want = signed(&previous)?
        .checked_add(signed(&today)?)?
        .checked_add(if with_other {
            signed(&other)?
        } else {
            Paisa::ZERO
        })?
        .checked_add(effect)?;
    if want != signed(&new)? {
        return Err(data_error("a row of the day sheet does not add up"));
    }
    let mut cells = vec![previous, today];
    if with_other {
        cells.push(other);
    }
    cells.push(paid);
    cells.push(new);
    Ok(cells)
}

fn build_table(
    kind: Kind,
    day: &str,
    rows: Vec<TableRow>,
    parties: &HashMap<(i64, Option<String>), PartyRef>,
    marks: &[mandi_db::dead::MarkRow],
) -> Result<ReportTable, ReportsError> {
    let rows = sort_by_party(rows, parties, TableRow::key);
    // The other-entries column is there only on a day where some row has other entries.
    let with_other = rows.iter().any(|r| r.has_other);
    let mut table_rows = Vec::with_capacity(rows.len());
    let (mut sums, mut with_dead) = (Sums::default(), Sums::default());
    let mut any_dead = false;
    for row in &rows {
        let mark = marks
            .iter()
            .find(|m| m.account_id == row.account_id && row.person.is_empty());
        let mut party = parties
            .get(&row.key())
            .ok_or_else(|| data_error("a party of the day sheet is missing"))?
            .clone();
        party.dead = mark.map(|m| m.info());
        with_dead.add(row)?;
        if mark.is_none() {
            sums.add(row)?;
        } else {
            any_dead = true;
        }
        let code = party
            .short_code
            .clone()
            .or_else(|| Some(party.code.clone()));
        let mut cells = vec![text_cell(code), Cell::Party { party }];
        cells.extend(figure_cells(
            kind,
            &Sums {
                previous: row.previous,
                today: row.today,
                other: row.other,
                paid: row.paid,
                new: row.new,
            },
            with_other,
        )?);
        table_rows.push(Row {
            cells,
            source: Some(RowSource {
                kind: SourceKind::Account,
                id: row.account_id,
            }),
            kind: RowKind::Normal,
        });
    }
    let (party_title, party_key, today_title, today_key, today_kind, title, key) = match kind {
        Kind::Buyers => (
            ids::COL_BUYER,
            "buyer",
            ids::COL_TODAY_TOTAL,
            "today_total",
            ColumnKind::Money,
            ids::R34_TABLE_BUYERS,
            "buyers",
        ),
        Kind::Sellers => (
            ids::COL_SELLER,
            "seller",
            ids::COL_NET_SALE,
            "net_sale",
            ColumnKind::Balance,
            ids::R34_TABLE_SELLERS,
            "sellers",
        ),
    };
    let mut columns: Vec<Column> = vec![
        col("code", ids::COL_CODE, ColumnKind::Text),
        col(party_key, party_title, ColumnKind::Party),
        col("previous", ids::COL_PREVIOUS, ColumnKind::Balance),
        col(today_key, today_title, today_kind),
    ];
    if with_other {
        columns.push(col("other", ids::COL_OTHER_ENTRIES, ColumnKind::Balance));
    }
    columns.push(col("paid", ids::COL_PAID_TODAY, ColumnKind::Money));
    columns.push(col(
        "new_balance",
        ids::COL_NEW_BALANCE,
        ColumnKind::Balance,
    ));

    let mut totals = vec![Cell::Empty, Cell::Empty];
    totals.extend(figure_cells(kind, &sums, with_other)?);
    // A person in the dead amount: a second total row adds him back (PRD FR-10.10).
    if any_dead {
        let mut cells = vec![
            Cell::Label {
                id: ids::R35_TOTAL_WITH_DEAD.to_owned(),
            },
            Cell::Empty,
        ];
        cells.extend(figure_cells(kind, &with_dead, with_other)?);
        table_rows.push(Row {
            cells,
            source: None,
            kind: RowKind::Group,
        });
    }
    Ok(ReportTable {
        key: key.to_owned(),
        day: day.to_owned(),
        title_id: title.to_owned(),
        columns,
        rows: table_rows,
        totals_label_id: ids::TOTAL.to_owned(),
        totals,
    })
}
