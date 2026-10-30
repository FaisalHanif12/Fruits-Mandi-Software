//! R34 روزانہ خلاصہ (the day summary) (PRD 19.2, FR-21.1, FR-19.10, E2E-36).
//!
//! For one day, or for each day of a range, the report lists what happened: the بکری of the day
//! (their count and خام بکری), the commission, the cash (opening, in, out, closing), the receipts,
//! payments and advances, the shop expenses, the money partners took and gave back, the capital,
//! the money lent and returned, the trips (bought and sold), each partner's share of the day,
//! the buyers' dues at the end of the day and the lots still in باقی مال. It is a flat list: every
//! row has its day, a label, an optional count and an optional amount, so it pages, prints and
//! exports like every other report. The same rows come in the same order for every day.
//! The garden rows (paid, money in) and the fruit-taken row (نگ and value of the saved entries of
//! the day, PRD 16.7) are in.
//!
//! A day is listed when anything was posted on it (a journal entry, a sale, a trip purchase or
//! entry, a stock move). A report for one day always lists that day. With no dates the report is
//! for today. Entries of a month close are left out everywhere: they move balances only and are
//! dated the day they are made (PRD FR-17.6).
//!
//! Where each number comes from, so it ties to the other screens:
//!
//! * بکری count, خام بکری and commission: the sales facts of the day, the same rows the day close
//!   reads (`analytics_facts`). The commission is the seller commission, the buyer commission and
//!   لاگا (PRD 4.9: 5,448); it is not the shared income of 5,488.
//! * cash in and out: every posted line on the cash account, as the cash book R12 reads it;
//!   opening is the cash at the end of the day before, closing is opening plus in less out.
//! * receipts, payments and advances: the saved vouchers of the day (an edited voucher keeps its
//!   row, a cancelled one is not saved), so the receipts equal the total of R23 for the day.
//! * expenses: head 05 of the journal, as R29 and R30 read it; partner money, capital and money
//!   lent: the live lines of the drawings, capital and 0103 accounts (a cancelled entry and its
//!   reversal are both left out).
//! * partner shares: the share rows of the day, income less expense (C12), as the day close.
//! * buyers' dues: the debit balances of the buyer accounts at the end of the day, one-time
//!   buyers person by person, so the last day of the worked 10-10-26 reads 21,564 as R20.
//! * lots in باقی مال: the lots of saved بکری with goods left at the end of the day (C16), as the
//!   day close counts them.
//!
//! Example (E2E-36, 10-10-26): 2 بکری, خام 41,400, commission 5,448, cash opening 50,000, in
//! 22,800, out 14,316, closing 58,484, buyers' dues 21,564.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::accounts as account_repo;
use mandi_db::ledger::totals_of;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{LIVE, add, col, count_of, names_cell, paisa};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{Cell, ColumnKind, ReportId, ReportPage, Row, RowKind};

/// The shop's one cash account (PRD FR-10.4).
const CASH_ACCOUNT_CODE: &str = "01010001";

/// Two sums for each day: the days are `YYYY-MM-DD` and sort as text.
pub(crate) type ByDay = BTreeMap<String, (i64, i64)>;

fn values(from: &str, to: &str) -> Vec<Value> {
    vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())]
}

