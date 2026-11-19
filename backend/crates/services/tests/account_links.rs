//! The link between the seller account and the buyer account of one person is one clean pair (PRD
//! E10, FR-10.2; the money review's note on link integrity, plan 80).
//!
//! The account form refuses a link that is not a seller-to-buyer pair, a link to an account that is
//! already linked elsewhere (the other account links to a third, or a third points at it), and a
//! chain. A refused save writes nothing. A pair that points both ways is one pair and is allowed,
//! and saving other fields of an account that already has its link must keep working.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::accounts::{AccountInput, AccountRow};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::{ServiceError, accounts, assist};
use support::World;

fn first(error: &ServiceError) -> (ErrorCode, Option<String>) {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    let item = items.first().expect("a refusal has an error");
    (item.code, item.field.clone())
}

/// The form of an existing account as the screen sends it back, with `link` typed as its link.
fn form_of(row: &AccountRow, link: Option<i64>) -> AccountInput {
    AccountInput {
        id: Some(row.id),
        version: Some(row.version),
        account_type: row.account_type,
        control_id: None,
        code: row.code.clone(),
        name_ur: row.name_ur.clone(),
        name_en: row.name_en.clone(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: row.father_or_firm_ur.clone(),
        father_or_firm_en: row.father_or_firm_en.clone(),
        city_id: None,
        adda_id: None,
        phone1: row.phone1.clone(),
        phone2: row.phone2.clone(),
        address: String::new(),
        linked_account_id: link,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: row.credit_allowed,
        credit_limit: row
            .credit_limit_paisa
            .map(|p| p.rupees().to_string())
            .unwrap_or_default(),
        default_marka_id: None,
        collector_staff_id: None,
        is_active: row.is_active,
        note: String::new(),
        tag: row.tag.clone().unwrap_or_default(),
    }
}

/// Saves a form, confirming whatever spelling the check asks about (not the subject here).
fn save(w: &World, mut form: AccountInput) -> Result<AccountRow, ServiceError> {
    loop {
        match accounts::save(&w.env.ctx, &w.owner, &form) {
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(words)) = item.values.get("words") {
                        form.confirmed_rules_words
                            .extend(words.split(' ').map(str::to_owned));
                    }
                }
            }
            other => return other,
        }
    }
}

/// Links `from` to `to` through the account form (`None` clears the link).
fn link(w: &World, from: i64, to: Option<i64>) -> Result<AccountRow, ServiceError> {
    let row = accounts::get(&w.env.ctx, from).unwrap();
    save(w, form_of(&row, to))
}

