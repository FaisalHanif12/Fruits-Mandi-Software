//! Plan 97 item 1 (PRD FR-16.7, FR-16.11 v1.59, C1, C27, E68): fruit taken from another commission
//! agent's shop, with the typed کل کٹوتی. The expected numbers are the PRD's and the money agent's.
//!
//! الف takes 2 پیٹی at 1,400 = 2,800 from the shop of حاجی نذیر with a deduction of 200: the net is
//! 2,600. الف's 0604 account is debited 2,600, the agent is credited 2,600, and الف's total goes
//! down by 2,600. For the shop's guests the same net 2,600 goes over the three partners by C27:
//! floors 866 each, and the 2 leftover rupees all go to the partner created first, 868 / 866 / 866.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::raw;
use mandi_services::core::accounts::{AccountScope, BalanceSide, SearchAccountsInput};
use mandi_services::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_services::core::fruit::{
    AgentShopInput, FruitCancelInput, FruitEditInput, FruitGetInput, FruitInput, FruitListInput,
    FruitPreview, FruitSource,
};
use mandi_services::core::statement::{PartnersStatementInput, StatementDetail};
use mandi_services::{fruit, ledger, partners, search};
use reports_world::worked_day;
use serde::Serialize;
use serde_json::{Value, json};

/// A new shop of another agent, added on the spot (PRD FR-7.69).
fn add_shop(w: &support::World, ur: &str, en: &str) -> i64 {
    fruit::agent_shop_add(
        &w.env.ctx,
        &w.owner,
        &AgentShopInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: String::new(),
            confirmed_rules_words: ur
                .split(' ')
                .chain(en.split(' '))
                .map(str::to_owned)
                .collect(),
        },
    )
    .unwrap()
    .id
}

fn nazeer(w: &support::World) -> i64 {
    add_shop(w, "حاجی نذیر", "Haji Nazeer")
}

/// Fruit from another shop: 2 پیٹی at 1,400, for الف or for the guests.
fn input(w: &support::World, agent: i64, guests: bool, deduction: Option<&str>) -> FruitInput {
    FruitInput {
        for_guests: guests,
        partner_id: (!guests).then_some(w.partners[0]),
        date: "10-10-2026".into(),
        source: FruitSource::OtherShop,
        lot_id: None,
        agent_account_id: Some(agent),
        item_id: Some(w.apple),
        packing_id: Some(w.box_),
        qty: "2".into(),
        weight: None,
        rate: "1,400".into(),
        deduction: deduction.map(str::to_owned),
        note: None,
        editing_id: None,
    }
}

fn totals(w: &support::World) -> Vec<i64> {
    partners::overview(&w.env.ctx)
        .unwrap()
        .into_iter()
        .map(|r| r.total_paisa.rupees())
        .collect()
}

fn balance(w: &support::World, account: i64) -> (BalanceSide, i64) {
    let b = ledger::account_balance(&w.env.ctx, account).unwrap();
    (b.side, b.amount_paisa.rupees())
}

fn codes(errors: &[FieldError]) -> Vec<(ErrorCode, String)> {
    errors.iter().map(|e| (e.code, e.field.clone())).collect()
}

