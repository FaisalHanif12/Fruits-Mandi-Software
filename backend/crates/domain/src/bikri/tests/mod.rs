//! Tests of `calc_posting`: the worked numbers of PRD 4.4, 4.5 and E2E-16, and the cases the
//! money agent required for plan 12.
//!
//! - `worked`: the PRD days, with the working of every amount checked against the amount.
//! - `cases`: charges, quantities, payment types, credit limit, rate check, weight, limits.
//! - `properties`: random drafts keep the books balanced.

mod cases;
mod edit;
mod own;
mod properties;
mod worked;

use std::collections::{HashMap, HashSet};

use mandi_core::bikri::{
    BikriDraft, ChargeInput, FreightPaidBy, HeaderInput, LineCalc, LotChargeMethod, LotInput,
    LotPreview, SaleInput, prepare_draft,
};
use mandi_core::dates::BusinessDate;
use mandi_core::ids::AccountId;
use mandi_core::money::{Bps, Paisa};
use mandi_core::rules::{ChargeSide, ChargeType, ChargeValue, ItemGroup, Working, WorkingValue};

use super::{
    AccountFacts, ItemFacts, ItemSales, Lookups, PartyKind, PostingCalc, PostingTarget,
    StoredPosting, Switches, calc_posting,
};
use crate::ledger::BalancedEntry;
use crate::ledger::bikri::{SystemAccounts, build_bikri_entry};
use crate::rules::{Rule, RuleSet};

pub const RAHIM: i64 = 1;
pub const KARIM: i64 = 2;
pub const AKRAM: i64 = 3;
pub const ARSHAD: i64 = 4;
pub const NAQDI: i64 = 5;
pub const PUKHTA_BUYER: i64 = 6;
pub const PUKHTA_SELLER: i64 = 7;
pub const OWN_GOODS: i64 = 8;
pub const CASH_ONLY_BUYER: i64 = 9;
pub const KHATA: i64 = 10;
pub const BANANA: i64 = 1;
pub const APPLE: i64 = 2;
pub const MOOLI: i64 = 3;
pub const GRAPES: i64 = 4;
pub const BOX: i64 = 1;

