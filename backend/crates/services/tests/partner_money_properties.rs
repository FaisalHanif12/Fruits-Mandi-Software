//! Property tests of plan 17 (Code Standards 9.3, PRD FR-16.2, FR-16.5, FR-16.6, FR-17.9, C20,
//! C22). A random run of sales, shop expenses, partner takes and returns, capital moves, money
//! lent and returned, الگ کھاتہ entries, جنرل entries, a market trip with an own sale, and
//! cancels. After every step:
//!
//! - the books balance (Σ بنام = Σ جمع);
//! - Σ of the partners' کل رقم + ابتدائی سرمایہ (+ ابتدائی فرق) = the firm's equity in the trial
//!   balance (capital, drawings, gardens/trips/fruit accounts, income less expenses);
//! - the open amount of every loan and of every partner's money taken is never negative and
//!   equals the balance of its account (the read-time working of FR-16.2, FR-16.5);
//! - a الگ کھاتہ entry changes no partner's total and writes no share row;
//! - the last running total of every partner's statement is his کل رقم.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::{raw, set_today};
use mandi_services::core::bikri::{BikriSaveInput, LotInput};
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::{
    PartnerTakesInput, PersonalExpenseInput, PersonalReturnInput, ShopExpenseInput,
};
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::khata::{KhataCreateInput, KhataInInput, KhataOutInput};
use mandi_services::core::lent::{
    LentListInput, LentPersonCreateInput, LentReturnMode, MoneyLentInput, MoneyLentReturnInput,
};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
use mandi_services::core::purchases::{PurchaseGetInput, PurchaseInput};
use mandi_services::core::statement::PartnersStatementInput;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::{
    VoucherCancelInput, VoucherEditInput, VoucherEntry, VoucherType,
};
use mandi_services::{
    ServiceError, bikri, khata, ledger, lent, masters, partners, purchases, trips, vouchers,
};
use proptest::prelude::*;
use support::World;

#[derive(Debug, Clone)]
enum Step {
    /// A sale of `qty` نگ at `rate` to اکرم on credit.
    Sale {
        qty: i64,
        rate: i64,
    },
    /// A shop expense of `amount` rupees.
    Shop {
        amount: i64,
    },
    /// A partner takes `amount` rupees for himself.
    Take {
        partner: usize,
        amount: i64,
    },
    /// A partner returns `amount` rupees (refused when more than he owes; then nothing happens).
    Return {
        partner: usize,
        amount: i64,
    },
    CapitalIn {
        partner: usize,
        amount: i64,
    },
    CapitalOut {
        partner: usize,
        amount: i64,
    },
    /// Money lent to the person.
    Lend {
        amount: i64,
    },
    /// Money returned by the person, in cash or from salary (refused when more than he owes).
    LendBack {
        amount: i64,
        salary: bool,
    },
    KhataOut {
        amount: i64,
    },
    KhataIn {
        amount: i64,
    },
    /// A balanced جنرل: the electricity head against cash.
    General {
        amount: i64,
    },
    /// An own sale of the trip's goods.
    OwnSale {
        qty: i64,
        rate: i64,
    },
    /// A cancel of an earlier voucher (refused when it has returns against it; then nothing happens).
    Cancel {
        pick: usize,
    },
    /// An edit of an earlier take or loan to another partner and amount (a loan keeps its
    /// person). Refused when returns would then stand above what was taken or lent, on the old
    /// account or the new one; then nothing happens (money agent, plan 17, finding 1).
    Edit {
        pick: usize,
        partner: usize,
        amount: i64,
    },
}

