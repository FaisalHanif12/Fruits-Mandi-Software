//! Plan 17 T2c: the full day close summary (PRD FR-21.1, 4.9). Every expected number is from the
//! PRD and is never changed to suit the code.
use super::*;
use mandi_services::core::bikri::CancelInput;
use mandi_services::core::vouchers::DayClosePreview;

use super::day_money::{event5, events_1_to_4, expense_head};
use mandi_services::core::bikri::{LaterSaleSaveInput, StockListInput};

fn preview(w: &support::World, up_to: &str) -> DayClosePreview {
    day::close_preview(
        &w.env.ctx,
        &DayClosePreviewInput {
            up_to: up_to.into(),
        },
    )
    .unwrap()
}

/// The partner rows of a summary as (income, expense, profit) in rupees.
fn shares(p: &DayClosePreview) -> Vec<(i64, i64, i64)> {
    p.summary
        .partners
        .iter()
        .map(|r| {
            (
                r.income_paisa.rupees(),
                r.expense_paisa.rupees(),
                r.profit_paisa.rupees(),
            )
        })
        .collect()
}

#[test]
fn the_summary_of_the_worked_day_is_the_values_of_prd_4_9() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);

    let p = preview(&w, "");
    assert_eq!(
        (p.from_date.as_str(), p.up_to.as_str()),
        ("2026-10-10", "2026-10-10")
    );
    // 2 بکری; خام 41,400; commission 3,928 + 1,520 = 5,448; shared income 5,488.
    let s = &p.summary;
    assert_eq!(
        (
            s.bikri_count,
            s.gross_paisa.rupees(),
            s.commission_paisa.rupees(),
            s.shared_income_paisa.rupees()
        ),
        (2, 41_400, 5_448, 5_488)
    );
    // Shares 2,196 - 1,200 = 996; 1,646 - 900 = 746; 746, in the order the partners were made.
    assert_eq!(
        shares(&p),
        vec![(2_196, 1_200, 996), (1_646, 900, 746), (1_646, 900, 746)]
    );
    let ids: Vec<i64> = s.partners.iter().map(|r| r.partner.id).collect();
    assert_eq!(ids, w.partners);
    // The partner profits add up to the firm's profit 5,488 - 3,000 = 2,488.
    let profit: i64 = shares(&p).iter().map(|r| r.2).sum();
    assert_eq!(profit, 2_488);
    // Cash 58,484; the trial balance difference is 0; nothing left in باقی مال.
    assert_eq!(p.cash_in_hand.amount_paisa.rupees(), 58_484);
    assert_eq!(p.trial_difference.amount_paisa.rupees(), 0);
    assert_eq!((s.open_lots, s.open_qty), (0, 0));
    // Reading the summary writes nothing.
    assert_eq!(preview(&w, ""), p);
}

#[test]
fn a_cancelled_bikri_adds_nothing_to_the_summary() {
    // E2E-15: بکری 2 is cancelled, so the day is بکری 1 alone.
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    let two = save_ok(&w, &event2(&w), false);
    let version = mandi_services::bikri::get(&w.env.ctx, two.bikri_id)
        .unwrap()
        .version;
    mandi_services::bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: two.bikri_id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let p = preview(&w, "");
    let s = &p.summary;
    assert_eq!(
        (
            s.bikri_count,
            s.gross_paisa.rupees(),
            s.commission_paisa.rupees(),
            s.shared_income_paisa.rupees()
        ),
        (1, 29_400, 3_928, 3_948)
    );
    assert_eq!(
        shares(&p),
        vec![(1_580, 0, 1_580), (1_184, 0, 1_184), (1_184, 0, 1_184)]
    );
    assert_eq!(p.trial_difference.amount_paisa.rupees(), 0);
}

#[test]
fn a_part_sold_bikri_leaves_lots_in_stock_until_the_later_sale() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save_ok(&w, &draft, false);
    // E2E-16: خام 26,500; commission 1,590 + 1,590 + 360 = 3,540; shared income 3,560; 5 نگ left.
    let p = preview(&w, "");
    let s = &p.summary;
    assert_eq!(
        (
            s.bikri_count,
            s.gross_paisa.rupees(),
            s.commission_paisa.rupees(),
            s.shared_income_paisa.rupees(),
            s.open_lots,
            s.open_qty
        ),
        (1, 26_500, 3_540, 3_560, 1, 5)
    );
    assert_eq!(
        shares(&p),
        vec![(1_424, 0, 1_424), (1_068, 0, 1_068), (1_068, 0, 1_068)]
    );

    // Close 10-10-26. On 11-10-26 the last 5 are sold (E2E-16b): one بکری with sales that day,
    // خام 2,800, commission 168 + 168 + 40 = 376, shares 152 / 112 / 112, nothing left.
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    let stock = mandi_services::bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    mandi_services::bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: stock.rows[0].lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let p = preview(&w, "");
    assert_eq!(p.from_date, "2026-10-11");
    let s = &p.summary;
    assert_eq!(
        (
            s.bikri_count,
            s.gross_paisa.rupees(),
            s.commission_paisa.rupees(),
            s.shared_income_paisa.rupees(),
            s.open_lots,
            s.open_qty
        ),
        (1, 2_800, 376, 376, 0, 0)
    );
    assert_eq!(
        shares(&p),
        vec![(152, 0, 152), (112, 0, 112), (112, 0, 112)]
    );
}

