//! Plan 44 T2e: the partner code (PRD FR-17.11, NFR-SEC-14, E80, E2E-46).
//!
//! E2E-46: after Events 1 to 5 the code is set to 4826. The partner section is locked until 4826 is
//! given; 1111 is refused and logged; with 4826 the overview shows the profit shares 996 / 746 / 746;
//! leaving the section, the idle lock and a logout lock it again.
//!
//! The commands call [`partner_code::require_unlocked`] after the sign-in check; the app test
//! `partner_code_ipc` holds the commands to it. Here are the service, the waits, the audit rows, the
//! hash, and the figures that are left out while locked.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::analytics::{AnalyticsFilters, AnalyticsView};
use mandi_reports::dashboard::{self, AttentionKind, BackupFacts};
use mandi_reports::partner_guard::{PartnerNeed, analytics_need, report_need};
use mandi_reports::reports::{ReportFilters, ReportId};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::money::Paisa;
use mandi_services::core::partner_code::{
    PartnerCodeChangeInput, PartnerCodeResetInput, PartnerCodeSetInput, PartnerCodeStatus,
    PartnerCodeUnlockInput,
};
use mandi_services::{Rejection, Secret, ServiceError, auth, partner_code, partners};
use reports_world::full_day;

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

/// The refusal of a service call.
fn refusal(error: ServiceError) -> Rejection {
    match error {
        ServiceError::Rejected(mut items) if items.len() == 1 => items.remove(0),
        other => panic!("not a single refusal: {other:?}"),
    }
}

fn number(rejection: &Rejection, key: &str) -> Option<i64> {
    match rejection.values.get(key) {
        Some(ErrorValue::Number(n)) => Some(*n),
        _ => None,
    }
}

fn set_input(code: &str, again: &str) -> PartnerCodeSetInput {
    PartnerCodeSetInput {
        password: OWNER_PASSWORD.to_owned(),
        code: code.to_owned(),
        code_again: again.to_owned(),
    }
}

fn unlock_input(code: &str) -> PartnerCodeUnlockInput {
    PartnerCodeUnlockInput {
        code: code.to_owned(),
    }
}

fn status(unlocked: bool, set: bool, wait: i64) -> PartnerCodeStatus {
    PartnerCodeStatus {
        is_set: set,
        unlocked,
        wait_seconds: wait,
    }
}

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

