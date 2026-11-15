//! Plan 76 points 5 and 10 on the PRD worked day: a separate khata and the loans of persons linked
//! to the shop or to one partner (PRD FR-16.4, FR-16.6, FR-16.13, FR-17.9 v1.41; Umar 14-11-2026,
//! U1 to U3). Every expected number is the plan's check and is never changed to suit the code.
//!
//! The state is the opening position: cash 50,000, shares 40 / 30 / 30, no entry yet.
use super::*;
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_services::core::error::ErrorValue;
use mandi_services::core::khata::{KhataCreateRequest, KhataSetLinkInput};
use mandi_services::core::lent::{
    LentDuesInput, LentPersonCreateInput, LentPersonCreateRequest, LentPersonUpdateInput,
    LentReturnMode, MoneyLentInput, MoneyLentRequest, MoneyLentReturnInput,
};
use mandi_services::core::links::{Link, LinkKind, PersonType};
use mandi_services::core::partner_code::{PartnerCodeSetInput, PartnerCodeUnlockInput};
use mandi_services::core::statement::{PartnersStatementInput, StatementDetail, StatementHeading};
use mandi_services::{ServiceError, khata, lent, partner_code, partners};

const SHOP: Link = Link {
    kind: LinkKind::Shop,
    partner_id: None,
};

fn partner_link(partner: i64) -> Link {
    Link {
        kind: LinkKind::Partner,
        partner_id: Some(partner),
    }
}

/// Makes a khata with a link, confirming the spellings the engine is unsure of.
fn khata_with(w: &support::World, ur: &str, en: &str, link: Option<Link>) -> i64 {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        let request = KhataCreateRequest {
            base: mandi_services::core::khata::KhataCreateInput {
                name_ur: ur.into(),
                name_en: en.into(),
                note: None,
                confirmed_rules_words: words.clone(),
            },
            link,
        };
        match khata::create_linked(&w.env.ctx, &w.owner, &request) {
            Ok(done) => return done.account.id,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

/// Makes a person who can be lent money, with a type and a link.
fn person_with(
    w: &support::World,
    ur: &str,
    en: &str,
    person_type: Option<PersonType>,
    link: Option<Link>,
) -> i64 {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        let request = LentPersonCreateRequest {
            base: LentPersonCreateInput {
                name_ur: ur.into(),
                name_en: en.into(),
                phone: None,
                staff_id: None,
                confirmed_rules_words: words.clone(),
            },
            person_type,
            link,
        };
        match lent::person_create_linked(&w.env.ctx, &w.owner, &request) {
            Ok(person) => return person.account.id,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

fn lent_request(person: i64, amount: &str, date: &str, link: Option<Link>) -> MoneyLentRequest {
    MoneyLentRequest {
        base: MoneyLentInput {
            date: date.into(),
            person_account_id: Some(person),
            amount: amount.into(),
            reason: "اگلے سیزن کا باغ".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
        link,
    }
}

fn lent_back(person: i64, amount: &str, date: &str, mode: LentReturnMode) -> MoneyLentReturnInput {
    MoneyLentReturnInput {
        date: date.into(),
        person_account_id: Some(person),
        amount: amount.into(),
        mode,
        salary_head_id: None,
        narration: None,
    }
}

/// Each partner's کل رقم, in rupees, in the order they were made.
fn totals(w: &support::World) -> Vec<i64> {
    partners::overview(&w.env.ctx)
        .unwrap()
        .iter()
        .map(|r| r.total_paisa.rupees())
        .collect()
}

fn cash(w: &support::World) -> i64 {
    super::day_money::cash_now(w)
}

/// FR-17.9: Σ کل رقم + the firm's opening capital = the equity of the books (every 04, 05 and 06
/// account, the 0605 accounts included). The contra account is in head 03, outside it.
fn equity_holds(w: &support::World) {
    let sum: i64 = totals(w).iter().sum();
    let books = scalar(
        w,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) / 100 FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id
         WHERE substr(a.code, 1, 2) IN ('04', '05', '06')",
    );
    assert_eq!(sum + 65_000, books);
}

/// The contra account 03070001 balance (جمع less بنام) in rupees.
fn offset_balance(w: &support::World) -> i64 {
    scalar(
        w,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) / 100 FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id WHERE a.code = '03070001'",
    )
}

fn khata_balance(w: &support::World, id: i64) -> i64 {
    let (side, amount) = balance(w, id);
    match side {
        BalanceSide::Cr => amount,
        BalanceSide::Dr => -amount,
        BalanceSide::Zero => 0,
    }
}

#[test]
fn a_khata_linked_to_the_shop_splits_every_movement_by_the_shares_of_its_day() {
    let w = worked_day();
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", Some(SHOP));

    // 10-10-26: 10,000 paid out. Shares 40 / 30 / 30: 4,000 / 3,000 / 3,000, entry 20,000 = 20,000.
    let out = vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(family, "10,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(
        entry_lines(&w, out.entry_id)
            .iter()
            .map(|(code, dr, cr, _)| (code.clone(), *dr, *cr))
            .collect::<Vec<_>>()
            .len(),
        6
    );
    let total_dr: i64 = entry_lines(&w, out.entry_id).iter().map(|l| l.1).sum();
    let total_cr: i64 = entry_lines(&w, out.entry_id).iter().map(|l| l.2).sum();
    assert_eq!((total_dr, total_cr), (20_000, 20_000));
    assert_eq!(khata_balance(&w, family), -10_000);
    assert_eq!(cash(&w), 40_000);
    assert_eq!(totals(&w), vec![-4_000, -3_000, -3_000]);
    assert_eq!(offset_balance(&w), 10_000);
    equity_holds(&w);

    // 11-10-26: 4,000 comes back: 1,600 / 1,200 / 1,200. The khata is -6,000, the partners
    // -2,400 / -1,800 / -1,800.
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_khata_in(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_in_input(family, "4,000", "11-10-2026"),
    )
    .unwrap();
    assert_eq!(khata_balance(&w, family), -6_000);
    assert_eq!(cash(&w), 44_000);
    assert_eq!(totals(&w), vec![-2_400, -1_800, -1_800]);
    assert_eq!(offset_balance(&w), 6_000);
    equity_holds(&w);

    // The profit, the shared income and the khata's own page do not change: no share row, no
    // income, and its running balance is the khata's own.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM partner_share_rows"), 0);
}

#[test]
fn a_khata_linked_to_one_partner_is_all_his() {
    let w = worked_day();
    let a = w.partners[0];
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", Some(partner_link(a)));
    vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(family, "10,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-10_000, 0, 0]);
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_khata_in(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_in_input(family, "4,000", "11-10-2026"),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-6_000, 0, 0]);
    assert_eq!(khata_balance(&w, family), -6_000);
    equity_holds(&w);
}

#[test]
fn an_unlinked_khata_posts_exactly_as_before() {
    let w = worked_day();
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", None);
    let out = vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(family, "10,000", "10-10-2026"),
    )
    .unwrap();
    // Two lines only: the khata بنام and روکڑ جمع; no partner is touched (E2E-38).
    assert_eq!(entry_lines(&w, out.entry_id).len(), 2);
    assert_eq!(totals(&w), vec![0, 0, 0]);
    assert_eq!(offset_balance(&w), 0);
}

#[test]
fn the_link_of_a_khata_changes_only_while_its_balance_is_zero() {
    let w = worked_day();
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", None);
    // No balance yet: the link can be set, and set again.
    khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: family,
            link: SHOP,
        },
    )
    .unwrap();
    vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(family, "10,000", "10-10-2026"),
    )
    .unwrap();
    let refused = khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: family,
            link: partner_link(w.partners[0]),
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrLinkLocked);
    assert_eq!(first_field(&refused).as_deref(), Some("link"));
    // The same link is no change, so it is fine even with a balance.
    khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: family,
            link: SHOP,
        },
    )
    .unwrap();
    // Back to 0, and the link may change.
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_khata_in(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_in_input(family, "10,000", "11-10-2026"),
    )
    .unwrap();
    let done = khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: family,
            link: partner_link(w.partners[1]),
        },
    )
    .unwrap();
    assert_eq!(done.link.kind, LinkKind::Partner);
    assert_eq!(done.link.partner_id, Some(w.partners[1]));
}

