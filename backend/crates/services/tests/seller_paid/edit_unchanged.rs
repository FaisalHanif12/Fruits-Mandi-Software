//! In an edit, the question "paid on the spot above today's credit" is asked only when that seller's
//! amount changed from what was saved (PRD FR-12.23, FR-6.7; plan 91 follow-up T3, Umar 22-11-2026).
//!
//! The worked check: رحیم owes 30,000. A بکری with one lot of credit 20,000 is saved with رحیم paid
//! 25,000 (above 20,000, so the save asked and was confirmed). Then:
//!
//! * an edit of only a buyer rate: his amount is still 25,000, so no seller question;
//! * an edit of his amount to 26,000: asked, and refused until confirmed;
//! * an edit that leaves the saved amount: not asked, whatever else it changes;
//! * cash below zero (FR-6.7) is not part of this rule and asks as always.
//!
//! The live preview follows the same rule, because the screen opens its yes or no box from the
//! preview before it saves (`needs_confirm` of the seller's summary).

use super::*;

/// What an edit needs to be sent: the saved بکری's ids.
struct Posting {
    bikri_id: i64,
    entry_id: i64,
}

/// رحیم owes 30,000; a بکری of credit 20,000 saved with him paid 25,000, confirmed at the save.
fn saved_with_25000(w: &World) -> Posting {
    owes(w, "30,000");
    let draft = rahim_draft(w, 20, Some("25000"));
    let refused = save_confirmed(w, &draft, false).unwrap_err();
    assert!(asks_seller(&refused), "{refused:?}");
    let saved = save_confirmed(w, &draft, true).unwrap();
    assert_eq!(balance(w, w.rahim), (BalanceSide::Dr, 35_000));
    Posting {
        bikri_id: saved.bikri_id,
        entry_id: saved.entry_id,
    }
}

/// The posting as the edit screen opens it, and its version.
fn open(w: &World, p: &Posting) -> (BikriDraft, i64) {
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: p.bikri_id,
            entry_id: p.entry_id,
        },
    )
    .unwrap();
    (opened.draft, opened.version)
}

/// Saves the edit. A saved edit puts a new posting in the place of the old one, so the entry the next
/// edit is about is the new one.
fn edit(
    w: &World,
    p: &mut Posting,
    version: i64,
    draft: &BikriDraft,
    confirm: bool,
) -> Result<mandi_services::core::bikri::BikriEdited, ServiceError> {
    let done = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: p.entry_id,
            version,
            draft: draft.clone(),
            reason: Some("a change".into()),
            confirm_cash_below_zero: confirm,
        },
    )?;
    p.entry_id = done.entry_id;
    Ok(done)
}

fn preview(w: &World, p: &Posting, draft: &BikriDraft) -> BikriPreview {
    let answer = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: p.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(answer.errors.is_empty(), "{:?}", answer.errors);
    answer
}

fn codes(error: &ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|i| (i.code, i.field.clone())).collect(),
        other => panic!("not a refusal: {other:?}"),
    }
}

fn asks_seller(error: &ServiceError) -> bool {
    codes(error)
        .iter()
        .any(|(code, _)| *code == ErrorCode::ErrSellerPaidAboveToday)
}

fn asks_cash(error: &ServiceError) -> bool {
    codes(error)
        .iter()
        .any(|(code, _)| *code == ErrorCode::ErrCashBelowZeroNotConfirmed)
}

/// The buyer's rate of the one line changed (the seller's rate and credit stay as they are).
fn change_buyer_rate(draft: &mut BikriDraft, rate: &str) {
    let sale = &mut draft.lots[0].sales[0];
    sale.rate_differs = true;
    sale.buyer_rate = Some(rate.into());
}

fn set_cash(draft: &mut BikriDraft, rupees: &str) {
    draft.lots[0].cash_paid = Some(rupees.into());
}

fn sellers_asking(p: &BikriPreview) -> Vec<i64> {
    p.sellers
        .iter()
        .filter(|s| s.needs_confirm)
        .map(|s| s.seller_account_id)
        .collect()
}