/// A fresh signed-in environment with no data.
fn ready() -> (
    tempfile::TempDir,
    mandi_services::testing::TestEnv,
    auth::Actor,
) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn failed_tries(dir: &tempfile::TempDir) -> i64 {
    raw(dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'partner_code' AND action = 'partner_code_failed'",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn e2e_46_the_partner_section_opens_only_with_the_code() {
    let w = full_day();
    let ctx = &w.env.ctx;

    // No code yet: the section is open, so the first-run partner setup works (plan 44 Section 9).
    assert_eq!(partner_code::status(ctx).unwrap(), status(false, false, 0));
    assert!(!partner_code::is_locked(ctx).unwrap());
    partner_code::require_unlocked(ctx).unwrap();

    // The code is set with the Owner's password; the section is open after it.
    let set = partner_code::set(ctx, &w.owner, &set_input("4826", "4826")).unwrap();
    assert_eq!(set, status(true, true, 0));

    // Leaving the section locks it. Every partner command refuses (`require_unlocked` is the check
    // they all start with); the reports a partner would read need the code.
    assert_eq!(partner_code::lock(ctx).unwrap(), status(false, true, 0));
    let locked = refusal(partner_code::require_unlocked(ctx).unwrap_err());
    assert_eq!(locked.code, ErrorCode::ErrPartnerLocked);
    assert!(partner_code::is_locked(ctx).unwrap());
    let plain = ReportFilters::default();
    assert_eq!(report_need(ReportId::R31, &plain), PartnerNeed::Refuse);
    assert_eq!(report_need(ReportId::R30, &plain), PartnerNeed::Refuse);
    assert_eq!(
        analytics_need(AnalyticsView::F, &AnalyticsFilters::default()),
        PartnerNeed::Refuse
    );
    // The names in a picker stay, without the shares.
    let hidden = partners::list_view(ctx).unwrap();
    assert!(hidden.partners_locked);
    assert_eq!(
        hidden
            .partners
            .iter()
            .map(|p| p.share_bps)
            .collect::<Vec<_>>(),
        [0, 0, 0]
    );
    assert_eq!(hidden.partners.len(), 3);

    // 1111 is refused, says how many tries are left, and is logged.
    let wrong = refusal(partner_code::unlock(ctx, &w.owner, &unlock_input("1111")).unwrap_err());
    assert_eq!(wrong.code, ErrorCode::ErrPartnerCodeWrong);
    assert_eq!(wrong.field.as_deref(), Some("code"));
    assert_eq!(number(&wrong, "tries_left"), Some(9));
    assert!(partner_code::is_locked(ctx).unwrap());
    assert_eq!(failed_tries(&w.dir), 1);

    // 4826 opens it, and the overview shows the profit shares of the worked day.
    let opened = partner_code::unlock(ctx, &w.owner, &unlock_input("4826")).unwrap();
    assert_eq!(opened, status(true, true, 0));
    partner_code::require_unlocked(ctx).unwrap();
    let overview = partners::overview(ctx).unwrap();
    assert_eq!(
        overview
            .iter()
            .map(|p| p.month_profit_paisa)
            .collect::<Vec<_>>(),
        [rs(996), rs(746), rs(746)]
    );
    let shown = partners::list_view(ctx).unwrap();
    assert!(!shown.partners_locked);
    assert_eq!(
        shown
            .partners
            .iter()
            .map(|p| p.share_bps)
            .collect::<Vec<_>>(),
        [4_000, 3_000, 3_000]
    );
    // A right code ends the run of wrong tries.
    let wrong_again =
        refusal(partner_code::unlock(ctx, &w.owner, &unlock_input("2222")).unwrap_err());
    assert_eq!(number(&wrong_again, "tries_left"), Some(9));

    // Leaving the section locks it again.
    partner_code::lock(ctx).unwrap();
    assert!(partner_code::is_locked(ctx).unwrap());

    // The idle lock locks it, and unlocking the session does not open it.
    partner_code::unlock(ctx, &w.owner, &unlock_input("4826")).unwrap();
    auth::lock(ctx).unwrap();
    assert!(partner_code::is_locked(ctx).unwrap());
    auth::unlock(ctx, &Secret::new(OWNER_PASSWORD)).unwrap();
    assert!(partner_code::is_locked(ctx).unwrap());

    // A logout and a new sign-in lock it.
    partner_code::unlock(ctx, &w.owner, &unlock_input("4826")).unwrap();
    auth::logout(ctx).unwrap();
    login_owner(&w.env);
    assert!(partner_code::is_locked(ctx).unwrap());
}

#[test]
fn what_is_behind_the_code_is_left_out_while_locked() {
    let w = full_day();
    let ctx = &w.env.ctx;
    partner_code::set(ctx, &w.owner, &set_input("4826", "4826")).unwrap();

    // The dashboard strip and the amount of money partners took past its day.
    let mut dash = dashboard::get(ctx.db(), off()).unwrap();
    assert_eq!(dash.partners.len(), 3);
    assert!(!dash.partners_locked);
    dash.hide_partners();
    assert!(dash.partners.is_empty() && dash.partners_locked && dash.attention.partners_locked);
    for item in &dash.attention.items {
        if item.kind == AttentionKind::TakesOverdue {
            assert_eq!(item.amount_paisa, None);
        }
    }

    // The day close summary: no partner lines.
    let mut preview = mandi_services::day::close_preview(
        ctx,
        &mandi_services::core::vouchers::DayClosePreviewInput {
            up_to: String::new(),
        },
    )
    .unwrap();
    assert_eq!(preview.summary.partners.len(), 3);
    preview.summary.hide_partners();
    assert!(preview.summary.partners.is_empty() && preview.summary.partners_locked);

    // The audit log keeps who and when of a partner event, and leaves out the values.
    let mut page = mandi_reports::audit::list(
        ctx.db(),
        &mandi_reports::audit::AuditListInput {
            filters: mandi_reports::audit::AuditFilters {
                entity: "partner".to_owned(),
                ..Default::default()
            },
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap();
    assert!(!page.rows.is_empty());
    assert!(page.rows.iter().any(|r| r.new_json.is_some()));
    page.hide_partners();
    assert!(page.partners_locked);
    for row in &page.rows {
        assert!(row.old_json.is_none() && row.new_json.is_none() && row.changes.is_empty());
    }
}

#[test]
fn a_wrong_code_waits_as_the_sign_in_does_and_the_wait_is_remembered() {
    let (dir, env, owner) = ready();
    let ctx = &env.ctx;
    partner_code::set(ctx, &owner, &set_input("4826", "4826")).unwrap();
    partner_code::lock(ctx).unwrap();

    // Tries 1 to 9 count down; the 10th starts a wait of 1 minute (10 in a step, Umar 12-11-2026).
    for left in (1..10).rev() {
        let e = refusal(partner_code::unlock(ctx, &owner, &unlock_input("1111")).unwrap_err());
        assert_eq!(number(&e, "tries_left"), Some(left));
    }
    let wait = refusal(partner_code::unlock(ctx, &owner, &unlock_input("1111")).unwrap_err());
    assert_eq!(wait.code, ErrorCode::ErrLockedWait);
    assert_eq!(number(&wait, "seconds"), Some(60));
    assert_eq!(partner_code::status(ctx).unwrap(), status(false, true, 60));
    // During the wait even the right code is refused, and the status says how long.
    env.mono.advance_seconds(20);
    let during = refusal(partner_code::unlock(ctx, &owner, &unlock_input("4826")).unwrap_err());
    assert_eq!(during.code, ErrorCode::ErrLockedWait);
    assert_eq!(number(&during, "seconds"), Some(40));
    assert!(partner_code::is_locked(ctx).unwrap());

    // After the wait the count goes on from 10: nine more tries, then 5 minutes.
    env.mono.advance_seconds(41);
    for left in (1..10).rev() {
        let e = refusal(partner_code::unlock(ctx, &owner, &unlock_input("1111")).unwrap_err());
        assert_eq!(number(&e, "tries_left"), Some(left));
    }
    let second = refusal(partner_code::unlock(ctx, &owner, &unlock_input("1111")).unwrap_err());
    assert_eq!(number(&second, "seconds"), Some(300));

    // The wait is also on disk: a new Context on the same file, as after a restart, still waits.
    let again = mandi_services::Context::new(ctx.db().clone());
    let stored = partner_code::status(&again).unwrap();
    assert!(
        stored.wait_seconds > 0 && stored.wait_seconds <= 300,
        "{stored:?}"
    );

    // After the wait a right code opens it, and the count starts again.
    env.mono.advance_seconds(301);
    partner_code::unlock(ctx, &owner, &unlock_input("4826")).unwrap();
    assert!(!partner_code::is_locked(ctx).unwrap());
    partner_code::lock(ctx).unwrap();
    let fresh = refusal(partner_code::unlock(ctx, &owner, &unlock_input("1111")).unwrap_err());
    assert_eq!(number(&fresh, "tries_left"), Some(9));
    // Every wrong try and every refused try in the wait is in the audit log: 20 + 1 + 1.
    assert_eq!(failed_tries(&dir), 20 + 1 + 1);
}

#[test]
fn the_code_is_stored_only_as_a_hash_and_never_logged() {
    let (dir, env, owner) = ready();
    let ctx = &env.ctx;
    partner_code::set(ctx, &owner, &set_input("4826", "4826")).unwrap();
    let _ = partner_code::unlock(ctx, &owner, &unlock_input("1111"));
    partner_code::change(
        ctx,
        &owner,
        &PartnerCodeChangeInput {
            old_code_or_password: "4826".to_owned(),
            code: "7391".to_owned(),
            code_again: "7391".to_owned(),
        },
    )
    .unwrap();

    let stored: String = raw(&dir)
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'partner_code'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(stored.contains("$argon2id$"), "{stored}");
    for secret in ["4826", "7391", "1111"] {
        assert!(!stored.contains(secret), "{secret} is in the setting");
        let in_audit: i64 = raw(&dir)
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE new_json LIKE ?1 OR old_json LIKE ?1 OR reason LIKE ?1",
                [format!("%{secret}%")],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(in_audit, 0, "{secret} is in the audit log");
    }
    // The set, the change and the wrong try are in the audit log.
    for action in [
        "partner_code_set",
        "partner_code_change",
        "partner_code_failed",
    ] {
        assert!(audit_count(&dir, action) >= 1, "{action}");
    }
}

#[test]
fn setting_changing_and_resetting_the_code_need_a_proof() {
    let (dir, env, owner) = ready();
    let ctx = &env.ctx;
    // Setting: the shape, the pair, the password.
    let bad_shape = refusal(partner_code::set(ctx, &owner, &set_input("482", "482")).unwrap_err());
    assert_eq!(
        (bad_shape.code, bad_shape.field.as_deref()),
        (ErrorCode::ErrPartnerCodeInvalid, Some("code"))
    );
    let letters = refusal(partner_code::set(ctx, &owner, &set_input("48a6", "48a6")).unwrap_err());
    assert_eq!(letters.code, ErrorCode::ErrPartnerCodeInvalid);
    let differ = refusal(partner_code::set(ctx, &owner, &set_input("4826", "4827")).unwrap_err());
    assert_eq!(
        (differ.code, differ.field.as_deref()),
        (ErrorCode::ErrPartnerCodesDiffer, Some("code_again"))
    );
    let mut wrong_password = set_input("4826", "4826");
    wrong_password.password = "not-it-1".to_owned();
    let wrong = refusal(partner_code::set(ctx, &owner, &wrong_password).unwrap_err());
    assert_eq!(
        (wrong.code, wrong.field.as_deref()),
        (ErrorCode::ErrWrongPassword, Some("password"))
    );
    assert!(number(&wrong, "tries_left").is_some());
    assert!(!partner_code::status(ctx).unwrap().is_set);
    // Changing or resetting before any code is set is refused.
    let change = |old: &str, new: &str| {
        partner_code::change(
            ctx,
            &owner,
            &PartnerCodeChangeInput {
                old_code_or_password: old.to_owned(),
                code: new.to_owned(),
                code_again: new.to_owned(),
            },
        )
    };
    assert_eq!(
        refusal(change("4826", "7391").unwrap_err()).code,
        ErrorCode::ErrPartnerCodeNotSet
    );

    partner_code::set(ctx, &owner, &set_input("4826", "4826")).unwrap();
    // A second set is refused.
    assert_eq!(
        refusal(partner_code::set(ctx, &owner, &set_input("1234", "1234")).unwrap_err()).code,
        ErrorCode::ErrPartnerCodeAlreadySet
    );

    // Changing: the old code, or the password; a wrong old code counts.
    partner_code::lock(ctx).unwrap();
    let wrong_old = refusal(change("1111", "7391").unwrap_err());
    assert_eq!(
        (wrong_old.code, wrong_old.field.as_deref()),
        (ErrorCode::ErrPartnerCodeWrong, Some("old_code_or_password"))
    );
    assert_eq!(number(&wrong_old, "tries_left"), Some(9));
    assert_eq!(change("4826", "7391").unwrap(), status(true, true, 0));
    // The old code no longer opens it; the new one does.
    partner_code::lock(ctx).unwrap();
    assert_eq!(
        refusal(partner_code::unlock(ctx, &owner, &unlock_input("4826")).unwrap_err()).code,
        ErrorCode::ErrPartnerCodeWrong
    );
    partner_code::unlock(ctx, &owner, &unlock_input("7391")).unwrap();
    // The password also proves the Owner (a password has at least 6 characters).
    assert_eq!(
        change(OWNER_PASSWORD, "2468").unwrap(),
        status(true, true, 0)
    );
    partner_code::lock(ctx).unwrap();
    partner_code::unlock(ctx, &owner, &unlock_input("2468")).unwrap();
    // A new code that is not 4 digits, or typed differently twice, is refused.
    assert_eq!(
        refusal(change(OWNER_PASSWORD, "24680").unwrap_err()).code,
        ErrorCode::ErrPartnerCodeInvalid
    );
    let reset = |proof: &str, code: &str, again: &str| {
        partner_code::reset(
            ctx,
            &owner,
            &PartnerCodeResetInput {
                password_or_recovery: proof.to_owned(),
                code: code.to_owned(),
                code_again: again.to_owned(),
            },
        )
    };
    assert_eq!(
        refusal(reset(OWNER_PASSWORD, "1357", "1358").unwrap_err()).code,
        ErrorCode::ErrPartnerCodesDiffer
    );

    // Reset ("code forgotten"): the Owner's password; a wrong proof counts as a wrong password.
    partner_code::lock(ctx).unwrap();
    let bad = refusal(reset("not-it-1", "1357", "1357").unwrap_err());
    assert_eq!(
        (bad.code, bad.field.as_deref()),
        (ErrorCode::ErrWrongPassword, Some("password_or_recovery"))
    );
    assert_eq!(
        reset(OWNER_PASSWORD, "1357", "1357").unwrap(),
        status(true, true, 0)
    );
    partner_code::lock(ctx).unwrap();
    partner_code::unlock(ctx, &owner, &unlock_input("1357")).unwrap();
    assert_eq!(audit_count(&dir, "partner_code_reset"), 1);
}

#[test]
fn a_reset_with_the_recovery_code_does_not_spend_it_and_ends_the_wait() {
    let (dir, env) = common::env();
    let recovery = set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let ctx = &env.ctx;
    partner_code::set(ctx, &owner, &set_input("4826", "4826")).unwrap();
    partner_code::lock(ctx).unwrap();
    for _ in 0..10 {
        let _ = partner_code::unlock(ctx, &owner, &unlock_input("1111"));
    }
    assert_eq!(partner_code::status(ctx).unwrap().wait_seconds, 60);

    let reset = |proof: &str| {
        partner_code::reset(
            ctx,
            &owner,
            &PartnerCodeResetInput {
                password_or_recovery: proof.to_owned(),
                code: "9753".to_owned(),
                code_again: "9753".to_owned(),
            },
        )
    };
    assert_eq!(reset(&recovery).unwrap(), status(true, true, 0));
    // The wait is over and the new code works.
    partner_code::lock(ctx).unwrap();
    partner_code::unlock(ctx, &owner, &unlock_input("9753")).unwrap();
    // The recovery code is not spent: it proves the Owner a second time.
    assert!(reset(&recovery).is_ok());
    assert_eq!(audit_count(&dir, "partner_code_reset"), 2);
}

#[test]
fn the_mixed_reports_leave_out_their_partner_parts_while_locked() {
    use mandi_reports::reports::{Cell, ReportPage, RowKind};
    let w = full_day();
    let day = || ReportFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        ..ReportFilters::default()
    };

    // R29 for the worked day: the shop expense (3,000) and the money الف took (2,000).
    let open = reports_world::report(&w, ReportId::R29, day(), 0, 100);
    let hidden = reports_world::report(
        &w,
        ReportId::R29,
        ReportFilters {
            hide_partners: true,
            ..day()
        },
        0,
        100,
    );
    let headings = |page: &ReportPage| {
        page.rows
            .iter()
            .filter(|r| r.kind == RowKind::Group)
            .count()
    };
    assert_eq!(headings(&open), 2);
    assert_eq!(headings(&hidden), 1);
    assert!(hidden.rows.len() < open.rows.len());
    // The shop expense total stays; the taken and returned totals are empty, not 0.
    assert_eq!(hidden.totals[2], open.totals[2]);
    assert_eq!(hidden.totals[3], Cell::Empty);
    assert_eq!(hidden.totals[4], Cell::Empty);
    assert_ne!(open.totals[3], Cell::Empty);

    // R34: the partner shares, the money taken and returned and the capital rows are left out.
    let open = reports_world::report(&w, ReportId::R34, day(), 0, 200);
    let hidden = reports_world::report(
        &w,
        ReportId::R34,
        ReportFilters {
            hide_partners: true,
            ..day()
        },
        0,
        200,
    );
    let count = |page: &ReportPage, id: &str| {
        page.rows
            .iter()
            .filter(|r| matches!(r.cells.get(1), Some(Cell::Label { id: label }) if label == id))
            .count()
    };
    for id in [
        "report-r34-partner-shares",
        "report-r34-partner-taken",
        "report-r34-partner-returned",
        "report-r34-capital",
    ] {
        assert_eq!(count(&open, id), 1, "{id}");
        assert_eq!(count(&hidden, id), 0, "{id}");
    }
    // Four rows fewer, and the three partners' names under the shares heading.
    assert_eq!(hidden.rows.len() + 4 + 3, open.rows.len());
}
