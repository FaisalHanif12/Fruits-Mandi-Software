//! R5 "Print All": one slip for each buyer of the filter (PRD 19.2 R5, FR-19.10, FR-19.11, FR-13.5,
//! FR-12.44, E2E-36; plan d5-late, round 12, plan 65).
//!
//! The data is made the way `golden_day.rs` makes it (the real services; `reports_world`). The
//! expected numbers are worked by hand from the PRD and the money agent's review: every slip line
//! is a stored line value, the slips add up to the totals of R5 and of R2, and what was paid on the
//! spot and what is left add up to the paid total of R3 and the left total of R4. Since v1.32 a
//! buyer who paid in full is on a slip too, with 0 left (D8 question Q1).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::ReportsError;
use mandi_reports::reports::{
    BuyerBillsPrintData, Cell, PartyFilter, PrintCell, PrintDetailPart, ReportFilters, ReportId,
    ReportPrintInput, buyer_bills_print_data, print_data,
};
use mandi_services::bikri;
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{
    CancelInput, EditOpenInput, EditSaveInput, LaterSaleSaveInput, StockListInput,
};
use mandi_services::core::error::ErrorCode;

use reports_world::*;

fn input(filters: ReportFilters, language: Language) -> ReportPrintInput {
    ReportPrintInput {
        group_by_month: false,
        sort: None,
        id: ReportId::R5,
        filters,
        language,
        paper: None,
    }
}

fn slips_of(w: &support::World, filters: ReportFilters) -> BuyerBillsPrintData {
    buyer_bills_print_data(w.env.ctx.db(), &input(filters, Language::En)).unwrap()
}

fn day(from: &str, to: &str) -> ReportFilters {
    ReportFilters {
        from: from.into(),
        to: to.into(),
        ..ReportFilters::default()
    }
}

fn text(cell: &PrintCell) -> String {
    match cell {
        PrintCell::Empty => String::new(),
        PrintCell::Number { text } | PrintCell::Money { text } => text.clone(),
        other => panic!("not a number or money: {other:?}"),
    }
}

/// "name, value, total, qty" of every slip, for a quick look.
fn overview(data: &BuyerBillsPrintData) -> Vec<(String, String, String, String)> {
    data.slips
        .iter()
        .map(|s| {
            (
                s.party.name.clone(),
                text(&s.totals[2]),
                text(&s.totals[3]),
                s.qty.clone(),
            )
        })
        .collect()
}

fn row(name: &str, value: &str, total: &str, qty: &str) -> (String, String, String, String) {
    (name.into(), value.into(), total.into(), qty.into())
}