#[test]
fn an_edit_of_only_a_buyer_rate_leaves_his_amount_and_asks_no_seller_question() {
    let w = rich_world();
    let mut p = saved_with_25000(&w);
    let (mut draft, version) = open(&w, &p);
    change_buyer_rate(&mut draft, "1010");
    // The live answer, which the screen opens its box from, does not ask either.
    let live = preview(&w, &p, &draft);
    let s = seller(&live, w.rahim);
    assert_eq!(rupees_of(s.paid_paisa), 25_000);
    assert_eq!(rupees_of(s.credit_paisa), 20_000);
    assert!(!s.needs_confirm, "the amount is the saved one");
    // The save does not ask.
    edit(&w, &mut p, version, &draft, false).unwrap();
    // Nothing of his money changed: 30,000 + 25,000 - 20,000.
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 35_000));
}

#[test]
fn changing_his_amount_to_26000_asks_until_confirmed_and_writes_nothing_before_that() {
    let w = rich_world();
    let mut p = saved_with_25000(&w);
    let (mut draft, version) = open(&w, &p);
    set_cash(&mut draft, "26000");
    let live = preview(&w, &p, &draft);
    assert!(seller(&live, w.rahim).needs_confirm);
    let refused = edit(&w, &mut p, version, &draft, false).unwrap_err();
    assert!(asks_seller(&refused), "{refused:?}");
    assert!(
        codes(&refused).contains(&(
            ErrorCode::ErrSellerPaidAboveToday,
            Some("sellers[0].paid".to_owned())
        )),
        "{refused:?}"
    );
    assert_eq!(
        balance(&w, w.rahim),
        (BalanceSide::Dr, 35_000),
        "nothing written"
    );
    edit(&w, &mut p, version, &draft, true).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 36_000));
}

#[test]
fn the_saved_amount_moves_with_each_edit_and_only_a_changed_amount_is_asked() {
    let w = rich_world();
    let mut p = saved_with_25000(&w);
    // 26,000 asked and confirmed: the saved amount is now 26,000.
    let (mut draft, version) = open(&w, &p);
    set_cash(&mut draft, "26000");
    assert!(asks_seller(
        &edit(&w, &mut p, version, &draft, false).unwrap_err()
    ));
    edit(&w, &mut p, version, &draft, true).unwrap();
    // An edit of a buyer rate leaves 26,000: not asked.
    let (mut draft, version) = open(&w, &p);
    change_buyer_rate(&mut draft, "1020");
    assert!(!seller(&preview(&w, &p, &draft), w.rahim).needs_confirm);
    edit(&w, &mut p, version, &draft, false).unwrap();
    // Back to 25,000 is a change from the saved 26,000: asked, and saved once confirmed.
    let (mut draft, version) = open(&w, &p);
    set_cash(&mut draft, "25000");
    assert!(seller(&preview(&w, &p, &draft), w.rahim).needs_confirm);
    let refused = edit(&w, &mut p, version, &draft, false).unwrap_err();
    assert!(asks_seller(&refused), "{refused:?}");
    edit(&w, &mut p, version, &draft, true).unwrap();
    // Now 25,000 is the saved amount: a further edit that leaves it is not asked.
    let (mut draft, version) = open(&w, &p);
    change_buyer_rate(&mut draft, "1030");
    assert!(!seller(&preview(&w, &p, &draft), w.rahim).needs_confirm);
    edit(&w, &mut p, version, &draft, false).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 35_000));
}

#[test]
fn a_refused_edit_does_not_change_what_is_saved() {
    let w = rich_world();
    let mut p = saved_with_25000(&w);
    // 26,000 is asked and declined (nothing saved).
    let (mut draft, version) = open(&w, &p);
    set_cash(&mut draft, "26000");
    assert!(asks_seller(
        &edit(&w, &mut p, version, &draft, false).unwrap_err()
    ));
    // A new edit that goes back to the saved 25,000 is not asked.
    let (mut draft, version) = open(&w, &p);
    change_buyer_rate(&mut draft, "1010");
    set_cash(&mut draft, "25000");
    edit(&w, &mut p, version, &draft, false).unwrap();
}

#[test]
fn the_spread_command_of_an_edit_follows_the_same_rule() {
    let w = rich_world();
    let p = saved_with_25000(&w);
    let (draft, _) = open(&w, &p);
    // The saved amount asked again: not a question.
    let same = spread(&w, &draft, Some(p.entry_id), &[(w.rahim, "25000")]);
    assert_eq!(rupees_of(seller(&same.preview, w.rahim).paid_paisa), 25_000);
    assert!(!seller(&same.preview, w.rahim).needs_confirm);
    // Another amount above the limit: a question.
    let more = spread(&w, &draft, Some(p.entry_id), &[(w.rahim, "26000")]);
    assert!(seller(&more.preview, w.rahim).needs_confirm);
    // A new بکری is as before: 25,000 above the 20,000 credit asks.
    let new = spread(&w, &rahim_draft(&w, 20, None), None, &[(w.rahim, "25000")]);
    assert!(seller(&new.preview, w.rahim).needs_confirm);
}