/// Writes a fixture file (when `MANDI_WRITE_FIXTURES` is set) or checks that it holds exactly
/// this text, like `day_money_fixtures.rs`: a fixture can never go stale.
fn write_or_check(file: &str, text: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(file);
    if std::env::var_os("MANDI_WRITE_FIXTURES").is_some() {
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with MANDI_WRITE_FIXTURES=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the real answer any more. If the change is on purpose, run with \
         MANDI_WRITE_FIXTURES=1, read the diff and tell frontend."
    );
}

/// The day close summary of the WHOLE worked day for the frontend (PRD 4.9, FR-21.1): the request
/// and the real answer of `day_close_preview` after Events 1 to 5. The numbers are the PRD's and
/// are asserted here before the file is checked.
#[test]
fn the_whole_day_close_preview_fixture_is_the_real_answer() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    let request = serde_json::json!({ "up_to": "" });
    let answer = day::close_preview(
        &w.env.ctx,
        &serde_json::from_value::<DayClosePreviewInput>(request.clone()).unwrap(),
    )
    .unwrap();
    let s = &answer.summary;
    assert_eq!(
        (
            s.bikri_count,
            s.gross_paisa.rupees(),
            s.commission_paisa.rupees(),
            s.shared_income_paisa.rupees(),
            s.open_lots,
            s.open_qty
        ),
        (2, 41_400, 5_448, 5_488, 0, 0)
    );
    assert_eq!(
        shares(&answer),
        vec![(2_196, 1_200, 996), (1_646, 900, 746), (1_646, 900, 746)]
    );
    assert_eq!(answer.cash_in_hand.amount_paisa.rupees(), 58_484);
    assert_eq!(
        (
            answer.trial_dr_paisa.rupees(),
            answer.trial_cr_paisa.rupees(),
            answer.trial_difference.amount_paisa.rupees()
        ),
        (85_048, 85_048, 0)
    );
    write_or_check(
        "day_close_preview_summary_worked_day_request.json",
        &(serde_json::to_string_pretty(&request).unwrap() + "\n"),
    );
    write_or_check(
        "day_close_preview_summary_worked_day.json",
        &(serde_json::to_string_pretty(&answer).unwrap() + "\n"),
    );
}

/// Money agent, plan 17, finding 4: the lots left are worked as of the day being closed (from the
/// stock moves dated up to that day), not from today's counts. E2E-16: 5 نگ were left on 10-10-26
/// and sold on 11-10-26; the preview of 10-10-26, made after that sale, still shows the 5.
#[test]
fn the_stock_left_is_the_stock_of_the_day_being_closed_even_after_a_later_sale() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save_ok(&w, &draft, false);
    set_today(&w.env, 2026, 10, 11);
    let stock = mandi_services::bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    mandi_services::bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: stock.rows[0].lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // Neither day is closed yet. The counts on the lot say 0 left today, but on 10-10-26 it was 5.
    let on_10 = preview(&w, "10-10-2026");
    assert_eq!((on_10.summary.open_lots, on_10.summary.open_qty), (1, 5));
    // Up to 11-10-26 the last 5 are sold.
    let on_11 = preview(&w, "11-10-2026");
    assert_eq!((on_11.summary.open_lots, on_11.summary.open_qty), (0, 0));
}

#[test]
fn a_range_of_days_adds_up_and_a_day_without_sales_is_empty() {
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    set_today(&w.env, 2026, 10, 12);
    // 10-10 to 12-10: the three days together are the one sale; 11-10 and 12-10 are empty.
    let all = preview(&w, "12-10-2026");
    assert_eq!(all.days, 3);
    assert_eq!(
        (
            all.summary.bikri_count,
            all.summary.gross_paisa.rupees(),
            all.summary.shared_income_paisa.rupees()
        ),
        (1, 29_400, 3_948)
    );
    // A closed-day preview with nothing to close shows the empty summary of today's numbers.
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "12-10-2026".into(),
        },
    )
    .unwrap();
    let none = preview(&w, "");
    assert_eq!(none.days, 0);
    assert_eq!(none.summary.bikri_count, 0);
    assert_eq!(none.summary.gross_paisa.rupees(), 0);
}
