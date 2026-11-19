//! The combined balance of a seller and a buyer account linked by `accounts.linked_account_id`
//! (PRD E10, FR-10.2, gate finding T4-2, plan 80). Read only: the party card and the R13 and R14
//! headings show it, nothing is posted, and every figure is worked out in the backend from both
//! ledgers.
//!
//! The numbers are the PRD worked day (Events 1 to 5): اکرم (buyer 0020) owes 2,880 بنام and کریم
//! (seller 0011) is owed 10,160 جمع. Linked, they give 10,160 − 2,880 = 7,280 جمع.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::reports::{Cell, ReportId};
use mandi_services::core::accounts::{Balance, BalanceSide, LinkedBalance, PartyCard};
use mandi_services::{accounts, assist};
use reports_world::*;
use support::World;

fn rupees_of(balance: Balance) -> (BalanceSide, i64) {
    (balance.side, balance.amount_paisa.rupees())
}

/// Links the account of `from` to the account of `to`, the way the account form stores it.
fn link(w: &World, from: i64, to: i64) {
    let conn = common::raw(&w.dir);
    conn.execute(
        "UPDATE accounts SET linked_account_id = ?1 WHERE id = ?2",
        [to, from],
    )
    .unwrap();
}

fn card(w: &World, id: i64) -> PartyCard {
    assist::party_card(&w.env.ctx, id).unwrap()
}

fn linked_of(card: &PartyCard) -> &LinkedBalance {
    card.linked.as_ref().expect("the card shows the link")
}

/// The summary line "both accounts together" of a ledger page, if it has one.
fn combined_line(w: &World, id: ReportId, account: i64) -> Option<(BalanceSide, i64)> {
    let page = report(w, id, party(account), 0, 0);
    page.summary
        .iter()
        .find(|s| s.label_id == "report-linked-combined")
        .map(|s| balance(&s.cell))
}

#[test]
fn an_account_with_no_link_shows_no_combined_balance() {
    let w = full_day();
    for id in [w.akram, w.karim, w.rahim] {
        assert!(card(&w, id).linked.is_none());
    }
    // Neither does a ledger: its summary is the opening and the closing balance only.
    for id in [ReportId::R13, ReportId::R14] {
        assert_eq!(combined_line(&w, id, w.akram), None);
        let page = report(&w, id, party(w.akram), 0, 0);
        assert!(
            page.summary
                .iter()
                .all(|s| s.label_id != "report-linked-combined")
        );
    }
}

#[test]
fn a_linked_seller_and_buyer_show_both_balances_and_the_two_added_together() {
    let w = full_day();
    assert_eq!(
        rupees_of(card(&w, w.akram).balance),
        (BalanceSide::Dr, 2_880)
    );
    assert_eq!(
        rupees_of(card(&w, w.karim).balance),
        (BalanceSide::Cr, 10_160)
    );
    link(&w, w.akram, w.karim);

    // The buyer's card: his own balance and the seller account of the same person.
    let buyer_card = card(&w, w.akram);
    assert_eq!(
        rupees_of(buyer_card.balance),
        (BalanceSide::Dr, 2_880),
        "his own balance is not changed by the link"
    );
    let other = linked_of(&buyer_card);
    assert_eq!(other.account_id, w.karim);
    assert_eq!(other.code, "0011");
    assert_eq!(other.name_en, "Karim");
    assert!(other.is_active);
    assert_eq!(rupees_of(other.balance), (BalanceSide::Cr, 10_160));
    assert_eq!(
        rupees_of(other.combined),
        (BalanceSide::Cr, 7_280),
        "10,160 جمع less 2,880 بنام"
    );

    // The link is stored on one account, and the other card shows it too, with the same figure.
    let seller_card = card(&w, w.karim);
    assert_eq!(rupees_of(seller_card.balance), (BalanceSide::Cr, 10_160));
    let other = linked_of(&seller_card);
    assert_eq!((other.account_id, other.code.as_str()), (w.akram, "0020"));
    assert_eq!(rupees_of(other.balance), (BalanceSide::Dr, 2_880));
    assert_eq!(rupees_of(other.combined), (BalanceSide::Cr, 7_280));

    // The ledger headings of R13 and R14 carry the same combined figure.
    for id in [ReportId::R13, ReportId::R14] {
        assert_eq!(
            combined_line(&w, id, w.akram),
            Some((BalanceSide::Cr, 7_280)),
            "{id:?} of the buyer"
        );
        assert_eq!(
            combined_line(&w, id, w.karim),
            Some((BalanceSide::Cr, 7_280)),
            "{id:?} of the seller"
        );
    }
    // Reading it posted nothing: the trial balance is still E2E-10.
    let sums = common::raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0) FROM journal_lines",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        )
        .unwrap();
    assert_eq!(sums.0, sums.1);
}