#[test]
fn a_bad_link_is_refused_and_writes_nothing() {
    let w = worked_day();
    let before = scalar(&w, "SELECT COUNT(*) FROM accounts");
    // The spelling question comes first; answer it, then the link is what is refused.
    let mut words: Vec<String> = Vec::new();
    let no_partner = loop {
        let result = khata::create_linked(
            &w.env.ctx,
            &w.owner,
            &KhataCreateRequest {
                base: mandi_services::core::khata::KhataCreateInput {
                    name_ur: "خاندانی کھاتہ".into(),
                    name_en: "Family Khata".into(),
                    note: None,
                    confirmed_rules_words: words.clone(),
                },
                link: Some(Link {
                    kind: LinkKind::Partner,
                    partner_id: None,
                }),
            },
        )
        .unwrap_err();
        match &result {
            ServiceError::Rejected(items)
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            _ => break result,
        }
    };
    assert_eq!(no_partner.code(), ErrorCode::ErrLinkPartnerRequired);
    assert_eq!(first_field(&no_partner).as_deref(), Some("link.partner_id"));
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM accounts"), before);
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", None);
    let missing = khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: family,
            link: partner_link(9_999),
        },
    )
    .unwrap_err();
    assert_eq!(missing.code(), ErrorCode::ErrNotFound);
    assert_eq!(first_field(&missing).as_deref(), Some("link.partner_id"));
}

#[test]
fn a_garden_owner_linked_to_a_partner_lowers_his_total_and_a_return_raises_it() {
    let w = worked_day();
    let a = w.partners[0];
    let nawaz = person_with(
        &w,
        "نواز",
        "Nawaz",
        Some(PersonType::GardenOwner),
        Some(partner_link(a)),
    );
    // 10-10-26 (the plan says 01-11-26; the day does not matter): 20,000 lent, linked to الف.
    let saved = vouchers::save_money_lent_linked(
        &w.env.ctx,
        &w.owner,
        &lent_request(nawaz, "20,000", "10-10-2026", None),
    )
    .unwrap();
    let total_dr: i64 = entry_lines(&w, saved.entry_id).iter().map(|l| l.1).sum();
    let total_cr: i64 = entry_lines(&w, saved.entry_id).iter().map(|l| l.2).sum();
    assert_eq!((total_dr, total_cr), (40_000, 40_000));
    assert_eq!(totals(&w), vec![-20_000, 0, 0]);
    assert_eq!(cash(&w), 30_000);
    equity_holds(&w);

    // 11-10-26: he returns 5,000: open 15,000, الف -15,000.
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(nawaz, "5,000", "11-10-2026", LentReturnMode::Cash),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-15_000, 0, 0]);
    assert_eq!(super::day_money::open_of_person(&w, nawaz), 15_000);
    equity_holds(&w);

    // The tile and the page: 15,000 and 1 person, the same figure.
    let tiles = dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles;
    assert_eq!(tiles.others_dues_paisa.rupees(), 15_000);
    assert_eq!(tiles.others_people_count, 1);
    let dues = lent::dues(&w.env.ctx, &LentDuesInput::default()).unwrap();
    assert_eq!(dues.month, "2026-10");
    assert_eq!(dues.totals.open_paisa, tiles.others_dues_paisa);
    assert_eq!(dues.totals.open_people_count, 1);
    assert_eq!(dues.totals.given_paisa.rupees(), 20_000);
    assert_eq!(dues.totals.returned_paisa.rupees(), 5_000);
    assert_eq!(dues.rows.len(), 2);
    assert_eq!(dues.people.len(), 1);
    assert_eq!(dues.people[0].link.partner_id, Some(a));
    assert_eq!(dues.people[0].person_type, PersonType::GardenOwner);
}

