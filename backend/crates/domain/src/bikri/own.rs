//! The firm's own goods in a بکری (PRD 12.10, FR-12.24 to FR-12.28, C21, plan 16b): the cost of the
//! goods an own lot sells, worked out per sale line from the purchase's cost left.
//!
//! The rule of C21 (money agent Section 5.4), for the نگ a line takes from the purchase:
//!
//! - the line that takes the last نگ takes all the cost that is left;
//! - any other line takes `split_by_qty(cost_left, [taken, left − taken])` for the first part:
//!   each part is rounded down to whole rupees and the leftover rupees go to the larger part (a
//!   tie: the part taken).
//!
//! Team decisions (money review of plan 16b): the take is per sale line, in line order, and the
//! lots of one بکری that use the same purchase share one running state, in lot order and then line
//! order. So the cost of a lot can differ by a rupee from the cost of one line with the same total
//! نگ (cost 10 over 4 نگ: two lines of 1 take 2 and 2, one line of 2 takes 5). It never takes more
//! than the cost left, and once no نگ is left, all the cost has been taken.
//!
//! In an edit, a line keeps its stored cost when its quantity and its lot's purchase did not
//! change, even if its rate or buyer did (PRD FR-12.20): the goods and the cost stay as they were.
//! Those kept lines are taken out of the state first; the other lines are worked out from what is
//! left.
//!
//! Example (E2E-34): a purchase of 20 نگ with a cost of 10,400. A line of 15 takes
//! `split_by_qty(10,400, [15, 5])` = 7,800 / 2,600, so its cost is 7,800 and 5 نگ with a cost of
//! 2,600 stay.

use std::collections::{HashMap, HashSet};

use mandi_core::bikri::{PreparedDraft, PreparedLot, PreparedSale};
use mandi_core::money::{Paisa, Qty, split_by_qty};
use mandi_core::rules::Working;

use super::lookups::Lookups;
use super::stored::StoredSale;
use super::{CalcError, Env, working};

/// C21: the cost of the نگ taken out of `qty_left` نگ whose cost is `cost_left`.
///
/// # Errors
/// [`CalcError::Inconsistent`] if `qty_taken` is not between 1 and `qty_left`, or the cost left is
/// negative; a money error on overflow.
pub fn own_cost(
    cost_left: Paisa,
    qty_left: i64,
    qty_taken: i64,
) -> Result<(Paisa, Working), CalcError> {
    if qty_taken < 1 || qty_taken > qty_left || cost_left.is_negative() {
        return Err(CalcError::Inconsistent(
            "own goods cost asked for a quantity that is not in the stock",
        ));
    }
    if qty_taken == qty_left {
        return Ok((cost_left, working::cost_of_goods_last(cost_left, qty_taken)));
    }
    let rest = qty_left
        .checked_sub(qty_taken)
        .ok_or(CalcError::Inconsistent("own goods quantity does not fit"))?;
    let parts = [(0_u8, Qty::new(qty_taken)?), (1_u8, Qty::new(rest)?)];
    let taken = split_by_qty(cost_left, &parts)?
        .into_iter()
        .find(|(id, _)| *id == 0)
        .map(|(_, part)| part)
        .ok_or(CalcError::Inconsistent("own goods split lost a part"))?;
    Ok((
        taken,
        working::cost_of_goods(cost_left, qty_taken, qty_left, taken),
    ))
}

/// The line of a posting being edited that keeps its stored cost: it is a line of the same lot, with
/// the same quantity, and the lot still takes from the same purchase (PRD FR-12.20).
pub(super) fn kept_line<'a>(
    env: &Env<'a>,
    lot: &PreparedLot,
    row: &PreparedSale,
) -> Option<&'a StoredSale> {
    let purchase_id = lot.purchase_id?;
    let lot_id = lot.lot_id?;
    let stored = env.stored.sales.get(&row.sale_id?)?;
    let identity = env.stored.lots.get(&lot_id)?.identity.as_ref()?;
    let same = stored.lot_id == lot_id
        && identity.purchase_id == Some(purchase_id)
        && Some(identity.seller_account_id) == lot.seller_account_id
        && row.qty.map(Qty::get) == Some(stored.qty);
    same.then_some(stored)
}

/// What is left of each purchase while a draft is worked out, shared by all its lots.
#[derive(Debug, Default)]
pub(super) struct OwnRun {
    left: HashMap<i64, (i64, Paisa)>,
    /// The stored lines already shown with their stored cost, so a line id sent twice is kept once.
    claimed: HashSet<i64>,
}

