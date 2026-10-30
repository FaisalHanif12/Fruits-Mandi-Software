//! Plan 40 T2 on the PRD worked day: the dead amount (ڈوبی رقم, PRD FR-10.10, 9.1, FR-19.9, E71 to
//! E73, A65 to A68; E2E-44). The tiles, the refusals and the trial balance live here; the R20 and
//! R38 rows are in `reports_dead.rs`. Every expected number is the PRD's (E2E-44) or the money
//! agent's worked case of plan 40. Nothing is changed to suit the code.
use super::*;
use mandi_reports::dashboard::{self, BackupFacts, DashboardTiles};
use mandi_services::core::dead::{DeadMarkInput, DeadPreviewInput, DeadReason, DeadUndoInput};
use mandi_services::{ServiceError, dead, partners};

fn tiles(w: &support::World) -> DashboardTiles {
    dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles
}

/// The five figures of PRD 9.1 that this plan changes, in rupees: buyers' dues, sellers' dues, dead
/// amount, how many are dead, total dues.
fn dues(w: &support::World) -> (i64, i64, i64, i64, i64) {
    let t = tiles(w);
    (
        t.buyers_dues_paisa.rupees(),
        t.sellers_dues_paisa.rupees(),
        t.dead_amount_paisa.rupees(),
        t.dead_count,
        t.total_dues_paisa.rupees(),
    )
}

fn mark_input(account: i64, date: &str, reason: DeadReason, note: &str) -> DeadMarkInput {
    DeadMarkInput {
        account_id: account,
        pukhta_person_key: None,
        entry_date: date.into(),
        reason,
        note: note.into(),
    }
}

fn mark_ok(w: &support::World, input: &DeadMarkInput) -> i64 {
    dead::mark(&w.env.ctx, &w.owner, input).unwrap().mark_id
}

fn totals_of_partners(w: &support::World) -> Vec<i64> {
    w.partners
        .iter()
        .map(|p| {
            partners::overview(&w.env.ctx)
                .unwrap()
                .into_iter()
                .find(|r| r.partner_id == *p)
                .unwrap()
                .total_paisa
                .rupees()
        })
        .collect()
}

fn trial(w: &support::World) -> (i64, i64) {
    let t = ledger::trial_balance(&w.env.ctx, None).unwrap();
    (t.total_dr.rupees(), t.total_cr.rupees())
}

fn is_active(w: &support::World, id: i64) -> bool {
    scalar(
        w,
        &format!("SELECT is_active FROM accounts WHERE id = {id}"),
    ) == 1
}

fn day_at(w: &support::World, day: u32) {
    set_today(&w.env, 2026, 10, day);
}

fn input_at(date: &str, mut input: ReceiptInput) -> ReceiptInput {
    input.date = date.into();
    input
}

/// Events 1 to 5 of the PRD (the world of E2E-44).
fn events_1_to_5() -> support::World {
    let w = worked_day();
    day_money::events_1_to_4(&w);
    event5_direct(&w);
    w
}

