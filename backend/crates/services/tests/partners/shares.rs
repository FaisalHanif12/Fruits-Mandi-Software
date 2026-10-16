//! Setting the shares (PRD FR-17.1, FR-17.2, E9, E18, A31): exactly 100%, every active partner
//! named once, no back-dating, history kept, one audit row, nothing written when refused.

use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::partners::{PartnerUpdateInput, PartnersCheckSharesInput, ShareInput};
use mandi_services::{ServiceError, partners};

use crate::common::{assert_chain_ok, audit_count, set_today};
use crate::support::*;

fn two_partners(
    env: &mandi_services::testing::TestEnv,
    owner: &mandi_services::auth::Actor,
) -> (i64, i64) {
    let a = new_partner(env, owner, "الف", "Partner A", None)
        .unwrap()
        .id;
    let b = new_partner(env, owner, "ب", "Partner B", None).unwrap().id;
    (a, b)
}

#[test]
fn shares_must_total_exactly_100_percent() {
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let history_before = count(&dir, "SELECT COUNT(*) FROM partner_shares");
    // 9,000 bps and 10,100 bps are both refused, on the field `shares`, with the total.
    let nine = set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "40"), (b, "30"), (c, "20")],
    )
    .unwrap_err();
    assert_eq!(
        first(&nine, "total_bps"),
        (
            ErrorCode::ErrSharesNot100,
            Some("shares".to_owned()),
            Some(9_000)
        )
    );
    let over = set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "40"), (b, "30"), (c, "31")],
    )
    .unwrap_err();
    assert_eq!(first(&over, "total_bps").2, Some(10_100));
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM partner_shares"),
        history_before,
        "nothing was written"
    );
}

#[test]
fn two_decimals_are_read_exactly() {
    let (_dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let rows = set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "33.33"), (b, "33.33"), (c, "33.34")],
    )
    .unwrap();
    assert_eq!(
        rows.iter().map(|r| r.share_bps).collect::<Vec<_>>(),
        vec![3_333, 3_333, 3_334]
    );
}

#[test]
fn every_active_partner_must_be_named_once() {
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let before = count(&dir, "SELECT COUNT(*) FROM partner_shares");
    // ج is missing (the total is 100%, so this is the only problem).
    let missing = set_shares(&env, &owner, "10-10-2026", &[(a, "50"), (b, "50")]).unwrap_err();
    assert_eq!(
        all(&missing),
        vec![(ErrorCode::ErrPartnerNotListed, "shares".to_owned())]
    );
    // ب is named twice.
    let twice = set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "40"), (b, "30"), (b, "30")],
    )
    .unwrap_err();
    assert!(twice.has_code(ErrorCode::ErrPartnerNotListed));
    // Somebody who is not a partner takes ج's place.
    let stranger = set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "40"), (b, "30"), (999, "30")],
    )
    .unwrap_err();
    assert!(stranger.has_code(ErrorCode::ErrPartnerNotListed));
    let _ = c;
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partner_shares"), before);
}

#[test]
fn a_bad_percent_names_its_row_and_the_good_rows_still_count() {
    let (_dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let error = set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "40"), (b, "abc"), (c, "30.555")],
    )
    .unwrap_err();
    let problems = all(&error);
    assert!(problems.contains(&(ErrorCode::ErrNotANumber, "shares[1].share".to_owned())));
    assert!(problems.contains(&(ErrorCode::ErrTooManyDecimals, "shares[2].share".to_owned())));
    let over = set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "101"), (b, "0"), (c, "0")],
    )
    .unwrap_err();
    assert!(over.has_code(ErrorCode::ErrPercentOver100));
}

#[test]
fn a_change_dated_before_today_is_refused_e18_a31() {
    let (dir, env, owner) = ready();
    let (a, b) = two_partners(&env, &owner);
    // The very first set is checked too.
    let error = set_shares(&env, &owner, "09-10-2026", &[(a, "60"), (b, "40")]).unwrap_err();
    let (code, field, _) = first(&error, "today");
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrShareBackdated, Some("effective_from"))
    );
    match &error {
        ServiceError::Rejected(items) => assert_eq!(
            items[0].values.get("today"),
            Some(&ErrorValue::Text("10-10-2026".to_owned()))
        ),
        other => panic!("{other:?}"),
    }
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partner_shares"), 0);
    // Today and later are fine.
    set_shares(&env, &owner, "10-10-2026", &[(a, "60"), (b, "40")]).unwrap();
    set_shares(&env, &owner, "11-10-2026", &[(a, "50"), (b, "50")]).unwrap();
    // A date that does not exist is an invalid date, not a back-dated one.
    let error = set_shares(&env, &owner, "31-02-2026", &[(a, "50"), (b, "50")]).unwrap_err();
    assert_eq!(first(&error, "x").0, ErrorCode::ErrDateInvalid);
}

