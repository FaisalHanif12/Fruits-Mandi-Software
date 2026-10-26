//! R5 "Print All": one slip for each credit buyer (PRD 19.2 R5, FR-19.10, FR-19.11, FR-13.5,
//! E2E-36; plan d5-late, round 12).
//!
//! The data is made the way `golden_day.rs` makes it (the real services; `reports_world`). The
//! expected numbers are worked by hand from the PRD and the money agent's review: every slip line
//! is a stored line value, the slips add up to the totals of R5 and of R4, and a cash line (نقدی)
//! is never on a slip.
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
    CancelInput, EditOpenInput, EditSaveInput, LaterSaleSaveInput, PaymentType, PukhtaInput,
    StockListInput,
};
use mandi_services::core::error::ErrorCode;

use reports_world::*;

fn input(filters: ReportFilters, language: Language) -> ReportPrintInput {
    ReportPrintInput {
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

/// The sums of the slips against R5 and R4 of the same filter (the money agent's rule: grand
/// totals equal the R5 totals row and the R4 totals).
fn assert_adds_up(w: &support::World, filters: &ReportFilters, data: &BuyerBillsPrintData) {
    let r5 = report(w, ReportId::R5, filters.clone(), 0, 0);
    let r4 = report(w, ReportId::R4, filters.clone(), 0, 0);
    assert_eq!(text(&data.totals[2]), format_rupees(money(&r5.totals[3])));
    assert_eq!(text(&data.totals[3]), format_rupees(money(&r5.totals[4])));
    // The quantity of all slips is the quantity total of R4, which sums `qty` in SQL.
    assert_eq!(data.qty, number(&r4.totals[5]).to_string());
    assert_eq!(money(&r4.totals[7]), money(&r5.totals[3]));
    assert_eq!(money(&r4.totals[8]), money(&r5.totals[4]));
    assert_eq!(data.slip_count, data.slips.len().to_string());
    // One slip for each row of R5, each with the sums of that row.
    assert_eq!(i64::try_from(data.slips.len()).unwrap(), r5.total_count);
    // A slip's rows add up to its totals: the serial inside the slip, the value, the total.
    for slip in &data.slips {
        let (mut value, mut total) = (0_i64, 0_i64);
        for (i, r) in slip.rows.iter().enumerate() {
            assert_eq!(text(&r.cells[0]), (i + 1).to_string());
            value += parse(&text(&r.cells[2]));
            total += parse(&text(&r.cells[3]));
        }
        assert_eq!(slip.row_count, slip.rows.len().to_string());
        assert_eq!(value, parse(&text(&slip.totals[2])), "{}", slip.party.name);
        assert_eq!(total, parse(&text(&slip.totals[3])), "{}", slip.party.name);
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
fn the_worked_day_has_a_slip_for_akram_and_one_for_arshad_and_none_for_shahid() {
    let w = full_day();
    let filters = day("10-10-2026", "10-10-2026");
    let data = slips_of(&w, filters.clone());
    assert_eq!(data.report, ReportId::R5);
    assert_eq!(data.title_id, "report-r5-title");
    assert_eq!(data.language, Language::En);
    assert_eq!(data.slip_count, "2");
    // Urdu order of the names, as R5: ارشد before اکرم. Shahid paid cash on the spot, so he has no
    // slip.
    assert_eq!(
        overview(&data),
        vec![
            row("Arshad Multani", "17,400", "18,684", "30"),
            row("Akram Toll Tax", "12,000", "12,880", "20"),
        ]
    );
    assert_eq!(data.slips[0].serial, "1");
    assert_eq!(data.slips[1].serial, "2");
    assert!(
        data.slips
            .iter()
            .all(|s| !s.party.name.contains("Shahid") && s.party.person_key.is_none())
    );
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
    // The sums of all slips: 50 نگ, 29,400 and 31,564, the totals of R5 and of R4.
    assert_eq!(data.qty, "50");
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("29,400".into(), "31,564".into())
    );
    assert_adds_up(&w, &filters, &data);
    // The columns are the R5 columns without the buyer: serial, detail, value, total after لاگا.
    let keys: Vec<&str> = data.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["serial", "detail", "value", "buyer_total"]);
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
        ["ارشد ملتانی", "اکرم ٹول ٹیکس"]
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
fn a_cash_line_of_a_credit_buyer_is_not_on_his_slip_and_not_in_his_totals() {
    // Akram buys 20 at 600 on credit (12,000 + 720 + 160 = 12,880) and 5 at 600 for cash
    // (3,000 + 180 + 40 = 3,220) in the same lot.
    let w = worked_day();
    let mut cash = support::sale("s-2", w.akram, "5", "600");
    cash.payment_type = Some(PaymentType::Cash);
    let mut credit = support::sale("s-1", w.akram, "20", "600");
    credit.payment_type = Some(PaymentType::Credit);
    let lot = support::with_sales(
        support::bare_lot(&w, "lot-1", w.rahim, w.banana, "25"),
        vec![credit, cash],
    );
    save(&w, &support::draft(vec![lot]));
    let filters = day("10-10-2026", "10-10-2026");
    let data = slips_of(&w, filters.clone());
    assert_eq!(
        overview(&data),
        vec![row("Akram Toll Tax", "12,000", "12,880", "20")]
    );
    assert_eq!(data.slips[0].rows.len(), 1);
    assert_adds_up(&w, &filters, &data);
    // The cash line is in R3 only.
    let r3 = report(&w, ReportId::R3, filters, 0, 0);
    assert_eq!(money(&r3.totals[8]), 3_220);
}

// ---- One-time persons (FR-13.5) ---------------------------------------------------------------

/// Three credit lines under the one-time account: شاہد with two phones, and ناصر (each person has
/// his own slip). The default 6% buyer commission and 8 per نگ لاگا on fruit.
fn one_time_world() -> support::World {
    let w = worked_day();
    let line = |key: &str, qty: &str, ur: &str, en: &str, phone: &str| {
        let mut l = support::sale(key, w.pukhta_buyer, qty, "200");
        l.pukhta = Some(PukhtaInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.into(),
        });
        l.payment_type = Some(PaymentType::Credit);
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "17");
    lot.sales = vec![
        line("s-1", "5", "شاہد", "Shahid", "0300-0000000"),
        line("s-2", "2", "شاہد", "Shahid", "0301-2222222"),
        line("s-3", "10", "ناصر", "Nasir", "0300-1111111"),
    ];
    save(&w, &support::draft(vec![lot]));
    w
}

#[test]
fn each_one_time_person_has_a_slip_of_his_own_even_with_the_same_name() {
    let w = one_time_world();
    let filters = ReportFilters::default();
    let data = slips_of(&w, filters.clone());
    // Same name, two phones: two slips, in a fixed order. ناصر after شاہد in the Urdu order.
    assert_eq!(
        overview(&data),
        vec![
            row("Shahid", "1,000", "1,100", "5"),
            row("Shahid", "400", "440", "2"),
            row("Nasir", "2,000", "2,200", "10"),
        ]
    );
    // The two شاہد slips are told apart by their person keys, in a fixed order.
    let keys: Vec<&str> = data
        .slips
        .iter()
        .map(|s| s.party.person_key.as_deref().unwrap())
        .collect();
    assert!(keys[0] < keys[1], "{keys:?}");
    assert!(data.slips.iter().all(|s| s.party.code == "0030"));
    // 17 نگ, 3,400 and 3,740 in all.
    assert_eq!(data.qty, "17");
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("3,400".into(), "3,740".into())
    );
    assert_adds_up(&w, &filters, &data);
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
fn a_cancelled_bikri_has_no_slips() {
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
    // بکری 2's line is cash, so it never was on a slip: nothing is left.
    let data = slips_of(&w, ReportFilters::default());
    assert!(data.slips.is_empty());
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("0".into(), "0".into())
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

/// 501 regular credit buyers, each with 1 at 100 (100 + 6 + 8 = 114), and the three one-time
/// persons of `one_time_world`: more buyers than one chunk of the IN list holds (500).
#[test]
fn more_buyers_than_one_chunk_each_have_their_slip_with_every_line_once() {
    use mandi_services::core::accounts::AccountType;
    let w = one_time_world();
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
            .map(|(i, b)| {
                let mut s = support::sale(&format!("s-{part}-{i}"), *b, "1", "100");
                s.payment_type = Some(PaymentType::Credit);
                s
            })
            .collect();
        save(&w, &support::draft(vec![lot]));
    }
    let filters = ReportFilters::default();
    let data = slips_of(&w, filters.clone());
    assert_eq!(data.slips.len(), 504);
    assert_eq!(data.slip_count, "504");
    // Values: 501 × 100 = 50,100 and 3,400, so 53,500. After commission and لاگا: 501 × 114 =
    // 57,114 and 3,740, so 60,854.
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("53,500".into(), "60,854".into())
    );
    assert_eq!(data.qty, (501 + 17).to_string());
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

/// More one-time persons than one chunk of the IN list holds (500): they all sit under ONE
/// account, so the account must be asked for once. Without that, its lines would be read twice
/// and printed twice (money agent, round 12).
#[test]
fn more_one_time_persons_than_one_chunk_have_their_lines_once() {
    let w = worked_day();
    // Two بکری of 251 and 250 persons (at most 500 lines each, NFR-SCL-3).
    for (part, range) in [(0, 1..=251), (1, 252..=501)] {
        let mut lot = support::bare_lot(
            &w,
            "lot-1",
            w.rahim,
            w.banana,
            &range.clone().count().to_string(),
        );
        lot.sales = range
            .map(|n| {
                let mut s = support::sale(&format!("s-{part}-{n}"), w.pukhta_buyer, "1", "100");
                s.pukhta = Some(PukhtaInput {
                    name_ur: format!("مہمان {n}"),
                    name_en: format!("Guest {n}"),
                    phone: format!("0300-{n:07}"),
                });
                s.payment_type = Some(PaymentType::Credit);
                s
            })
            .collect();
        save(&w, &support::draft(vec![lot]));
    }
    let filters = ReportFilters::default();
    let data = slips_of(&w, filters.clone());
    assert_eq!(data.slips.len(), 501);
    // 501 persons, each with one line of 100 and 114 (6 + 8).
    assert!(data.slips.iter().all(|s| s.rows.len() == 1
        && text(&s.rows[0].cells[2]) == "100"
        && text(&s.rows[0].cells[3]) == "114"));
    assert_eq!(data.qty, "501");
    assert_eq!(
        (text(&data.totals[2]), text(&data.totals[3])),
        ("50,100".into(), "57,114".into())
    );
    assert_adds_up(&w, &filters, &data);
    // The plain R5 print reads every row too; its details must not be doubled.
    assert_same_as_plain_print(&w, &filters, &data);
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
    s.payment_type = Some(PaymentType::Credit);
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
    s.payment_type = Some(PaymentType::Credit);
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
