//! One sale line (PRD 12.4, 6.1 of the money agent): its amounts, its three charges and their
//! workings. Pure; used by the preview now and by the save in plan 13 (Code Standards B5).

use std::collections::HashMap;

use mandi_core::bikri::{
    ChargeSource, LineCalc, LineCharge, LineWorkings, PaymentType, PreparedSale, RuleLabel,
    StoredMethod,
};
use mandi_core::error::ErrorCode;
use mandi_core::money::{Grams, Paisa, Qty, line_value, weight_value};
use mandi_core::rules::{ChargeSide, ChargeType, ChargeValue, RuleLevel};

use super::checks::{self, Issues};
use super::lookups::{AccountFacts, ItemFacts};
use super::person::clean_person;
use super::stored::{StoredCharge, StoredSale};
use super::working;
use super::{CalcError, Env, PostingKind};
use crate::ledger::MAX_LINE_PAISA;
use crate::rules::{LineOverride, Query, Winner, charge_amount, resolve};

/// The lot a line belongs to, as the line maths needs it.
pub(super) struct LotRef {
    /// The saved lot, in an edit or a later sale.
    pub lot_id: Option<i64>,
    /// The seller (the party of the seller-side charges).
    pub seller_id: i64,
    /// True if the seller is a separate khata: its seller-side defaults are 0 (PRD FR-16.7).
    pub seller_is_khata: bool,
    /// The item.
    pub item_id: i64,
    /// The item's facts.
    pub item: ItemFacts,
    /// True if an edit changed the seller, the item or the one-time seller of this lot: nothing
    /// stored for the old lot is kept (money agent review of plan 16).
    pub identity_changed: bool,
}

/// Everything one line needs.
pub(super) struct LineArgs<'a> {
    pub env: &'a Env<'a>,
    pub lot: &'a LotRef,
    /// True once the lot's per-lot seller commission has been charged on an earlier line.
    pub per_lot_charged: &'a mut bool,
    /// The running amounts buyers owe, for the credit limit (see [`checks::credit_limit`]).
    pub owed: &'a mut HashMap<i64, Paisa>,
    pub row: &'a PreparedSale,
    pub buyer_id: i64,
    pub buyer: &'a AccountFacts,
    pub payment: PaymentType,
    /// The stored line, when the row is an edited line that the user did not change (PRD
    /// FR-12.20): its commissions and لاگا are kept exactly.
    pub stored: Option<&'a StoredSale>,
    /// The per-lot seller commission of a line that an edit removed, waiting for the first line
    /// that takes it (PRD FR-11.9).
    pub pending_move: &'a mut Option<StoredCharge>,
    /// C21: the cost of the own goods this line sells and how it was worked out, for a line of an
    /// own lot (PRD FR-12.26). `None` for a line of any other lot: its cost is 0.
    pub cost: Option<(Paisa, mandi_core::rules::Working)>,
}

/// The stored line of a row, if the user did not change it (PRD FR-12.20): the buyer, the
/// one-time person, the quantity, the weight, both rates, how the buyer pays and the commissions
/// and لاگا typed on the line are all what was stored. A row with no `sale_id`, or whose
/// `sale_id` is not stored in this posting, is a new line.
pub(super) fn unchanged_stored<'a>(
    env: &Env<'a>,
    lot: &LotRef,
    row: &PreparedSale,
) -> Option<&'a StoredSale> {
    let stored = env.stored.sales.get(&row.sale_id?)?;
    if lot.identity_changed || lot.lot_id != Some(stored.lot_id) {
        return None;
    }
    let buyer_id = row.buyer_account_id?;
    if buyer_id != stored.buyer_account_id {
        return None;
    }
    let buyer = env.lookups.accounts.get(&buyer_id)?;
    if checks::resolve_payment(buyer, row.payment_type)? != stored.payment_type {
        return None;
    }
    let person_key = if buyer.is_pukhta {
        Some(clean_person(&row.pukhta)?.key)
    } else {
        None
    };
    if person_key != stored.person_key {
        return None;
    }
    let qty = row.qty?;
    let seller_rate = row.seller_rate?;
    let buyer_rate = if row.rate_differs {
        row.buyer_rate?
    } else {
        seller_rate
    };
    // A weight typed for an item sold by نگ is ignored (calc_line does the same).
    let weight = if lot.item.by_weight {
        row.weight.map(Grams::get)
    } else {
        None
    };
    let same = qty.get() == stored.qty
        && weight == stored.weight_grams
        && seller_rate == stored.seller_rate
        && buyer_rate == stored.buyer_rate
        && row.seller_comm == stored.seller_comm.typed()
        && row.buyer_comm == stored.buyer_comm.typed()
        && row.laga == stored.laga.typed();
    same.then_some(stored)
}

