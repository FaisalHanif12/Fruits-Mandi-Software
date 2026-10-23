//! The checks of a بکری draft that need the world outside it (PRD FR-12.4 to FR-12.7, FR-13.2,
//! FR-7.55): accounts, one-time people, payment type, credit limit and the rate check.

use std::collections::HashMap;

use mandi_core::bikri::{PaymentType, PreparedPukhta};
use mandi_core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_core::money::{MoneyError, Paisa, Qty, avg_rate};
use mandi_core::validate;

use super::lookups::{AccountFacts, ItemSales, Lookups, PartyKind};

/// Errors and warnings found while working out a draft, each at its field.
#[derive(Debug, Default)]
pub(super) struct Issues {
    pub errors: Vec<FieldError>,
    pub warnings: Vec<FieldError>,
}

impl Issues {
    pub(super) fn error(&mut self, code: ErrorCode, field: &str) {
        self.errors.push(FieldError::new(code, field));
    }

    pub(super) fn error_with(&mut self, code: ErrorCode, field: &str, values: &[(&str, i64)]) {
        self.errors.push(with_values(code, field, values));
    }

    pub(super) fn warn_with(&mut self, code: ErrorCode, field: &str, values: &[(&str, i64)]) {
        self.warnings.push(with_values(code, field, values));
    }
}

fn with_values(code: ErrorCode, field: &str, values: &[(&str, i64)]) -> FieldError {
    values
        .iter()
        .fold(FieldError::new(code, field), |e, &(k, v)| {
            e.with(k, ErrorValue::Number(v))
        })
}

/// What kind of party a field wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Wanted {
    /// A seller. The اپنا مال account is refused (`ERR_OWN_GOODS_ACCOUNT_RESTRICTED`).
    Seller,
    /// The seller of a lot: a seller, or the اپنا مال account for a lot of own goods.
    LotSeller,
    /// A buyer.
    Buyer,
}

/// Looks an account up for a field and checks that it is the right kind and active (PRD FR-10.2).
/// `required` is the code for an empty field.
pub(super) fn account<'a>(
    issues: &mut Issues,
    lookups: &'a Lookups,
    id: Option<i64>,
    wanted: Wanted,
    field: &str,
    required: Option<ErrorCode>,
) -> Option<&'a AccountFacts> {
    let Some(id) = id else {
        if let Some(code) = required {
            issues.error(code, field);
        }
        return None;
    };
    let Some(facts) = lookups.accounts.get(&id) else {
        issues.error(ErrorCode::ErrNotFound, field);
        return None;
    };
    // The اپنا مال account is offered only as the seller of a lot (PRD FR-12.26, FR-10.4).
    if facts.is_own_goods && wanted != Wanted::LotSeller {
        issues.error(ErrorCode::ErrOwnGoodsAccountRestricted, field);
        return None;
    }
    let kind_ok = match wanted {
        Wanted::Seller => facts.kind == PartyKind::Seller,
        // A separate khata can sell its goods through a lot (PRD FR-16.7).
        Wanted::LotSeller => matches!(facts.kind, PartyKind::Seller | PartyKind::Khata),
        Wanted::Buyer => facts.kind == PartyKind::Buyer,
    };
    if !kind_ok {
        let code = match wanted {
            Wanted::Seller | Wanted::LotSeller => ErrorCode::ErrNotASeller,
            Wanted::Buyer => ErrorCode::ErrNotABuyer,
        };
        issues.error(code, field);
        return None;
    }
    if !facts.is_active {
        issues.error(ErrorCode::ErrAccountInactive, field);
        return None;
    }
    Some(facts)
}

/// A master (packing, مارکہ, store, truck stand) that must exist when it is given.
pub(super) fn master_exists(
    issues: &mut Issues,
    set: &std::collections::HashSet<i64>,
    id: Option<i64>,
    field: &str,
) -> bool {
    match id {
        Some(id) if !set.contains(&id) => {
            issues.error(ErrorCode::ErrNotFound, field);
            false
        }
        _ => true,
    }
}

/// The one-time person of a پختہ کھاتہ account (PRD FR-13.2): both names, each in its own
/// script (PRD FR-7.28), and a phone when `need_phone`. A phone that is typed must be valid.
/// `prefix` is the path of the lot or line, for example `lots[0].sales[1]`.
pub(super) fn pukhta(
    issues: &mut Issues,
    prefix: &str,
    person: &PreparedPukhta,
    need_phone: bool,
) -> bool {
    let mut ok = true;
    let name_ur = format!("{prefix}.pukhta.name_ur");
    let name_en = format!("{prefix}.pukhta.name_en");
    let phone = format!("{prefix}.pukhta.phone");
    if person.name_ur.is_empty() {
        issues.error(ErrorCode::ErrPukhtaNameRequired, &name_ur);
        ok = false;
    } else if let Err(e) = validate::name_ur(&name_ur, &person.name_ur) {
        issues.errors.push(e);
        ok = false;
    }
    if person.name_en.is_empty() {
        issues.error(ErrorCode::ErrPukhtaNameRequired, &name_en);
        ok = false;
    } else if let Err(e) = validate::name_en(&name_en, &person.name_en) {
        issues.errors.push(e);
        ok = false;
    }
    if person.phone.is_empty() {
        if need_phone {
            issues.error(ErrorCode::ErrPukhtaPhoneRequired, &phone);
            ok = false;
        }
    } else if let Err(e) = validate::phone(&phone, &person.phone) {
        issues.errors.push(e);
        ok = false;
    }
    ok
}

