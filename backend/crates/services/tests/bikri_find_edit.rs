//! Plan 67 T1 and T2 (D8 Feature 7; PRD FR-12.13, FR-7.70, FR-7.71, FR-12.18 to FR-12.21, decision
//! E1 of plan 60): finding a saved بکری by each filter, and what an edit does when it changes the
//! buyer of a line or the seller of a lot.
//!
//! Part 1 (find) is a test for each filter of `bikri_find`, on the PRD worked day after Events 1 to 5:
//! بکری 1 is رحیم's کیلا on truck LES-1234, sold to اکرم and ارشد; بکری 2 is کریم's سیب on truck
//! LHR-0077, sold to the walk-in buyer شاہد (a چکتا buyer, with no account of his own).
//!
//! Part 2 (edit) uses the numbers of plan 67 Section 2: بکری 1 line 1 moves from اکرم to طارق, a
//! new buyer on the default rules. اکرم then has no 12,880 bill, and his 10,000 receipt is left over
//! (10,000 جمع); طارق owes 12,880; commission and the partner split do not change; the trial
//! balance is 95,048 on both sides. Changing it back gives 85,048 and every original number.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::accounts::{AccountType, BalanceSide};
use mandi_services::core::bikri::{
    BikriDraft, BikriEdited, BikriFindInput, BikriSaved, EditOpenInput, EditPreviewInput,
    EditSaveInput, EditableBikri, SaleState,
};
use mandi_services::core::dead::{DeadMarkInput, DeadReason};
use mandi_services::core::edit::ChangeValue;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::rules::{ChargesInput, DefaultChargeInput};
use mandi_services::core::vouchers::AdvanceInput;
use mandi_services::{assist, bikri, day, dead, edit, ledger, rules, vouchers};
use reports_world::*;
use support::World;

/// The worked day after Events 1 to 5, keeping the two بکری saved.
fn day() -> (World, BikriSaved, BikriSaved) {
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    let two = save(&w, &event2(&w));
    // A new بکری has no truck (PRD v1.34): these two are old بکری that have one.
    support::give_truck(&w, one.number, "LES-1234");
    support::give_truck(&w, two.number, "LHR-0077");
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "4,316")).unwrap();
    event5(&w);
    (w, one, two)
}

fn find(w: &World, input: BikriFindInput) -> Vec<i64> {
    bikri::find(&w.env.ctx, &input)
        .unwrap()
        .iter()
        .map(|r| r.number)
        .collect()
}

// ---- Part 1: find, a test for each filter -------------------------------------------------

#[test]
fn find_by_number_date_truck_seller_and_item() {
    let (w, _, _) = day();
    let by = |input: BikriFindInput| find(&w, input);
    // Number.
    assert_eq!(
        by(BikriFindInput {
            number: Some(1),
            ..Default::default()
        }),
        [1]
    );
    assert_eq!(
        by(BikriFindInput {
            number: Some(2),
            ..Default::default()
        }),
        [2]
    );
    assert_eq!(
        by(BikriFindInput {
            number: Some(3),
            ..Default::default()
        }),
        Vec::<i64>::new()
    );
    // Day: newest number first.
    assert_eq!(
        by(BikriFindInput {
            date: Some("10-10-2026".into()),
            ..Default::default()
        }),
        [2, 1]
    );
    assert_eq!(
        by(BikriFindInput {
            date: Some("11-10-2026".into()),
            ..Default::default()
        }),
        Vec::<i64>::new()
    );
    // The start of the truck number, letter case ignored.
    assert_eq!(
        by(BikriFindInput {
            truck_no: Some("LES".into()),
            ..Default::default()
        }),
        [1]
    );
    assert_eq!(
        by(BikriFindInput {
            truck_no: Some("lhr-00".into()),
            ..Default::default()
        }),
        [2]
    );
    // Seller and item.
    assert_eq!(
        by(BikriFindInput {
            seller_id: Some(w.rahim),
            ..Default::default()
        }),
        [1]
    );
    assert_eq!(
        by(BikriFindInput {
            seller_id: Some(w.karim),
            ..Default::default()
        }),
        [2]
    );
    assert_eq!(
        by(BikriFindInput {
            item_id: Some(w.banana),
            ..Default::default()
        }),
        [1]
    );
    assert_eq!(
        by(BikriFindInput {
            item_id: Some(w.apple),
            ..Default::default()
        }),
        [2]
    );
}