fn too_large(amount: Paisa) -> bool {
    amount.as_paisa() > MAX_LINE_PAISA
}

/// Maximum of one amount, in rupees, for the error message.
fn max_rupees() -> i64 {
    MAX_LINE_PAISA.checked_div(100).unwrap_or(i64::MAX)
}

/// C1: the value of a line at one rate. By weight: `round(kg × rate per kg)`; otherwise
/// `qty × rate`. Returns the amount and its working.
fn value_at(
    qty: Qty,
    weight: Option<Grams>,
    rate: Paisa,
) -> Result<(Paisa, mandi_core::rules::Working), CalcError> {
    match weight {
        Some(grams) => {
            let value = weight_value(grams, rate)?;
            Ok((value, working::weight_value(grams.get(), rate, value)))
        }
        None => {
            let value = line_value(qty, rate)?;
            Ok((value, working::line_value(qty.get(), rate, value)))
        }
    }
}

/// The charge of one side and kind for one line, from the rule that wins or the value typed on
/// the line (PRD 11.3). A seller or buyer commission with no rule at all is broken data, never 0.
#[allow(clippy::too_many_arguments)]
fn charge(
    args: &mut LineArgs<'_>,
    side: ChargeSide,
    kind: ChargeType,
    party: i64,
    base: Paisa,
    qty: Qty,
    typed: Option<ChargeValue>,
    field: &str,
    issues: &mut Issues,
) -> Result<LineCharge, CalcError> {
    // A typed value counts only while the Owner allows it (PRD FR-11.10). When it is off, the
    // typed value is refused at its field and the rule decides the number shown meanwhile.
    let line_override = match typed {
        Some(value) if args.env.switches.allow_line_override => Some(LineOverride { value }),
        Some(_) => {
            issues.error(ErrorCode::ErrOverrideNotAllowed, field);
            None
        }
        None => None,
    };
    let found = resolve(
        args.env.rules,
        &Query {
            side,
            charge: kind,
            party_id: Some(party),
            item_id: args.lot.item_id,
            item_group: Some(args.lot.item.group),
            date: args.env.date,
            line_override,
        },
    );
    // PRD FR-16.7: for the goods of a separate khata every seller-side default is 0, so a seller
    // commission with no rule of the khata's own is 0, not broken data.
    let khata_default = side == ChargeSide::Seller
        && args.lot.seller_is_khata
        && found
            .winner
            .as_ref()
            .is_some_and(|w| w.level == RuleLevel::Default);
    let winner = found.winner.filter(|_| !khata_default);
    let Some(winner) = winner else {
        if kind == ChargeType::Commission && !khata_default {
            return Err(CalcError::NoRule { side, charge: kind });
        }
        return Ok(no_charge());
    };
    // A per-lot commission is charged once per lot, on the first line that follows the rule
    // (PRD FR-11.9, money agent review point 2), and only in the first posting.
    // Only the seller side has a per-lot charge (PRD 11.2); a bad rule can never take the flag.
    let per_lot_due = matches!(winner.value, ChargeValue::PerLot(_))
        && side == ChargeSide::Seller
        && args.env.kind == PostingKind::First
        && !*args.per_lot_charged;
    if per_lot_due {
        *args.per_lot_charged = true;
    }
    let charged = charge_amount(Some(&winner), kind, base, qty, per_lot_due)?;
    line_charge_of(&winner, charged.amount, charged.working)
}

