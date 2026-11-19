//! Plan 72 P8 (PRD 12.7, FR-12.46, FR-19.3, FR-19.10, FR-19.13, Code Standards B31, M13): the print
//! data and the Excel file of پچھلی بکری hold every row of today's section and of the window, grouped
//! by day with a heading for each, with the same filters, the same switch "show every بکری" and the
//! same figures as the screen. The data is the eight بکری of `past_bikri.rs`, saved on today =
//! 12-11-2026; the window is 05-11 to 11-11.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;
mod xlsx_support;

use common::set_today;
use mandi_reports::export::{BikriListExportInput, ExportProgress, export_bikri_list};
use mandi_reports::reports::{
    BikriListPrintData, BikriListPrintInput, PrintCell, bikri_list_print_data,
};
use mandi_services::ServiceError;
use mandi_services::bikri;
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{
    BikriListInput, BikriListWholeInput, BikriSaved, BikriStatus, CancelInput,
};
use mandi_services::core::error::ErrorCode;
use support::World;
use xlsx_support::Value;

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

/// Eight بکری; number 5 is cancelled and number 6 is edited (the rate goes to 610).
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

fn window() -> BikriListWholeInput {
    BikriListWholeInput {
        from: "05-11-2026".into(),
        to: "11-11-2026".into(),
        with_today: true,
        seller_id: None,
        item_id: None,
        status: None,
        show_finished: false,
        search: None,
        all: false,
    }
}

fn print(w: &World, list: BikriListWholeInput, language: Language) -> BikriListPrintData {
    let input = BikriListPrintInput {
        list,
        language,
        paper: None,
    };
    let whole = bikri::list_whole(&w.env.ctx, &input.list).unwrap();
    bikri_list_print_data(w.env.ctx.db(), &whole, &input).unwrap()
}

fn text_of(cell: &PrintCell) -> String {
    match cell {
        PrintCell::Text { text }
        | PrintCell::Number { text }
        | PrintCell::Money { text }
        | PrintCell::Date { text }
        | PrintCell::Names { text } => text.clone(),
        PrintCell::Label { id } | PrintCell::Message { id, .. } => id.clone(),
        PrintCell::Empty => String::new(),
        other => panic!("unexpected cell {other:?}"),
    }
}

fn numbers_of(day: &mandi_reports::reports::BikriListPrintDay) -> Vec<String> {
    day.rows
        .iter()
        .map(|r| text_of(r.cells.first().unwrap()))
        .collect()
}

#[test]
fn the_print_holds_today_and_every_day_of_the_window_newest_first_with_empty_days() {
    let w = world_of_days();
    let data = print(&w, window(), Language::En);
    assert_eq!(data.title_id, "bikri-list-print-title");
    assert_eq!(
        data.columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        [
            "number",
            "date",
            "time",
            "truck",
            "sellers",
            "items",
            "qty_sold",
            "qty_received",
            "gross",
            "net",
            "buyer_total",
            "status"
        ]
    );
    // Today's section: the two بکری of 12-11, newest first.
    let today = data.today.as_ref().unwrap();
    assert_eq!(
        (today.date.as_str(), today.date_iso.as_str()),
        ("12-11-2026", "2026-11-12")
    );
    assert_eq!(today.count, "2");
    assert_eq!(numbers_of(today), ["8", "7"]);
    // 2,428 + 2,432 (4 sold at 600 + n).
    assert_eq!(today.gross, "4,860");
    // The window: 7 days, newest first; 08-11 holds the cancelled one, hidden by default.
    let dates: Vec<&str> = data.days.iter().map(|d| d.date.as_str()).collect();
    assert_eq!(
        dates,
        [
            "11-11-2026",
            "10-11-2026",
            "09-11-2026",
            "08-11-2026",
            "07-11-2026",
            "06-11-2026",
            "05-11-2026"
        ]
    );
    let counts: Vec<&str> = data.days.iter().map(|d| d.count.as_str()).collect();
    assert_eq!(counts, ["1", "0", "0", "0", "0", "0", "2"]);
    // The edit moved number 6 to 4 x 610 = 2,440; 05-11 is 2,412 + 2,416.
    assert_eq!(data.days.first().unwrap().gross, "2,440");
    assert_eq!(data.days.last().unwrap().gross, "4,828");
    assert!(data.days.get(1).unwrap().rows.is_empty());
    assert_eq!(numbers_of(data.days.last().unwrap()), ["4", "3"]);
    // The totals row is the stored sums over the whole filter: 12 sold, 30 received, 7,268.
    // Every بکری the print holds: 3 of the window and 2 of today.
    assert_eq!(data.row_count, "5");
    let totals: Vec<String> = data.totals.iter().map(text_of).collect();
    assert_eq!(&totals[6..9], ["12", "30", "7,268"]);
    assert_eq!(data.totals_label_id, "report-total");
    // The facts under the title: the window.
    assert_eq!(
        data.filters
            .iter()
            .map(|f| (f.label_id.as_str(), f.value.as_str()))
            .collect::<Vec<_>>(),
        [
            ("report-filter-from", "05-11-2026"),
            ("report-filter-to", "11-11-2026")
        ]
    );
}

