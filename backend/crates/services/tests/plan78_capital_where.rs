//! The card "where the capital is" (plan 78 E, PRD FR-16.14 v1.43): where the partners' money is now
//! and whose it is. The two totals are always equal.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::cashbook;
use mandi_services::core::cashbook::{CapitalWhere, CapitalWhereKind, CapitalWhoseKind};
use reports_world::*;

fn line(card: &CapitalWhere, kind: CapitalWhereKind) -> i64 {
    card.where_lines
        .iter()
        .find(|l| l.kind == kind)
        .map_or(0, |l| l.amount_paisa.rupees())
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
    assert_eq!(line(&card, CapitalWhereKind::Cash), 58_484);
    assert_eq!(line(&card, CapitalWhereKind::BuyersOwe), 21_564);
    assert_eq!(line(&card, CapitalWhereKind::SellersPayable), -10_160);
    assert_eq!(line(&card, CapitalWhereKind::OwedForCharges), -4_400);
    for kind in [
        CapitalWhereKind::SellerAdvances,
        CapitalWhereKind::SellersOweBeyond,
        CapitalWhereKind::MoneyLent,
        CapitalWhereKind::SeparateKhatas,
        CapitalWhereKind::GardensHeld,
        CapitalWhereKind::LinkedInPartnerTotals,
        CapitalWhereKind::Other,
    ] {
        assert_eq!(line(&card, kind), 0, "{kind:?}");
    }
    assert_eq!(card.where_total_paisa.rupees(), 65_488);
    // Whose it is: the partners (−1,004, 746, 746), ابتدائی سرمایہ 65,000, no advance open.
    assert_eq!(
        whose(&card, CapitalWhoseKind::PartnerTotal),
        vec![-1_004, 746, 746]
    );
    assert_eq!(whose(&card, CapitalWhoseKind::OpeningCapital), vec![65_000]);
    assert_eq!(whose(&card, CapitalWhoseKind::AdvancesOpen), vec![0]);
    assert!(whose(&card, CapitalWhoseKind::UnrecordedWhere).is_empty());
    assert!(whose(&card, CapitalWhoseKind::Other).is_empty());
    assert_eq!(card.whose_total_paisa.rupees(), 65_488);
    assert_eq!(card.where_total_paisa, card.whose_total_paisa);
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
    assert_eq!(line(&card, CapitalWhereKind::SellerAdvances), 15_000);
    assert_eq!(line(&card, CapitalWhereKind::SellersPayable), -19_316);
    assert_eq!(line(&card, CapitalWhereKind::SellersOweBeyond), 0);
    assert_eq!(whose(&card, CapitalWhoseKind::AdvancesOpen), vec![15_000]);
    assert!(whose(&card, CapitalWhoseKind::Other).is_empty());
    assert!(
        card.where_lines
            .iter()
            .all(|l| l.kind != CapitalWhereKind::Other)
    );
    assert_eq!(card.where_total_paisa, card.whose_total_paisa);
}