#[test]
fn a_loan_linked_to_the_shop_posts_as_before_and_shows_a_note_in_each_statement() {
    let w = worked_day();
    // No link means the shop.
    let aslam = person_with(&w, "اسلم", "Aslam", Some(PersonType::Other), None);
    let saved = vouchers::save_money_lent_linked(
        &w.env.ctx,
        &w.owner,
        &lent_request(aslam, "5,000", "10-10-2026", None),
    )
    .unwrap();
    // Exactly E2E-30: two lines, no partner total moves.
    assert_eq!(entry_lines(&w, saved.entry_id).len(), 2);
    assert_eq!(totals(&w), vec![0, 0, 0]);
    assert_eq!(offset_balance(&w), 0);
    // Each statement shows his % as a note with no amount: 5,000 at 40 / 30 / 30.
    let mut parts = Vec::new();
    for partner in &w.partners {
        let statement = partners::statement(
            &w.env.ctx,
            &PartnersStatementInput {
                group_by_month: false,
                sort: None,
                partner_id: *partner,
                from: None,
                to: None,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap();
        let note = statement
            .lines
            .iter()
            .find(|l| l.detail == StatementDetail::LentShareNote)
            .expect("a note line");
        assert_eq!(note.heading, StatementHeading::Linked);
        assert_eq!(note.amount_paisa, None);
        let Some(ErrorValue::Number(part)) = note.values.get("part_paisa") else {
            panic!("the note has no part");
        };
        parts.push(*part / 100);
        assert_eq!(statement.closing_total_paisa.rupees(), 0);
    }
    assert_eq!(parts, vec![2_000, 1_500, 1_500]);
    // A salary deduction is still allowed for a shop loan (E2E-30).
    super::day_money::expense_head(&w, "تنخواہ", "Salaries");
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(aslam, "1,000", "11-10-2026", LentReturnMode::Salary),
    )
    .unwrap();
}

#[test]
fn a_partner_linked_loan_is_not_paid_back_from_a_salary_and_its_link_is_fixed_while_owed() {
    let w = worked_day();
    let a = w.partners[0];
    let nawaz = person_with(
        &w,
        "نواز",
        "Nawaz",
        Some(PersonType::GardenOwner),
        Some(partner_link(a)),
    );
    vouchers::save_money_lent_linked(
        &w.env.ctx,
        &w.owner,
        &lent_request(nawaz, "20,000", "10-10-2026", None),
    )
    .unwrap();
    // U3: a salary deduction on a loan linked to a partner is refused, at `mode`.
    set_today(&w.env, 2026, 10, 11);
    let salary = vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(nawaz, "1,000", "11-10-2026", LentReturnMode::Salary),
    )
    .unwrap_err();
    assert_eq!(salary.code(), ErrorCode::ErrSalaryNotAllowedForLinked);
    assert_eq!(first_field(&salary).as_deref(), Some("mode"));
    // U2: while he owes, a different link is refused and the loan with it is not saved.
    let vouchers_before = scalar(&w, "SELECT COUNT(*) FROM vouchers");
    let moved = vouchers::save_money_lent_linked(
        &w.env.ctx,
        &w.owner,
        &lent_request(nawaz, "1,000", "11-10-2026", Some(SHOP)),
    )
    .unwrap_err();
    assert_eq!(moved.code(), ErrorCode::ErrLinkFixedWhileOwed);
    assert_eq!(first_field(&moved).as_deref(), Some("link"));
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM vouchers"), vouchers_before);
    assert_eq!(totals(&w), vec![-20_000, 0, 0]);
    // The same link is fine.
    vouchers::save_money_lent_linked(
        &w.env.ctx,
        &w.owner,
        &lent_request(nawaz, "1,000", "11-10-2026", Some(partner_link(a))),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-21_000, 0, 0]);
    // And lent_person_update refuses the move too.
    let update = lent::person_update(
        &w.env.ctx,
        &w.owner,
        &LentPersonUpdateInput {
            person_account_id: nawaz,
            person_type: Some(PersonType::Other),
            link: Some(SHOP),
        },
    )
    .unwrap_err();
    assert_eq!(update.code(), ErrorCode::ErrLinkFixedWhileOwed);
    // He pays it all back, and the link may change.
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(nawaz, "21,000", "11-10-2026", LentReturnMode::Cash),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![0, 0, 0]);
    let done = lent::person_update(
        &w.env.ctx,
        &w.owner,
        &LentPersonUpdateInput {
            person_account_id: nawaz,
            person_type: Some(PersonType::Other),
            link: Some(SHOP),
        },
    )
    .unwrap();
    assert_eq!(done.link.kind, LinkKind::Shop);
    assert_eq!(done.person_type, PersonType::Other);
    equity_holds(&w);
}

