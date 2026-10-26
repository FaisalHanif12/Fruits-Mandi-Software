//! Fruit taken (PRD 16.7, FR-16.10 to FR-16.12, C27, E2E-42, plan 17d) on the worked day's world. The
//! expected numbers are the PRD's and are never changed to suit the code.

use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::CancelInput;
use mandi_services::core::error::ErrorValue;
use mandi_services::core::fruit::{
    FruitCancelInput, FruitEditInput, FruitGetInput, FruitInput, FruitListInput, FruitLotsInput,
    FruitSource, FruitStatus,
};
use mandi_services::core::statement::{PartnerStatement, PartnersStatementInput, StatementDetail};
use mandi_services::{ServiceError, fruit, partners};

use super::edit::event1_part_sold;
use super::own_goods::{
    assert_equity_is_the_partner_totals, cash_balance, partner_total, totals_of_three, trial,
};
use super::*;

/// الف takes 2 پیٹی کیلا at 580 from رحیم's lot (E2E-42).
fn home(w: &support::World, qty: &str, rate: &str) -> FruitInput {
    FruitInput {
        for_guests: false,
        partner_id: Some(w.partners[0]),
        date: "10-10-2026".into(),
        source: FruitSource::Shop,
        lot_id: Some(1),
        agent_account_id: None,
        item_id: None,
        packing_id: None,
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        note: None,
        editing_id: None,
    }
}

/// Fruit for the shop's guests from another agent's shop.
fn guests(w: &support::World, agent: i64, qty: &str, rate: &str) -> FruitInput {
    FruitInput {
        for_guests: true,
        partner_id: None,
        date: "10-10-2026".into(),
        source: FruitSource::OtherShop,
        lot_id: None,
        agent_account_id: Some(agent),
        item_id: Some(w.apple),
        packing_id: Some(w.box_),
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        note: None,
        editing_id: None,
    }
}

/// حاجی نذیر, seller 0012.
fn nazeer(w: &support::World) -> i64 {
    support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "حاجی نذیر",
        "Haji Nazeer",
        true,
        "",
    )
}

/// The world after E2E-16: رحیم's lot of 50 کیلا with 45 sold and 5 left.
fn after_e2e_16() -> support::World {
    let w = worked_day();
    save_ok(&w, &event1_part_sold(&w), false);
    w
}

fn code_at(error: ServiceError, code: ErrorCode, field: &str) -> Vec<(ErrorCode, Option<String>)> {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    let found: Vec<(ErrorCode, Option<String>)> =
        items.into_iter().map(|i| (i.code, i.field)).collect();
    assert!(
        found
            .iter()
            .any(|(c, f)| *c == code && f.as_deref() == Some(field)),
        "{code:?} at {field} not in {found:?}"
    );
    found
}

fn preview_codes(w: &support::World, input: &FruitInput) -> Vec<(ErrorCode, Option<String>)> {
    fruit::preview(&w.env.ctx, input)
        .unwrap()
        .errors
        .into_iter()
        .map(|e| (e.code, Some(e.field)))
        .collect()
}

fn statement_of(w: &support::World, partner: i64) -> PartnerStatement {
    partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            partner_id: partner,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
}

fn value_of(line: &mandi_services::core::statement::StatementLine, name: &str) -> ErrorValue {
    line.values
        .get(name)
        .cloned()
        .unwrap_or_else(|| panic!("no value {name}"))
}

