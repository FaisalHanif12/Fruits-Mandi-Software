//! Monthly series for the assistant's estimates (plan 55 F1 to F5, PRD FR-7.86, Code Standards Q3).
//!
//! An estimate needs one whole-rupee figure for each full month. Every figure here is read by the
//! screen's own query, with a month bucket added next to it and the screen's own conditions shared,
//! so the months add up to what the screen shows for the same dates (the tests hold the two
//! together):
//!
//! | Series | Read by | Screen it adds up to |
//! |---|---|---|
//! | buyer commission, لاگا | view G's fact sums (`trend::fact_sums_by_period`) | view A / G |
//! | receipts | R23's conditions (`vouchers::receipts_by_month`) | R23 total row |
//! | one expense head | R29's conditions (`expenses::head_by_month`) | the head's row of R29 |
//! | garden sales | the garden journal lines of our shop and other shops (`garden_figures`) | view I `sales_our_shop` + `sales_other_shops` |
//! | trip sales | view G's trip money (`trend::trip_sales_money`) | view J `sales_money` |
//!
//! Then the balances, for the estimate "at this pace" ([`balance_series`]): the balance now, from
//! the very function of its screen, and the net change of each full month.
//!
//! * A party: debits less credits of his one account.
//! * The cash in hand: debits less credits of the cash account, without the capital the partners put
//!   in or take out (PRD FR-17.7): one-off owner movements are not the shop's trading pace.
//! * The buyers' dues: for each owner (a buyer, or a person of the چکتا کھاتہ account) how much the
//!   debit part of his whole balance moves by his lines of the month other than an opening, added
//!   up. See [`dues_changes`].
//!
//! Only trading entries count (PRD FR-7.86 v1.27 names what the pace leaves out: openings and their
//! reversals, "make regular" moves, partner capital in the cash, the dead amount in the dues).
//! Entries with source type `account` are not a pace: opening balances,
//! the reversal of a changed opening (dated on the old opening day) and the moves of "make regular"
//! (FR-10.9, FR-13.8). They are left out of a party's and the cash's change, and in the dues they
//! are applied before the trading lines of their month, so the 0 floor of what a buyer owes acts on
//! his whole balance as R20 does. (For the dues only an opening and the reversal of a changed one
//! are "opening lines": "make regular" is a transfer between two owners and counts as trading.) For the dues the dead mark is ignored (a move to the dead amount
//! posts nothing, so it is a one-off and never a trend, FR-10.10) and the lines of a dead owner
//! after his move are left out (a recovery from him is a one-off too); the balance now is still the
//! tile's, without the dead amount. Without openings and dead people the changes are the very
//! differences of R20's totals "as of" the month-ends (a test holds the two together).
//!
//! The series gives the last 12 full months of change, but not before the subject's first month of
//! activity; a month with nothing in it is a change of 0. The current month is never in it (not
//! finished). The estimate averages only the last 3 of them (the same pace as the other
//! estimates, PRD FR-7.86 "Balances too"); the rest is the history the chart shows, and the number
//! of months tells whether there are the 3 full months the estimate needs.
//!
//! Example (fixture B of the forecast tests, today 10-10-26): Akram's changes April to September
//! are 1,400; 10,000; 18,600; 27,200; 35,800; 44,400 and his balance now is 137,400.

use std::collections::{BTreeMap, HashMap};

use mandi_core::dates::BusinessDate;
use mandi_core::money::{MAX_RUPEES, Paisa};
use mandi_db::Db;
use mandi_db::ledger::{account_totals_in_controls, totals_of};
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, OptionalExtension, params_from_iter};
use mandi_db::seed::CASH_ACCOUNT_CODE;

use super::filters::resolve;
use super::model::{AnalyticsFilters, AnalyticsGroupBy, AnalyticsView};
use super::trend;
use crate::dashboard::narrow;
use crate::error::ReportsError;
use crate::reports::common::{add, in_snapshot, paisa, today as today_of};
use crate::reports::filters::Typed as ReportTyped;

/// Most months of change a balance series gives (the last 12). The pace uses the last 3 of them.
pub const BALANCE_WINDOW_MONTHS: i64 = 12;
/// The first and the last day a range may have when it is open.
const FIRST_DAY: &str = "0000-01-01";
const LAST_DAY: &str = "9999-12-31";

