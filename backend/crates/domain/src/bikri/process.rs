//! One lot of a draft (PRD 12.3): its checks, its rows, and its money. Called by
//! [`calc_posting`](super::calc_posting) for each lot in order.

use std::collections::HashMap;

use mandi_core::bikri::{
    FreightPaidBy, LinePreview, LotCharge, LotPreview, PaymentType, PreparedLot, PreparedSale,
    RateHint, RateHintSource, SummaryTotals,
};
use mandi_core::error::ErrorCode;
use mandi_core::ids::AccountId;
use mandi_core::money::{Paisa, Qty};
use mandi_core::rules::ChargeType;

use super::checks::{self, Issues, Wanted};
use super::line::{LineArgs, LotRef, calc_line};
use super::lookups::{AccountFacts, ItemFacts, Lookups};
use super::lot::{Acc, LotKey, Scope, lot_charge};
use super::{CalcError, Env, PostingKind};
use crate::ledger::MAX_LINE_PAISA;
use crate::ledger::bikri::{JournalLot, JournalSale};

/// State carried from lot to lot.
pub(super) struct Run {
    /// What each buyer would owe, for the credit limit.
    pub owed: HashMap<i64, Paisa>,
    /// The last seller rate typed for each (item, packing), for the rate hint.
    pub last_rates: HashMap<(i64, i64), Paisa>,
}

/// What one lot gives back to the whole.
pub(super) struct LotOutcome {
    pub preview: LotPreview,
    pub acc: Option<Acc>,
    pub journal: Option<JournalLot>,
}

/// What a lot is worked out from.
struct LotCtx<'a> {
    env: Option<&'a Env<'a>>,
    lookups: &'a Lookups,
    date_iso: &'a str,
    index: usize,
    prefix: String,
    lot: &'a PreparedLot,
    /// Set only when the lot is complete: then its rows can be worked out.
    lot_ref: Option<LotRef>,
    facts: LotFacts<'a>,
}

fn too_large(issues: &mut Issues, field: &str) {
    let max = MAX_LINE_PAISA.checked_div(100).unwrap_or(i64::MAX);
    issues.error_with(ErrorCode::ErrAmountTooLarge, field, &[("max", max)]);
}

/// The hint under an empty rate cell (PRD FR-7.53): the nearest earlier line of this بکری with
/// the same item and packing, else the saved rate of today or the last day it sold.
fn rate_hint(
    run: &Run,
    lookups: &Lookups,
    key: Option<(i64, i64)>,
    date_iso: &str,
) -> Option<RateHint> {
    let key = key?;
    if let Some(rate) = run.last_rates.get(&key) {
        return Some(RateHint {
            rate_paisa: *rate,
            source: RateHintSource::ThisBikri,
            date: date_iso.to_owned(),
        });
    }
    lookups.saved_hints.get(&key).cloned()
}

/// The facts of a lot that are needed to work out its rows.
struct LotFacts<'a> {
    seller: Option<&'a AccountFacts>,
    item: Option<ItemFacts>,
    complete: bool,
}

fn lot_facts<'a>(
    issues: &mut Issues,
    lookups: &'a Lookups,
    env_present: bool,
    prefix: &str,
    lot: &PreparedLot,
) -> LotFacts<'a> {
    let mut complete = env_present && !lot.has_errors && lot.qty_received.is_some();
    let seller = checks::account(
        issues,
        lookups,
        lot.seller_account_id,
        Wanted::Seller,
        &format!("{prefix}.seller_account_id"),
        Some(ErrorCode::ErrSellerRequired),
    );
    match seller {
        Some(facts) if facts.is_pukhta => {
            // A one-time seller needs his name and phone (PRD FR-13.2).
            complete &= checks::pukhta(issues, prefix, &lot.pukhta, true);
        }
        Some(_) => {}
        None => complete = false,
    }
    let item = match lot.item_id {
        None => {
            issues.error(ErrorCode::ErrRequired, &format!("{prefix}.item_id"));
            None
        }
        Some(id) => {
            let found = lookups.items.get(&id).copied();
            if found.is_none() {
                issues.error(ErrorCode::ErrNotFound, &format!("{prefix}.item_id"));
            }
            found
        }
    };
    complete &= item.is_some();
    match lot.packing_id {
        None => {
            issues.error(ErrorCode::ErrRequired, &format!("{prefix}.packing_id"));
            complete = false;
        }
        Some(_) => {
            complete &= checks::master_exists(
                issues,
                &lookups.packings,
                lot.packing_id,
                &format!("{prefix}.packing_id"),
            );
        }
    }
    complete &= checks::master_exists(
        issues,
        &lookups.markas,
        lot.marka_id,
        &format!("{prefix}.marka_id"),
    );
    complete &= checks::master_exists(
        issues,
        &lookups.stores,
        lot.store_id,
        &format!("{prefix}.store_id"),
    );
    if item.is_some_and(|i| i.by_weight) && lot.weight_received.is_none() {
        issues.error(
            ErrorCode::ErrWeightRequired,
            &format!("{prefix}.weight_received"),
        );
        complete = false;
    }
    LotFacts {
        seller,
        item,
        complete,
    }
}