#[test]
fn e2e_44_the_move_the_recovery_the_tiles_and_the_refusals() {
    let w = events_1_to_5();
    let cash = support::id_of(&w.env, "01010001");
    // The end of 10-10-26: nobody is dead, so کل اگراہی is the buyers' dues.
    assert_eq!(dues(&w), (21_564, 0, 0, 0, 21_564));
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 58_484));
    assert_eq!(trial(&w), (85_048, 85_048));
    let partner_totals = totals_of_partners(&w);

    // 11-10-26: ارشد dies. The preview says what he owes and that he can be moved.
    day_at(&w, 11);
    let preview = dead::preview(
        &w.env.ctx,
        &DeadPreviewInput {
            account_id: w.arshad,
            pukhta_person_key: None,
        },
    )
    .unwrap();
    assert_eq!(
        (
            preview.owed_paisa.rupees(),
            preview.allowed,
            preview.reason_not_allowed,
            preview.last_posting_date.as_deref(),
            preview.linked.is_none()
        ),
        (18_684, true, None, Some("2026-10-10"), true)
    );
    let mark_id = mark_ok(
        &w,
        &mark_input(w.arshad, "11-10-2026", DeadReason::Died, "فوت ہو گئے"),
    );

    // After the move (E2E-44): گاہکوں کے ذمے 2,880, بیوپاریوں کے ذمے 0, ڈوبی رقم 18,684 for 1
    // person, کل اگراہی 21,564. Nothing posted: cash, the trial balance and every partner's
    // کل رقم are as they were (PRD A65).
    assert_eq!(dues(&w), (2_880, 0, 18_684, 1, 21_564));
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 58_484));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 18_684));
    assert_eq!(trial(&w), (85_048, 85_048));
    assert_eq!(totals_of_partners(&w), partner_totals);
    assert!(!is_active(&w, w.arshad), "the account is made inactive");
    assert!(is_active(&w, w.akram));
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'dead_mark'"
        ),
        1
    );

    // A new بکری line for him is refused as on any inactive account.
    let refused = bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrAccountInactive);
    // So is a payment (cash lent) to him.
    let refused = vouchers::save_payment(
        &w.env.ctx,
        &w.owner,
        &PaymentInput {
            date: "11-10-2026".into(),
            ..payment_input(w.arshad, "100", "نقد ادھار")
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrAccountInactive);
    // Nothing was written by the two refusals.
    assert_eq!(dues(&w), (2_880, 0, 18_684, 1, 21_564));
    assert_eq!(trial(&w), (85_048, 85_048));

    // 12-10-26: his son pays 4,000. A وصولی on him is still allowed, with normal matching.
    day_at(&w, 12);
    let saved = vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &input_at("12-10-2026", receipt_input(w.arshad, "4,000")),
    )
    .unwrap();
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(14_684))
    );
    // E2E-44: ڈوبی رقم 14,684, کل اگراہی 17,564, cash 62,484, the trial balance still 85,048.
    assert_eq!(dues(&w), (2_880, 0, 14_684, 1, 17_564));
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 62_484));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 14_684));
    assert_eq!(trial(&w), (85_048, 85_048));
    assert_eq!(totals_of_partners(&w), partner_totals);
    assert!(
        !is_active(&w, w.arshad),
        "a recovery does not make him active"
    );
    // The bill he owed is settled oldest first by the receipt.
    assert_eq!(open_bills_of(&w, w.arshad).open_paisa.rupees(), 14_684);

    // A recovery of one rupee more than he still owes is refused, with what is owed.
    let refused = vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &input_at("12-10-2026", receipt_input(w.arshad, "14,685")),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    assert_eq!(first_field(&refused).as_deref(), Some("amount"));
    match &refused {
        ServiceError::Rejected(items) => assert_eq!(
            items[0].values.get("open_paisa"),
            Some(&mandi_services::core::error::ErrorValue::Number(
                rupees(14_684).as_paisa()
            ))
        ),
        other => panic!("{other:?}"),
    }
    assert_eq!(dues(&w), (2_880, 0, 14_684, 1, 17_564));

    // Exactly what he owes is allowed: he owes 0, the move stays in the history, the tile is
    // 0 and nobody is counted.
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &input_at("12-10-2026", receipt_input(w.arshad, "14,684")),
    )
    .unwrap();
    assert_eq!(dues(&w), (2_880, 0, 0, 0, 2_880));
    assert_eq!(balance(&w, w.arshad).0, BalanceSide::Zero);
    assert_eq!(trial(&w), (85_048, 85_048));

    // E73: دوبارہ فعال کریں needs a reason; then he is active again.
    let refused = dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id,
            reason: "  ".into(),
        },
    )
    .unwrap_err();
    assert_eq!(
        (refused.code(), first_field(&refused).as_deref()),
        (ErrorCode::ErrReasonRequired, Some("reason"))
    );
    assert!(!is_active(&w, w.arshad));
    dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id,
            reason: "غلط اندراج".into(),
        },
    )
    .unwrap();
    assert!(is_active(&w, w.arshad));
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'dead_undo'"
        ),
        1
    );
    // A second undo of the same move is not found.
    let again = dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id,
            reason: "x".into(),
        },
    )
    .unwrap_err();
    assert_eq!(again.code(), ErrorCode::ErrNotFound);
}