/// The total of one calendar month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthAmount {
    /// The year, for example 2026.
    pub year: i32,
    /// The month, 1 to 12.
    pub month: u32,
    /// Its total in whole rupees, signed.
    pub amount: Paisa,
}

/// Which part of the sale facts a series takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SalePart {
    /// The commission charged to the buyers (without لاگا).
    BuyerCommission,
    /// لاگا.
    Laga,
}

/// What a balance series is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalanceSubject {
    /// One account (a seller, a buyer, a loan or a khata), by its id.
    Party(i64),
    /// The cash in hand (account 01010001).
    Cash,
    /// What the buyers owe (the total of R20).
    BuyersDues,
}

/// A balance and the net change of each full month before today.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceSeries {
    /// The balance now, signed (debit positive), from the screen's own function.
    pub now: Paisa,
    /// The net change of each month, oldest first, consecutive, ending the month before today's
    /// month. Empty when there is nothing before today's month, and also when the balance now is
    /// above the largest allowed amount in size: the series is not read then, because the estimate
    /// is refused for that balance whatever the months say (see [`exceeds_limit`]).
    pub changes: Vec<MonthAmount>,
}

// ---- months ------------------------------------------------------------------------------------

/// A month as one number, so months can be counted: `year × 12 + month − 1`.
fn index_of(year: i32, month: u32) -> Option<i64> {
    i64::from(year)
        .checked_mul(12)?
        .checked_add(i64::from(month.checked_sub(1)?))
}

/// The month of a month number.
fn month_at(index: i64) -> Option<(i32, u32)> {
    let year = i32::try_from(index.checked_div_euclid(12)?).ok()?;
    let month = u32::try_from(index.checked_rem_euclid(12)?.checked_add(1)?).ok()?;
    Some((year, month))
}

/// The year and month of a bucket `YYYY-MM`.
fn parse_bucket(bucket: &str) -> Result<(i32, u32), ReportsError> {
    let bad = || ReportsError::Data(format!("a month bucket is wrong: {bucket}"));
    let (year, month) = bucket.split_once('-').ok_or_else(bad)?;
    let (year, month): (i32, u32) = (
        year.parse().map_err(|_| bad())?,
        month.parse().map_err(|_| bad())?,
    );
    if (1..=12).contains(&month) {
        Ok((year, month))
    } else {
        Err(bad())
    }
}

/// The first day of a month, `YYYY-MM-DD`.
fn first_day(year: i32, month: u32) -> Result<String, ReportsError> {
    BusinessDate::from_ymd(year, month, 1)
        .map(BusinessDate::to_iso)
        .ok_or_else(|| ReportsError::Data("a month is outside the calendar".to_owned()))
}

/// The last day of a month, `YYYY-MM-DD`.
fn last_day(year: i32, month: u32) -> Result<String, ReportsError> {
    let out = || ReportsError::Data("a month is outside the calendar".to_owned());
    let next = index_of(year, month)
        .and_then(|i| i.checked_add(1))
        .and_then(month_at)
        .ok_or_else(out)?;
    BusinessDate::from_ymd(next.0, next.1, 1)
        .and_then(BusinessDate::previous_day)
        .map(BusinessDate::to_iso)
        .ok_or_else(out)
}

/// A series as months, oldest first: the sums of a bucket map, checked.
fn months_of(by_month: &BTreeMap<String, Paisa>) -> Result<Vec<MonthAmount>, ReportsError> {
    let mut out = Vec::new();
    for (bucket, amount) in by_month {
        let (year, month) = parse_bucket(bucket)?;
        out.push(MonthAmount {
            year,
            month,
            amount: *amount,
        });
    }
    Ok(out)
}

fn add_to(
    map: &mut BTreeMap<String, Paisa>,
    bucket: String,
    amount: Paisa,
) -> Result<(), ReportsError> {
    add(map.entry(bucket).or_insert(Paisa::ZERO), amount)?;
    Ok(())
}

/// A moment of the books: one snapshot, with today from the database clock.
fn in_moment<T>(
    db: &Db,
    read: impl FnOnce(&Connection, BusinessDate) -> Result<T, ReportsError>,
) -> Result<T, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            read(conn, today)
        })
    })
}

