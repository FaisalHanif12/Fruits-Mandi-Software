//! Narrow reads of the dashboard tiles (plan 48 P48-S1, NFR-AI-3, NFR-SPD-2).
//!
//! `get` reads everything the dashboard shows. The assistant's quick chips and its `tiles` tool
//! need a tile or two, and on the NFR-SCL-1 volume the full read takes 7 seconds: the ages of
//! every buyer's open bills (R20) and the attention list are most of it. The functions here read
//! one tile each, or all the tiles without those two, from the very same sources as
//! [`tiles::build`](super::tiles): the same posted-record queries, the same dead set, the same sums
//! added in the same order. Nothing is stored and nothing is written (Code Standards B29).
//!
//! The buyers' dues are the balances of the rows of R20 (PRD FR-19.9) without the ages of their
//! bills: `buyers_balances_now` is the first half of what R20 does, so the sum can never differ
//! from the total row of R20, and every tile of [`tiles`] equals the tile of `get`.
//!
//! Each function reads in ONE snapshot (PRD FR-6.1). The ones that depend on the day take it from
//! the caller, so a tile and its second place are read for the same day even across midnight;
//! `tiles_now` takes its own day, like `get`, and gives it back. The day given must be TODAY: the
//! sales and the stock are read for that day, but the balances (buyers, sellers, cash) are the
//! balances now, as in `get`. A past day would mix the two (money review of P48-S1, finding D).
//!
//! Example (PRD 4.9, after Events 1 to 5): `cash_in_hand` is 58,484, `buyers_dues` 21,564,
//! `sales_today` 41,400 in 2 بکری with commission 5,448, `total_dues` 21,564.

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::Db;
use mandi_db::accounts as account_repo;
use mandi_db::ledger::totals_of;
use mandi_db::rusqlite::Connection;
use mandi_db::seed::CASH_ACCOUNT_CODE;

use super::model::DashboardTiles;
use super::{Facts, lent, tiles};
use crate::error::ReportsError;
use crate::reports::business::day::sales_by_day;
use crate::reports::collections::dues::buyers_balances_now;
use crate::reports::collections::payables::candidates;
use crate::reports::common::{add, in_snapshot, paisa, today as today_of};
use crate::reports::filters::Typed;

/// Runs a read in one snapshot of the books with one value of today.
fn in_one_moment<T>(
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

/// Today's sales: خام بکری, how many بکری and the commission (seller + buyer + لاگا).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SalesToday {
    /// Σ خام بکری of today's postings.
    pub gross_paisa: Paisa,
    /// How many بکری have sales today.
    pub count: i64,
    /// Seller commission + buyer commission + لاگا today.
    pub commission_paisa: Paisa,
}

/// What the sellers hold: the sums of the rows of R24.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellersNow {
    /// Σ of what is payable to them (the `open` of the workings).
    pub payable_paisa: Paisa,
    /// Σ of their open advances.
    pub advances_open_paisa: Paisa,
    /// Σ of what they owe beyond their advances: the "بیوپاری کے ذمے" total of R24.
    pub dues_paisa: Paisa,
}

/// The three parts of کل اگراہی and their sum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuesParts {
    /// What buyers owe (the total row of R20).
    pub buyers_paisa: Paisa,
    /// What sellers owe beyond their advances (R24).
    pub sellers_paisa: Paisa,
    /// The dead amount (R38).
    pub dead_paisa: Paisa,
    /// The three added: کل اگراہی.
    pub total_paisa: Paisa,
}

/// All the tiles of PRD 9.1, with the day they were read for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TilesNow {
    /// Today in Pakistan, `YYYY-MM-DD`: the day the tiles were read for.
    pub today: String,
    /// The tiles, the very ones `get` gives.
    pub tiles: DashboardTiles,
}

fn cash_of(conn: &Connection) -> Result<Paisa, ReportsError> {
    let cash_id = account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ReportsError::Data("the cash account is missing".to_owned()))?;
    let cash = totals_of(conn, cash_id, None, None)?;
    Ok(paisa(cash.dr_paisa)?.checked_sub(paisa(cash.cr_paisa)?)?)
}

fn sales_of(conn: &Connection, today: &str) -> Result<SalesToday, ReportsError> {
    let sales = sales_by_day(conn, today, today)?
        .remove(today)
        .unwrap_or_default();
    let mut commission = Paisa::ZERO;
    for part in sales.commission_parts {
        add(&mut commission, paisa(part)?)?;
    }
    Ok(SalesToday {
        gross_paisa: paisa(sales.gross)?,
        count: sales.count,
        commission_paisa: commission,
    })
}