/// The rows of a lot as worked out.
struct Rows {
    lines: Vec<LinePreview>,
    acc: Acc,
    sales: Vec<JournalSale>,
    /// Quantity of the rows that have a valid quantity (blank rows left out).
    sold: i64,
    any_row: bool,
}

/// The buyer of a row, how he pays, and whether his one-time person is in order (PRD 12.4,
/// FR-13.2). `None` if the row cannot be worked out.
fn row_parties<'a>(
    ctx: &LotCtx<'a>,
    issues: &mut Issues,
    row: &PreparedSale,
    line_prefix: &str,
) -> Option<(&'a AccountFacts, i64, PaymentType)> {
    let buyer = checks::account(
        issues,
        ctx.lookups,
        row.buyer_account_id,
        Wanted::Buyer,
        &format!("{line_prefix}.buyer_account_id"),
        Some(ErrorCode::ErrBuyerRequired),
    );
    let payment = buyer.and_then(|b| {
        checks::payment_type(
            issues,
            b,
            row.payment_type,
            &format!("{line_prefix}.payment_type"),
        )
    });
    let person_ok = match buyer {
        Some(b) if b.is_pukhta => checks::pukhta(
            issues,
            line_prefix,
            &row.pukhta,
            payment == Some(PaymentType::Credit),
        ),
        _ => true,
    };
    match (buyer, row.buyer_account_id, payment, person_ok) {
        (Some(buyer), Some(id), Some(payment), true) => Some((buyer, id, payment)),
        _ => None,
    }
}

/// The money of a worked row as the journal needs it.
fn journal_sale(buyer_id: i64, calc: &mandi_core::bikri::LineCalc) -> JournalSale {
    JournalSale {
        buyer_account_id: AccountId::new(buyer_id),
        person_key: None,
        buyer_total: calc.buyer_total,
        payment_type: calc.payment_type,
        seller_comm: calc.seller_comm.amount,
        buyer_comm: calc.buyer_comm.amount,
        laga: calc.laga.amount,
        rate_diff: calc.rate_diff,
    }
}

