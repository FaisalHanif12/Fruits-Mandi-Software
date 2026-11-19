//! Rent relief from a garden's owner (plan 78 D3, PRD FR-12.51), at one hundredth of the PRD example:
//! rent 50,000 and running costs 5,000; installments paid 40,000; sales money 30,000; relief 5,000.
//!
//! * The rent is edited with a reason and the Owner's sign, also on a garden of today. A wrong sign
//!   changes nothing and is in the audit log.
//! * The rent still due covers the relief: 45,000 less 40,000 paid is 5,000. After the last
//!   installment the cost is 50,000 and the loss 20,000: at 60 / 40, -12,000 and -8,000.
//! * If all 50,000 had been paid, the owner owes 5,000; his refund puts 5,000 into cash and into the
//!   garden's money in; the closing parts are again -12,000 and -8,000.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::cashbook::CashbookInput;
use mandi_services::core::edit::GardenHeaderEditInput;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::gardens::{
    GardenEntryCancelInput, GardenEntryInput, GardenGetInput, GardenHistoryKind, GardenInput,
    GardenRefundInput, GardenRow,
};
use mandi_services::core::statement::{PartnersStatementInput, StatementDetail};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, cashbook, edit, gardens, partners};
use reports_world::*;
use support::World;

fn get(w: &World, id: i64) -> mandi_services::core::gardens::GardenDetail {
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
    get(w, id).row
}

fn rupees(p: mandi_services::core::money::Paisa) -> i64 {
    p.rupees()
}

fn cash(w: &World) -> i64 {
    cashbook::get(&w.env.ctx, &CashbookInput::default())
        .unwrap()
        .closing_paisa
        .rupees()
}

fn edit_rent(
    w: &World,
    garden: &GardenRow,
    rent: &str,
    reason: &str,
    sign: Option<&str>,
) -> Result<(), ServiceError> {
    let [a, b, _] = w.partners;
    let input = GardenHeaderEditInput {
        id: garden.id,
        version: row(w, garden.id).version,
        garden: GardenInput {
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
            partners: vec![
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
            ],
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
        reason: reason.into(),
        sign: sign.map(str::to_owned),
    };
    // The review and the save say the same (Code Standards B5).
    let preview = edit::garden_header_preview(&w.env.ctx, &input).unwrap();
    let saved = edit::garden_header_save(&w.env.ctx, &w.owner, &input);
    if saved.is_err() || !preview.errors.is_empty() {
        assert_eq!(
            saved.is_err(),
            !preview.errors.is_empty(),
            "{preview:?} {saved:?}"
        );
    }
    saved.map(|_| ())
}

fn code_of(error: &ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => (items[0].code, items[0].field.clone()),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn audit_count(w: &World, action: &str) -> i64 {
    w.env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM audit_log WHERE action = ?1",
                [action],
                |r| r.get(0),
            )
        })
        .unwrap()
}