#[test]
fn find_by_buyer_account() {
    let (w, _, _) = day();
    let by = |buyer: i64| {
        find(
            &w,
            BikriFindInput {
                buyer_account_id: Some(buyer),
                ..Default::default()
            },
        )
    };
    // اکرم and ارشد both bought in بکری 1.
    assert_eq!(by(w.akram), [1]);
    assert_eq!(by(w.arshad), [1]);
    // The walk-in buyer's shared account is not a buyer to look for: his lines have no account of
    // his own, and a buyer with no line finds nothing.
    assert_eq!(by(w.naqdi), Vec::<i64>::new());
    // Filters given together must all match: اکرم with a truck that is not his.
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                buyer_account_id: Some(w.akram),
                truck_no: Some("LHR".into()),
                ..Default::default()
            }
        ),
        Vec::<i64>::new()
    );
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                buyer_account_id: Some(w.akram),
                seller_id: Some(w.rahim),
                date: Some("10-10-2026".into()),
                ..Default::default()
            }
        ),
        [1]
    );
}

#[test]
fn find_a_walk_in_chukta_buyer_by_name() {
    let (w, _, _) = day();
    let by = |text: &str| {
        find(
            &w,
            BikriFindInput {
                buyer_text: Some(text.into()),
                ..Default::default()
            },
        )
    };
    // شاہد bought in بکری 2: by his Urdu name, his English name in any letter case, or a part of it.
    assert_eq!(by("شاہد"), [2]);
    assert_eq!(by("Shahid"), [2]);
    assert_eq!(by("shahid"), [2]);
    assert_eq!(by("  shah  "), [2]);
    assert_eq!(by("شا"), [2]);
    // A name nobody has, and the name of an ordinary buyer (an account, found by his id), find
    // nothing: the text looks only among the walk-in buyers.
    assert_eq!(by("ناصر"), Vec::<i64>::new());
    assert_eq!(by("Akram"), Vec::<i64>::new());
    // Blank text is no filter.
    assert_eq!(by("   "), [2, 1]);
    // Text and account together must both match, on one بکری or on two.
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                buyer_text: Some("Shahid".into()),
                buyer_account_id: Some(w.akram),
                ..Default::default()
            }
        ),
        Vec::<i64>::new()
    );
}

// ---- Part 2: what an edit does with a changed buyer and a changed seller -----------------------

fn open(w: &World, saved: &BikriSaved) -> EditableBikri {
    bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap()
}

/// Previews and saves an edit; the saved preview is the previewed one (Code Standards B5).
fn edit(w: &World, opened: &EditableBikri, draft: BikriDraft) -> BikriEdited {
    let preview = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let saved = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("غلط گاہک".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(saved.preview, preview);
    saved
}

fn balance(w: &World, id: i64) -> (BalanceSide, i64) {
    let b = ledger::account_balance(&w.env.ctx, id).unwrap();
    (b.side, b.amount_paisa.rupees())
}

fn trial(w: &World) -> (i64, i64) {
    let t = ledger::trial_balance(&w.env.ctx, None).unwrap();
    (t.total_dr.rupees(), t.total_cr.rupees())
}

/// The shared income of the first posting's split: 1,580 / 1,184 / 1,184 (E2E-4).
fn shares(w: &World, entry_id: i64) -> Vec<i64> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT SUM(amount_paisa) FROM partner_share_rows WHERE journal_entry_id = ?1
             GROUP BY partner_id ORDER BY partner_id",
        )
        .unwrap();
    statement
        .query_map([entry_id], |r| Ok(r.get::<_, i64>(0)? / 100))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn e2e_68_a_line_moves_to_another_buyer_and_back() {
    let (w, one, _) = day();
    let tariq = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق",
        "Tariq",
        true,
        "",
    );
    assert_eq!(trial(&w), (85_048, 85_048));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));
    let opened = open(&w, &one);

    // Line 1 (20 at 600) was for طارق, not اکرم.
    let mut draft = opened.draft.clone();
    assert_eq!(draft.lots[0].sales[0].buyer_account_id, Some(w.akram));
    draft.lots[0].sales[0].buyer_account_id = Some(tariq);
    let edited = edit(&w, &opened, draft);

    // اکرم has no bill left: his 10,000 receipt is credit. طارق owes the 12,880.
    assert_eq!(balance(&w, w.akram), (BalanceSide::Cr, 10_000));
    assert_eq!(balance(&w, tariq), (BalanceSide::Dr, 12_880));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 18_684));
    // Commission and the partner split are the same (the line is the same, only its buyer changed).
    assert_eq!(edited.preview.summary.shared_income.rupees(), 3_948);
    assert_eq!(shares(&w, edited.entry_id), [1_580, 1_184, 1_184]);
    // The books: the debit side is 85,048 − 2,880 (اکرم no longer owes) + 12,880 (طارق owes) =
    // 95,048. The credit side is 85,048 + 10,000 (اکرم's receipt is no longer netted against his
    // bill, so it stands as his credit) = 95,048.
    assert_eq!(trial(&w), (95_048, 95_048));
    // The find follows: the old buyer lost the بکری, the new one has it.
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                buyer_account_id: Some(w.akram),
                ..Default::default()
            }
        ),
        Vec::<i64>::new()
    );
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                buyer_account_id: Some(tariq),
                ..Default::default()
            }
        ),
        [1]
    );
    ledger::verify_journal_chain(&w.env.ctx).unwrap();

    // Back to اکرم: every original number returns.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one.bikri_id,
            entry_id: edited.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(w.akram);
    let back = edit(&w, &opened, draft);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));
    assert_eq!(balance(&w, tariq), (BalanceSide::Zero, 0));
    assert_eq!(trial(&w), (85_048, 85_048));
    assert_eq!(shares(&w, back.entry_id), [1_580, 1_184, 1_184]);
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                buyer_account_id: Some(w.akram),
                ..Default::default()
            }
        ),
        [1]
    );
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                buyer_account_id: Some(tariq),
                ..Default::default()
            }
        ),
        Vec::<i64>::new()
    );
}