pub fn p(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

pub fn today() -> BusinessDate {
    BusinessDate::from_ymd(2026, 10, 10).unwrap()
}

fn rule(id: i64, side: ChargeSide, charge: ChargeType, value: ChargeValue) -> Rule {
    Rule {
        id,
        side,
        charge,
        account_id: None,
        item_id: None,
        item_group: None,
        value,
        from_date: BusinessDate::from_ymd(2026, 1, 1).unwrap(),
        to_date: None,
        is_active: true,
    }
}

/// The defaults of PRD 4.2: seller 6%, buyer 6%, لاگا 8 per نگ on fruit.
pub fn default_rules() -> Vec<Rule> {
    let pct = |bps| ChargeValue::Percent(Bps::new(bps).unwrap());
    vec![
        rule(1, ChargeSide::Seller, ChargeType::Commission, pct(600)),
        rule(2, ChargeSide::Buyer, ChargeType::Commission, pct(600)),
        Rule {
            item_group: Some(ItemGroup::Fruit),
            ..rule(
                3,
                ChargeSide::Buyer,
                ChargeType::Laga,
                ChargeValue::PerUnit(p(8)),
            )
        },
    ]
}

fn account(kind: PartyKind) -> AccountFacts {
    AccountFacts {
        kind,
        is_pukhta: false,
        is_naqdi_mandi: false,
        is_own_goods: false,
        garden_closed: false,
        is_active: true,
        credit_allowed: true,
        credit_limit: None,
        balance: Paisa::ZERO,
    }
}

pub fn lookups() -> Lookups {
    let seller = || account(PartyKind::Seller);
    let buyer = || account(PartyKind::Buyer);
    let accounts: HashMap<i64, AccountFacts> = HashMap::from([
        (RAHIM, seller()),
        (KARIM, seller()),
        (AKRAM, buyer()),
        (ARSHAD, buyer()),
        (
            NAQDI,
            AccountFacts {
                is_naqdi_mandi: true,
                ..buyer()
            },
        ),
        (
            PUKHTA_BUYER,
            AccountFacts {
                is_pukhta: true,
                ..buyer()
            },
        ),
        (
            PUKHTA_SELLER,
            AccountFacts {
                is_pukhta: true,
                ..seller()
            },
        ),
        (
            OWN_GOODS,
            AccountFacts {
                is_own_goods: true,
                garden_closed: false,
                ..seller()
            },
        ),
        (
            KHATA,
            AccountFacts {
                kind: PartyKind::Khata,
                ..account(PartyKind::Seller)
            },
        ),
        (
            CASH_ONLY_BUYER,
            AccountFacts {
                credit_allowed: false,
                ..buyer()
            },
        ),
    ]);
    let item = |group, by_weight| ItemFacts {
        group,
        by_weight,
        is_active: true,
    };
    Lookups {
        accounts,
        items: HashMap::from([
            (BANANA, item(ItemGroup::Fruit, false)),
            (APPLE, item(ItemGroup::Fruit, false)),
            (MOOLI, item(ItemGroup::Vegetable, false)),
            (GRAPES, item(ItemGroup::Fruit, true)),
        ]),
        packings: HashSet::from([BOX]),
        ..Lookups::default()
    }
}

pub fn switches() -> Switches {
    Switches {
        allow_line_override: true,
        credit_limit_blocks: false,
        rate_check_percent: Some(30),
    }
}

// ---- Draft builders ----

pub fn header() -> HeaderInput {
    HeaderInput {
        entry_date: "10-10-2026".into(),
        bill_date: String::new(),
        book: None,
        truck_no: None,
        zamindar_account_id: None,
        adda_id: None,
        driver_name: None,
        driver_phone: None,
        total_freight: None,
        store_id: None,
        note: None,
    }
}

pub fn manual(value: &str) -> Option<ChargeInput> {
    Some(ChargeInput {
        method: LotChargeMethod::Manual,
        value: value.into(),
    })
}

pub fn per_unit(value: &str) -> Option<ChargeInput> {
    Some(ChargeInput {
        method: LotChargeMethod::PerUnit,
        value: value.into(),
    })
}

/// The lot of PRD 4.4: freight 3,000 (firm), labour 5 per نگ, munshiana 50, phone 20, نقد 5,000.
pub fn lot(seller: i64, item: i64, qty_received: &str) -> LotInput {
    LotInput {
        key: format!("lot-{seller}-{item}"),
        lot_id: None,
        seller_account_id: Some(seller),
        purchase_id: None,
        pukhta: None,
        item_id: Some(item),
        packing_id: Some(BOX),
        marka_id: None,
        store_id: None,
        qty_received: qty_received.into(),
        weight_received: None,
        freight: Some("3,000".into()),
        freight_paid_by: FreightPaidBy::Firm,
        labour: per_unit("5"),
        munshiana: manual("50"),
        phone: manual("20"),
        cash_paid: Some("5,000".into()),
        store_rent: None,
        adjust_advance: true,
        sales: Vec::new(),
    }
}

/// A lot with no typed charge at all, so the rules decide.
pub fn bare_lot(seller: i64, item: i64, qty_received: &str) -> LotInput {
    LotInput {
        lot_id: None,
        freight: None,
        labour: None,
        munshiana: None,
        phone: None,
        cash_paid: None,
        ..lot(seller, item, qty_received)
    }
}

pub fn sale(buyer: i64, qty: &str, rate: &str) -> SaleInput {
    SaleInput {
        key: format!("s-{buyer}-{qty}-{rate}"),
        sale_id: None,
        buyer_account_id: Some(buyer),
        pukhta: None,
        qty: qty.into(),
        weight: None,
        seller_rate: rate.into(),
        rate_differs: false,
        buyer_rate: None,
        payment_type: None,
        seller_comm: None,
        buyer_comm: None,
        laga: None,
        note: None,
    }
}

pub fn blank(qty: &str) -> SaleInput {
    SaleInput {
        sale_id: None,
        buyer_account_id: None,
        ..sale(0, qty, "")
    }
}

pub fn with(mut lot: LotInput, sales: Vec<SaleInput>) -> LotInput {
    lot.sales = sales;
    lot
}

pub fn draft(lots: Vec<LotInput>) -> BikriDraft {
    BikriDraft {
        header: header(),
        lots,
    }
}

/// PRD 4.4, Event 1.
pub fn event1() -> BikriDraft {
    draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![sale(AKRAM, "20", "600"), sale(ARSHAD, "30", "580")],
    )])
}

pub fn run_with(
    draft: &BikriDraft,
    lookups: &Lookups,
    rules: Vec<Rule>,
    switches: Switches,
) -> PostingCalc {
    let prepared = prepare_draft(draft, today(), None);
    calc_posting(
        PostingTarget::New,
        &prepared,
        lookups,
        &RuleSet::build(rules),
        switches,
        &StoredPosting::none(),
    )
    .unwrap()
}

pub fn run(draft: &BikriDraft) -> PostingCalc {
    run_with(draft, &lookups(), default_rules(), switches())
}

// ---- Reading a result ----

pub fn line(calc: &PostingCalc, lot: usize, line: usize) -> &LineCalc {
    calc.lots[lot].lines[line]
        .calc
        .as_ref()
        .unwrap_or_else(|| panic!("line {line} has no numbers: {:?}", calc.errors))
}

pub fn lot_of(calc: &PostingCalc, lot: usize) -> &LotPreview {
    &calc.lots[lot]
}