/// How the buyer of a line pays (PRD 12.4, FR-13.4). Typed credit is refused for the نقدی منڈی
/// account (always cash) and for a buyer whose "ادھار کی اجازت" is off (PRD 10.2: cash only).
/// With nothing chosen: credit if the buyer may have credit, else cash; the one-time buyer
/// defaults to cash. A one-time buyer may take credit (with a phone, checked by [`pukhta`]).
pub(super) fn payment_type(
    issues: &mut Issues,
    buyer: &AccountFacts,
    chosen: Option<PaymentType>,
    field: &str,
) -> Option<PaymentType> {
    let paid = resolve_payment(buyer, chosen);
    if paid.is_none() {
        issues.error(ErrorCode::ErrCreditNotAllowed, field);
    }
    paid
}

/// The payment type of [`payment_type`] without the error: `None` when credit was chosen for a
/// buyer who may only pay cash.
pub(super) fn resolve_payment(
    buyer: &AccountFacts,
    chosen: Option<PaymentType>,
) -> Option<PaymentType> {
    let may_credit = !buyer.is_naqdi_mandi && (buyer.is_pukhta || buyer.credit_allowed);
    match chosen {
        Some(PaymentType::Credit) if !may_credit => None,
        Some(kind) => Some(kind),
        None if buyer.is_pukhta || !may_credit => Some(PaymentType::Cash),
        None => Some(PaymentType::Credit),
    }
}

/// The credit limit check (PRD FR-12.4, FR-7.56). `owed` is a running map: the buyer's signed
/// balance now plus the buyer totals of his credit rows in this draft, in the order of the
/// draft (lot order, then line order). Only a named buyer with a limit is checked, and only when
/// what he would owe is MORE than the limit (exactly at the limit is fine).
///
/// Example: balance 0 and limit 12,879: the first credit row of 12,880 takes him to 12,880,
/// which is above the limit, so it warns (or is an error when the setting is block).
pub(super) fn credit_limit(
    issues: &mut Issues,
    owed: &mut HashMap<i64, Paisa>,
    buyer_id: i64,
    buyer: &AccountFacts,
    buyer_total: Paisa,
    blocks: bool,
    field: &str,
) -> Result<(), MoneyError> {
    let Some(limit) = buyer.credit_limit else {
        return Ok(());
    };
    if buyer.is_pukhta || buyer.is_naqdi_mandi {
        return Ok(());
    }
    let before = *owed.entry(buyer_id).or_insert(buyer.balance);
    let would_owe = before.checked_add(buyer_total)?;
    owed.insert(buyer_id, would_owe);
    if would_owe > limit {
        let values = [
            ("limit_paisa", limit.as_paisa()),
            ("would_owe_paisa", would_owe.as_paisa()),
        ];
        if blocks {
            issues.error_with(ErrorCode::ErrCreditLimitExceeded, field, &values);
        } else {
            issues.warn_with(ErrorCode::ErrCreditLimitExceeded, field, &values);
        }
    }
    Ok(())
}

/// The rate check (PRD FR-7.55, E8), in whole numbers. `sales` are the item's saved sales of the
/// last 7 days: Σ qty = Q and Σ seller amount = A. A rate R is far when it is more than `p`
/// percent above or below the weighted average A ÷ Q:
///
/// - above: `R × Q × 100 > A × (100 + p)`;
/// - below: `R × Q × 100 < A × (100 − p)`, which can only happen when `p` is below 100.
///
/// The average is never rounded before the comparison (money agent review). Returns the average
/// for the message (rounded, display only) if the rate is far, else `None`. No sales (Q ≤ 0 or
/// A ≤ 0) is no check.
///
/// Example: average 1,000 (Q = 10, A = 10,000), p = 30: 1,300 is fine, 1,301 is far; 700 is
/// fine, 699 is far.
pub(super) fn far_from_average(
    rate: Paisa,
    sales: ItemSales,
    percent: u32,
) -> Result<Option<Paisa>, MoneyError> {
    if sales.qty <= 0 || sales.amount <= Paisa::ZERO {
        return Ok(None);
    }
    let times = |a: i128, b: i128| a.checked_mul(b).ok_or(MoneyError::Overflow);
    let lhs = times(
        times(i128::from(rate.as_paisa()), i128::from(sales.qty))?,
        100,
    )?;
    let amount = i128::from(sales.amount.as_paisa());
    let percent = i128::from(percent);
    let above = lhs > times(amount, 100_i128.saturating_add(percent))?;
    let below = percent < 100 && lhs < times(amount, 100_i128.saturating_sub(percent))?;
    if !(above || below) {
        return Ok(None);
    }
    avg_rate(sales.amount, Qty::new(sales.qty)?).map(Some)
}