#[test]
fn a_lots_seller_can_change_and_both_sellers_ledgers_follow() {
    let (w, one, _) = day();
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    assert_eq!(draft.lots[0].seller_account_id, Some(w.rahim));
    // رحیم's 50 کیلا were کریم's.
    draft.lots[0].seller_account_id = Some(w.karim);
    let edited = edit(&w, &opened, draft);
    // The seller credit 24,316 and the cash given on arrival (5,000) go to کریم; رحیم is left with
    // the advance he took (15,000) and the payment he was given (4,316), neither of which this بکری
    // now pays for. کریم also has his own 10,160 of بکری 2.
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 19_316));
    assert_eq!(balance(&w, w.karim), (BalanceSide::Cr, 29_476));
    assert_eq!(edited.preview.summary.shared_income.rupees(), 3_948);
    // The trial balance adds each account's balance: رحیم's 19,316 debit and کریم's 19,316 more
    // credit are one more 19,316 on each side.
    assert_eq!(trial(&w), (104_364, 104_364));
    // His advance is no longer paid back by this بکری: the 15,000 is open again.
    assert_eq!(
        assist::advances(&w.env.ctx, w.rahim)
            .unwrap()
            .open_paisa
            .rupees(),
        15_000
    );
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                seller_id: Some(w.rahim),
                ..Default::default()
            }
        ),
        Vec::<i64>::new()
    );
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                seller_id: Some(w.karim),
                ..Default::default()
            }
        ),
        [2, 1]
    );
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

/// The live bill matches of one account: how many, and the rupees they settle.
fn live_matches(w: &World, account: i64) -> (i64, i64) {
    raw(&w.dir)
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(amount_paisa), 0) FROM bill_allocations
             WHERE account_id = ?1 AND is_active = 1",
            [account],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? / 100)),
        )
        .unwrap()
}

#[test]
fn a_moved_line_re_runs_the_matching_of_both_buyers() {
    // PRD FR-15.4, FR-12.19: اکرم's 10,000 receipt settled his oldest bill. When the line is no
    // longer his, nothing is left to settle: the receipt is credit. طارق's bill has no payment.
    let (w, one, _) = day();
    let tariq = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق",
        "Tariq",
        true,
        "",
    );
    assert_eq!(live_matches(&w, w.akram), (1, 10_000));
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(tariq);
    let edited = edit(&w, &opened, draft);
    assert_eq!(live_matches(&w, w.akram), (0, 0));
    assert_eq!(live_matches(&w, tariq), (0, 0));
    // Back: his receipt settles his bill again.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one.bikri_id,
            entry_id: edited.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(w.akram);
    edit(&w, &opened, draft);
    assert_eq!(live_matches(&w, w.akram), (1, 10_000));
    assert_eq!(live_matches(&w, tariq), (0, 0));
}