#[test]
fn the_combined_balance_follows_the_ledgers_and_can_cross_zero() {
    let w = full_day();
    link(&w, w.akram, w.karim);
    // He pays the 2,880 he owes: the buyer account is 0, so the combined balance is the seller's.
    mandi_services::vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "2,880"))
        .unwrap();
    let buyer_card = card(&w, w.akram);
    assert_eq!(rupees_of(buyer_card.balance), (BalanceSide::Zero, 0));
    assert_eq!(
        rupees_of(linked_of(&buyer_card).combined),
        (BalanceSide::Cr, 10_160)
    );
    // He pays 3,000 more: his account turns 3,000 جمع and the two together are 13,160 جمع.
    mandi_services::vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "3,000"))
        .unwrap();
    let buyer_card = card(&w, w.akram);
    assert_eq!(rupees_of(buyer_card.balance), (BalanceSide::Cr, 3_000));
    assert_eq!(
        rupees_of(linked_of(&buyer_card).combined),
        (BalanceSide::Cr, 13_160)
    );
    assert_eq!(
        combined_line(&w, ReportId::R13, w.akram),
        Some((BalanceSide::Cr, 13_160))
    );
}

/// Two balances that cancel give exactly 0: the side is Zero (never بنام or جمع with 0), on the card
/// and on the ledger heading. اکرم owes 2,880 بنام; کریم is paid 7,280 of his 10,160, so he is
/// owed 2,880 جمع.
#[test]
fn balances_that_cancel_each_other_combine_to_exactly_zero() {
    let w = full_day();
    link(&w, w.akram, w.karim);
    mandi_services::vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.karim, "7,280"))
        .unwrap();
    let buyer_card = card(&w, w.akram);
    assert_eq!(rupees_of(buyer_card.balance), (BalanceSide::Dr, 2_880));
    let other = linked_of(&buyer_card);
    assert_eq!(rupees_of(other.balance), (BalanceSide::Cr, 2_880));
    assert_eq!(rupees_of(other.combined), (BalanceSide::Zero, 0));
    assert_eq!(
        combined_line(&w, ReportId::R13, w.akram),
        Some((BalanceSide::Zero, 0))
    );
    assert_eq!(
        combined_line(&w, ReportId::R14, w.karim),
        Some((BalanceSide::Zero, 0))
    );
}

/// A link to an inactive account still counts: its balance is real. The card says it is inactive.
#[test]
fn a_link_to_an_inactive_account_is_shown_with_its_flag() {
    let w = full_day();
    link(&w, w.akram, w.karim);
    accounts::set_active(&w.env.ctx, &w.owner, w.karim, false).unwrap();
    let buyer_card = card(&w, w.akram);
    let other = linked_of(&buyer_card);
    assert!(!other.is_active);
    assert_eq!(rupees_of(other.balance), (BalanceSide::Cr, 10_160));
    assert_eq!(rupees_of(other.combined), (BalanceSide::Cr, 7_280));
    assert_eq!(
        combined_line(&w, ReportId::R13, w.akram),
        Some((BalanceSide::Cr, 7_280))
    );
}

/// A link to a missing account or to چکتا کھاتہ (no ledger, PRD FR-13.6) shows nothing, and a
/// ledger or card never fails because of a bad link.
#[test]
fn a_link_to_nothing_or_to_chukta_khata_shows_no_combined_balance() {
    let w = full_day();
    link(&w, w.akram, w.chukta_khata);
    assert!(card(&w, w.akram).linked.is_none());
    assert_eq!(combined_line(&w, ReportId::R13, w.akram), None);
    // Foreign keys are off for this write: a row made outside the app can point at nothing.
    let conn = common::raw(&w.dir);
    conn.execute_batch("PRAGMA foreign_keys = OFF").unwrap();
    conn.execute(
        "UPDATE accounts SET linked_account_id = 987654 WHERE id = ?1",
        [w.akram],
    )
    .unwrap();
    assert!(card(&w, w.akram).linked.is_none());
    assert_eq!(combined_line(&w, ReportId::R14, w.akram), None);
}

/// The combined line of a ledger up to a past day uses the other account up to that day too, so the
/// two balances are of one moment.
#[test]
fn the_ledger_up_to_a_day_combines_both_accounts_as_of_that_day() {
    let w = full_day();
    link(&w, w.akram, w.karim);
    // Before 10-10-2026 nothing is posted on either account: both balances and the sum are 0.
    let mut filters = party(w.akram);
    filters.to = "09-10-2026".into();
    let page = report(&w, ReportId::R13, filters, 0, 0);
    let line = page
        .summary
        .iter()
        .find(|s| s.label_id == "report-linked-combined")
        .map(|s| balance(&s.cell));
    assert_eq!(line, Some((BalanceSide::Zero, 0)));
    // Up to the worked day the figure is the card's.
    let mut filters = party(w.akram);
    filters.to = "10-10-2026".into();
    let page = report(&w, ReportId::R13, filters, 0, 0);
    let line = page
        .summary
        .iter()
        .find(|s| s.label_id == "report-linked-combined")
        .map(|s| balance(&s.cell));
    assert_eq!(line, Some((BalanceSide::Cr, 7_280)));
    assert!(matches!(
        page.summary.last().map(|s| &s.cell),
        Some(Cell::Balance { .. })
    ));
}
