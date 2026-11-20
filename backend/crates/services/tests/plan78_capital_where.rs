//! The card "where the capital is" (plan 78 E, PRD FR-16.14 v1.43, v1.50): where the partners' money
//! is now and whose it is. The two sides are always equal. The full cases of plan 87 (the two blocks,
//! the warning, the fix) are in `plan87_capital_card.rs`.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::cashbook;
use mandi_services::core::cashbook::{
    CapitalSide, CapitalWhere, CapitalWhereKind, CapitalWhoseKind,
};
use reports_world::*;

fn line(card: &CapitalWhere, kind: CapitalWhereKind) -> Option<(CapitalSide, i64)> {
    card.have_lines
        .iter()
        .chain(&card.owe_lines)
        .find(|l| l.kind == kind)
        .map(|l| (l.side, l.amount_paisa.rupees()))
}

fn whose(card: &CapitalWhere, kind: CapitalWhoseKind) -> Vec<i64> {
    card.whose_lines
        .iter()
        .filter(|l| l.kind == kind)
        .map(|l| l.amount_paisa.rupees())
        .collect()
}

#[test]
fn after_events_1_to_5_the_card_is_58484_plus_21564_less_10160_and_4400() {
    let w = full_day();
    let card = cashbook::capital_where(&w.env.ctx).unwrap();
    assert_eq!(
        line(&card, CapitalWhereKind::Cash),
        Some((CapitalSide::Have, 58_484))
    );
    assert_eq!(
        line(&card, CapitalWhereKind::BuyersOwe),
        Some((CapitalSide::Have, 21_564))
    );
    assert_eq!(
        line(&card, CapitalWhereKind::SellersPayable),
        Some((CapitalSide::Owe, 10_160))
    );
    let charges: i64 = card
        .owe_lines
        .iter()
        .filter(|l| l.kind == CapitalWhereKind::OwedForCharges)
        .map(|l| l.amount_paisa.rupees())
        .sum();
    assert_eq!(charges, 4_400);
    // Nothing else is on the card: no other line, no advance, no khata, no garden.
    for kind in [
        CapitalWhereKind::SellerAdvances,
        CapitalWhereKind::SellersOweBeyond,
        CapitalWhereKind::MoneyLent,
        CapitalWhereKind::SeparateKhatas,
        CapitalWhereKind::GardensHeld,
        CapitalWhereKind::LinkedInPartnerTotals,
        CapitalWhereKind::BuyerCredits,
        CapitalWhereKind::Other,
    ] {
        assert_eq!(line(&card, kind), None, "{kind:?}");
    }
    assert_eq!(card.net_paisa.rupees(), 65_488);
    // Whose it is: the partners (−1,004, 746, 746), ابتدائی سرمایہ 65,000, no advance open.
    assert_eq!(
        whose(&card, CapitalWhoseKind::PartnerTotal),
        vec![-1_004, 746, 746]
    );
    assert_eq!(whose(&card, CapitalWhoseKind::OpeningCapital), vec![65_000]);
    assert_eq!(whose(&card, CapitalWhoseKind::AdvancesOpen), vec![0]);
    assert!(whose(&card, CapitalWhoseKind::Other).is_empty());
    assert!(card.difference.is_none());
    assert_eq!(card.whose_total_paisa.rupees(), 65_488);
    assert_eq!(card.net_paisa, card.whose_total_paisa);
}

#[test]
fn an_open_advance_is_with_the_seller_and_in_the_partners_open_parts() {
    // E2E-28: Event 1 with the advance tick off. رحیم's payable is 24,316 - 5,000 = 19,316 and his
    // advance of 15,000 is still open; his ledger is 4,316 جمع.
    let w = worked_day();
    let mut draft = support::event1(&w);
    for lot in &mut draft.lots {
        lot.adjust_advance = false;
    }
    save(&w, &draft);
    let card = cashbook::capital_where(&w.env.ctx).unwrap();
    assert_eq!(
        line(&card, CapitalWhereKind::SellerAdvances),
        Some((CapitalSide::Have, 15_000))
    );
    assert_eq!(
        line(&card, CapitalWhereKind::SellersPayable),
        Some((CapitalSide::Owe, 19_316))
    );
    assert_eq!(line(&card, CapitalWhereKind::SellersOweBeyond), None);
    assert_eq!(whose(&card, CapitalWhoseKind::AdvancesOpen), vec![15_000]);
    assert!(whose(&card, CapitalWhoseKind::Other).is_empty());
    assert_eq!(line(&card, CapitalWhereKind::Other), None);
    assert_eq!(card.net_paisa, card.whose_total_paisa);
}
