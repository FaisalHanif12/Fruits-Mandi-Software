//! Plan 72 T1 (PRD 12.7 پچھلی بکری, FR-19.11, NFR-SPD-24, Code Standards B31): `bikri_list` reads a
//! window of days with one call: the page of rows, the count and the totals over the whole filter,
//! and with `group_by_day` one heading for each date of the window (empty days too), each with its
//! count and خام بکری. `all` sets the filters aside (P8). A date after today is refused (P6).
//!
//! The data is eight بکری, saved on today = 12-11-2026 (each has one lot of 10 نگ and one line of
//! 4 sold to اکرم at 600 + n):
//!
//! | n | day | seller | item | later |
//! |---|---|---|---|---|
//! | 1 | 29-10 | رحیم | کیلا | |
//! | 2 | 02-11 | کریم | سیب | |
//! | 3 | 05-11 | رحیم | سیب | |
//! | 4 | 05-11 | کریم | کیلا | |
//! | 5 | 08-11 | رحیم | کیلا | cancelled |
//! | 6 | 11-11 | رحیم | کیلا | edited (the rate goes to 610) |
//! | 7 | 12-11 | کریم | سیب | today |
//! | 8 | 12-11 | رحیم | کیلا | today |
//!
//! The first window ends yesterday: 05-11 to 11-11. One step back is 29-10 to 04-11.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use common::set_today;
use mandi_reports::reports::{
    Cell, ReportFilters, ReportId, ReportPage, ReportRunInput, run as run_report,
};
use mandi_services::ServiceError;
use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriDay, BikriFindInput, BikriList, BikriListInput, BikriListRow, BikriSaved, BikriStatus,
    CancelInput, LaterSaleSaveInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::money::Paisa;
use mandi_services::core::paging::{SortDirection, SortInput};
use support::World;

/// A بکری on a day: 10 نگ from `seller`, 4 sold to اکرم at `600 + n`. Truck `TRK-<n>`.
fn bikri_on(w: &World, n: i64, date: &str, seller: i64, item: i64) -> BikriSaved {
    let lot = support::with_sales(
        support::bare_lot(w, "lot-1", seller, item, "10"),
        vec![support::sale("s-1", w.akram, "4", &format!("{}", 600 + n))],
    );
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = date.into();
    draft.header.truck_no = Some(format!("TRK-{n}"));
    support::save_old(w, &draft, false)
}

fn world_of_days() -> World {
    let w = support::world();
    set_today(&w.env, 2026, 11, 12);
    let (rahim, karim, banana, apple) = (w.rahim, w.karim, w.banana, w.apple);
    bikri_on(&w, 1, "29-10-2026", rahim, banana);
    bikri_on(&w, 2, "02-11-2026", karim, apple);
    bikri_on(&w, 3, "05-11-2026", rahim, apple);
    bikri_on(&w, 4, "05-11-2026", karim, banana);
    let five = bikri_on(&w, 5, "08-11-2026", rahim, banana);
    let six = bikri_on(&w, 6, "11-11-2026", rahim, banana);
    bikri_on(&w, 7, "12-11-2026", karim, apple);
    bikri_on(&w, 8, "12-11-2026", rahim, banana);
    // Number 5 is cancelled and number 6 is edited (PRD FR-7.71).
    let detail = bikri::get(&w.env.ctx, five.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: five.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let opened = tools::open(&w, six.bikri_id, six.entry_id);
    let mut draft = opened.draft.clone();
    draft
        .lots
        .first_mut()
        .unwrap()
        .sales
        .first_mut()
        .unwrap()
        .seller_rate = "610".into();
    tools::edit(&w, &opened, &draft);
    w
}

fn window(from: &str, to: &str) -> BikriListInput {
    BikriListInput {
        from: Some(from.into()),
        to: Some(to.into()),
        group_by_day: true,
        ..BikriListInput::default()
    }
}

fn list(w: &World, input: &BikriListInput) -> BikriList {
    bikri::list(&w.env.ctx, input).unwrap()
}

fn numbers(rows: &[BikriListRow]) -> Vec<i64> {
    rows.iter().map(|r| r.number).collect()
}

/// `(date, count)` of each day, in the order sent.
fn day_counts(days: &[BikriDay]) -> Vec<(&str, i64)> {
    days.iter().map(|d| (d.date.as_str(), d.count)).collect()
}

fn refused(error: &ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => {
            let first = items.first().unwrap();
            (first.code, first.field.clone())
        }
        other => panic!("not a rejection: {other:?}"),
    }
}

