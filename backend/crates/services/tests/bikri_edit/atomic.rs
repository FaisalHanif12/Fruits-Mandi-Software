//! All or nothing (PRD FR-12.8, NFR-REL-1, E20, plan 16): a failure forced after each step of an
//! edit, a cancel, an undo, a later sale, a write-off and the undo of a write-off leaves no row,
//! no changed row and no audit row.
//!
//! The failure is a SQLite trigger made on a second connection that refuses the next INSERT or
//! UPDATE on one table, exactly where the action has got to. This is the power cut of the plan.
//! The fingerprint holds every column of every table the actions write.

use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriDraft, LaterSaleSaveInput, StockListInput, UndoInput, WriteOffInput, WriteOffUndoInput,
};
use mandi_services::core::error::ErrorCode;

use crate::common::{assert_chain_ok, raw};
use crate::helpers::*;
use crate::support::*;
use crate::tools;

/// The steps of the first posting of a big بکری.
const EDIT_STEPS: &[(&str, &str)] = &[
    ("journal_entries", "INSERT"),
    ("journal_lines", "INSERT"),
    ("journal_entries", "UPDATE"),
    ("settings", "UPDATE"),
    ("partner_share_rows", "INSERT"),
    ("stock_moves", "INSERT"),
    ("analytics_facts", "INSERT"),
    ("bikri_sales", "UPDATE"),
    ("bikri_sales", "INSERT"),
    ("bikri_lots", "UPDATE"),
    ("bikri_entries", "UPDATE"),
    ("bill_allocations", "INSERT"),
    ("bill_allocations", "UPDATE"),
    ("advance_allocations", "INSERT"),
    ("advance_allocations", "UPDATE"),
    ("audit_log", "INSERT"),
];

const CANCEL_STEPS: &[(&str, &str)] = &[
    ("journal_entries", "INSERT"),
    ("journal_lines", "INSERT"),
    ("journal_entries", "UPDATE"),
    ("settings", "UPDATE"),
    ("partner_share_rows", "INSERT"),
    ("stock_moves", "INSERT"),
    ("analytics_facts", "INSERT"),
    ("bikri_sales", "UPDATE"),
    ("bikri_entries", "UPDATE"),
    ("bill_allocations", "UPDATE"),
    ("advance_allocations", "UPDATE"),
    ("audit_log", "INSERT"),
];

const LATER_STEPS: &[(&str, &str)] = &[
    ("journal_entries", "INSERT"),
    ("journal_lines", "INSERT"),
    ("journal_entries", "UPDATE"),
    ("settings", "UPDATE"),
    ("partner_share_rows", "INSERT"),
    ("bikri_sales", "INSERT"),
    ("analytics_facts", "INSERT"),
    ("bikri_lots", "UPDATE"),
    ("bikri_entries", "UPDATE"),
    ("bill_allocations", "INSERT"),
    ("advance_allocations", "INSERT"),
    ("audit_log", "INSERT"),
];

const WRITE_OFF_STEPS: &[(&str, &str)] = &[
    ("stock_moves", "INSERT"),
    ("bikri_lots", "UPDATE"),
    ("audit_log", "INSERT"),
];

/// A بکری that touches every table: Event 1 with 45 of 50 sold (advance adjusted) and a second
/// lot with a cash sale to a one-time buyer (pinned), then receipts from اکرم and ارشد.
fn big_world() -> (World, mandi_services::core::bikri::BikriSaved) {
    let w = world();
    // More advance than the first posting uses (21,590), so a later sale adjusts some too.
    give_advance(&w, w.rahim, "30,000");
    let mut line = sale("s-9", w.pukhta_buyer, "10", "1200");
    line.pukhta = person("شاہد", "Shahid", "0300-0000000");
    let mut second = lot(&w, "lot-2", w.karim, w.apple, "12");
    second.freight = Some("1,000".into());
    second.cash_paid = None;
    second.sales = vec![line];
    let mut d = event1(&w);
    d.lots[0].sales[1].qty = "25".into();
    d.lots.push(second);
    let saved = save_ok(&w, &d);
    tools::receipt(&w, w.akram, "5,000");
    // ارشد pays more than his bill (15,570), so a later sale to him is matched with his credit.
    tools::receipt(&w, w.arshad, "20,000");
    (w, saved)
}