#[test]
fn the_review_of_a_buyer_change_lists_the_change_and_both_buyers_balances() {
    // PRD FR-7.70: the screen lists each change and its effect before the save.
    let (w, one, _) = day();
    let tariq = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق",
        "Tariq",
        true,
        "",
    );
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(tariq);
    let preview = edit::bikri_review(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft,
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let changes: Vec<(String, Option<i64>)> = preview
        .review
        .changes
        .iter()
        .map(|c| (c.field.clone(), c.place.line_no))
        .collect();
    assert!(
        changes
            .iter()
            .any(|(field, line)| field == "buyer" && *line == Some(1)),
        "the review does not list the buyer change: {changes:?}"
    );
    let party = |id: i64| {
        preview
            .review
            .effect
            .parties
            .iter()
            .find(|p| p.account.id == id)
            .map(|p| (p.before, p.after))
    };
    let akram = party(w.akram).expect("the review does not show اکرم");
    assert_eq!(
        (akram.0.side, akram.0.amount_paisa.rupees()),
        (BalanceSide::Dr, 2_880)
    );
    assert_eq!(
        (akram.1.side, akram.1.amount_paisa.rupees()),
        (BalanceSide::Cr, 10_000)
    );
    let new_buyer = party(tariq).expect("the review does not show طارق");
    assert_eq!(
        (new_buyer.1.side, new_buyer.1.amount_paisa.rupees()),
        (BalanceSide::Dr, 12_880)
    );
}

#[test]
fn a_closed_day_is_fixed_through_reopen_and_fix_with_a_buyer_change() {
    // PRD FR-21.4, FR-7.70: the same edit on a closed day goes through "reopen and fix".
    let (w, one, _) = day();
    let ctx = &w.env.ctx;
    let tariq = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق",
        "Tariq",
        true,
        "",
    );
    day::close(
        ctx,
        &w.owner,
        &mandi_services::core::vouchers::DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(tariq);
    let save_input = EditSaveInput {
        entry_id: opened.entry_id,
        version: opened.version,
        draft,
        reason: Some("غلط گاہک".into()),
        confirm_cash_below_zero: false,
    };
    // Refused while the day is closed.
    let refused = bikri::edit_save(ctx, &w.owner, &save_input).unwrap_err();
    assert_eq!(
        refused.code(),
        mandi_services::core::error::ErrorCode::ErrDateClosed
    );
    let fix = day::fix_open(
        ctx,
        &w.owner,
        &mandi_services::core::edit::DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: "غلط گاہک".into(),
        },
    )
    .unwrap();
    let saved = bikri::edit_save(ctx, &w.owner, &save_input).unwrap();
    assert!(saved.preview.errors.is_empty());
    day::fix_close(
        ctx,
        &w.owner,
        &mandi_services::core::edit::DayFixCloseInput { token: fix.token },
    )
    .unwrap();
    assert_eq!(balance(&w, w.akram), (BalanceSide::Cr, 10_000));
    assert_eq!(balance(&w, tariq), (BalanceSide::Dr, 12_880));
    assert_eq!(trial(&w), (95_048, 95_048));
    assert_eq!(
        day::status(ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-10-10")
    );
}

#[test]
fn a_cancelled_bikri_is_found_by_its_number_but_not_by_a_buyer() {
    // A cancelled بکری has no live line: no buyer has it any more (decision E1: the old party loses
    // the entry). Its number, day, truck, seller and item still find it.
    let (w, one, _) = day();
    let version = bikri::get(&w.env.ctx, one.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: one.bikri_id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                number: Some(1),
                ..Default::default()
            }
        ),
        [1]
    );
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                seller_id: Some(w.rahim),
                ..Default::default()
            }
        ),
        [1]
    );
    assert_eq!(
        find(
            &w,
            BikriFindInput {
                buyer_account_id: Some(w.akram),
                ..Default::default()
            }
        ),
        Vec::<i64>::new()
    );
}