#[derive(Debug, Clone)]
struct Case {
    weights: Vec<u32>,
    pockets: Vec<bool>,
    /// The trip's purchase: نگ and rate, or none.
    trip: Option<(i64, i64)>,
    /// Each step with the day it is dated: 10-10, 11-10 or 12-10-2026 (all open).
    steps: Vec<(Step, u8)>,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => (1_i64..30, 1_i64..2_000).prop_map(|(qty, rate)| Step::Sale { qty, rate }),
        2 => (1_i64..5_000).prop_map(|amount| Step::Shop { amount }),
        2 => (0_usize..3, 1_i64..5_000).prop_map(|(partner, amount)| Step::Take { partner, amount }),
        2 => (0_usize..3, 1_i64..5_000).prop_map(|(partner, amount)| Step::Return { partner, amount }),
        2 => (0_usize..3, 1_i64..60_000).prop_map(|(partner, amount)| Step::CapitalIn { partner, amount }),
        1 => (0_usize..3, 1_i64..60_000).prop_map(|(partner, amount)| Step::CapitalOut { partner, amount }),
        2 => (1_i64..5_000).prop_map(|amount| Step::Lend { amount }),
        2 => (1_i64..5_000, any::<bool>()).prop_map(|(amount, salary)| Step::LendBack { amount, salary }),
        1 => (1_i64..5_000).prop_map(|amount| Step::KhataOut { amount }),
        1 => (1_i64..5_000).prop_map(|amount| Step::KhataIn { amount }),
        1 => (1_i64..5_000).prop_map(|amount| Step::General { amount }),
        2 => (1_i64..20, 1_i64..3_000).prop_map(|(qty, rate)| Step::OwnSale { qty, rate }),
        1 => (0_usize..12).prop_map(|pick| Step::Cancel { pick }),
        2 => (0_usize..12, 0_usize..3, 1_i64..5_000)
            .prop_map(|(pick, partner, amount)| Step::Edit { pick, partner, amount }),
    ]
}

fn case() -> impl Strategy<Value = Case> {
    (
        proptest::collection::vec(1_u32..1_000, 1..4),
        proptest::collection::vec(any::<bool>(), 3),
        proptest::option::of((2_i64..40, 1_i64..2_000)),
        proptest::collection::vec((step(), 0_u8..3), 1..10),
    )
        .prop_map(|(weights, pockets, trip, steps)| Case {
            weights,
            pockets,
            trip,
            steps,
        })
}

/// Shares in bps that total exactly 10,000, from weights.
fn shares_of(weights: &[u32]) -> Vec<i32> {
    let total: u32 = weights.iter().sum();
    let mut out: Vec<i32> = weights
        .iter()
        .map(|w| i32::try_from(u64::from(*w) * 10_000 / u64::from(total)).unwrap())
        .collect();
    let used: i32 = out.iter().sum();
    out[0] += 10_000 - used;
    out
}

fn percent(bps: i32) -> String {
    format!("{}.{:02}", bps / 100, bps % 100)
}