/// A charge of a line the user did not change (PRD FR-12.20, FR-11.6): its stored method, value
/// and rule, worked out again only to make its working, and checked to give the stored amount. A
/// per-lot commission keeps what it had: the carrier its amount, the other lines 0.
fn stored_charge(
    args: &mut LineArgs<'_>,
    side: ChargeSide,
    kind: ChargeType,
    stored: &StoredCharge,
    base: Paisa,
    qty: Qty,
) -> Result<LineCharge, CalcError> {
    let Some(winner) = stored.winner() else {
        // Nothing applied when it was saved, so it is still 0.
        if stored.amount != Paisa::ZERO {
            return Err(CalcError::Inconsistent(
                "a stored charge has an amount but no value",
            ));
        }
        return Ok(no_charge());
    };
    let carries = stored.carries_per_lot()
        && side == ChargeSide::Seller
        && args.env.kind == PostingKind::First;
    if carries {
        *args.per_lot_charged = true;
    }
    let charged = charge_amount(Some(&winner), kind, base, qty, carries)?;
    if charged.amount != stored.amount {
        return Err(CalcError::Inconsistent(
            "a stored charge does not match its method and value",
        ));
    }
    line_charge_of(&winner, charged.amount, charged.working)
}

/// The per-lot seller commission of a line that an edit removed, now carried by this line (PRD
/// FR-11.9): its stored method, value and rule, with no new rule lookup, so the lot is still
/// charged exactly once.
fn moved_charge(
    args: &mut LineArgs<'_>,
    moved: &StoredCharge,
    base: Paisa,
    qty: Qty,
) -> Result<LineCharge, CalcError> {
    let winner = moved.winner().ok_or(CalcError::Inconsistent(
        "the per-lot commission to move has no value",
    ))?;
    *args.per_lot_charged = true;
    let charged = charge_amount(Some(&winner), ChargeType::Commission, base, qty, true)?;
    line_charge_of(&winner, charged.amount, charged.working)
}

/// A line charge when no rule applies and nothing was typed (a vegetable's لاگا).
fn no_charge() -> LineCharge {
    LineCharge {
        method: StoredMethod::Manual,
        value_bps: None,
        value_paisa: None,
        amount: Paisa::ZERO,
        rule_id: None,
        rule_label: None,
        source: ChargeSource::None,
        // The message of a line charge says the amount is 0 because no rule applies; the
        // lot-charge wording ("typed on the lot") would be wrong here (money agent review,
        // point 1).
        working: working::none_line(),
    }
}

fn line_charge_of(
    winner: &Winner,
    amount: Paisa,
    working: mandi_core::rules::Working,
) -> Result<LineCharge, CalcError> {
    let (bps, paisa) = winner.value.columns();
    let method = if winner.manual {
        StoredMethod::Manual
    } else {
        match winner.value {
            ChargeValue::Percent(_) => StoredMethod::Percent,
            ChargeValue::PerUnit(_) => StoredMethod::PerUnit,
            ChargeValue::PerLot(_) => StoredMethod::PerLot,
        }
    };
    let source = if winner.manual {
        ChargeSource::Typed
    } else if winner.level == RuleLevel::Default {
        ChargeSource::Default
    } else {
        ChargeSource::Rule
    };
    Ok(LineCharge {
        method,
        value_bps: bps,
        value_paisa: paisa.map(Paisa::try_from_paisa).transpose()?,
        amount,
        rule_id: winner.rule_id,
        rule_label: Some(RuleLabel {
            level: winner.level,
            rule_id: winner.rule_id,
        }),
        source,
        working,
    })
}