pub fn r(value: Paisa) -> i64 {
    value.rupees()
}

pub fn has(
    list: &[mandi_core::error::FieldError],
    code: mandi_core::error::ErrorCode,
    field: &str,
) -> bool {
    list.iter().any(|e| e.code == code && e.field == field)
}

pub fn system_accounts() -> SystemAccounts {
    let id = AccountId::new;
    SystemAccounts {
        cash: id(101),
        seller_commission: id(102),
        buyer_commission: id(103),
        laga: id(104),
        phone: id(105),
        rate_diff: id(106),
        labour: id(107),
        freight: id(108),
        store_rent: id(109),
        munshiana: id(110),
    }
}

pub fn entry_of(calc: &PostingCalc) -> BalancedEntry {
    build_bikri_entry(&calc.journal, &system_accounts()).unwrap()
}

/// The value of a named argument of a working, as paisa or count.
fn arg(working: &Working, name: &str) -> Option<i64> {
    working
        .args
        .iter()
        .find(|a| a.name == name)
        .map(|a| match a.value {
            WorkingValue::Money { paisa } => paisa.as_paisa(),
            WorkingValue::Count { value } => value,
            WorkingValue::Percent { bps } => i64::from(bps),
            WorkingValue::Level { .. } => 0,
        })
}

/// Panics unless the `result` of the working is the amount (Code Standards B30).
pub fn assert_result(name: &str, amount: Paisa, working: &Working) {
    assert_eq!(
        arg(working, "result"),
        Some(amount.as_paisa()),
        "working of {name} ({}) does not end at the amount",
        working.message_id
    );
}

/// Walks every amount of a result and checks that its working ends at the amount.
pub fn assert_every_working_matches(calc: &PostingCalc) {
    for (i, lot) in calc.lots.iter().enumerate() {
        for (j, line) in lot.lines.iter().enumerate() {
            let Some(c) = &line.calc else { continue };
            let n = |what: &str| format!("lot {i} line {j} {what}");
            assert_result(
                &n("seller_amount"),
                c.seller_amount,
                &c.workings.seller_amount,
            );
            assert_result(&n("buyer_value"), c.buyer_value, &c.workings.buyer_value);
            assert_result(&n("buyer_total"), c.buyer_total, &c.workings.buyer_total);
            assert_result(&n("rate_diff"), c.rate_diff, &c.workings.rate_diff);
            for (what, charge) in [
                ("seller_comm", &c.seller_comm),
                ("buyer_comm", &c.buyer_comm),
                ("laga", &c.laga),
            ] {
                assert_result(&n(what), charge.amount, &charge.working);
            }
        }
        for (what, charge) in [
            ("labour", &lot.labour),
            ("munshiana", &lot.munshiana),
            ("phone", &lot.phone),
        ] {
            if let Some(charge) = charge {
                assert_result(&format!("lot {i} {what}"), charge.amount, &charge.working);
            }
        }
        if let Some(t) = &lot.totals {
            totals_match(&format!("lot {i}"), t);
        }
    }
    totals_match("whole", &calc.summary);
}

fn totals_match(scope: &str, t: &mandi_core::bikri::SummaryTotals) {
    let w = &t.workings;
    for (what, amount, working) in [
        ("gross", t.gross, &w.gross),
        ("freight", t.freight, &w.freight),
        ("seller_comm", t.seller_comm, &w.seller_comm),
        ("labour", t.labour, &w.labour),
        ("munshiana", t.munshiana, &w.munshiana),
        ("cash_paid", t.cash_paid, &w.cash_paid),
        ("store_rent", t.store_rent, &w.store_rent),
        ("phone", t.phone, &w.phone),
        ("deductions", t.deductions, &w.deductions),
        ("net", t.net, &w.net),
        ("seller_credit", t.seller_credit, &w.seller_credit),
        ("buyer_comm", t.buyer_comm, &w.buyer_comm),
        ("laga", t.laga, &w.laga),
        ("rate_diff", t.rate_diff, &w.rate_diff),
        ("buyer_credit", t.buyer_credit, &w.buyer_credit),
        ("buyer_cash", t.buyer_cash, &w.buyer_cash),
        ("buyer_total", t.buyer_total, &w.buyer_total),
        ("shared_income", t.shared_income, &w.shared_income),
    ] {
        assert_result(&format!("{scope} {what}"), amount, working);
    }
}

pub fn single_lot_totals(calc: &PostingCalc) -> &mandi_core::bikri::SummaryTotals {
    calc.lots[0].totals.as_ref().expect("lot totals")
}

pub fn assert_no_errors(calc: &PostingCalc) {
    assert!(calc.errors.is_empty(), "{:?}", calc.errors);
}

pub fn _unused(_: ItemSales) {}
