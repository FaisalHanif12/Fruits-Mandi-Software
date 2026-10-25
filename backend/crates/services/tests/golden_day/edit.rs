//! Plan 16 on the PRD worked day: edit (E2E-13, E2E-13b, E2E-14), cancel (E2E-15), and a part-sold
//! بکری with its later sale (E2E-16, E2E-16b). The expected numbers are the PRD's and the plan's;
//! they are never changed to suit the code.

use mandi_services::core::bikri::{
    BikriEdited, CancelInput, EditOpenInput, EditPreviewInput, EditSaveInput, EditableBikri,
    LaterSaleInput, LaterSaleSaveInput, StockListInput,
};
use mandi_services::core::rules::ChargesInput;
use mandi_services::rules;

use super::*;

/// The first posting of a saved بکری, loaded for editing.
fn open(w: &support::World, bikri_id: i64, entry_id: i64) -> EditableBikri {
    bikri::open_for_edit(&w.env.ctx, &EditOpenInput { bikri_id, entry_id }).unwrap()
}

/// Edits and saves; the preview just before must be the numbers the save wrote (Code Standards
/// B5).
fn edit(w: &support::World, opened: &EditableBikri, draft: BikriDraft) -> BikriEdited {
    let preview = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let saved = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("ریٹ کی درستی".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(saved.preview, preview);
    saved
}

/// The stored lot columns (gross, seller commission, deductions, net) in rupees.
fn lot_numbers(w: &support::World, bikri_id: i64) -> (i64, i64, i64, i64) {
    let lot = &bikri::get(&w.env.ctx, bikri_id).unwrap().lots[0];
    (
        lot.gross.rupees(),
        lot.seller_comm.rupees(),
        lot.deductions.rupees(),
        lot.net.rupees(),
    )
}

/// Σ `sign × column` over the analytics facts, in rupees.
fn facts(w: &support::World, column: &str) -> i64 {
    scalar(
        w,
        &format!("SELECT COALESCE(SUM(sign * {column}), 0) FROM analytics_facts"),
    ) / 100
}

/// The sale lines in force of a بکری as (buyer total, seller commission, buyer commission), rupees.
fn live_lines(w: &support::World) -> Vec<(i64, i64, i64)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT buyer_total_paisa, seller_comm_paisa, buyer_comm_paisa FROM bikri_sales
             WHERE is_active = 1 ORDER BY lot_id, line_no, id",
        )
        .unwrap();
    statement
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)? / 100,
                r.get::<_, i64>(1)? / 100,
                r.get::<_, i64>(2)? / 100,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn open_advance(w: &support::World) -> i64 {
    assist::advances(&w.env.ctx, w.rahim)
        .unwrap()
        .open_paisa
        .rupees()
}