#[test]
fn e2e_42_home_fruit_and_guest_fruit() {
    let w = after_e2e_16();
    let ctx = &w.env.ctx;
    let [a, b, c] = w.partners;
    assert_eq!(cash_balance(&w), 45_000);
    assert_eq!(totals_of_three(&w), [1_424, 1_068, 1_068]);

    // Home fruit: 2 x 580 = 1,160 for الف from رحیم's lot.
    let preview = fruit::preview(ctx, &home(&w, "2", "580")).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let n = preview.numbers.as_ref().unwrap();
    assert_eq!(n.value_paisa.rupees(), 1_160);
    assert_eq!(n.parts.len(), 1);
    assert_eq!(
        (
            n.parts[0].partner.id,
            n.parts[0].amount_paisa.rupees(),
            n.parts[0].total_before_paisa.rupees(),
            n.parts[0].total_after_paisa.rupees()
        ),
        (a, 1_160, 1_424, 264)
    );
    assert!(n.parts_working.is_none());
    assert_eq!(n.credit.account.id, w.rahim);
    let lot = n.lot.as_ref().unwrap();
    assert_eq!((lot.balance_before, lot.balance_after), (5, 3));
    let saved = fruit::save(ctx, &w.owner, &home(&w, "2", "580")).unwrap();
    assert_eq!(saved.number, 1);
    assert_eq!(saved.preview, preview, "the save is the preview");

    // The lot balance is 3 (C16) and the 2 taken show in باقی مال.
    let stock = bikri::stock_list(ctx, &Default::default()).unwrap();
    assert_eq!(
        (
            stock.rows[0].qty_taken,
            stock.rows[0].balance,
            stock.balance_total
        ),
        (2, 3, 3)
    );
    // رحیم: ledger 1,590 + 1,160 = 2,750 جمع; the fruit credit is a payable posting (FR-7.59).
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 2_750));
    let payables = payables_of(&w, w.rahim);
    assert_eq!(payables.open_paisa.rupees(), 2_750);
    assert_eq!(payables.advances_open_paisa.rupees(), 0);
    let fruit_posting = payables
        .postings
        .iter()
        .find(|p| p.source == LineSource::Fruit)
        .unwrap();
    assert_eq!(
        (fruit_posting.number, fruit_posting.credit_paisa.rupees()),
        (Some(1), 1_160)
    );
    assert_eq!(partner_total(&w, 0), 264);
    assert_eq!(cash_balance(&w), 45_000, "no cash moves");

    // Guest fruit: 3 x 1,000 = 3,000 from حاجی نذیر's shop, split 1,000 / 1,000 / 1,000.
    let agent = nazeer(&w);
    let preview = fruit::preview(ctx, &guests(&w, agent, "3", "1,000")).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let n = preview.numbers.as_ref().unwrap();
    assert_eq!(n.value_paisa.rupees(), 3_000);
    let parts: Vec<(i64, i64)> = n
        .parts
        .iter()
        .map(|p| (p.partner.id, p.amount_paisa.rupees()))
        .collect();
    assert_eq!(parts, [(a, 1_000), (b, 1_000), (c, 1_000)]);
    assert!(n.parts_working.is_some());
    assert!(n.lot.is_none());
    let guests_saved = fruit::save(ctx, &w.owner, &guests(&w, agent, "3", "1,000")).unwrap();
    assert_eq!(guests_saved.number, 2);
    assert_eq!(payables_of(&w, agent).open_paisa.rupees(), 3_000);
    assert_eq!(balance(&w, agent), (BalanceSide::Cr, 3_000));

    // The books: totals -736 / 68 / 68, Σ -600 + 65,000 = 64,400 = equity, trial balance 77,610,
    // cash 45,000.
    assert_eq!(totals_of_three(&w), [-736, 68, 68]);
    assert_equity_is_the_partner_totals(&w, 64_400);
    assert_eq!(trial(&w), 77_610);
    assert_eq!(cash_balance(&w), 45_000);
    // Fruit is never a shop expense: no partner share rows, no analytics facts.
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM partner_share_rows WHERE journal_entry_id IN
             (SELECT id FROM journal_entries WHERE posting_type = 'fruit_taken')"
        ),
        0
    );
    // One stock move of kind taken for the lot, none for the other shop.
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM stock_moves WHERE kind = 'taken' AND lot_id = 1 AND qty = -2"
        ),
        1
    );
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM stock_moves WHERE kind = 'taken'"),
        1
    );

    // The saved rows say what the previews said.
    let list = fruit::list(ctx, &FruitListInput::default()).unwrap();
    assert_eq!(list.total_count, 2);
    assert_eq!(
        (list.totals.qty, list.totals.value_paisa.rupees()),
        (5, 4_160)
    );
    assert_eq!(list.rows[0].number, 2, "newest first");
    let row = fruit::get(
        ctx,
        &FruitGetInput {
            id: guests_saved.id,
        },
    )
    .unwrap();
    assert!(row.partner.is_none());
    assert_eq!(row.source, FruitSource::OtherShop);
    let row_parts: Vec<i64> = row.parts.iter().map(|p| p.amount_paisa.rupees()).collect();
    assert_eq!(row_parts, [1_000, 1_000, 1_000]);
    assert!(row.can_change);
    // The filters: الف's entries, the guests' entries, one item.
    let only_alif = fruit::list(
        ctx,
        &FruitListInput {
            partner_id: Some(a),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(only_alif.total_count, 1);
    let only_guests = fruit::list(
        ctx,
        &FruitListInput {
            guests_only: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (
            only_guests.total_count,
            only_guests.totals.value_paisa.rupees()
        ),
        (1, 3_000)
    );
    let apples = fruit::list(
        ctx,
        &FruitListInput {
            item_id: Some(w.apple),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(apples.total_count, 1);

    // الف's statement: the home fruit line is -1,160 with its item, packing, qty, rate and value;
    // the guest line is -1,000.
    let statement = statement_of(&w, a);
    let home_line = statement
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::FruitTaken && l.number == Some(1))
        .unwrap();
    assert_eq!(home_line.amount_paisa.unwrap().rupees(), -1_160);
    assert_eq!(home_line.subject.as_ref().unwrap().id, w.banana);
    assert_eq!(home_line.packing.as_ref().unwrap().id, w.box_);
    assert_eq!(value_of(home_line, "qty"), ErrorValue::Number(2));
    assert_eq!(
        value_of(home_line, "rate_paisa"),
        ErrorValue::Number(58_000)
    );
    assert_eq!(
        value_of(home_line, "value_paisa"),
        ErrorValue::Number(116_000)
    );
    assert_eq!(value_of(home_line, "for"), ErrorValue::Text("home".into()));
    let guest_line = statement
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::FruitTaken && l.number == Some(2))
        .unwrap();
    assert_eq!(guest_line.amount_paisa.unwrap().rupees(), -1_000);
    assert_eq!(
        value_of(guest_line, "for"),
        ErrorValue::Text("guests".into())
    );
    assert_eq!(statement.closing_total_paisa.rupees(), -736);
}

#[test]
fn c27_one_thousand_over_three_partners_is_334_333_333() {
    let w = after_e2e_16();
    let agent = nazeer(&w);
    let [a, b, c] = w.partners;
    fruit::save(&w.env.ctx, &w.owner, &guests(&w, agent, "1", "1,000")).unwrap();
    // The leftover rupee goes to the partner created first (E68).
    assert_eq!(totals_of_three(&w), [1_424 - 334, 1_068 - 333, 1_068 - 333]);
    let row = fruit::list(&w.env.ctx, &FruitListInput::default())
        .unwrap()
        .rows
        .remove(0);
    let parts: Vec<(i64, i64)> = row
        .parts
        .iter()
        .map(|p| (p.partner.id, p.amount_paisa.rupees()))
        .collect();
    assert_eq!(parts, [(a, 334), (b, 333), (c, 333)]);
    trial(&w);
}

#[test]
fn one_rupee_for_the_guests_gives_the_first_partner_all_of_it_and_the_others_no_line() {
    let w = after_e2e_16();
    let agent = nazeer(&w);
    let preview = fruit::preview(&w.env.ctx, &guests(&w, agent, "1", "1")).unwrap();
    let parts: Vec<i64> = preview
        .numbers
        .unwrap()
        .parts
        .iter()
        .map(|p| p.amount_paisa.rupees())
        .collect();
    assert_eq!(parts, [1], "a part of 0 is left out");
    let saved = fruit::save(&w.env.ctx, &w.owner, &guests(&w, agent, "1", "1")).unwrap();
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM journal_lines WHERE entry_id = {}",
                saved.journal_entry_id
            )
        ),
        2,
        "one partner line and the credit"
    );
    let row = fruit::get(&w.env.ctx, &FruitGetInput { id: saved.id }).unwrap();
    assert_eq!(row.parts.len(), 1);
    assert_eq!(totals_of_three(&w), [1_423, 1_068, 1_068]);
}