#[test]
fn undo_puts_the_balance_back_into_the_normal_tiles() {
    let w = events_1_to_5();
    day_at(&w, 11);
    let mark_id = mark_ok(
        &w,
        &mark_input(w.arshad, "11-10-2026", DeadReason::Left, ""),
    );
    assert_eq!(dues(&w), (2_880, 0, 18_684, 1, 21_564));
    dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id,
            reason: "واپس آ گئے".into(),
        },
    )
    .unwrap();
    // Back in the normal tiles: 2,880 + 18,684; nobody dead; the total is the same.
    assert_eq!(dues(&w), (21_564, 0, 0, 0, 21_564));
    // He can be moved again later; the second move has its own amount.
    mark_ok(
        &w,
        &mark_input(w.arshad, "11-10-2026", DeadReason::Died, ""),
    );
    assert_eq!(dues(&w), (2_880, 0, 18_684, 1, 21_564));
}

#[test]
fn every_refusal_of_a_move_has_its_code_and_its_field() {
    let w = events_1_to_5();
    day_at(&w, 11);
    let code = |input: &DeadMarkInput| -> (ErrorCode, Option<String>) {
        let e = dead::mark(&w.env.ctx, &w.owner, input).unwrap_err();
        (e.code(), first_field(&e))
    };
    // Nothing owed: رحیم was paid in full (Event 4), کریم is owed by the firm.
    assert_eq!(
        code(&mark_input(w.rahim, "11-10-2026", DeadReason::Died, "")),
        (ErrorCode::ErrNothingOwed, Some("account_id".into()))
    );
    assert_eq!(
        code(&mark_input(w.karim, "11-10-2026", DeadReason::Died, "")),
        (ErrorCode::ErrNothingOwed, Some("account_id".into()))
    );
    // Never moved: a partner's capital, cash, the cash buyer.
    for account in [
        support::id_of(&w.env, "06020001"),
        support::id_of(&w.env, "01010001"),
        w.naqdi,
    ] {
        assert_eq!(
            code(&mark_input(account, "11-10-2026", DeadReason::Died, "")),
            (ErrorCode::ErrDeadNotAllowed, Some("account_id".into())),
            "account {account}"
        );
    }
    // A reason of other needs a note.
    assert_eq!(
        code(&mark_input(w.arshad, "11-10-2026", DeadReason::Other, "  ")),
        (ErrorCode::ErrNoteRequired, Some("note".into()))
    );
    // A پختہ کھاتہ account needs its person.
    assert_eq!(
        code(&mark_input(
            w.pukhta_buyer,
            "11-10-2026",
            DeadReason::Died,
            ""
        )),
        (
            ErrorCode::ErrPersonRequired,
            Some("pukhta_person_key".into())
        )
    );
    // A plain account refuses a person key.
    let mut with_key = mark_input(w.arshad, "11-10-2026", DeadReason::Died, "");
    with_key.pukhta_person_key = Some("x|1".into());
    assert_eq!(
        code(&with_key),
        (
            ErrorCode::ErrFormatInvalid,
            Some("pukhta_person_key".into())
        )
    );
    // Unknown account.
    assert_eq!(
        code(&mark_input(9_999_999, "11-10-2026", DeadReason::Died, "")),
        (ErrorCode::ErrNotFound, Some("account_id".into()))
    );
    // The date: not after today, not on a closed day, and not before his last posting.
    assert_eq!(
        code(&mark_input(w.arshad, "12-10-2026", DeadReason::Died, "")),
        (ErrorCode::ErrDateInFuture, Some("entry_date".into()))
    );
    assert_eq!(
        code(&mark_input(w.arshad, "09-10-2026", DeadReason::Died, "")),
        (ErrorCode::ErrDateClosed, Some("entry_date".into()))
    );
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &input_at("11-10-2026", receipt_input(w.arshad, "100")),
    )
    .unwrap();
    assert_eq!(
        code(&mark_input(w.arshad, "10-10-2026", DeadReason::Died, "")),
        (
            ErrorCode::ErrDateBeforeLastPosting,
            Some("entry_date".into())
        )
    );
    // Moved already.
    mark_ok(
        &w,
        &mark_input(w.arshad, "11-10-2026", DeadReason::Died, ""),
    );
    assert_eq!(
        code(&mark_input(w.arshad, "11-10-2026", DeadReason::Died, "")),
        (ErrorCode::ErrAlreadyDead, Some("account_id".into()))
    );
    // The preview of a person who cannot be moved says so with the code (not an error).
    let preview = dead::preview(
        &w.env.ctx,
        &DeadPreviewInput {
            account_id: w.arshad,
            pukhta_person_key: None,
        },
    )
    .unwrap();
    assert_eq!(
        (preview.allowed, preview.reason_not_allowed),
        (false, Some(ErrorCode::ErrAlreadyDead))
    );
    // An undo of a move that does not exist.
    let none = dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id: 9_999,
            reason: "x".into(),
        },
    )
    .unwrap_err();
    assert_eq!(
        (none.code(), first_field(&none).as_deref()),
        (ErrorCode::ErrNotFound, Some("mark_id"))
    );
    // Nothing from the refusals changed the books.
    assert_eq!(trial(&w).0, trial(&w).1);
}