#[test]
fn e2e_13_and_14_edit_akrams_rate_to_610_and_back_to_600() {
    let w = worked_day();
    let saved = save_ok(&w, &support::event1(&w), false);
    let before_lines = entry_lines(&w, saved.entry_id);
    let opened = open(&w, saved.bikri_id, saved.entry_id);
    assert!(opened.editable && opened.posting_type == "bikri_first");
    assert_eq!(opened.draft.lots[0].lot_id, Some(1));
    assert_eq!(opened.draft.lots[0].sales[0].sale_id, Some(1));
    assert_eq!(opened.draft.lots[0].sales[0].seller_rate, "600");
    // Rule-decided charges come back empty: nothing was typed.
    assert!(opened.draft.lots[0].sales[0].seller_comm.is_none());

    // E2E-13: اکرم's rate 600 → 610.
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    let edited = edit(&w, &opened, draft);
    let summary = &edited.preview.summary;
    assert_eq!(
        (
            summary.gross.rupees(),
            summary.seller_comm.rupees(),
            summary.deductions.rupees(),
            summary.net.rupees(),
            summary.shared_income.rupees()
        ),
        (29_600, 1_776, 10_096, 19_504, 3_972)
    );
    assert_eq!(
        lot_numbers(&w, saved.bikri_id),
        (29_600, 1_776, 10_096, 19_504)
    );
    // His line: 12,200 + 732 + 160 = 13,092. ارشد's line is unchanged and keeps 1,044.
    assert_eq!(live_lines(&w), [(13_092, 732, 732), (18_684, 1_044, 1_044)]);
    assert_eq!(shares_of(&w, edited.entry_id), [1_590, 1_191, 1_191]);
    // The old posting is not touched; its reversal is its exact mirror, dated the same day.
    assert_eq!(entry_lines(&w, saved.entry_id), before_lines);
    assert_eq!(shares_of(&w, saved.entry_id), [1_580, 1_184, 1_184]);
    assert_eq!(
        shares_of(&w, edited.reversal_entry_id),
        [-1_580, -1_184, -1_184]
    );
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM journal_entries WHERE id = {} AND posting_type = 'reversal'
                   AND reverses_entry_id = {} AND entry_date = '2026-10-10'",
                edited.reversal_entry_id, saved.entry_id
            )
        ),
        1
    );
    // Balances: اکرم 13,092; رحیم's advance of 15,000 stays fully adjusted, so he is owed
    // 24,504 − 5,000 − 15,000 = 4,504.
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 13_092));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 18_684));
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 4_504));
    assert_eq!(open_advance(&w), 0);
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM advance_allocations WHERE is_active = 1"
        ),
        1_500_000
    );
    // Stock, facts and lines.
    assert_eq!(
        scalar(&w, "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"),
        0
    );
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM stock_moves"), 9);
    assert_eq!(facts(&w, "seller_amount_paisa"), 29_600);
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(sign * qty), 0) FROM analytics_facts"
        ),
        50
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bikri_sales WHERE is_active = 0 AND ended = 'replaced'
               AND replaced_by_sale_id IS NOT NULL"
        ),
        2
    );
    // The audit log holds the old version and the new one, and the reason.
    let (old_json, new_json, reason): (String, String, String) = raw(&w.dir)
        .query_row(
            "SELECT old_json, new_json, reason FROM audit_log WHERE entity = 'bikri' AND action = 'edit'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert!(old_json.contains("\"seller_rate_paisa\":60000") && old_json.contains("72000"));
    assert!(new_json.contains("\"seller_rate_paisa\":61000") && new_json.contains("73200"));
    assert_eq!(reason, "ریٹ کی درستی");
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);

    // E2E-14: edit back to 600. Every number returns exactly to E2E-1 to E2E-4.
    let again = open(&w, saved.bikri_id, edited.entry_id);
    assert_eq!(again.version, edited.version);
    let mut draft = again.draft.clone();
    draft.lots[0].sales[0].seller_rate = "600".into();
    let back = edit(&w, &again, draft);
    let summary = &back.preview.summary;
    assert_eq!(
        (
            summary.gross.rupees(),
            summary.deductions.rupees(),
            summary.net.rupees(),
            summary.shared_income.rupees()
        ),
        (29_400, 10_084, 19_316, 3_948)
    );
    assert_eq!(
        lot_numbers(&w, saved.bikri_id),
        (29_400, 1_764, 10_084, 19_316)
    );
    assert_eq!(live_lines(&w), [(12_880, 720, 720), (18_684, 1_044, 1_044)]);
    assert_eq!(shares_of(&w, back.entry_id), [1_580, 1_184, 1_184]);
    assert_eq!(
        entry_lines(&w, back.entry_id),
        before_lines,
        "the journal is exactly the one of Event 1"
    );
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 12_880));
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 4_316));
    assert_eq!(open_advance(&w), 0);
    assert_eq!(facts(&w, "seller_amount_paisa"), 29_400);
    assert_eq!(
        scalar(&w, "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"),
        0
    );
    // Cash: only the نقد of 5,000 left the till, once.
    assert_eq!(
        balance(&w, support::id_of(&w.env, "01010001")),
        (BalanceSide::Dr, 45_000)
    );
    // Income after both edits: the partner shares sum to Event 1's alone.
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows WHERE kind = 'income'"
        ),
        394_800
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