#[test]
fn the_partner_statement_shows_the_linked_money_and_its_closing_is_his_total() {
    let w = worked_day();
    let a = w.partners[0];
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", Some(SHOP));
    vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(family, "10,000", "10-10-2026"),
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_khata_in(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_in_input(family, "4,000", "11-10-2026"),
    )
    .unwrap();
    let statement = partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            group_by_month: false,
            sort: None,
            partner_id: a,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let linked: Vec<(StatementDetail, i64)> = statement
        .lines
        .iter()
        .filter(|l| l.heading == StatementHeading::Linked)
        .map(|l| (l.detail, l.amount_paisa.unwrap().rupees()))
        .collect();
    // الف: 40% of 10,000 paid out, 40% of 4,000 back.
    assert_eq!(
        linked,
        vec![
            (StatementDetail::KhataPaid, -4_000),
            (StatementDetail::KhataReceived, 1_600)
        ]
    );
    assert_eq!(statement.closing_total_paisa.rupees(), -2_400);
    // The khata is named on each line.
    assert!(
        statement
            .lines
            .iter()
            .filter(|l| l.heading == StatementHeading::Linked)
            .all(|l| l.subject.as_ref().is_some_and(|s| s.id == family))
    );
}

#[test]
fn an_edit_of_a_linked_voucher_mirrors_the_new_version_and_a_cancel_takes_both_back() {
    let w = worked_day();
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", Some(SHOP));
    let out = vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(family, "10,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-4_000, -3_000, -3_000]);
    // An edit to 9,000 on the same day: the old parts and their reversal net to 0, the new ones are
    // 40 / 30 / 30 of 9,000 (3,600 / 2,700 / 2,700), and the contra holds 9,000.
    edit_khata_out(&w, out.voucher_id, family, "9,000").unwrap();
    assert_eq!(totals(&w), vec![-3_600, -2_700, -2_700]);
    assert_eq!(offset_balance(&w), 9_000);
    assert_eq!(khata_balance(&w, family), -9_000);
    equity_holds(&w);
    let row = vouchers::get(&w.env.ctx, out.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::VoucherCancelInput {
            id: out.voucher_id,
            version: row.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // The reversal copies the stored lines, mirror lines included: everything is back to 0.
    assert_eq!(totals(&w), vec![0, 0, 0]);
    assert_eq!(offset_balance(&w), 0);
    assert_eq!(khata_balance(&w, family), 0);
    assert_eq!(cash(&w), 50_000);
    equity_holds(&w);
}

#[test]
fn the_preview_shows_each_partner_part_worked_out_by_the_same_function_as_the_save() {
    use mandi_services::core::vouchers::VoucherEntry;
    let w = worked_day();
    let shop = khata_with(&w, "گھر کھاتہ", "Home Khata", Some(SHOP));
    let plain = khata_with(&w, "خاندانی کھاتہ", "Family Khata", None);
    let a = w.partners[0];
    let own = khata_with(&w, "ذاتی کھاتہ", "Personal Khata", Some(partner_link(a)));
    let parts = |entry: VoucherEntry| -> Vec<(i64, i64)> {
        preview_of(&w, entry)
            .linked_parts
            .iter()
            .map(|p| (p.partner_id, p.amount_paisa.rupees()))
            .collect()
    };
    // Paid out: minus, 40 / 30 / 30 of 10,000. Back: plus.
    let [pa, pb, pc] = w.partners;
    assert_eq!(
        parts(VoucherEntry::KhataOut(
            super::partner_money::khata_out_input(shop, "10,000", "10-10-2026")
        )),
        vec![(pa, -4_000), (pb, -3_000), (pc, -3_000)]
    );
    assert_eq!(
        parts(VoucherEntry::KhataIn(super::partner_money::khata_in_input(
            shop,
            "4,000",
            "10-10-2026"
        ))),
        vec![(pa, 1_600), (pb, 1_200), (pc, 1_200)]
    );
    // One partner: all his. An unlinked khata: nothing.
    assert_eq!(
        parts(VoucherEntry::KhataOut(
            super::partner_money::khata_out_input(own, "10,000", "10-10-2026")
        )),
        vec![(pa, -10_000)]
    );
    assert!(
        parts(VoucherEntry::KhataOut(
            super::partner_money::khata_out_input(plain, "10,000", "10-10-2026")
        ))
        .is_empty()
    );
    // The preview wrote nothing, and the save then books exactly the previewed parts.
    assert_eq!(totals(&w), vec![0, 0, 0]);
    vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(shop, "10,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-4_000, -3_000, -3_000]);
}

fn edit_khata_out(
    w: &support::World,
    voucher_id: i64,
    khata: i64,
    amount: &str,
) -> Result<mandi_services::core::vouchers::VoucherSaved, ServiceError> {
    use mandi_services::core::vouchers::{VoucherEditInput, VoucherEntry};
    let row = vouchers::get(&w.env.ctx, voucher_id).unwrap();
    vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: voucher_id,
            version: row.version,
            entry: VoucherEntry::KhataOut(super::partner_money::khata_out_input(
                khata,
                amount,
                "10-10-2026",
            )),
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
}

#[test]
fn an_entry_made_under_another_link_is_not_edited_and_a_cancel_still_takes_it_back() {
    // Money agent F1: shop khata, 10,000 out and back at 40 / 30 / 30, the link changed to partner ج
    // at balance 0. Editing the old entry would split the new version by the new link while its
    // reversal gives back the old one: refused, in the save and in the preview. A cancel is exact.
    let w = worked_day();
    let [pa, pb, pc] = w.partners;
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", Some(SHOP));
    let out = vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(family, "10,000", "10-10-2026"),
    )
    .unwrap();
    vouchers::save_khata_in(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_in_input(family, "10,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![0, 0, 0]);
    khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: family,
            link: partner_link(pc),
        },
    )
    .unwrap();
    let refused = edit_khata_out(&w, out.voucher_id, family, "9,000").unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrLinkChangedForEntry);
    assert_eq!(first_field(&refused).as_deref(), Some("link"));
    assert_eq!(totals(&w), vec![0, 0, 0], "a refused edit writes nothing");
    // F4: the preview of the edit says the same.
    let preview = vouchers::preview(
        &w.env.ctx,
        &mandi_services::core::vouchers::VoucherPreviewInput {
            entry: mandi_services::core::vouchers::VoucherEntry::KhataOut(
                super::partner_money::khata_out_input(family, "9,000", "10-10-2026"),
            ),
            replacing_voucher_id: Some(out.voucher_id),
        },
    )
    .unwrap();
    assert!(
        preview
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrLinkChangedForEntry)
    );
    assert!(preview.linked_parts.is_empty());
    // A cancel gives back exactly what the entry did under its own link.
    let row = vouchers::get(&w.env.ctx, out.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::VoucherCancelInput {
            id: out.voucher_id,
            version: row.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(totals(&w), vec![4_000, 3_000, 3_000]);
    assert_eq!(khata_balance(&w, family), 10_000);
    equity_holds(&w);
    let _ = (pa, pb);
}

#[test]
fn an_entry_made_before_the_link_stays_unmirrored_when_edited_and_one_moved_onto_a_linked_khata_is_mirrored()
 {
    // Money agent F2 and the pre-link rule.
    let w = worked_day();
    let plain = khata_with(&w, "خاندانی کھاتہ", "Family Khata", None);
    let shop = khata_with(&w, "گھر کھاتہ", "Home Khata", Some(SHOP));
    // Before the link: 5,000 out and back, then the khata is linked at balance 0.
    let old_out = vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(plain, "5,000", "10-10-2026"),
    )
    .unwrap();
    vouchers::save_khata_in(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_in_input(plain, "5,000", "10-10-2026"),
    )
    .unwrap();
    khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: plain,
            link: SHOP,
        },
    )
    .unwrap();
    // Editing the old entry: no mirror on the new version (its reversal has none either).
    edit_khata_out(&w, old_out.voucher_id, plain, "4,000").unwrap();
    assert_eq!(totals(&w), vec![0, 0, 0]);
    assert_eq!(offset_balance(&w), 0);
    // A new entry on the now linked khata is mirrored as usual.
    let fresh = vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(plain, "10,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-4_000, -3_000, -3_000]);
    // F2: an entry that was on an unlinked khata is moved to a linked one: the new version is
    // mirrored, so the later return does not leave the partners with a plus and no minus.
    let unlinked = khata_with(&w, "ذاتی کھاتہ", "Personal Khata", None);
    let moved = vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(unlinked, "10,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-4_000, -3_000, -3_000]);
    edit_khata_out(&w, moved.voucher_id, shop, "10,000").unwrap();
    assert_eq!(totals(&w), vec![-8_000, -6_000, -6_000]);
    assert_eq!(offset_balance(&w), 20_000);
    let _ = fresh;
    equity_holds(&w);
}

#[test]
fn a_typed_line_on_the_contra_account_or_a_partner_0605_account_is_refused() {
    use mandi_services::core::general::{GeneralInput, GeneralLineInput};
    let w = worked_day();
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    let cash_id = support::id_of(&w.env, "01010001");
    for code in ["03070001", "06050001"] {
        let account = support::id_of(&w.env, code);
        let refused = vouchers::general_save(
            &w.env.ctx,
            &w.owner,
            &GeneralInput {
                date: "10-10-2026".into(),
                narration: None,
                lines: vec![line(cash_id, "100", ""), line(account, "", "100")],
                confirm_cash_below_zero: false,
            },
        )
        .unwrap_err();
        assert!(
            matches!(&refused, ServiceError::Rejected(_)),
            "{code}: {refused:?}"
        );
    }
    assert_eq!(totals(&w), vec![0, 0, 0]);
    assert_eq!(cash(&w), 50_000);
}

#[test]
fn a_partner_with_a_linked_balance_is_not_made_inactive_and_his_0605_account_follows_his_name() {
    use mandi_services::core::partners::PartnerUpdateInput;
    let w = worked_day();
    let a = w.partners[0];
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", Some(partner_link(a)));
    vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &super::partner_money::khata_out_input(family, "1,000", "10-10-2026"),
    )
    .unwrap();
    let row = partners::list(&w.env.ctx)
        .unwrap()
        .into_iter()
        .find(|r| r.id == a)
        .unwrap();
    let update = |name_en: &str, active: bool| PartnerUpdateInput {
        id: a,
        version: row.version,
        name_ur: row.name_ur.clone(),
        name_en: name_en.into(),
        is_active: active,
        confirmed_rules_words: vec![],
    };
    let refused = partners::update(&w.env.ctx, &w.owner, &update("A", false)).unwrap_err();
    // The share check comes first when he still has a share; either way he is not made inactive.
    assert!(matches!(
        refused.code(),
        ErrorCode::ErrPartnerHasShare | ErrorCode::ErrPartnerHasLinkedBalance
    ));
    // A rename follows into the 0605 account.
    partners::update(&w.env.ctx, &w.owner, &update("Alif", true)).unwrap();
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM accounts a JOIN partners p ON p.linked_account_id = a.id
                 WHERE p.id = {a} AND a.name_en LIKE '%Alif%'"
            )
        ),
        1
    );
}