fn confirming<T>(mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

/// An expense head: its id and the id of its account.
fn make_head(w: &World, ur: &str, en: &str) -> (i64, i64) {
    confirming(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::ExpenseHead(ExpenseHeadInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: ur.into(),
                name_en: en.into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
    let row = masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .into_iter()
        .find(|r| r.head.name_en == en)
        .unwrap();
    (row.id, row.account_id)
}

fn scalar(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// Credits less debits of one account in paisa.
fn credit_net(w: &World, account_id: i64) -> i64 {
    scalar(
        w,
        &format!(
            "SELECT COALESCE(SUM(cr_paisa - dr_paisa), 0) FROM journal_lines WHERE account_id = {account_id}"
        ),
    )
}

/// The firm's equity from the books: credits less debits of the income, expense and capital heads.
fn equity(w: &World) -> i64 {
    scalar(
        w,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0)
         FROM journal_lines l JOIN accounts a ON a.id = l.account_id
         JOIN account_controls c ON c.id = a.control_id
         JOIN account_heads h ON h.id = c.head_id
         WHERE h.code IN ('04', '05', '06')",
    )
}

fn totals(w: &World) -> Vec<i64> {
    let rows = partners::overview(&w.env.ctx).unwrap();
    w.partners
        .iter()
        .map(|p| {
            rows.iter()
                .find(|r| r.partner_id == *p)
                .map_or(0, |r| r.total_paisa.as_paisa())
        })
        .collect()
}

fn share_rows(w: &World) -> i64 {
    scalar(w, "SELECT COUNT(*) FROM partner_share_rows")
}

/// The lowest balance (debit less credit) at the end of any day of one account, from the journal
/// by plain SQL (a second way, apart from the one the app uses). Below 0 means returns stood above
/// what was taken or lent on some day.
fn lowest_day_end(w: &World, account_id: i64) -> i64 {
    scalar(
        w,
        &format!(
            "SELECT COALESCE(MIN(running), 0) FROM (
                 SELECT SUM(net) OVER (ORDER BY day) AS running FROM (
                     SELECT e.entry_date AS day, SUM(l.dr_paisa) - SUM(l.cr_paisa) AS net
                     FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                     WHERE e.status = 'posted' AND l.account_id = {account_id}
                     GROUP BY e.entry_date))"
        ),
    )
}

/// The invariants that hold after every step.
fn check(w: &World, lent_person: i64) {
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr, "the books balance");

    // C22 and FR-17.9: Σ کل رقم + ابتدائی سرمایہ (+ ابتدائی فرق) = equity.
    let opening = credit_net(w, support::id_of(&w.env, "06010001"))
        + credit_net(w, support::id_of(&w.env, "06019999"));
    let sum: i64 = totals(w).iter().sum();
    assert_eq!(
        sum + opening,
        equity(w),
        "partner totals + opening = equity"
    );

    // FR-16.2: money taken and still open equals the drawings account, never negative.
    for partner in w.partners {
        let takes = partners::takes(
            &w.env.ctx,
            &PartnerTakesInput {
                partner_id: partner,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap();
        assert!(takes.rows.iter().all(|r| r.open_paisa.as_paisa() >= 0));
        let drawings = scalar(
            w,
            &format!(
                "SELECT COALESCE(SUM(l.dr_paisa - l.cr_paisa), 0) FROM journal_lines l
                 JOIN partners p ON p.drawings_account_id = l.account_id WHERE p.id = {partner}"
            ),
        );
        assert_eq!(
            takes.totals.open_paisa.as_paisa(),
            drawings,
            "takes = drawings"
        );
    }

    // On no day do returns stand above what was taken (drawings) or lent (the person).
    for partner in w.partners {
        let drawings = scalar(
            w,
            &format!("SELECT drawings_account_id FROM partners WHERE id = {partner}"),
        );
        assert!(
            lowest_day_end(w, drawings) >= 0,
            "a day ended with returns above the take"
        );
    }
    assert!(
        lowest_day_end(w, lent_person) >= 0,
        "a day ended with returns above the loan"
    );

    // FR-16.5: a loan's open amount equals the person's account, never negative.
    let loans = lent::list(
        &w.env.ctx,
        &LentListInput {
            person_id: lent_person,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert!(loans.rows.iter().all(|r| r.open_paisa.as_paisa() >= 0));
    assert_eq!(
        loans.totals.open_paisa.as_paisa(),
        -credit_net(w, lent_person),
        "loans = the person's account"
    );

    // FR-17.10: the last running total of the statement is the partner's کل رقم.
    let now = totals(w);
    for (i, partner) in w.partners.iter().enumerate() {
        let first = partners::statement(
            &w.env.ctx,
            &PartnersStatementInput {
                partner_id: *partner,
                from: None,
                to: None,
                offset: 0,
                limit: 1,
            },
        )
        .unwrap();
        assert_eq!(first.closing_total_paisa.as_paisa(), now[i]);
        if first.total_count == 0 {
            assert_eq!(now[i], 0);
            continue;
        }
        let last = partners::statement(
            &w.env.ctx,
            &PartnersStatementInput {
                partner_id: *partner,
                from: None,
                to: None,
                offset: u32::try_from(first.total_count - 1).unwrap(),
                limit: 1,
            },
        )
        .unwrap();
        assert_eq!(
            last.lines[0].running_paisa.as_paisa(),
            now[i],
            "the statement ends at کل رقم"
        );
    }
}

fn own_lot(w: &World, purchase_id: i64, qty: i64) -> LotInput {
    LotInput {
        seller_account_id: Some(support::id_of(&w.env, "030119998")),
        purchase_id: Some(purchase_id),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..support::bare_lot(w, "own-1", w.rahim, w.banana, &qty.to_string())
    }
}

/// The days a step can be dated on.
const DAYS: [&str; 3] = ["10-10-2026", "11-10-2026", "12-10-2026"];

fn run(case: &Case) {
    let w = support::world();
    set_today(&w.env, 2026, 10, 12);
    let date = "10-10-2026";
    let (head_id, head_account) = make_head(&w, "بجلی", "Electricity");
    make_head(&w, "تنخواہ", "Salaries");
    let family = confirming(|words| {
        khata::create(
            &w.env.ctx,
            &w.owner,
            &KhataCreateInput {
                name_ur: "خاندانی کھاتہ".into(),
                name_en: "Family Khata".into(),
                note: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id;
    let person = confirming(|words| {
        lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                phone: None,
                staff_id: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id;

    // The trip: random shares and money sources, one purchase paid at once.
    let mut purchase: Option<i64> = None;
    if let Some((qty, rate)) = case.trip {
        let shares = shares_of(&case.weights);
        let listed: Vec<VenturePartnerInput> = w
            .partners
            .iter()
            .enumerate()
            .map(|(i, id)| VenturePartnerInput {
                partner_id: *id,
                share: shares.get(i).map_or("0".to_owned(), |bps| percent(*bps)),
                source: if case.pockets[i] {
                    MoneySource::Pocket
                } else {
                    MoneySource::Shop
                },
            })
            .collect();
        let trip = trips::create(
            &w.env.ctx,
            &w.owner,
            &TripInput {
                date: date.into(),
                market: None,
                sent_person: None,
                note: None,
                partners: listed,
            },
        )
        .unwrap()
        .id;
        let item = support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit");
        purchase = Some(
            purchases::save(
                &w.env.ctx,
                &w.owner,
                &PurchaseInput {
                    trip_id: Some(trip),
                    date: date.into(),
                    supplier: None,
                    item_id: Some(item),
                    packing_id: Some(w.box_),
                    qty: qty.to_string(),
                    weight: None,
                    rate: rate.to_string(),
                    freight: None,
                    labour: None,
                    other_cost: None,
                    note: None,
                    parts: Vec::new(),
                    confirm_cash_below_zero: true,
                },
            )
            .unwrap()
            .id,
        );
    }
    check(&w, person);

    let mut saved: Vec<i64> = Vec::new();
    for (step, day) in &case.steps {
        // Takes, returns, loans and edits fall on different days, so a return can come before
        // its take and a take can be moved past its return (money agent, plan 17, finding 6).
        let date = DAYS[usize::from(*day)];
        let before = (totals(&w), share_rows(&w));
        let mut khata_step = false;
        let keep = |result: Result<mandi_services::core::vouchers::VoucherSaved, ServiceError>,
                    saved: &mut Vec<i64>| match result {
            Ok(done) => saved.push(done.voucher_id),
            Err(error) => assert_eq!(error.code(), ErrorCode::ErrReturnExceedsOpen, "{error:?}"),
        };
        match step {
            Step::Sale { qty, rate } => {
                let lot = support::with_sales(
                    support::bare_lot(&w, "lot-1", w.rahim, w.banana, &qty.to_string()),
                    vec![support::sale(
                        "s-1",
                        w.akram,
                        &qty.to_string(),
                        &rate.to_string(),
                    )],
                );
                bikri::save(
                    &w.env.ctx,
                    &w.owner,
                    &BikriSaveInput {
                        draft: support::draft(vec![lot]),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
            }
            Step::Shop { amount } => keep(
                vouchers::save_shop_expense(
                    &w.env.ctx,
                    &w.owner,
                    &ShopExpenseInput {
                        date: date.into(),
                        expense_head_id: Some(head_id),
                        amount: amount.to_string(),
                        paid_to: None,
                        narration: None,
                        confirm_cash_below_zero: true,
                    },
                ),
                &mut saved,
            ),
            Step::Take { partner, amount } => keep(
                vouchers::save_personal_expense(
                    &w.env.ctx,
                    &w.owner,
                    &PersonalExpenseInput {
                        date: date.into(),
                        partner_id: w.partners.get(*partner).copied(),
                        amount: amount.to_string(),
                        reason: "home".into(),
                        return_by: None,
                        taken_by: None,
                        narration: None,
                        confirm_cash_below_zero: true,
                    },
                ),
                &mut saved,
            ),
            Step::Return { partner, amount } => keep(
                vouchers::save_personal_return(
                    &w.env.ctx,
                    &w.owner,
                    &PersonalReturnInput {
                        date: date.into(),
                        partner_id: w.partners.get(*partner).copied(),
                        amount: amount.to_string(),
                        narration: None,
                    },
                ),
                &mut saved,
            ),
            Step::CapitalIn { partner, amount } | Step::CapitalOut { partner, amount } => {
                let input = CapitalInput {
                    date: date.into(),
                    partner_id: w.partners.get(*partner).copied(),
                    amount: amount.to_string(),
                    narration: None,
                    confirm_cash_below_zero: true,
                };
                keep(
                    if matches!(step, Step::CapitalIn { .. }) {
                        vouchers::save_capital_in(&w.env.ctx, &w.owner, &input)
                    } else {
                        vouchers::save_capital_out(&w.env.ctx, &w.owner, &input)
                    },
                    &mut saved,
                );
            }
            Step::Lend { amount } => keep(
                vouchers::save_money_lent(
                    &w.env.ctx,
                    &w.owner,
                    &MoneyLentInput {
                        date: date.into(),
                        person_account_id: Some(person),
                        amount: amount.to_string(),
                        reason: "need".into(),
                        return_by: None,
                        narration: None,
                        confirm_cash_below_zero: true,
                    },
                ),
                &mut saved,
            ),
            Step::LendBack { amount, salary } => keep(
                vouchers::save_money_lent_return(
                    &w.env.ctx,
                    &w.owner,
                    &MoneyLentReturnInput {
                        date: date.into(),
                        person_account_id: Some(person),
                        amount: amount.to_string(),
                        mode: if *salary {
                            LentReturnMode::Salary
                        } else {
                            LentReturnMode::Cash
                        },
                        salary_head_id: None,
                        narration: None,
                    },
                ),
                &mut saved,
            ),
            Step::KhataOut { amount } => {
                khata_step = true;
                keep(
                    vouchers::save_khata_out(
                        &w.env.ctx,
                        &w.owner,
                        &KhataOutInput {
                            date: date.into(),
                            khata_account_id: Some(family),
                            amount: amount.to_string(),
                            reason: "family".into(),
                            narration: None,
                            confirm_cash_below_zero: true,
                        },
                    ),
                    &mut saved,
                );
            }
            Step::KhataIn { amount } => {
                khata_step = true;
                keep(
                    vouchers::save_khata_in(
                        &w.env.ctx,
                        &w.owner,
                        &KhataInInput {
                            date: date.into(),
                            khata_account_id: Some(family),
                            amount: amount.to_string(),
                            reason: "back".into(),
                            narration: None,
                            confirm_cash_below_zero: true,
                        },
                    ),
                    &mut saved,
                );
            }
            Step::General { amount } => {
                let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
                    account_id: Some(account),
                    person_key: None,
                    dr: dr.into(),
                    cr: cr.into(),
                    narration: None,
                };
                let text = amount.to_string();
                keep(
                    vouchers::general_save(
                        &w.env.ctx,
                        &w.owner,
                        &GeneralInput {
                            date: date.into(),
                            narration: None,
                            lines: vec![
                                line(head_account, &text, ""),
                                line(support::id_of(&w.env, "01010001"), "", &text),
                            ],
                            confirm_cash_below_zero: true,
                        },
                    ),
                    &mut saved,
                );
            }
            Step::OwnSale { qty, rate } => {
                let Some(purchase_id) = purchase else {
                    continue;
                };
                let left = purchases::get(&w.env.ctx, &PurchaseGetInput { id: purchase_id })
                    .unwrap()
                    .qty_left;
                if left == 0 {
                    continue;
                }
                let qty = (*qty).min(left);
                let lot = support::with_sales(
                    own_lot(&w, purchase_id, qty),
                    vec![support::sale(
                        "s-1",
                        w.akram,
                        &qty.to_string(),
                        &rate.to_string(),
                    )],
                );
                bikri::save(
                    &w.env.ctx,
                    &w.owner,
                    &BikriSaveInput {
                        draft: support::draft(vec![lot]),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
            }
            Step::Cancel { pick } => {
                if saved.is_empty() {
                    continue;
                }
                let id = saved[pick % saved.len()];
                let row = vouchers::get(&w.env.ctx, id).unwrap();
                // A voucher with returns against it, or one already cancelled, is refused.
                match vouchers::cancel(
                    &w.env.ctx,
                    &w.owner,
                    &VoucherCancelInput {
                        id,
                        version: row.version,
                        reason: "mistake".into(),
                        confirm_cash_below_zero: true,
                    },
                ) {
                    Ok(_) => saved.retain(|v| *v != id),
                    Err(ServiceError::Rejected(_)) => {}
                    Err(other) => panic!("{other:?}"),
                }
            }
            Step::Edit {
                pick,
                partner,
                amount,
            } => {
                if saved.is_empty() {
                    continue;
                }
                let id = saved[pick % saved.len()];
                let row = vouchers::get(&w.env.ctx, id).unwrap();
                let entry = match row.voucher_type {
                    VoucherType::PersonalExpense => {
                        VoucherEntry::PersonalExpense(PersonalExpenseInput {
                            date: date.into(),
                            partner_id: w.partners.get(*partner).copied(),
                            amount: amount.to_string(),
                            reason: "home".into(),
                            return_by: None,
                            taken_by: None,
                            narration: None,
                            confirm_cash_below_zero: true,
                        })
                    }
                    VoucherType::MoneyLent => VoucherEntry::MoneyLent(MoneyLentInput {
                        date: date.into(),
                        person_account_id: Some(person),
                        amount: amount.to_string(),
                        reason: "need".into(),
                        return_by: None,
                        narration: None,
                        confirm_cash_below_zero: true,
                    }),
                    _ => continue,
                };
                match vouchers::edit(
                    &w.env.ctx,
                    &w.owner,
                    &VoucherEditInput {
                        id,
                        version: row.version,
                        entry,
                        reason: String::new(),
                        confirm_cash_below_zero: true,
                    },
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        assert_eq!(error.code(), ErrorCode::ErrReturnExceedsOpen, "{error:?}");
                    }
                }
            }
        }
        if khata_step {
            // FR-16.6: a الگ کھاتہ entry reaches no partner.
            assert_eq!(
                (totals(&w), share_rows(&w)),
                before,
                "a khata reaches no partner"
            );
        }
        check(&w, person);
    }
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM partner_share_rows r JOIN journal_entries e ON e.id = r.journal_entry_id
             WHERE e.posting_type IN ('khata_out', 'khata_in')"
        ),
        0
    );
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

/// Cases per run when `PROPTEST_CASES` is not set.
const DEFAULT_CASES: u32 = 64;

/// The number of cases: the environment variable `PROPTEST_CASES` when it is a number, else
/// [`DEFAULT_CASES`]. A long run: `PROPTEST_CASES=10000 cargo test -p mandi-services --test
/// partner_money_properties --release`.
fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(DEFAULT_CASES)
}

proptest! {
    // Each case builds a whole database and checks every partner after every step, so the normal
    // run is small (the pure money maths has 10,000 cases in the domain crate). The count follows
    // the environment variable PROPTEST_CASES.
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    /// Plan 17 Section 4 "Tests": the partner totals, the open amounts, the khata and the
    /// statement, for any run of the postings of this plan.
    #[test]
    fn partner_money_adds_up_after_every_step(case in case()) {
        run(&case);
    }
}