#[test]
fn all_the_problems_of_one_try_come_back_together() {
    let (_dir, env, owner) = ready();
    let (a, b) = two_partners(&env, &owner);
    let error = set_shares(&env, &owner, "01-10-2026", &[(a, "40"), (b, "x")]).unwrap_err();
    let problems = all(&error);
    assert!(problems.contains(&(ErrorCode::ErrShareBackdated, "effective_from".to_owned())));
    assert!(problems.contains(&(ErrorCode::ErrNotANumber, "shares[1].share".to_owned())));
}

#[test]
fn history_is_kept_and_the_last_row_of_a_day_wins() {
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    set_shares(
        &env,
        &owner,
        "20-10-2026",
        &[(a, "50"), (b, "30"), (c, "20")],
    )
    .unwrap();
    // The same day again: the row saved last is the share of that day.
    set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "45"), (b, "30"), (c, "25")],
    )
    .unwrap();
    let detail = partners::get(&env.ctx, a).unwrap();
    assert_eq!(detail.row.share_bps, 4_500);
    assert_eq!(
        detail
            .shares
            .iter()
            .map(|s| (s.share_bps, s.effective_from.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (4_000, "2026-10-10"),
            (4_500, "2026-10-10"),
            (5_000, "2026-10-20")
        ]
    );
    // One audit row for each set, with the old and the new shares (S7).
    assert_eq!(audit_count(&dir, "set_shares"), 3);
    let (old, new): (String, String) = crate::common::raw(&dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE action = 'set_shares' ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(old.contains("4000"), "{old}");
    assert!(new.contains("4500") && new.contains("2026-10-10"), "{new}");
    assert_chain_ok(&dir);
}

#[test]
fn the_check_gives_the_same_total_and_errors_as_the_save() {
    let (_dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let input = |rows: &[(i64, &str)]| PartnersCheckSharesInput {
        shares: rows
            .iter()
            .map(|&(partner_id, share)| ShareInput {
                partner_id,
                share: share.to_owned(),
            })
            .collect(),
    };
    let good = partners::check(&env.ctx, &input(&[(a, "40"), (b, "30"), (c, "30")])).unwrap();
    assert_eq!(
        (good.total_bps, good.is_100, good.errors.len()),
        (10_000, true, 0)
    );
    let nine = partners::check(&env.ctx, &input(&[(a, "40"), (b, "30"), (c, "20")])).unwrap();
    assert_eq!((nine.total_bps, nine.is_100), (9_000, false));
    assert_eq!(nine.errors[0].code, ErrorCode::ErrSharesNot100);
    assert_eq!(nine.errors[0].field, "shares");
    // A partner left out shows the same problem the save would.
    let missing = partners::check(&env.ctx, &input(&[(a, "50"), (b, "50")])).unwrap();
    assert!(!missing.is_100);
    assert!(
        missing
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrPartnerNotListed)
    );
    // The check writes nothing.
    let bad = partners::check(&env.ctx, &input(&[(a, "x")])).unwrap();
    assert_eq!(bad.errors[0].field, "shares[0].share");
}

#[test]
fn an_inactive_partner_keeps_his_history_and_is_left_out_of_later_sets() {
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    set_shares(
        &env,
        &owner,
        "20-10-2026",
        &[(a, "50"), (b, "50"), (c, "0")],
    )
    .unwrap();
    set_today(&env, 2026, 10, 20);
    let row = partners::get(&env.ctx, c).unwrap().row;
    partners::update(
        &env.ctx,
        &owner,
        &PartnerUpdateInput {
            id: c,
            version: row.version,
            name_ur: row.name_ur.clone(),
            name_en: row.name_en.clone(),
            is_active: false,
            confirmed_rules_words: Vec::new(),
        },
    )
    .unwrap();
    // The next set names only the two active partners; naming ج is refused.
    let error = set_shares(
        &env,
        &owner,
        "20-10-2026",
        &[(a, "50"), (b, "30"), (c, "20")],
    )
    .unwrap_err();
    assert!(error.has_code(ErrorCode::ErrPartnerNotListed));
    set_shares(&env, &owner, "20-10-2026", &[(a, "60"), (b, "40")]).unwrap();
    // A posting dated 10-10 (still open) splits with ج at his 30% of that day, because the
    // shares of a date are the shares of that date; one dated 20-10 has no row for ج.
    let income = id_of(&env, "04010001");
    let old_day = against_cash(&env, &owner, "2026-10-10", income, Cr, 1_000).unwrap();
    assert_eq!(amounts(&rows_of(&dir, old_day)), vec![400, 300, 300]);
    let new_day = against_cash(&env, &owner, "2026-10-20", income, Cr, 1_000).unwrap();
    let rows = rows_of(&dir, new_day);
    assert_eq!(amounts(&rows), vec![600, 400]);
    assert!(rows.iter().all(|r| r.partner_id != c));
}
