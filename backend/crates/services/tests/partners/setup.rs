//! Adding, editing, listing and reading partners (PRD 17.1, FR-2.4, FR-7.24, FR-7.66, FR-10.9,
//! NFR-SCL-5, S7).

use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::partners::{PartnerCreateInput, PartnerUpdateInput};
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::{day, ledger, partners};

use crate::common::{assert_chain_ok, audit_total};
use crate::support::*;

fn account_names(env: &mandi_services::testing::TestEnv, id: i64) -> (String, String, String) {
    env.ctx
        .db()
        .read(|c| {
            Ok::<_, mandi_services::ServiceError>(
                c.query_row(
                    "SELECT code, name_ur, name_en FROM accounts WHERE id = ?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .unwrap(),
            )
        })
        .unwrap()
}

#[test]
fn a_new_partner_gets_a_capital_and_a_drawings_account_in_both_scripts() {
    let (dir, env, owner) = ready();
    let a = new_partner(&env, &owner, "الف", "Partner A", None).unwrap();
    assert_eq!(a.capital_account_code, "06020001");
    assert_eq!(a.drawings_account_code, "06030001");
    assert_eq!(a.share_bps, 0, "no share until the shares are set");
    assert!(a.is_active);
    assert_eq!(
        account_names(&env, a.capital_account_id),
        (
            "06020001".to_owned(),
            "سرمایہ - الف".to_owned(),
            "Capital - Partner A".to_owned()
        )
    );
    assert_eq!(
        account_names(&env, a.drawings_account_id),
        (
            "06030001".to_owned(),
            "ذاتی خرچہ - الف".to_owned(),
            "Drawings - Partner A".to_owned()
        )
    );
    // The second partner gets the next serials. His accounts start empty.
    let b = new_partner(&env, &owner, "ب", "Partner B", None).unwrap();
    assert!(b.id > a.id, "created later, so a higher id (C10 tie rule)");
    assert_eq!(b.capital_account_code, "06020002");
    assert_eq!(b.drawings_account_code, "06030002");
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        0,
        "no money moved"
    );
    // S7: one audit row for each partner, and the chain still holds.
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'partner' AND action = 'create'"
        ),
        2
    );
    assert_chain_ok(&dir);
}

#[test]
fn no_user_is_made_for_a_partner() {
    // FR-2.4: partners do not log in.
    let (dir, env, owner) = ready();
    three_partners(&env, &owner);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM users"), 1);
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM partners WHERE user_id IS NOT NULL"
        ),
        0
    );
}

#[test]
fn an_opening_capital_is_posted_as_the_opening_credit_of_his_capital_account() {
    // FR-10.9: his capital account جمع, ابتدائی فرق بنام, dated today.
    let (dir, env, owner) = ready();
    let b = new_partner(&env, &owner, "ب", "Partner B", Some("50,000")).unwrap();
    let balance = ledger::account_balance(&env.ctx, b.capital_account_id).unwrap();
    assert_eq!(
        (balance.side, balance.amount_paisa),
        (BalanceSide::Cr, rupees(50_000))
    );
    let difference = ledger::account_balance(&env.ctx, id_of(&env, "06019999")).unwrap();
    assert_eq!(
        (difference.side, difference.amount_paisa),
        (BalanceSide::Dr, rupees(50_000))
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_entries WHERE posting_type = 'opening'
             AND entry_date = '2026-10-10'"
        ),
        1
    );
    let overview = partners::overview(&env.ctx).unwrap();
    assert_eq!(overview.len(), 1);
    assert_eq!(overview[0].capital_balance_paisa, rupees(50_000));
    assert_chain_ok(&dir);
}

#[test]
fn an_empty_opening_capital_means_none() {
    let (dir, env, owner) = ready();
    new_partner(&env, &owner, "الف", "Partner A", Some("")).unwrap();
    new_partner(&env, &owner, "ب", "Partner B", Some("0")).unwrap();
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), 0);
}

#[test]
fn a_bad_opening_capital_saves_nothing() {
    let (dir, env, owner) = ready();
    let before = audit_total(&dir);
    let error = new_partner(&env, &owner, "الف", "Partner A", Some("50.5")).unwrap_err();
    assert_eq!(
        first(&error, "x"),
        (
            ErrorCode::ErrDecimalsNotAllowed,
            Some("opening_capital".to_owned()),
            None
        )
    );
    let error = new_partner(&env, &owner, "الف", "Partner A", Some("abc")).unwrap_err();
    assert_eq!(first(&error, "x").0, ErrorCode::ErrNotANumber);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partners"), 0);
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM accounts WHERE code LIKE '0602%' OR code LIKE '0603%'"
        ),
        // The seeded controls have no account of their own; only partner accounts match.
        0
    );
    assert_eq!(audit_total(&dir), before);
}