#[test]
fn the_refusals_come_back_at_their_fields() {
    let w = after_e2e_16();
    let agent = nazeer(&w);
    // More than the lot's balance (5 left): ERR_QTY_EXCEEDS_LOT with the balance.
    let preview = fruit::preview(&w.env.ctx, &home(&w, "6", "580")).unwrap();
    let error = preview
        .errors
        .iter()
        .find(|e| e.code == ErrorCode::ErrQtyExceedsLot)
        .unwrap();
    assert_eq!(error.field, "qty");
    assert_eq!(error.values.get("available"), Some(&ErrorValue::Number(5)));
    // The save refuses the same, with nothing written.
    let refused = fruit::save(&w.env.ctx, &w.owner, &home(&w, "6", "580")).unwrap_err();
    code_at(refused, ErrorCode::ErrQtyExceedsLot, "qty");
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM fruit_takes"), 0);
    // A rate of 0, a missing rate and decimals.
    assert!(
        preview_codes(&w, &home(&w, "2", "0"))
            .contains(&(ErrorCode::ErrMustBeMoreThanZero, Some("rate".into())))
    );
    assert!(
        preview_codes(&w, &home(&w, "2", "580.5"))
            .iter()
            .any(|(c, f)| *c == ErrorCode::ErrDecimalsNotAllowed && f.as_deref() == Some("rate"))
    );
    // For whom.
    let mut nobody = home(&w, "2", "580");
    nobody.partner_id = None;
    assert!(
        preview_codes(&w, &nobody)
            .contains(&(ErrorCode::ErrPartnerRequired, Some("partner_id".into())))
    );
    let mut both = guests(&w, agent, "1", "1,000");
    both.partner_id = Some(w.partners[0]);
    assert!(
        preview_codes(&w, &both)
            .contains(&(ErrorCode::ErrFormatInvalid, Some("partner_id".into())))
    );
    // The source: no lot, an unknown lot, no agent, a one-time seller as the agent.
    let mut no_lot = home(&w, "2", "580");
    no_lot.lot_id = None;
    assert!(preview_codes(&w, &no_lot).contains(&(ErrorCode::ErrRequired, Some("lot_id".into()))));
    let mut unknown = home(&w, "2", "580");
    unknown.lot_id = Some(999);
    assert!(
        preview_codes(&w, &unknown).contains(&(ErrorCode::ErrLotNotFound, Some("lot_id".into())))
    );
    let mut no_agent = guests(&w, agent, "1", "1,000");
    no_agent.agent_account_id = None;
    assert!(
        preview_codes(&w, &no_agent)
            .contains(&(ErrorCode::ErrRequired, Some("agent_account_id".into())))
    );
    let pukhta_seller = support::id_of(&w.env, "0031");
    let mut one_time = guests(&w, pukhta_seller, "1", "1,000");
    one_time.agent_account_id = Some(pukhta_seller);
    assert!(
        preview_codes(&w, &one_time)
            .contains(&(ErrorCode::ErrFormatInvalid, Some("agent_account_id".into())))
    );
    // A weight on an item sold by نگ.
    let mut weighed = guests(&w, agent, "1", "1,000");
    weighed.weight = Some("2.5".into());
    assert!(
        preview_codes(&w, &weighed).contains(&(ErrorCode::ErrFormatInvalid, Some("weight".into())))
    );
    // A closed day.
    let mut closed = home(&w, "2", "580");
    closed.date = "09-10-2026".into();
    assert!(preview_codes(&w, &closed).contains(&(ErrorCode::ErrDateClosed, Some("date".into()))));
}