/// Runs a query that gives (day, first, second) and keeps the rows by day.
pub(crate) fn by_day(conn: &Connection, sql: &str, args: &[Value]) -> Result<ByDay, ReportsError> {
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map(params_from_iter(args.iter()), |r| {
        Ok((
            r.get::<_, String>(0)?,
            (r.get::<_, i64>(1)?, r.get::<_, i64>(2)?),
        ))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The paisa pair of a day, or zeros.
pub(crate) fn pair(map: &ByDay, day: &str) -> Result<(Paisa, Paisa), ReportsError> {
    let (a, b) = map.get(day).copied().unwrap_or((0, 0));
    Ok((paisa(a)?, paisa(b)?))
}

/// The journal lines of an account set on live entries, by day: Σ debit and Σ credit. `join` joins
/// the accounts that make the set (`l.account_id` is theirs); entries of a month close never count.
fn live_lines(conn: &Connection, join: &str, from: &str, to: &str) -> Result<ByDay, ReportsError> {
    let sql = format!(
        "SELECT e.entry_date, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id {join}
         WHERE {LIVE} AND e.source_type <> 'month_close'
           AND e.entry_date >= ?1 AND e.entry_date <= ?2
         GROUP BY e.entry_date"
    );
    by_day(conn, &sql, &values(from, to))
}

/// The shop expenses by day: Σ debit and Σ credit of the head 05 accounts on posted entries, a
/// month close left out (the dashboard reads the same sums, PRD 9.1).
pub(crate) fn expenses_by_day(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<ByDay, ReportsError> {
    by_day(
        conn,
        "SELECT e.entry_date, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id JOIN account_controls k ON k.id = a.control_id
         JOIN account_heads h ON h.id = k.head_id
         WHERE e.status = 'posted' AND e.source_type <> 'month_close' AND h.code = '05'
           AND e.entry_date >= ?1 AND e.entry_date <= ?2 GROUP BY e.entry_date",
        &values(from, to),
    )
}

/// The days of the range on which anything happened.
fn active_days(conn: &Connection, from: &str, to: &str) -> Result<BTreeSet<String>, ReportsError> {
    let mut statement = conn.prepare(
        "SELECT entry_date FROM journal_entries
           WHERE status = 'posted' AND source_type <> 'month_close'
             AND entry_date >= ?1 AND entry_date <= ?2
         UNION SELECT entry_date FROM analytics_facts WHERE entry_date >= ?1 AND entry_date <= ?2
         UNION SELECT entry_date FROM purchases
           WHERE status = 'saved' AND entry_date >= ?1 AND entry_date <= ?2
         UNION SELECT entry_date FROM venture_entries
           WHERE status = 'saved' AND entry_date >= ?1 AND entry_date <= ?2
         UNION SELECT move_date FROM stock_moves WHERE move_date >= ?1 AND move_date <= ?2",
    )?;
    let rows = statement.query_map([from, to], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// What the sales of a day add up to.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Sales {
    pub(crate) count: i64,
    pub(crate) gross: i64,
    pub(crate) commission_parts: [i64; 3],
}

pub(crate) fn sales_by_day(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<BTreeMap<String, Sales>, ReportsError> {
    let mut out: BTreeMap<String, Sales> = BTreeMap::new();
    let mut statement = conn.prepare(
        "SELECT entry_date, COALESCE(SUM(sign * seller_amount_paisa), 0),
                COALESCE(SUM(sign * seller_comm_paisa), 0), COALESCE(SUM(sign * buyer_comm_paisa), 0),
                COALESCE(SUM(sign * laga_paisa), 0)
         FROM analytics_facts WHERE entry_date >= ?1 AND entry_date <= ?2 GROUP BY entry_date",
    )?;
    let rows = statement.query_map([from, to], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            [
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
            ],
        ))
    })?;
    for row in rows {
        let (day, gross, parts) = row?;
        let sales = out.entry(day).or_default();
        sales.gross = gross;
        sales.commission_parts = parts;
    }
    // A بکری counts on a day when its sales of that day add up to more than 0 (a cancelled one
    // adds up to 0).
    let mut statement = conn.prepare(
        "SELECT entry_date, COUNT(*) FROM (
             SELECT entry_date, bikri_id FROM analytics_facts
             WHERE entry_date >= ?1 AND entry_date <= ?2
             GROUP BY entry_date, bikri_id HAVING SUM(sign) > 0)
         GROUP BY entry_date",
    )?;
    let rows = statement.query_map([from, to], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })?;
    for row in rows {
        let (day, count) = row?;
        out.entry(day).or_default().count = count;
    }
    Ok(out)
}

/// The count and amount of the saved vouchers of a kind, by day.
fn vouchers_by_day(
    conn: &Connection,
    kind: &str,
    from: &str,
    to: &str,
) -> Result<ByDay, ReportsError> {
    by_day(
        conn,
        "SELECT entry_date, COUNT(*), COALESCE(SUM(amount_paisa), 0) FROM vouchers
         WHERE status = 'saved' AND type = ?3 AND entry_date >= ?1 AND entry_date <= ?2
         GROUP BY entry_date",
        &{
            let mut args = values(from, to);
            args.push(Value::Text(kind.to_owned()));
            args
        },
    )
}

/// The money of the sales of trip goods by day: the partners' parts of the seller credits (by the
/// day of the posting) and the money of the sales in other shops.
fn trips_sold_by_day(conn: &Connection, from: &str, to: &str) -> Result<ByDay, ReportsError> {
    by_day(
        conn,
        "SELECT day, COALESCE(SUM(own), 0), COALESCE(SUM(other), 0) FROM (
             SELECT e.entry_date AS day, c.amount_paisa AS own, 0 AS other
             FROM lot_credit_parts c JOIN journal_entries e ON e.id = c.journal_entry_id
             WHERE e.entry_date >= ?1 AND e.entry_date <= ?2
             UNION ALL
             SELECT x.entry_date, 0, x.amount_paisa FROM venture_entries x
             JOIN ventures v ON v.id = x.venture_id AND v.kind = 'trip'
             WHERE x.kind = 'other_sale' AND x.status = 'saved'
               AND x.entry_date >= ?1 AND x.entry_date <= ?2)
         GROUP BY day",
        &values(from, to),
    )
}