#[test]
fn the_buyer_filter_is_one_search_on_the_buyer_index() {
    // Code Standards B18, NFR-SPD-24: finding a buyer's بکری does not read every بکری or every line.
    use mandi_services::db::bikri::saved::{ListFilter, list_query};
    let (w, _, _) = day();
    let conn = raw(&w.dir);
    for filter in [
        ListFilter {
            buyer_id: Some(w.akram),
            limit: 50,
            ..Default::default()
        },
        ListFilter {
            chukta_name_ur: Some("شاہد".into()),
            chukta_name_en: Some("shahid".into()),
            limit: 50,
            ..Default::default()
        },
    ] {
        let (sql, binds) = list_query(&filter);
        let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let plan: Vec<String> = statement
            .query_map(
                mandi_services::db::rusqlite::params_from_iter(binds.iter()),
                |r| r.get::<_, String>(3),
            )
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let text = plan.join(" | ");
        // The inner search uses the buyer index; no full scan of `bikri_sales` is in the plan. (The
        // list of چکتا buyers is read whole: its names are searched with LIKE '%..%'.)
        assert!(!text.contains("SCAN s2"), "{text}");
        assert!(!text.contains("SCAN bikri_sales"), "{text}");
    }
}

// ---- Plan 67 review: which side of a line an edit reads again (PRD FR-12.20, 10-11-2026) -------

/// The seller commission, the buyer commission and the buyer total of each line in force, in
/// rupees, in line order.
fn live(w: &World, bikri_id: i64) -> Vec<(i64, i64, i64)> {
    bikri::get(&w.env.ctx, bikri_id)
        .unwrap()
        .lots
        .iter()
        .flat_map(|lot| lot.sales.iter())
        .filter(|s| s.state == SaleState::InForce)
        .map(|s| {
            (
                s.seller_comm.amount.rupees(),
                s.buyer_comm.amount.rupees(),
                s.buyer_total.rupees(),
            )
        })
        .collect()
}

/// The Owner changes the default seller and buyer commission, from today.
fn set_defaults(w: &World, seller: &str, buyer: &str) {
    let c = rules::charges(&w.env.ctx).unwrap();
    let keep = |d: &mandi_services::core::rules::DefaultCharge, value: &str| DefaultChargeInput {
        version: d.version,
        method: d.method,
        value: value.into(),
    };
    rules::set_charges(
        &w.env.ctx,
        &w.owner,
        &ChargesInput {
            seller_commission: keep(&c.seller_commission, seller),
            buyer_commission: keep(&c.buyer_commission, buyer),
            laga_fruit: keep(&c.laga_fruit, &c.laga_fruit.value_text),
        },
    )
    .unwrap();
}

fn tariq(w: &World) -> i64 {
    support::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق",
        "Tariq",
        true,
        "",
    )
}

