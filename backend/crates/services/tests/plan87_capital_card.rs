//! Plan 87 (PRD FR-16.14 v1.50, 17.1, FR-10.9, FR-6.3, FR-21.4): the card "where the capital is" with
//! its two blocks, the warning for ابتدائی فرق, and the fix that places a partner's opening capital in
//! روکڑ.
//!
//! The first world is shaped like Umar's data of 20-11-2026: two partners with an opening capital of
//! 500,000 each against ابتدائی فرق, party openings (buyers 105,000 owed to the shop, sellers 40,000
//! owed by the shop), cash below zero, the opening day closed. Its numbers are worked by hand:
//!
//! * partner openings: ابتدائی فرق بنام 1,000,000; party openings: جمع 105,000, بنام 40,000, so
//!   ابتدائی فرق stands بنام 1,000,000 + 40,000 − 105,000 = 935,000;
//! * cash −161,740 (an advance of 161,740 to a seller), so the advance is 161,740 with the seller;
//! * have 105,000 + 161,740 = 266,740; owe 161,740 (cash short) + 40,000 = 201,740; صافی 65,000;
//! * after the fix روکڑ is −161,740 + 1,000,000 = 838,260 and ابتدائی فرق stands جمع 65,000.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::reports::capital_card_print_data;
use mandi_services::core::accounts::{AccountType, BalanceSide, OpeningLineInput, OpeningsInput};
use mandi_services::core::auth::Language;
use mandi_services::core::cashbook::{
    CapitalDifferenceKind, CapitalDifferenceSide, CapitalLink, CapitalSide, CapitalWhere,
    CapitalWhereInput, CapitalWhereKind, CapitalWhoseKind,
};
use mandi_services::core::dead::{DeadMarkInput, DeadReason};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::partners::{
    OpeningCapitalWhere, PartnerCreateInput, PartnersPlaceOpeningCapitalInput,
};
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::month::{MonthInput, close as close_month};
use mandi_services::{
    ServiceError, accounts, capital_place, cashbook, day, dead, ledger, partners,
};
use support::World;

struct Umar {
    w: World,
    adnan: i64,
    usman: i64,
    adnan_capital: i64,
    usman_capital: i64,
}