/// The posted lines of the journal entry of a fruit entry, as (account id, debit, credit) in paisa.
fn journal_of(w: &support::World, entry_id: i64) -> Vec<(i64, i64, i64)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT account_id, dr_paisa, cr_paisa FROM journal_lines WHERE entry_id = ?1
             ORDER BY line_no",
        )
        .unwrap();
    statement
        .query_map([entry_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn preview(w: &support::World, input: &FruitInput) -> FruitPreview {
    fruit::preview(&w.env.ctx, input).unwrap()
}

/// The money agent's table: 2 × 1,400 = 2,800, deduction 200, net 2,600. The preview, the saved row,
/// the journal and the partner's total all say 2,600 (Code Standards B5).
#[test]
fn a_deduction_is_subtracted_and_the_partner_and_the_agent_get_the_net() {
    let w = worked_day();
    let agent = nazeer(&w);
    let before = totals(&w);
    let typed = input(&w, agent, false, Some("200"));
    let shown = preview(&w, &typed);
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    let n = shown.numbers.as_ref().unwrap();
    assert_eq!(
        (
            n.value_paisa.rupees(),
            n.deduction_paisa.rupees(),
            n.net_paisa.rupees()
        ),
        (2_800, 200, 2_600)
    );
    assert_eq!(
        n.net_working.as_ref().unwrap().message_id,
        "working-fruit-net"
    );
    let parts: Vec<i64> = n.parts.iter().map(|p| p.amount_paisa.rupees()).collect();
    assert_eq!(parts, [2_600], "the part is the net");
    assert_eq!(n.parts[0].total_after_paisa.rupees(), before[0] - 2_600);

    let saved = fruit::save(&w.env.ctx, &w.owner, &typed).unwrap();
    assert_eq!(saved.preview, shown, "the save is the preview");
    // The journal: الف's 0604 debit 2,600 and the agent's credit 2,600, nothing else.
    let lines = journal_of(&w, saved.journal_entry_id);
    assert_eq!(lines.len(), 2);
    let credit: i64 = lines.iter().map(|l| l.2).sum();
    let debit: i64 = lines.iter().map(|l| l.1).sum();
    assert_eq!((debit, credit), (260_000, 260_000));
    assert!(lines.contains(&(agent, 0, 260_000)));
    assert_eq!(balance(&w, agent), (BalanceSide::Cr, 2_600));
    // Σ preview parts == the net == the stored value == the journal credit.
    let row = fruit::get(&w.env.ctx, &FruitGetInput { id: saved.id }).unwrap();
    assert_eq!(
        row.value_paisa.rupees(),
        2_600,
        "the stored value is the net"
    );
    assert_eq!(
        (row.deduction_paisa.rupees(), row.gross_paisa.rupees()),
        (200, 2_800)
    );
    assert_eq!(row.value_paisa, n.net_paisa);
    assert_eq!(
        row.parts
            .iter()
            .map(|p| p.amount_paisa.rupees())
            .sum::<i64>(),
        2_600
    );
    // الف's total went down by 2,600 and nobody else's moved.
    let after = totals(&w);
    assert_eq!(after[0], before[0] - 2_600);
    assert_eq!(&after[1..], &before[1..]);
    // The books still balance.
    let (dr, cr) = trial(&w);
    assert_eq!(dr, cr);
}

fn trial(w: &support::World) -> (i64, i64) {
    let t = ledger::trial_balance(&w.env.ctx, None).unwrap();
    (t.total_dr.rupees(), t.total_cr.rupees())
}

/// The guests of three partners are charged the net by C27: 868 / 866 / 866 (E68), not 867 / 867 / 866.
#[test]
fn the_guests_share_the_net_by_c27() {
    let w = worked_day();
    let agent = nazeer(&w);
    let before = totals(&w);
    let typed = input(&w, agent, true, Some("200"));
    let shown = preview(&w, &typed);
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    let n = shown.numbers.as_ref().unwrap();
    let parts: Vec<i64> = n.parts.iter().map(|p| p.amount_paisa.rupees()).collect();
    assert_eq!(parts, [868, 866, 866]);
    assert_eq!(parts.iter().sum::<i64>(), n.net_paisa.rupees());
    let saved = fruit::save(&w.env.ctx, &w.owner, &typed).unwrap();
    let lines = journal_of(&w, saved.journal_entry_id);
    assert_eq!(lines.len(), 4);
    assert!(lines.contains(&(agent, 0, 260_000)));
    assert_eq!(balance(&w, agent), (BalanceSide::Cr, 2_600));
    let after = totals(&w);
    assert_eq!(
        after
            .iter()
            .zip(&before)
            .map(|(a, b)| b - a)
            .collect::<Vec<_>>(),
        [868, 866, 866]
    );
}

/// A deduction of 0, and no deduction at all, post exactly as fruit did before the deduction existed.
#[test]
fn no_deduction_and_a_deduction_of_zero_post_exactly_as_before() {
    let w = worked_day();
    let agent = nazeer(&w);
    let none = fruit::save(&w.env.ctx, &w.owner, &input(&w, agent, false, None)).unwrap();
    let zero = fruit::save(&w.env.ctx, &w.owner, &input(&w, agent, false, Some("0"))).unwrap();
    let empty = fruit::save(&w.env.ctx, &w.owner, &input(&w, agent, false, Some(" "))).unwrap();
    for saved in [&none, &zero, &empty] {
        let n = saved.preview.numbers.as_ref().unwrap();
        assert_eq!(
            (
                n.value_paisa.rupees(),
                n.deduction_paisa.rupees(),
                n.net_paisa.rupees()
            ),
            (2_800, 0, 2_800)
        );
        assert!(n.net_working.is_none());
        let lines = journal_of(&w, saved.journal_entry_id);
        assert!(lines.contains(&(agent, 0, 280_000)));
        let row = fruit::get(&w.env.ctx, &FruitGetInput { id: saved.id }).unwrap();
        assert_eq!(
            (row.value_paisa.rupees(), row.deduction_paisa.rupees()),
            (2_800, 0)
        );
        assert_eq!(row.gross_paisa, row.value_paisa);
    }
    assert_eq!(balance(&w, agent), (BalanceSide::Cr, 8_400));
}

/// The deduction must be less than the value: 2,800 is refused with the most (2,799) and the value.
#[test]
fn a_deduction_not_below_the_value_is_refused_at_its_field() {
    let w = worked_day();
    let agent = nazeer(&w);
    for typed in ["2,800", "3,000"] {
        let shown = preview(&w, &input(&w, agent, false, Some(typed)));
        assert!(shown.numbers.is_none());
        assert_eq!(
            codes(&shown.errors),
            [(ErrorCode::ErrDeductionNotBelowValue, "deduction".to_owned())]
        );
        assert_eq!(
            shown.errors[0].values.get("max"),
            Some(&ErrorValue::Number(2_799))
        );
        assert_eq!(
            shown.errors[0].values.get("value"),
            Some(&ErrorValue::Number(2_800))
        );
        // The save refuses it too, and writes nothing.
        let error =
            fruit::save(&w.env.ctx, &w.owner, &input(&w, agent, false, Some(typed))).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ErrDeductionNotBelowValue);
    }
    assert_eq!(balance(&w, agent), (BalanceSide::Zero, 0));
    // 2,799 is allowed: one rupee is left to credit.
    let shown = preview(&w, &input(&w, agent, false, Some("2,799")));
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    assert_eq!(shown.numbers.unwrap().net_paisa.rupees(), 1);
}

/// A bad amount is the usual money error at the field; a deduction on fruit from our own shop is not
/// allowed, but 0 there is fine (a type switch may carry the empty field).
#[test]
fn bad_amounts_and_a_deduction_on_our_own_shops_fruit_are_refused() {
    let w = worked_day();
    let agent = nazeer(&w);
    for typed in ["200.50", "abc", "-100"] {
        let shown = preview(&w, &input(&w, agent, false, Some(typed)));
        assert!(
            shown.errors.iter().any(|e| e.field == "deduction"),
            "{typed}: {:?}",
            shown.errors
        );
        assert!(shown.numbers.is_none());
    }
    // A bad deduction is told even when the rate is wrong too.
    let mut bad = input(&w, agent, false, Some("abc"));
    bad.rate = "x".into();
    let fields: Vec<String> = preview(&w, &bad)
        .errors
        .into_iter()
        .map(|e| e.field)
        .collect();
    assert!(fields.contains(&"deduction".to_owned()) && fields.contains(&"rate".to_owned()));
    // Our own shop (a lot): refused above 0, accepted at 0 or empty. The lot does not exist here, so
    // only the deduction's own error is looked at.
    let mut own = input(&w, agent, false, Some("100"));
    own.source = FruitSource::Shop;
    own.agent_account_id = None;
    own.lot_id = Some(1);
    let refused = preview(&w, &own);
    assert!(
        codes(&refused.errors).contains(&(ErrorCode::ErrFormatInvalid, "deduction".to_owned())),
        "{:?}",
        refused.errors
    );
    own.deduction = Some("0".into());
    let fine = preview(&w, &own);
    assert!(fine.errors.iter().all(|e| e.field != "deduction"));
}

/// Edit by reversal: the deduction changes from 200 to 300, so the net is 2,500; the old posting is
/// reversed exactly (2,600) and the new one is posted. A cancel gives back the net.
#[test]
fn an_edit_and_a_cancel_move_exactly_the_net() {
    let w = worked_day();
    let agent = nazeer(&w);
    let before = totals(&w);
    let saved = fruit::save(&w.env.ctx, &w.owner, &input(&w, agent, false, Some("200"))).unwrap();
    assert_eq!(totals(&w)[0], before[0] - 2_600);
    let mut changed = input(&w, agent, false, Some("300"));
    changed.editing_id = Some(saved.id);
    let shown = preview(&w, &changed);
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    let n = shown.numbers.as_ref().unwrap();
    assert_eq!(n.net_paisa.rupees(), 2_500);
    // The old part is given back first: his total before is what it was before the entry.
    assert_eq!(n.parts[0].total_before_paisa.rupees(), before[0]);
    assert_eq!(n.parts[0].total_after_paisa.rupees(), before[0] - 2_500);
    let row = fruit::get(&w.env.ctx, &FruitGetInput { id: saved.id }).unwrap();
    let edited = fruit::edit(
        &w.env.ctx,
        &w.owner,
        &FruitEditInput {
            id: saved.id,
            version: row.version,
            entry: changed,
            reason: "deduction was 300".into(),
        },
    )
    .unwrap();
    assert_eq!(edited.preview, shown);
    assert_eq!(totals(&w)[0], before[0] - 2_500);
    assert_eq!(balance(&w, agent), (BalanceSide::Cr, 2_500));
    let row = fruit::get(&w.env.ctx, &FruitGetInput { id: saved.id }).unwrap();
    assert_eq!(
        (
            row.value_paisa.rupees(),
            row.deduction_paisa.rupees(),
            row.gross_paisa.rupees()
        ),
        (2_500, 300, 2_800)
    );
    // The audit row of the edit holds the old and the new deduction (PRD FR-7.71).
    let audit = raw(&w.dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE entity = 'fruit_take' AND action LIKE '%edit%'
             ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .unwrap();
    assert!(audit.0.contains("\"deduction_paisa\":20000"), "{audit:?}");
    assert!(audit.1.contains("\"deduction_paisa\":30000"), "{audit:?}");
    fruit::cancel(
        &w.env.ctx,
        &w.owner,
        &FruitCancelInput {
            id: saved.id,
            version: row.version,
            reason: "mistake".into(),
        },
    )
    .unwrap();
    assert_eq!(totals(&w), before);
    assert_eq!(balance(&w, agent), (BalanceSide::Zero, 0));
}

/// The partner's statement says "2 @ 1,400 = 2,800" and gives the deduction; its amount is the net.
#[test]
fn the_statement_line_adds_up_and_its_amount_is_the_net() {
    let w = worked_day();
    let agent = nazeer(&w);
    let saved = fruit::save(&w.env.ctx, &w.owner, &input(&w, agent, false, Some("200"))).unwrap();
    let statement = partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            group_by_month: false,
            sort: None,
            partner_id: w.partners[0],
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let line = statement
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::FruitTaken && l.number == Some(saved.number))
        .unwrap();
    assert_eq!(line.amount_paisa.unwrap().rupees(), -2_600);
    let value = |name: &str| line.values.get(name).cloned().unwrap();
    assert_eq!(value("qty"), ErrorValue::Number(2));
    assert_eq!(value("rate_paisa"), ErrorValue::Number(140_000));
    assert_eq!(value("value_paisa"), ErrorValue::Number(280_000));
    assert_eq!(value("deduction_paisa"), ErrorValue::Number(20_000));
}

/// A new shop is a seller account marked آڑھتی دکان. The آڑھتی box lists it (and only marked ones),
/// the seller field of a lot does not, and every other seller list still does (PRD FR-16.11 v1.59).
#[test]
fn a_new_shop_is_listed_by_the_agent_box_and_hidden_from_the_lot_seller_field() {
    let w = worked_day();
    let shop = nazeer(&w);
    let row = mandi_services::accounts::get(&w.env.ctx, shop).unwrap();
    assert!(row.is_agent_shop);
    assert_eq!(
        row.account_type,
        mandi_services::core::accounts::AccountType::Seller
    );
    assert!(row.code.starts_with("001"), "a seller code: {}", row.code);
    // رحیم is an ordinary seller: not a shop.
    assert!(
        !mandi_services::accounts::get(&w.env.ctx, w.rahim)
            .unwrap()
            .is_agent_shop
    );
    let ids = |scope: AccountScope, text: &str| -> Vec<i64> {
        search::search_accounts(
            &w.env.ctx,
            &w.owner,
            &SearchAccountsInput {
                scope,
                control_id: None,
                text: text.into(),
                limit: 50,
                offset: 0,
                include_dead: None,
                with_chukta: None,
            },
        )
        .unwrap()
        .into_iter()
        .map(|h| h.id)
        .collect()
    };
    // An empty text gives the whole list of the field.
    assert_eq!(ids(AccountScope::AgentShop, ""), [shop]);
    assert_eq!(ids(AccountScope::AgentShop, "nazeer"), [shop]);
    assert!(ids(AccountScope::AgentShop, "rahim").is_empty());
    let sellers = ids(AccountScope::Seller, "");
    assert!(sellers.contains(&shop) && sellers.contains(&w.rahim));
    let lot_sellers = ids(AccountScope::LotSeller, "");
    assert!(lot_sellers.contains(&w.rahim) && !lot_sellers.contains(&shop));
    // A short code finds the shop in its own box, by the running number of a seller's code.
    let by_name = ids(AccountScope::AgentShop, "حاجی");
    assert_eq!(by_name, [shop]);
}

/// The new shop has to be a valid account: both names are checked like any seller's.
#[test]
fn a_shop_needs_both_names_in_their_own_script() {
    let w = worked_day();
    let error = fruit::agent_shop_add(
        &w.env.ctx,
        &w.owner,
        &AgentShopInput {
            name_ur: "Haji".into(),
            name_en: "Haji Nazeer".into(),
            phone: String::new(),
            confirmed_rules_words: Vec::new(),
        },
    )
    .unwrap_err();
    let mandi_services::ServiceError::Rejected(items) = error else {
        panic!("not a refusal");
    };
    assert!(items.iter().any(|i| i.field.as_deref() == Some("name_ur")));
    // Nothing was made.
    let count: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM accounts WHERE is_agent_shop = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

/// The list totals add up to what was posted: the net, with the deduction in its own column.
#[test]
fn the_list_and_its_totals_hold_the_net_and_the_deduction() {
    let w = worked_day();
    let agent = nazeer(&w);
    fruit::save(&w.env.ctx, &w.owner, &input(&w, agent, false, Some("200"))).unwrap();
    fruit::save(&w.env.ctx, &w.owner, &input(&w, agent, true, None)).unwrap();
    let list = fruit::list(&w.env.ctx, &FruitListInput::default()).unwrap();
    assert_eq!(list.rows.len(), 2);
    let posted: i64 = list.rows.iter().map(|r| r.value_paisa.rupees()).sum();
    assert_eq!(posted, 2_600 + 2_800);
    assert_eq!(list.totals.value_paisa.rupees(), posted);
    // The deduction has its own total, from the backend: gross = net + deduction.
    assert_eq!(list.totals.deduction_paisa.rupees(), 200);
    assert_eq!(list.totals.gross_paisa.rupees(), posted + 200);
    assert_eq!(
        list.rows
            .iter()
            .map(|r| r.gross_paisa.rupees())
            .sum::<i64>(),
        posted + 200
    );
}

/// A deduction field holding only an invisible mark (a pasted zero-width character) means 0.
#[test]
fn an_invisible_mark_in_the_deduction_field_means_zero() {
    let w = worked_day();
    let agent = nazeer(&w);
    let shown = preview(&w, &input(&w, agent, false, Some("\u{200B}")));
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    assert_eq!(shown.numbers.unwrap().deduction_paisa.rupees(), 0);
}

// ---- Real answers for the screen (fixtures) -------------------------------------------------------

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// A fixture is a pair: `<name>_request.json` is the `input` of the command, `<name>.json` the real
/// answer. The test fails if a file differs; after a deliberate change write them again with
/// `MANDI_WRITE_FIXTURES=1`, read the diff and tell `frontend`.
fn write_or_check(file: &str, text: &str) {
    let path = folder().join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::create_dir_all(folder()).unwrap();
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

fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

/// The `input` of `fruit_preview` and `fruit_save` as the screen sends it.
fn fruit_json(w: &support::World, agent: i64, guests: bool, deduction: Option<&str>) -> Value {
    json!({
        "input": {
            "for_guests": guests,
            "partner_id": (!guests).then_some(w.partners[0]),
            "date": "10-10-2026",
            "source": "other_shop",
            "lot_id": null,
            "agent_account_id": agent,
            "item_id": w.apple,
            "packing_id": w.box_,
            "qty": "2",
            "weight": null,
            "rate": "1,400",
            "deduction": deduction,
            "note": null,
            "editing_id": null
        }
    })
}

/// Fruit from the shop of another agent with a کل کٹوتی, as the screen sees it: the shop added on the
/// spot, the box that lists it, the preview, the refusal, the save, the row, the list and the partner's
/// statement line. 2 × 1,400 = 2,800, کل کٹوتی 200, net 2,600.
#[test]
fn the_real_answers_of_fruit_from_another_shop_with_a_deduction_are_the_fixtures() {
    let w = worked_day();
    let ctx = &w.env.ctx;
    let shop = json!({ "input": {
        "name_ur": "حاجی نذیر", "name_en": "Haji Nazeer", "phone": "", "confirmed_rules_words": []
    }});
    let added = fruit::agent_shop_add(
        ctx,
        &w.owner,
        &AgentShopInput {
            name_ur: "حاجی نذیر".into(),
            name_en: "Haji Nazeer".into(),
            phone: String::new(),
            confirmed_rules_words: ["حاجی", "نذیر", "Haji", "Nazeer"]
                .map(str::to_owned)
                .to_vec(),
        },
    )
    .unwrap();
    pair("fruit_agent_shop_add", &shop, &added);
    let search_request = json!({ "input": {
        "scope": "agent_shop", "control_id": null, "text": "", "limit": 50, "offset": 0
    }});
    let hits = search::search_accounts(
        ctx,
        &w.owner,
        &SearchAccountsInput {
            scope: AccountScope::AgentShop,
            control_id: None,
            text: String::new(),
            limit: 50,
            offset: 0,
            include_dead: None,
            with_chukta: None,
        },
    )
    .unwrap();
    assert_eq!(hits.len(), 1);
    pair("search_accounts_agent_shop", &search_request, &hits);

    let agent = added.id;
    let request = fruit_json(&w, agent, false, Some("200"));
    let preview = fruit::preview(ctx, &input(&w, agent, false, Some("200"))).unwrap();
    pair("fruit_preview_other_shop_deduction", &request, &preview);
    let refused = fruit::preview(ctx, &input(&w, agent, false, Some("2,800"))).unwrap();
    pair(
        "fruit_preview_deduction_not_below_value",
        &fruit_json(&w, agent, false, Some("2,800")),
        &refused,
    );
    let saved = fruit::save(ctx, &w.owner, &input(&w, agent, false, Some("200"))).unwrap();
    pair("fruit_save_other_shop_deduction", &request, &saved);
    let row = fruit::get(ctx, &FruitGetInput { id: saved.id }).unwrap();
    pair(
        "fruit_get_other_shop_deduction",
        &json!({ "input": { "id": saved.id } }),
        &row,
    );
    let guests = fruit::preview(ctx, &input(&w, agent, true, Some("200"))).unwrap();
    pair(
        "fruit_preview_guests_deduction",
        &fruit_json(&w, agent, true, Some("200")),
        &guests,
    );
    let list = fruit::list(ctx, &FruitListInput::default()).unwrap();
    pair(
        "fruit_list_other_shop_deduction",
        &json!({ "input": {} }),
        &list,
    );
    let statement_request = json!({ "input": {
        "group_by_month": false, "sort": null, "partner_id": w.partners[0], "from": null,
        "to": null, "offset": 0, "limit": 0
    }});
    let statement = partners::statement(
        ctx,
        &PartnersStatementInput {
            group_by_month: false,
            sort: None,
            partner_id: w.partners[0],
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    pair(
        "partners_statement_fruit_deduction",
        &statement_request,
        &statement,
    );
}