#[test]
fn e2e_13b_an_unchanged_line_keeps_its_stored_commission_when_the_default_changes() {
    let w = worked_day();
    let saved = save_ok(&w, &support::event1(&w), false);
    // The Owner changes the default buyer commission to 7%, from today.
    let c = rules::charges(&w.env.ctx).unwrap();
    rules::set_charges(
        &w.env.ctx,
        &w.owner,
        &ChargesInput {
            seller_commission: mandi_services::core::rules::DefaultChargeInput {
                version: c.seller_commission.version,
                method: c.seller_commission.method,
                value: c.seller_commission.value_text.clone(),
            },
            buyer_commission: mandi_services::core::rules::DefaultChargeInput {
                version: c.buyer_commission.version,
                method: c.buyer_commission.method,
                value: "7".into(),
            },
            laga_fruit: mandi_services::core::rules::DefaultChargeInput {
                version: c.laga_fruit.version,
                method: c.laga_fruit.method,
                value: c.laga_fruit.value_text.clone(),
            },
        },
    )
    .unwrap();
    let opened = open(&w, saved.bikri_id, saved.entry_id);
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    let edited = edit(&w, &opened, draft);
    // اکرم's changed line looks the rules up again: 12,200 × 7 ÷ 100 = 854; he pays
    // 12,200 + 854 + 160 = 13,214. ارشد's unchanged line keeps its stored 6%: 1,044.
    assert_eq!(live_lines(&w), [(13_214, 732, 854), (18_684, 1_044, 1_044)]);
    // Shared income 1,776 + 1,898 + 400 + 20 = 4,094; split 1,638 / 1,228 / 1,228.
    assert_eq!(edited.preview.summary.shared_income.rupees(), 4_094);
    assert_eq!(shares_of(&w, edited.entry_id), [1_638, 1_228, 1_228]);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn e2e_15_cancel_bikri_2_reverses_everything_and_keeps_the_books_balanced() {
    let w = worked_day();
    let one = save_ok(&w, &support::event1(&w), false);
    let two = save_ok(&w, &event2(&w), false);
    let cash = support::id_of(&w.env, "01010001");
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 57_800));
    let detail = bikri::get(&w.env.ctx, two.bikri_id).unwrap();
    let preview = bikri::cancel_preview(
        &w.env.ctx,
        &mandi_services::core::bikri::CancelPreviewInput { id: two.bikri_id },
    )
    .unwrap();
    assert!(preview.can_cancel && preview.closed_dates.is_empty());
    assert_eq!(preview.cash.change_paisa.rupees(), -12_800);
    // A reason is required.
    let refused = bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: two.bikri_id,
            version: detail.version,
            reason: "  ".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(first_field(&refused).as_deref(), Some("reason"));
    let done = bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: two.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!((done.number, done.reversal_entry_ids.len()), (2, 1));
    // The register shows شاہد's line cancelled.
    let register = bikri::register(&w.env.ctx, &Default::default()).unwrap();
    assert_eq!(register.rows.len(), 1);
    assert!(register.rows[0].cancelled && register.rows[0].bikri_number == 2);
    // The سیب facts net to zero (and Event 1's stay): analytics excludes it.
    let apple = scalar(
        &w,
        &format!(
            "SELECT COALESCE(SUM(sign * seller_amount_paisa), 0) FROM analytics_facts WHERE item_id = {}",
            w.apple
        ),
    );
    assert_eq!(apple, 0);
    assert_eq!(facts(&w, "seller_amount_paisa"), 29_400);
    // The split 616 / 462 / 462 is reversed.
    assert_eq!(
        shares_of(&w, done.reversal_entry_ids[0]),
        [-616, -462, -462]
    );
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows WHERE kind = 'income'
                   AND journal_entry_id IN ({}, {}, {})",
                two.entry_id, done.reversal_entry_ids[0], one.entry_id
            )
        ),
        394_800
    );
    // Cash down by 12,800, back to 45,000.
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 45_000));
    // The بکری keeps its number and is cancelled; its lines are cancelled, not deleted.
    let after = bikri::get(&w.env.ctx, two.bikri_id).unwrap();
    assert_eq!(after.number, 2);
    assert_eq!(
        after.status,
        mandi_services::core::bikri::BikriStatus::Cancelled
    );
    assert_eq!(after.cancel_reason.as_deref(), Some("غلط اندراج"));
    assert!(
        after.lots[0].sales.iter().all(|s| {
            !s.is_active && s.state == mandi_services::core::bikri::SaleState::Cancelled
        })
    );
    assert_eq!(
        scalar(&w, "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"),
        0
    );
    // Cancelling twice is refused; the numbers are never reused.
    let again = bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: two.bikri_id,
            version: after.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(code_and_value(&again, "x").0, ErrorCode::ErrBikriCancelled);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

/// Event 1 with only 45 of 50 sold: اکرم 20 at 600, ارشد 25 at 580 (E2E-16).
pub(super) fn event1_part_sold(w: &support::World) -> BikriDraft {
    let mut draft = support::event1(w);
    draft.lots[0].sales[1].qty = "25".into();
    draft
}