fn make_partner(w: &World, ur: &str, en: &str, opening: &str) -> (i64, i64) {
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = PartnerCreateInput {
            name_ur: ur.into(),
            name_en: en.into(),
            opening_capital: Some(opening.into()),
            opening_capital_where: Some(OpeningCapitalWhere::CountedElsewhere),
            confirmed_rules_words: words.clone(),
        };
        match partners::create(&w.env.ctx, &w.owner, &input) {
            Ok(row) => return (row.id, row.capital_account_id),
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                        words.extend(found.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn open_party(w: &World, kind: AccountType, ur: &str, en: &str, dr: &str, cr: &str) -> i64 {
    let id = support::add_account(&w.env, &w.owner, kind, ur, en, true, "");
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: id,
                version: accounts::get(&w.env.ctx, id).unwrap().version,
                opening_date: "10-10-2026".into(),
                opening_dr: dr.into(),
                opening_cr: cr.into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    id
}

/// Umar's world with the opening day (10-10-2026) still open.
fn umar_open() -> Umar {
    let w = support::world_with(0, true);
    let (adnan, adnan_capital) = make_partner(&w, "عدنان", "Adnan", "500,000");
    let (usman, usman_capital) = make_partner(&w, "عثمان", "Usman", "500,000");
    // Buyers owe 30,000 + 15,000 + 60,000 = 105,000; sellers are owed 10,000 + 10,000 + 20,000.
    open_party(&w, AccountType::Buyer, "مانی", "Mani", "30,000", "");
    open_party(&w, AccountType::Buyer, "انس", "Anas", "15,000", "");
    open_party(
        &w,
        AccountType::Buyer,
        "عدنان خان",
        "Adnan Khan",
        "60,000",
        "",
    );
    open_party(&w, AccountType::Seller, "فیصل", "Faisal", "", "10,000");
    open_party(&w, AccountType::Seller, "ہارون", "Haroon", "", "10,000");
    open_party(&w, AccountType::Seller, "احسان", "Ehsan", "", "20,000");
    // Cash goes below zero: an advance of 161,740 to رحیم, confirmed (PRD FR-6.7).
    mandi_services::vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::AdvanceInput {
            confirm_cash_below_zero: true,
            date: "10-10-2026".into(),
            seller_account_id: w.rahim,
            amount: "161,740".into(),
            removed: Default::default(),
            expected_goods: String::new(),
            narration: String::new(),
            reason: None,
        },
    )
    .unwrap();
    Umar {
        w,
        adnan,
        usman,
        adnan_capital,
        usman_capital,
    }
}

/// Umar's world with the opening day closed (the days up to 19-11-2026), today 20-11-2026.
fn umar_closed() -> Umar {
    let u = umar_open();
    common::set_today(&u.w.env, 2026, 11, 20);
    day::close(
        &u.w.env.ctx,
        &u.w.owner,
        &DayCloseInput {
            up_to: "19-11-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(closed_up_to(&u.w).as_deref(), Some("2026-11-19"));
    u
}

fn closed_up_to(w: &World) -> Option<String> {
    day::status(&w.env.ctx).unwrap().closed_up_to
}

fn card(w: &World) -> CapitalWhere {
    cashbook::capital_where(&w.env.ctx).unwrap()
}

/// The side and the amount (rupees) of the first line of a kind, in either block.
fn find(card: &CapitalWhere, kind: CapitalWhereKind) -> Option<(CapitalSide, i64)> {
    card.have_lines
        .iter()
        .chain(&card.owe_lines)
        .find(|l| l.kind == kind)
        .map(|l| (l.side, l.amount_paisa.rupees()))
}

fn kinds(card: &CapitalWhere, side: CapitalSide) -> Vec<CapitalWhereKind> {
    let lines = match side {
        CapitalSide::Have => &card.have_lines,
        CapitalSide::Owe => &card.owe_lines,
    };
    lines.iter().map(|l| l.kind).collect()
}

fn partner_totals(card: &CapitalWhere) -> Vec<(i64, i64)> {
    card.whose_lines
        .iter()
        .filter(|l| l.kind == CapitalWhoseKind::PartnerTotal)
        .map(|l| (l.partner.as_ref().unwrap().id, l.amount_paisa.rupees()))
        .collect()
}

fn cash(card: &CapitalWhere) -> (CapitalSide, i64) {
    find(card, CapitalWhereKind::Cash).expect("cash is always on the card")
}

fn input(sign: &str, ids: Vec<i64>) -> PartnersPlaceOpeningCapitalInput {
    PartnersPlaceOpeningCapitalInput {
        partner_ids: ids,
        reason: "سرمایہ روکڑ میں درج کریں".into(),
        sign: sign.into(),
    }
}

fn refusal(error: &ServiceError) -> (ErrorCode, Option<String>) {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal of the entry: {error:?}");
    };
    (items[0].code, items[0].field.clone())
}

fn balance(w: &World, account: i64) -> (BalanceSide, i64) {
    let b = ledger::account_balance(&w.env.ctx, account).unwrap();
    (b.side, b.amount_paisa.rupees())
}

/// The rows of the tables the fix must not touch (PRD FR-16.14 v1.50), then the journal entries.
fn row_counts(w: &World) -> Vec<i64> {
    let conn = common::raw(&w.dir);
    [
        "partner_share_rows",
        "bill_allocations",
        "advance_allocations",
        "analytics_facts",
        "stock_moves",
    ]
    .iter()
    .map(|table| {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    })
    .collect()
}

fn journal_entries(w: &World) -> i64 {
    common::raw(&w.dir)
        .query_row("SELECT COUNT(*) FROM journal_entries", [], |r| r.get(0))
        .unwrap()
}

fn assert_balanced(w: &World) {
    let tb = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(tb.total_dr, tb.total_cr, "the trial balance must balance");
}

#[test]
fn before_the_fix_the_card_names_the_unplaced_capital_and_the_party_openings() {
    let u = umar_closed();
    let c = card(&u.w);
    // Where the money is: no minus figure anywhere, cash below zero is a line of the owe block.
    assert_eq!(cash(&c), (CapitalSide::Owe, 161_740));
    assert_eq!(
        find(&c, CapitalWhereKind::BuyersOwe),
        Some((CapitalSide::Have, 105_000))
    );
    assert_eq!(
        find(&c, CapitalWhereKind::SellerAdvances),
        Some((CapitalSide::Have, 161_740))
    );
    assert_eq!(
        find(&c, CapitalWhereKind::SellersPayable),
        Some((CapitalSide::Owe, 40_000))
    );
    assert_eq!(
        find(&c, CapitalWhereKind::DeadAmount),
        Some((CapitalSide::Have, 0)),
        "the dead amount is always on its own line"
    );
    assert_eq!(c.have_total_paisa.rupees(), 266_740);
    assert_eq!(c.owe_total_paisa.rupees(), 201_740);
    assert_eq!(c.net_paisa.rupees(), 65_000);
    assert_eq!(c.net_side, CapitalSide::Have);
    assert!(
        c.have_lines
            .iter()
            .chain(&c.owe_lines)
            .all(|l| !l.amount_paisa.is_negative()),
        "no line is below 0"
    );
    // Whose it is: the five partners and 1,000,000 of capital less the advance parts.
    assert_eq!(c.whose_lines_total_paisa.rupees(), 1_000_000);
    assert!(c.sides_equal);
    // Shares of the partners' money on the card (1,000,000): Adnan and Usman 50% each (5,000 bps),
    // the open advance parts 161,740 of 1,000,000 = 16.17% (1,617); a partner below 0 has none.
    let share_of = |kind: CapitalWhoseKind, partner: Option<i64>| {
        c.whose_lines
            .iter()
            .find(|l| l.kind == kind && l.partner.as_ref().map(|p| p.id) == partner)
            .map(|l| l.share_bps)
    };
    assert_eq!(
        share_of(CapitalWhoseKind::PartnerTotal, Some(u.adnan)),
        Some(Some(5_000))
    );
    assert_eq!(
        share_of(CapitalWhoseKind::PartnerTotal, Some(u.usman)),
        Some(Some(5_000))
    );
    assert_eq!(
        share_of(CapitalWhoseKind::PartnerTotal, Some(u.w.partners[0])),
        Some(None)
    );
    assert_eq!(
        share_of(CapitalWhoseKind::AdvancesOpen, None),
        Some(Some(1_617))
    );
    assert_eq!(c.whose_total_paisa, c.net_paisa);
    // The warning: 1,000,000 of partner capital against ابتدائی فرق, party openings 65,000 (جمع),
    // net 935,000 بنام.
    let d = c.difference.as_ref().expect("the warning");
    assert_eq!(d.amount_paisa.rupees(), 935_000);
    assert_eq!(d.side, CapitalDifferenceSide::Dr);
    let capital = &d.groups[0];
    assert_eq!(capital.kind, CapitalDifferenceKind::PartnerCapital);
    assert_eq!(capital.amount_paisa.rupees(), 1_000_000);
    assert_eq!(capital.side, CapitalDifferenceSide::Dr);
    assert_eq!(capital.parts.len(), 2);
    assert!(
        capital
            .parts
            .iter()
            .all(|p| p.can_place && p.partner.is_some())
    );
    assert!(capital.parts.iter().all(|p| p.date == "2026-10-10"));
    let openings = &d.groups[1];
    assert_eq!(openings.kind, CapitalDifferenceKind::AccountOpening);
    assert_eq!(openings.amount_paisa.rupees(), 65_000);
    assert_eq!(openings.side, CapitalDifferenceSide::Cr);
    assert_eq!(openings.parts.len(), 6, "three buyers and three sellers");
    assert!(openings.parts.iter().all(|p| !p.can_place));
    assert_eq!(d.groups.len(), 2);
    let placeable: Vec<i64> = d.placeable_partners.iter().map(|p| p.id).collect();
    assert_eq!(placeable, vec![u.adnan, u.usman]);
}

#[test]
fn the_fix_moves_the_capital_into_cash_and_nothing_else() {
    let u = umar_closed();
    let before = card(&u.w);
    let totals_before = partner_totals(&before);
    let rows_before = row_counts(&u.w);
    let entries_before = journal_entries(&u.w);
    let placed = capital_place::place(
        &u.w.env.ctx,
        &u.w.owner,
        &input("adnan", vec![u.adnan, u.usman]),
    )
    .unwrap();
    assert_eq!(placed.placed.len(), 2);
    assert!(placed.skipped.is_empty());
    assert_eq!(placed.reopened_from.as_deref(), Some("2026-10-10"));
    assert_eq!(placed.closed_up_to.as_deref(), Some("2026-11-19"));
    assert_eq!(placed.cash_after_paisa.rupees(), 838_260);
    for p in &placed.placed {
        assert_eq!(p.date, "2026-10-10");
        assert_eq!(p.amount_paisa.rupees(), 500_000);
        assert_ne!(p.old_entry_id, p.new_entry_id);
    }
    // Only the journal changed: a reversal and a new opening for each partner (4 entries). No share
    // row, matching row, analytics fact or stock move was written (PRD FR-16.14 v1.50).
    assert_eq!(row_counts(&u.w), rows_before);
    assert_eq!(journal_entries(&u.w), entries_before + 4);
    // The closed-up-to day is back, the books balance, the chain holds.
    assert_eq!(closed_up_to(&u.w).as_deref(), Some("2026-11-19"));
    assert_balanced(&u.w);
    common::assert_chain_ok(&u.w.dir);
    // Cash is exactly 1,000,000 higher; ابتدائی فرق is 65,000 جمع; the capital accounts are untouched.
    let after = card(&u.w);
    assert_eq!(cash(&after), (CapitalSide::Have, 838_260));
    assert_eq!(
        balance(&u.w, support::id_of(&u.w.env, "06019999")),
        (BalanceSide::Cr, 65_000)
    );
    assert_eq!(balance(&u.w, u.adnan_capital), (BalanceSide::Cr, 500_000));
    assert_eq!(balance(&u.w, u.usman_capital), (BalanceSide::Cr, 500_000));
    // Partner totals are unchanged; so is everything else of the card but cash and the difference.
    assert_eq!(partner_totals(&after), totals_before);
    assert_eq!(after.have_total_paisa.rupees(), 838_260 + 105_000 + 161_740);
    assert_eq!(after.owe_total_paisa.rupees(), 40_000);
    assert_eq!(after.net_paisa.rupees(), 1_065_000);
    assert_eq!(after.whose_lines_total_paisa.rupees(), 1_000_000);
    assert_eq!(after.whose_total_paisa, after.net_paisa);
    let d = after.difference.as_ref().expect("old books difference");
    assert_eq!(d.amount_paisa.rupees(), 65_000);
    assert_eq!(d.side, CapitalDifferenceSide::Cr);
    assert_eq!(d.groups.len(), 1);
    assert_eq!(d.groups[0].kind, CapitalDifferenceKind::AccountOpening);
    assert!(d.placeable_partners.is_empty());
    // The same two days: the cash book shows the two opening capital rows on 10-10-2026.
    let book = cashbook::get(
        &u.w.env.ctx,
        &mandi_services::core::cashbook::CashbookInput::default(),
    )
    .unwrap();
    assert_eq!(book.closing_paisa.rupees(), 838_260);
    // One audit row for each partner, with the reopen and the close beside them.
    assert_eq!(common::audit_count(&u.w.dir, "opening_capital_placed"), 2);
    assert_eq!(common::audit_count(&u.w.dir, "day_reopen"), 1);
    assert_eq!(common::audit_count(&u.w.dir, "day_close"), 2);
}

#[test]
fn a_wrong_sign_changes_nothing_and_is_audited_without_its_text() {
    let u = umar_closed();
    let before = card(&u.w);
    let error = capital_place::place(
        &u.w.env.ctx,
        &u.w.owner,
        &input("munshi", vec![u.adnan, u.usman]),
    )
    .unwrap_err();
    assert_eq!(
        refusal(&error),
        (ErrorCode::ErrSignWrong, Some("sign".into()))
    );
    assert_eq!(card(&u.w), before);
    assert_eq!(closed_up_to(&u.w).as_deref(), Some("2026-11-19"));
    assert_eq!(common::audit_count(&u.w.dir, "capital_place_sign_wrong"), 1);
    assert_eq!(common::audit_count(&u.w.dir, "opening_capital_placed"), 0);
    assert_eq!(common::audit_count(&u.w.dir, "day_reopen"), 0);
    // A missing sign or reason is refused together, before anything is read.
    let mut empty = input("", vec![u.adnan]);
    empty.reason = String::new();
    let error = capital_place::place(&u.w.env.ctx, &u.w.owner, &empty).unwrap_err();
    let ServiceError::Rejected(items) = &error else {
        panic!("{error:?}");
    };
    let codes: Vec<ErrorCode> = items.iter().map(|i| i.code).collect();
    assert!(codes.contains(&ErrorCode::ErrReasonRequired));
    assert!(codes.contains(&ErrorCode::ErrSignRequired));
    assert_eq!(card(&u.w), before);
}

#[test]
fn a_second_call_has_nothing_to_place_and_other_inputs_are_checked() {
    let u = umar_closed();
    capital_place::place(
        &u.w.env.ctx,
        &u.w.owner,
        &input("adnan", vec![u.adnan, u.usman]),
    )
    .unwrap();
    let after = card(&u.w);
    let again = capital_place::place(
        &u.w.env.ctx,
        &u.w.owner,
        &input("adnan", vec![u.adnan, u.usman]),
    )
    .unwrap_err();
    assert_eq!(
        refusal(&again),
        (ErrorCode::ErrNothingToPlace, Some("partner_ids".into()))
    );
    // A partner who is not there, and no partner at all.
    let missing =
        capital_place::place(&u.w.env.ctx, &u.w.owner, &input("adnan", vec![9_999])).unwrap_err();
    assert_eq!(
        refusal(&missing),
        (ErrorCode::ErrNotFound, Some("partner_ids[0]".into()))
    );
    let none =
        capital_place::place(&u.w.env.ctx, &u.w.owner, &input("adnan", Vec::new())).unwrap_err();
    assert_eq!(
        refusal(&none),
        (ErrorCode::ErrRequired, Some("partner_ids".into()))
    );
    assert_eq!(card(&u.w), after, "the refusals changed nothing");
    assert_eq!(closed_up_to(&u.w).as_deref(), Some("2026-11-19"));
}

#[test]
fn a_listed_partner_with_nothing_to_move_is_skipped() {
    let u = umar_closed();
    let a = u.w.partners[0];
    let placed =
        capital_place::place(&u.w.env.ctx, &u.w.owner, &input("adnan", vec![a, u.adnan])).unwrap();
    assert_eq!(placed.placed.len(), 1);
    assert_eq!(placed.placed[0].partner.id, u.adnan);
    assert_eq!(
        placed.skipped.iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![a]
    );
    // Only Adnan's 500,000 came into cash; Usman is still on the difference and can be placed.
    let c = card(&u.w);
    assert_eq!(cash(&c), (CapitalSide::Have, 338_260));
    let d = c.difference.unwrap();
    assert_eq!(
        d.placeable_partners
            .iter()
            .map(|p| p.id)
            .collect::<Vec<_>>(),
        vec![u.usman]
    );
    assert_balanced(&u.w);
}

#[test]
fn an_opening_day_that_is_open_is_not_reopened() {
    let u = umar_open();
    let placed = capital_place::place(
        &u.w.env.ctx,
        &u.w.owner,
        &input("ADNAN ", vec![u.adnan, u.usman]),
    )
    .unwrap();
    assert_eq!(placed.reopened_from, None);
    assert_eq!(placed.closed_up_to, None);
    assert_eq!(common::audit_count(&u.w.dir, "day_reopen"), 0);
    assert_eq!(cash(&card(&u.w)), (CapitalSide::Have, 838_260));
    assert_balanced(&u.w);
}

#[test]
fn the_card_as_of_a_day_counts_only_the_entries_up_to_it() {
    let u = umar_closed();
    let today = card(&u.w);
    let at = |text: &str| {
        cashbook::capital_where_at(
            &u.w.env.ctx,
            &CapitalWhereInput {
                as_of: Some(text.into()),
            },
        )
    };
    // Everything was dated 10-10-2026, so that day shows today's figures (the dated reads).
    let on = at("10-10-2026").unwrap();
    assert_eq!(on.as_of, "2026-10-10");
    let mut same = on.clone();
    same.as_of = today.as_of.clone();
    assert_eq!(same, today);
    // The day before there is nothing.
    let before = at("09-10-2026").unwrap();
    assert_eq!(before.net_paisa.rupees(), 0);
    assert!(before.difference.is_none());
    assert_eq!(cash(&before), (CapitalSide::Have, 0));
    assert!(partner_totals(&before).iter().all(|(_, total)| *total == 0));
    // A day after today is refused, and so is a text that is not a date.
    let future = at("21-11-2026").unwrap_err();
    assert_eq!(
        refusal(&future),
        (ErrorCode::ErrDateInFuture, Some("as_of".into()))
    );
    assert!(at("not a date").is_err());
}

#[test]
fn the_fix_leaves_the_card_of_the_opening_day_with_cash_added() {
    // The dated card (as of 10-10-2026) follows the fix too, because the openings keep their day.
    let u = umar_closed();
    capital_place::place(
        &u.w.env.ctx,
        &u.w.owner,
        &input("adnan", vec![u.adnan, u.usman]),
    )
    .unwrap();
    let on = cashbook::capital_where_at(
        &u.w.env.ctx,
        &CapitalWhereInput {
            as_of: Some("10-10-2026".into()),
        },
    )
    .unwrap();
    assert_eq!(cash(&on), (CapitalSide::Have, 838_260));
    assert_eq!(on.net_paisa.rupees(), 1_065_000);
}

#[test]
fn after_events_1_to_5_the_card_is_the_worked_day() {
    // 58,484 + 21,564 = 80,048 held; 10,160 + 4,400 owed; صافی 65,488 = −1,004 + 746 + 746 + 65,000.
    let w = reports_world::full_day();
    let c = card(&w);
    assert_eq!(cash(&c), (CapitalSide::Have, 58_484));
    assert_eq!(
        find(&c, CapitalWhereKind::BuyersOwe),
        Some((CapitalSide::Have, 21_564))
    );
    assert_eq!(
        find(&c, CapitalWhereKind::SellersPayable),
        Some((CapitalSide::Owe, 10_160))
    );
    let charges: Vec<i64> = c
        .owe_lines
        .iter()
        .filter(|l| l.kind == CapitalWhereKind::OwedForCharges)
        .map(|l| {
            assert!(l.account.is_some(), "a charge line names its account");
            assert_eq!(l.link, CapitalLink::AccountLedger);
            l.amount_paisa.rupees()
        })
        .collect();
    assert_eq!(charges, vec![300, 4_000, 100]);
    assert_eq!(c.have_total_paisa.rupees(), 80_048);
    assert_eq!(c.owe_total_paisa.rupees(), 14_560);
    assert_eq!(c.net_paisa.rupees(), 65_488);
    assert_eq!(
        kinds(&c, CapitalSide::Have),
        vec![
            CapitalWhereKind::Cash,
            CapitalWhereKind::BuyersOwe,
            CapitalWhereKind::DeadAmount
        ]
    );
    let totals: Vec<i64> = partner_totals(&c).into_iter().map(|(_, t)| t).collect();
    assert_eq!(totals, vec![-1_004, 746, 746]);
    let opening: Vec<i64> = c
        .whose_lines
        .iter()
        .filter(|l| l.kind == CapitalWhoseKind::OpeningCapital)
        .map(|l| l.amount_paisa.rupees())
        .collect();
    assert_eq!(opening, vec![65_000]);
    assert!(c.difference.is_none());
    assert_eq!(c.whose_total_paisa.rupees(), 65_488);
    assert_eq!(c.whose_total_paisa, c.net_paisa);
    // The share of the total, from the shared formula: 746 of 65,488 is 1.14% (114), 65,000 is 99.25%
    // (9,925); a partner below 0 has none.
    let shares: Vec<Option<i32>> = c.whose_lines.iter().map(|l| l.share_bps).collect();
    assert_eq!(shares, vec![None, Some(114), Some(114), Some(9_925), None]);
}

#[test]
fn a_person_in_the_dead_amount_is_on_his_own_line_and_the_net_does_not_move() {
    // E2E-44: ارشد dies on 11-10-26 owing 18,684; the dead amount is 18,684 and buyers' dues 2,880.
    let w = reports_world::full_day();
    common::set_today(&w.env, 2026, 10, 11);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.arshad,
            entry_date: "11-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();
    let c = card(&w);
    assert_eq!(
        find(&c, CapitalWhereKind::BuyersOwe),
        Some((CapitalSide::Have, 2_880))
    );
    assert_eq!(
        find(&c, CapitalWhereKind::DeadAmount),
        Some((CapitalSide::Have, 18_684))
    );
    assert_eq!(c.net_paisa.rupees(), 65_488);
    assert_eq!(c.whose_total_paisa, c.net_paisa);
    // As of the day before the move he was still a buyer who owes.
    let before = cashbook::capital_where_at(
        &w.env.ctx,
        &CapitalWhereInput {
            as_of: Some("10-10-2026".into()),
        },
    )
    .unwrap();
    assert_eq!(
        find(&before, CapitalWhereKind::BuyersOwe),
        Some((CapitalSide::Have, 21_564))
    );
    assert_eq!(
        find(&before, CapitalWhereKind::DeadAmount),
        Some((CapitalSide::Have, 0))
    );
}

#[test]
fn a_month_close_on_the_days_to_reopen_refuses_the_fix_and_changes_nothing() {
    // October is closed as a month (PRD FR-17.6): its days cannot be reopened (FR-21.2), so the fix
    // is refused before anything is written.
    let w = reports_world::full_day();
    let (dawood, _) = make_partner(&w, "داؤد", "Dawood", "50,000");
    common::set_today(&w.env, 2026, 11, 2);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .unwrap();
    close_month(
        &w.env.ctx,
        &w.owner,
        &MonthInput {
            month: "2026-10".into(),
        },
    )
    .unwrap();
    let before = card(&w);
    let rows = row_counts(&w);
    let entries = journal_entries(&w);
    let error =
        capital_place::place(&w.env.ctx, &w.owner, &input("adnan", vec![dawood])).unwrap_err();
    assert_eq!(
        refusal(&error),
        (ErrorCode::ErrMonthClosed, Some("date".into()))
    );
    assert_eq!(card(&w), before);
    assert_eq!(row_counts(&w), rows);
    assert_eq!(journal_entries(&w), entries);
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-31"));
    assert_eq!(common::audit_count(&w.dir, "opening_capital_placed"), 0);
    assert_eq!(common::audit_count(&w.dir, "day_reopen"), 0);
}

#[test]
fn a_dead_seller_with_an_open_advance_is_in_the_dead_amount_and_the_partners_advance_parts() {
    // رحیم holds an open advance of 15,000 since 01-09-2026 and owes it. In the dead amount his whole
    // balance is one line on the left (PRD FR-10.10), while the partners' open advance parts on the
    // right (FR-17.13) still count it: the two lines differ by it, and the sides stay equal.
    let w = reports_world::worked_day();
    common::set_today(&w.env, 2026, 10, 10);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.rahim,
            entry_date: "10-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();
    let c = card(&w);
    assert_eq!(find(&c, CapitalWhereKind::SellerAdvances), None);
    assert_eq!(
        find(&c, CapitalWhereKind::DeadAmount),
        Some((CapitalSide::Have, 15_000))
    );
    assert_eq!(cash(&c), (CapitalSide::Have, 50_000));
    let open_parts: Vec<i64> = c
        .whose_lines
        .iter()
        .filter(|l| l.kind == CapitalWhoseKind::AdvancesOpen)
        .map(|l| l.amount_paisa.rupees())
        .collect();
    assert_eq!(open_parts, vec![15_000]);
    assert_eq!(c.net_paisa.rupees(), 65_000);
    assert_eq!(c.whose_total_paisa, c.net_paisa);
}

#[test]
fn a_net_below_zero_is_sent_unsigned_with_its_side() {
    // Only a seller's opening: the shop owes him 100,000 and holds nothing, so the headline is
    // 100,000 on the `owe` side, never a minus figure (PRD FR-16.14 v1.50).
    let w = support::world_with(0, false);
    open_party(&w, AccountType::Seller, "فیصل", "Faisal", "", "100,000");
    let c = card(&w);
    assert_eq!(c.have_total_paisa.rupees(), 0);
    assert_eq!(c.owe_total_paisa.rupees(), 100_000);
    assert_eq!(c.net_paisa.rupees(), 100_000);
    assert_eq!(c.net_side, CapitalSide::Owe);
    assert_eq!(c.whose_total_paisa, c.net_paisa);
    assert_eq!(c.whose_lines_total_paisa.rupees(), 0);
    assert_eq!(c.whose_lines_side, CapitalSide::Have);
    let d = c.difference.expect("the opening has no capital behind it");
    assert_eq!(d.amount_paisa.rupees(), 100_000);
    assert_eq!(d.side, CapitalDifferenceSide::Dr);
    assert!(
        c.have_lines
            .iter()
            .chain(&c.owe_lines)
            .all(|l| !l.amount_paisa.is_negative())
    );
}

#[test]
fn the_print_is_the_card_laid_out_in_the_print_language() {
    // The worked day: every figure of the print is the card's, written with digits 0 to 9, and the
    // totals row is the partners' total capital (PRD FR-16.14 v1.50, FR-19.6).
    let w = reports_world::full_day();
    let c = card(&w);
    for language in [Language::Ur, Language::En] {
        let data = capital_card_print_data(w.env.ctx.db(), language, None, &c).unwrap();
        assert_eq!(data.totals_label_id, "print-capital-total");
        let text = serde_json::to_string(&data).unwrap();
        assert!(text.contains("print-capital-title"));
        for figure in [
            "58,484", "21,564", "10,160", "65,488", "65,000", "80,048", "14,560",
        ] {
            assert!(
                text.contains(figure),
                "{figure} is missing in the {language:?} print"
            );
        }
        assert!(
            !text
                .chars()
                .any(|ch| ('\u{06F0}'..='\u{06F9}').contains(&ch)
                    || ('\u{0660}'..='\u{0669}').contains(&ch)),
            "digits must be 0 to 9"
        );
        // The first row heads the have block; the partner below 0 is an amount with its side.
        assert_eq!(data.rows[0].cells.len(), 3);
        assert!(
            text.contains("print-capital-have-title") && text.contains("print-capital-owe-title")
        );
        assert!(
            !text.contains("print-capital-diff-title"),
            "no warning on the worked day"
        );
    }
}

// ---- Real answers for the frontend (plan 87) ----
//
// Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes and
// `<name>.json` the answer of the real service. The test compares the files with the real answers and
// FAILS if they differ. After a deliberate change write them again with
// `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test plan87_capital_card fixtures`, read the
// diff and tell `frontend`.

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn fixture_folder() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn write_or_check(file: &str, text: &str) {
    let path = fixture_folder().join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::create_dir_all(fixture_folder()).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the real answer any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

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

/// A refusal as a command returns it. `ref` is a stand-in: the real one is a counter.
fn wire(error: &ServiceError) -> serde_json::Value {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    serde_json::json!({
        "ok": false,
        "errors": items
            .iter()
            .map(|i| serde_json::json!({ "code": i.code, "field": i.field, "values": i.values }))
            .collect::<Vec<_>>(),
        "ref": "E-20261120-0001"
    })
}

#[test]
fn fixtures_of_the_card_and_the_fix() {
    let u = umar_closed();
    let ctx = &u.w.env.ctx;
    // The card before the fix: cash short, both blocks, the warning with its two groups.
    pair(
        "cashbook_capital_where_unplaced",
        &serde_json::json!({}),
        &cashbook::capital_where(ctx).unwrap(),
    );
    // The print of the same card (with the warning), in both languages.
    for language in ["ur", "en"] {
        let request = serde_json::json!({ "language": language });
        let sheet: mandi_services::core::cashbook::CapitalPrintInput =
            serde_json::from_value(request.clone()).unwrap();
        let card = cashbook::capital_where_at(
            ctx,
            &CapitalWhereInput {
                as_of: sheet.as_of.clone(),
            },
        )
        .unwrap();
        pair(
            &format!("cashbook_capital_print_data_{language}"),
            &request,
            &capital_card_print_data(ctx.db(), sheet.language, sheet.paper, &card).unwrap(),
        );
    }
    // The same card as of a day.
    pair(
        "cashbook_capital_where_as_of",
        &serde_json::json!({ "as_of": "10-10-2026" }),
        &cashbook::capital_where_at(
            ctx,
            &CapitalWhereInput {
                as_of: Some("10-10-2026".into()),
            },
        )
        .unwrap(),
    );
    // A wrong sign: nothing changes.
    let wrong = serde_json::json!({
        "partner_ids": [u.adnan, u.usman],
        "reason": "سرمایہ روکڑ میں درج کریں",
        "sign": "munshi",
    });
    let error = capital_place::place(
        ctx,
        &u.w.owner,
        &serde_json::from_value(wrong.clone()).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrSignWrong);
    pair(
        "partners_place_opening_capital_sign_wrong",
        &wrong,
        &wire(&error),
    );
    // The fix and the card after it.
    let right = serde_json::json!({
        "partner_ids": [u.adnan, u.usman],
        "reason": "سرمایہ روکڑ میں درج کریں",
        "sign": "adnan",
    });
    pair(
        "partners_place_opening_capital",
        &right,
        &capital_place::place(
            ctx,
            &u.w.owner,
            &serde_json::from_value(right.clone()).unwrap(),
        )
        .unwrap(),
    );
    pair(
        "cashbook_capital_where_placed",
        &serde_json::json!({}),
        &cashbook::capital_where(ctx).unwrap(),
    );
    // Nothing left to place.
    let again = capital_place::place(
        ctx,
        &u.w.owner,
        &serde_json::from_value(right.clone()).unwrap(),
    )
    .unwrap_err();
    assert_eq!(again.code(), ErrorCode::ErrNothingToPlace);
    pair(
        "partners_place_opening_capital_nothing",
        &right,
        &wire(&again),
    );
}