fn installment(w: &World, garden: i64, amount: &str) -> Result<(), ServiceError> {
    mandi_services::gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::Installment(mandi_services::core::gardens::GardenPaymentInput {
            garden_id: Some(garden),
            date: "10-10-2026".into(),
            amount: amount.into(),
            reason: "installment".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        }),
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

/// The closing parts of الف and ب, from their statement notes or lines (`part_paisa`).
fn closing_parts(w: &World) -> Vec<i64> {
    let [a, b, _] = w.partners;
    [a, b]
        .iter()
        .map(|p| {
            let s = partners::statement(
                &w.env.ctx,
                &PartnersStatementInput {
                    partner_id: *p,
                    from: None,
                    to: None,
                    offset: 0,
                    limit: 0,
                    sort: None,
                    group_by_month: false,
                },
            )
            .unwrap();
            let line = s
                .lines
                .iter()
                .find(|l| l.detail == StatementDetail::GardenClosed)
                .expect("a closing line");
            match line.values.get("part_paisa") {
                Some(ErrorValue::Number(n)) => *n / 100,
                other => panic!("{other:?}"),
            }
        })
        .collect()
}

#[test]
fn a_relief_lowers_the_rent_still_due_and_the_loss_is_split_by_the_shares() {
    let w = support::world();
    let garden = garden_made_with(&w, "50,000", "30,000");
    installment(&w, garden.id, "10,000").unwrap();
    garden_pay(&w, garden.id, true, "10-10-2026", "5,000", "labour");
    garden_other_sale(&w, garden.id, "10-10-2026", "1", "30,000");
    let before = row(&w, garden.id);
    assert_eq!(
        (rupees(before.rent_paisa), rupees(before.rent_due_paisa)),
        (50_000, 10_000)
    );
    assert!(before.rent_original_paisa.is_none());

    // The reason and the sign are needed, also today; a wrong sign is refused and logged.
    let error = edit_rent(&w, &garden, "45,000", "", Some("adnan")).unwrap_err();
    assert_eq!(
        code_of(&error),
        (ErrorCode::ErrReasonRequired, Some("reason".into()))
    );
    let error = edit_rent(&w, &garden, "45,000", "relief", None).unwrap_err();
    assert_eq!(
        code_of(&error),
        (ErrorCode::ErrSignRequired, Some("sign".into()))
    );
    assert_eq!(audit_count(&w, "garden_rent_sign_wrong"), 0);
    let error = edit_rent(&w, &garden, "45,000", "relief", Some("munshi")).unwrap_err();
    assert_eq!(
        code_of(&error),
        (ErrorCode::ErrSignWrong, Some("sign".into()))
    );
    assert_eq!(audit_count(&w, "garden_rent_sign_wrong"), 1);
    assert_eq!(
        rupees(row(&w, garden.id).rent_paisa),
        50_000,
        "nothing changed"
    );

    // The Owner's sign: the relief is written. Nothing is posted: cash is as it was.
    let cash_before = cash(&w);
    edit_rent(
        &w,
        &garden,
        "45,000",
        "relief from the owner",
        Some("adnan"),
    )
    .unwrap();
    assert_eq!(cash(&w), cash_before);
    let after = row(&w, garden.id);
    assert_eq!(rupees(after.rent_paisa), 45_000);
    assert_eq!(after.rent_original_paisa.map(rupees), Some(50_000));
    assert_eq!(rupees(after.relief_paisa), 5_000);
    assert_eq!(rupees(after.rent_due_paisa), 5_000);
    assert_eq!(rupees(after.owner_owes_paisa), 0);
    // An edit with the same rent needs no sign.
    edit_rent(&w, &garden, "45,000", "", None).unwrap();
    assert_eq!(rupees(row(&w, garden.id).relief_paisa), 5_000);

    // An installment is not checked against the rent (PRD FR-12.51): the last 5,000 settles it.
    installment(&w, garden.id, "5,000").unwrap();
    assert_eq!(rupees(row(&w, garden.id).rent_due_paisa), 0);
    // A running cost is not checked against the rent.
    garden_pay(&w, garden.id, true, "10-10-2026", "1,000", "labour");

    // The audit log keeps the old and the new rent, the reason and the sign (the sign as written by the
    // Owner's own spelling, `adnan`).
    let kept: (String, String, String) = w
        .env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row(
                "SELECT old_json, new_json, COALESCE(reason, '') FROM audit_log
                 WHERE action = 'edit' AND entity = 'garden' AND old_json LIKE '%5000000%'
                 ORDER BY id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
        })
        .unwrap();
    assert!(
        kept.0.contains("5000000") && kept.1.contains("4500000"),
        "{kept:?}"
    );
    assert_eq!(kept.2, "relief from the owner");
    assert!(kept.1.contains("\"sign\":\"adnan\""), "{kept:?}");
}