/// The (code, debit, credit) of the lines of an entry that are on the linked controls: the 0605
/// accounts of the partners and the contra account.
fn mirror_lines(w: &support::World, entry_id: i64) -> Vec<(String, i64, i64)> {
    let mut found: Vec<(String, i64, i64)> = entry_lines(w, entry_id)
        .into_iter()
        .filter(|(code, ..)| code.starts_with("0605") || code == "03070001")
        .map(|(code, dr, cr, _)| (code, dr, cr))
        .collect();
    found.sort();
    found
}

/// The money agent's check 5a (plan 76): every kind of line on a shop-linked khata gets the same
/// mirror pair, the goods it sells (E2E-38), fruit credited to it and a جنرل line, not only
/// the money paid out and in. Shares 40 / 30 / 30.
#[test]
fn a_lot_credit_fruit_and_a_general_line_on_a_shop_linked_khata_are_mirrored() {
    use mandi_services::core::bikri::LotInput;
    use mandi_services::core::fruit::{FruitInput, FruitSource};
    use mandi_services::core::general::{GeneralInput, GeneralLineInput};
    let w = worked_day();
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", Some(SHOP));

    // E2E-38's lot of the khata sells 6 of its 10 پیٹی کیلا to ارشد at 900 on credit: the lot credit
    // 6 x 900 = 5,400. Shop link: 2,160 / 1,620 / 1,620 on the partners, 5,400 on the contra.
    let mut draft = support::draft(vec![support::with_sales(
        LotInput {
            adjust_advance: false,
            ..support::bare_lot(&w, "khata-1", family, w.banana, "10")
        },
        vec![support::sale("s-1", w.arshad, "6", "900")],
    )]);
    draft.header.entry_date = "10-10-2026".into();
    let saved = save_ok(&w, &draft, false);
    assert_eq!(
        mirror_lines(&w, saved.entry_id),
        vec![
            ("03070001".to_owned(), 5_400, 0),
            ("06050001".to_owned(), 0, 2_160),
            ("06050002".to_owned(), 0, 1_620),
            ("06050003".to_owned(), 0, 1_620),
        ]
    );
    // The buyer side is income as always (comm 324 + لاگا 48); the khata's own page keeps +5,400.
    assert_eq!(khata_balance(&w, family), 5_400);
    assert_eq!(offset_balance(&w), -5_400);
    equity_holds(&w);

    // Fruit: الف takes 2 of the 4 left at 580 from the khata's lot. The khata (the lot's seller)
    // is credited 1,160 with no commission, mirrored 464 / 348 / 348.
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let fruit_saved = mandi_services::fruit::save(
        &w.env.ctx,
        &w.owner,
        &FruitInput {
            for_guests: false,
            partner_id: Some(w.partners[0]),
            date: "10-10-2026".into(),
            source: FruitSource::Shop,
            lot_id: Some(lot_id),
            agent_account_id: None,
            item_id: None,
            packing_id: None,
            qty: "2".into(),
            weight: None,
            rate: "580".into(),
            note: None,
            editing_id: None,
        },
    )
    .unwrap();
    let fruit_entry = fruit_saved.journal_entry_id;
    assert_eq!(
        mirror_lines(&w, fruit_entry),
        vec![
            ("03070001".to_owned(), 1_160, 0),
            ("06050001".to_owned(), 0, 464),
            ("06050002".to_owned(), 0, 348),
            ("06050003".to_owned(), 0, 348),
        ]
    );
    assert_eq!(khata_balance(&w, family), 6_560);
    equity_holds(&w);

    // A جنرل line: 1,000 credited to the khata against cash, mirrored 400 / 300 / 300.
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    let cash_id = support::id_of(&w.env, "01010001");
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &GeneralInput {
            date: "10-10-2026".into(),
            narration: None,
            lines: vec![line(cash_id, "1,000", ""), line(family, "", "1,000")],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let general_entry = scalar(&w, "SELECT MAX(id) FROM journal_entries");
    assert_eq!(
        mirror_lines(&w, general_entry),
        vec![
            ("03070001".to_owned(), 1_000, 0),
            ("06050001".to_owned(), 0, 400),
            ("06050002".to_owned(), 0, 300),
            ("06050003".to_owned(), 0, 300),
        ]
    );
    // The khata's balance is its own, and the contra is the opposite of it, to the rupee.
    assert_eq!(khata_balance(&w, family), 7_560);
    assert_eq!(offset_balance(&w), -7_560);
    equity_holds(&w);
}

/// Money agent 10a (plan 76): the loan preview works as if the link typed on the loan form were
/// already the person's, by the same function as the save (Code Standards B5), and refuses what the
/// save refuses.
#[test]
fn the_loan_preview_uses_the_link_typed_on_the_form() {
    use mandi_services::core::vouchers::{VoucherEntry, VoucherPreviewInput};
    let w = worked_day();
    let a = w.partners[0];
    // نواز has no link, so his loans belong to the shop.
    let nawaz = person_with(&w, "نواز", "Nawaz", Some(PersonType::GardenOwner), None);
    let preview = |link: Option<&Link>| {
        vouchers::preview_with_link(
            &w.env.ctx,
            &VoucherPreviewInput {
                entry: VoucherEntry::MoneyLent(super::day_money::lent_input(
                    nawaz,
                    "20,000",
                    "10-10-2026",
                    None,
                )),
                replacing_voucher_id: None,
            },
            link,
        )
        .unwrap()
    };
    let parts = |link: Option<&Link>| -> Vec<(i64, i64)> {
        preview(link)
            .linked_parts
            .iter()
            .map(|p| (p.partner_id, p.amount_paisa.rupees()))
            .collect()
    };
    // No typed link: the stored one, the shop, which posts as before and has no parts.
    assert!(parts(None).is_empty());
    // A partner link typed on the form: all of it is الف's, as the save will book it.
    assert_eq!(parts(Some(&partner_link(a))), vec![(a, -20_000)]);
    // The preview wrote nothing and did not set the link.
    assert_eq!(totals(&w), vec![0, 0, 0]);
    assert_eq!(
        scalar(
            &w,
            &format!("SELECT COUNT(*) FROM accounts WHERE id = {nawaz} AND link_kind = 'partner'")
        ),
        0
    );
    // A partner who does not exist is refused in the preview, at `link.partner_id`.
    let missing = Link {
        kind: LinkKind::Partner,
        partner_id: Some(99_999),
    };
    assert!(
        preview(Some(&missing))
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrNotFound && e.field == "link.partner_id")
    );
    // The save with the same typed link books exactly the previewed parts.
    vouchers::save_money_lent_linked(
        &w.env.ctx,
        &w.owner,
        &lent_request(nawaz, "20,000", "10-10-2026", Some(partner_link(a))),
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-20_000, 0, 0]);
    // While he owes, a different typed link is refused in the preview as it is in the save, and
    // the same link still shows his part.
    assert!(
        preview(Some(&SHOP))
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrLinkFixedWhileOwed && e.field == "link")
    );
    assert_eq!(parts(Some(&partner_link(a))), vec![(a, -20_000)]);
    assert_eq!(parts(None), vec![(a, -20_000)]);
    equity_holds(&w);
}