#[test]
fn an_item_sold_by_weight_needs_a_weight_and_is_rounded_once() {
    let w = after_e2e_16();
    let agent = nazeer(&w);
    let grapes = support::add_item(&w.env, &w.owner, "انگور", "Grapes", "fruit", "weight");
    let mut input = guests(&w, agent, "1", "100");
    input.item_id = Some(grapes);
    // No weight.
    assert!(
        preview_codes(&w, &input).contains(&(ErrorCode::ErrWeightRequired, Some("weight".into())))
    );
    // 12.345 kg at 100 per kg = 1,234.5 -> 1,235 (C1).
    input.weight = Some("12.345".into());
    let preview = fruit::preview(&w.env.ctx, &input).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let n = preview.numbers.unwrap();
    assert_eq!(n.value_paisa.rupees(), 1_235);
    assert_eq!(n.weight_grams, Some(12_345));
    // 1 gram at 100 per kg is 0.1 rupee: a value of 0 is refused, not saved.
    input.weight = Some("0.001".into());
    assert!(
        preview_codes(&w, &input)
            .contains(&(ErrorCode::ErrMustBeMoreThanZero, Some("weight".into())))
    );
}

#[test]
fn a_fruit_entry_is_edited_by_reversal_and_cancelled_by_reversal() {
    let w = after_e2e_16();
    let ctx = &w.env.ctx;
    let saved = fruit::save(ctx, &w.owner, &home(&w, "2", "580")).unwrap();
    let before_edit = statement_of(&w, w.partners[0]);
    assert_eq!(before_edit.closing_total_paisa.rupees(), 264);

    // Edit 2 -> 3 نگ at 580 = 1,740. The preview (with editing_id) gives the old 2 back: balance 5.
    let mut edited = home(&w, "3", "580");
    edited.editing_id = Some(saved.id);
    let preview = fruit::preview(ctx, &edited).unwrap();
    let n = preview.numbers.unwrap();
    assert_eq!(
        n.lot.as_ref().map(|l| (l.balance_before, l.balance_after)),
        Some((5, 2))
    );
    assert_eq!(
        n.parts[0].total_before_paisa.rupees(),
        1_424,
        "his old part is given back"
    );
    assert_eq!(n.parts[0].total_after_paisa.rupees(), 1_424 - 1_740);
    // The edit asks for a reason.
    let no_reason = fruit::edit(
        ctx,
        &w.owner,
        &FruitEditInput {
            id: saved.id,
            version: saved.version,
            entry: edited.clone(),
            reason: " ".into(),
        },
    )
    .unwrap_err();
    code_at(no_reason, ErrorCode::ErrReasonRequired, "reason");
    let done = fruit::edit(
        ctx,
        &w.owner,
        &FruitEditInput {
            id: saved.id,
            version: saved.version,
            entry: edited,
            reason: "wrong quantity".into(),
        },
    )
    .unwrap();
    assert_eq!(done.number, 1);
    assert!(done.reversal_entry_id > 0 && done.journal_entry_id > done.reversal_entry_id);
    let stock = bikri::stock_list(ctx, &Default::default()).unwrap();
    assert_eq!((stock.rows[0].qty_taken, stock.rows[0].balance), (3, 2));
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 1_590 + 1_740));
    assert_eq!(partner_total(&w, 0), 1_424 - 1_740);
    trial(&w);
    // The old version is stale now.
    let stale = fruit::cancel(
        ctx,
        &w.owner,
        &FruitCancelInput {
            id: saved.id,
            version: saved.version,
            reason: "x".into(),
        },
    )
    .unwrap_err();
    code_at(stale, ErrorCode::ErrVersionChanged, "version");

    // While the fruit is taken the بکری cannot be cancelled (ERR_BIKRI_HAS_FRUIT).
    let bikri_version = bikri::get(ctx, 1).unwrap().version;
    let blocked = bikri::cancel(
        ctx,
        &w.owner,
        &CancelInput {
            id: 1,
            version: bikri_version,
            reason: "x".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap_err();
    code_at(blocked, ErrorCode::ErrBikriHasFruit, "id");

    // Cancel: everything comes back.
    let cancelled = fruit::cancel(
        ctx,
        &w.owner,
        &FruitCancelInput {
            id: saved.id,
            version: done.version,
            reason: "not taken".into(),
        },
    )
    .unwrap();
    assert_eq!(cancelled.number, 1);
    let stock = bikri::stock_list(ctx, &Default::default()).unwrap();
    assert_eq!((stock.rows[0].qty_taken, stock.rows[0].balance), (0, 5));
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 1_590));
    assert_eq!(totals_of_three(&w), [1_424, 1_068, 1_068]);
    let row = fruit::get(ctx, &FruitGetInput { id: saved.id }).unwrap();
    assert_eq!(row.status, FruitStatus::Cancelled);
    assert_eq!(row.cancel_reason.as_deref(), Some("not taken"));
    assert!(!row.can_change);
    assert_eq!(row.journal_entry_id, None);
    // A cancelled entry counts for nothing in the totals and cannot be cancelled again.
    let list = fruit::list(ctx, &FruitListInput::default()).unwrap();
    assert_eq!(list.total_count, 1);
    assert_eq!((list.totals.qty, list.totals.value_paisa.rupees()), (0, 0));
    let again = fruit::cancel(
        ctx,
        &w.owner,
        &FruitCancelInput {
            id: saved.id,
            version: row.version,
            reason: "x".into(),
        },
    )
    .unwrap_err();
    code_at(again, ErrorCode::ErrFruitCancelled, "id");
    // The audit chain stays intact and the books balance.
    assert_chain_ok(&w.dir);
    // Now the بکری can be cancelled.
    let version = bikri::get(ctx, 1).unwrap().version;
    bikri::cancel(
        ctx,
        &w.owner,
        &CancelInput {
            id: 1,
            version,
            reason: "x".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
}

#[test]
fn the_lot_picker_lists_lots_with_a_balance_and_the_last_fruit_empties_the_lot() {
    let w = after_e2e_16();
    let ctx = &w.env.ctx;
    let lots = fruit::lots(ctx, &FruitLotsInput::default()).unwrap();
    assert_eq!(lots.total_count, 1);
    let row = &lots.rows[0];
    assert_eq!(
        (
            row.lot_id,
            row.bikri_number,
            row.balance,
            row.sold_by_weight
        ),
        (1, 1, 5, false)
    );
    assert_eq!(row.seller.id, w.rahim);
    assert_eq!(row.item.id, w.banana);
    // A search by the seller's name, by the بکری number and a search that finds nothing.
    for (text, count) in [("rahim", 1), ("1", 1), ("nobody", 0)] {
        let found = fruit::lots(
            ctx,
            &FruitLotsInput {
                search: Some(text.into()),
                limit: 0,
            },
        )
        .unwrap();
        assert_eq!(found.total_count, count, "search {text}");
    }
    // Taking all 5 empties the lot: it is no longer offered, and 1 more is refused.
    fruit::save(ctx, &w.owner, &home(&w, "5", "580")).unwrap();
    assert_eq!(
        fruit::lots(ctx, &FruitLotsInput::default())
            .unwrap()
            .total_count,
        0
    );
    let refused = fruit::save(ctx, &w.owner, &home(&w, "1", "580")).unwrap_err();
    code_at(refused, ErrorCode::ErrQtyExceedsLot, "qty");
    assert!(
        bikri::stock_list(ctx, &Default::default())
            .unwrap()
            .rows
            .is_empty()
    );
}

/// Adds a partner, confirming the words the spelling check names (PRD FR-7.66).
fn new_partner(w: &support::World, name_ur: &str, name_en: &str) -> i64 {
    use mandi_services::core::partners::PartnerCreateInput;
    let mut words: Vec<String> = Vec::new();
    for _ in 0..4 {
        let input = PartnerCreateInput {
            name_ur: name_ur.to_owned(),
            name_en: name_en.to_owned(),
            opening_capital: None,
            confirmed_rules_words: words.clone(),
        };
        match partners::create(&w.env.ctx, &w.owner, &input) {
            Err(ServiceError::Rejected(items))
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                        words.extend(found.split(' ').map(str::to_owned));
                    }
                }
            }
            other => return other.unwrap().id,
        }
    }
    panic!("the partner was not accepted");
}

