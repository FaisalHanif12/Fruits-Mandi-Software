//! The figures of a garden as money, for the garden reports (PRD 12.11, FR-12.37, FR-12.38; plan
//! d5-late round 11). The garden screens (`services::gardens`) work the same figures out from the
//! same stored sums; a parity test (`tests/garden_parity.rs`) holds the two together on random
//! sequences of payments, sales, other-shop money and closings, so the reports, the analytics and
//! the dashboard tile can never show a number the garden screen does not.
//!
//! Only checked sums and differences: nothing is split or rounded here. Sums of money are `SUM()`
//! on `INTEGER` columns, read apart and subtracted in Rust (money agent Section 9).

use mandi_core::money::{MoneyError, Paisa};
use rusqlite::Connection;

use crate::gardens::Figures;

/// The part of a `WHERE` that leaves out the closing entries and their reversals: the closing moves
/// the money out to the partners, it is not money in. `e` is `journal_entries`. The same text as the
/// predicate of `gardens.rs`; the parity test fails if the two ever differ.
pub const NOT_A_CLOSING: &str = "e.posting_type <> 'garden_close'
       AND (e.reverses_entry_id IS NULL OR NOT EXISTS
            (SELECT 1 FROM journal_entries c WHERE c.id = e.reverses_entry_id AND c.posting_type = 'garden_close'))";

/// Why a garden's figures could not be worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyDataError {
    /// A stored sum is not whole rupees, or a sum or a difference does not fit.
    Money(MoneyError),
    /// The parts paid from the shop and from pockets do not add up to what was paid (C23).
    PartsDoNotAdd,
}

impl From<MoneyError> for MoneyDataError {
    fn from(error: MoneyError) -> Self {
        MoneyDataError::Money(error)
    }
}

/// The figures of a garden as money, worked out from the stored sums. The garden reports, the
/// analytics and the dashboard tile all call [`Money::of`]. The garden screen works the same
/// figures out in `services::gardens::read` (its own copy of this formula); the parity test holds
/// the two together. Only checked sums and differences: nothing is split or rounded here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    /// Σ live installments.
    pub installments: Paisa,
    /// Σ live running costs.
    pub costs: Paisa,
    /// Everything paid: installments and costs.
    pub paid: Paisa,
    /// The part paid from the shop's cash.
    pub paid_shop: Paisa,
    /// The part paid from partners' pockets.
    pub paid_pocket: Paisa,
    /// The money of sales in other shops.
    pub other_shop: Paisa,
    /// The rent still to pay: rent less installments. Signed (more paid than the rent is negative).
    pub rent_due: Paisa,
    /// Money in: credits less debits on its account, the closing and its undo left out.
    pub money_in: Paisa,
    /// The running balance: money in less everything paid. After the closing it is the result.
    pub balance: Paisa,
    /// The balance of the account itself, every line counted (0 after the closing).
    pub account_balance: Paisa,
    /// The latest day of a live entry or line, never before the day the garden was made.
    pub last_date: Option<String>,
}

impl Money {
    /// Works the figures out from the stored sums of a garden and its rent.
    ///
    /// # Errors
    /// `MoneyDataError::Money` if a stored sum is not whole rupees or a sum does not fit;
    /// `MoneyDataError::PartsDoNotAdd` if the shop and pocket parts do not add up to what was paid.
    pub fn of(rent_paisa: i64, f: &Figures) -> Result<Money, MoneyDataError> {
        let whole = Paisa::try_from_paisa;
        let installments = whole(f.installments_paisa)?;
        let costs = whole(f.costs_paisa)?;
        let paid = installments.checked_add(costs)?;
        let paid_shop = whole(f.paid_shop_paisa)?;
        let paid_pocket = whole(f.paid_pocket_paisa)?;
        // The parts of every payment add up to the payment (C23), so the two sources add up to
        // what was paid.
        if paid_shop.checked_add(paid_pocket)? != paid {
            return Err(MoneyDataError::PartsDoNotAdd);
        }
        let money_in = whole(f.credits_paisa)?.checked_sub(whole(f.debits_paisa)?)?;
        let balance = money_in.checked_sub(paid)?;
        let account_balance =
            whole(f.account_credits_paisa)?.checked_sub(whole(f.account_debits_paisa)?)?;
        let rent_due = whole(rent_paisa)?.checked_sub(installments)?;
        Ok(Money {
            installments,
            costs,
            paid,
            paid_shop,
            paid_pocket,
            other_shop: whole(f.other_shop_paisa)?,
            rent_due,
            money_in,
            balance,
            account_balance,
            last_date: f.last_date.clone(),
        })
    }
}

/// What the lines on the accounts of some gardens came from: for each garden account and kind of
/// record (`bikri`, `venture`, `fruit`), `(account id, source type, Σ credits, Σ debits)`, the
/// closing and its undo left out. One query for all the accounts, so money in splits into where it
/// came from (our shop, other shops, fruit taken) and the split can be checked against money in.
///
/// # Errors
/// Any SQLite error.
pub fn money_in_by_source(
    conn: &Connection,
    account_ids: &[i64],
) -> rusqlite::Result<Vec<(i64, String, i64, i64)>> {
    let mut out = Vec::new();
    for chunk in account_ids.chunks(500) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&format!(
            "SELECT l.account_id, e.source_type, COALESCE(SUM(l.cr_paisa), 0), COALESCE(SUM(l.dr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id IN ({marks}) AND e.status = 'posted' AND {NOT_A_CLOSING}
             GROUP BY l.account_id, e.source_type ORDER BY l.account_id, e.source_type"
        ))?;
        let rows = statement.query_map(rusqlite::params_from_iter(chunk), |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?;
        for row in rows {
            out.push(row?);
        }
    }
    Ok(out)
}