fn buyers_of(conn: &Connection, today: BusinessDate) -> Result<Paisa, ReportsError> {
    let mut total = Paisa::ZERO;
    for buyer in buyers_balances_now(conn, today)? {
        add(&mut total, buyer.balance)?;
    }
    Ok(total)
}

fn sellers_of(conn: &Connection, today: BusinessDate) -> Result<SellersNow, ReportsError> {
    let typed = Typed {
        today: Some(today),
        ..Typed::default()
    };
    let (sellers, _) = candidates(conn, &typed, &today.to_iso(), true)?;
    let (mut payable, mut advances, mut dues) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    for seller in &sellers {
        if let Some(working) = seller.working {
            add(&mut payable, working.open)?;
            add(&mut advances, working.advances_open)?;
            add(&mut dues, working.owes_firm)?;
        }
    }
    Ok(SellersNow {
        payable_paisa: payable,
        advances_open_paisa: advances,
        dues_paisa: dues,
    })
}

fn dues_of(conn: &Connection, today: BusinessDate) -> Result<DuesParts, ReportsError> {
    let buyers = buyers_of(conn, today)?;
    let sellers = sellers_of(conn, today)?.dues_paisa;
    let (dead, _) = crate::dead::tile(conn)?;
    let mut total = buyers;
    add(&mut total, sellers)?;
    add(&mut total, dead)?;
    Ok(DuesParts {
        buyers_paisa: buyers,
        sellers_paisa: sellers,
        dead_paisa: dead,
        total_paisa: total,
    })
}

/// The cash in hand: Σ debit − Σ credit of the cash account (it can be below 0, PRD FR-6.7).
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees;
/// `ReportsError::Data` if the cash account is missing.
pub fn cash_in_hand(db: &Db) -> Result<Paisa, ReportsError> {
    in_one_moment(db, |conn, _| cash_of(conn))
}

/// Today's sales and commission, from the sales facts of today.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit.
pub fn sales_today(db: &Db, today: BusinessDate) -> Result<SalesToday, ReportsError> {
    db.read(|conn| in_snapshot(conn, |conn| sales_of(conn, &today.to_iso())))
}

/// The lots with goods left and their نگ, from the stock moves up to today.
///
/// # Errors
/// A database error.
pub fn stock(db: &Db, today: BusinessDate) -> Result<(i64, i64), ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            Ok(mandi_db::vouchers::open_stock(conn, &today.to_iso())?)
        })
    })
}

/// What buyers owe: the total row of R20, with nobody of the dead amount.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit.
pub fn buyers_dues(db: &Db, today: BusinessDate) -> Result<Paisa, ReportsError> {
    db.read(|conn| in_snapshot(conn, |conn| buyers_of(conn, today)))
}

/// The sums of the sellers' rows: payable, open advances and what they owe beyond them.
///
/// # Errors
/// As [`buyers_dues`].
pub fn sellers(db: &Db, today: BusinessDate) -> Result<SellersNow, ReportsError> {
    db.read(|conn| in_snapshot(conn, |conn| sellers_of(conn, today)))
}

/// The dead amount and how many people it holds (PRD FR-10.10).
///
/// # Errors
/// As [`buyers_dues`].
pub fn dead(db: &Db) -> Result<(Paisa, i64), ReportsError> {
    in_one_moment(db, |conn, _| crate::dead::tile(conn))
}

/// کل اگراہی: the buyers' dues, the sellers' dues and the dead amount, each read once in the same
/// moment, and their sum.
///
/// # Errors
/// As [`buyers_dues`].
pub fn total_dues(db: &Db, today: BusinessDate) -> Result<DuesParts, ReportsError> {
    db.read(|conn| in_snapshot(conn, |conn| dues_of(conn, today)))
}

/// All the tiles of PRD 9.1 without the ages of the buyers' bills and without the attention list,
/// the partner strip, the lists and the charts. Every tile is the tile `get` gives.
///
/// # Errors
/// As [`buyers_dues`].
pub fn tiles_now(db: &Db) -> Result<TilesNow, ReportsError> {
    in_one_moment(db, |conn, today| {
        let today_iso = today.to_iso();
        let buyers = buyers_balances_now(conn, today)?;
        let typed = Typed {
            today: Some(today),
            ..Typed::default()
        };
        let (sellers, sums) = candidates(conn, &typed, &today_iso, true)?;
        let facts = Facts {
            today,
            today_iso,
            buyers,
            sellers,
            sums: sums.unwrap_or_default(),
        };
        let lent = lent::facts(conn, today)?;
        let gardens = crate::gardens::open_gardens(conn)?;
        Ok(TilesNow {
            today: facts.today_iso.clone(),
            tiles: tiles::build(conn, &facts, &lent, &gardens)?,
        })
    })
}