impl OwnRun {
    /// Starts from what the purchases can give this posting.
    pub(super) fn new(lookups: &Lookups) -> Self {
        OwnRun {
            left: lookups
                .purchases
                .iter()
                .map(|(id, p)| (*id, (p.qty_left, p.cost_left)))
                .collect(),
            claimed: HashSet::new(),
        }
    }

    /// True the first time a stored line asks to keep its cost. A line id sent twice is a mistake
    /// the edit reports; only the first copy is shown with the stored cost, because the stock was
    /// reserved once (money review of plan 16b, second pass, finding 4).
    pub(super) fn claim_kept(&mut self, sale_id: i64) -> bool {
        self.claimed.insert(sale_id)
    }

    /// The نگ still available from a purchase.
    pub(super) fn qty_left(&self, purchase_id: i64) -> i64 {
        self.left.get(&purchase_id).map_or(0, |(qty, _)| *qty)
    }

    /// Takes out the lines of an edit that keep their stored cost, before any other line is worked
    /// out.
    ///
    /// # Errors
    /// [`CalcError::Inconsistent`] if stored lines do not fit the stock (a bug signal).
    pub(super) fn reserve_kept(
        &mut self,
        env: &Env<'_>,
        draft: &PreparedDraft,
    ) -> Result<(), CalcError> {
        // A stored line can be kept once. The same sale id sent twice is a mistake the edit
        // reports (`line_not_in_posting`); here the second one must not take the stock again.
        let mut seen: HashSet<i64> = HashSet::new();
        for lot in &draft.lots {
            for row in &lot.sales {
                if row.is_blank || row.has_errors {
                    continue;
                }
                let (Some(purchase_id), Some(stored)) = (lot.purchase_id, kept_line(env, lot, row))
                else {
                    continue;
                };
                if !row.sale_id.is_some_and(|id| seen.insert(id)) {
                    continue;
                }
                self.give(purchase_id, stored.qty, stored.cost)?;
            }
        }
        Ok(())
    }

    fn give(&mut self, purchase_id: i64, qty: i64, cost: Paisa) -> Result<(), CalcError> {
        let state = self
            .left
            .get_mut(&purchase_id)
            .ok_or(CalcError::Inconsistent("a lot names an unknown purchase"))?;
        let qty_left =
            state
                .0
                .checked_sub(qty)
                .filter(|left| *left >= 0)
                .ok_or(CalcError::Inconsistent(
                    "stored own goods lines are more than the purchase can give",
                ))?;
        let cost_left = state.1.checked_sub(cost)?;
        if cost_left.is_negative() {
            return Err(CalcError::Inconsistent(
                "stored own goods cost is more than the purchase can give",
            ));
        }
        *state = (qty_left, cost_left);
        Ok(())
    }

    /// Gives back what [`OwnRun::take`] took for a line that could not be worked out after all
    /// (it has a mistake), so the lines after it are shown with the stock they would really have
    /// (money review of plan 16b, second pass). Only the preview can see such a line: a save with a
    /// mistake in it writes nothing.
    ///
    /// # Errors
    /// [`CalcError::Inconsistent`] if the purchase is unknown; a money error on overflow.
    pub(super) fn put_back(
        &mut self,
        purchase_id: i64,
        qty: i64,
        cost: Paisa,
    ) -> Result<(), CalcError> {
        let state = self
            .left
            .get_mut(&purchase_id)
            .ok_or(CalcError::Inconsistent("a lot names an unknown purchase"))?;
        let qty_left = state
            .0
            .checked_add(qty)
            .ok_or(CalcError::Inconsistent("own goods quantity does not fit"))?;
        let cost_left = state.1.checked_add(cost)?;
        *state = (qty_left, cost_left);
        Ok(())
    }

    /// Takes `qty` نگ for a line and gives its cost (C21), or `None` if the purchase has fewer
    /// than `qty` left (the line then has its `ERR_QTY_EXCEEDS_LOT`).
    ///
    /// # Errors
    /// A money error on overflow.
    pub(super) fn take(
        &mut self,
        purchase_id: i64,
        qty: i64,
    ) -> Result<Option<(Paisa, Working)>, CalcError> {
        let Some(&(qty_left, cost_left)) = self.left.get(&purchase_id) else {
            return Ok(None);
        };
        if qty < 1 || qty > qty_left {
            return Ok(None);
        }
        let (cost, shown) = own_cost(cost_left, qty_left, qty)?;
        self.give(purchase_id, qty, cost)?;
        Ok(Some((cost, shown)))
    }
}