/// Money agent 10b (plan 76, decided by the lead): a جنرل entry that credits a person whose loans are
/// linked to a partner against a shop expense head is the salary deduction written by hand, so it is
/// refused like the deduction on its own screen. The same entry for a person linked to the shop, or
/// against a head that is not an expense, is not.
#[test]
fn a_general_entry_cannot_do_the_salary_deduction_of_a_partner_linked_loan() {
    use mandi_services::core::general::{GeneralInput, GeneralLineInput};
    let w = worked_day();
    let a = w.partners[0];
    let salary_head = super::day_money::expense_head(&w, "تنخواہ", "Salaries");
    let salary_account = scalar(
        &w,
        &format!("SELECT account_id FROM expense_heads WHERE id = {salary_head}"),
    );
    let cash_id = support::id_of(&w.env, "01010001");
    let linked_person = person_with(
        &w,
        "نواز",
        "Nawaz",
        Some(PersonType::GardenOwner),
        Some(partner_link(a)),
    );
    let shop_person = person_with(&w, "اشرف", "Ashraf", Some(PersonType::Other), Some(SHOP));
    for person in [linked_person, shop_person] {
        vouchers::save_money_lent_linked(
            &w.env.ctx,
            &w.owner,
            &lent_request(person, "5,000", "10-10-2026", None),
        )
        .unwrap();
    }
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    let input = |person: i64, expense: i64| GeneralInput {
        date: "10-10-2026".into(),
        narration: None,
        lines: vec![line(expense, "2,000", ""), line(person, "", "2,000")],
        confirm_cash_below_zero: false,
    };
    let before = (
        scalar(&w, "SELECT COUNT(*) FROM journal_entries"),
        totals(&w),
    );
    // Partner-linked person against the salary head: refused at the credit line, nothing written,
    // in the preview too (the same plan).
    let refused =
        vouchers::general_save(&w.env.ctx, &w.owner, &input(linked_person, salary_account))
            .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrSalaryNotAllowedForLinked);
    assert_eq!(
        first_field(&refused).as_deref(),
        Some("lines[1].account_id")
    );
    assert_eq!(
        (
            scalar(&w, "SELECT COUNT(*) FROM journal_entries"),
            totals(&w)
        ),
        before
    );
    // The same against an income account (ریٹ فرق, head 04) is refused too: it lowers the shared
    // income of all the partners (money agent B1).
    let income_account = support::id_of(&w.env, "04010005");
    let refused_income =
        vouchers::general_save(&w.env.ctx, &w.owner, &input(linked_person, income_account))
            .unwrap_err();
    assert_eq!(
        refused_income.code(),
        ErrorCode::ErrSalaryNotAllowedForLinked
    );
    // A person linked to the shop is still returned from a salary this way (his loan is the shop's).
    vouchers::general_save(&w.env.ctx, &w.owner, &input(shop_person, salary_account)).unwrap();
    // A partner-linked person credited against cash (a cash return written as a جنرل) is fine: it
    // is not a salary deduction.
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &GeneralInput {
            date: "10-10-2026".into(),
            narration: None,
            lines: vec![line(cash_id, "1,000", ""), line(linked_person, "", "1,000")],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    equity_holds(&w);
}