/// The filters of a series by month, read and checked like view G's.
fn month_filters(filters: &AnalyticsFilters) -> AnalyticsFilters {
    let mut found = filters.clone();
    found.group_by = Some(AnalyticsGroupBy::Month);
    found
}

// ---- flows: the series of an estimate (F1, F2) -------------------------------------------------

/// The buyer commission or the لاگا of each month under the sale filters of `filters` (item,
/// seller, buyer, garden, trip, dates): the net of the sale facts, as view G reads them.
///
/// Example (fixture B): buyer commission 600; 1,200; 1,800; 2,400; 3,000; 3,600.
///
/// # Errors
/// `ReportsError::Rejected` with the problems of the filters; a database or money error.
pub fn sale_part_months(
    db: &Db,
    part: SalePart,
    filters: &AnalyticsFilters,
) -> Result<Vec<MonthAmount>, ReportsError> {
    let input = month_filters(filters);
    in_moment(db, |conn, today| {
        let typed = resolve(conn, today, AnalyticsView::G, &input)?;
        let mut by_month = BTreeMap::new();
        for (bucket, sums) in trend::fact_sums_by_period(conn, &typed)? {
            let amount = match part {
                SalePart::BuyerCommission => sums.buyer_comm,
                SalePart::Laga => sums.laga,
            };
            add_to(&mut by_month, bucket, amount)?;
        }
        months_of(&by_month)
    })
}

/// The money the garden sales brought in each month (in our shop and in other shops, not the
/// fruit taken), for the gardens the filters pick (the garden, the garden's main item): credits
/// less debits on the garden accounts, the closing and its undo left out.
///
/// # Errors
/// As [`sale_part_months`].
pub fn garden_sales_months(
    db: &Db,
    filters: &AnalyticsFilters,
) -> Result<Vec<MonthAmount>, ReportsError> {
    let input = month_filters(filters);
    in_moment(db, |conn, today| {
        let typed = resolve(conn, today, AnalyticsView::G, &input)?;
        // No garden belongs to an item group (PRD v1.31), as in view G.
        if typed.item_group.is_some() {
            return Ok(Vec::new());
        }
        let recs = crate::gardens::pick_recs(
            conn,
            &crate::gardens::Pick {
                item_id: typed.item_id,
                garden_id: typed.garden_id,
                ..crate::gardens::Pick::default()
            },
        )?;
        let accounts: Vec<i64> = recs.iter().map(|r| r.account.id).collect();
        let first = typed.from.as_deref().unwrap_or(FIRST_DAY);
        let last = typed.to.as_deref().unwrap_or(LAST_DAY);
        let mut by_month = BTreeMap::new();
        for (bucket, credits, debits) in
            mandi_db::garden_figures::sales_money_by_month(conn, &accounts, first, last)?
        {
            add_to(
                &mut by_month,
                bucket,
                paisa(credits)?.checked_sub(paisa(debits)?)?,
            )?;
        }
        months_of(&by_month)
    })
}

/// The money the trip sales brought in each month (in our shop and in other shops) for the trips
/// the filters pick (the trip, the purchase's item), as view J's `sales_money`.
///
/// # Errors
/// As [`sale_part_months`].
pub fn trip_sales_months(
    db: &Db,
    filters: &AnalyticsFilters,
) -> Result<Vec<MonthAmount>, ReportsError> {
    let input = month_filters(filters);
    in_moment(db, |conn, today| {
        let typed = resolve(conn, today, AnalyticsView::G, &input)?;
        months_of(&trend::trip_sales_money(conn, &typed)?)
    })
}

/// The saved receipts of each month up to `to` (a day, `YYYY-MM-DD`), all of them or only the
/// account `account_id`'s, as R23.
///
/// # Errors
/// A database or money error.
pub fn receipts_months(
    db: &Db,
    account_id: Option<i64>,
    to: Option<&str>,
) -> Result<Vec<MonthAmount>, ReportsError> {
    in_moment(db, |conn, today| {
        let typed = ReportTyped {
            today: Some(today),
            to: to.map(str::to_owned),
            party_id: account_id,
            ..ReportTyped::default()
        };
        let mut by_month = BTreeMap::new();
        for (bucket, amount) in crate::reports::vouchers::receipts_by_month(conn, &typed)? {
            add_to(&mut by_month, bucket, amount)?;
        }
        months_of(&by_month)
    })
}

