//! The quality of a sale line (درجہ, PRD FR-12.42, plan 58): optional, three fixed words, and no
//! change to any figure.
//!
//! The worked numbers are those of plan 58 section 3: Ehsan's 20 پیٹی سیب, seller commission 6%:
//! اکرم takes 10 اول at 1,200 (12,000, commission 720), 6 دوم at 800 (4,800, commission 288),
//! ارشد takes 4 سوم at 500 (2,000, commission 120). Together 20, 18,800 and 1,128, the same as
//! with no quality at all.

use mandi_services::bikri;
use mandi_services::core::bikri::{BikriDraft, LaterSaleSaveInput, SaleGrade, SaleInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::rules::{ChargesInput, DefaultChargeInput};
use mandi_services::rules;

use crate::common::raw;
use crate::helpers::*;
use crate::support::*;
use crate::tools::{self, flagged};

fn graded(mut line: SaleInput, grade: &str) -> SaleInput {
    line.grade = Some(grade.into());
    line
}

/// The 20 apples of plan 58, with the qualities or without them.
fn apples(w: &World, qualities: bool) -> BikriDraft {
    let q = |line: SaleInput, grade: &str| {
        if qualities { graded(line, grade) } else { line }
    };
    draft(vec![with_sales(
        bare_lot(w, "lot-1", w.rahim, w.apple, "20"),
        vec![
            q(sale("s-1", w.akram, "10", "1200"), "first"),
            q(sale("s-2", w.akram, "6", "800"), "second"),
            q(sale("s-3", w.arshad, "4", "500"), "third"),
        ],
    )])
}

/// The live lines of the saved sales, with every stored amount and rule of the line as text, and
/// the quality: `(grade, the rest)`.
fn stored_lines(w: &World, active: bool) -> Vec<(String, String)> {
    let conn = raw(&w.dir);
    let sql = format!(
        "SELECT quote(grade),
                quote(qty) || '|' || quote(seller_rate_paisa) || '|' || quote(seller_amount_paisa)
                || '|' || quote(seller_comm_method) || '|' || quote(seller_comm_value_bps)
                || '|' || quote(seller_comm_rule_id) || '|' || quote(seller_comm_paisa)
                || '|' || quote(buyer_rate_paisa) || '|' || quote(buyer_value_paisa)
                || '|' || quote(buyer_comm_method) || '|' || quote(buyer_comm_value_bps)
                || '|' || quote(buyer_comm_rule_id) || '|' || quote(buyer_comm_paisa)
                || '|' || quote(laga_method) || '|' || quote(laga_value_paisa)
                || '|' || quote(laga_rule_id) || '|' || quote(laga_paisa)
                || '|' || quote(buyer_total_paisa) || '|' || quote(rate_diff_paisa)
         FROM bikri_sales WHERE is_active = {} ORDER BY line_no, id",
        i32::from(active)
    );
    let mut statement = conn.prepare(&sql).unwrap();
    statement
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn a_quality_changes_no_figure() {
    // PRD FR-12.42: the figures are the same as with no quality at all.
    let w = world();
    let with = preview(&w, &apples(&w, true));
    let without = preview(&w, &apples(&w, false));
    assert!(with.errors.is_empty(), "{:?}", with.errors);
    let t = with.lots[0].totals.as_ref().unwrap();
    assert_eq!((r(t.gross), r(t.seller_comm)), (18_800, 1_128));
    let line = |i: usize| {
        let c = calc(&with.lots[0], i);
        (
            r(c.seller_amount),
            r(c.seller_comm.amount),
            r(c.buyer_comm.amount),
            r(c.laga.amount),
            r(c.buyer_total),
        )
    };
    assert_eq!(line(0), (12_000, 720, 720, 80, 12_800));
    assert_eq!(line(1), (4_800, 288, 288, 48, 5_136));
    assert_eq!(line(2), (2_000, 120, 120, 32, 2_152));
    for i in 0..3 {
        assert_eq!(with.lots[0].lines[i].calc, without.lots[0].lines[i].calc);
    }
    assert_eq!(with.summary, without.summary);
    assert_eq!(with.split, without.split);
}

#[test]
fn only_the_three_words_are_a_quality_and_an_empty_text_is_none() {
    let w = world();
    let mut bad = apples(&w, false);
    bad.lots[0].sales[0].grade = Some("fourth".into());
    let p = preview(&w, &bad);
    assert!(
        flagged(
            &p.errors,
            ErrorCode::ErrOutOfRange,
            "lots[0].sales[0].grade"
        ),
        "{:?}",
        p.errors
    );
    // The line with the unknown word is not worked out until it is fixed.
    assert!(p.lots[0].lines[0].calc.is_none());
    assert!(p.lots[0].lines[1].calc.is_some());
    // The save refuses it at the same field, and nothing is written.
    let error = save(&w, &bad).unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrOutOfRange, "lots[0].sales[0].grade"),
        "{error:?}"
    );
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_sales"), 0);
    // Capital letters and Urdu are not the words either; an empty text or spaces is no quality.
    for text in ["First", "اول", " second "] {
        let mut d = apples(&w, false);
        d.lots[0].sales[0].grade = Some(text.into());
        let p = preview(&w, &d);
        let bad_word = flagged(
            &p.errors,
            ErrorCode::ErrOutOfRange,
            "lots[0].sales[0].grade",
        );
        // Only surrounding spaces are forgiven.
        assert_eq!(bad_word, text != " second ", "{text:?}");
    }
    for text in ["", "   "] {
        let mut d = apples(&w, false);
        d.lots[0].sales[0].grade = Some(text.into());
        let p = preview(&w, &d);
        assert!(p.errors.is_empty(), "{text:?}: {:?}", p.errors);
    }
}