/// Plan 76 speed review: a shop link or no link that names a partner is a mistake of the caller (a
/// stale value left in a form). It is refused with `ERR_FORMAT_INVALID` on `link.partner_id`, never
/// dropped, and nothing changes.
#[test]
fn a_shop_link_or_no_link_that_names_a_partner_is_refused() {
    let w = worked_day();
    let a = w.partners[0];
    let family = khata_with(&w, "خاندانی کھاتہ", "Family Khata", None);
    for kind in [LinkKind::Shop, LinkKind::None] {
        let refused = khata::set_link(
            &w.env.ctx,
            &w.owner,
            &KhataSetLinkInput {
                khata_account_id: family,
                link: Link {
                    kind,
                    partner_id: Some(a),
                },
            },
        )
        .unwrap_err();
        assert_eq!(refused.code(), ErrorCode::ErrFormatInvalid, "{kind:?}");
        assert_eq!(first_field(&refused).as_deref(), Some("link.partner_id"));
    }
    // Still unlinked; a clean shop link and a partner link both work.
    khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: family,
            link: SHOP,
        },
    )
    .unwrap();
    khata::set_link(
        &w.env.ctx,
        &w.owner,
        &KhataSetLinkInput {
            khata_account_id: family,
            link: partner_link(a),
        },
    )
    .unwrap();
}