fn run_steps(w: &World, steps: &[(&str, &str)], what: &str, act: &dyn Fn() -> bool) {
    let conn = raw(&w.dir);
    let before = tools::fingerprint(&conn);
    for (table, event) in steps {
        conn.execute_batch(&format!(
            "CREATE TRIGGER force_fail BEFORE {event} ON {table} BEGIN SELECT RAISE(ABORT, 'forced'); END;"
        ))
        .unwrap();
        let failed = act();
        conn.execute_batch("DROP TRIGGER force_fail;").unwrap();
        assert!(
            failed,
            "{what} went through although {event} on {table} was refused"
        );
        assert_eq!(
            tools::fingerprint(&conn),
            before,
            "{what}: {event} on {table} left something behind"
        );
    }
}

fn edited_draft(w: &World, opened: &mandi_services::core::bikri::EditableBikri) -> BikriDraft {
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    d.lots[1].sales[0].seller_rate = "1300".into();
    let _ = w;
    d
}

#[test]
fn a_failure_forced_after_every_step_of_an_edit_leaves_nothing_behind() {
    let (w, saved) = big_world();
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let d = edited_draft(&w, &opened);
    run_steps(&w, EDIT_STEPS, "the edit", &|| {
        tools::try_edit(&w, &opened, &d, true).is_err()
    });
    // The same edit without a failure goes through: the draft was fine.
    let done = tools::edit(&w, &opened, &d);
    assert_eq!(tools::trial_difference(&w), 0);
    assert_chain_ok(&w.dir);
    mandi_services::ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_ne!(done.entry_id, saved.entry_id);
}

#[test]
fn a_failure_forced_after_every_step_of_a_cancel_and_an_undo_leaves_nothing_behind() {
    let (w, saved) = big_world();
    run_steps(&w, CANCEL_STEPS, "the cancel", &|| {
        tools::cancel(&w, saved.bikri_id, true).is_err()
    });
    run_steps(&w, CANCEL_STEPS, "the undo", &|| {
        bikri::undo(
            &w.env.ctx,
            &w.owner,
            &UndoInput {
                id: saved.bikri_id,
                confirm_cash_below_zero: true,
            },
        )
        .is_err()
    });
    tools::cancel(&w, saved.bikri_id, true).unwrap();
    assert_eq!(tools::trial_difference(&w), 0);
    assert_chain_ok(&w.dir);
}

#[test]
fn a_failure_forced_after_every_step_of_a_later_sale_leaves_nothing_behind() {
    let (w, saved) = big_world();
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let input = LaterSaleSaveInput {
        lot_id,
        date: "10-10-2026".into(),
        sales: vec![sale("l-1", w.arshad, "5", "560")],
        confirm_cash_below_zero: true,
    };
    run_steps(&w, LATER_STEPS, "the later sale", &|| {
        bikri::later_sale_save(&w.env.ctx, &w.owner, &input).is_err()
    });
    bikri::later_sale_save(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn a_failure_forced_after_every_step_of_a_write_off_and_its_undo_leaves_nothing_behind() {
    let (w, saved) = big_world();
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let off = WriteOffInput {
        lot_id,
        qty: "3".into(),
        reason: "خراب".into(),
    };
    run_steps(&w, WRITE_OFF_STEPS, "the write-off", &|| {
        bikri::write_off(&w.env.ctx, &w.owner, &off).is_err()
    });
    let done = bikri::write_off(&w.env.ctx, &w.owner, &off).unwrap();
    let undo = WriteOffUndoInput {
        move_id: done.move_id,
        reason: "غلطی".into(),
    };
    run_steps(&w, WRITE_OFF_STEPS, "the undo of the write-off", &|| {
        bikri::write_off_undo(&w.env.ctx, &w.owner, &undo).is_err()
    });
    bikri::write_off_undo(&w.env.ctx, &w.owner, &undo).unwrap();
    assert_eq!(
        bikri::stock_list(&w.env.ctx, &StockListInput::default())
            .unwrap()
            .rows
            .len(),
        2
    );
}

#[test]
fn a_refused_action_writes_nothing_at_all() {
    // Plain refusals (not forced failures) leave the fingerprint alone as well.
    let (w, saved) = big_world();
    let conn = raw(&w.dir);
    let before = tools::fingerprint(&conn);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].qty = "9999".into();
    assert!(tools::try_edit(&w, &opened, &d, true).is_err());
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].sale_id = Some(424_242);
    assert!(tools::try_edit(&w, &opened, &d, true).is_err());
    let e = bikri::write_off(
        &w.env.ctx,
        &w.owner,
        &WriteOffInput {
            lot_id: 1,
            qty: "999".into(),
            reason: "x".into(),
        },
    )
    .unwrap_err();
    assert!(tools::refused(
        &e,
        ErrorCode::ErrWriteOffExceedsBalance,
        "qty"
    ));
    assert_eq!(tools::fingerprint(&conn), before);
}
