//! The tiles of the dashboard (PRD 9.1, FR-9.1). Every number is read from posted records
//! (PRD FR-6.1) and ties to the screen or report that shows the same figure:
//!
//! * today's sales, count and commission: the sales facts of today, as R34 and the day close
//!   (PRD 4.9: 41,400 in 2 بکری, commission 5,448, which is NOT the shared income 5,488);
//! * cash in hand: Σ debit − Σ credit of 01010001, as R12 (it can be below 0, FR-6.7);
//! * buyers' dues: Σ of the R20 rows, so it equals the R20 total (FR-19.9); sellers' dues: Σ of the
//!   "بیوپاری کے ذمے" column of R24; the dead amount: the R38 total (FR-10.10); کل اگراہی is the three
//!   added;
//! * sellers payable and advances open: Σ of the R24 workings (`open` and `advances_open`,
//!   FR-7.59, FR-15.8); advances are not taken off the payable;
//! * goods not sold: lots of saved بکری with goods left, from the stock moves up to today (a
//!   fruit take writes a `taken` move, so nothing is taken off twice);
//! * market trips: the same read as `trips_summary`;
//! * money lent: the R32 working;
//! * today's expenses: the head 05 accounts today, a month close left out.
//!
//! * gardens: the open gardens and the total of their running balances, each read with the one
//!   function the garden screen uses (`mandi_db::gardens::Money::of`), so the tile equals
//!   `gardens_summary` (PRD 9.1, FR-12.37).
//!
//! Example (PRD 4.9, after Events 1 to 5): 41,400 in 2 بکری, commission 5,448, cash 58,484,
//! dues 21,564, payable 10,160, advances 0, 0 lots, expenses 3,000.

use mandi_core::money::Paisa;
use mandi_db::accounts as account_repo;
use mandi_db::ledger::totals_of;
use mandi_db::rusqlite::Connection;

use super::Facts;
use super::lent::LentFacts;
use super::model::DashboardTiles;
use crate::error::ReportsError;
use crate::gardens::OpenGarden;
use crate::reports::business::day::{expenses_by_day, pair, sales_by_day};
use crate::reports::common::{add, paisa};

/// The shop's one cash account (PRD FR-10.4).
const CASH_ACCOUNT_CODE: &str = "01010001";

/// Builds the tiles.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit; `ReportsError::Data` if the cash account is missing.
pub(crate) fn build(
    conn: &Connection,
    facts: &Facts,
    lent: &LentFacts,
    gardens: &[OpenGarden],
) -> Result<DashboardTiles, ReportsError> {
    let today = facts.today_iso.as_str();

    // Sales and commission: the three commission columns are summed apart and added here.
    let sales = sales_by_day(conn, today, today)?
        .remove(today)
        .unwrap_or_default();
    let mut commission = Paisa::ZERO;
    for part in sales.commission_parts {
        add(&mut commission, paisa(part)?)?;
    }

    let cash_id = account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ReportsError::Data("the cash account is missing".to_owned()))?;
    let cash = totals_of(conn, cash_id, None, None)?;
    let cash_in_hand = paisa(cash.dr_paisa)?.checked_sub(paisa(cash.cr_paisa)?)?;

    let mut buyers_dues = Paisa::ZERO;
    for buyer in &facts.buyers {
        add(&mut buyers_dues, buyer.balance)?;
    }
    let (mut sellers_payable, mut advances_open, mut sellers_dues) =
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    for seller in &facts.sellers {
        if let Some(working) = seller.working {
            add(&mut sellers_payable, working.open)?;
            add(&mut advances_open, working.advances_open)?;
            // What a seller owes beyond his advances: the "بیوپاری کے ذمے" total of R24 (PRD
            // FR-15.9, 9.1).
            add(&mut sellers_dues, working.owes_firm)?;
        }
    }
    // The dead amount (PRD FR-10.10): what the people moved to it still owe. The rows above leave
    // them out, so nothing is counted twice, and کل اگراہی is the three together (FR-19.9).
    let (dead_amount, dead_count) = super::super::dead::tile(conn)?;
    let mut total_dues = buyers_dues;
    add(&mut total_dues, sellers_dues)?;
    add(&mut total_dues, dead_amount)?;

    let (stock_lots, stock_qty) = mandi_db::vouchers::open_stock(conn, today)?;
    let (trips_open, trips_purchases, trips_qty, trips_cost) =
        mandi_db::ventures::tile_totals(conn)?;

    let mut gardens_balance = Paisa::ZERO;
    for garden in gardens {
        add(&mut gardens_balance, garden.balance)?;
    }

    let (expense_dr, expense_cr) = pair(&expenses_by_day(conn, today, today)?, today)?;

    Ok(DashboardTiles {
        sales_gross_paisa: paisa(sales.gross)?,
        sales_count: sales.count,
        commission_paisa: commission,
        cash_in_hand_paisa: cash_in_hand,
        buyers_dues_paisa: buyers_dues,
        sellers_dues_paisa: sellers_dues,
        total_dues_paisa: total_dues,
        dead_amount_paisa: dead_amount,
        dead_count,
        sellers_payable_paisa: sellers_payable,
        advances_open_paisa: advances_open,
        stock_lots,
        stock_qty,
        trips_open,
        trips_purchases_with_stock: trips_purchases,
        trips_qty_left: trips_qty,
        trips_cost_left_paisa: paisa(trips_cost)?,
        gardens_open: i64::try_from(gardens.len())
            .map_err(|_| ReportsError::Data("the number of gardens does not fit".to_owned()))?,
        gardens_balance_paisa: gardens_balance,
        lent_open_paisa: lent.open,
        lent_overdue_paisa: lent.overdue,
        expenses_today_paisa: expense_dr.checked_sub(expense_cr)?,
    })
}