/// The sums of the slips against R5, R2, R3 and R4 of the same filter (the money agent's rule: grand
/// totals equal the R5 totals row, the value and buyer total of R2, the paid total of R3 and the left
/// total of R4).
fn assert_adds_up(w: &support::World, filters: &ReportFilters, data: &BuyerBillsPrintData) {
    let r5 = report(w, ReportId::R5, filters.clone(), 0, 0);
    let r2 = report(w, ReportId::R2, filters.clone(), 0, 0);
    let r3 = report(w, ReportId::R3, filters.clone(), 0, 0);
    let r4 = report(w, ReportId::R4, filters.clone(), 0, 0);
    // The totals of the slips are the totals row of R5: value, total, paid, left.
    for (slip_cell, r5_cell) in [(2, 3), (3, 4), (4, 5), (5, 6)] {
        assert_eq!(
            text(&data.totals[slip_cell]),
            format_rupees(money(&r5.totals[r5_cell])),
            "totals column {slip_cell}"
        );
    }
    // The quantity of all slips is the quantity total of R2, which sums `qty` in SQL. The columns of R2
    // are found by their key: a register gains a `seller_rate` column (and a `grade` or `packing` one)
    // only when the filter has a line that needs it (D7, plan 59), which moves the ones after it.
    let at = |page: &mandi_reports::reports::ReportPage, key: &str| {
        page.columns.iter().position(|c| c.key == key).unwrap()
    };
    assert_eq!(data.qty, number(&r2.totals[at(&r2, "qty")]).to_string());
    assert_eq!(money(&r2.totals[at(&r2, "value")]), money(&r5.totals[3]));
    assert_eq!(
        money(&r2.totals[at(&r2, "buyer_total")]),
        money(&r5.totals[4])
    );
    // Paid on the spot adds up to R3, what is left to R4 (a part-paid line is on both).
    assert_eq!(money(&r3.totals[at(&r3, "paid")]), money(&r5.totals[5]));
    assert_eq!(money(&r4.totals[at(&r4, "left")]), money(&r5.totals[6]));
    assert_eq!(data.slip_count, data.slips.len().to_string());
    // One slip for each row of R5, each with the sums of that row.
    assert_eq!(i64::try_from(data.slips.len()).unwrap(), r5.total_count);
    // A slip's rows add up to its totals: the serial inside the slip, the value, the total, paid, left.
    for slip in &data.slips {
        let (mut value, mut total, mut paid, mut left) = (0_i64, 0_i64, 0_i64, 0_i64);
        for (i, r) in slip.rows.iter().enumerate() {
            assert_eq!(text(&r.cells[0]), (i + 1).to_string());
            value += parse(&text(&r.cells[2]));
            total += parse(&text(&r.cells[3]));
            paid += parse(&text(&r.cells[4]));
            left += parse(&text(&r.cells[5]));
            // paid + left = the line's total.
            assert_eq!(
                parse(&text(&r.cells[4])) + parse(&text(&r.cells[5])),
                parse(&text(&r.cells[3]))
            );
        }
        assert_eq!(slip.row_count, slip.rows.len().to_string());
        assert_eq!(value, parse(&text(&slip.totals[2])), "{}", slip.party.name);
        assert_eq!(total, parse(&text(&slip.totals[3])), "{}", slip.party.name);
        assert_eq!(paid, parse(&text(&slip.totals[4])), "{}", slip.party.name);
        assert_eq!(left, parse(&text(&slip.totals[5])), "{}", slip.party.name);
    }
}

fn number(cell: &Cell) -> i64 {
    match cell {
        Cell::Number { value } => *value,
        other => panic!("not a number: {other:?}"),
    }
}

fn parse(text: &str) -> i64 {
    text.replace(',', "").parse().unwrap()
}