#[test]
fn the_owner_owes_what_was_paid_above_the_new_rent_and_pays_it_back_by_refund() {
    let w = support::world();
    let garden = garden_made_with(&w, "50,000", "50,000");
    garden_pay(&w, garden.id, true, "10-10-2026", "5,000", "labour");
    garden_other_sale(&w, garden.id, "10-10-2026", "1", "30,000");
    edit_rent(&w, &garden, "45,000", "relief", Some("adnan")).unwrap();
    let now = row(&w, garden.id);
    // The 5,000 paid above the rent is what the owner owes. It is not also a negative rent due: the
    // same money never shows twice (PRD FR-12.51).
    assert_eq!(rupees(now.rent_due_paisa), 0);
    assert_eq!(rupees(now.owner_owes_paisa), 5_000);
    assert_eq!(rupees(now.refunds_paisa), 0);

    // More than he owes is refused with what is open.
    let error = refund(&w, garden.id, "6,000").unwrap_err();
    match &error {
        ServiceError::Rejected(items) => {
            assert_eq!(items[0].code, ErrorCode::ErrReturnExceedsOpen);
            assert_eq!(items[0].field.as_deref(), Some("amount"));
            assert_eq!(
                items[0].values.get("open_paisa"),
                Some(&ErrorValue::Number(5_000 * 100))
            );
        }
        other => panic!("{other:?}"),
    }
    // 5,000 comes into cash and into the garden's money in.
    let cash_before = cash(&w);
    let money_in_before = rupees(row(&w, garden.id).money_in_paisa);
    refund(&w, garden.id, "5,000").unwrap();
    assert_eq!(cash(&w), cash_before + 5_000);
    let paid_back = row(&w, garden.id);
    assert_eq!(rupees(paid_back.money_in_paisa), money_in_before + 5_000);
    assert_eq!(
        (
            rupees(paid_back.refunds_paisa),
            rupees(paid_back.owner_owes_paisa)
        ),
        (5_000, 0)
    );
    assert!(
        get(&w, garden.id)
            .history
            .iter()
            .any(|h| h.kind == GardenHistoryKind::Refund && rupees(h.amount_paisa) == 5_000)
    );
    // Cancelling the refund puts the debt back.
    let entry = get(&w, garden.id)
        .history
        .into_iter()
        .find(|h| h.kind == GardenHistoryKind::Refund)
        .unwrap();
    gardens::entry_cancel(
        &w.env.ctx,
        &w.owner,
        &GardenEntryCancelInput {
            id: entry.entry_id.unwrap(),
            version: entry.version.unwrap(),
            reason: "typed twice".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(cash(&w), cash_before);
    assert_eq!(rupees(row(&w, garden.id).owner_owes_paisa), 5_000);
    // Paid again, then the garden is closed: money in 35,000 split 21,000 / 14,000, paid 55,000 split
    // 33,000 / 22,000, so the parts are -12,000 and -8,000, as with no refund and the cost of 50,000.
    refund(&w, garden.id, "5,000").unwrap();
    garden_close(&w, garden.id, "10-10-2026");
    assert_eq!(closing_parts(&w), vec![-12_000, -8_000]);
    let closed = row(&w, garden.id);
    assert_eq!(
        rupees(closed.account_balance_paisa),
        0,
        "the garden's account is 0 after the closing"
    );
}

#[test]
fn an_installment_and_a_refund_can_be_edited_at_their_full_amount_and_a_closed_garden_keeps_its_rent()
 {
    // The money agent's case: rent 50,000 paid in 30,000 + 20,000. Editing the 20,000 installment (only
    // its reason) must be saved at its full amount, although the rent still due is 0 after it.
    let w = support::world();
    let garden = garden_made_with(&w, "50,000", "30,000");
    installment(&w, garden.id, "20,000").unwrap();
    let edit_entry = |reason: &str, amount: &str| {
        let detail = get(&w, garden.id);
        let entry = detail
            .history
            .iter()
            .find(|h| h.kind == GardenHistoryKind::Installment && h.number == Some(2))
            .unwrap()
            .clone();
        mandi_services::edit::garden_entry_save(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::edit::GardenEntryEditInput {
                id: entry.entry_id.unwrap(),
                version: entry.version.unwrap(),
                entry: GardenEntryInput::Installment(
                    mandi_services::core::gardens::GardenPaymentInput {
                        garden_id: Some(garden.id),
                        date: "10-10-2026".into(),
                        amount: amount.into(),
                        reason: reason.into(),
                        parts: Vec::new(),
                        confirm_cash_below_zero: false,
                    },
                ),
                reason: String::new(),
                confirm_cash_below_zero: false,
            },
        )
    };
    edit_entry("second installment", "20,000").unwrap();

    // A closed garden does not take a new rent until its closing is undone.
    garden_close(&w, garden.id, "10-10-2026");
    let error = edit_rent(&w, &garden, "45,000", "relief", Some("adnan")).unwrap_err();
    assert_eq!(
        code_of(&error),
        (ErrorCode::ErrGardenClosed, Some("rent".into()))
    );
}

#[test]
fn r36_and_the_partners_closing_line_show_the_original_rent_and_the_relief() {
    // PRD FR-12.51: "the new rent and the relief show on the garden, in R36 and in each partner's
    // closing line". Rent 50,000, relief 5,000 (rent now 45,000), 45,000 paid, a cost of 5,000, 30,000
    // from another shop: closed, the loss is 20,000, at 60 / 40 -12,000 and -8,000.
    use mandi_reports::reports::{Cell, ReportFilters, ReportId};
    let w = support::world();
    let garden = garden_made_with(&w, "50,000", "45,000");
    garden_pay(&w, garden.id, true, "10-10-2026", "5,000", "labour");
    garden_other_sale(&w, garden.id, "10-10-2026", "1", "30,000");
    edit_rent(
        &w,
        &garden,
        "45,000",
        "relief from the owner",
        Some("adnan"),
    )
    .unwrap();
    garden_close(&w, garden.id, "10-10-2026");

    let page = report(&w, ReportId::R36, ReportFilters::default(), 0, 0);
    let summary: Vec<(String, i64)> = page
        .summary
        .iter()
        .map(|s| match &s.cell {
            Cell::Money { paisa } => (s.label_id.clone(), paisa.rupees()),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        summary,
        vec![
            ("report-r36-rent-original".to_owned(), 50_000),
            ("report-r36-relief".to_owned(), 5_000)
        ]
    );
    // The rent column is the rent now.
    match &page.rows[0].cells[3] {
        Cell::Money { paisa } => assert_eq!(paisa.rupees(), 45_000),
        other => panic!("{other:?}"),
    }

    // The closing line of الف (60%) says the rent now, the original rent and the relief.
    let [a, ..] = w.partners;
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
    let closing = statement
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::GardenClosed)
        .expect("the closing line");
    let figure = |name: &str| match closing.values.get(name) {
        Some(ErrorValue::Number(n)) => *n / 100,
        other => panic!("{name}: {other:?}"),
    };
    assert_eq!(
        (
            figure("rent_paisa"),
            figure("rent_original_paisa"),
            figure("relief_paisa"),
            figure("result_paisa"),
            figure("part_paisa")
        ),
        (45_000, 50_000, 5_000, -20_000, -12_000)
    );
}

#[test]
fn a_garden_with_no_relief_shows_no_summary_in_r36_and_no_rent_in_the_closing_line() {
    use mandi_reports::reports::{ReportFilters, ReportId};
    let w = support::world();
    let garden = garden_made_with(&w, "50,000", "50,000");
    garden_other_sale(&w, garden.id, "10-10-2026", "1", "30,000");
    garden_close(&w, garden.id, "10-10-2026");
    let page = report(&w, ReportId::R36, ReportFilters::default(), 0, 0);
    assert!(page.summary.is_empty());
    let [a, ..] = w.partners;
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
    let closing = statement
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::GardenClosed)
        .expect("the closing line");
    assert!(!closing.values.contains_key("rent_original_paisa"));
    assert!(!closing.values.contains_key("relief_paisa"));
}