#[test]
fn an_edit_that_changes_nothing_that_decides_the_split_keeps_the_saved_split() {
    // Money agent rule 13: guests 1,000 on 10-10 split 334 / 333 / 333. A fourth partner then gets a
    // share from 10-10. An edit of the note only keeps the saved split; an edit of the value splits
    // again between the four.
    use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
    let w = after_e2e_16();
    let agent = nazeer(&w);
    let ctx = &w.env.ctx;
    let saved = fruit::save(ctx, &w.owner, &guests(&w, agent, "1", "1,000")).unwrap();
    let fourth = new_partner(&w, "ارشد", "Arshad");
    let [a, b, c] = w.partners;
    partners::set_shares(
        ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "10-10-2026".into(),
            shares: [(a, "25"), (b, "25"), (c, "25"), (fourth, "25")]
                .iter()
                .map(|&(partner_id, share)| ShareInput {
                    partner_id,
                    share: share.to_owned(),
                })
                .collect(),
        },
    )
    .unwrap();
    let parts_of = |id: i64| -> Vec<(i64, i64)> {
        fruit::get(ctx, &FruitGetInput { id })
            .unwrap()
            .parts
            .iter()
            .map(|p| (p.partner.id, p.amount_paisa.rupees()))
            .collect()
    };
    assert_eq!(parts_of(saved.id), [(a, 334), (b, 333), (c, 333)]);
    // A new entry of 1,000 now splits four ways.
    let fresh = fruit::preview(ctx, &guests(&w, agent, "1", "1,000")).unwrap();
    let fresh_parts: Vec<i64> = fresh
        .numbers
        .unwrap()
        .parts
        .iter()
        .map(|p| p.amount_paisa.rupees())
        .collect();
    assert_eq!(fresh_parts, [250, 250, 250, 250]);
    // The edit of the note only keeps the saved split.
    let mut note_only = guests(&w, agent, "1", "1,000");
    note_only.note = Some("for the visit".into());
    note_only.editing_id = Some(saved.id);
    let done = fruit::edit(
        ctx,
        &w.owner,
        &FruitEditInput {
            id: saved.id,
            version: saved.version,
            entry: note_only,
            reason: "note".into(),
        },
    )
    .unwrap();
    assert_eq!(parts_of(saved.id), [(a, 334), (b, 333), (c, 333)]);
    // The edit of the value splits again, between the four.
    let mut bigger = guests(&w, agent, "2", "1,000");
    bigger.editing_id = Some(saved.id);
    fruit::edit(
        ctx,
        &w.owner,
        &FruitEditInput {
            id: saved.id,
            version: done.version,
            entry: bigger,
            reason: "bigger".into(),
        },
    )
    .unwrap();
    assert_eq!(
        parts_of(saved.id),
        [(a, 500), (b, 500), (c, 500), (fourth, 500)]
    );
    trial(&w);
}