/// The net shop expense of the expense head `head_id` in each month up to `to`, as R29. `None`
/// when there is no such head.
///
/// # Errors
/// A database or money error.
pub fn expense_head_months(
    db: &Db,
    head_id: i64,
    to: Option<&str>,
) -> Result<Option<Vec<MonthAmount>>, ReportsError> {
    in_moment(db, |conn, today| {
        let Some(head) = mandi_db::expense_heads::head(conn, head_id)? else {
            return Ok(None);
        };
        let typed = ReportTyped {
            today: Some(today),
            to: to.map(str::to_owned),
            ..ReportTyped::default()
        };
        let mut by_month = BTreeMap::new();
        for (bucket, amount) in
            crate::reports::business::expenses::head_by_month(conn, &typed, head.account_id)?
        {
            add_to(&mut by_month, bucket, amount)?;
        }
        Ok(Some(months_of(&by_month)?))
    })
}

// ---- balances: the series of "at this pace" (F4, F5) -----------------------------------------

/// The lines a balance series reads: one account or all the buyer accounts, in posted entries. A
/// party's and the cash's change leave out the entries of source type `account` (see the module
/// text); the dues read them apart ([`dues_changes`]).
fn scope_of(subject: BalanceSubject, account_id: Option<i64>) -> (&'static str, Vec<Value>) {
    match (subject, account_id) {
        (BalanceSubject::BuyersDues, _) => (
            "l.account_id IN (SELECT a.id FROM accounts a
                JOIN account_controls k ON k.id = a.control_id WHERE k.code = '0304')",
            Vec::new(),
        ),
        (_, Some(id)) => ("l.account_id = ?", vec![Value::Integer(id)]),
        (_, None) => ("0", Vec::new()),
    }
}

/// What else a subject leaves out, as conditions on `e` (`journal_entries`; PRD FR-7.86 v1.27): the
/// cash leaves out the partner capital entries and their reversals (a reversal is dated like the entry it undoes and nets in
/// the same month, but is left out with it).
fn also_left_out(subject: BalanceSubject) -> &'static str {
    match subject {
        BalanceSubject::Cash => {
            " AND e.posting_type NOT IN ('capital_in', 'capital_out')
              AND (e.reverses_entry_id IS NULL OR NOT EXISTS
                   (SELECT 1 FROM journal_entries c WHERE c.id = e.reverses_entry_id
                      AND c.posting_type IN ('capital_in', 'capital_out')))"
        }
        BalanceSubject::Party(_) | BalanceSubject::BuyersDues => "",
    }
}

/// True when the subject has a line dated before `before`.
fn has_activity_before(
    conn: &Connection,
    scope: &str,
    left_out: &str,
    args: &[Value],
    before: &str,
) -> Result<bool, ReportsError> {
    let sql = format!(
        "SELECT EXISTS(SELECT 1 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE {scope} AND e.status = 'posted' AND e.source_type <> 'account'{left_out}
           AND e.entry_date < ?)"
    );
    let mut all = args.to_vec();
    all.push(Value::Text(before.to_owned()));
    let found: i64 = conn.query_row(&sql, params_from_iter(all.iter()), |r| r.get(0))?;
    Ok(found == 1)
}