#[test]
fn a_row_has_the_old_truck_the_lot_names_the_time_and_the_edited_mark() {
    let w = world_of_days();
    let data = print(&w, window(), Language::En);
    let six = &data.days.first().unwrap().rows.first().unwrap().cells;
    let texts: Vec<String> = six.iter().map(text_of).collect();
    assert_eq!(texts[0], "6");
    assert_eq!(texts[1], "11-11-2026");
    // The time is Pakistan time, `HH:MM`, digits 0 to 9.
    assert_eq!(texts[2].len(), 5);
    assert!(texts[2].chars().all(|c| c.is_ascii_digit() || c == ':'));
    assert_eq!(texts[3], "TRK-6");
    assert_eq!(texts[4], "Rahim");
    // رحیم's third banana lot (PRD FR-12.45).
    assert_eq!(texts[5], "Banana-3");
    assert_eq!(
        &texts[6..11],
        ["4", "10", "2,440", texts[9].as_str(), texts[10].as_str()]
    );
    // Edited: the status is the message with the status as its argument (PRD FR-7.71).
    assert!(
        matches!(six.get(11), Some(PrintCell::Message { id, .. }) if id == "bikri-list-status-edited")
    );
    let four = &data.days.last().unwrap().rows.first().unwrap().cells;
    assert!(matches!(four.get(11), Some(PrintCell::Label { id }) if id == "bikri-status-saved"));
}

#[test]
fn urdu_prints_urdu_names_with_digits_0_to_9() {
    let w = world_of_days();
    let data = print(&w, window(), Language::Ur);
    let six = &data.days.first().unwrap().rows.first().unwrap().cells;
    let texts: Vec<String> = six.iter().map(text_of).collect();
    assert_eq!(texts[4], "رحیم");
    assert_eq!(texts[5], "کیلا-3");
    // No Urdu or Arabic-Indic digit anywhere in the data (PRD FR-7.32).
    let json = serde_json::to_string(&data).unwrap();
    assert!(
        !json
            .chars()
            .any(|c| ('\u{06F0}'..='\u{06F9}').contains(&c)
                || ('\u{0660}'..='\u{0669}').contains(&c))
    );
    assert_eq!(data.dir, mandi_reports::print::TextDir::Rtl);
}

#[test]
fn the_filters_and_the_switch_show_everything_follow_the_screen() {
    let w = world_of_days();
    // A seller filter: رحیم has 6 and 3 in the window.
    let rahim = print(
        &w,
        BikriListWholeInput {
            seller_id: Some(w.rahim),
            ..window()
        },
        Language::En,
    );
    let counts: Vec<&str> = rahim.days.iter().map(|d| d.count.as_str()).collect();
    assert_eq!(counts, ["1", "0", "0", "0", "0", "0", "1"]);
    assert!(
        rahim
            .filters
            .iter()
            .any(|f| f.label_id == "report-filter-seller" && f.value.contains("Rahim"))
    );
    // The switch sets the filters aside: every بکری of the days, the cancelled one too, and the
    // header says so instead of naming the filters.
    let all = print(
        &w,
        BikriListWholeInput {
            all: true,
            seller_id: Some(w.karim),
            status: Some(BikriStatus::Saved),
            search: Some("nothing".into()),
            ..window()
        },
        Language::En,
    );
    assert_eq!(all.row_count, "6");
    let eight = all.days.get(3).unwrap();
    assert_eq!(
        (eight.date.as_str(), eight.count.as_str()),
        ("08-11-2026", "1")
    );
    assert!(matches!(
        eight.rows.first().unwrap().cells.get(11),
        Some(PrintCell::Label { id }) if id == "bikri-status-cancelled"
    ));
    let labels: Vec<&str> = all.filters.iter().map(|f| f.label_id.as_str()).collect();
    assert_eq!(
        labels,
        [
            "report-filter-from",
            "report-filter-to",
            "bikri-list-print-all"
        ]
    );
    // The totals still leave the cancelled بکری out of the money.
    assert_eq!(all.totals.iter().map(text_of).nth(8).unwrap(), "7,268");
    // The same data as the screen: its list of the same window says the same.
    let screen = bikri::list(
        &w.env.ctx,
        &BikriListInput {
            from: Some("05-11-2026".into()),
            to: Some("11-11-2026".into()),
            all: true,
            group_by_day: true,
            ..BikriListInput::default()
        },
    )
    .unwrap();
    assert_eq!(screen.total_count, 4);
    assert_eq!(
        screen
            .days
            .iter()
            .map(|d| d.count.to_string())
            .collect::<Vec<_>>(),
        all.days.iter().map(|d| d.count.clone()).collect::<Vec<_>>()
    );
}