#[test]
fn a_dead_seller_is_netted_and_the_money_agent_invariant_holds() {
    // E2E-28 setup: the advance tick is off, so رحیم has a payable of 19,316 and an open advance
    // of 15,000. He is paid 19,316: ledger 15,000 بنام, advance open 15,000.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save_ok(&w, &draft, false);
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment_input(w.rahim, "19,316", "")).unwrap();
    day_at(&w, 11);

    // The invariant of the money agent: کل اگراہی + پیشگی باقی + ادھار رقم باقی − بیوپاریوں کو
    // ادائیگی باقی is the same before and after any move or undo (a seller's balance is his
    // advance less his payable plus what he owes beyond his advance).
    let invariant = |w: &support::World| {
        let t = tiles(w);
        t.total_dues_paisa.rupees() + t.advances_open_paisa.rupees() + t.lent_open_paisa.rupees()
            - t.sellers_payable_paisa.rupees()
    };
    let before = invariant(&w);
    assert_eq!(
        (
            tiles(&w).advances_open_paisa.rupees(),
            tiles(&w).sellers_payable_paisa.rupees()
        ),
        (15_000, 0)
    );

    let preview = dead::preview(
        &w.env.ctx,
        &DeadPreviewInput {
            account_id: w.rahim,
            pukhta_person_key: None,
        },
    )
    .unwrap();
    assert_eq!(preview.owed_paisa.rupees(), 15_000);
    let mark_id = mark_ok(&w, &mark_input(w.rahim, "11-10-2026", DeadReason::Left, ""));

    // A66: his whole ledger balance joins ڈوبی رقم and کل اگراہی; his advance leaves پیشگی باقی.
    let t = tiles(&w);
    assert_eq!(
        (
            t.dead_amount_paisa.rupees(),
            t.advances_open_paisa.rupees(),
            t.sellers_payable_paisa.rupees(),
            t.total_dues_paisa.rupees()
        ),
        // Only بکری 1 is saved here: اکرم 12,880 + ارشد 18,684 = 31,564 of buyers' dues.
        (15_000, 0, 0, 31_564 + 15_000)
    );
    assert_eq!(invariant(&w), before);
    dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id,
            reason: "x".into(),
        },
    )
    .unwrap();
    assert_eq!(invariant(&w), before);
    assert_eq!(tiles(&w).advances_open_paisa.rupees(), 15_000);
}