/// The net movement (debits less credits) of the subject for each month from `first` to `last`.
fn movement_by_month(
    conn: &Connection,
    scope: &str,
    left_out: &str,
    args: &[Value],
    first: &str,
    last: &str,
) -> Result<BTreeMap<String, Paisa>, ReportsError> {
    let sql = format!(
        "SELECT substr(e.entry_date, 1, 7), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE {scope} AND e.status = 'posted' AND e.source_type <> 'account'{left_out}
           AND e.entry_date >= ? AND e.entry_date <= ?
         GROUP BY 1 ORDER BY 1"
    );
    let mut all = args.to_vec();
    all.push(Value::Text(first.to_owned()));
    all.push(Value::Text(last.to_owned()));
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(all.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = BTreeMap::new();
    for (bucket, dr, cr) in rows {
        add_to(&mut out, bucket, paisa(dr)?.checked_sub(paisa(cr)?)?)?;
    }
    Ok(out)
}

/// The months of the window as month numbers, oldest first: the last
/// [`BALANCE_WINDOW_MONTHS`] full months before `current`, but not before the first month of
/// activity. `first_active` is the first month inside the window that has any movement (`YYYY-MM`)
/// and `older` is true when the subject had activity before the window. Empty when it has none.
fn window_months(
    current: i64,
    first_active: Option<&str>,
    older: bool,
) -> Result<Vec<i64>, ReportsError> {
    let window_first = current
        .checked_sub(BALANCE_WINDOW_MONTHS)
        .ok_or_else(|| ReportsError::Data("the window does not fit".to_owned()))?;
    let start = if older {
        window_first
    } else {
        let Some(bucket) = first_active else {
            return Ok(Vec::new());
        };
        let (year, month) = parse_bucket(bucket)?;
        index_of(year, month)
            .ok_or_else(|| ReportsError::Data("a month is outside the calendar".to_owned()))?
    };
    let mut months = Vec::new();
    let mut at = start;
    while at < current {
        months.push(at);
        at = at
            .checked_add(1)
            .ok_or_else(|| ReportsError::Data("the window does not fit".to_owned()))?;
    }
    Ok(months)
}

/// A buyer, or one person of the چکتا کھاتہ account: the group whose balance counts as owed when it
/// is a debit.
type Owner = (i64, Option<String>);

/// The lines of an opening balance and of the reversal of a changed one (`e` is `journal_entries`).
/// (The old "make regular" move of plan 13 was a transfer between two owners and counted as trading;
/// it was removed in plan 64.)
const IS_OPENING: &str = "(e.posting_type = 'opening' OR (e.reverses_entry_id IS NOT NULL
        AND EXISTS (SELECT 1 FROM journal_entries c
                    WHERE c.id = e.reverses_entry_id AND c.posting_type = 'opening')))";

/// The control code of the buyers (PRD FR-10.2).
const BUYERS_CONTROL: &str = "0304";

/// The lines of the buyers' accounts, as a join after `journal_lines l` (`e` is `journal_entries`;
/// the control code is the parameter `?2`). The entries come first (by the date index) and each
/// one's lines are found by the entry, so a window of a year reads that year's lines and not all of
/// the buyers' history. It is a `CROSS JOIN` on purpose: written as `account_id IN (...)`, SQLite
/// loops over every buyer account for every entry of the window.
const BUYER_LINES: &str = "CROSS JOIN accounts a ON a.id = l.account_id
        AND a.control_id = (SELECT k.id FROM account_controls k WHERE k.code = ?2)";

/// 1 for a line the series leaves out because its owner is in the dead amount (PRD FR-7.86 v1.27):
/// one dated on or after the day of his move. A recovery from a dead person is a one-off, like the
/// move itself, and R20 and the tile do not move for it (he is already out of them). `l` is
/// `journal_lines` and `e` is `journal_entries`. The first test is a cheap pre-filter: nobody is
/// in the dead amount in most books.
const IS_AFTER_HIS_MOVE: &str = "CASE WHEN l.account_id IN
          (SELECT d.account_id FROM dead_marks d WHERE d.is_active = 1)
        AND EXISTS (SELECT 1 FROM dead_marks d
          WHERE d.is_active = 1 AND d.account_id = l.account_id
            AND d.chukta_person_key IS l.chukta_person_key AND e.entry_date >= d.entry_date)
        THEN 1 ELSE 0 END";

/// The two sides (debits, credits) of every owner of the buyers' accounts, in paisa.
type Sides = HashMap<Owner, (i64, i64)>;

/// The first month inside `first ..= last` that has an entry of the buyers (posted, not of source
/// type `account`), as `YYYY-MM`, or `None`. It stops at the first entry found.
fn first_dues_month(
    conn: &Connection,
    first: &str,
    last: &str,
) -> Result<Option<String>, ReportsError> {
    let sql = format!(
        "SELECT substr(e.entry_date, 1, 7) FROM journal_entries e
         WHERE e.status = 'posted' AND e.source_type <> 'account'
           AND e.entry_date >= ?1 AND e.entry_date <= ?3
           AND EXISTS (SELECT 1 FROM journal_lines l {BUYER_LINES} WHERE l.entry_id = e.id)
         ORDER BY e.entry_date LIMIT 1"
    );
    Ok(conn
        .query_row(&sql, (first, BUYERS_CONTROL, last), |r| {
            r.get::<_, String>(0)
        })
        .optional()?)
}