/// Works out one line: the amounts C1, the three charges, C5 and C19, and the payment type. The
/// caller has checked the buyer, the payment type and the one-time person. Returns `None` (with
/// the error pushed) when a number cannot be worked out.
///
/// Example (PRD 4.4): 20 × 600 for Akram gives value 12,000, seller commission 720, buyer
/// commission 720, لاگا 160 and buyer total 12,880.
pub(super) fn calc_line(
    mut args: LineArgs<'_>,
    issues: &mut Issues,
    prefix: &str,
) -> Result<Option<LineCalc>, CalcError> {
    let row = args.row;
    let (Some(qty), Some(seller_rate)) = (row.qty, row.seller_rate) else {
        return Ok(None);
    };
    let by_weight = args.lot.item.by_weight;
    // A weight typed for an item sold by نگ is ignored (money agent review, point 1).
    let weight = if by_weight { row.weight } else { None };
    if by_weight && weight.is_none() {
        issues.error(ErrorCode::ErrWeightRequired, &format!("{prefix}.weight"));
        return Ok(None);
    }
    let buyer_rate = if row.rate_differs {
        match row.buyer_rate {
            Some(rate) => rate,
            None => return Ok(None),
        }
    } else {
        seller_rate
    };
    let (seller_amount, seller_amount_working) = value_at(qty, weight, seller_rate)?;
    let (buyer_value, buyer_value_working) = value_at(qty, weight, buyer_rate)?;
    for (amount, field) in [(seller_amount, "seller_rate"), (buyer_value, "buyer_rate")] {
        let field = if field == "buyer_rate" && !row.rate_differs {
            "seller_rate"
        } else {
            field
        };
        if too_large(amount) {
            issues.error_with(
                ErrorCode::ErrAmountTooLarge,
                &format!("{prefix}.{field}"),
                &[("max", max_rupees())],
            );
            return Ok(None);
        }
        // A weight so small that the value rounds to 0 is not a sale (PRD E24).
        if by_weight && amount == Paisa::ZERO {
            issues.error(
                ErrorCode::ErrMustBeMoreThanZero,
                &format!("{prefix}.weight"),
            );
            return Ok(None);
        }
    }

    // The per-lot commission is spent only if this line is worked out; if the line is refused
    // below, the next line carries it (money agent review, F2).
    let per_lot_before = *args.per_lot_charged;
    let pending_before = args.pending_move.clone();
    let (lot_seller, buyer_id) = (args.lot.seller_id, args.buyer_id);
    // A removed carrier's per-lot commission goes to the first line the user did not edit that was
    // saved with the per-lot method (PRD FR-11.9, FR-12.20: the one change allowed on an unedited
    // line). A line the user typed a commission on cannot carry it, and neither can an unchanged
    // line whose own stored commission is a percent or a per-نگ amount: that amount would be
    // thrown away (money agent review of plan 16, finding 1). A changed or new line looks its rules
    // up again. Nothing takes the charge twice: once any line carries it, it is spent.
    let takes_move = !*args.per_lot_charged
        && args
            .stored
            .is_some_and(|s| s.seller_comm.method == StoredMethod::PerLot);
    let moved = if row.seller_comm.is_none() && args.env.kind == PostingKind::First && takes_move {
        args.pending_move.take()
    } else {
        None
    };
    let seller_comm = if let Some(moved) = &moved {
        moved_charge(&mut args, moved, seller_amount, qty)?
    } else if let Some(stored) = args.stored {
        // A line the user did not change keeps what it had (PRD FR-12.20).
        stored_charge(
            &mut args,
            ChargeSide::Seller,
            ChargeType::Commission,
            &stored.seller_comm,
            seller_amount,
            qty,
        )?
    } else {
        charge(
            &mut args,
            ChargeSide::Seller,
            ChargeType::Commission,
            lot_seller,
            seller_amount,
            qty,
            row.seller_comm,
            &format!("{prefix}.seller_comm"),
            issues,
        )?
    };
    let (buyer_comm, laga) = if let Some(stored) = args.stored {
        (
            stored_charge(
                &mut args,
                ChargeSide::Buyer,
                ChargeType::Commission,
                &stored.buyer_comm,
                buyer_value,
                qty,
            )?,
            stored_charge(
                &mut args,
                ChargeSide::Buyer,
                ChargeType::Laga,
                &stored.laga,
                buyer_value,
                qty,
            )?,
        )
    } else {
        (
            charge(
                &mut args,
                ChargeSide::Buyer,
                ChargeType::Commission,
                buyer_id,
                buyer_value,
                qty,
                row.buyer_comm,
                &format!("{prefix}.buyer_comm"),
                issues,
            )?,
            charge(
                &mut args,
                ChargeSide::Buyer,
                ChargeType::Laga,
                buyer_id,
                buyer_value,
                qty,
                row.laga,
                &format!("{prefix}.laga"),
                issues,
            )?,
        )
    };

    // C5 and C19.
    let buyer_total = buyer_value
        .checked_add(buyer_comm.amount)?
        .checked_add(laga.amount)?;
    let rate_diff = buyer_value.checked_sub(seller_amount)?;
    if too_large(buyer_total) {
        *args.per_lot_charged = per_lot_before;
        *args.pending_move = pending_before;
        issues.error_with(
            ErrorCode::ErrAmountTooLarge,
            &format!("{prefix}.seller_rate"),
            &[("max", max_rupees())],
        );
        return Ok(None);
    }
    let (cost_of_goods, cost_working) = args
        .cost
        .clone()
        .unwrap_or_else(|| (Paisa::ZERO, working::none_line()));
    let workings = LineWorkings {
        seller_amount: seller_amount_working,
        buyer_value: buyer_value_working,
        buyer_total: working::buyer_total(buyer_value, buyer_comm.amount, laga.amount, buyer_total),
        rate_diff: working::rate_diff(buyer_value, seller_amount, rate_diff),
        cost_of_goods: cost_working,
    };

    // The checks that need the numbers: credit limit (PRD FR-12.4) and rate check (FR-7.55).
    if args.payment == PaymentType::Credit {
        checks::credit_limit(
            issues,
            args.owed,
            buyer_id,
            args.buyer,
            buyer_total,
            args.env.switches.credit_limit_blocks,
            &format!("{prefix}.buyer_account_id"),
        )?;
    }
    rate_check(&args, issues, prefix, seller_rate, buyer_rate)?;

    Ok(Some(LineCalc {
        qty: qty.get(),
        weight_grams: weight.map(Grams::get),
        seller_rate,
        buyer_rate,
        seller_amount,
        buyer_value,
        seller_comm,
        buyer_comm,
        laga,
        buyer_total,
        rate_diff,
        cost_of_goods,
        payment_type: args.payment,
        workings,
    }))
}