/// Goes through the rows of a lot in order: remaining quantity, hint, checks and numbers.
fn process_rows(ctx: &LotCtx<'_>, run: &mut Run, issues: &mut Issues) -> Result<Rows, CalcError> {
    let lot = ctx.lot;
    let received = lot.qty_received.map(Qty::get);
    let hint_key = lot.item_id.zip(lot.packing_id);
    let mut per_lot_charged = false;
    let mut rows = Rows {
        lines: Vec::with_capacity(lot.sales.len()),
        acc: Acc::default(),
        sales: Vec::new(),
        sold: 0,
        any_row: false,
    };
    for (j, row) in lot.sales.iter().enumerate() {
        let line_prefix = format!("{}.sales[{j}]", ctx.prefix);
        let remaining = received.map_or(0, |r| r.saturating_sub(rows.sold).max(0));
        let mut preview = LinePreview {
            key: row.key.clone(),
            line_no: i64::try_from(j).unwrap_or(i64::MAX).saturating_add(1),
            is_blank: row.is_blank,
            remaining_qty: remaining,
            rate_hint: rate_hint(run, ctx.lookups, hint_key, ctx.date_iso),
            calc: None,
            warnings: Vec::new(),
        };
        if row.is_blank {
            rows.lines.push(preview);
            continue;
        }
        rows.any_row = true;
        if let (Some(r), Some(q)) = (received, row.qty)
            && rows.sold.saturating_add(q.get()) > r
        {
            issues.error_with(
                ErrorCode::ErrQtyExceedsLot,
                &format!("{line_prefix}.qty"),
                &[("available", remaining)],
            );
        }
        let parties = row_parties(ctx, issues, row, &line_prefix);
        if let (Some(env), Some(lot_ref), Some((buyer, buyer_id, payment)), false) =
            (ctx.env, ctx.lot_ref.as_ref(), parties, row.has_errors)
        {
            preview.calc = calc_line(
                LineArgs {
                    env,
                    lot: lot_ref,
                    per_lot_charged: &mut per_lot_charged,
                    owed: &mut run.owed,
                    row,
                    buyer_id,
                    buyer,
                    payment,
                },
                issues,
                &line_prefix,
            )?;
        }
        if let (Some(calc), Some((_, buyer_id, _))) = (&preview.calc, parties) {
            rows.acc.add_line(calc)?;
            rows.sales.push(journal_sale(buyer_id, calc));
        }
        if let Some(q) = row.qty {
            rows.sold = rows.sold.saturating_add(q.get());
        }
        if let (Some(rate), Some(key)) = (row.seller_rate, hint_key) {
            run.last_rates.insert(key, rate);
        }
        rows.lines.push(preview);
    }
    Ok(rows)
}

/// The charges, totals and journal of a complete lot.
struct Money {
    labour: Option<LotCharge>,
    munshiana: Option<LotCharge>,
    phone: Option<LotCharge>,
    totals: Option<SummaryTotals>,
    acc: Option<Acc>,
    journal: Option<JournalLot>,
}

/// The lot charges, then (if a row was worked out) the totals C6 to C9 and the journal lot.
fn lot_money(ctx: &LotCtx<'_>, rows: &mut Rows, issues: &mut Issues) -> Result<Money, CalcError> {
    let mut money = Money {
        labour: None,
        munshiana: None,
        phone: None,
        totals: None,
        acc: None,
        journal: None,
    };
    let (Some(env), Some(lot_ref), Some(qty_received)) =
        (ctx.env, ctx.lot_ref.as_ref(), ctx.lot.qty_received)
    else {
        return Ok(money);
    };
    let lot = ctx.lot;
    let key = LotKey {
        seller_id: lot_ref.seller_id,
        item_id: lot_ref.item_id,
        item: lot_ref.item,
        qty_received,
    };
    let labour = lot_charge(env, &key, ChargeType::Labour, lot.labour)?;
    let munshiana = lot_charge(env, &key, ChargeType::Munshiana, lot.munshiana)?;
    let phone = lot_charge(env, &key, ChargeType::Phone, lot.phone)?;
    for (charge, name) in [
        (&labour, "labour"),
        (&munshiana, "munshiana"),
        (&phone, "phone"),
    ] {
        if charge.amount.as_paisa() > MAX_LINE_PAISA {
            too_large(issues, &format!("{}.{name}", ctx.prefix));
        }
    }
    // A lot with rows needs one worked-out row before it has totals. A lot with no row at all
    // (received, nothing sold today) has totals too: its charges and a net that may be negative
    // (PRD E3, FR-12.7, tech-lead decision on FR-12.8 check 1).
    if rows.acc.count > 0 || !rows.any_row {
        let acc = &mut rows.acc;
        // Freight, store rent and the lot charges belong to the first posting only (PRD
        // FR-12.17). `lot_charge` already gives 0 for a later sale; so do these two.
        let first = env.kind == PostingKind::First;
        acc.freight = match (first, lot.freight_paid_by) {
            (true, FreightPaidBy::Firm) => lot.freight,
            _ => Paisa::ZERO,
        };
        acc.labour = labour.amount;
        acc.munshiana = munshiana.amount;
        acc.phone = phone.amount;
        acc.store_rent = if first { lot.store_rent } else { Paisa::ZERO };
        // نقد: as typed; a one-time seller is paid his whole net on the spot (PRD FR-13.10).
        let pays_on_the_spot =
            lot.cash_paid.is_none() && ctx.facts.seller.is_some_and(|s| s.is_pukhta);
        let credit = acc.seller_credit()?;
        acc.cash_paid = match lot.cash_paid {
            Some(cash) => cash,
            None if pays_on_the_spot && credit > Paisa::ZERO => credit,
            None => Paisa::ZERO,
        };
        let totals = acc.totals(&Scope::Lot {
            labour: &labour.working,
            munshiana: &munshiana.working,
            phone: &phone.working,
            cash_is_default: pays_on_the_spot,
        })?;
        let too_big = [totals.gross, totals.buyer_total, totals.seller_credit]
            .iter()
            .any(|a| a.as_paisa().saturating_abs() > MAX_LINE_PAISA);
        if too_big {
            too_large(issues, &format!("{}.sales", ctx.prefix));
        } else {
            if totals.net.is_negative() {
                issues.warn_with(
                    ErrorCode::ErrLotNetNegative,
                    &format!("{}.seller_account_id", ctx.prefix),
                    &[("net_paisa", totals.net.as_paisa())],
                );
            }
            money.journal = Some(JournalLot {
                seller_account_id: AccountId::new(lot_ref.seller_id),
                person_key: None,
                seller_credit: totals.seller_credit,
                cash_paid: totals.cash_paid,
                freight_deducted: totals.freight,
                labour: totals.labour,
                munshiana: totals.munshiana,
                store_rent: totals.store_rent,
                phone: totals.phone,
                sales: std::mem::take(&mut rows.sales),
            });
            money.acc = Some(*acc);
            money.totals = Some(totals);
        }
    }
    money.labour = Some(labour);
    money.munshiana = Some(munshiana);
    money.phone = Some(phone);
    Ok(money)
}