#[test]
fn a_one_time_person_is_marked_by_his_key_and_the_account_stays_active() {
    use mandi_services::core::bikri::{LineChargeMethod, PaymentType};
    let w = worked_day();
    let line = |key: &str, qty: &str, name_ur: &str, name_en: &str, phone: &str| {
        let mut l = support::sale(key, w.pukhta_buyer, qty, "1000");
        l.pukhta = Some(PukhtaInput {
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            phone: phone.into(),
        });
        l.payment_type = Some(PaymentType::Credit);
        l.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        l.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        l.laga = support::typed(LineChargeMethod::PerUnit, "0");
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "3");
    lot.sales = vec![
        line("s-1", "1", "شاہد", "Shahid", "0300-0000000"),
        line("s-2", "2", "ناصر", "Nasir", "0300-1111111"),
    ];
    save_ok(&w, &support::draft(vec![lot]), false);
    let shahid = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    let nasir = mandi_services::core::text::person_key("ناصر", "0300-1111111");
    assert_eq!(dues(&w), (3_000, 0, 0, 0, 3_000));

    day_at(&w, 11);
    let mut input = mark_input(w.pukhta_buyer, "11-10-2026", DeadReason::Died, "");
    input.pukhta_person_key = Some("نامعلوم|03000000000".into());
    assert_eq!(
        dead::mark(&w.env.ctx, &w.owner, &input).unwrap_err().code(),
        ErrorCode::ErrNotFound
    );
    input.pukhta_person_key = Some(nasir.clone());
    let preview = dead::preview(
        &w.env.ctx,
        &DeadPreviewInput {
            account_id: w.pukhta_buyer,
            pukhta_person_key: Some(nasir.clone()),
        },
    )
    .unwrap();
    assert_eq!(preview.owed_paisa.rupees(), 2_000);
    mark_ok(&w, &input);

    // Only ناصر's key is marked: the account stays active, شاہد is still in the dues.
    assert!(is_active(&w, w.pukhta_buyer));
    assert_eq!(dues(&w), (1_000, 0, 2_000, 1, 3_000));
    // Marking him again names the person.
    let again = dead::mark(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(
        (again.code(), first_field(&again).as_deref()),
        (ErrorCode::ErrPersonInDeadAmount, Some("pukhta_person_key"))
    );
    // A new line with his key is refused; a line for شاہد is not.
    let mut lot = support::bare_lot(&w, "lot-2", w.rahim, w.banana, "2");
    lot.sales = vec![line("s-3", "1", "ناصر", "Nasir", "0300-1111111")];
    let refused = bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: support::draft(vec![lot]),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrPersonInDeadAmount);
    let mut lot = support::bare_lot(&w, "lot-3", w.rahim, w.banana, "1");
    lot.sales = vec![line("s-4", "1", "شاہد", "Shahid", "0300-0000000")];
    save_ok(&w, &support::draft(vec![lot]), false);
    assert_eq!(dues(&w), (2_000, 0, 2_000, 1, 4_000));

    // A receipt for ناصر is a recovery: allowed, capped at 2,000; one for شاہد is a normal one.
    let mut over = input_at("11-10-2026", receipt_input(w.pukhta_buyer, "2,001"));
    over.person_key = Some(nasir.clone());
    assert_eq!(
        vouchers::save_receipt(&w.env.ctx, &w.owner, &over)
            .unwrap_err()
            .code(),
        ErrorCode::ErrReturnExceedsOpen
    );
    over.amount = "500".into();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &over).unwrap();
    assert_eq!(dues(&w), (2_000, 0, 1_500, 1, 3_500));
    let mut other = input_at("11-10-2026", receipt_input(w.pukhta_buyer, "1,000"));
    other.person_key = Some(shahid);
    vouchers::save_receipt(&w.env.ctx, &w.owner, &other).unwrap();
    assert_eq!(dues(&w), (1_000, 0, 1_500, 1, 2_500));
}

#[test]
fn a67_an_edit_of_an_earlier_posting_with_a_dead_buyers_unchanged_line_is_allowed() {
    use mandi_services::core::bikri::{EditOpenInput, EditPreviewInput, EditSaveInput};
    let w = worked_day();
    let saved = save_ok(&w, &support::event1(&w), false);
    day_at(&w, 11);
    mark_ok(
        &w,
        &mark_input(w.arshad, "11-10-2026", DeadReason::Died, ""),
    );
    assert!(!is_active(&w, w.arshad));
    let before = trial(&w);

    // E2E-13 on top of the move: اکرم's rate 600 -> 610. ارشد's line is unchanged and stays on his
    // (inactive) account; the edit is a reversal plus a new posting and it saves.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    let preview = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("ریٹ کی درستی".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // اکرم 13,092 (E2E-13); ارشد's 18,684 is unchanged and still dead; the books balance.
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 13_092));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 18_684));
    assert!(!is_active(&w, w.arshad));
    let after = trial(&w);
    assert_eq!(after.0, after.1);
    assert_ne!(after, before, "the edit changed اکرم's bill");
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 18_684);
    // The edit nets to a correction of 0 for him.
    let page = mandi_reports::dead::page(
        w.env.ctx.db(),
        &mandi_services::core::dead::DeadAmountInput {
            filter: mandi_services::core::dead::DeadFilter::Owed,
            search: String::new(),
            offset: 0,
            limit: 10,
        },
    )
    .unwrap();
    assert_eq!(
        (
            page.rows[0].recovered_paisa.rupees(),
            page.rows[0].corrections_paisa,
            page.rows[0].still_owed_paisa.rupees()
        ),
        (0, 0, 18_684)
    );
}