#[test]
fn the_one_time_seller_of_a_lot_with_fruit_taken_keeps_his_name_and_phone() {
    // The fruit credit rests on his person key, so an edit of the بکری cannot rename him.
    use mandi_services::core::bikri::{EditOpenInput, EditPreviewInput, PukhtaInput};
    let w = worked_day();
    let person = |name_ur: &str, name_en: &str| PukhtaInput {
        name_ur: name_ur.into(),
        name_en: name_en.into(),
        phone: "0301-1111111".into(),
    };
    let mut lot = support::with_sales(
        support::bare_lot(&w, "lot-1", w.pukhta_seller, w.banana, "10"),
        vec![support::sale("s-1", w.akram, "5", "600")],
    );
    lot.pukhta = Some(person("ناصر", "Nasir"));
    let saved = save_ok(&w, &support::draft(vec![lot]), false);
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let mut take = home(&w, "2", "580");
    take.lot_id = Some(lot_id);
    let taken = fruit::save(&w.env.ctx, &w.owner, &take).unwrap();
    // The credit is on the one-time seller's account, with his person key.
    let credit = taken.preview.numbers.unwrap().credit;
    assert_eq!(credit.account.id, w.pukhta_seller);
    assert_eq!(credit.pukhta.unwrap().name_en, "Nasir");
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM journal_lines WHERE entry_id = {} AND pukhta_person_key IS NOT NULL",
                taken.journal_entry_id
            )
        ),
        1
    );
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let preview_with = |pukhta: PukhtaInput| {
        let mut draft = opened.draft.clone();
        draft.lots[0].pukhta = Some(pukhta);
        bikri::edit_preview(
            &w.env.ctx,
            &EditPreviewInput {
                entry_id: saved.entry_id,
                draft,
            },
        )
        .unwrap()
    };
    // Another name is refused at the lot's pukhta; the same person is fine.
    let renamed = preview_with(person("شاہد", "Shahid"));
    assert!(
        renamed.errors.iter().any(|e| e.field == "lots[0].pukhta"),
        "{:?}",
        renamed.errors
    );
    let same = preview_with(person("ناصر", "Nasir"));
    assert!(
        same.errors.iter().all(|e| e.field != "lots[0].pukhta"),
        "{:?}",
        same.errors
    );
}

