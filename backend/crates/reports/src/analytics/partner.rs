//! View F, by partner, شراکت دار وار (PRD 18.2, FR-17.5, FR-17.9, FR-17.10, C12, C22).
//!
//! One row for each partner who has a number in the period: his income share, his expense share
//! (shown as a positive number), his profit share (C12, income less expense), and side by side
//! his money from trips and his net change.
//!
//! * **Shares.** The sums of his share rows (`partner_share_rows`) dated in the period: income
//!   rows are positive, expense rows negative, a reversal row is the opposite of the row it takes
//!   back and carries its date, so a cancelled sale takes itself out of its own day.
//! * **Trips.** The credits less the debits on his gardens, trips and fruit account (0604) made by
//!   trip postings in the period: the shop's part of a purchase and of a trip cost (a debit), his
//!   part of the seller credit of an own lot and of the money of a sale in another shop (a
//!   credit). A pocket part posts nothing, so it is not here. Entries of a month close and
//!   entries that were reversed, with their reversal, are left out (the live entries).
//! * **Gardens.** The same for the lines of garden entries: the shop's part of an installment or a
//!   running cost (a debit) and his part of the closing (a credit, a debit in a loss closing). A
//!   garden's sales credit the garden's own account, not his, so they come to him only at the
//!   closing. A pocket part posts nothing.
//! * **Fruit.** The credits less the debits on his 0604 account made by fruit postings in the
//!   period: the fruit he took home (a debit of its value) and his equal part of the fruit served
//!   to the shop's guests (C27, a debit of his part), so it is negative. It is read from the
//!   journal, the books, and the parts of all partners add up to minus the value of the fruit
//!   entries of view K. Entries that were reversed, with their reversal, are left out (the live
//!   entries), so an edited or cancelled entry counts once or not at all.
//! * **Other.** Whatever else is on his 0604 account in the period (a جنرل line), so the columns
//!   always add up to his whole 0604 net.
//! * **Net change.** Profit share plus the whole 0604 net: what his کل رقم changed by in the
//!   period (C22), without capital and without money taken.
//!
//! The totals row adds the rows. A row opens the share rows behind it (FR-18.2).
//!
//! Example (E2E-34 after the sale, 10-10-26): الف profit share 480 and trips −4,160 + 3,384 = −776,
//! so his net change is −296, as his partner total in the PRD.

use mandi_core::money::Paisa;
use mandi_db::partners::{PartnerRecord, share_sums};
use mandi_db::rusqlite::{Connection, params};

use super::filters::Typed;
use super::ids;
use super::model::{AnalyticsKey, AnalyticsPartnerMoney};
use super::table::{Table, col, count_of, row};
use crate::error::ReportsError;
use crate::reports::common::{LIVE, add, paisa};
use crate::reports::{Cell, ColumnKind, Names, PageSpec};

/// The first and the last day the period may have, when it is open.
const FIRST_DAY: &str = "0000-01-01";
const LAST_DAY: &str = "9999-12-31";

/// The entries that move a partner's money for a trip (PRD FR-12.30, FR-12.31, FR-12.32,
/// FR-12.26), by where they came from: a purchase, a بکری (whose own-lot credits go to the
/// partners' 0604 accounts), and an entry of a venture that is a trip (a trip cost, a sale in
/// another shop). The text is a constant of this crate (Code Standards B14).
pub(crate) const TRIP_POSTINGS: &str = "(e.source_type IN ('purchase', 'bikri')
     OR (e.source_type = 'venture' AND EXISTS (
            SELECT 1 FROM venture_entries x JOIN ventures v ON v.id = x.venture_id
            WHERE x.id = e.source_id AND v.kind = 'trip')))";

/// The entries that move a partner's money for a garden (PRD FR-12.30, FR-12.35, FR-12.38): an
/// entry of a venture that is a garden, that is the shop's part of an installment or a running
/// cost (a debit) and his part of the closing (a credit, or a debit in a loss closing). A garden's
/// sales credit the garden's own account, never a partner's 0604, and fruit taken has its own
/// source, so this set is the exact set of garden lines. It does not meet [`TRIP_POSTINGS`].
pub(crate) const GARDEN_POSTINGS: &str = "(e.source_type = 'venture' AND EXISTS (
            SELECT 1 FROM venture_entries x JOIN ventures v ON v.id = x.venture_id
            WHERE x.id = e.source_id AND v.kind = 'garden'))";

