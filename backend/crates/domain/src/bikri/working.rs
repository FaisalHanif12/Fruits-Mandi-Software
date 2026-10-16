//! The `working` records of the amounts of a بکری (PRD FR-7.43, Code Standards B30). Each is
//! built from the same values as the amount, so its `result` is the amount.
//!
//! The message ids are the keys of the Fluent files. The charges (commission, لاگا, lot charges)
//! use the messages of `mandi_domain::rules::charge_amount`.

use mandi_core::money::Paisa;
use mandi_core::rules::{Working, WorkingArg, WorkingValue};

pub(super) fn money(name: &str, paisa: Paisa) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Money { paisa },
    }
}

pub(super) fn count(name: &str, value: i64) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Count { value },
    }
}

fn working(message_id: &str, args: Vec<WorkingArg>) -> Working {
    Working {
        message_id: message_id.to_owned(),
        args,
        rule_id: None,
    }
}

/// C1 by نگ: `qty × rate`.
pub(super) fn line_value(qty: i64, rate: Paisa, result: Paisa) -> Working {
    working(
        "working-line-value",
        vec![
            count("qty", qty),
            money("rate", rate),
            money("result", result),
        ],
    )
}

/// C1 by weight: `kg × rate per kg`, rounded to the rupee.
pub(super) fn weight_value(grams: i64, rate_per_kg: Paisa, result: Paisa) -> Working {
    working(
        "working-weight-value",
        vec![
            count("grams", grams),
            money("rate", rate_per_kg),
            money("result", result),
        ],
    )
}

/// C5.
pub(super) fn buyer_total(value: Paisa, comm: Paisa, laga: Paisa, result: Paisa) -> Working {
    working(
        "working-buyer-total",
        vec![
            money("value", value),
            money("comm", comm),
            money("laga", laga),
            money("result", result),
        ],
    )
}

/// C19.
pub(super) fn rate_diff(buyer_value: Paisa, seller_amount: Paisa, result: Paisa) -> Working {
    working(
        "working-rate-diff",
        vec![
            money("buyer_value", buyer_value),
            money("seller_amount", seller_amount),
            money("result", result),
        ],
    )
}

/// A sum of `count` lines (or lots).
pub(super) fn sum(items: i64, result: Paisa) -> Working {
    working(
        "working-sum",
        vec![count("count", items), money("result", result)],
    )
}

/// An amount typed by hand (freight, نقد, کرایہ سٹور of a lot).
pub(super) fn typed(result: Paisa) -> Working {
    working("working-manual", vec![money("result", result)])
}

/// No rule applies to a charge of a sale line, so it is 0.
pub(super) fn none_line() -> Working {
    working("working-none-line", vec![money("result", Paisa::ZERO)])
}

/// A one-time seller is paid his whole net on the spot (PRD FR-13.10).
pub(super) fn pukhta_cash(result: Paisa) -> Working {
    working("working-cash-pukhta", vec![money("result", result)])
}

/// C6.
#[allow(clippy::too_many_arguments)]
pub(super) fn deductions(
    freight: Paisa,
    comm: Paisa,
    labour: Paisa,
    munshiana: Paisa,
    cash: Paisa,
    store_rent: Paisa,
    phone: Paisa,
    result: Paisa,
) -> Working {
    working(
        "working-deductions",
        vec![
            money("freight", freight),
            money("comm", comm),
            money("labour", labour),
            money("munshiana", munshiana),
            money("cash", cash),
            money("store_rent", store_rent),
            money("phone", phone),
            money("result", result),
        ],
    )
}

/// C7.
pub(super) fn net(gross: Paisa, deductions: Paisa, result: Paisa) -> Working {
    working(
        "working-net",
        vec![
            money("gross", gross),
            money("deductions", deductions),
            money("result", result),
        ],
    )
}

/// C8.
#[allow(clippy::too_many_arguments)]
pub(super) fn seller_credit(
    gross: Paisa,
    comm: Paisa,
    freight: Paisa,
    labour: Paisa,
    munshiana: Paisa,
    store_rent: Paisa,
    phone: Paisa,
    result: Paisa,
) -> Working {
    working(
        "working-seller-credit",
        vec![
            money("gross", gross),
            money("comm", comm),
            money("freight", freight),
            money("labour", labour),
            money("munshiana", munshiana),
            money("store_rent", store_rent),
            money("phone", phone),
            money("result", result),
        ],
    )
}

/// C9.
pub(super) fn shared_income(
    seller_comm: Paisa,
    buyer_comm: Paisa,
    laga: Paisa,
    phone: Paisa,
    rate_diff: Paisa,
    result: Paisa,
) -> Working {
    working(
        "working-shared-income",
        vec![
            money("seller_comm", seller_comm),
            money("buyer_comm", buyer_comm),
            money("laga", laga),
            money("phone", phone),
            money("rate_diff", rate_diff),
            money("result", result),
        ],
    )
}

/// A partner's part of the shared income (PRD C10). `result` is the part the split gave, which
/// for the largest share includes the leftover rupees; the message must not claim that
/// `base × bps` equals it.
///
/// Example: base 3,948 at 4,000 bps gives 1,580.
pub fn partner_share(base: Paisa, bps: i32, result: Paisa) -> Working {
    working(
        "working-partner-share",
        vec![
            money("base", base),
            WorkingArg {
                name: "bps".to_owned(),
                value: WorkingValue::Percent { bps },
            },
            money("result", result),
        ],
    )
}