fn format_rupees(value: i64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn sale_detail(part: &PrintDetailPart) -> (String, String, String, String, String) {
    match part {
        PrintDetailPart::Sale {
            item,
            qty,
            rate,
            value,
            commission,
            ..
        } => (
            item.clone(),
            qty.clone(),
            rate.clone(),
            value.clone(),
            commission.clone(),
        ),
        other => panic!("not a sale: {other:?}"),
    }
}

fn parts(cell: &PrintCell) -> &Vec<PrintDetailPart> {
    match cell {
        PrintCell::Detail { parts } => parts,
        other => panic!("not a detail: {other:?}"),
    }
}

// ---- The worked day (E2E-36) ------------------------------------------------------------------

#[test]
fn the_worked_day_has_a_slip_for_arshad_akram_and_shahid_with_paid_and_left() {
    let w = full_day();
    let filters = day("10-10-2026", "10-10-2026");
    let data = slips_of(&w, filters.clone());
    assert_eq!(data.report, ReportId::R5);
    assert_eq!(data.title_id, "report-r5-title");
    assert_eq!(data.language, Language::En);
    assert_eq!(data.slip_count, "3");
    // Urdu order of the names, as R5: ارشد, اکرم, شاہد. Shahid is a چکتا buyer who paid it all on the
    // spot (12,800): since v1.32 he has a slip too, with 0 left.
    assert_eq!(
        overview(&data),
        vec![
            row("Arshad Multani", "17,400", "18,684", "30"),
            row("Akram Toll Tax", "12,000", "12,880", "20"),
            row("Shahid", "12,000", "12,800", "10"),
        ]
    );
    assert_eq!(data.slips[0].serial, "1");
    assert_eq!(data.slips[1].serial, "2");
    assert_eq!(data.slips[2].serial, "3");
    assert!(data.slips[0].party.person_key.is_none() && data.slips[1].party.person_key.is_none());
    assert!(
        data.slips[2].party.person_key.is_some(),
        "a چکتا buyer has his person key"
    );
    // The line of a slip is written plain, as the client's own print writes it: "(Apple 10 . 1200)",
    // with the rate and the quantity ungrouped (PRD 19.2 R5). Only the value, which is not in that
    // short line, is grouped.
    let shahid = parts(&data.slips[2].rows[0].cells[1]);
    assert_eq!(
        sale_detail(&shahid[0]),
        (
            "Apple".to_owned(),
            "10".to_owned(),
            "1200".to_owned(),
            "12,000".to_owned(),
            "720".to_owned()
        )
    );
    // Paid on the spot and left, in the totals of each slip: nothing paid by the two regular buyers.
    let paid_left = |i: usize| {
        (
            text(&data.slips[i].totals[4]),
            text(&data.slips[i].totals[5]),
        )
    };
    assert_eq!(paid_left(0), ("0".to_owned(), "18,684".to_owned()));
    assert_eq!(paid_left(1), ("0".to_owned(), "12,880".to_owned()));
    assert_eq!(paid_left(2), ("12,800".to_owned(), "0".to_owned()));
    // Akram's slip: the one line, serial 1, "(Banana 20 . 600)", 12,000 and 12,880.
    let akram = &data.slips[1];
    assert_eq!(akram.party.code, "0020");
    assert_eq!(akram.party.short_code.as_deref(), Some("0"));
    assert_eq!(akram.rows.len(), 1);
    assert_eq!(text(&akram.rows[0].cells[0]), "1");
    let detail = parts(&akram.rows[0].cells[1]);
    assert_eq!(detail.len(), 1);
    assert_eq!(
        sale_detail(&detail[0]),
        (
            "Banana".to_owned(),
            "20".to_owned(),
            "600".to_owned(),
            "12,000".to_owned(),
            "720".to_owned()
        )
    );
    assert_eq!(text(&akram.rows[0].cells[2]), "12,000");
    assert_eq!(text(&akram.rows[0].cells[3]), "12,880");
    // The sums of all slips: 60 نگ, 41,400 and 44,364 (the totals of R5 and of R2), 12,800 paid on the
    // spot and 31,564 left.
    assert_eq!(data.qty, "60");
    assert_eq!(
        (
            text(&data.totals[2]),
            text(&data.totals[3]),
            text(&data.totals[4]),
            text(&data.totals[5])
        ),
        (
            "41,400".into(),
            "44,364".into(),
            "12,800".into(),
            "31,564".into()
        )
    );
    assert_adds_up(&w, &filters, &data);
    // The columns are the R5 columns without the buyer: serial, detail, value, total after لاگا, paid
    // on the spot and left.
    let keys: Vec<&str> = data.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(
        keys,
        ["serial", "detail", "value", "buyer_total", "paid", "left"]
    );
    // The header: the dates of the filter, in the print language.
    let labels: Vec<(&str, &str)> = data
        .filters
        .iter()
        .map(|f| (f.label_id.as_str(), f.value.as_str()))
        .collect();
    assert_eq!(
        labels,
        [
            ("report-filter-from", "10-10-2026"),
            ("report-filter-to", "10-10-2026")
        ]
    );
}

#[test]
fn the_urdu_print_has_urdu_names_and_the_right_to_left_direction() {
    let w = full_day();
    let data = buyer_bills_print_data(
        w.env.ctx.db(),
        &input(day("10-10-2026", "10-10-2026"), Language::Ur),
    )
    .unwrap();
    assert_eq!(
        data.slips
            .iter()
            .map(|s| s.party.name.as_str())
            .collect::<Vec<_>>(),
        ["ارشد ملتانی", "اکرم ٹول ٹیکس", "شاہد"]
    );
    let json = serde_json::to_value(&data).unwrap();
    assert_eq!(json["dir"], "rtl");
    assert_eq!(json["language"], "ur");
    // The item is in the print language too.
    let PrintDetailPart::Sale { item, .. } = &parts(&data.slips[0].rows[0].cells[1])[0] else {
        panic!("not a sale");
    };
    assert_eq!(item, "کیلا");
}

#[test]
fn the_slips_follow_the_filters_of_r5_buyer_and_day() {
    let w = full_day();
    let mut filters = day("10-10-2026", "10-10-2026");
    filters.party = Some(PartyFilter {
        account_id: w.akram,
        person_key: None,
    });
    let data = slips_of(&w, filters.clone());
    assert_eq!(
        overview(&data),
        vec![row("Akram Toll Tax", "12,000", "12,880", "20")]
    );
    assert_adds_up(&w, &filters, &data);
    // The header lists the buyer as well as the dates.
    assert!(
        data.filters
            .iter()
            .any(|f| f.label_id == "report-filter-party")
    );
    // A day with no credit sale has no slip, and the sums are 0.
    let none = slips_of(&w, day("09-10-2026", "09-10-2026"));
    assert!(none.slips.is_empty());
    assert_eq!((none.slip_count.as_str(), none.qty.as_str()), ("0", "0"));
    assert_eq!(
        (text(&none.totals[2]), text(&none.totals[3])),
        ("0".into(), "0".into())
    );
}

#[test]
fn only_r5_has_slips_and_the_input_is_checked_like_a_report() {
    let w = full_day();
    let wrong = ReportPrintInput {
        id: ReportId::R4,
        ..input(ReportFilters::default(), Language::En)
    };
    match buyer_bills_print_data(w.env.ctx.db(), &wrong) {
        Err(ReportsError::Rejected(items)) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].code, ErrorCode::ErrOutOfRange);
            assert_eq!(items[0].field, "id");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    // A bad date is refused at its field, as in R5.
    match buyer_bills_print_data(w.env.ctx.db(), &input(day("xx", ""), Language::En)) {
        Err(ReportsError::Rejected(items)) => {
            assert_eq!(items[0].code, ErrorCode::ErrDateInvalid);
            assert_eq!(items[0].field, "filters.from");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

// ---- Cash and credit on the same day ----------------------------------------------------------

#[test]
fn a_cash_line_of_a_credit_buyer_is_on_his_slip_with_what_he_paid_and_what_is_left() {
    // Akram buys 20 at 600 on credit (12,000 + 720 + 160 = 12,880) and 5 at 600 and pays all of it
    // on the spot (3,000 + 180 + 40 = 3,220), in the same lot. Since v1.32 both lines are on his
    // slip (PRD FR-12.44, D8 question Q1): paid 3,220, left 12,880.
    let w = worked_day();
    let mut cash = support::sale("s-2", w.akram, "5", "600");
    cash.paid = Some("3,220".into());
    let credit = support::sale("s-1", w.akram, "20", "600");
    let lot = support::with_sales(
        support::bare_lot(&w, "lot-1", w.rahim, w.banana, "25"),
        vec![credit, cash],
    );
    save(&w, &support::draft(vec![lot]));
    let filters = day("10-10-2026", "10-10-2026");
    let data = slips_of(&w, filters.clone());
    assert_eq!(
        overview(&data),
        vec![row("Akram Toll Tax", "15,000", "16,100", "25")]
    );
    assert_eq!(data.slips[0].rows.len(), 2);
    // The columns after the total: paid on the spot and left on the account, line by line.
    let paid_left =
        |r: &mandi_reports::reports::PrintReportRow| (text(&r.cells[4]), text(&r.cells[5]));
    assert_eq!(
        paid_left(&data.slips[0].rows[0]),
        ("0".into(), "12,880".into())
    );
    assert_eq!(
        paid_left(&data.slips[0].rows[1]),
        ("3,220".into(), "0".into())
    );
    assert_eq!(
        (
            text(&data.slips[0].totals[4]),
            text(&data.slips[0].totals[5])
        ),
        ("3,220".into(), "12,880".into())
    );
    assert_adds_up(&w, &filters, &data);
    // The money paid on the spot is on R3 (paid 3,220) and what is left on R4 (left 12,880).
    let r3 = report(&w, ReportId::R3, filters, 0, 0);
    assert_eq!(money(&r3.totals[r3.totals.len() - 2]), 3_220);
}

// ---- Edit and cancel --------------------------------------------------------------------------

fn edit_akram_rate(w: &support::World, bikri_id: i64, from_entry: i64, rate: &str) -> i64 {
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id,
            entry_id: from_entry,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = rate.into();
    bikri::edit_save(
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
    .unwrap()
    .entry_id
}

#[test]
fn an_edit_shows_the_new_rate_once_and_leaves_an_unchanged_line_exactly_once_e2e_13_14() {
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    let filters = day("10-10-2026", "10-10-2026");

    let edited = edit_akram_rate(&w, saved.bikri_id, saved.entry_id, "610");
    let data = slips_of(&w, filters.clone());
    assert_eq!(
        overview(&data),
        vec![
            row("Arshad Multani", "17,400", "18,684", "30"),
            row("Akram Toll Tax", "12,200", "13,092", "20"),
        ]
    );
    // Arshad's line was not edited: it is on his slip exactly once.
    assert_eq!(data.slips[0].rows.len(), 1);
    assert_eq!(data.slips[1].rows.len(), 1);
    let detail = parts(&data.slips[1].rows[0].cells[1]);
    assert_eq!(sale_detail(&detail[0]).2, "610");
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("29,600".into(), "31,776".into())
    );
    assert_adds_up(&w, &filters, &data);

    // E2E-14: back to 600, the numbers of E2E-1 and E2E-2 return.
    edit_akram_rate(&w, saved.bikri_id, edited, "600");
    let back = slips_of(&w, filters.clone());
    assert_eq!(
        overview(&back),
        vec![
            row("Arshad Multani", "17,400", "18,684", "30"),
            row("Akram Toll Tax", "12,000", "12,880", "20"),
        ]
    );
    assert_eq!(
        (text(&back.totals[2]), text(&back.totals[3])),
        ("29,400".into(), "31,564".into())
    );
    assert_adds_up(&w, &filters, &back);
}

#[test]
fn a_cancelled_bikri_has_no_slips_and_a_standing_one_keeps_its_own() {
    let w = full_day();
    let id: i64 = w
        .env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row("SELECT id FROM bikri_entries WHERE number = 1", [], |r| {
                r.get(0)
            })
        })
        .unwrap();
    let version = bikri::get(&w.env.ctx, id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // بکری 1 is cancelled, so اکرم and ارشد have no slip. شاہد's line is in بکری 2, which stands: he
    // paid it all on the spot, and his slip says so.
    let data = slips_of(&w, ReportFilters::default());
    assert_eq!(
        overview(&data),
        vec![row("Shahid", "12,000", "12,800", "10")]
    );
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("12,000".into(), "12,800".into())
    );
    assert_adds_up(&w, &ReportFilters::default(), &data);
}

// ---- A later sale (E2E-16, E2E-16b) -----------------------------------------------------------

#[test]
fn a_later_sale_is_a_second_line_on_the_slip_dated_by_its_posting() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    set_today(&w.env, 2026, 10, 11);
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    bikri::later_sale_save(
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

    // Both days: Arshad has two lines, 25 at 580 (14,500 + 870 + 200 = 15,570) and 5 at 560
    // (2,800 + 168 + 40 = 3,008).
    let both = day("10-10-2026", "11-10-2026");
    let data = slips_of(&w, both.clone());
    assert_eq!(
        overview(&data),
        vec![
            row("Arshad Multani", "17,300", "18,578", "30"),
            row("Akram Toll Tax", "12,000", "12,880", "20"),
        ]
    );
    let arshad = &data.slips[0];
    assert_eq!(arshad.rows.len(), 2);
    assert_eq!(text(&arshad.rows[0].cells[3]), "15,570");
    assert_eq!(text(&arshad.rows[1].cells[3]), "3,008");
    assert_eq!(text(&arshad.rows[1].cells[0]), "2");
    assert_eq!(data.qty, "50");
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("29,300".into(), "31,458".into())
    );
    assert_adds_up(&w, &both, &data);

    // The day of the later sale alone: one slip, one line, dated by its posting.
    let later = day("11-10-2026", "11-10-2026");
    let only = slips_of(&w, later.clone());
    assert_eq!(
        overview(&only),
        vec![row("Arshad Multani", "2,800", "3,008", "5")]
    );
    assert_adds_up(&w, &later, &only);
}