/// The entries that move a partner's money for fruit taken (PRD FR-16.10): the fruit he took
/// home and his part of the fruit served to the shop's guests. They are their own source
/// (`fruit`), so this set meets neither [`TRIP_POSTINGS`] nor [`GARDEN_POSTINGS`].
pub(crate) const FRUIT_POSTINGS: &str = "(e.source_type = 'fruit')";

/// One partner's figures.
#[derive(Debug, Clone, Copy, Default)]
struct Figures {
    income: Paisa,
    expense: Paisa,
    profit: Paisa,
    gardens: Paisa,
    trips: Paisa,
    fruit: Paisa,
    other: Paisa,
    net: Paisa,
}

impl Figures {
    fn is_zero(&self) -> bool {
        self.income == Paisa::ZERO
            && self.expense == Paisa::ZERO
            && self.gardens == Paisa::ZERO
            && self.trips == Paisa::ZERO
            && self.fruit == Paisa::ZERO
            && self.other == Paisa::ZERO
            && self.net == Paisa::ZERO
    }

    fn add(&mut self, other: &Figures) -> Result<(), ReportsError> {
        add(&mut self.income, other.income)?;
        add(&mut self.expense, other.expense)?;
        add(&mut self.profit, other.profit)?;
        add(&mut self.gardens, other.gardens)?;
        add(&mut self.trips, other.trips)?;
        add(&mut self.fruit, other.fruit)?;
        add(&mut self.other, other.other)?;
        add(&mut self.net, other.net)?;
        Ok(())
    }

    fn cells(&self, lead: Cell) -> Vec<Cell> {
        let money = |paisa| Cell::Money { paisa };
        vec![
            lead,
            money(self.income),
            money(self.expense),
            money(self.profit),
            money(self.gardens),
            money(self.trips),
            money(self.fruit),
            money(self.other),
            money(self.net),
        ]
    }
}

/// The credits less the debits on a partner's 0604 account in the period, on the live entries and
/// not of a month close, by where they came from.
#[derive(Debug, Clone, Copy, Default)]
struct VentureNets {
    trips: Paisa,
    gardens: Paisa,
    fruit: Paisa,
    /// The whole account.
    whole: Paisa,
}

fn venture_net(
    conn: &Connection,
    partner: &PartnerRecord,
    from: &str,
    to: &str,
) -> Result<VentureNets, ReportsError> {
    if partner.venture_account_id == 0 {
        return Ok(VentureNets::default());
    }
    let sql = format!(
        "SELECT COALESCE(SUM(CASE WHEN {TRIP_POSTINGS} THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN {TRIP_POSTINGS} THEN l.cr_paisa END), 0),
                COALESCE(SUM(CASE WHEN {GARDEN_POSTINGS} THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN {GARDEN_POSTINGS} THEN l.cr_paisa END), 0),
                COALESCE(SUM(CASE WHEN {FRUIT_POSTINGS} THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN {FRUIT_POSTINGS} THEN l.cr_paisa END), 0),
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND {LIVE} AND e.source_type <> 'month_close'
           AND e.entry_date >= ?2 AND e.entry_date <= ?3"
    );
    let sums: [i64; 8] =
        conn.query_row(&sql, params![partner.venture_account_id, from, to], |r| {
            let mut out = [0_i64; 8];
            for (index, slot) in out.iter_mut().enumerate() {
                *slot = r.get(index)?;
            }
            Ok(out)
        })?;
    let [
        trip_dr,
        trip_cr,
        garden_dr,
        garden_cr,
        fruit_dr,
        fruit_cr,
        all_dr,
        all_cr,
    ] = sums;
    Ok(VentureNets {
        trips: paisa(trip_cr)?.checked_sub(paisa(trip_dr)?)?,
        gardens: paisa(garden_cr)?.checked_sub(paisa(garden_dr)?)?,
        fruit: paisa(fruit_cr)?.checked_sub(paisa(fruit_dr)?)?,
        whole: paisa(all_cr)?.checked_sub(paisa(all_dr)?)?,
    })
}

/// The money of the trips of one partner by period, in one query: the credits less the debits of
/// his trip lines inside the period, grouped by `bucket_sql` of the entry date. Used by view G
/// with a partner picked, so each bucket is the same number view F gives for that bucket's days.
pub(crate) fn trips_net_by_bucket(
    conn: &Connection,
    partner: &PartnerRecord,
    from: &str,
    to: &str,
    group_by: super::model::AnalyticsGroupBy,
) -> Result<Vec<(String, Paisa)>, ReportsError> {
    net_by_bucket(conn, partner, from, to, group_by, TRIP_POSTINGS)
}