#[test]
fn the_quality_is_stored_on_the_line_and_read_back_and_left_out_of_the_json_when_empty() {
    let w = world();
    let mut d = apples(&w, true);
    d.lots[0].sales[2].grade = None;
    let saved = save_ok(&w, &d);
    let got = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let grades: Vec<Option<SaleGrade>> = got.lots[0].sales.iter().map(|s| s.grade).collect();
    assert_eq!(
        grades,
        vec![Some(SaleGrade::First), Some(SaleGrade::Second), None]
    );
    // The JSON the screen reads: the word when there is a quality, no key when there is none.
    let json = |i: usize| serde_json::to_value(&got.lots[0].sales[i]).unwrap();
    assert_eq!(json(0)["grade"], "first");
    assert_eq!(json(1)["grade"], "second");
    assert!(json(2).get("grade").is_none());
    // The stored column, and the stored amounts are the ones of the no-quality case.
    assert_eq!(
        stored_lines(&w, true)
            .iter()
            .map(|(g, _)| g.as_str())
            .collect::<Vec<_>>(),
        vec!["'first'", "'second'", "NULL"]
    );
    assert_eq!(
        scalar(&w, "SELECT SUM(seller_amount_paisa) FROM bikri_sales"),
        1_880_000
    );
    assert_eq!(
        scalar(&w, "SELECT SUM(seller_comm_paisa) FROM bikri_sales"),
        112_800
    );
    // The column accepts nothing but the three words, and the quality of a saved line is frozen
    // like the rest of its snapshot: only an edit, which is audited, can give the goods another
    // quality (migration 19).
    let conn = raw(&w.dir);
    let refused_word = conn.execute("UPDATE bikri_sales SET grade = 'fourth' WHERE id = 1", []);
    assert!(refused_word.is_err());
    let frozen = conn
        .execute("UPDATE bikri_sales SET grade = 'third' WHERE id = 1", [])
        .unwrap_err();
    assert!(frozen.to_string().contains("ERR_APPEND_ONLY"), "{frozen}");
    let cleared = conn
        .execute("UPDATE bikri_sales SET grade = NULL WHERE id = 1", [])
        .unwrap_err();
    assert!(cleared.to_string().contains("ERR_APPEND_ONLY"), "{cleared}");
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn the_edit_screen_gets_the_quality_back_and_a_quality_only_edit_keeps_every_stored_rule() {
    let w = world();
    let saved = save_ok(&w, &apples(&w, true));
    let first_save = stored_lines(&w, true);
    assert_eq!(first_save.len(), 3);
    // The Owner changes the default buyer commission to 7% after the sale was saved.
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
    // The edit screen starts from the saved qualities.
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let loaded: Vec<Option<String>> = opened.draft.lots[0]
        .sales
        .iter()
        .map(|s| s.grade.clone())
        .collect();
    assert_eq!(
        loaded,
        vec![
            Some("first".to_owned()),
            Some("second".to_owned()),
            Some("third".to_owned())
        ]
    );
    // Only the qualities change: line 2 becomes third, line 3 has none.
    let mut d = opened.draft.clone();
    d.lots[0].sales[1].grade = Some("third".into());
    d.lots[0].sales[2].grade = None;
    let edited = tools::edit(&w, &opened, &d);
    // Every line is "unchanged": the stored rules and amounts are kept (not the 7% of today), so
    // the new rows hold the same amounts as the old ones, with the new qualities.
    let after = stored_lines(&w, true);
    assert_eq!(after.len(), 3);
    for (old, new) in first_save.iter().zip(&after) {
        assert_eq!(old.1, new.1, "a stored rule or amount changed");
    }
    assert_eq!(
        after.iter().map(|(g, _)| g.as_str()).collect::<Vec<_>>(),
        vec!["'first'", "'third'", "NULL"]
    );
    // The replaced rows keep the qualities they were saved with.
    let old_rows = stored_lines(&w, false);
    assert_eq!(
        old_rows.iter().map(|(g, _)| g.as_str()).collect::<Vec<_>>(),
        vec!["'first'", "'second'", "'third'"]
    );
    // The audit log shows the quality change (old and new side), though no amount changed.
    let (old_json, new_json): (String, String) = raw(&w.dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE action = 'edit' ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(old_json.contains("\"grade\":\"second\""), "{old_json}");
    assert!(new_json.contains("\"grade\":\"third\""), "{new_json}");
    assert!(!new_json.contains("\"grade\":\"second\""), "{new_json}");
    // No figure moved: the edited posting's totals are the ones of the first.
    assert_eq!(r(edited.preview.summary.gross), 18_800);
    assert_eq!(r(edited.preview.summary.buyer_comm), 1_128);
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn a_later_sale_carries_its_own_quality() {
    let w = world();
    let mut d = apples(&w, false);
    // 10 of the 20 are sold now, with no quality.
    d.lots[0].sales.truncate(1);
    let saved = save_ok(&w, &d);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let lot_id = opened.draft.lots[0].lot_id.unwrap();
    let mut line = sale("l-1", w.arshad, "10", "700");
    line.grade = Some("second".into());
    let later = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert!(later.entry_id > saved.entry_id);
    let got = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let grades: Vec<Option<SaleGrade>> = got.lots[0].sales.iter().map(|s| s.grade).collect();
    assert_eq!(grades, vec![None, Some(SaleGrade::Second)]);
    assert_eq!(tools::trial_difference(&w), 0);
}