// ---- Many buyers (the chunks of the IN list) --------------------------------------------------

/// 501 regular credit buyers, each with 1 at 100 (100 + 6 + 8 = 114): more buyers than one chunk of
/// the IN list holds (500).
#[test]
fn more_buyers_than_one_chunk_each_have_their_slip_with_every_line_once() {
    use mandi_services::core::accounts::AccountType;
    let w = worked_day();
    let buyers: Vec<i64> = (1..=501)
        .map(|n| {
            support::add_account(
                &w.env,
                &w.owner,
                AccountType::Buyer,
                &format!("خریدار {n}"),
                &format!("Buyer {n}"),
                true,
                "0",
            )
        })
        .collect();
    // Two بکری of at most 500 lines (NFR-SCL-3): 251 and 250 buyers.
    for (part, ids) in [(0, &buyers[..251]), (1, &buyers[251..])] {
        let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, &ids.len().to_string());
        lot.sales = ids
            .iter()
            .enumerate()
            .map(|(i, b)| support::sale(&format!("s-{part}-{i}"), *b, "1", "100"))
            .collect();
        save(&w, &support::draft(vec![lot]));
    }
    let filters = ReportFilters::default();
    let data = slips_of(&w, filters.clone());
    assert_eq!(data.slips.len(), 501);
    assert_eq!(data.slip_count, "501");
    // Values: 501 × 100 = 50,100. After commission and لاگا: 501 × 114 = 57,114.
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("50,100".into(), "57,114".into())
    );
    assert_eq!(data.qty, "501");
    // Every line once: each of the 501 buyers has one line of 100 and 114.
    let regular: Vec<_> = data
        .slips
        .iter()
        .filter(|s| s.party.person_key.is_none())
        .collect();
    assert_eq!(regular.len(), 501);
    assert!(regular.iter().all(|s| s.rows.len() == 1
        && text(&s.rows[0].cells[2]) == "100"
        && text(&s.rows[0].cells[3]) == "114"));
    assert_adds_up(&w, &filters, &data);
}