/// The same for the lines of a partner's gardens: view G with a partner picked, so each bucket is
/// the number view F gives for that bucket's days.
pub(crate) fn gardens_net_by_bucket(
    conn: &Connection,
    partner: &PartnerRecord,
    from: &str,
    to: &str,
    group_by: super::model::AnalyticsGroupBy,
) -> Result<Vec<(String, Paisa)>, ReportsError> {
    net_by_bucket(conn, partner, from, to, group_by, GARDEN_POSTINGS)
}

fn net_by_bucket(
    conn: &Connection,
    partner: &PartnerRecord,
    from: &str,
    to: &str,
    group_by: super::model::AnalyticsGroupBy,
    postings: &str,
) -> Result<Vec<(String, Paisa)>, ReportsError> {
    if partner.venture_account_id == 0 {
        return Ok(Vec::new());
    }
    let bucket = super::trend::bucket_sql(group_by, "e.entry_date");
    let sql = format!(
        "SELECT {bucket}, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND {LIVE} AND e.source_type <> 'month_close'
           AND e.entry_date >= ?2 AND e.entry_date <= ?3 AND {postings}
         GROUP BY 1 ORDER BY 1"
    );
    let rows = conn
        .prepare(&sql)?
        .query_map(params![partner.venture_account_id, from, to], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (bucket, debit, credit) in rows {
        out.push((bucket, paisa(credit)?.checked_sub(paisa(debit)?)?));
    }
    Ok(out)
}

/// Builds view F.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let from = typed.from.as_deref().unwrap_or(FIRST_DAY);
    let to = typed.to.as_deref().unwrap_or(LAST_DAY);
    let sums = share_sums(conn, from, to)?;
    let mut entries: Vec<(PartnerRecord, Figures)> = Vec::new();
    for partner in mandi_db::partners::list(conn)? {
        if typed.partner_id.is_some_and(|id| id != partner.id) {
            continue;
        }
        let (mut income, mut expense_rows, mut profit) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        for s in sums.iter().filter(|s| s.partner_id == partner.id) {
            let amount = paisa(s.total_paisa)?;
            add(&mut profit, amount)?;
            match s.kind.as_str() {
                "income" => add(&mut income, amount)?,
                _ => add(&mut expense_rows, amount)?,
            }
        }
        // Expense rows are negative; the view shows the expense as a positive number.
        let expense = Paisa::ZERO.checked_sub(expense_rows)?;
        let nets = venture_net(conn, &partner, from, to)?;
        let figures = Figures {
            income,
            expense,
            profit,
            gardens: nets.gardens,
            trips: nets.trips,
            fruit: nets.fruit,
            other: nets
                .whole
                .checked_sub(nets.trips)?
                .checked_sub(nets.gardens)?
                .checked_sub(nets.fruit)?,
            net: profit.checked_add(nets.whole)?,
        };
        if partner.is_active || !figures.is_zero() {
            entries.push((partner, figures));
        }
    }

    // The three columns of venture money open his lines on that account (FR-18.2).
    let money_col = |key: &str, title: &str, money: AnalyticsPartnerMoney| {
        let mut column = col(key, title, ColumnKind::Money);
        column.drill = Some(AnalyticsKey {
            money: Some(money),
            ..AnalyticsKey::default()
        });
        column
    };
    let columns = vec![
        col("partner", ids::COL_PARTNER, ColumnKind::Names),
        col("income_share", ids::COL_INCOME_SHARE, ColumnKind::Money),
        col("expense_share", ids::COL_EXPENSE_SHARE, ColumnKind::Money),
        col("profit_share", ids::COL_PROFIT_SHARE, ColumnKind::Money),
        money_col("gardens", ids::COL_GARDENS, AnalyticsPartnerMoney::Gardens),
        money_col("trips", ids::COL_TRIPS, AnalyticsPartnerMoney::Trips),
        money_col("fruit", ids::COL_FRUIT, AnalyticsPartnerMoney::Fruit),
        col("other", ids::COL_OTHER_MONEY, ColumnKind::Money),
        col("net_change", ids::COL_NET_CHANGE, ColumnKind::Money),
    ];
    let mut grand = Figures::default();
    let mut all_rows = Vec::with_capacity(entries.len());
    for (partner, figures) in &entries {
        grand.add(figures)?;
        all_rows.push(row(
            figures.cells(Cell::Names {
                names: Names {
                    ur: partner.name_ur.clone(),
                    en: partner.name_en.clone(),
                },
            }),
            AnalyticsKey {
                partner_id: Some(partner.id),
                ..AnalyticsKey::default()
            },
        ));
    }
    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals: grand.cells(Cell::Empty),
    })
}