/// Plan 76 speed review: which partner a loan is linked to is a partner fact (PRD FR-17.11). While
/// the section is locked the dues page keeps the shop's amounts and totals, but a partner link has
/// no partner, the page says it is locked, and a filter by partner is refused.
#[test]
fn the_dues_page_does_not_name_the_partner_of_a_link_while_the_section_is_locked() {
    let w = worked_day();
    let a = w.partners[0];
    let nawaz = person_with(
        &w,
        "نواز",
        "Nawaz",
        Some(PersonType::GardenOwner),
        Some(partner_link(a)),
    );
    vouchers::save_money_lent_linked(
        &w.env.ctx,
        &w.owner,
        &lent_request(nawaz, "20,000", "10-10-2026", None),
    )
    .unwrap();
    let open = lent::dues(&w.env.ctx, &LentDuesInput::default()).unwrap();
    assert!(!open.partner_locked);
    assert_eq!(open.people[0].link, partner_link(a));
    assert_eq!(open.rows[0].link, partner_link(a));

    partner_code::set(
        &w.env.ctx,
        &w.owner,
        &PartnerCodeSetInput {
            password: "secret1".to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        },
    )
    .unwrap();
    partner_code::lock(&w.env.ctx).unwrap();
    let locked = lent::dues(&w.env.ctx, &LentDuesInput::default()).unwrap();
    assert!(locked.partner_locked);
    let hidden = Link {
        kind: LinkKind::Partner,
        partner_id: None,
    };
    assert_eq!(locked.people[0].link, hidden);
    assert_eq!(locked.rows[0].link, hidden);
    // The shop's figures are the same: the loan is owed to the shop, only its partner is hidden.
    assert_eq!(locked.totals, open.totals);
    assert_eq!(locked.total_count, open.total_count);
    assert_eq!(locked.people[0].open_paisa, open.people[0].open_paisa);
    // A filter by partner would reveal the link, so it is refused.
    let refused = lent::dues(
        &w.env.ctx,
        &LentDuesInput {
            partner_id: Some(a),
            ..LentDuesInput::default()
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrPartnerLocked);
    assert_eq!(first_field(&refused).as_deref(), Some("partner_id"));
    // The kind of link is not a partner fact, so that filter still works.
    let shop_only = lent::dues(
        &w.env.ctx,
        &LentDuesInput {
            link_kinds: vec![LinkKind::Shop],
            ..LentDuesInput::default()
        },
    )
    .unwrap();
    assert!(shop_only.people.is_empty());
    // After the code the partner is named again.
    partner_code::unlock(
        &w.env.ctx,
        &w.owner,
        &PartnerCodeUnlockInput {
            code: "4826".to_owned(),
        },
    )
    .unwrap();
    let again = lent::dues(&w.env.ctx, &LentDuesInput::default()).unwrap();
    assert!(!again.partner_locked);
    assert_eq!(again.people[0].link, partner_link(a));
}

/// Plan 76 speed review (Code Standards B31): the per-person list of the dues page is paged with
/// its count, so a person past the first page can be reached. Totals are over everyone.
#[test]
fn the_people_of_the_dues_page_are_paged_with_their_count() {
    let w = worked_day();
    let a = w.partners[0];
    // Three people with open amounts 20,000, 15,000 and 10,000 (the biggest comes first).
    for (ur, en, amount) in [
        ("نواز", "Nawaz", "20,000"),
        ("اسلم", "Aslam", "15,000"),
        ("بلال", "Bilal", "10,000"),
    ] {
        let person = person_with(
            &w,
            ur,
            en,
            Some(PersonType::GardenOwner),
            Some(partner_link(a)),
        );
        vouchers::save_money_lent_linked(
            &w.env.ctx,
            &w.owner,
            &lent_request(person, amount, "10-10-2026", None),
        )
        .unwrap();
    }
    let names = |page: &mandi_services::core::lent::LentDues| -> Vec<String> {
        page.people
            .iter()
            .map(|p| p.person.name_en.clone())
            .collect()
    };
    let all = lent::dues(&w.env.ctx, &LentDuesInput::default()).unwrap();
    assert_eq!(all.people_total_count, 3);
    assert_eq!(names(&all), ["Nawaz", "Aslam", "Bilal"]);
    assert_eq!(all.totals.open_paisa.rupees(), 45_000);

    let first = lent::dues(
        &w.env.ctx,
        &LentDuesInput {
            people_limit: 2,
            ..LentDuesInput::default()
        },
    )
    .unwrap();
    assert_eq!(names(&first), ["Nawaz", "Aslam"]);
    assert_eq!(first.people_total_count, 3);
    let second = lent::dues(
        &w.env.ctx,
        &LentDuesInput {
            people_offset: 2,
            people_limit: 2,
            ..LentDuesInput::default()
        },
    )
    .unwrap();
    assert_eq!(names(&second), ["Bilal"]);
    assert_eq!(second.people_total_count, 3);
    // The totals and the entries do not depend on the page of people.
    assert_eq!(second.totals, all.totals);
    assert_eq!(second.total_count, all.total_count);
    assert_eq!(second.rows, all.rows);
    // Past the end: no people, the same count.
    let past = lent::dues(
        &w.env.ctx,
        &LentDuesInput {
            people_offset: 3,
            ..LentDuesInput::default()
        },
    )
    .unwrap();
    assert!(past.people.is_empty());
    assert_eq!(past.people_total_count, 3);
}