// ---- Weight lines and a buyer rate that differs -----------------------------------------------

#[test]
fn a_weight_line_prints_the_stored_value_not_qty_times_rate() {
    // 2 نگ of grapes, 12.345 kg at 100 a kilo: 1,234.5 rounds to 1,235 (C1). Commission 6% of
    // 1,235 = 74.1 → 74; لاگا 8 for each of the 2 نگ = 16. After لاگا 1,235 + 74 + 16 = 1,325.
    let w = worked_day();
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.weight_fruit, "2");
    lot.weight_received = Some("12.345".into());
    let mut s = support::sale("s-1", w.akram, "2", "100");
    s.weight = Some("12.345".into());
    lot.sales = vec![s];
    save(&w, &support::draft(vec![lot]));
    let filters = ReportFilters::default();
    let data = slips_of(&w, filters.clone());
    assert_eq!(
        overview(&data),
        vec![row("Akram Toll Tax", "1,235", "1,325", "2")]
    );
    // The line shows 2 نگ and a rate of 100 a kilo; the value is the stored one, not 2 × 100.
    let PrintDetailPart::Sale {
        qty,
        rate,
        value,
        commission,
        laga,
        ..
    } = &parts(&data.slips[0].rows[0].cells[1])[0]
    else {
        panic!("not a sale");
    };
    assert_eq!(
        (
            qty.as_str(),
            rate.as_str(),
            value.as_str(),
            commission.as_str(),
            laga.as_str()
        ),
        ("2", "100", "1,235", "74", "16")
    );
    assert_adds_up(&w, &filters, &data);
    assert_same_as_plain_print(&w, &filters, &data);
}