#[test]
fn a_seller_only_correction_keeps_the_buyers_bill_after_the_rules_changed() {
    // Umar, 10-11-2026: the defaults are 7% after the save and only the seller of the lot is
    // corrected (رحیم to کریم). Every buyer keeps his stored bill: اکرم stays 12,880. The seller
    // commission follows the rules: 12,000 × 7% = 840 and 17,400 × 7% = 1,218.
    let (w, one, _) = day();
    set_defaults(&w, "7", "7");
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    draft.lots[0].seller_account_id = Some(w.karim);
    let edited = edit(&w, &opened, draft);
    assert_eq!(
        live(&w, one.bikri_id),
        [(840, 720, 12_880), (1_218, 1_044, 18_684)]
    );
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 18_684));
    // کریم: his own 10,160, the seller credit 29,400 − 2,058 − 3,000 − 250 − 50 − 20 = 24,022, less
    // the 5,000 cash he was given.
    assert_eq!(balance(&w, w.karim), (BalanceSide::Cr, 29_182));
    // Shared income 2,058 + 1,764 + 400 + 20 = 4,242: 1,696 / 1,272 / 1,272 and the 2 left over to
    // the largest share.
    assert_eq!(edited.preview.summary.shared_income.rupees(), 4_242);
    assert_eq!(shares(&w, edited.entry_id), [1_698, 1_272, 1_272]);
    let (dr, cr) = trial(&w);
    assert_eq!(dr, cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

#[test]
fn a_buyer_only_correction_keeps_the_seller_commission_after_the_rules_changed() {
    // Umar, 10-11-2026: the defaults are 7% after the save and only the buyer of line 1 is
    // corrected (اکرم to طارق). The seller commission of that line stays 720 (رحیم's stored 6%);
    // the buyer side follows the rules: 12,000 × 7% = 840, so طارق pays 12,000 + 840 + 160.
    let (w, one, _) = day();
    let tariq = tariq(&w);
    set_defaults(&w, "7", "7");
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(tariq);
    let edited = edit(&w, &opened, draft);
    assert_eq!(
        live(&w, one.bikri_id),
        [(720, 840, 13_000), (1_044, 1_044, 18_684)]
    );
    assert_eq!(balance(&w, tariq), (BalanceSide::Dr, 13_000));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Cr, 10_000));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 18_684));
    // The seller credit is as saved, so رحیم's ledger does not move: 0 after his payment (E2E-7).
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Zero, 0));
    // Shared income 1,764 + 1,884 + 400 + 20 = 4,068: 1,627 / 1,220 / 1,220 and 1 left over.
    assert_eq!(edited.preview.summary.shared_income.rupees(), 4_068);
    assert_eq!(shares(&w, edited.entry_id), [1_628, 1_220, 1_220]);
    let (dr, cr) = trial(&w);
    assert_eq!(dr, cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

/// What a saved line was paid on the spot, in rupees.
fn paid_of(w: &World, bikri_id: i64, line: usize) -> i64 {
    bikri::get(&w.env.ctx, bikri_id)
        .unwrap()
        .lots
        .iter()
        .flat_map(|lot| lot.sales.iter())
        .filter(|s| s.state == SaleState::InForce)
        .nth(line)
        .map(|s| s.paid_paisa.rupees())
        .unwrap()
}

#[test]
fn a_line_paid_on_the_spot_moves_with_its_paid_or_pays_in_full_for_a_must_pay_buyer() {
    // PRD FR-12.44, FR-12.20: اکرم paid 5,000 of his 12,880 on the spot, so 7,880 is left.
    let w = worked_day();
    let cash = support::id_of(&w.env, "01010001");
    let mut first = support::event1(&w);
    first.lots[0].sales[0].paid = Some("5,000".into());
    let one = save(&w, &first);
    // Cash: 50,000 − 5,000 given to the seller + 5,000 paid on the spot.
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 50_000));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 7_880));
    let tariq = tariq(&w);

    // Only the seller is corrected: the line keeps its buyer, his bill and the 5,000.
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    draft.lots[0].seller_account_id = Some(w.karim);
    let after_seller = edit(&w, &opened, draft);
    assert_eq!(paid_of(&w, one.bikri_id, 0), 5_000);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 7_880));
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 50_000));

    // Only the buyer is corrected, to a buyer who may take credit: the paid goes with the line.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one.bikri_id,
            entry_id: after_seller.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(tariq);
    let to_tariq = edit(&w, &opened, draft);
    assert_eq!(paid_of(&w, one.bikri_id, 0), 5_000);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Zero, 0));
    assert_eq!(balance(&w, tariq), (BalanceSide::Dr, 7_880));
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 50_000));

    // Then to نقدی منڈی, who always pays in full: paid = the whole 12,880, whatever was typed.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one.bikri_id,
            entry_id: to_tariq.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(w.naqdi);
    edit(&w, &opened, draft);
    assert_eq!(paid_of(&w, one.bikri_id, 0), 12_880);
    assert_eq!(balance(&w, tariq), (BalanceSide::Zero, 0));
    assert_eq!(balance(&w, w.naqdi), (BalanceSide::Zero, 0));
    // The cash part follows: 50,000 + the 7,880 more that was paid on the spot.
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 57_880));
    let (dr, cr) = trial(&w);
    assert_eq!(dr, cr);
}

/// The open advance of a seller, in rupees.
fn open_advance(w: &World, seller: i64) -> i64 {
    assist::advances(&w.env.ctx, seller)
        .unwrap()
        .open_paisa
        .rupees()
}

#[test]
fn a_new_seller_with_an_open_advance_has_it_adjusted_when_the_tick_is_on() {
    // PRD FR-12.23, FR-15.8: کریم took 5,000 in advance. The lot's tick is on, so when he becomes
    // the seller his advance is adjusted by the seller credit, and رحیم's 15,000 opens again.
    let w = worked_day();
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: false,
            date: "10-10-2026".into(),
            seller_account_id: w.karim,
            amount: "5,000".into(),
            removed: Default::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
    let one = save(&w, &support::event1(&w));
    assert_eq!(
        (open_advance(&w, w.rahim), open_advance(&w, w.karim)),
        (0, 5_000)
    );
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    assert!(draft.lots[0].adjust_advance);
    draft.lots[0].seller_account_id = Some(w.karim);
    let edited = edit(&w, &opened, draft);
    assert_eq!(
        (open_advance(&w, w.rahim), open_advance(&w, w.karim)),
        (15_000, 0)
    );
    // And back to رحیم: his advance is adjusted again and کریم's opens again.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one.bikri_id,
            entry_id: edited.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].seller_account_id = Some(w.rahim);
    edit(&w, &opened, draft);
    assert_eq!(
        (open_advance(&w, w.rahim), open_advance(&w, w.karim)),
        (0, 5_000)
    );
    let (dr, cr) = trial(&w);
    assert_eq!(dr, cr);
}