/// The paisa of a money cell, or 0 for an empty one.
fn paisa_of(cell: Option<&Cell>) -> Paisa {
    match cell {
        Some(Cell::Money { paisa }) => *paisa,
        Some(Cell::Empty) | None => Paisa::ZERO,
        other => panic!("not money: {other:?}"),
    }
}

fn count_of(cell: Option<&Cell>) -> i64 {
    match cell {
        Some(Cell::Number { value }) => *value,
        Some(Cell::Empty) | None => 0,
        other => panic!("not a number: {other:?}"),
    }
}

fn r2(w: &World, filters: ReportFilters) -> ReportPage {
    run_report(
        w.env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R2,
            filters,
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap()
}

// ---- the window and its days -------------------------------------------------------------

#[test]
fn the_first_window_ends_yesterday_and_one_step_back_goes_seven_days_earlier() {
    let w = world_of_days();
    // Today 12-11-2026: the window is 05-11 to 11-11, newest day first, empty days included.
    let first = list(&w, &window("05-11-2026", "11-11-2026"));
    assert_eq!(numbers(&first.rows), [6, 4, 3]);
    assert_eq!(first.total_count, 3);
    assert_eq!(
        day_counts(&first.days),
        [
            ("2026-11-11", 1),
            ("2026-11-10", 0),
            ("2026-11-09", 0),
            ("2026-11-08", 0), // number 5 is cancelled and hidden
            ("2026-11-07", 0),
            ("2026-11-06", 0),
            ("2026-11-05", 2),
        ]
    );
    // One step back: 29-10 to 04-11.
    let back = list(&w, &window("29-10-2026", "04-11-2026"));
    assert_eq!(numbers(&back.rows), [2, 1]);
    assert_eq!(
        day_counts(&back.days),
        [
            ("2026-11-04", 0),
            ("2026-11-03", 0),
            ("2026-11-02", 1),
            ("2026-11-01", 0),
            ("2026-10-31", 0),
            ("2026-10-30", 0),
            ("2026-10-29", 1),
        ]
    );
    // Today has its own section: one day, two بکری.
    let today = list(&w, &window("12-11-2026", "12-11-2026"));
    assert_eq!(numbers(&today.rows), [8, 7]);
    assert_eq!(day_counts(&today.days), [("2026-11-12", 2)]);
    // The window ending on a date typed in the date field (P6): 7 days up to 05-11.
    let jump = list(&w, &window("30-10-2026", "05-11-2026"));
    assert_eq!(jump.days.len(), 7);
    assert_eq!(numbers(&jump.rows), [4, 3, 2]);
}

#[test]
fn a_day_has_its_count_and_its_gross_from_the_stored_lots() {
    let w = world_of_days();
    let first = list(&w, &window("05-11-2026", "11-11-2026"));
    // Each بکری sold 4 at 600 + n: gross 4 x 603 = 2,412 and 4 x 604 = 2,416 on 05-11; the edit
    // moved number 6 to 4 x 610 = 2,440 on 11-11.
    let gross = |date: &str| {
        first
            .days
            .iter()
            .find(|d| d.date == date)
            .map(|d| d.gross_paisa)
            .unwrap()
    };
    assert_eq!(
        gross("2026-11-05"),
        Paisa::from_rupees(2_412 + 2_416).unwrap()
    );
    assert_eq!(gross("2026-11-11"), Paisa::from_rupees(2_440).unwrap());
    // An empty day has 0 and 0.
    assert_eq!(gross("2026-11-10"), Paisa::ZERO);
    // The days add up to the count and the gross of the whole filter.
    let days_count: i64 = first.days.iter().map(|d| d.count).sum();
    let days_gross = first
        .days
        .iter()
        .try_fold(Paisa::ZERO, |sum, d| sum.checked_add(d.gross_paisa))
        .unwrap();
    assert_eq!(days_count, first.total_count);
    assert_eq!(days_gross, first.totals.gross);
}

#[test]
fn the_days_follow_show_finished_a_cancelled_bikri_counts_but_has_no_gross() {
    let w = world_of_days();
    let shown = list(
        &w,
        &BikriListInput {
            show_finished: true,
            ..window("05-11-2026", "11-11-2026")
        },
    );
    let day = shown.days.iter().find(|d| d.date == "2026-11-08").unwrap();
    // Number 5 is cancelled: it counts on its day, and its gross is out of the day and the totals.
    assert_eq!((day.count, day.gross_paisa), (1, Paisa::ZERO));
    let days_count: i64 = shown.days.iter().map(|d| d.count).sum();
    assert_eq!(days_count, shown.total_count);
    // Only the cancelled ones: the gross of every day is 0, like the totals.
    let cancelled = list(
        &w,
        &BikriListInput {
            status: Some(BikriStatus::Cancelled),
            ..window("05-11-2026", "11-11-2026")
        },
    );
    assert_eq!(
        day_counts(&cancelled.days).iter().map(|d| d.1).sum::<i64>(),
        1
    );
    assert!(cancelled.days.iter().all(|d| d.gross_paisa == Paisa::ZERO));
    assert_eq!(cancelled.totals.gross, Paisa::ZERO);
}

#[test]
fn a_later_sale_counts_under_the_day_of_its_bikri_so_a_heading_equals_its_rows() {
    let w = world_of_days();
    // Number 3 (05-11) has 6 نگ left. On today, 12-11, 3 of them are sold to اکرم at 700.
    let rows = list(&w, &window("05-11-2026", "11-11-2026")).rows;
    let three = rows.iter().find(|r| r.number == 3).unwrap();
    let detail = bikri::get(&w.env.ctx, three.id).unwrap();
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: detail.lots.first().unwrap().id,
            date: "12-11-2026".into(),
            sales: vec![support::sale("later", w.akram, "3", "700")],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // The window: the later sale's 3 x 700 = 2,100 is in the خام بکری of number 3 and of its day, 05-11.
    let first = list(&w, &window("05-11-2026", "11-11-2026"));
    let day = first.days.iter().find(|d| d.date == "2026-11-05").unwrap();
    assert_eq!(
        day.gross_paisa,
        Paisa::from_rupees(2_412 + 2_100 + 2_416).unwrap()
    );
    let row_sum = first
        .rows
        .iter()
        .filter(|r| r.entry_date == "2026-11-05")
        .try_fold(Paisa::ZERO, |sum, r| sum.checked_add(r.gross))
        .unwrap();
    assert_eq!(row_sum, day.gross_paisa, "a heading is the sum of its rows");
    // Today's section holds only the بکری dated today: 2,428 + 2,432.
    let today = list(&w, &window("12-11-2026", "12-11-2026"));
    assert_eq!(
        today.totals.gross,
        Paisa::from_rupees(2_428 + 2_432).unwrap()
    );
    // R2 reads the postings of each day, so it differs from the list by exactly the later sale.
    let r2_gross = |from: &str, to: &str| {
        paisa_of(
            r2(
                &w,
                ReportFilters {
                    from: from.into(),
                    to: to.into(),
                    ..ReportFilters::default()
                },
            )
            .totals
            .get(7),
        )
    };
    assert_eq!(
        r2_gross("05-11-2026", "11-11-2026"),
        Paisa::from_rupees(2_412 + 2_416 + 2_440).unwrap(),
        "the window of R2 leaves the later sale out"
    );
    assert_eq!(
        r2_gross("12-11-2026", "12-11-2026"),
        Paisa::from_rupees(2_428 + 2_432 + 2_100).unwrap(),
        "the day of R2 holds it"
    );
}

#[test]
fn a_window_with_no_bikri_still_has_every_day_with_zero() {
    let w = world_of_days();
    let empty = list(&w, &window("13-10-2026", "19-10-2026"));
    assert!(empty.rows.is_empty());
    assert_eq!(empty.total_count, 0);
    assert_eq!(empty.days.len(), 7);
    assert!(
        empty
            .days
            .iter()
            .all(|d| d.count == 0 && d.gross_paisa == Paisa::ZERO)
    );
    assert_eq!(empty.totals.gross, Paisa::ZERO);
    // Newest day first.
    assert_eq!(
        empty.days.first().map(|d| d.date.as_str()),
        Some("2026-10-19")
    );
}

#[test]
fn days_come_only_when_asked_and_a_single_date_is_a_window_of_one_day() {
    let w = world_of_days();
    let plain = bikri::list(
        &w.env.ctx,
        &BikriListInput {
            from: Some("05-11-2026".into()),
            to: Some("11-11-2026".into()),
            ..BikriListInput::default()
        },
    )
    .unwrap();
    assert!(plain.days.is_empty());
    assert_eq!(numbers(&plain.rows), [6, 4, 3]);
    let one = list(
        &w,
        &BikriListInput {
            date: Some("05-11-2026".into()),
            group_by_day: true,
            ..BikriListInput::default()
        },
    );
    assert_eq!(day_counts(&one.days), [("2026-11-05", 2)]);
    assert_eq!(numbers(&one.rows), [4, 3]);
}

#[test]
fn sorted_by_day_oldest_first_the_days_follow() {
    let w = world_of_days();
    let oldest = list(
        &w,
        &BikriListInput {
            sort: Some(SortInput {
                language: None,
                column: "entry_date".into(),
                direction: SortDirection::Asc,
            }),
            ..window("05-11-2026", "11-11-2026")
        },
    );
    assert_eq!(numbers(&oldest.rows), [3, 4, 6]);
    assert_eq!(
        oldest.days.first().map(|d| d.date.as_str()),
        Some("2026-11-05")
    );
    assert_eq!(
        oldest.days.last().map(|d| d.date.as_str()),
        Some("2026-11-11")
    );
}

// ---- the row ----------------------------------------------------------------------------

#[test]
fn a_row_carries_the_truck_the_lot_names_and_the_edited_and_cancelled_marks() {
    let w = world_of_days();
    let all = list(
        &w,
        &BikriListInput {
            all: true,
            ..window("05-11-2026", "11-11-2026")
        },
    );
    let row = |n: i64| all.rows.iter().find(|r| r.number == n).unwrap();
    // The truck of old data.
    assert_eq!(row(6).truck_no.as_deref(), Some("TRK-6"));
    // The lot name of plan 71: رحیم's third کیلا lot (numbers 1, 5, 6 are his کیلا بکری so far).
    let lot = row(6).lots.first().unwrap();
    assert_eq!(
        (lot.seller.id, lot.item.id, lot.counter),
        (w.rahim, w.banana, 3)
    );
    // The edit stamp of FR-7.71 is on number 6 only.
    assert!(row(6).flags.edited.is_some());
    assert!(row(4).flags.edited.is_none());
    // Number 5 is cancelled: its status says so and its buttons are locked.
    assert_eq!(row(5).status, BikriStatus::Cancelled);
    assert!(!row(5).flags.can_edit);
}

// ---- the filters, and "show everything" --------------------------------------------------

#[test]
fn the_filters_decide_the_rows_the_count_and_the_days() {
    let w = world_of_days();
    let by = |input: BikriListInput| list(&w, &input);
    let base = || window("05-11-2026", "11-11-2026");
    // A seller: رحیم has 3 and 6 in the window (5 is cancelled and hidden).
    let rahim = by(BikriListInput {
        seller_id: Some(w.rahim),
        ..base()
    });
    assert_eq!(numbers(&rahim.rows), [6, 3]);
    assert_eq!(rahim.total_count, 2);
    assert_eq!(
        rahim
            .days
            .iter()
            .filter(|d| d.count > 0)
            .map(|d| (d.date.as_str(), d.count))
            .collect::<Vec<_>>(),
        [("2026-11-11", 1), ("2026-11-05", 1)]
    );
    // An item: سیب only on number 3.
    let apple = by(BikriListInput {
        item_id: Some(w.apple),
        ..base()
    });
    assert_eq!(numbers(&apple.rows), [3]);
    // A number, and the start of a truck number.
    let number = by(BikriListInput {
        search: Some("4".into()),
        ..base()
    });
    assert_eq!(numbers(&number.rows), [4]);
    let truck = by(BikriListInput {
        search: Some("TRK-6".into()),
        ..base()
    });
    assert_eq!(numbers(&truck.rows), [6]);
    // A status, and "show finished".
    let cancelled = by(BikriListInput {
        status: Some(BikriStatus::Cancelled),
        ..base()
    });
    assert_eq!(numbers(&cancelled.rows), [5]);
    let finished = by(BikriListInput {
        show_finished: true,
        ..base()
    });
    assert_eq!(numbers(&finished.rows), [6, 5, 4, 3]);
    assert_eq!(finished.total_count, 4);
    // The totals leave a cancelled بکری out, whatever the count says.
    assert_eq!(finished.totals.gross, by(base()).totals.gross);
}

#[test]
fn all_sets_the_filters_aside_and_keeps_the_dates() {
    let w = world_of_days();
    // Filters that would match nothing, and a status: all of them are set aside.
    let everything = list(
        &w,
        &BikriListInput {
            all: true,
            seller_id: Some(w.karim),
            item_id: Some(w.apple),
            search: Some("nothing-like-this".into()),
            status: Some(BikriStatus::Cancelled),
            ..window("05-11-2026", "11-11-2026")
        },
    );
    assert_eq!(numbers(&everything.rows), [6, 5, 4, 3]);
    assert_eq!(everything.total_count, 4);
    assert_eq!(
        everything
            .days
            .iter()
            .find(|d| d.date == "2026-11-08")
            .map(|d| d.count),
        Some(1),
        "the cancelled بکری of 08-11 counts in its day"
    );
    // The dates still decide: one step back shows only 2 and 1.
    let back = list(
        &w,
        &BikriListInput {
            all: true,
            seller_id: Some(w.karim),
            ..window("29-10-2026", "04-11-2026")
        },
    );
    assert_eq!(numbers(&back.rows), [2, 1]);
    // Today's section follows the same choice.
    let today = list(
        &w,
        &BikriListInput {
            all: true,
            item_id: Some(w.mooli),
            ..window("12-11-2026", "12-11-2026")
        },
    );
    assert_eq!(numbers(&today.rows), [8, 7]);
}

// ---- totals ------------------------------------------------------------------------------

#[test]
fn the_totals_are_over_the_whole_filter_and_equal_the_sum_of_the_pages_and_r2() {
    let w = world_of_days();
    let page = |offset: u32| {
        list(
            &w,
            &BikriListInput {
                all: false,
                show_finished: true,
                offset,
                limit: 2,
                ..window("05-11-2026", "11-11-2026")
            },
        )
    };
    let (one, two, three) = (page(0), page(2), page(4));
    assert_eq!(numbers(&one.rows), [6, 5]);
    assert_eq!(numbers(&two.rows), [4, 3]);
    assert!(three.rows.is_empty());
    // Every page says the same about the whole filter.
    for p in [&one, &two, &three] {
        assert_eq!(p.total_count, 4);
        assert_eq!(p.totals, one.totals);
        assert_eq!(p.days, one.days);
    }
    // The totals are the sums of the saved rows of all pages (a cancelled بکری is out).
    let rows: Vec<&BikriListRow> = one.rows.iter().chain(&two.rows).collect();
    let saved: Vec<&&BikriListRow> = rows
        .iter()
        .filter(|r| r.status == BikriStatus::Saved)
        .collect();
    let sum = |value: fn(&BikriListRow) -> Paisa| {
        saved
            .iter()
            .try_fold(Paisa::ZERO, |sum, r| sum.checked_add(value(r)))
            .unwrap()
    };
    assert_eq!(sum(|r| r.gross), one.totals.gross);
    assert_eq!(sum(|r| r.net), one.totals.net);
    assert_eq!(sum(|r| r.buyer_total), one.totals.buyer_total);
    assert_eq!(
        saved.iter().map(|r| r.qty_sold).sum::<i64>(),
        one.totals.qty_sold
    );
    assert_eq!(
        saved.iter().map(|r| r.qty_received).sum::<i64>(),
        one.totals.qty_received
    );
    // R2 for the same dates reads the same bills: value = خام بکری, buyer total, نگ sold.
    let report = r2(
        &w,
        ReportFilters {
            from: "05-11-2026".into(),
            to: "11-11-2026".into(),
            ..ReportFilters::default()
        },
    );
    assert_eq!(paisa_of(report.totals.get(7)), one.totals.gross);
    assert_eq!(paisa_of(report.totals.get(8)), one.totals.buyer_total);
    assert_eq!(count_of(report.totals.get(5)), one.totals.qty_sold);
    // And for one seller.
    let rahim = list(
        &w,
        &BikriListInput {
            seller_id: Some(w.rahim),
            ..window("05-11-2026", "11-11-2026")
        },
    );
    let report = r2(
        &w,
        ReportFilters {
            from: "05-11-2026".into(),
            to: "11-11-2026".into(),
            seller_id: Some(w.rahim),
            ..ReportFilters::default()
        },
    );
    assert_eq!(paisa_of(report.totals.get(7)), rahim.totals.gross);
    assert_eq!(paisa_of(report.totals.get(8)), rahim.totals.buyer_total);
}

// ---- refusals ----------------------------------------------------------------------------

#[test]
fn a_date_after_today_is_refused() {
    let w = world_of_days();
    for (field, input) in [
        (
            "to",
            BikriListInput {
                from: Some("05-11-2026".into()),
                to: Some("13-11-2026".into()),
                group_by_day: true,
                ..BikriListInput::default()
            },
        ),
        (
            "from",
            BikriListInput {
                from: Some("13-11-2026".into()),
                to: Some("13-11-2026".into()),
                ..BikriListInput::default()
            },
        ),
        (
            "date",
            BikriListInput {
                date: Some("13-11-2026".into()),
                ..BikriListInput::default()
            },
        ),
    ] {
        let error = bikri::list(&w.env.ctx, &input).unwrap_err();
        let (code, at) = refused(&error);
        assert_eq!(code, ErrorCode::ErrDateInFuture, "{field}");
        // `from` and `to` are both after today in the second input: the first problem is `from`.
        assert_eq!(at.as_deref(), Some(field));
        // The error says which day today is, so the screen can offer it.
        if let ServiceError::Rejected(items) = &error {
            assert_eq!(
                items.first().and_then(|i| i.values.get("today")),
                Some(&ErrorValue::Text("12-11-2026".into()))
            );
        }
    }
    // Today itself is fine.
    assert!(
        bikri::list(&w.env.ctx, &window("12-11-2026", "12-11-2026")).is_ok(),
        "today"
    );
}

#[test]
fn typed_text_that_is_too_long_is_refused_with_a_code_never_as_a_database_error() {
    let w = world_of_days();
    let most = mandi_services::search::MAX_QUERY_CHARS;
    // The list: the search text.
    let search = |text: String| {
        bikri::list(
            &w.env.ctx,
            &BikriListInput {
                search: Some(text),
                ..window("05-11-2026", "11-11-2026")
            },
        )
    };
    assert!(search("L".repeat(most)).is_ok());
    for len in [most + 1, 50_000, 1_000_000] {
        let error = search("L".repeat(len)).unwrap_err();
        assert_eq!(
            refused(&error),
            (ErrorCode::ErrTextTooLong, Some("search".into())),
            "{len} letters"
        );
    }
    // Set aside by "show everything" or not, the typed text is checked.
    let aside = bikri::list(
        &w.env.ctx,
        &BikriListInput {
            all: true,
            search: Some("L".repeat(most + 1)),
            ..window("05-11-2026", "11-11-2026")
        },
    );
    assert_eq!(refused(&aside.unwrap_err()).0, ErrorCode::ErrTextTooLong);
    // The find: the truck text and the name of a چکتا buyer.
    for (field, input) in [
        (
            "truck_no",
            BikriFindInput {
                truck_no: Some("T".repeat(most + 1)),
                ..BikriFindInput::default()
            },
        ),
        (
            "buyer_text",
            BikriFindInput {
                buyer_text: Some("T".repeat(most + 1)),
                ..BikriFindInput::default()
            },
        ),
    ] {
        let error = bikri::find(&w.env.ctx, &input).unwrap_err();
        assert_eq!(
            refused(&error),
            (ErrorCode::ErrTextTooLong, Some(field.into()))
        );
    }
    assert!(
        bikri::find(
            &w.env.ctx,
            &BikriFindInput {
                truck_no: Some("T".repeat(most)),
                ..BikriFindInput::default()
            }
        )
        .is_ok()
    );
}

#[test]
fn the_days_of_a_window_need_both_ends_in_order_and_at_most_a_year() {
    let w = world_of_days();
    let try_list = |from: Option<&str>, to: Option<&str>| {
        refused(
            &bikri::list(
                &w.env.ctx,
                &BikriListInput {
                    from: from.map(str::to_owned),
                    to: to.map(str::to_owned),
                    group_by_day: true,
                    ..BikriListInput::default()
                },
            )
            .unwrap_err(),
        )
    };
    assert_eq!(
        try_list(None, Some("11-11-2026")),
        (ErrorCode::ErrRequired, Some("from".into()))
    );
    assert_eq!(
        try_list(Some("05-11-2026"), None),
        (ErrorCode::ErrRequired, Some("to".into()))
    );
    assert_eq!(
        try_list(Some("11-11-2026"), Some("05-11-2026")),
        (ErrorCode::ErrDateRange, Some("to".into()))
    );
    // 366 days (12-11-2025 to 12-11-2026) are allowed; 367 are not.
    let year = list(&w, &window("12-11-2025", "12-11-2026"));
    assert_eq!(year.days.len(), 366);
    assert_eq!(
        try_list(Some("11-11-2025"), Some("12-11-2026")),
        (ErrorCode::ErrOutOfRange, Some("to".into()))
    );
    // Without `group_by_day` a long range is an ordinary list, as before.
    assert!(
        bikri::list(
            &w.env.ctx,
            &BikriListInput {
                from: Some("01-01-2020".into()),
                to: Some("12-11-2026".into()),
                ..BikriListInput::default()
            },
        )
        .is_ok()
    );
}
