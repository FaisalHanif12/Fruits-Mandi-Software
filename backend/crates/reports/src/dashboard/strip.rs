//! The partner strip of the dashboard (PRD 9.2, 17.3, 17.9): for each partner his profit share
//! today, this calendar month and this calendar year up to today, and his کل رقم.
//!
//! * The profit share is the sum of his share rows (income rows positive, expense rows negative,
//!   so the sum is income share − expense share, C12). A reversal row has the date of the row it
//!   takes back, so a cancel or an edit changes the day it belongs to, not today. A month close
//!   writes no share rows.
//! * کل رقم is C22, worked from the same posted records as the partner screen
//!   (`mandi_db::partners::total_parts`; the formula is the one of `services::partners::money`,
//!   kept here because `mandi-reports` may not depend on the services, and a test holds the two
//!   side by side).
//! * An inactive partner is listed only if some number is not 0.
//!
//! Example (PRD 4.9, E2E-37): الف 996 / 996 / 996 and کل رقم −1,004; ب 746 / 746 / 746 and 746
//! (50,746 after he invests 50,000); ج 746 / 746 / 746 and 746.

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::partners::{PartnerRecord, ShareSum, share_sums, total_parts};
use mandi_db::rusqlite::Connection;

use super::model::DashboardPartner;
use crate::error::ReportsError;
use crate::reports::common::{add, paisa};

/// Σ of the share rows of one partner in these sums.
fn share_of(sums: &[ShareSum], partner: i64) -> Result<Paisa, ReportsError> {
    let mut total = Paisa::ZERO;
    for sum in sums.iter().filter(|s| s.partner_id == partner) {
        add(&mut total, paisa(sum.total_paisa)?)?;
    }
    Ok(total)
}

/// C22 for one partner, now.
fn total_of(conn: &Connection, partner: &PartnerRecord) -> Result<Paisa, ReportsError> {
    let parts = total_parts(conn, partner, None)?;
    let capital = paisa(parts.capital_cr)?.checked_sub(paisa(parts.capital_dr)?)?;
    let taken_net = paisa(parts.drawings_dr)?.checked_sub(paisa(parts.drawings_cr)?)?;
    let ventures = paisa(parts.venture_cr)?.checked_sub(paisa(parts.venture_dr)?)?;
    Ok(capital
        .checked_add(paisa(parts.shares)?)?
        .checked_sub(taken_net)?
        .checked_add(ventures)?)
}

/// The strip, in the order the partners were made.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit; `ReportsError::Data` for a day outside the calendar.
pub(crate) fn build(
    conn: &Connection,
    today: BusinessDate,
) -> Result<Vec<DashboardPartner>, ReportsError> {
    let today_iso = today.to_iso();
    let month_start = BusinessDate::from_ymd(today.year(), today.month(), 1)
        .ok_or_else(|| ReportsError::Data("the first day of the month".to_owned()))?
        .to_iso();
    let year_start = BusinessDate::from_ymd(today.year(), 1, 1)
        .ok_or_else(|| ReportsError::Data("the first day of the year".to_owned()))?
        .to_iso();
    let day_sums = share_sums(conn, &today_iso, &today_iso)?;
    let month_sums = share_sums(conn, &month_start, &today_iso)?;
    let year_sums = share_sums(conn, &year_start, &today_iso)?;
    let mut out = Vec::new();
    for partner in mandi_db::partners::list(conn)? {
        let numbers = [
            share_of(&day_sums, partner.id)?,
            share_of(&month_sums, partner.id)?,
            share_of(&year_sums, partner.id)?,
            total_of(conn, &partner)?,
        ];
        if !partner.is_active && numbers.iter().all(|n| *n == Paisa::ZERO) {
            continue;
        }
        let [today_paisa, month_paisa, year_paisa, total_paisa] = numbers;
        out.push(DashboardPartner {
            partner_id: partner.id,
            name_ur: partner.name_ur,
            name_en: partner.name_en,
            today_paisa,
            month_paisa,
            year_paisa,
            total_paisa,
        });
    }
    Ok(out)
}