#[test]
fn a_lot_of_an_inactive_seller_is_refused_in_the_preview_like_in_the_save() {
    // The posting refuses an inactive account, so the preview says so at the field too (B5).
    let w = after_e2e_16();
    mandi_services::accounts::set_active(&w.env.ctx, &w.owner, w.rahim, false).unwrap();
    assert!(
        preview_codes(&w, &home(&w, "2", "580"))
            .contains(&(ErrorCode::ErrAccountInactive, Some("lot_id".into())))
    );
    let refused = fruit::save(&w.env.ctx, &w.owner, &home(&w, "2", "580")).unwrap_err();
    code_at(refused, ErrorCode::ErrAccountInactive, "lot_id");
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM fruit_takes"), 0);
}

#[test]
fn an_unknown_or_inactive_partner_is_not_found_at_partner_id() {
    // The lead's contract point: not ERR_PARTNER_NOT_LISTED, whose words are for the share list.
    let w = after_e2e_16();
    let mut unknown = home(&w, "2", "580");
    unknown.partner_id = Some(999);
    assert!(
        preview_codes(&w, &unknown).contains(&(ErrorCode::ErrNotFound, Some("partner_id".into())))
    );
    raw(&w.dir)
        .execute(
            "UPDATE partners SET is_active = 0 WHERE id = ?1",
            [w.partners[2]],
        )
        .unwrap();
    let mut inactive = home(&w, "2", "580");
    inactive.partner_id = Some(w.partners[2]);
    assert!(
        preview_codes(&w, &inactive).contains(&(ErrorCode::ErrNotFound, Some("partner_id".into())))
    );
    let refused = fruit::save(&w.env.ctx, &w.owner, &inactive).unwrap_err();
    code_at(refused, ErrorCode::ErrNotFound, "partner_id");
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM fruit_takes"), 0);
}