/// Every account with its link and version, to show that a refusal writes nothing.
fn links(w: &World) -> String {
    common::raw(&w.dir)
        .query_row(
            "SELECT group_concat(id || ':' || COALESCE(linked_account_id, '') || ':' || version, ',')
             FROM (SELECT id, linked_account_id, version FROM accounts ORDER BY id)",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

fn refused(w: &World, from: i64, to: i64, code: ErrorCode) {
    let before = links(w);
    let error = link(w, from, Some(to)).unwrap_err();
    assert_eq!(
        first(&error),
        (code, Some("linked_account_id".to_owned())),
        "{from} -> {to}"
    );
    assert_eq!(links(w), before, "a refused link writes nothing");
}

fn stored_link(w: &World, id: i64) -> Option<i64> {
    common::raw(&w.dir)
        .query_row(
            "SELECT linked_account_id FROM accounts WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn a_seller_and_a_buyer_can_be_linked_and_the_link_can_be_cleared() {
    let w = support::world();
    let saved = link(&w, w.rahim, Some(w.akram)).unwrap();
    assert_eq!(saved.linked_account.as_ref().map(|r| r.id), Some(w.akram));
    assert_eq!(stored_link(&w, w.rahim), Some(w.akram));
    // The party card of either account shows the pair (the link is found from both ends).
    assert!(
        assist::party_card(&w.env.ctx, w.rahim)
            .unwrap()
            .linked
            .is_some()
    );
    assert!(
        assist::party_card(&w.env.ctx, w.akram)
            .unwrap()
            .linked
            .is_some()
    );
    // Clearing it is always fine.
    link(&w, w.rahim, None).unwrap();
    assert_eq!(stored_link(&w, w.rahim), None);
    assert!(
        assist::party_card(&w.env.ctx, w.akram)
            .unwrap()
            .linked
            .is_none()
    );
}

#[test]
fn a_new_buyer_can_be_made_already_linked_to_a_seller_and_the_other_way_round() {
    let w = support::world();
    let row = accounts::get(&w.env.ctx, w.akram).unwrap();
    let mut form = form_of(&row, Some(w.rahim));
    form.id = None;
    form.version = None;
    form.code = String::new();
    form.name_ur = "طارق محمود".into();
    form.name_en = "Tariq Mehmood".into();
    let made = save(&w, form).unwrap();
    assert_eq!(made.linked_account.as_ref().map(|r| r.id), Some(w.rahim));
    // رحیم is now taken: a second buyer cannot link to him.
    let row = accounts::get(&w.env.ctx, w.arshad).unwrap();
    let before = links(&w);
    let error = save(&w, form_of(&row, Some(w.rahim))).unwrap_err();
    assert_eq!(first(&error).0, ErrorCode::ErrLinkAlreadyLinked);
    assert_eq!(links(&w), before);
}

#[test]
fn a_link_that_is_not_a_seller_and_a_buyer_is_refused() {
    let w = support::world();
    // Two sellers, two buyers.
    refused(&w, w.rahim, w.karim, ErrorCode::ErrLinkNotSellerBuyer);
    refused(&w, w.akram, w.arshad, ErrorCode::ErrLinkNotSellerBuyer);
    // چکتا کھاتہ, the cash buyer نقدی منڈی (a system account) and the cash account.
    for target in [w.chukta_khata, w.naqdi, support::id_of(&w.env, "01010001")] {
        refused(&w, w.karim, target, ErrorCode::ErrLinkNotSellerBuyer);
    }
}

#[test]
fn an_account_already_linked_elsewhere_cannot_be_linked_again_in_either_direction() {
    let w = support::world();
    link(&w, w.rahim, Some(w.akram)).unwrap();
    // A third seller to اکرم: a second account points at him.
    refused(&w, w.karim, w.akram, ErrorCode::ErrLinkAlreadyLinked);
    // A second buyer to رحیم: رحیم links to اکرم already (the target links elsewhere).
    refused(&w, w.arshad, w.rahim, ErrorCode::ErrLinkAlreadyLinked);
    // اکرم to another seller, although رحیم already points at him: that would be a chain
    // رحیم, اکرم, کریم.
    refused(&w, w.akram, w.karim, ErrorCode::ErrLinkAlreadyLinked);
    // The pair itself is untouched.
    assert_eq!(stored_link(&w, w.rahim), Some(w.akram));
    assert_eq!(stored_link(&w, w.akram), None);
    assert_eq!(stored_link(&w, w.karim), None);
    assert_eq!(stored_link(&w, w.arshad), None);
}

#[test]
fn a_chain_of_links_is_refused_whichever_link_is_made_last() {
    let w = support::world();
    // رحیم to اکرم, then کریم to ارشد: two separate pairs.
    link(&w, w.rahim, Some(w.akram)).unwrap();
    link(&w, w.karim, Some(w.arshad)).unwrap();
    // Joining the pairs in any way makes a chain of three or four.
    refused(&w, w.arshad, w.rahim, ErrorCode::ErrLinkAlreadyLinked);
    refused(&w, w.akram, w.karim, ErrorCode::ErrLinkAlreadyLinked);
    refused(&w, w.rahim, w.arshad, ErrorCode::ErrLinkAlreadyLinked);
    refused(&w, w.karim, w.akram, ErrorCode::ErrLinkAlreadyLinked);
}

#[test]
fn a_link_can_be_moved_to_another_account_and_a_pair_may_point_both_ways() {
    let w = support::world();
    link(&w, w.rahim, Some(w.akram)).unwrap();
    // Moving رحیم's link to ارشد is not a chain: اکرم is free again.
    link(&w, w.rahim, Some(w.arshad)).unwrap();
    assert_eq!(stored_link(&w, w.rahim), Some(w.arshad));
    assert!(
        assist::party_card(&w.env.ctx, w.akram)
            .unwrap()
            .linked
            .is_none()
    );
    // اکرم can now be linked to کریم.
    link(&w, w.karim, Some(w.akram)).unwrap();
    // A pair that points both ways is one pair.
    link(&w, w.arshad, Some(w.rahim)).unwrap();
    assert_eq!(stored_link(&w, w.rahim), Some(w.arshad));
    assert_eq!(stored_link(&w, w.arshad), Some(w.rahim));
    // The card of each shows the other, once.
    let card = assist::party_card(&w.env.ctx, w.rahim).unwrap();
    assert_eq!(card.linked.as_ref().map(|l| l.account_id), Some(w.arshad));
}

/// Saving the form of an account that already has its link, with another field changed, must keep
/// working: the link it already has is not "linked elsewhere".
#[test]
fn saving_other_fields_of_a_linked_account_keeps_its_link() {
    let w = support::world();
    link(&w, w.rahim, Some(w.akram)).unwrap();
    for id in [w.rahim, w.akram] {
        let row = accounts::get(&w.env.ctx, id).unwrap();
        let mut form = form_of(&row, row.linked_account.as_ref().map(|r| r.id));
        form.phone1 = "0300-1234567".into();
        let saved = save(&w, form).unwrap();
        assert_eq!(saved.phone1, "03001234567");
    }
    assert_eq!(stored_link(&w, w.rahim), Some(w.akram));
}