#[test]
fn a_line_with_a_different_buyer_rate_prints_the_buyer_rate_and_buyer_value() {
    // Seller rate 600, buyer rate 610 (ریٹ مختلف): the buyer pays 20 × 610 = 12,200 + 732 + 160 =
    // 13,092 (the numbers of E2E-13). The slip shows the buyer's rate and value, not the seller's.
    let w = worked_day();
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "20");
    let mut s = support::sale("s-1", w.akram, "20", "600");
    s.rate_differs = true;
    s.buyer_rate = Some("610".into());
    lot.sales = vec![s];
    save(&w, &support::draft(vec![lot]));
    let filters = ReportFilters::default();
    let data = slips_of(&w, filters.clone());
    assert_eq!(
        overview(&data),
        vec![row("Akram Toll Tax", "12,200", "13,092", "20")]
    );
    let PrintDetailPart::Sale { rate, value, .. } = &parts(&data.slips[0].rows[0].cells[1])[0]
    else {
        panic!("not a sale");
    };
    assert_eq!((rate.as_str(), value.as_str()), ("610", "12,200"));
    assert_adds_up(&w, &filters, &data);
}

// ---- The checks refuse a print that does not add up --------------------------------------------

#[test]
fn a_stored_line_that_is_not_value_plus_commission_plus_laga_is_refused_not_printed() {
    let w = full_day();
    // The guards of the books (the append-only trigger and the CHECK of C5 on the line) block a
    // change of a stored line; this throwaway copy of the data file has them taken away, to prove
    // the print refuses a line the books would never hold.
    let conn = raw(&w.dir);
    conn.pragma_update(None, "ignore_check_constraints", "ON")
        .unwrap();
    let triggers: Vec<String> = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'trigger' AND tbl_name = 'bikri_sales'",
        )
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    for name in &triggers {
        conn.execute(&format!("DROP TRIGGER \"{name}\""), [])
            .unwrap();
    }
    conn.execute(
        "UPDATE bikri_sales SET buyer_total_paisa = buyer_total_paisa + 10000 WHERE id = 1",
        [],
    )
    .unwrap();
    drop(conn);
    match buyer_bills_print_data(
        w.env.ctx.db(),
        &input(ReportFilters::default(), Language::En),
    ) {
        Err(ReportsError::Data(message)) => assert!(message.contains("laga"), "{message}"),
        other => panic!("expected a data error, got {other:?}"),
    }
}

