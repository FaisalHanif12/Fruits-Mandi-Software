//! The garden and trip cards of the summary (PRD FR-12.52, plan 97 item 5).
//!
//! No new posting and no new money rule. A garden lot credits the garden's account with its seller
//! credit (C8, PRD FR-12.36) and a trip lot credits the trip's partners (C25, PRD FR-12.26). Both
//! credits are already in the totals of each lot, worked out by [`calc_posting`](super::calc_posting)
//! and read by the journal. This file only adds them up per garden and per trip, with checked
//! `Paisa` arithmetic, so the card shows exactly the line the journal posts (Code Standards B5).
//!
//! A garden lot and a trip lot take no نقد (`process.rs`), so a card has no "paid on the spot" line
//! and a trip's credit is always its gross less its deductions.
//!
//! Worked numbers (E2E-34, a trip lot): 15 پیٹی at 600 = 9,000 gross, seller commission 540, no lot
//! charges: deductions 540, credit 8,460. Worked numbers (a garden): its account holds 12,000 جمع
//! (previous −12,000) and its lots credit 56,400: new −68,400, shown as 68,400 جمع.

use mandi_core::accounts::BalanceSide;
use mandi_core::bikri::{GardenCardLines, LotPreview, PreparedDraft, TripCardLines};
use mandi_core::money::{MoneyError, Paisa};
use mandi_core::rules::Working;
use mandi_core::ventures::{GardenRef, TripRef};

use super::lookups::{Lookups, PartyKind};
use super::seller_paid::{money, shown};

/// The card of one garden: `new = previous - credit`, all signed with debit positive (a جمع balance
/// is negative).
///
/// * `previous` is the garden account's balance before this بکری;
/// * `credit` is the sum of the seller credits (C8) of its lots in this بکری. It can be negative
///   when the charges of a lot are more than its sales: the account is then debited.
///
/// Example: previous −12,000, credit 56,400: new −68,400 (68,400 جمع).
///
/// # Errors
/// [`MoneyError::Overflow`].
pub fn garden_lines(
    garden: GardenRef,
    previous: Paisa,
    credit: Paisa,
) -> Result<GardenCardLines, MoneyError> {
    let new = previous.checked_sub(credit)?;
    let (previous_paisa, previous_side) = shown(previous)?;
    let (new_paisa, new_side) = shown(new)?;
    Ok(GardenCardLines {
        garden,
        previous_paisa,
        previous_side,
        credit_paisa: credit,
        new_paisa,
        new_side,
        working: Working {
            message_id: "working-garden-new-balance".to_owned(),
            args: vec![
                money("previous", previous),
                money("credit", credit),
                money("result", new),
            ],
            rule_id: None,
        },
    })
}

/// The card of one trip: the gross and the deductions of its lots, and the credit that goes to the
/// trip's partners (C25). The credit is the sum of the seller credits (C8), the number the journal
/// posts; with no نقد on a trip lot it is `gross - deductions`.
///
/// Example (E2E-34): gross 9,000, deductions 540, credit 8,460.
///
/// # Errors
/// [`MoneyError::Overflow`] (never raised here; kept so the callers read the same way as for a
/// garden).
pub fn trip_lines(
    trip: TripRef,
    gross: Paisa,
    deductions: Paisa,
    credit: Paisa,
) -> Result<TripCardLines, MoneyError> {
    Ok(TripCardLines {
        trip,
        gross_paisa: gross,
        deductions_paisa: deductions,
        credit_paisa: credit,
        working: Working {
            message_id: "working-trip-card-credit".to_owned(),
            args: vec![
                money("gross", gross),
                money("deductions", deductions),
                money("result", credit),
            ],
            rule_id: None,
        },
    })
}