#[test]
fn a_seller_with_no_saved_lot_is_asked_like_a_new_bikri() {
    let w = rich_world();
    let mut p = saved_with_25000(&w);
    let (mut draft, version) = open(&w, &p);
    // A new lot of کریم (credit 20,000) with 25,000 paid: his saved amount is 0, so this is a change.
    draft.lots.push(credit_lot(
        &w,
        "lot-2",
        w.karim,
        "20",
        "1000",
        Some("25000"),
    ));
    let live = preview(&w, &p, &draft);
    assert_eq!(sellers_asking(&live), vec![w.karim]);
    let refused = edit(&w, &mut p, version, &draft, false).unwrap_err();
    let list = codes(&refused);
    assert!(
        list.contains(&(
            ErrorCode::ErrSellerPaidAboveToday,
            Some("sellers[1].paid".to_owned())
        )),
        "{list:?}"
    );
    assert!(
        !list.contains(&(
            ErrorCode::ErrSellerPaidAboveToday,
            Some("sellers[0].paid".to_owned())
        )),
        "{list:?}"
    );
    edit(&w, &mut p, version, &draft, true).unwrap();
}

#[test]
fn cash_below_zero_still_asks_when_his_amount_is_unchanged() {
    // 100,000 in hand: the advance of 30,000 and رحیم's 25,000 leave 45,000.
    let w = support::world_with(100_000, true);
    let mut p = saved_with_25000(&w);
    assert_eq!(cash(&w), (BalanceSide::Dr, 45_000));
    let (mut draft, version) = open(&w, &p);
    // A new lot of کریم takes 50,000 on the spot (credit 60,000, so no seller question): cash would
    // end at -5,000. رحیم's amount stays 25,000.
    draft.lots.push(credit_lot(
        &w,
        "lot-2",
        w.karim,
        "60",
        "1000",
        Some("50000"),
    ));
    let live = preview(&w, &p, &draft);
    assert!(live.cash.needs_confirm);
    assert!(sellers_asking(&live).is_empty(), "{:?}", live.sellers);
    let refused = edit(&w, &mut p, version, &draft, false).unwrap_err();
    assert!(asks_cash(&refused), "{refused:?}");
    assert!(!asks_seller(&refused), "{refused:?}");
    edit(&w, &mut p, version, &draft, true).unwrap();
}

#[test]
fn cash_moved_between_two_lots_of_one_seller_with_the_same_sum_is_not_asked() {
    let w = rich_world();
    owes(&w, "30,000");
    // Credits 12,000 + 8,000 = 20,000; نقد 13,000 + 12,000 = 25,000 (asked and confirmed).
    let draft = support::draft(vec![
        credit_lot(&w, "lot-1", w.rahim, "12", "1000", Some("13000")),
        credit_lot(&w, "lot-2", w.rahim, "8", "1000", Some("12000")),
    ]);
    let refused = save_confirmed(&w, &draft, false).unwrap_err();
    assert!(asks_seller(&refused), "{refused:?}");
    let saved = save_confirmed(&w, &draft, true).unwrap();
    let mut p = Posting {
        bikri_id: saved.bikri_id,
        entry_id: saved.entry_id,
    };
    let (mut draft, version) = open(&w, &p);
    // The money moves from one lot to the other. His sum is the saved 25,000.
    draft.lots[0].cash_paid = Some("12000".into());
    draft.lots[1].cash_paid = Some("13000".into());
    let live = preview(&w, &p, &draft);
    assert_eq!(rupees_of(seller(&live, w.rahim).paid_paisa), 25_000);
    assert!(sellers_asking(&live).is_empty(), "{:?}", live.sellers);
    edit(&w, &mut p, version, &draft, false).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 35_000));
}

/// The seller's rate of the one line changed. With the buyer rate following it, his credit for the
/// day moves with it (qty 20, commission typed at 0%): rate 750 gives a credit of 15,000.
fn change_seller_rate(draft: &mut BikriDraft, rate: &str) {
    draft.lots[0].sales[0].seller_rate = rate.into();
}

// ---- Umar, 22-11-2026: an unchanged amount is asked again when the edit lowers his credit ----