// ---- The slips and the plain print agree ------------------------------------------------------

/// The slips hold the lines of the plain R5 print row for row: the buyer, the details and the
/// sums of a row are those of the slip.
fn assert_same_as_plain_print(
    w: &support::World,
    filters: &ReportFilters,
    slips: &BuyerBillsPrintData,
) {
    let plain = print_data(w.env.ctx.db(), &input(filters.clone(), Language::En)).unwrap();
    assert_eq!(plain.rows.len(), slips.slips.len());
    for (row, slip) in plain.rows.iter().zip(&slips.slips) {
        let PrintCell::Party { party } = &row.cells[1] else {
            panic!("not a party");
        };
        assert_eq!(party, &slip.party);
        let all: Vec<PrintDetailPart> = slip
            .rows
            .iter()
            .flat_map(|r| parts(&r.cells[1]).clone())
            .collect();
        assert_eq!(parts(&row.cells[2]), &all, "{}", slip.party.name);
        assert_eq!(row.cells[3], slip.totals[2]);
        assert_eq!(row.cells[4], slip.totals[3]);
    }
    assert_eq!(plain.totals[3], slips.totals[2]);
    assert_eq!(plain.totals[4], slips.totals[3]);
}

#[test]
fn the_slips_hold_the_lines_of_the_r5_print_row_for_row() {
    let w = full_day();
    let filters = day("10-10-2026", "10-10-2026");
    let slips = slips_of(&w, filters.clone());
    assert_same_as_plain_print(&w, &filters, &slips);
}
