//! Money paid on the spot in an edit (PRD FR-12.44, FR-12.20, plan 60 P11, plan 65 point 5): a line
//! whose only change is its paid amount keeps every stored rule and amount, and only the cash part
//! of the posting moves. The numbers are those of E2E-13b (the default buyer commission is 7% after
//! the sale was saved) with ارشد paying 10,000 of his 18,684 on the spot.

use mandi_services::core::rules::{ChargesInput, DefaultChargeInput};
use mandi_services::rules;

use crate::common::raw;
use crate::helpers::*;
use crate::support::*;
use crate::tools;

/// The Owner changes the default buyer commission to 7%, after the sale was saved.
fn buyer_commission_7(w: &World) {
    let c = rules::charges(&w.env.ctx).unwrap();
    rules::set_charges(
        &w.env.ctx,
        &w.owner,
        &ChargesInput {
            seller_commission: DefaultChargeInput {
                version: c.seller_commission.version,
                method: c.seller_commission.method,
                value: c.seller_commission.value_text.clone(),
            },
            buyer_commission: DefaultChargeInput {
                version: c.buyer_commission.version,
                method: c.buyer_commission.method,
                value: "7".into(),
            },
            laga_fruit: DefaultChargeInput {
                version: c.laga_fruit.version,
                method: c.laga_fruit.method,
                value: c.laga_fruit.value_text.clone(),
            },
        },
    )
    .unwrap();
}

/// The stored rule and amount columns of the live lines, as text, without the paid amount.
fn stored_rules(w: &World) -> Vec<String> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT quote(buyer_account_id) || '|' || quote(qty) || '|' || quote(seller_rate_paisa)
                || '|' || quote(seller_comm_method) || '|' || quote(seller_comm_value_bps)
                || '|' || quote(seller_comm_rule_id) || '|' || quote(seller_comm_paisa)
                || '|' || quote(buyer_comm_method) || '|' || quote(buyer_comm_value_bps)
                || '|' || quote(buyer_comm_rule_id) || '|' || quote(buyer_comm_paisa)
                || '|' || quote(laga_method) || '|' || quote(laga_value_paisa)
                || '|' || quote(laga_rule_id) || '|' || quote(laga_paisa)
                || '|' || quote(buyer_total_paisa)
             FROM bikri_sales WHERE is_active = 1 ORDER BY line_no, id",
        )
        .unwrap();
    statement
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn a_paid_only_edit_keeps_the_stored_rules_and_moves_only_the_cash_part() {
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    let before = stored_rules(&w);
    assert_eq!(before.len(), 2);
    // Event 1: nothing was paid on the spot; cash is 50,000 - 5,000 نقد.
    assert_eq!(tools::cash(&w), 45_000);
    buyer_commission_7(&w);

    // Only ارشد's paid on the spot changes: 10,000 of 18,684. The edit screen starts with it empty.
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert_eq!(opened.draft.lots[0].sales[1].paid, None);
    let mut d = opened.draft.clone();
    d.lots[0].sales[1].paid = Some("10,000".into());
    let edited = tools::edit(&w, &opened, &d);

    // Every stored rule and amount is as it was saved (not the 7% of today).
    assert_eq!(stored_rules(&w), before, "a stored rule or amount changed");
    // ارشد's new line: 18,684 with 10,000 paid and 8,684 left; اکرم's line is as it was.
    let lines: Vec<(i64, i64, i64)> = raw(&w.dir)
        .prepare(
            "SELECT buyer_total_paisa / 100, paid_paisa / 100, payment_type = 'cash'
             FROM bikri_sales WHERE is_active = 1 ORDER BY line_no, id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(lines, [(12_880, 0, 0), (18_684, 10_000, 0)]);
    // The shared income and its split are those of Event 1 (PRD E2E-4).
    assert_eq!(r(edited.preview.summary.shared_income), 3_948);
    assert_eq!(tools::shares_of(&w, edited.entry_id), [1_580, 1_184, 1_184]);
    // The posting now has a cash part of 10,000 besides his 5,000 نقد, and cash is 10,000 up.
    let cash_part: Vec<(String, i64, i64)> = entry_lines(&w, edited.entry_id)
        .into_iter()
        .filter(|l| l.0 == "01010001" || l.0 == "0021")
        .map(|l| (l.0, l.1, l.2))
        .collect();
    assert_eq!(
        cash_part,
        [
            ("0021".to_owned(), 18_684, 0),
            ("01010001".to_owned(), 0, 5_000),
            ("01010001".to_owned(), 10_000, 0),
            ("0021".to_owned(), 0, 10_000),
        ]
    );
    assert_eq!(tools::cash(&w), 55_000);
    // Matching: his 10,000 is pinned to his own bill; 8,684 of it is open. اکرم's bill is open whole.
    assert_eq!(tools::open_bills(&w, w.arshad), (8_684, 0));
    assert_eq!(tools::open_bills(&w, w.akram), (12_880, 0));
    // The audit log shows the paid amount of the new posting.
    let new_json: String = raw(&w.dir)
        .query_row(
            "SELECT new_json FROM audit_log WHERE action = 'edit' ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(new_json.contains("\"paid_paisa\":1000000"), "{new_json}");
    assert_eq!(tools::trial_difference(&w), 0);

    // Clearing the cell again takes the 10,000 back: paid is 0, the rules are still the stored ones.
    let opened = tools::open(&w, saved.bikri_id, edited.entry_id);
    assert_eq!(
        opened.draft.lots[0].sales[1].paid.as_deref(),
        Some("10000"),
        "the edit screen is given what was saved"
    );
    let mut d = opened.draft.clone();
    d.lots[0].sales[1].paid = None;
    tools::edit(&w, &opened, &d);
    assert_eq!(stored_rules(&w), before);
    assert_eq!(tools::cash(&w), 45_000);
    assert_eq!(tools::open_bills(&w, w.arshad), (18_684, 0));
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn a_paid_amount_above_the_stored_total_is_refused_by_the_edit_and_nothing_changes() {
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[1].paid = Some("18,685".into());
    let error = tools::try_edit(&w, &opened, &d, true).unwrap_err();
    assert!(
        tools::refused(
            &error,
            mandi_services::core::error::ErrorCode::ErrPaidExceedsTotal,
            "lots[0].sales[1].paid"
        ),
        "{error:?}"
    );
    assert_eq!(
        tools::refusal_number(
            &error,
            mandi_services::core::error::ErrorCode::ErrPaidExceedsTotal,
            "max"
        ),
        Some(18_684)
    );
    assert_eq!(tools::cash(&w), 45_000);
}