#[test]
fn an_unchanged_amount_is_asked_again_when_the_edit_lowers_his_credit_below_it() {
    let w = rich_world();
    let mut p = saved_with_25000(&w);
    let (mut draft, version) = open(&w, &p);
    // Credit 20,000 -> 15,000, paid still 25,000: he owes 40,000 instead of 35,000.
    change_seller_rate(&mut draft, "750");
    let live = preview(&w, &p, &draft);
    let s = seller(&live, w.rahim);
    assert_eq!(rupees_of(s.credit_paisa), 15_000);
    assert_eq!(rupees_of(s.paid_paisa), 25_000);
    assert!(s.needs_confirm, "the credit went down and he owes more");
    let refused = edit(&w, &mut p, version, &draft, false).unwrap_err();
    assert!(
        codes(&refused).contains(&(
            ErrorCode::ErrSellerPaidAboveToday,
            Some("sellers[0].paid".to_owned())
        )),
        "{refused:?}"
    );
    assert_eq!(
        balance(&w, w.rahim),
        (BalanceSide::Dr, 35_000),
        "nothing written"
    );
    edit(&w, &mut p, version, &draft, true).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 40_000));
    // The saved credit is now 15,000: an edit that leaves it is not asked again.
    let (mut draft, version) = open(&w, &p);
    change_buyer_rate(&mut draft, "1010");
    assert!(!seller(&preview(&w, &p, &draft), w.rahim).needs_confirm);
    edit(&w, &mut p, version, &draft, false).unwrap();
}

#[test]
fn an_amount_that_was_never_asked_is_asked_when_the_edit_lowers_his_credit_below_it() {
    let w = rich_world();
    owes(&w, "30,000");
    // Paid 15,000 on credit 20,000: under the limit, so the save did not ask.
    let saved = save_confirmed(&w, &rahim_draft(&w, 20, Some("15000")), false).unwrap();
    let mut p = Posting {
        bikri_id: saved.bikri_id,
        entry_id: saved.entry_id,
    };
    let (mut draft, version) = open(&w, &p);
    // Credit 20,000 -> 10,000: 15,000 is now above it.
    change_seller_rate(&mut draft, "500");
    let live = preview(&w, &p, &draft);
    assert_eq!(rupees_of(seller(&live, w.rahim).credit_paisa), 10_000);
    assert!(seller(&live, w.rahim).needs_confirm);
    let refused = edit(&w, &mut p, version, &draft, false).unwrap_err();
    assert!(asks_seller(&refused), "{refused:?}");
    edit(&w, &mut p, version, &draft, true).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 35_000));
}

#[test]
fn a_raised_credit_or_a_lowered_credit_that_stays_above_his_amount_asks_nothing() {
    let w = rich_world();
    owes(&w, "30,000");
    let saved = save_confirmed(&w, &rahim_draft(&w, 20, Some("15000")), false).unwrap();
    let mut p = Posting {
        bikri_id: saved.bikri_id,
        entry_id: saved.entry_id,
    };
    // Credit 20,000 -> 16,000: lowered, but still 15,000 or more, so he is not over his limit.
    let (mut draft, version) = open(&w, &p);
    change_seller_rate(&mut draft, "800");
    assert!(!seller(&preview(&w, &p, &draft), w.rahim).needs_confirm);
    edit(&w, &mut p, version, &draft, false).unwrap();
    // A credit raised from 16,000 to 22,000 with the amount unchanged: no question.
    let (mut draft, version) = open(&w, &p);
    change_seller_rate(&mut draft, "1100");
    let live = preview(&w, &p, &draft);
    assert_eq!(rupees_of(seller(&live, w.rahim).credit_paisa), 22_000);
    assert!(!seller(&live, w.rahim).needs_confirm);
    edit(&w, &mut p, version, &draft, false).unwrap();
}

#[test]
fn an_amount_confirmed_above_his_credit_stays_unasked_when_the_credit_is_raised() {
    let w = rich_world();
    let mut p = saved_with_25000(&w);
    // Credit 20,000 -> 23,000: still under 25,000, but not lowered and the amount is the saved one.
    let (mut draft, version) = open(&w, &p);
    change_seller_rate(&mut draft, "1150");
    let live = preview(&w, &p, &draft);
    assert_eq!(rupees_of(seller(&live, w.rahim).credit_paisa), 23_000);
    assert!(!seller(&live, w.rahim).needs_confirm);
    edit(&w, &mut p, version, &draft, false).unwrap();
}
