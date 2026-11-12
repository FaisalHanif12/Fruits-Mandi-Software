//! Plan 44 T2c on the PRD worked day: E2E-45, an entry of a closed day is fixed through "reopen
//! and fix" (PRD FR-21.4, FR-7.70). The expected numbers are the PRD's (E2E-13); they are never
//! changed to suit the code.
//!
//! After Events 1 to 5 the day 10-10-26 is closed and today is 11-10-26. اکرم was charged 600 but
//! the right rate was 610. The edit is refused while the day is closed; `day_fix_open` reopens it
//! with a reason; the review shows what the edit does (اکرم +212, الف +10); the save gives every
//! figure of E2E-13; `day_fix_close` closes the day again.

use mandi_services::core::bikri::{EditOpenInput, EditPreviewInput, EditSaveInput};
use mandi_services::core::edit::{DayFixCloseInput, DayFixOpenInput};
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::edit;

use super::day_close::write_or_check;
use super::dead_amount::events_1_to_5;
use super::own_goods::trial;
use super::*;

/// The request and the real answer of a command, as the two files the frontend copies.
fn pair<A: serde::Serialize>(name: &str, request: &serde_json::Value, answer: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

fn audit_actions(w: &support::World) -> Vec<String> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT action FROM audit_log
             WHERE entity IN ('day', 'bikri') AND action IN
                   ('day_close', 'day_reopen', 'day_fix_open', 'day_fix_close', 'edit')
             ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn e2e_45_an_entry_of_a_closed_day_is_fixed_through_reopen_and_fix() {
    let w = events_1_to_5();
    let ctx = &w.env.ctx;
    // The day is closed at night; the next morning the mistake is found.
    day::close(
        ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    assert_eq!(
        day::status(ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-10-10")
    );
    // The rows of the closed day still offer Edit and Cancel, through a reopen; nothing was edited.
    let detail = bikri::get(ctx, 1).unwrap();
    assert!(detail.flags.can_edit && detail.flags.can_cancel && detail.flags.needs_reopen);
    assert!(detail.flags.locked_reason.is_none() && detail.flags.edited.is_none());
    let listed = bikri::list(ctx, &Default::default()).unwrap();
    assert!(
        listed
            .rows
            .iter()
            .all(|r| r.flags.can_edit && r.flags.needs_reopen && r.flags.edited.is_none())
    );
    let receipts = vouchers::list(
        ctx,
        &mandi_services::core::vouchers::VouchersListInput {
            sort: None,
            group_by_month: false,
            search: None,
            show_finished: true,
            account_id: None,
            voucher_type: None,
            from: String::new(),
            to: String::new(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert!(
        receipts
            .rows
            .iter()
            .all(|r| r.flags.can_edit && r.flags.needs_reopen)
    );
    let saved_entry: i64 = scalar(
        &w,
        "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first'",
    );
    let opened = bikri::open_for_edit(
        ctx,
        &EditOpenInput {
            bikri_id: 1,
            entry_id: saved_entry,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    draft.lots[0].sales[0].buyer_rate = Some("610".into());
    let save_input = EditSaveInput {
        entry_id: opened.entry_id,
        version: opened.version,
        draft: draft.clone(),
        reason: Some("اکرم کا ریٹ غلط تھا".to_owned()),
        confirm_cash_below_zero: false,
    };

    // The day is closed: the edit is refused, and the review says a reopen is needed.
    let refused = bikri::edit_save(ctx, &w.owner, &save_input).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrDateClosed);
    let review = edit::bikri_review(
        ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(review.review.needs_reopen && review.review.reason_required);

    // A token that is not the right one, or none given, does nothing.
    let no_fix = day::fix_close(
        ctx,
        &w.owner,
        &DayFixCloseInput {
            token: "fix-0000".into(),
        },
    )
    .unwrap_err();
    assert_eq!(no_fix.code(), ErrorCode::ErrFixTokenInvalid);

    // Reopen with a reason. A reason is required; the open day is not a closed one.
    let no_reason = day::fix_open(
        ctx,
        &w.owner,
        &DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: " ".into(),
            sign: "adnan".into(),
        },
    )
    .unwrap_err();
    assert_eq!(no_reason.code(), ErrorCode::ErrReasonRequired);
    let fix = day::fix_open(
        ctx,
        &w.owner,
        &DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: "اکرم کا ریٹ غلط تھا".into(),
            sign: "adnan".into(),
        },
    )
    .unwrap();
    pair(
        "day_fix_open_e2e_45",
        &serde_json::json!({ "date": "10-10-2026", "reason": "اکرم کا ریٹ غلط تھا", "sign": "adnan" }),
        &fix,
    );
    assert_eq!(fix.days, ["2026-10-10"]);
    assert_eq!(fix.closed_up_to.as_deref(), Some("2026-10-09"));
    assert!(fix.token.starts_with("fix-") && fix.token.len() > 20);
    let not_closed = day::fix_open(
        ctx,
        &w.owner,
        &DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: "x".into(),
            sign: "adnan".into(),
        },
    )
    .unwrap_err();
    assert_eq!(not_closed.code(), ErrorCode::ErrDayNotClosed);

    // The review of the open day: اکرم 12,880 - 10,000 = 2,880 owed becomes 3,092 owed (+212), and
    // الف's share of the posting 1,580 becomes 1,590 (+10). The commission moves by 24 (12 + 12).
    let review = edit::bikri_review(
        ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(review.errors.is_empty(), "{:?}", review.errors);
    assert!(!review.review.needs_reopen && review.review.reason_required);
    let akram = review
        .review
        .effect
        .parties
        .iter()
        .find(|p| p.account.id == w.akram)
        .unwrap();
    assert_eq!(
        (akram.before.amount_paisa, akram.after.amount_paisa),
        (rupees(2_880), rupees(3_092))
    );
    let alif = &review.review.effect.partners[0];
    assert_eq!(
        (alif.before_paisa, alif.after_paisa),
        (rupees(1_580), rupees(1_590))
    );

    // The save: every figure of E2E-13, the date stays 10-10-26.
    let edited = bikri::edit_save(ctx, &w.owner, &save_input).unwrap();
    let summary = &edited.preview.summary;
    assert_eq!(
        (
            summary.gross.rupees(),
            summary.seller_comm.rupees(),
            summary.deductions.rupees(),
            summary.net.rupees(),
            summary.shared_income.rupees()
        ),
        (29_600, 1_776, 10_096, 19_504, 3_972)
    );
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM journal_entries WHERE id IN ({}, {}) AND entry_date = '2026-10-10'",
                edited.entry_id, edited.reversal_entry_id
            )
        ),
        2
    );
    let line_totals: Vec<i64> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare("SELECT buyer_total_paisa FROM bikri_sales WHERE is_active = 1 ORDER BY id")
            .unwrap();
        statement
            .query_map([], |r| r.get::<_, i64>(0))
            .unwrap()
            .map(|v| v.unwrap() / 100)
            .collect()
    };
    // شاہد's 12,800 of Event 2, and Event 1's two lines (اکرم 13,092 now); order is by row.
    let mut line_totals = line_totals;
    line_totals.sort_unstable();
    assert_eq!(line_totals, [12_800, 13_092, 18_684]);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 3_092));

    // The days are still open until the fix is closed; a token that was used is spent.
    assert_eq!(
        day::status(ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-10-09")
    );
    let closed = day::fix_close(
        ctx,
        &w.owner,
        &DayFixCloseInput {
            token: fix.token.clone(),
        },
    )
    .unwrap();
    pair(
        "day_fix_close_e2e_45",
        &serde_json::json!({ "token": fix.token }),
        &closed,
    );
    assert_eq!(closed.closed, ["2026-10-10"]);
    assert!(closed.stayed_open.is_empty());
    assert_eq!(closed.closed_up_to.as_deref(), Some("2026-10-10"));
    assert_eq!(
        day::status(ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-10-10")
    );
    let spent = day::fix_close(ctx, &w.owner, &DayFixCloseInput { token: fix.token }).unwrap_err();
    assert_eq!(spent.code(), ErrorCode::ErrFixTokenInvalid);
    // The edit left its stamp: the time, who, how many times and why. The day is closed again, so
    // the row asks for a reopen again. The history has the changed fields (plan 44 T2b).
    let stamped = bikri::get(ctx, 1).unwrap().flags;
    let stamp = stamped.edited.as_ref().expect("the edit left a stamp");
    assert_eq!(stamp.count, 1);
    assert_eq!(stamp.reason.as_deref(), Some("اکرم کا ریٹ غلط تھا"));
    assert_eq!(stamp.local_date, "2026-10-11");
    assert!(!stamp.by.is_empty() && stamp.local_time.len() == 5);
    assert!(stamped.needs_reopen && stamped.can_edit);
    let listed = bikri::list(ctx, &Default::default()).unwrap();
    assert_eq!(
        listed
            .rows
            .iter()
            .filter(|r| r.flags.edited.is_some())
            .count(),
        1,
        "only the edited بکری is stamped"
    );
    let stamp_request = serde_json::json!({ "entity": "bikri", "ids": [1, 2] });
    let stamps = mandi_reports::audit::edit_stamps(
        ctx.db(),
        &mandi_reports::audit::EditStampInput {
            entity: mandi_services::core::edit::EditEntity::Bikri,
            ids: vec![1, 2],
        },
    )
    .unwrap();
    pair("audit_edit_stamp_bikri", &stamp_request, &stamps);
    assert_eq!(stamps[0].stamp.as_ref(), Some(stamp));
    assert!(stamps[1].stamp.is_none());
    let history_request = serde_json::json!({ "entity": "bikri", "id": 1 });
    let history = mandi_reports::audit::edit_history(
        ctx.db(),
        &mandi_reports::audit::EditHistoryInput {
            entity: mandi_services::core::edit::EditEntity::Bikri,
            id: 1,
        },
    )
    .unwrap();
    pair("audit_edit_history_bikri", &history_request, &history);
    assert_eq!(history.versions.len(), 1);
    let paths: Vec<&str> = history.versions[0]
        .changes
        .iter()
        .map(|c| c.path.as_str())
        .collect();
    assert!(
        paths.contains(&"lots[0].sales[0].seller_rate_paisa")
            && paths.contains(&"lots[0].sales[0].buyer_total_paisa"),
        "{paths:?}"
    );
    // The trial balance still balances, and the audit log has the whole story in order.
    // The books balance (the helper asserts it); E2E-13 adds 188 to both sides.
    assert_eq!(trial(&w), 85_048 + 212);
    let actions = audit_actions(&w);
    assert_eq!(
        actions[actions.len() - 6..],
        [
            "day_close",
            "day_reopen",
            "day_fix_open",
            "edit",
            "day_close",
            "day_fix_close"
        ]
    );
}

#[test]
fn a_fix_that_is_widened_or_overtaken_by_another_close_is_handled() {
    let w = events_1_to_5();
    let ctx = &w.env.ctx;
    day::close(
        ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 12);
    day::close(
        ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "11-10-2026".into(),
        },
    )
    .unwrap();
    // Reopen 11-10: that day and every later closed day open. Then 10-10: the fix is widened and
    // the token covers 10-10 and 11-10.
    let first = day::fix_open(
        ctx,
        &w.owner,
        &DayFixOpenInput {
            date: "11-10-2026".into(),
            reason: "x".into(),
            sign: "adnan".into(),
        },
    )
    .unwrap();
    assert_eq!(first.days, ["2026-10-11"]);
    let second = day::fix_open(
        ctx,
        &w.owner,
        &DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: "y".into(),
            sign: "adnan".into(),
        },
    )
    .unwrap();
    assert_eq!(second.days, ["2026-10-10", "2026-10-11"]);
    // The first token is spent by the second reopen.
    assert_eq!(
        day::fix_close(ctx, &w.owner, &DayFixCloseInput { token: first.token })
            .unwrap_err()
            .code(),
        ErrorCode::ErrFixTokenInvalid
    );
    // If someone closes the days some other way first, the token no longer fits.
    day::close(
        ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(
        day::fix_close(
            ctx,
            &w.owner,
            &DayFixCloseInput {
                token: second.token
            }
        )
        .unwrap_err()
        .code(),
        ErrorCode::ErrFixTokenInvalid
    );
}
