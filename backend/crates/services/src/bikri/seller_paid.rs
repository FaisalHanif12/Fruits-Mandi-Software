//! `bikri_seller_paid_spread`: what a seller takes on the spot, spread over the نقد of his lots
//! (PRD FR-12.23 v1.53, plan 91). Read only.
//!
//! The screen's seller field is a way to fill the lots' نقد fields, which stay visible and
//! editable (NFR-USE-27). This function works out what goes into each lot, with the one domain
//! function [`spread_seller_paid`], and gives back the preview of the draft with those نقد values in
//! place. The save then takes the lots' نقد as typed, like any other, and every figure of the
//! summary comes from `calc_posting` (Code Standards B5).
//!
//! Two passes: the first reads each lot's credit (C8, which does not depend on نقد), the second
//! works the draft out again with the spread نقد.
//!
//! Worked numbers: رحیم has two lots with credits 12,000 and 8,000 and `15000` is asked: lot 1 gets
//! 12,000 and lot 2 gets 3,000.

use std::collections::HashSet;

use mandi_domain::bikri::{SpreadLot, spread_seller_paid};
use mandi_domain::core::bikri::{
    LotCashSpread, SellerPaidSpread, SellerPaidSpreadInput, prepare_draft,
};
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;

use super::edit::plan_edit;
use super::shared::{Basis, switches, work_out};
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::rules;

/// Works out the نقد of each lot of the sellers named, and the preview with it in place.
///
/// An entry with an empty amount is ignored; `0` puts 0 on every lot he has. A seller who has no lot
/// that takes cash (no lot yet, the lot is not complete, trip goods, a garden, a later posting) is
/// `ERR_FORMAT_INVALID` at `seller_paid[k].seller_account_id` when the amount is more than 0. The
/// same seller twice is that error on the second entry. A bad amount is the usual money error at
/// `seller_paid[k].amount`. All of them come back inside `preview.errors`.
///
/// # Errors
/// `ERR_NOT_FOUND` on `entry_id` for an edit of a posting that does not exist;
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix.
pub fn seller_paid_spread(
    ctx: &Context,
    input: &SellerPaidSpreadInput,
) -> Result<SellerPaidSpread, ServiceError> {
    let window = day::window(ctx)?;
    // Fetched before the read below: the read connection is not re-entrant.
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    ctx.db.read(|conn| {
        let (mut prepared, basis, mut extra) = match input.entry_id {
            Some(entry_id) => {
                let plan = plan_edit(conn, entry_id, &input.draft, window)?;
                (plan.prepared, plan.basis, plan.extra)
            }
            None => (
                prepare_draft(&input.draft, window.1, window.0),
                Basis::new_bikri(),
                Vec::new(),
            ),
        };
        // Pass 1: the credit of each lot.
        let first = work_out(conn, prepared.clone(), &rule_set, switches, &basis)?;
        let first_posting =
            basis.target.kind(&basis.stored) == mandi_domain::bikri::PostingKind::First;
        // A lot takes cash when its seller is one that can be paid on the spot (his summary exists),
        // the lot was worked out, and this is a first posting.
        let payable: HashSet<i64> = first
            .calc
            .sellers
            .iter()
            .map(|s| s.seller_account_id)
            .collect();
        let lots: Vec<SpreadLot> = prepared
            .lots
            .iter()
            .zip(&first.calc.lots)
            .enumerate()
            .filter_map(|(index, (lot, preview))| {
                let seller_id = lot.seller_account_id?;
                let totals = preview.totals.as_ref()?;
                Some(SpreadLot {
                    index,
                    seller_id,
                    credit: totals.seller_credit,
                    takes_cash: first_posting && payable.contains(&seller_id),
                })
            })
            .collect();

        let mut spread: Vec<(usize, Paisa)> = Vec::new();
        let mut seen: HashSet<i64> = HashSet::new();
        for (k, request) in input.seller_paid.iter().enumerate() {
            let amount_field = format!("seller_paid[{k}].amount");
            let seller_field = format!("seller_paid[{k}].seller_account_id");
            if request.amount.trim().is_empty() {
                continue;
            }
            if !seen.insert(request.seller_account_id) {
                extra.push(FieldError::new(ErrorCode::ErrFormatInvalid, &seller_field));
                continue;
            }
            let amount = match validate::money(&amount_field, &request.amount) {
                Ok(amount) => amount,
                Err(error) => {
                    extra.push(error);
                    continue;
                }
            };
            match spread_seller_paid(&lots, request.seller_account_id, amount)
                .map_err(money_error)?
            {
                Some(parts) => {
                    for (index, cash) in parts {
                        if let Some(lot) = prepared.lots.get_mut(index) {
                            lot.cash_paid = Some(cash);
                            spread.push((index, cash));
                        }
                    }
                }
                // Nothing to place: 0 is nothing to do, more than 0 has no lot to go on.
                None if amount == Paisa::ZERO => {}
                None => extra.push(FieldError::new(ErrorCode::ErrFormatInvalid, &seller_field)),
            }
        }
        spread.sort_by_key(|(index, _)| *index);
        let lots_cash: Vec<LotCashSpread> = spread
            .into_iter()
            .filter_map(|(index, cash)| {
                prepared.lots.get(index).map(|lot| LotCashSpread {
                    key: lot.key.clone(),
                    cash_paid_paisa: cash,
                })
            })
            .collect();

        // Pass 2: the draft with the spread نقد.
        let mut worked = work_out(conn, prepared, &rule_set, switches, &basis)?;
        worked.errors.extend(extra);
        Ok(SellerPaidSpread {
            lots: lots_cash,
            preview: worked.preview(),
        })
    })
}