#[test]
fn a_closed_day_refuses_the_opening_capital_and_saves_nothing() {
    let (dir, env, owner) = ready();
    day::close(
        &env.ctx,
        &owner,
        &DayCloseInput {
            up_to: "10-10-2026".to_owned(),
        },
    )
    .unwrap();
    let error = new_partner(&env, &owner, "الف", "Partner A", Some("1,000")).unwrap_err();
    assert_eq!(
        first(&error, "x"),
        (
            ErrorCode::ErrDateClosed,
            Some("opening_capital".to_owned()),
            None
        )
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partners"), 0);
    // Without an opening capital the partner needs no posting date and is saved.
    new_partner(&env, &owner, "الف", "Partner A", None).unwrap();
}

#[test]
fn both_names_are_checked_in_their_own_script_and_the_errors_come_together() {
    let (_dir, env, owner) = ready();
    let error = partners::create(
        &env.ctx,
        &owner,
        &PartnerCreateInput {
            name_ur: "Akram".to_owned(),
            name_en: "اکرم".to_owned(),
            opening_capital: None,
            confirmed_rules_words: Vec::new(),
        },
    )
    .unwrap_err();
    assert_eq!(
        all(&error),
        vec![
            (ErrorCode::ErrNameUrScript, "name_ur".to_owned()),
            (ErrorCode::ErrNameEnScript, "name_en".to_owned()),
        ]
    );
}

#[test]
fn a_word_made_by_spelling_rules_alone_needs_the_owner_to_confirm_it() {
    // FR-7.66: a rules-only spelling is never saved without a confirmation. The first try
    // names the words; the second, with them confirmed, saves and teaches the pair.
    let (dir, env, owner) = ready();
    let error = partners::create(
        &env.ctx,
        &owner,
        &PartnerCreateInput {
            name_ur: "علی قزکس".to_owned(),
            name_en: "Ali Qzxwv".to_owned(),
            opening_capital: None,
            confirmed_rules_words: Vec::new(),
        },
    )
    .unwrap_err();
    assert!(
        all(&error)
            .iter()
            .all(|(code, _)| *code == ErrorCode::ErrSpellingUnconfirmed),
        "{error:?}"
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partners"), 0);
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM accounts WHERE code LIKE '0602%'"
        ),
        0
    );
    let learned_before = count(&dir, "SELECT COUNT(*) FROM translit_pairs");
    new_partner(&env, &owner, "علی قزکس", "Ali Qzxwv", None).unwrap();
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partners"), 1);
    assert!(count(&dir, "SELECT COUNT(*) FROM translit_pairs") > learned_before);
}

#[test]
fn the_eleventh_active_partner_is_refused_nfr_scl_5() {
    let (dir, env, owner) = ready();
    for number in 1..=10 {
        new_partner(
            &env,
            &owner,
            &format!("شریک {number}"),
            &format!("Partner {number}"),
            None,
        )
        .unwrap();
    }
    let error = new_partner(&env, &owner, "شریک 11", "Partner 11", None).unwrap_err();
    assert_eq!(
        first(&error, "max"),
        (ErrorCode::ErrTooManyPartners, None, Some(10))
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partners"), 10);
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM accounts WHERE code LIKE '0602%'"
        ),
        10,
        "the refused partner left no account behind"
    );
}

fn update_input(
    row: &mandi_services::core::partners::PartnerRow,
    name_ur: &str,
    name_en: &str,
    is_active: bool,
) -> PartnerUpdateInput {
    PartnerUpdateInput {
        id: row.id,
        version: row.version,
        name_ur: name_ur.to_owned(),
        name_en: name_en.to_owned(),
        is_active,
        confirmed_rules_words: Vec::new(),
    }
}

#[test]
fn new_names_rename_the_partner_and_his_two_accounts() {
    let (dir, env, owner) = ready();
    let a = new_partner(&env, &owner, "الف", "Partner A", None).unwrap();
    let saved =
        partners::update(&env.ctx, &owner, &update_input(&a, "احمد", "Ahmed", true)).unwrap();
    assert_eq!(saved.version, a.version + 1);
    assert_eq!(
        (saved.name_ur.as_str(), saved.name_en.as_str()),
        ("احمد", "Ahmed")
    );
    assert_eq!(account_names(&env, a.capital_account_id).1, "سرمایہ - احمد");
    assert_eq!(
        account_names(&env, a.drawings_account_id).2,
        "Drawings - Ahmed"
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'partner' AND action = 'edit'"
        ),
        1
    );
    assert_chain_ok(&dir);
}