#[test]
fn e2e_16_and_16b_a_part_sold_bikri_and_its_later_sale() {
    let w = worked_day();
    let saved = save_ok(&w, &event1_part_sold(&w), false);
    // E2E-16: خام 26,500; seller commission 720 + 870 = 1,590; کل اخراجات 9,910; صافی 16,590;
    // buyer commission 1,590; لاگا 45 × 8 = 360; shared income 3,560; split 1,424 / 1,068 / 1,068.
    let s = &saved.preview.summary;
    assert_eq!(
        (
            s.gross.rupees(),
            s.seller_comm.rupees(),
            s.deductions.rupees(),
            s.net.rupees(),
            s.buyer_comm.rupees(),
            s.laga.rupees(),
            s.shared_income.rupees()
        ),
        (26_500, 1_590, 9_910, 16_590, 1_590, 360, 3_560)
    );
    assert_eq!(shares_of(&w, saved.entry_id), [1_424, 1_068, 1_068]);
    // باقی مال: 5 نگ کیلا.
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(stock.rows.len(), 1);
    let row = &stock.rows[0];
    assert_eq!(
        (
            row.lot_id,
            row.bikri_number,
            row.qty_received,
            row.qty_sold,
            row.balance,
            row.days_waiting
        ),
        (1, 1, 50, 45, 5, 0)
    );
    assert_eq!(stock.balance_total, 5);
    assert_eq!(row.item.id, w.banana);

    // E2E-16b: on 11-10-26 the last 5 are sold to ارشد at 560.
    set_today(&w.env, 2026, 10, 11);
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    let preview = bikri::later_sale_preview(
        &w.env.ctx,
        &LaterSaleInput {
            lot_id: row.lot_id,
            date: "11-10-2026".into(),
            sales: vec![line.clone()],
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let calc = preview.lots[0].lines[0].calc.as_ref().unwrap();
    assert_eq!(
        (
            calc.seller_amount.rupees(),
            calc.seller_comm.amount.rupees(),
            calc.buyer_comm.amount.rupees(),
            calc.laga.amount.rupees(),
            calc.buyer_total.rupees()
        ),
        (2_800, 168, 168, 40, 3_008)
    );
    let totals = preview.lots[0].totals.as_ref().unwrap();
    // Seller credit 2,800 − 168 = 2,632, and no lot charge again.
    assert_eq!(totals.seller_credit.rupees(), 2_632);
    assert_eq!(
        (
            totals.freight.rupees(),
            totals.labour.rupees(),
            totals.munshiana.rupees(),
            totals.phone.rupees(),
            totals.cash_paid.rupees()
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(preview.summary.shared_income.rupees(), 376);
    let done = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: row.lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(done.preview, preview);
    assert_eq!(shares_of(&w, done.entry_id), [152, 112, 112]);
    // The lot totals: خام 29,300, کل اخراجات 10,078, صافی 19,222.
    assert_eq!(
        lot_numbers(&w, saved.bikri_id),
        (29_300, 1_758, 10_078, 19_222)
    );
    // ارشد pays 3,008 more; باقی مال is empty.
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 15_570 + 3_008));
    assert!(
        bikri::stock_list(&w.env.ctx, &StockListInput::default())
            .unwrap()
            .rows
            .is_empty()
    );
    // The original posting and its day are untouched.
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM journal_entries WHERE id = {} AND entry_date = '2026-10-10'",
                saved.entry_id
            )
        ),
        1
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

#[test]
fn a_lot_balance_subtracts_the_fruit_taken_in_every_place_c16() {
    // C16: balance = received - sold - written off - taken. After E2E-16 the lot has 5 left; 2 of
    // them are taken as fruit (plan 17d writes `qty_taken`; here it is set directly).
    use mandi_services::core::bikri::WriteOffInput;
    let w = worked_day();
    let saved = save_ok(&w, &event1_part_sold(&w), false);
    raw(&w.dir)
        .execute("UPDATE bikri_lots SET qty_taken = 2 WHERE id = 1", [])
        .unwrap();
    // باقی مال shows 3 left and the 2 taken.
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    let row = &stock.rows[0];
    assert_eq!(
        (
            row.qty_received,
            row.qty_sold,
            row.qty_written_off,
            row.qty_taken,
            row.balance
        ),
        (50, 45, 0, 2, 3)
    );
    assert_eq!(stock.balance_total, 3);
    // The saved lot carries the same figure.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(detail.lots[0].qty_taken, 2);
    // A later sale of 4 is refused (3 left); a sale of 3 is allowed.
    set_today(&w.env, 2026, 10, 11);
    let later = |qty: &str| {
        let mut line = support::sale("s-3", w.arshad, qty, "560");
        line.sale_id = None;
        bikri::later_sale_preview(
            &w.env.ctx,
            &LaterSaleInput {
                lot_id: row.lot_id,
                date: "11-10-2026".into(),
                sales: vec![line],
            },
        )
        .unwrap()
    };
    assert!(
        later("4")
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrQtyExceedsLot),
        "4 is more than the 3 left"
    );
    assert!(later("3").errors.is_empty());
    // A write-off of 4 is refused with 3 available.
    let error = bikri::write_off(
        &w.env.ctx,
        &w.owner,
        &WriteOffInput {
            lot_id: row.lot_id,
            qty: "4".into(),
            reason: "spoiled".into(),
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrWriteOffExceedsBalance);
    // A write-off of 3 empties the lot: nothing is left in باقی مال.
    let done = bikri::write_off(
        &w.env.ctx,
        &w.owner,
        &WriteOffInput {
            lot_id: row.lot_id,
            qty: "3".into(),
            reason: "spoiled".into(),
        },
    )
    .unwrap();
    assert_eq!((done.qty_written_off, done.balance), (3, 0));
    assert!(
        bikri::stock_list(&w.env.ctx, &StockListInput::default())
            .unwrap()
            .rows
            .is_empty()
    );
}