/// One card for each garden and each trip that has a complete lot in the draft, in the order of the
/// first lot of each. A lot that could not be worked out has none (it has no totals), and neither has
/// a lot whose garden or trip the lookups do not know.
///
/// # Errors
/// [`MoneyError::Overflow`].
pub(super) fn cards_of(
    draft: &PreparedDraft,
    lookups: &Lookups,
    lots: &[LotPreview],
) -> Result<(Vec<GardenCardLines>, Vec<TripCardLines>), MoneyError> {
    // (garden, credit) and (trip, gross, deductions, credit), in the order of the first lot.
    let mut gardens: Vec<(GardenRef, Paisa)> = Vec::new();
    let mut trips: Vec<(TripRef, Paisa, Paisa, Paisa)> = Vec::new();
    for (lot, preview) in draft.lots.iter().zip(lots) {
        let (Some(seller_id), Some(totals)) = (lot.seller_account_id, preview.totals.as_ref())
        else {
            continue;
        };
        let Some(facts) = lookups.accounts.get(&seller_id) else {
            continue;
        };
        if facts.is_own_goods {
            let Some(trip) = lot
                .purchase_id
                .and_then(|id| lookups.purchases.get(&id))
                .map(|purchase| &purchase.trip)
            else {
                continue;
            };
            match trips.iter_mut().find(|(t, _, _, _)| t.id == trip.id) {
                Some((_, gross, deductions, credit)) => {
                    *gross = gross.checked_add(totals.gross)?;
                    *deductions = deductions.checked_add(totals.deductions)?;
                    *credit = credit.checked_add(totals.seller_credit)?;
                }
                None => trips.push((
                    trip.clone(),
                    totals.gross,
                    totals.deductions,
                    totals.seller_credit,
                )),
            }
        } else if facts.kind == PartyKind::Garden {
            // The garden of the seller's account; a lot linked to an arrival may also name it there.
            let from_arrival = lot
                .arrival_id
                .and_then(|id| lookups.arrivals.get(&id))
                .map(|arrival| &arrival.garden)
                .filter(|garden| garden.account.id == seller_id);
            let Some(garden) = lookups.gardens.get(&seller_id).or(from_arrival) else {
                continue;
            };
            match gardens.iter_mut().find(|(g, _)| g.id == garden.id) {
                Some((_, credit)) => *credit = credit.checked_add(totals.seller_credit)?,
                None => gardens.push((garden.clone(), totals.seller_credit)),
            }
        }
    }
    let gardens = gardens
        .into_iter()
        .map(|(garden, credit)| {
            let previous = lookups
                .previous
                .get(&garden.account.id)
                .copied()
                .unwrap_or(Paisa::ZERO);
            garden_lines(garden, previous, credit)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let trips = trips
        .into_iter()
        .map(|(trip, gross, deductions, credit)| trip_lines(trip, gross, deductions, credit))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((gardens, trips))
}

/// The previous balance of a garden card as a signed amount, debit positive.
fn previous_signed(card: &GardenCardLines) -> Result<Paisa, MoneyError> {
    match card.previous_side {
        BalanceSide::Cr => card.previous_paisa.checked_neg(),
        BalanceSide::Dr | BalanceSide::Zero => Ok(card.previous_paisa),
    }
}

/// Puts together the garden cards of the parts of a mixed screen (PRD FR-12.17, FR-12.48): a garden
/// that has lots in more than one part has one card. Its previous balance is the one all the parts
/// were worked out against (nothing of the save is written yet) and its credit is the sum over the
/// parts, so `new` is what its account holds just after the whole posting. The order is that of the
/// first part it is in.
///
/// # Errors
/// [`MoneyError::Overflow`].
pub fn merge_gardens(parts: &[&[GardenCardLines]]) -> Result<Vec<GardenCardLines>, MoneyError> {
    // (garden, previous signed, credit)
    let mut sums: Vec<(GardenRef, Paisa, Paisa)> = Vec::new();
    for card in parts.iter().flat_map(|part| part.iter()) {
        match sums.iter_mut().find(|(g, _, _)| g.id == card.garden.id) {
            Some((_, _, credit)) => *credit = credit.checked_add(card.credit_paisa)?,
            None => sums.push((
                card.garden.clone(),
                previous_signed(card)?,
                card.credit_paisa,
            )),
        }
    }
    sums.into_iter()
        .map(|(garden, previous, credit)| garden_lines(garden, previous, credit))
        .collect()
}

/// Puts together the trip cards of the parts of a mixed screen: a trip that has lots in more than
/// one part has one card, with the sums of its parts. The order is that of the first part it is in.
///
/// # Errors
/// [`MoneyError::Overflow`].
pub fn merge_trips(parts: &[&[TripCardLines]]) -> Result<Vec<TripCardLines>, MoneyError> {
    // (trip, gross, deductions, credit)
    let mut sums: Vec<(TripRef, Paisa, Paisa, Paisa)> = Vec::new();
    for card in parts.iter().flat_map(|part| part.iter()) {
        match sums.iter_mut().find(|(t, _, _, _)| t.id == card.trip.id) {
            Some((_, gross, deductions, credit)) => {
                *gross = gross.checked_add(card.gross_paisa)?;
                *deductions = deductions.checked_add(card.deductions_paisa)?;
                *credit = credit.checked_add(card.credit_paisa)?;
            }
            None => sums.push((
                card.trip.clone(),
                card.gross_paisa,
                card.deductions_paisa,
                card.credit_paisa,
            )),
        }
    }
    sums.into_iter()
        .map(|(trip, gross, deductions, credit)| trip_lines(trip, gross, deductions, credit))
        .collect()
}