/// Works out one lot: its facts, its rows, its charges and totals, and its warnings.
pub(super) fn process_lot(
    env: Option<&Env<'_>>,
    lookups: &Lookups,
    run: &mut Run,
    date_iso: &str,
    index: usize,
    lot: &PreparedLot,
    issues: &mut Issues,
) -> Result<LotOutcome, CalcError> {
    let prefix = format!("lots[{index}]");
    let facts = lot_facts(issues, lookups, env.is_some(), &prefix, lot);
    let lot_ref = match (&facts, lot.seller_account_id, lot.item_id) {
        (
            LotFacts {
                item: Some(item),
                complete: true,
                ..
            },
            Some(seller_id),
            Some(item_id),
        ) => Some(LotRef {
            seller_id,
            item_id,
            item: *item,
        }),
        _ => None,
    };
    let ctx = LotCtx {
        env,
        lookups,
        date_iso,
        index,
        prefix,
        lot,
        lot_ref,
        facts,
    };
    let mut rows = process_rows(&ctx, run, issues)?;
    let balance_qty = lot
        .qty_received
        .map(|r| r.get().saturating_sub(rows.sold).max(0));
    if let Some(left) = balance_qty
        && left > 0
    {
        issues.warn_with(
            ErrorCode::ErrQtyLeftUnsold,
            &format!("{}.qty_received", ctx.prefix),
            &[("left", left)],
        );
    }
    let money = lot_money(&ctx, &mut rows, issues)?;
    let net_is_negative = money.totals.as_ref().is_some_and(|t| t.net.is_negative());
    Ok(LotOutcome {
        preview: LotPreview {
            key: lot.key.clone(),
            lot_no: i64::try_from(ctx.index)
                .unwrap_or(i64::MAX)
                .saturating_add(1),
            freight: lot.freight,
            freight_paid_by: lot.freight_paid_by,
            sold_by_weight: ctx.facts.item.is_some_and(|item| item.by_weight),
            labour: money.labour,
            munshiana: money.munshiana,
            phone: money.phone,
            sold_qty: rows.sold,
            balance_qty,
            net_is_negative,
            totals: money.totals,
            lines: rows.lines,
        },
        acc: money.acc,
        journal: money.journal,
    })
}
