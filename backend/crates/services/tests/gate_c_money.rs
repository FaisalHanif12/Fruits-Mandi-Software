//! Plan 80 gate fixes of backend-c that touch money, each with a test that fails on the code before
//! the fix (the "before" is named in each test):
//!
//! * R4-3 (PRD FR-12.51): a rent raised, an installment cancelled or an installment edited down may
//!   not leave the owner's refunds above `max(0, installments paid − rent)`.
//! * Q-R4-2 (PRD FR-12.51, FR-12.38): closing a garden while the owner still owes asks first.
//! * Q-R4-1 (PRD FR-12.29): goods arrived locks the shares of a garden (the service check; the
//!   trigger of migration 26 is in `crates/db/tests/migration.rs`).
//! * R4-4 (PRD FR-6.7): changing an opening that is in روکڑ asks the cash box instead of refusing.
//! * Q-R1-3 (PRD FR-17.13, 4.3): an advance is refused while partners exist and no share was set.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::arrivals::ArrivalInput;
use mandi_services::core::cashbook::CashbookInput;
use mandi_services::core::edit::{GardenEntryEditInput, GardenHeaderEditInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::gardens::{
    GardenCloseInput, GardenEntryCancelInput, GardenEntryInput, GardenGetInput, GardenHistoryKind,
    GardenInput, GardenPaymentInput, GardenRefundInput, GardenRow, GardenSetPartnersInput,
};
use mandi_services::core::partners::{OpeningCapitalWhere, PartnerCreateInput, PartnerRow};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::AdvanceInput;
use mandi_services::{ServiceError, accounts, cashbook, edit, gardens, partners, vouchers};
use reports_world::*;
use support::World;

// ---- Shared helpers ----

fn detail(w: &World, id: i64) -> mandi_services::core::gardens::GardenDetail {
    gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
}

fn row(w: &World, id: i64) -> GardenRow {
    detail(w, id).row
}

fn partners_of(w: &World) -> Vec<VenturePartnerInput> {
    let [a, b, _] = w.partners;
    vec![
        VenturePartnerInput {
            partner_id: a,
            share: "60".into(),
            source: MoneySource::Shop,
        },
        VenturePartnerInput {
            partner_id: b,
            share: "40".into(),
            source: MoneySource::Pocket,
        },
    ]
}

fn garden_input(w: &World, rent: &str) -> GardenInput {
    GardenInput {
        name_ur: "شاہ پور باغ".into(),
        name_en: "Shahpur Garden".into(),
        confirmed_rules_words: vec![
            "شاہ".into(),
            "پور".into(),
            "باغ".into(),
            "Shahpur".into(),
            "Garden".into(),
        ],
        date: "10-10-2026".into(),
        item_id: Some(w.banana),
        item_ids: Vec::new(),
        rent: rent.into(),
        season_end: None,
        note: None,
        partners: partners_of(w),
        paid_now: None,
        paid_now_reason: None,
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

fn edit_rent(w: &World, garden: &GardenRow, rent: &str) -> Result<(), ServiceError> {
    edit::garden_header_save(
        &w.env.ctx,
        &w.owner,
        &GardenHeaderEditInput {
            id: garden.id,
            version: row(w, garden.id).version,
            garden: garden_input(w, rent),
            reason: "relief from the owner".into(),
            sign: Some("adnan".into()),
        },
    )
    .map(|_| ())
}

fn refund(w: &World, garden: i64, amount: &str) -> Result<(), ServiceError> {
    gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::Refund(GardenRefundInput {
            garden_id: Some(garden),
            date: "10-10-2026".into(),
            amount: amount.into(),
            note: None,
        }),
    )
    .map(|_| ())
}

fn first_installment(w: &World, garden: i64) -> mandi_services::core::gardens::GardenHistoryRow {
    detail(w, garden)
        .history
        .into_iter()
        .find(|h| h.kind == GardenHistoryKind::Installment && h.number == Some(1))
        .expect("the first installment")
}

fn edit_installment(w: &World, garden: i64, amount: &str) -> Result<(), ServiceError> {
    let entry = first_installment(w, garden);
    edit::garden_entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryEditInput {
            id: entry.entry_id.unwrap(),
            version: entry.version.unwrap(),
            entry: GardenEntryInput::Installment(GardenPaymentInput {
                garden_id: Some(garden),
                date: "10-10-2026".into(),
                amount: amount.into(),
                reason: "first installment".into(),
                parts: Vec::new(),
                confirm_cash_below_zero: false,
            }),
            reason: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .map(|_| ())
}

fn rejection(error: &ServiceError) -> (ErrorCode, Option<String>, Vec<(String, i64)>) {
    match error {
        ServiceError::Rejected(items) => (
            items[0].code,
            items[0].field.clone(),
            items[0]
                .values
                .iter()
                .filter_map(|(k, v)| match v {
                    ErrorValue::Number(n) => Some((k.clone(), *n)),
                    ErrorValue::Text(_) => None,
                })
                .collect(),
        ),
        other => panic!("not a rejection: {other:?}"),
    }
}

/// The PRD FR-12.51 example at one hundredth: rent 50,000 all paid in one installment, a running
/// cost of 5,000, 30,000 from a sale in another shop, then a relief of 5,000 (rent 45,000). The
/// owner owes 5,000, and he has paid it back by a refund of 5,000.
fn refunded_garden(w: &World) -> GardenRow {
    let garden = garden_made_with(w, "50,000", "50,000");
    garden_pay(w, garden.id, true, "10-10-2026", "5,000", "labour");
    garden_other_sale(w, garden.id, "10-10-2026", "1", "30,000");
    edit_rent(w, &garden, "45,000").unwrap();
    assert_eq!(row(w, garden.id).owner_owes_paisa.rupees(), 5_000);
    refund(w, garden.id, "5,000").unwrap();
    assert_eq!(row(w, garden.id).owner_owes_paisa.rupees(), 0);
    garden
}

// ---- R4-3 ----

#[test]
fn r4_3_the_refunds_may_not_end_above_what_the_owner_could_owe() {
    let w = support::world();
    let garden = refunded_garden(&w);
    let cash_now = |w: &World| {
        cashbook::get(&w.env.ctx, &CashbookInput::default())
            .unwrap()
            .closing_paisa
            .rupees()
    };
    let cash_before = cash_now(&w);

    // (a) The rent raised back to 50,000: the owner would owe max(0, 50,000 - 50,000) = 0 against
    // the 5,000 he refunded. Before the fix the rent was saved and the refund stood alone.
    let error = edit_rent(&w, &garden, "50,000").unwrap_err();
    assert_eq!(
        rejection(&error),
        (
            ErrorCode::ErrReturnExceedsOpen,
            Some("rent".into()),
            vec![("open_paisa".into(), 0), ("refunded_paisa".into(), 500_000)]
        )
    );
    assert_eq!(row(&w, garden.id).rent_paisa.rupees(), 45_000);
    // Up to the most the refund allows: installments 50,000 less the refund 5,000 is a rent of 45,000
    // (an unchanged rent saves; a lower rent is always allowed).
    edit_rent(&w, &garden, "45,000").unwrap();
    edit_rent(&w, &garden, "44,000").unwrap();
    edit_rent(&w, &garden, "45,000").unwrap();

    // (b) The installment cancelled: 0 paid, the owner could owe 0.
    let entry = first_installment(&w, garden.id);
    let error = gardens::entry_cancel(
        &w.env.ctx,
        &w.owner,
        &GardenEntryCancelInput {
            id: entry.entry_id.unwrap(),
            version: entry.version.unwrap(),
            reason: "typed twice".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(
        rejection(&error),
        (
            ErrorCode::ErrReturnExceedsOpen,
            Some("id".into()),
            vec![("open_paisa".into(), 0), ("refunded_paisa".into(), 500_000)]
        )
    );

    // (c) The installment edited down by one rupee: 49,999 paid, the owner could owe 4,999 against
    // 5,000 refunded.
    let error = edit_installment(&w, garden.id, "49,999").unwrap_err();
    assert_eq!(
        rejection(&error),
        (
            ErrorCode::ErrReturnExceedsOpen,
            Some("amount".into()),
            vec![
                ("open_paisa".into(), 499_900),
                ("refunded_paisa".into(), 500_000)
            ]
        )
    );
    // Unchanged, or raised, it saves.
    edit_installment(&w, garden.id, "50,000").unwrap();
    edit_installment(&w, garden.id, "50,001").unwrap();
    // Nothing of the refused changes was written: the installments are the first one, edited up.
    assert_eq!(row(&w, garden.id).installments_paisa.rupees(), 50_001);
    assert_eq!(row(&w, garden.id).refunds_paisa.rupees(), 5_000);
    assert!(cash_now(&w) <= cash_before);
}

// ---- Q-R4-2 ----

#[test]
fn q_r4_2_closing_a_garden_while_the_owner_owes_asks_first() {
    let w = support::world();
    // Rent 50,000 all paid, a relief of 5,000: the owner owes 5,000 and has not paid it back.
    let garden = garden_made_with(&w, "50,000", "50,000");
    garden_pay(&w, garden.id, true, "10-10-2026", "5,000", "labour");
    garden_other_sale(&w, garden.id, "10-10-2026", "1", "30,000");
    edit_rent(&w, &garden, "45,000").unwrap();
    let input = || GardenCloseInput {
        id: garden.id,
        version: row(&w, garden.id).version,
        date: "10-10-2026".into(),
    };
    // Before the fix the garden was closed here, and no refund could be entered any more.
    let error = gardens::close(&w.env.ctx, &w.owner, &input()).unwrap_err();
    assert_eq!(
        rejection(&error),
        (
            ErrorCode::ErrOwnerOwesNotConfirmed,
            Some("id".into()),
            vec![("owes_paisa".into(), 500_000)]
        )
    );
    assert_eq!(
        row(&w, garden.id).status,
        mandi_services::core::ventures::VentureStatus::Open
    );
    // The same refusal with the answer "no".
    let error = gardens::close_with_confirm(&w.env.ctx, &w.owner, &input(), false).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrOwnerOwesNotConfirmed);
    // After he pays it back, nothing is asked: the plain close works.
    refund(&w, garden.id, "5,000").unwrap();
    gardens::close(&w.env.ctx, &w.owner, &input()).unwrap();
    assert_eq!(
        row(&w, garden.id).status,
        mandi_services::core::ventures::VentureStatus::Closed
    );

    // With the answer "yes" the garden closes while he still owes.
    let w2 = support::world();
    let g2 = garden_made_with(&w2, "50,000", "50,000");
    garden_pay(&w2, g2.id, true, "10-10-2026", "5,000", "labour");
    garden_other_sale(&w2, g2.id, "10-10-2026", "1", "30,000");
    edit_rent(&w2, &g2, "45,000").unwrap();
    gardens::close_with_confirm(
        &w2.env.ctx,
        &w2.owner,
        &GardenCloseInput {
            id: g2.id,
            version: row(&w2, g2.id).version,
            date: "10-10-2026".into(),
        },
        true,
    )
    .unwrap();
    assert_eq!(
        row(&w2, g2.id).status,
        mandi_services::core::ventures::VentureStatus::Closed
    );
}

// ---- Q-R4-1 ----

#[test]
fn q_r4_1_goods_arrived_locks_the_shares_of_a_garden() {
    let w = support::world();
    let garden = gardens::create(&w.env.ctx, &w.owner, &garden_input(&w, "50,000"))
        .unwrap()
        .row;
    let [a, b, c] = w.partners;
    let three = |first: &str, second: &str, third: &str| -> Vec<VenturePartnerInput> {
        [(a, first), (b, second), (c, third)]
            .into_iter()
            .map(|(partner_id, share)| VenturePartnerInput {
                partner_id,
                share: share.into(),
                source: MoneySource::Shop,
            })
            .collect()
    };
    let set = |partners: Vec<VenturePartnerInput>| {
        gardens::set_partners(
            &w.env.ctx,
            &w.owner,
            &GardenSetPartnersInput {
                id: garden.id,
                version: row(&w, garden.id).version,
                partners,
            },
        )
    };
    // No entry and no goods arrived: the shares can still be set again.
    set(three("40", "30", "30")).unwrap();
    // Goods arrived: it posts nothing and is not a venture entry, but the shares are fixed now.
    gardens::arrivals::save(
        &w.env.ctx,
        &w.owner,
        &ArrivalInput {
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            sell_date: None,
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            note: None,
        },
    )
    .unwrap();
    let error = set(three("50", "25", "25")).unwrap_err();
    assert_eq!(
        rejection(&error),
        (
            ErrorCode::ErrVentureSharesLocked,
            Some("partners".into()),
            Vec::new()
        )
    );
    // The database says the same by itself (migration 26), whatever the service does.
    let raw = w.env.ctx.db().write(|tx| {
        tx.execute(
            "DELETE FROM venture_partners WHERE venture_id = ?1",
            [garden.id],
        )?;
        Ok::<_, ServiceError>(())
    });
    assert!(
        raw.is_err(),
        "the trigger must refuse a delete of the shares"
    );
    assert_eq!(
        row(&w, garden.id).partners.len(),
        3,
        "the shares are unchanged"
    );
}

// ---- R4-4 ----

fn make_partner(w: &World, opening: &str) -> PartnerRow {
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = PartnerCreateInput {
            name_ur: "داؤد".into(),
            name_en: "Dawood".into(),
            opening_capital: Some(opening.into()),
            opening_capital_where: Some(OpeningCapitalWhere::Cash),
            confirmed_rules_words: words.clone(),
        };
        match partners::create(&w.env.ctx, &w.owner, &input) {
            Ok(row) => return row,
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

#[test]
fn r4_4_changing_an_opening_in_cash_asks_the_cash_box_instead_of_refusing() {
    // Cash 50,000 and Dawood's 50,000 is in it (cash 100,000). After 80,000 is paid out cash is
    // 20,000, and lowering his opening to 20,000 takes 30,000 out: cash would be -10,000.
    let w = support::world();
    let d = make_partner(&w, "50,000");
    let cash = |w: &World| {
        cashbook::get(&w.env.ctx, &CashbookInput::default())
            .unwrap()
            .closing_paisa
            .rupees()
    };
    assert_eq!(cash(&w), 100_000);
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: true,
            date: "10-10-2026".into(),
            seller_account_id: w.rahim,
            amount: "80,000".into(),
            removed: Default::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
    assert_eq!(cash(&w), 20_000);
    let version = accounts::get(&w.env.ctx, d.capital_account_id)
        .unwrap()
        .version;
    let input = OpeningsInput {
        lines: vec![OpeningLineInput {
            account_id: d.capital_account_id,
            version,
            opening_date: "10-10-2026".into(),
            opening_dr: String::new(),
            opening_cr: "20,000".into(),
            opening_is_advance: false,
        }],
    };
    // The refusal carries the cash after the change: this is what the box shows (FR-6.7).
    let preview = accounts::openings_preview(&w.env.ctx, &input).unwrap();
    assert_eq!(
        preview.rows[0].errors[0].code,
        ErrorCode::ErrCashBelowZeroNotConfirmed
    );
    assert_eq!(
        preview.rows[0].errors[0].values.get("cash_after_paisa"),
        Some(&ErrorValue::Number(-1_000_000))
    );
    let error = accounts::openings_save(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrCashBelowZeroNotConfirmed);
    assert_eq!(cash(&w), 20_000, "nothing changed");
    // With the answer "yes" the preview has no error and the save goes through. Before the fix there
    // was no way to say yes.
    let confirmed = accounts::openings_preview_with_confirm(&w.env.ctx, &input, true).unwrap();
    assert!(confirmed.rows[0].errors.is_empty());
    accounts::openings_save_with_confirm(&w.env.ctx, &w.owner, &input, true).unwrap();
    assert_eq!(cash(&w), -10_000);
}

// ---- Q-R1-3 ----

fn advance(w: &World, amount: &str) -> Result<(), ServiceError> {
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: true,
            date: "10-10-2026".into(),
            seller_account_id: w.rahim,
            amount: amount.into(),
            removed: Default::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .map(|_| ())
}

#[test]
fn q_r1_3_an_advance_is_refused_while_partners_exist_and_no_share_was_set() {
    // Partners A, B and C exist and no share was ever set: an advance would have no partner parts and
    // the check of PRD FR-17.9 would miss it. Before the fix it was saved.
    let w = support::world_with(50_000, false);
    let error = advance(&w, "15,000").unwrap_err();
    assert_eq!(
        rejection(&error),
        (ErrorCode::ErrSharesNot100, Some("date".into()), Vec::new())
    );
    // So is an opening advance of a seller.
    let karim_version = accounts::get(&w.env.ctx, w.karim).unwrap().version;
    let opening = OpeningsInput {
        lines: vec![OpeningLineInput {
            account_id: w.karim,
            version: karim_version,
            opening_date: "10-10-2026".into(),
            opening_dr: "5,000".into(),
            opening_cr: String::new(),
            opening_is_advance: true,
        }],
    };
    let error = accounts::openings_save(&w.env.ctx, &w.owner, &opening).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrSharesNot100);
    // An opening that is not an advance is not asked about shares.
    let mut plain = opening.clone();
    plain.lines[0].opening_is_advance = false;
    accounts::openings_save(&w.env.ctx, &w.owner, &plain).unwrap();

    // With the shares set (PRD FR-17.1) both work.
    let w = support::world();
    advance(&w, "15,000").unwrap();
}

#[test]
fn q_r1_3_before_any_partner_exists_an_advance_still_saves_as_in_the_worked_day() {
    // PRD 4.3: رحیم takes 15,000 on 01-09-26, before any partner or share exists. The parts are worked
    // out later from the first shares ever set (FR-17.13), so this must keep saving.
    let (dir, env) = common::env();
    common::set_up(&env);
    common::login_owner(&env);
    let owner = mandi_services::auth::require(&env.ctx).unwrap();
    let rahim = support::add_account(
        &env,
        &owner,
        mandi_services::core::accounts::AccountType::Seller,
        "رحیم",
        "Rahim",
        true,
        "",
    );
    vouchers::save_advance(
        &env.ctx,
        &owner,
        &AdvanceInput {
            confirm_cash_below_zero: true,
            date: "10-10-2026".into(),
            seller_account_id: rahim,
            amount: "15,000".into(),
            removed: Default::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
    drop(dir);
}