/// True when a line of a normal buyer account carries a person key. Only the shared چکتا کھاتہ
/// account holds many people (PRD FR-13.5), so the balance of a normal account is one owner's.
/// The database does not enforce it, so the series checks it and refuses if it does not hold (one
/// index seek for each account).
fn normal_account_has_a_person_key(conn: &Connection) -> Result<bool, ReportsError> {
    let found: i64 = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM accounts a
            WHERE a.control_id = (SELECT k.id FROM account_controls k WHERE k.code = ?1)
              AND a.is_chukta_khata = 0
              AND EXISTS (SELECT 1 FROM journal_lines l
                          WHERE l.account_id = a.id AND l.chukta_person_key IS NOT NULL))",
        [BUYERS_CONTROL],
        |r| r.get(0),
    )?;
    Ok(found == 1)
}

/// The two sides of every owner of the buyers' accounts over all posted lines: a normal account is
/// one owner, a person of the shared چکتا کھاتہ account (and its lines with no person) is one
/// each. R20 reads them in the same way (`owing()`).
fn buyer_sides(conn: &Connection) -> Result<Sides, ReportsError> {
    let mut sides: Sides = HashMap::new();
    let mut shared: Vec<i64> = Vec::new();
    let mut statement = conn.prepare(
        "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
         WHERE k.code = ?1 AND a.is_chukta_khata = 1",
    )?;
    for id in statement.query_map([BUYERS_CONTROL], |r| r.get::<_, i64>(0))? {
        shared.push(id?);
    }
    for (account_id, totals) in account_totals_in_controls(conn, &[BUYERS_CONTROL], None)? {
        if !shared.contains(&account_id) {
            sides.insert((account_id, None), (totals.dr_paisa, totals.cr_paisa));
        }
    }
    let mut persons = conn.prepare(
        "SELECT l.chukta_person_key, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND e.status = 'posted' GROUP BY l.chukta_person_key",
    )?;
    for account_id in shared {
        let rows = persons
            .query_map([account_id], |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (key, dr, cr) in rows {
            sides.insert((account_id, key), (dr, cr));
        }
    }
    Ok(sides)
}

/// The change of what the buyers owe in each month of `months`: for each owner, how much the debit
/// part of his balance moves by his lines of the month other than an opening, added up over the
/// owners.
///
/// The debit part is R20's rule (`owing()`): an owner owes what his whole balance is above 0, and
/// the whole balance includes his opening. So the balance is the whole one, and only the opening
/// lines are taken out of the change: in a month the lines of an opening and of the reversal of a
/// changed one ([`IS_OPENING`]) are applied first, then the other lines ("make regular" among
/// them: a transfer between two owners), and the change is the debit part after both less the
/// debit part after the first. A buyer paying
/// down an opening of 50,000 by 2,000 a month therefore moves the dues by -2,000 a month, and one
/// with an opening credit adds nothing until his balance is a debit. The dead mark is ignored (a
/// move posts nothing) and the lines after a move are left out (see [`IS_AFTER_HIS_MOVE`]).
///
/// How it is read (K8: 8 s down to under 1 s on 3.5 million lines): one pass over the lines from
/// the first month of the series on, grouped by owner, month, opening or not and after his move or
/// not; one pass over the totals of the buyers' accounts, which R20 reads too; and the balance of
/// each owner before the series is his total less what that first pass found from the series on
/// (each side apart, in whole rupees, checked). That is exactly what a grouped pass over the lines
/// before the series gave, without reading all of the history twice more. (An owner who moved to
/// the dead amount before the series has every line of it left out of the months, so his balance
/// before it is never read.) Without openings and dead people the changes are the differences of
/// R20's totals "as of" the month-ends.
fn dues_changes(conn: &Connection, months: &[i64]) -> Result<Vec<MonthAmount>, ReportsError> {
    let out = || ReportsError::Data("a month is outside the calendar".to_owned());
    let (Some(&first), Some(&last)) = (months.first(), months.last()) else {
        return Ok(Vec::new());
    };
    let (first_year, first_month) = month_at(first).ok_or_else(out)?;
    let first_iso = first_day(first_year, first_month)?;
    let last_bucket = {
        let (year, month) = month_at(last).ok_or_else(out)?;
        format!("{year:04}-{month:02}")
    };
    if normal_account_has_a_person_key(conn)? {
        return Err(ReportsError::Data(
            "a line of a normal buyer account has a person key".to_owned(),
        ));
    }
    let owed = |balance: Paisa| {
        if balance > Paisa::ZERO {
            balance
        } else {
            Paisa::ZERO
        }
    };

    // One pass over the lines from the first day of the series on (this month's too: they are
    // taken off the totals below): for each owner and month, the opening lines and the other lines
    // apart, and the lines after the owner's move to the dead amount apart.
    let window_sql = format!(
        "SELECT l.account_id, l.chukta_person_key, substr(e.entry_date, 1, 7),
                CASE WHEN {IS_OPENING} THEN 1 ELSE 0 END, {IS_AFTER_HIS_MOVE},
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_entries e CROSS JOIN journal_lines l ON l.entry_id = e.id {BUYER_LINES}
         WHERE e.status = 'posted' AND e.entry_date >= ?1
         GROUP BY 1, 2, 3, 4, 5"
    );
    let mut statement = conn.prepare(&window_sql)?;
    let rows = statement
        .query_map((&first_iso, BUYERS_CONTROL), |r| {
            Ok((
                (r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?),
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)? == 1,
                r.get::<_, i64>(4)? == 1,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // The balance of each owner before the series is the whole balance, openings included: all his
    // lines less those from the first day of the series on (each side apart, checked) ...
    let mut sides = buyer_sides(conn)?;
    // ... and, for each owner and month, the net of the opening lines and of the other lines.
    let mut by_month: BTreeMap<String, HashMap<Owner, (Paisa, Paisa)>> = BTreeMap::new();
    for (owner, bucket, is_opening, after_his_move, dr, cr) in rows {
        let Some((all_dr, all_cr)) = sides.get_mut(&owner) else {
            return Err(ReportsError::Data(
                "a buyer line is missing from the totals".to_owned(),
            ));
        };
        // Every line from the first day on is taken off, those after a move too.
        *all_dr = paisa(*all_dr)?.checked_sub(paisa(dr)?)?.as_paisa();
        *all_cr = paisa(*all_cr)?.checked_sub(paisa(cr)?)?.as_paisa();
        if after_his_move || bucket > last_bucket {
            continue;
        }
        let net = paisa(dr)?.checked_sub(paisa(cr)?)?;
        let slot = by_month
            .entry(bucket)
            .or_default()
            .entry(owner)
            .or_insert((Paisa::ZERO, Paisa::ZERO));
        if is_opening {
            slot.0 = slot.0.checked_add(net)?;
        } else {
            slot.1 = slot.1.checked_add(net)?;
        }
    }
    let mut balances: HashMap<Owner, Paisa> = HashMap::new();
    for (owner, (dr, cr)) in &sides {
        balances.insert(owner.clone(), paisa(*dr)?.checked_sub(paisa(*cr)?)?);
    }
    // An owner who moved to the dead amount before the series has his lines from that day on left
    // out of every month, so his balance before the series is never read: nothing is taken off it.
    let mut changes = Vec::new();
    for &at in months {
        let (year, month) = month_at(at).ok_or_else(out)?;
        let mut change = Paisa::ZERO;
        for (owner, (opening, trading)) in by_month
            .remove(&format!("{year:04}-{month:02}"))
            .unwrap_or_default()
        {
            let old = balances.get(&owner).copied().unwrap_or(Paisa::ZERO);
            // The opening lines first: they are not a pace ...
            let after_opening = old.checked_add(opening)?;
            // ... then the trading lines, whose effect on what he owes is the month's change.
            let after_all = after_opening.checked_add(trading)?;
            change = change
                .checked_add(owed(after_all))?
                .checked_sub(owed(after_opening))?;
            balances.insert(owner, after_all);
        }
        changes.push(MonthAmount {
            year,
            month,
            amount: change,
        });
    }
    Ok(changes)
}

/// True when a balance is above the largest allowed amount in size: the very test
/// `domain::forecast::within_limit` makes (this crate cannot use the domain). An estimate of such a
/// balance is refused whatever the months say, so its series is not read.
fn exceeds_limit(balance: Paisa) -> bool {
    let Ok(largest) = Paisa::from_rupees(MAX_RUPEES) else {
        return true;
    };
    balance
        .as_paisa()
        .checked_abs()
        .is_none_or(|size| size > largest.as_paisa())
}

/// The balance of the subject now and the net change of each full month before today (see the
/// module text). One snapshot of the books; `today` is the day of the question.
///
/// Example (fixture B, today 10-10-26): for Akram `now` is 137,400 and the changes April to
/// September are 1,400; 10,000; 18,600; 27,200; 35,800; 44,400.
///
/// # Errors
/// A database or money error; `ReportsError::NotFound` for a party that does not exist.
pub fn balance_series(
    db: &Db,
    subject: BalanceSubject,
    today: BusinessDate,
) -> Result<BalanceSeries, ReportsError> {
    db.read(|conn| in_snapshot(conn, |conn| read_balance_series(conn, subject, today)))
}

fn read_balance_series(
    conn: &Connection,
    subject: BalanceSubject,
    today: BusinessDate,
) -> Result<BalanceSeries, ReportsError> {
    let out = || ReportsError::Data("a month is outside the calendar".to_owned());
    let current = index_of(today.year(), today.month()).ok_or_else(out)?;
    let window_first = current.checked_sub(BALANCE_WINDOW_MONTHS).ok_or_else(out)?;
    let (first_year, first_month) = month_at(window_first).ok_or_else(out)?;
    let (prev_year, prev_month) =
        month_at(current.checked_sub(1).ok_or_else(out)?).ok_or_else(out)?;
    let first = first_day(first_year, first_month)?;
    let last = last_day(prev_year, prev_month)?;

    // The account and the balance now, by the function of the screen the figure comes from.
    let (account_id, now) = match subject {
        BalanceSubject::Party(id) => (Some(id), account_balance(conn, id)?),
        BalanceSubject::Cash => {
            let id = mandi_db::accounts::id_by_code(conn, CASH_ACCOUNT_CODE)?
                .ok_or_else(|| ReportsError::Data("the cash account is missing".to_owned()))?;
            (Some(id), narrow::cash_of(conn)?)
        }
        BalanceSubject::BuyersDues => (None, narrow::buyers_of(conn, today)?),
    };
    // A balance over the largest allowed amount is refused whatever its months say: no series.
    if exceeds_limit(now) {
        return Ok(BalanceSeries {
            now,
            changes: Vec::new(),
        });
    }
    let (scope, args) = scope_of(subject, account_id);
    let left_out = also_left_out(subject);
    let older = has_activity_before(conn, scope, left_out, &args, &first)?;

    let mut changes = Vec::new();
    if subject == BalanceSubject::BuyersDues {
        // Only the first month with an entry matters (when nothing is older), not the movements.
        let first_active = if older {
            None
        } else {
            first_dues_month(conn, &first, &last)?
        };
        let months = window_months(current, first_active.as_deref(), older)?;
        changes = dues_changes(conn, &months)?;
    } else {
        let activity = movement_by_month(conn, scope, left_out, &args, &first, &last)?;
        let months = window_months(current, activity.keys().next().map(String::as_str), older)?;
        for &at in &months {
            let (year, month) = month_at(at).ok_or_else(out)?;
            let bucket = format!("{year:04}-{month:02}");
            changes.push(MonthAmount {
                year,
                month,
                amount: activity.get(&bucket).copied().unwrap_or(Paisa::ZERO),
            });
        }
    }
    Ok(BalanceSeries { now, changes })
}

/// The balance of one account now: debits less credits, as the party card reads it.
fn account_balance(conn: &Connection, account_id: i64) -> Result<Paisa, ReportsError> {
    if mandi_db::accounts::get(conn, account_id)?.is_none() {
        return Err(ReportsError::NotFound);
    }
    let totals = totals_of(conn, account_id, None, None)?;
    Ok(paisa(totals.dr_paisa)?.checked_sub(paisa(totals.cr_paisa)?)?)
}