#[test]
fn without_today_there_is_no_section_and_a_window_that_reaches_today_is_refused() {
    let w = world_of_days();
    let data = print(
        &w,
        BikriListWholeInput {
            with_today: false,
            ..window()
        },
        Language::En,
    );
    assert!(data.today.is_none());
    // Today's section and a window that ends today would print today twice.
    let error = bikri::list_whole(
        &w.env.ctx,
        &BikriListWholeInput {
            to: "12-11-2026".into(),
            ..window()
        },
    )
    .unwrap_err();
    match error {
        ServiceError::Rejected(items) => {
            let first = items.first().unwrap();
            assert_eq!(first.code, ErrorCode::ErrDateRange);
            assert_eq!(first.field.as_deref(), Some("to"));
        }
        other => panic!("{other:?}"),
    }
    // Without today's section it is fine.
    assert!(
        bikri::list_whole(
            &w.env.ctx,
            &BikriListWholeInput {
                to: "12-11-2026".into(),
                with_today: false,
                ..window()
            }
        )
        .is_ok()
    );
    // A date after today, and a missing end.
    for (from, to, code, field) in [
        ("05-11-2026", "13-11-2026", ErrorCode::ErrDateInFuture, "to"),
        ("", "11-11-2026", ErrorCode::ErrRequired, "from"),
    ] {
        let error = bikri::list_whole(
            &w.env.ctx,
            &BikriListWholeInput {
                from: from.into(),
                to: to.into(),
                ..window()
            },
        )
        .unwrap_err();
        match error {
            ServiceError::Rejected(items) => {
                let first = items.first().unwrap();
                assert_eq!((first.code, first.field.as_deref()), (code, Some(field)));
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn a_long_window_is_read_whole_across_pages_in_one_snapshot() {
    // 7 بکری on one day, read 2 rows at a time: every number once, newest first, and the count, the
    // totals and the day heading are those of the whole filter. The answer is the same as the
    // default page of 500 rows.
    let w = support::world();
    set_today(&w.env, 2026, 11, 12);
    for n in 1..=7 {
        bikri_on(&w, n, "10-11-2026", w.rahim, w.banana);
    }
    let small = bikri::list_whole_paged(&w.env.ctx, &window(), 2).unwrap();
    let numbers: Vec<i64> = small.window.rows.iter().map(|r| r.number).collect();
    assert_eq!(numbers, [7, 6, 5, 4, 3, 2, 1]);
    assert_eq!(small.window.total_count, 7);
    let day = small
        .window
        .days
        .iter()
        .find(|d| d.date == "2026-11-10")
        .unwrap();
    assert_eq!(day.count, 7);
    assert_eq!(bikri::list_whole(&w.env.ctx, &window()).unwrap(), small);
}

#[test]
fn the_excel_file_has_every_row_a_heading_for_each_day_and_the_totals() {
    let w = world_of_days();
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("past.xlsx");
    let input = BikriListExportInput {
        list: window(),
        language: Language::En,
        path: path.display().to_string(),
    };
    let whole = bikri::list_whole(&w.env.ctx, &input.list).unwrap();
    let mut events: Vec<ExportProgress> = Vec::new();
    let made = export_bikri_list(w.env.ctx.db(), &whole, &input, &mut |p| events.push(p)).unwrap();
    // 2 rows of today and 3 of the window.
    assert_eq!(made.rows, 5);
    assert!(made.size_bytes > 0);
    assert!(events.iter().any(|e| format!("{:?}", e.stage) == "Done"));
    let sheet = xlsx_support::read(&path);
    assert!(!sheet.right_to_left);
    assert_eq!(sheet.formulas, 0, "a sheet holds values, no formulas");
    // The money of each day heading, the rows and the totals are numbers (Code Standards M13).
    let nums: Vec<String> = (0..=sheet.last_row())
        .flat_map(|r| (0..12).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            Value::Num(text) => Some(text.clone()),
            _ => None,
        })
        .collect();
    // 2,440 (11-11) and 4,828 (05-11) are day headings; 2,412 and 2,416 are rows; 7,268 is the total.
    for expected in [
        "4860", "2428", "2432", "2440", "4828", "2412", "2416", "7268",
    ] {
        assert!(nums.iter().any(|n| n == expected), "{expected} in {nums:?}");
    }
    // The truck is text, the rows hold their numbers.
    assert!(sheet.all_texts().iter().any(|t| t == "TRK-3"));
}