/// Warns when the seller rate, or the buyer rate if it differs, is far from the item's average
/// of the last 7 days. Skipped for an item sold by weight (its rate is per kg, the facts hold
/// an amount per نگ) and when the Owner switched the check off. Never blocks.
fn rate_check(
    args: &LineArgs<'_>,
    issues: &mut Issues,
    prefix: &str,
    seller_rate: Paisa,
    buyer_rate: Paisa,
) -> Result<(), CalcError> {
    let Some(percent) = args.env.switches.rate_check_percent else {
        return Ok(());
    };
    if args.lot.item.by_weight {
        return Ok(());
    }
    let Some(sales) = args.env.lookups.item_sales.get(&args.lot.item_id) else {
        return Ok(());
    };
    let mut rates = vec![("seller_rate", seller_rate)];
    if args.row.rate_differs {
        rates.push(("buyer_rate", buyer_rate));
    }
    for (name, rate) in rates {
        if let Some(average) = checks::far_from_average(rate, *sales, percent)? {
            issues.warn_with(
                ErrorCode::ErrRateFarFromAverage,
                &format!("{prefix}.{name}"),
                &[
                    ("average_paisa", average.as_paisa()),
                    ("percent", i64::from(percent)),
                ],
            );
        }
    }
    Ok(())
}