#[test]
fn a_person_in_the_dead_amount_keeps_his_own_lines_but_no_line_moves_onto_him() {
    // PRD FR-10.10, A67: ارشد died after بکری 1. His own line may still be edited while the day is
    // open. A line that was اکرم's cannot be moved onto him, and a new line cannot be added for him.
    let (w, one, _) = day();
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.arshad,
            entry_date: "10-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();
    let opened = open(&w, &one);
    let preview_of = |draft: BikriDraft| {
        bikri::edit_preview(
            &w.env.ctx,
            &EditPreviewInput {
                entry_id: opened.entry_id,
                draft,
            },
        )
        .unwrap()
    };
    let buyer_field = |line: usize| format!("lots[0].sales[{line}].buyer_account_id");

    // Line 1 moved onto the dead buyer: refused at that line's buyer, and the save is refused too.
    let mut moved = opened.draft.clone();
    moved.lots[0].sales[0].buyer_account_id = Some(w.arshad);
    let preview = preview_of(moved.clone());
    assert!(
        support::has(
            &preview.errors,
            ErrorCode::ErrAccountInactive,
            &buyer_field(0)
        ),
        "{:?}",
        preview.errors
    );
    assert!(!support::has(
        &preview.errors,
        ErrorCode::ErrAccountInactive,
        &buyer_field(1)
    ));
    let refused = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft: moved,
            reason: Some("غلط گاہک".into()),
            confirm_cash_below_zero: false,
        },
    );
    assert!(refused.is_err());

    // A new line for him: refused. (Line 2 is cut to 25 so the lot has 5 left.)
    let mut added = opened.draft.clone();
    added.lots[0].sales[1].qty = "25".into();
    added.lots[0]
        .sales
        .push(support::sale("n1", w.arshad, "5", "580"));
    let preview = preview_of(added);
    assert!(
        support::has(
            &preview.errors,
            ErrorCode::ErrAccountInactive,
            &buyer_field(2)
        ),
        "{:?}",
        preview.errors
    );
    assert!(!support::has(
        &preview.errors,
        ErrorCode::ErrAccountInactive,
        &buyer_field(1)
    ));

    // His own line, with its rate corrected, and a correction of the seller alone: both allowed.
    let mut own = opened.draft.clone();
    own.lots[0].sales[1].seller_rate = "590".into();
    let preview = preview_of(own);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let mut seller_only = opened.draft.clone();
    seller_only.lots[0].seller_account_id = Some(w.karim);
    let preview = preview_of(seller_only);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
}

#[test]
fn the_review_tells_two_people_of_the_same_name_apart() {
    // PRD FR-7.70, 10.2: اسلم and اسلم BA are two people. Moving a line from one to the other is a
    // change, and the list shows the tag so the two do not read the same.
    let (w, one, _) = day();
    let plain = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "اسلم",
        "Aslam",
        true,
        "",
    );
    let tagged = support::add_account_tagged(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        ("اسلم", "Aslam"),
        (true, ""),
        "BA",
    );
    let opened = open(&w, &one);
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(plain);
    let moved = edit(&w, &opened, draft);
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one.bikri_id,
            entry_id: moved.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].buyer_account_id = Some(tagged);
    let preview = edit::bikri_review(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft,
        },
    )
    .unwrap();
    let change = preview
        .review
        .changes
        .iter()
        .find(|c| c.field == "buyer" && c.place.line_no == Some(1))
        .expect("moving the line from اسلم to اسلم BA is not in the list");
    assert_eq!(
        change.before,
        ChangeValue::Names {
            ur: "اسلم".into(),
            en: "Aslam".into()
        }
    );
    assert_eq!(
        change.after,
        ChangeValue::Names {
            ur: "اسلم BA".into(),
            en: "Aslam BA".into()
        }
    );
}