/// One balance movement of an account (and person) on a day.
struct Move {
    day: String,
    account_id: i64,
    person: String,
    dr: i64,
    cr: i64,
}

/// The balances of the buyer accounts (control 0304) before `from`, and their movements from
/// `from` to `to`, each by account and person (a پختہ کھاتہ account person by person, like R20).
fn buyer_moves(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<(Vec<Move>, Vec<Move>), ReportsError> {
    let base = "SELECT e.entry_date, l.account_id, COALESCE(l.pukhta_person_key, ''),
                       COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                JOIN accounts a ON a.id = l.account_id
                JOIN account_controls k ON k.id = a.control_id
                WHERE e.status = 'posted' AND k.code = '0304'";
    let read = |sql: String, args: Vec<Value>| -> Result<Vec<Move>, ReportsError> {
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(args.iter()), |r| {
            Ok(Move {
                day: r.get(0)?,
                account_id: r.get(1)?,
                person: r.get(2)?,
                dr: r.get(3)?,
                cr: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    };
    // Before the first day, all together (the day text is the first day, never used).
    let before = read(
        format!(
            "{base} AND e.entry_date < ?1 GROUP BY l.account_id, l.pukhta_person_key",
            base = base.replacen("e.entry_date,", "?1,", 1)
        ),
        vec![Value::Text(from.to_owned())],
    )?;
    let during = read(
        format!(
            "{base} AND e.entry_date >= ?1 AND e.entry_date <= ?2
             GROUP BY e.entry_date, l.account_id, l.pukhta_person_key ORDER BY e.entry_date"
        ),
        values(from, to),
    )?;
    Ok((before, during))
}

/// The stock moves of lots, as (day, lot id, Σ نگ).
type LotMoves = Vec<(String, i64, i64)>;

/// The change of one lot's stock on a day: moves of saved بکری only.
fn stock_moves(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<(LotMoves, LotMoves), ReportsError> {
    let read = |sql: &str, args: Vec<Value>| -> Result<LotMoves, ReportsError> {
        let mut statement = conn.prepare(sql)?;
        let rows = statement.query_map(params_from_iter(args.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    };
    let before = read(
        "SELECT ?1, m.lot_id, SUM(m.qty) FROM stock_moves m
         JOIN bikri_lots l ON l.id = m.lot_id JOIN bikri_entries e ON e.id = l.bikri_id
         WHERE e.status = 'saved' AND m.move_date < ?1 GROUP BY m.lot_id",
        vec![Value::Text(from.to_owned())],
    )?;
    let during = read(
        "SELECT m.move_date, m.lot_id, SUM(m.qty) FROM stock_moves m
         JOIN bikri_lots l ON l.id = m.lot_id JOIN bikri_entries e ON e.id = l.bikri_id
         WHERE e.status = 'saved' AND m.move_date >= ?1 AND m.move_date <= ?2
         GROUP BY m.move_date, m.lot_id ORDER BY m.move_date",
        values(from, to),
    )?;
    Ok((before, during))
}

/// The people in the dead amount on one day, and what they still owed at its end (PRD FR-10.10).
struct DeadDay {
    /// Σ what the people in the dead amount owed at the end of the day.
    amount: Paisa,
    /// How many of them owed something.
    count: i64,
    /// The part of it that is a buyer's, which the buyers' dues leave out.
    buyers: Paisa,
}

/// The dead amount at the end of `day`: the moves dated on or before it and not undone by then,
/// each with his balance at the end of that day (never below 0). Nothing is read when nobody has
/// ever been moved.
fn dead_on(
    conn: &Connection,
    marks: &[mandi_db::dead::MarkRow],
    buyer_accounts: &std::collections::HashSet<i64>,
    day: &str,
) -> Result<DeadDay, ReportsError> {
    let mut out = DeadDay {
        amount: Paisa::ZERO,
        count: 0,
        buyers: Paisa::ZERO,
    };
    for mark in marks {
        let alive_then = mark.entry_date.as_str() <= day
            && mark
                .undone_date
                .as_deref()
                .is_none_or(|undone| undone > day);
        if !alive_then {
            continue;
        }
        let t = totals_of(conn, mark.account_id, mark.person_key.as_deref(), Some(day))?;
        let owed = owing(paisa(t.dr_paisa)?, paisa(t.cr_paisa)?)?;
        if owed == Paisa::ZERO {
            continue;
        }
        add(&mut out.amount, owed)?;
        out.count = out
            .count
            .checked_add(1)
            .ok_or_else(|| ReportsError::Data("the dead count does not fit".to_owned()))?;
        if buyer_accounts.contains(&mark.account_id) {
            add(&mut out.buyers, owed)?;
        }
    }
    Ok(out)
}

/// The positive part of a balance: what the account owes.
fn owing(dr: Paisa, cr: Paisa) -> Result<Paisa, ReportsError> {
    Ok(if dr > cr {
        dr.checked_sub(cr)?
    } else {
        Paisa::ZERO
    })
}

/// Running buyers' dues: apply movements day by day and keep the sum of the debit balances.
struct Dues {
    balances: HashMap<(i64, String), (Paisa, Paisa)>,
    total: Paisa,
    moves: Vec<Move>,
    next: usize,
}

impl Dues {
    fn new(before: Vec<Move>, moves: Vec<Move>) -> Result<Self, ReportsError> {
        let mut balances = HashMap::new();
        let mut total = Paisa::ZERO;
        for m in before {
            let (dr, cr) = (paisa(m.dr)?, paisa(m.cr)?);
            add(&mut total, owing(dr, cr)?)?;
            balances.insert((m.account_id, m.person), (dr, cr));
        }
        Ok(Self {
            balances,
            total,
            moves,
            next: 0,
        })
    }

    /// The dues at the end of `day`: every movement up to and including it is applied first.
    fn at_end_of(&mut self, day: &str) -> Result<Paisa, ReportsError> {
        while let Some(m) = self.moves.get(self.next) {
            if m.day.as_str() > day {
                break;
            }
            let slot = self
                .balances
                .entry((m.account_id, m.person.clone()))
                .or_insert((Paisa::ZERO, Paisa::ZERO));
            let (old_dr, old_cr) = *slot;
            let (new_dr, new_cr) = (
                old_dr.checked_add(paisa(m.dr)?)?,
                old_cr.checked_add(paisa(m.cr)?)?,
            );
            *slot = (new_dr, new_cr);
            self.total = self
                .total
                .checked_sub(owing(old_dr, old_cr)?)?
                .checked_add(owing(new_dr, new_cr)?)?;
            self.next = self.next.saturating_add(1);
        }
        Ok(self.total)
    }
}

/// Running stock: the lots with goods left and their نگ at the end of a day.
struct Stock {
    left: HashMap<i64, i64>,
    lots: i64,
    qty: i64,
    moves: LotMoves,
    next: usize,
}

impl Stock {
    fn new(before: LotMoves, moves: LotMoves) -> Result<Self, ReportsError> {
        let mut stock = Stock {
            left: HashMap::new(),
            lots: 0,
            qty: 0,
            moves: Vec::new(),
            next: 0,
        };
        for (_, lot, qty) in before {
            stock.apply(lot, qty)?;
        }
        stock.moves = moves;
        Ok(stock)
    }

    fn apply(&mut self, lot: i64, change: i64) -> Result<(), ReportsError> {
        let slot = self.left.entry(lot).or_insert(0);
        let old = *slot;
        let new = old
            .checked_add(change)
            .ok_or_else(|| ReportsError::Data("the stock of a lot does not fit".to_owned()))?;
        *slot = new;
        let in_stock = |n: i64| i64::from(n > 0);
        self.lots = self
            .lots
            .checked_sub(in_stock(old))
            .and_then(|v| v.checked_add(in_stock(new)))
            .ok_or_else(|| ReportsError::Data("the lots in stock do not fit".to_owned()))?;
        self.qty = self
            .qty
            .checked_sub(old.max(0))
            .and_then(|v| v.checked_add(new.max(0)))
            .ok_or_else(|| ReportsError::Data("the stock does not fit".to_owned()))?;
        Ok(())
    }

    fn at_end_of(&mut self, day: &str) -> Result<(i64, i64), ReportsError> {
        while let Some((d, lot, qty)) = self.moves.get(self.next).cloned() {
            if d.as_str() > day {
                break;
            }
            self.apply(lot, qty)?;
            self.next = self.next.saturating_add(1);
        }
        Ok((self.lots, self.qty))
    }
}

/// One row of the list.
fn row(day: &str, label: Cell, count: Option<i64>, amount: Option<Paisa>) -> Row {
    Row {
        cells: vec![
            Cell::Date {
                iso: day.to_owned(),
            },
            label,
            count.map_or(Cell::Empty, |value| Cell::Number { value }),
            amount.map_or(Cell::Empty, |paisa| Cell::Money { paisa }),
        ],
        source: None,
        kind: RowKind::Normal,
    }
}

fn label(id: &str) -> Cell {
    Cell::Label { id: id.to_owned() }
}

/// Builds R34.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit; `ReportsError::Data` if the cash account is missing.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let today = typed
        .today
        .ok_or_else(|| ReportsError::Data("the report has no today".to_owned()))?
        .to_iso();
    // No dates: today. Only a last day: that one day. Only a first day: up to today.
    let from = typed
        .from
        .clone()
        .or_else(|| typed.to.clone())
        .unwrap_or_else(|| today.clone());
    let to = typed.to.clone().unwrap_or_else(|| today.clone());
    let cash_id = account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ReportsError::Data("the cash account is missing".to_owned()))?;

    let mut days = active_days(conn, &from, &to)?;
    if from == to {
        days.insert(from.clone());
    }

    let sales = sales_by_day(conn, &from, &to)?;
    let cash = by_day(
        conn,
        "SELECT e.entry_date, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?3 AND e.status = 'posted'
           AND e.entry_date >= ?1 AND e.entry_date <= ?2 GROUP BY e.entry_date",
        &{
            let mut args = values(&from, &to);
            args.push(Value::Integer(cash_id));
            args
        },
    )?;
    let receipts = vouchers_by_day(conn, "receipt", &from, &to)?;
    let payments = vouchers_by_day(conn, "payment", &from, &to)?;
    let advances = vouchers_by_day(conn, "advance", &from, &to)?;
    let expenses = expenses_by_day(conn, &from, &to)?;
    let drawings = live_lines(
        conn,
        "JOIN partners p ON p.drawings_account_id = l.account_id",
        &from,
        &to,
    )?;
    let capital = live_lines(
        conn,
        "JOIN partners p ON p.capital_account_id = l.account_id",
        &from,
        &to,
    )?;
    let lent = live_lines(
        conn,
        "JOIN accounts a ON a.id = l.account_id JOIN account_controls k ON k.id = a.control_id
         AND k.code = '0103'",
        &from,
        &to,
    )?;
    let trips_bought = by_day(
        conn,
        "SELECT entry_date, COALESCE(SUM(qty), 0), COALESCE(SUM(paid_now_paisa), 0) FROM purchases
         WHERE status = 'saved' AND entry_date >= ?1 AND entry_date <= ?2 GROUP BY entry_date",
        &values(&from, &to),
    )?;
    let trips_sold = trips_sold_by_day(conn, &from, &to)?;
    // Gardens: what was paid (installments and costs, from the shop and from pockets) and the
    // money that came into the gardens' accounts (the closing and its undo left out). Both are
    // for information: pocket money is not cash, and a credit sale is not cash yet, so neither is
    // added to the cash lines (money agent, round 11).
    let gardens_paid = by_day(
        conn,
        "SELECT x.entry_date, COUNT(*), COALESCE(SUM(x.amount_paisa), 0)
         FROM venture_entries x JOIN ventures v ON v.id = x.venture_id
         WHERE v.kind = 'garden' AND x.kind IN ('installment', 'cost') AND x.status = 'saved'
           AND x.entry_date >= ?1 AND x.entry_date <= ?2 GROUP BY x.entry_date",
        &values(&from, &to),
    )?;
    let gardens_in = by_day(
        conn,
        &format!(
            "SELECT e.entry_date, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             JOIN ventures v ON v.account_id = l.account_id AND v.kind = 'garden'
             WHERE e.status = 'posted' AND {}
               AND e.entry_date >= ?1 AND e.entry_date <= ?2 GROUP BY e.entry_date",
            mandi_db::garden_figures::NOT_A_CLOSING
        ),
        &values(&from, &to),
    )?;

    // Fruit taken: the saved entries of the day, نگ and value as stored (PRD 16.7). No cash moves
    // and the value is already in the partners' totals, so it is for information like the gardens.
    let fruit_taken = by_day(
        conn,
        "SELECT entry_date, COALESCE(SUM(qty), 0), COALESCE(SUM(value_paisa), 0) FROM fruit_takes
         WHERE status = 'saved' AND entry_date >= ?1 AND entry_date <= ?2 GROUP BY entry_date",
        &values(&from, &to),
    )?;

    // The share of each partner on each day: income rows less expense rows (C12).
    let mut shares: BTreeMap<String, Vec<(i64, i64)>> = BTreeMap::new();
    {
        let mut statement = conn.prepare(
            "SELECT entry_date, partner_id, COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows
             WHERE entry_date >= ?1 AND entry_date <= ?2
             GROUP BY entry_date, partner_id ORDER BY entry_date, partner_id",
        )?;
        let rows = statement.query_map([&from, &to], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for r in rows {
            let (day, partner, amount) = r?;
            shares.entry(day).or_default().push((partner, amount));
        }
    }
    let partner_names: HashMap<i64, (String, String)> = {
        let mut statement = conn.prepare("SELECT id, name_ur, name_en FROM partners")?;
        let rows = statement.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                (r.get::<_, String>(1)?, r.get::<_, String>(2)?),
            ))
        })?;
        rows.collect::<Result<_, _>>()?
    };

    // The running figures: the cash before each day, the buyers' dues and the stock at its end.
    let (open_dr, open_cr) =
        match BusinessDate::parse_iso(&from).and_then(BusinessDate::previous_day) {
            Some(day) => {
                let t = totals_of(conn, cash_id, None, Some(&day.to_iso()))?;
                (paisa(t.dr_paisa)?, paisa(t.cr_paisa)?)
            }
            None => (Paisa::ZERO, Paisa::ZERO),
        };
    let (mut run_dr, mut run_cr) = (open_dr, open_cr);
    let cash_days: Vec<(&String, &(i64, i64))> = cash.iter().collect();
    let mut cash_next = 0_usize;
    let (dues_before, dues_moves) = buyer_moves(conn, &from, &to)?;
    let mut dues = Dues::new(dues_before, dues_moves)?;
    // The dead amount (PRD FR-10.10): the moves, and which of the moved accounts are buyers'.
    let dead_marks = mandi_db::dead::all_marks(conn)?;
    let mut buyer_accounts = std::collections::HashSet::new();
    for mark in &dead_marks {
        if let Some(facts) = account_repo::facts(conn, mark.account_id)?
            && facts.control_code == mandi_core::accounts::BUYER_CONTROL
        {
            buyer_accounts.insert(mark.account_id);
        }
    }
    // Sellers' dues are the read-time working of PRD FR-7.59, which is not dated: it is given for
    // today or later only, and nothing is guessed for an earlier day (as R24 does).
    let today_iso = typed.today.map(|t| t.to_iso());
    let sellers_dues_now = match &today_iso {
        Some(today) => {
            let (sellers, _) = crate::reports::collections::payables::candidates(
                conn,
                &Typed {
                    today: typed.today,
                    ..Typed::default()
                },
                today,
                true,
            )?;
            let mut total = Paisa::ZERO;
            for seller in &sellers {
                if let Some(working) = seller.working {
                    add(&mut total, working.owes_firm)?;
                }
            }
            Some(total)
        }
        None => None,
    };
    let (stock_before, stock_during) = stock_moves(conn, &from, &to)?;
    let mut stock = Stock::new(stock_before, stock_during)?;

    let mut all_rows = Vec::new();
    for day in &days {
        // Cash: every movement before this day is in the opening.
        while let Some((d, (dr, cr))) = cash_days.get(cash_next) {
            if d.as_str() >= day.as_str() {
                break;
            }
            add(&mut run_dr, paisa(*dr)?)?;
            add(&mut run_cr, paisa(*cr)?)?;
            cash_next = cash_next.saturating_add(1);
        }
        let opening = run_dr.checked_sub(run_cr)?;
        let (cash_in, cash_out) = pair(&cash, day)?;
        let closing = opening.checked_add(cash_in)?.checked_sub(cash_out)?;

        let s = sales.get(day).copied().unwrap_or_default();
        let mut commission = Paisa::ZERO;
        for part in s.commission_parts {
            add(&mut commission, paisa(part)?)?;
        }
        let (receipt_count, receipt_sum) = vouchers_row(&receipts, day)?;
        let (payment_count, payment_sum) = vouchers_row(&payments, day)?;
        let (advance_count, advance_sum) = vouchers_row(&advances, day)?;
        let (exp_dr, exp_cr) = pair(&expenses, day)?;
        let (taken, returned) = pair(&drawings, day)?;
        let (cap_dr, cap_cr) = pair(&capital, day)?;
        let (lent_out, lent_back) = pair(&lent, day)?;
        let (bought_qty, bought_cost) = trips_bought.get(day).copied().unwrap_or((0, 0));
        let (sold_own, sold_other) = pair(&trips_sold, day)?;
        let (garden_payments, garden_paid) = gardens_paid.get(day).copied().unwrap_or((0, 0));
        let (garden_dr, garden_cr) = pair(&gardens_in, day)?;
        let (fruit_qty, fruit_value) = fruit_taken.get(day).copied().unwrap_or((0, 0));
        let dues_all = dues.at_end_of(day)?;
        let (open_lots, open_qty) = stock.at_end_of(day)?;
        // PRD FR-10.10: the buyers' dues leave out the people in the dead amount, whose balances
        // are in the dead amount; the total dues are the three together (FR-19.9).
        let dead = dead_on(conn, &dead_marks, &buyer_accounts, day)?;
        let dues_now = dues_all.checked_sub(dead.buyers)?;
        let sellers_now = match (&sellers_dues_now, &today_iso) {
            (Some(total), Some(today)) if day.as_str() >= today.as_str() => Some(*total),
            _ => None,
        };
        let total_dues = match sellers_now {
            Some(sellers) => Some(dues_now.checked_add(sellers)?.checked_add(dead.amount)?),
            None => None,
        };

        let amount = |p: Paisa| Some(p);
        all_rows.push(row(
            day,
            label(ids::R34_BIKRI),
            Some(s.count),
            amount(paisa(s.gross)?),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_COMMISSION),
            None,
            amount(commission),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_CASH_OPENING),
            None,
            amount(opening),
        ));
        all_rows.push(row(day, label(ids::R34_CASH_IN), None, amount(cash_in)));
        all_rows.push(row(day, label(ids::R34_CASH_OUT), None, amount(cash_out)));
        all_rows.push(row(
            day,
            label(ids::R34_CASH_CLOSING),
            None,
            amount(closing),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_RECEIPTS),
            Some(receipt_count),
            amount(receipt_sum),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_PAYMENTS),
            Some(payment_count),
            amount(payment_sum),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_ADVANCES),
            Some(advance_count),
            amount(advance_sum),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_EXPENSES),
            None,
            amount(exp_dr.checked_sub(exp_cr)?),
        ));
        all_rows.push(row(day, label(ids::R34_PARTNER_TAKEN), None, amount(taken)));
        all_rows.push(row(
            day,
            label(ids::R34_PARTNER_RETURNED),
            None,
            amount(returned),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_CAPITAL),
            None,
            amount(cap_cr.checked_sub(cap_dr)?),
        ));
        all_rows.push(row(day, label(ids::R34_MONEY_LENT), None, amount(lent_out)));
        all_rows.push(row(
            day,
            label(ids::R34_MONEY_LENT_BACK),
            None,
            amount(lent_back),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_TRIPS_BOUGHT),
            Some(bought_qty),
            amount(paisa(bought_cost)?),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_TRIPS_SOLD),
            None,
            amount(sold_own.checked_add(sold_other)?),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_GARDENS_PAID),
            Some(garden_payments),
            amount(paisa(garden_paid)?),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_GARDENS_IN),
            None,
            amount(garden_cr.checked_sub(garden_dr)?),
        ));
        all_rows.push(row(
            day,
            label(ids::R34_FRUIT_TAKEN),
            Some(fruit_qty),
            amount(paisa(fruit_value)?),
        ));
        if let Some(day_shares) = shares.get(day) {
            // A heading row, so a partner's name below it reads as his share of the day.
            all_rows.push(row(day, label(ids::R34_PARTNER_SHARES), None, None));
            for (partner, profit) in day_shares {
                let (ur, en) = partner_names.get(partner).cloned().unwrap_or_default();
                all_rows.push(row(
                    day,
                    names_cell(Some(ur), Some(en)),
                    None,
                    amount(paisa(*profit)?),
                ));
            }
        }
        all_rows.push(row(
            day,
            label(ids::R34_BUYERS_DUES),
            None,
            amount(dues_now),
        ));
        all_rows.push(row(day, label(ids::R34_SELLERS_DUES), None, sellers_now));
        all_rows.push(row(
            day,
            label(ids::R34_DEAD_AMOUNT),
            Some(dead.count),
            amount(dead.amount),
        ));
        all_rows.push(row(day, label(ids::R34_TOTAL_DUES), None, total_dues));
        all_rows.push(row(day, label(ids::R34_OPEN_LOTS), Some(open_lots), None));
        all_rows.push(row(day, label(ids::R34_OPEN_QTY), Some(open_qty), None));
    }

    let mut filters = typed.resolved.clone();
    filters.from = Some(from);
    filters.to = Some(to);
    Ok(ReportPage {
        group_above: None,
        id: ReportId::R34,
        title_id: ids::title_id("R34"),
        filters,
        columns: vec![
            col("date", ids::COL_DATE, ColumnKind::Date),
            col("item", ids::COL_LINE, ColumnKind::Label),
            col("count", ids::COL_COUNT, ColumnKind::Number),
            col("amount", ids::COL_AMOUNT, ColumnKind::Money),
        ],
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        // Rows of different kinds do not add up to one number: there is no totals row.
        totals: vec![Cell::Empty; 4],
        summary: Vec::new(),
    })
}

/// The count and the amount of a kind of voucher on a day.
fn vouchers_row(map: &ByDay, day: &str) -> Result<(i64, Paisa), ReportsError> {
    let (count, sum) = map.get(day).copied().unwrap_or((0, 0));
    Ok((count, paisa(sum)?))
}