#[test]
fn an_edit_with_an_old_version_or_an_unknown_id_is_refused() {
    let (_dir, env, owner) = ready();
    let a = new_partner(&env, &owner, "الف", "Partner A", None).unwrap();
    partners::update(&env.ctx, &owner, &update_input(&a, "احمد", "Ahmed", true)).unwrap();
    // The second edit still carries the first version.
    let stale =
        partners::update(&env.ctx, &owner, &update_input(&a, "اکرم", "Akram", true)).unwrap_err();
    assert_eq!(first(&stale, "x").0, ErrorCode::ErrVersionChanged);
    let mut missing = update_input(&a, "احمد", "Ahmed", true);
    missing.id = 999;
    let error = partners::update(&env.ctx, &owner, &missing).unwrap_err();
    assert_eq!(
        first(&error, "x"),
        (ErrorCode::ErrNotFound, Some("id".to_owned()), None)
    );
}

#[test]
fn a_partner_with_a_share_today_or_later_cannot_be_made_inactive() {
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let row = partners::get(&env.ctx, c).unwrap().row;
    let error = partners::update(
        &env.ctx,
        &owner,
        &update_input(&row, "ج", "Partner C", false),
    )
    .unwrap_err();
    assert_eq!(
        first(&error, "x"),
        (
            ErrorCode::ErrPartnerHasShare,
            Some("is_active".to_owned()),
            None
        )
    );
    // Giving ج 0% from today lets him go. The others are listed too.
    let rows = set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "50"), (b, "50"), (c, "0")],
    )
    .unwrap();
    let c_row = rows.iter().find(|r| r.id == c).unwrap();
    assert_eq!(c_row.share_bps, 0);
    let gone = partners::update(
        &env.ctx,
        &owner,
        &update_input(c_row, "ج", "Partner C", false),
    )
    .unwrap();
    assert!(!gone.is_active);
    // A share set for a later day also blocks it.
    let b_row = partners::get(&env.ctx, b).unwrap().row;
    set_shares(&env, &owner, "20-10-2026", &[(a, "60"), (b, "40")]).unwrap();
    let error = partners::update(
        &env.ctx,
        &owner,
        &update_input(&b_row, "ب", "Partner B", false),
    )
    .unwrap_err();
    assert_eq!(first(&error, "x").0, ErrorCode::ErrPartnerHasShare);
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM partners WHERE is_active = 0"),
        1
    );
}

#[test]
fn the_list_has_active_partners_first_then_by_id_and_each_share_today() {
    let (_dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "50"), (b, "0"), (c, "50")],
    )
    .unwrap();
    let row = partners::get(&env.ctx, b).unwrap().row;
    partners::update(
        &env.ctx,
        &owner,
        &update_input(&row, "ب", "Partner B", false),
    )
    .unwrap();
    let list = partners::list(&env.ctx).unwrap();
    assert_eq!(list.iter().map(|p| p.id).collect::<Vec<_>>(), vec![a, c, b]);
    assert_eq!(
        list.iter().map(|p| p.share_bps).collect::<Vec<_>>(),
        vec![5_000, 5_000, 0]
    );
    assert_eq!(
        list.iter().map(|p| p.is_active).collect::<Vec<_>>(),
        vec![true, true, false]
    );
}

#[test]
fn get_returns_the_share_history_oldest_first_and_unknown_ids_are_not_found() {
    let (_dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    set_shares(
        &env,
        &owner,
        "20-10-2026",
        &[(a, "50"), (b, "30"), (c, "20")],
    )
    .unwrap();
    let detail = partners::get(&env.ctx, a).unwrap();
    assert_eq!(
        detail.row.share_bps, 4_000,
        "today's share, not the later one"
    );
    let history: Vec<(i32, &str)> = detail
        .shares
        .iter()
        .map(|s| (s.share_bps, s.effective_from.as_str()))
        .collect();
    assert_eq!(history, vec![(4_000, "2026-10-10"), (5_000, "2026-10-20")]);
    let error = partners::get(&env.ctx, 999).unwrap_err();
    assert_eq!(
        first(&error, "x"),
        (ErrorCode::ErrNotFound, Some("id".to_owned()), None)
    );
}
