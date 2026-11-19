//! Plan 74 T1 (PRD FR-12.47, FR-19.5, FR-19.7, FR-19.13): one party's statement on paper. The print
//! holds every date with an entry in the period, oldest first: the goods of the day, the commission
//! and لاگا on one line, the day's bill, what was paid on the spot, each وصولی or ادائیگی, and the
//! balance at the end of the day; then the totals of the period and the closing balance in words.
//!
//! The numbers are the PRD's: the example of FR-12.47, and the worked day (PRD Section 4) with اکرم's
//! later receipts. Every figure is also held to the statement of the screen and to R13.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::reports::{
    ReportId, StatementKind, StatementPrintData, StatementPrintDataInput, StatementSide,
    party_statement_print_data,
};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{BikriDraft, LineChargeMethod};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::VoucherCancelInput;
use mandi_services::{party_ledgers, vouchers};
use reports_world::*;
use support::World;

fn print(
    w: &World,
    account: i64,
    from: &str,
    to: &str,
    language: Language,
) -> Result<StatementPrintData, mandi_services::ServiceError> {
    let whole = party_ledgers::statement_whole(&w.env.ctx, account, from, to)?;
    Ok(party_statement_print_data(
        w.env.ctx.db(),
        &whole,
        &StatementPrintDataInput {
            account_id: account,
            from: from.to_owned(),
            to: to.to_owned(),
            language,
            paper: None,
        },
    )
    .unwrap())
}

fn amounts(list: &[mandi_reports::print::PrintAmount]) -> Vec<(&str, &str)> {
    list.iter()
        .map(|a| (a.label_id.as_str(), a.amount.as_str()))
        .collect()
}

fn entries(day: &mandi_reports::reports::StatementDay) -> Vec<(&str, &str)> {
    day.entries
        .iter()
        .map(|e| (e.label_id.as_str(), e.amount.as_str()))
        .collect()
}

/// A بکری of 05-11 or 06-11 whose one line is اکرم's, with commission and لاگا typed as 0 so that the
/// bill is the goods (PRD FR-11.10).
fn sale_day(w: &World, date: &str, qty: &str, rate: &str, paid: Option<&str>) -> BikriDraft {
    let mut line = support::sale("s-1", w.akram, qty, rate);
    line.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
    line.laga = support::typed(LineChargeMethod::PerUnit, "0");
    line.paid = paid.map(str::to_owned);
    let lot = support::with_sales(
        support::bare_lot(w, "lot-1", w.rahim, w.banana, qty),
        vec![line],
    );
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = date.into();
    draft
}

#[test]
fn the_prd_example_prints_two_dates_with_its_totals_and_the_closing_in_words() {
    let w = support::world();
    // 05-11-2026: he bought 30,000 and paid nothing. 06-11-2026: he bought 21,360 and paid it on the
    // spot, so his balance is still 30,000 بنام.
    set_today(&w.env, 2026, 11, 5);
    save(&w, &sale_day(&w, "05-11-2026", "30", "1,000", None));
    set_today(&w.env, 2026, 11, 6);
    save(
        &w,
        &sale_day(&w, "06-11-2026", "20", "1,068", Some("21,360")),
    );
    let data = print(&w, w.akram, "01-11-2026", "06-11-2026", Language::En).unwrap();
    assert_eq!(data.kind, StatementKind::Buyer);
    assert_eq!(
        (data.period_from.as_str(), data.period_to.as_str()),
        ("01-11-2026", "06-11-2026")
    );
    assert_eq!(data.opening.amount, "0");
    assert_eq!(data.opening.side, StatementSide::None);
    // Oldest first.
    let dates: Vec<&str> = data.days.iter().map(|d| d.date.as_str()).collect();
    assert_eq!(dates, ["05-11-2026", "06-11-2026"]);
    let (first, second) = (&data.days[0], &data.days[1]);
    assert_eq!(first.items.len(), 1);
    let item = &first.items[0];
    assert_eq!(
        (
            item.name.as_str(),
            item.qty.as_str(),
            item.rate.as_str(),
            item.amount.as_str()
        ),
        ("Banana", "30", "1,000", "30,000")
    );
    assert!(
        first.charges.is_empty(),
        "commission and laga were typed as 0"
    );
    assert_eq!(
        first.bill.as_ref().map(|b| b.amount.as_str()),
        Some("30,000")
    );
    assert_eq!(
        first.bill.as_ref().map(|b| b.label_id.as_str()),
        Some("print-statement-bill-buyer")
    );
    assert_eq!(first.paid_on_spot, None);
    assert!(first.entries.is_empty());
    assert_eq!(
        (first.closing.amount.as_str(), first.closing.side),
        ("30,000", StatementSide::Debit)
    );
    // The second: 21,360 bought, all paid on the spot, still 30,000 بنام.
    assert_eq!(second.items[0].amount, "21,360");
    assert_eq!(
        second.bill.as_ref().map(|b| b.amount.as_str()),
        Some("21,360")
    );
    assert_eq!(second.paid_on_spot.as_deref(), Some("21,360"));
    assert_eq!(
        (second.closing.amount.as_str(), second.closing.side),
        ("30,000", StatementSide::Debit)
    );
    // The totals: bought 51,360, paid on the spot 21,360; nothing else is 0 and printed.
    assert_eq!(
        amounts(&data.totals),
        [
            ("print-statement-total-bought", "51,360"),
            ("print-statement-total-paid-on-spot", "21,360"),
        ]
    );
    assert_eq!(
        (data.closing.amount.as_str(), data.closing.side),
        ("30,000", StatementSide::Debit)
    );
    let words = data.words.expect("the closing is in words");
    assert!(words.text.contains("Thirty thousand"), "{}", words.text);
}

/// The worked day and اکرم paying 1,000, 500, 380 and 1,000 on four later days. Today is 05-12-2026.
fn world_with_receipts() -> World {
    let w = full_day();
    for (today, date, amount, note) in [
        ((2026, 10, 11), "11-10-2026", "1,000", ""),
        ((2026, 10, 12), "12-10-2026", "500", ""),
        ((2026, 11, 15), "15-11-2026", "380", "پرانا حساب"),
        ((2026, 12, 2), "02-12-2026", "1,000", ""),
    ] {
        set_today(&w.env, today.0, today.1, today.2);
        let mut input = receipt(w.akram, amount);
        input.date = date.into();
        input.narration = note.into();
        vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    }
    set_today(&w.env, 2026, 12, 5);
    w
}

#[test]
fn a_buyers_statement_of_the_worked_day_has_the_goods_the_charges_and_every_receipt() {
    let w = world_with_receipts();
    let data = print(&w, w.akram, "01-10-2026", "05-12-2026", Language::En).unwrap();
    let dates: Vec<&str> = data.days.iter().map(|d| d.date.as_str()).collect();
    assert_eq!(
        dates,
        [
            "10-10-2026",
            "11-10-2026",
            "12-10-2026",
            "15-11-2026",
            "02-12-2026"
        ]
    );
    // 10-10: 20 Banana at 600 = 12,000; commission 720 and laga 160 on one line; the bill 12,880 (PRD
    // E2E-2); the receipt of 10,000 (Event 3); 2,880 بنام.
    let day = &data.days[0];
    assert_eq!(day.items.len(), 1);
    assert_eq!(day.items[0].amount, "12,000");
    assert_eq!(
        amounts(&day.charges),
        [("print-buyer-commission", "720"), ("print-laga", "160")]
    );
    assert_eq!(day.bill.as_ref().map(|b| b.amount.as_str()), Some("12,880"));
    assert_eq!(day.paid_on_spot, None);
    assert_eq!(entries(day), [("print-statement-entry-receipt", "10,000")]);
    assert_eq!(
        (day.closing.amount.as_str(), day.closing.side),
        ("2,880", StatementSide::Debit)
    );
    // The later days have a receipt and no goods, no charges and no bill.
    let later = &data.days[1];
    assert!(later.items.is_empty() && later.charges.is_empty() && later.bill.is_none());
    assert_eq!(entries(later), [("print-statement-entry-receipt", "1,000")]);
    assert_eq!(later.closing.amount, "1,880");
    // The narration of a receipt is its note.
    let noted = &data.days[3];
    assert_eq!(noted.entries[0].note.as_deref(), Some("پرانا حساب"));
    assert_eq!(noted.closing.amount, "1,000");
    // Paid in full on 02-12-2026.
    let last = &data.days[4];
    assert_eq!(
        (last.closing.amount.as_str(), last.closing.side),
        ("0", StatementSide::None)
    );
    // Totals: bought 12,880, commission and laga 880, received 12,880; closing 0, so no words.
    assert_eq!(
        amounts(&data.totals),
        [
            ("print-statement-total-bought", "12,880"),
            ("print-statement-total-charges", "880"),
            ("print-statement-total-received", "12,880"),
        ]
    );
    assert_eq!(data.closing.side, StatementSide::None);
    assert!(data.words.is_none());
    // The party block and the header facts.
    assert_eq!(data.party.code.as_deref(), Some("0020"));
    assert_eq!(data.party.short_code.as_deref(), Some("0"));
    assert_eq!(data.party.name, "Akram Toll Tax");
    assert_eq!(data.party.role_id, "print-party-buyer");
    assert_eq!(data.filters.len(), 3);
    assert_eq!(data.filters[0].value, "01-10-2026");
    assert_eq!(data.filters[1].value, "05-12-2026");
    assert_eq!(data.title_id, "print-statement-title");
}

#[test]
fn the_urdu_print_has_the_urdu_names_and_the_same_digits() {
    let w = world_with_receipts();
    let data = print(&w, w.akram, "01-10-2026", "05-12-2026", Language::Ur).unwrap();
    assert_eq!(data.language, Language::Ur);
    assert_eq!(data.party.name, "اکرم ٹول ٹیکس");
    assert_eq!(data.days[0].items[0].name, "کیلا");
    // Digits 0 to 9 with grouping, never the Urdu digits (PRD FR-7.32).
    assert_eq!(
        data.days[0].bill.as_ref().map(|b| b.amount.as_str()),
        Some("12,880")
    );
    assert_eq!(data.days[0].date, "10-10-2026");
}

#[test]
fn a_sellers_statement_has_his_deductions_his_cash_and_what_was_paid_to_him() {
    let w = full_day();
    let data = print(&w, w.rahim, "01-09-2026", "10-10-2026", Language::En).unwrap();
    assert_eq!(data.kind, StatementKind::Seller);
    assert_eq!(data.party.role_id, "print-party-seller");
    let dates: Vec<&str> = data.days.iter().map(|d| d.date.as_str()).collect();
    assert_eq!(dates, ["01-09-2026", "10-10-2026"]);
    // 01-09: the advance of 15,000 given to him, a debit.
    let advance = &data.days[0];
    assert_eq!(
        entries(advance),
        [("print-statement-entry-advance", "15,000")]
    );
    assert_eq!(
        (advance.closing.amount.as_str(), advance.closing.side),
        ("15,000", StatementSide::Debit)
    );
    assert!(advance.bill.is_none());
    // 10-10 (PRD 4.4): two lines of Banana, 20 at 600 and 30 at 580, gross 29,400; his deductions
    // 1,764 + 3,000 + 250 + 50 + 20 (store rent is 0 and not printed); the net sale 24,316 (C8); the
    // نقد 5,000 given on the spot; the payment of 4,316 (Event 4); the balance 0.
    let day = &data.days[1];
    assert_eq!(day.items.len(), 2);
    assert_eq!(
        (
            day.items[0].qty.as_str(),
            day.items[0].rate.as_str(),
            day.items[0].amount.as_str()
        ),
        ("20", "600", "12,000")
    );
    assert_eq!(day.items[1].amount, "17,400");
    assert_eq!(
        amounts(&day.charges),
        [
            ("print-commission", "1,764"),
            ("print-freight", "3,000"),
            ("print-labour", "250"),
            ("print-munshiana", "50"),
            ("print-phone-charge", "20"),
        ]
    );
    let bill = day.bill.as_ref().unwrap();
    assert_eq!(
        (bill.label_id.as_str(), bill.amount.as_str()),
        ("print-statement-bill-seller", "24,316")
    );
    assert_eq!(day.paid_on_spot.as_deref(), Some("5,000"));
    assert_eq!(entries(day), [("print-statement-entry-payment", "4,316")]);
    assert_eq!(
        (day.closing.amount.as_str(), day.closing.side),
        ("0", StatementSide::None)
    );
    // Totals: sold 29,400; deductions 5,084 (the نقد is not one); نقد 5,000; paid 4,316; advanced 15,000.
    assert_eq!(
        amounts(&data.totals),
        [
            ("print-statement-total-sold", "29,400"),
            ("print-statement-total-deductions", "5,084"),
            ("print-statement-total-cash", "5,000"),
            ("print-statement-total-paid", "4,316"),
            ("print-statement-total-advanced", "15,000"),
        ]
    );
    assert!(data.words.is_none());
}

#[test]
fn a_voucher_cancelled_on_its_own_day_is_not_listed_and_changes_nothing() {
    let w = world_with_receipts();
    let before = print(&w, w.akram, "01-10-2026", "05-12-2026", Language::En).unwrap();
    // A receipt of 700 saved and cancelled on 05-12-2026: both halves are left out, as on the slip.
    let mut input = receipt(w.akram, "700");
    input.date = "05-12-2026".into();
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: saved.voucher_id,
            version: row.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let after = print(&w, w.akram, "01-10-2026", "05-12-2026", Language::En).unwrap();
    assert_eq!(after, before);
}

#[test]
fn another_entry_on_the_party_has_its_own_label_and_total() {
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
    let w = full_day();
    // A جنرل line of 700 on اکرم on 10-10-2026: a debit that is neither a sale nor a receipt.
    let cash = support::id_of(&w.env, "01010001");
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(w.akram), Side::Dr, rupees(700)),
        LedgerLine::new(AccountId::new(cash), Side::Cr, rupees(700)),
    ])
    .unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            mandi_services::ledger::post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10)
                        .unwrap(),
                    posting_type: PostingType::General,
                    source_type: SourceType::Voucher,
                    source_id: 9_003,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    let data = print(&w, w.akram, "10-10-2026", "10-10-2026", Language::En).unwrap();
    let day = &data.days[0];
    assert_eq!(
        entries(day),
        [
            ("print-statement-entry-receipt", "10,000"),
            ("print-statement-entry-other-debit", "700"),
        ]
    );
    // 12,880 + 700 − 10,000 = 3,580 بنام.
    assert_eq!(
        (day.closing.amount.as_str(), day.closing.side),
        ("3,580", StatementSide::Debit)
    );
    assert_eq!(
        amounts(&data.totals),
        [
            ("print-statement-total-bought", "12,880"),
            ("print-statement-total-charges", "880"),
            ("print-statement-total-received", "10,000"),
            ("print-statement-total-other-debit", "700"),
        ]
    );
}

#[test]
fn a_period_with_no_entry_prints_its_opening_and_closing_and_nothing_else() {
    let w = world_with_receipts();
    let data = print(&w, w.akram, "20-11-2026", "30-11-2026", Language::En).unwrap();
    assert!(data.days.is_empty());
    assert_eq!(
        (data.opening.amount.as_str(), data.opening.side),
        ("1,000", StatementSide::Debit)
    );
    assert_eq!(
        (data.closing.amount.as_str(), data.closing.side),
        ("1,000", StatementSide::Debit)
    );
    assert_eq!(
        amounts(&data.totals),
        [("print-statement-total-bought", "0")]
    );
    assert!(data.words.is_some());
}

#[test]
fn the_print_agrees_with_the_statement_of_the_screen_and_with_r13() {
    let w = world_with_receipts();
    let whole =
        party_ledgers::statement_whole(&w.env.ctx, w.akram, "01-10-2026", "05-12-2026").unwrap();
    let data = print(&w, w.akram, "01-10-2026", "05-12-2026", Language::En).unwrap();
    // The same dates, the same balances after each of them, newest first on the screen and oldest first
    // on paper.
    assert_eq!(data.days.len(), whole.statement.rows.len());
    for (printed, row) in data.days.iter().rev().zip(&whole.statement.rows) {
        assert_eq!(
            printed.closing.amount.replace(',', ""),
            row.balance_paisa.rupees().to_string()
        );
    }
    // R13's closing balance is the print's closing balance.
    let r13 = report(&w, ReportId::R13, party(w.akram), 0, 0);
    assert_eq!(
        balance(&r13.summary.last().unwrap().cell),
        (
            whole.statement.closing_side,
            whole.statement.closing_paisa.rupees()
        )
    );
    assert_eq!(whole.statement.closing_paisa.rupees(), 0);
}

#[test]
fn a_bad_request_is_refused_with_its_code() {
    let w = full_day();
    let err = |from: &str, to: &str, account: i64| {
        party_ledgers::statement_whole(&w.env.ctx, account, from, to)
            .unwrap_err()
            .code()
    };
    assert_eq!(err("", "10-10-2026", w.akram), ErrorCode::ErrRequired);
    assert_eq!(err("10-10-2026", "", w.akram), ErrorCode::ErrRequired);
    assert_eq!(
        err("31-02-2026", "10-10-2026", w.akram),
        ErrorCode::ErrDateInvalid
    );
    assert_eq!(
        err("11-10-2026", "10-10-2026", w.akram),
        ErrorCode::ErrDateRange
    );
    // The shared چکتا account has no ledger (PRD FR-13.6).
    assert_eq!(
        err("01-10-2026", "10-10-2026", w.chukta_khata),
        ErrorCode::ErrNotFound
    );
    // A `to` after today is allowed, as on the screen's statement: nothing is posted after today, so
    // the closing balance is his balance now.
    let future =
        party_ledgers::statement_whole(&w.env.ctx, w.akram, "01-10-2026", "31-12-2026").unwrap();
    assert_eq!(future.statement.closing_paisa.rupees(), 2_880);
}

#[test]
fn the_voucher_lines_query_reads_the_lines_of_one_party_by_its_index() {
    // Code Standards B18: the lines of the party come from the index on his account, and the
    // entries are found by their key, never by scanning every line of the journal.
    let w = full_day();
    let conn = raw(&w.dir);
    let sql = format!(
        "EXPLAIN QUERY PLAN {}",
        mandi_reports::reports::entry_lines_sql()
    );
    let mut statement = conn.prepare(&sql).unwrap();
    let plan = statement
        .query_map(
            mandi_services::db::rusqlite::params![w.akram, "2026-10-01", "2026-12-05"],
            |r| r.get::<_, String>(3),
        )
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(plan.contains("SEARCH l USING"), "{plan}");
    assert!(!plan.contains("SCAN l"), "{plan}");
    assert!(!plan.contains("SCAN e"), "{plan}");
}

#[test]
fn lines_of_one_item_packing_and_rate_are_one_line_and_another_packing_is_another() {
    let w = support::world();
    let crate_ = support::add_packing(&w.env, &w.owner, "کریٹ", "Crate");
    // Two lines of 10 and 5 Banana at 600 in one lot join; 4 at 600 in another packing stay apart.
    let first = support::with_sales(
        support::bare_lot(&w, "lot-1", w.rahim, w.banana, "15"),
        vec![
            support::sale("s-1", w.akram, "10", "600"),
            support::sale("s-2", w.akram, "5", "600"),
        ],
    );
    let mut second = support::with_sales(
        support::bare_lot(&w, "lot-2", w.rahim, w.banana, "4"),
        vec![support::sale("s-3", w.akram, "4", "600")],
    );
    second.packing_id = Some(crate_);
    save(&w, &support::draft(vec![first, second]));
    let data = print(&w, w.akram, "10-10-2026", "10-10-2026", Language::En).unwrap();
    let items: Vec<(&str, &str, &str)> = data.days[0]
        .items
        .iter()
        .map(|i| (i.qty.as_str(), i.rate.as_str(), i.amount.as_str()))
        .collect();
    assert_eq!(items, [("15", "600", "9,000"), ("4", "600", "2,400")]);
}

// ---- The real answers as JSON files for the frontend (plan 74 T3) ----------------------------
//
// Each fixture is a pair: `<name>_request.json` is exactly the `input` of
// `party_ledgers_statement_print_data` and `<name>.json` is the answer the real service gives. The
// test FAILS if a file is not the real answer; after a deliberate change write them again with
// `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test party_statement_print` and tell
// frontend.

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn write_or_check(file: &str, text: &str) {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let path = folder.join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the real answer any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

/// Writes or checks the pair of one request and the real answer for it.
fn fixture(w: &World, name: &str, account: i64, from: &str, to: &str, language: &str) {
    let request = serde_json::json!({
        "account_id": account, "from": from, "to": to, "language": language
    });
    let input: StatementPrintDataInput = serde_json::from_value(request.clone()).unwrap();
    let whole = party_ledgers::statement_whole(&w.env.ctx, account, from, to).unwrap();
    let answer = party_statement_print_data(w.env.ctx.db(), &whole, &input).unwrap();
    write_or_check(
        &format!("party_ledgers_statement_print_{name}_request.json"),
        &(serde_json::to_string_pretty(&request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("party_ledgers_statement_print_{name}.json"),
        &(serde_json::to_string_pretty(&answer).unwrap() + "\n"),
    );
}

#[test]
fn the_statement_print_fixtures_are_the_real_answers() {
    // PRD FR-12.47: a buyer, 05-11-2026 bought 30,000, 06-11-2026 bought 21,360 and paid it.
    let w = support::world();
    set_today(&w.env, 2026, 11, 5);
    save(&w, &sale_day(&w, "05-11-2026", "30", "1,000", None));
    set_today(&w.env, 2026, 11, 6);
    save(
        &w,
        &sale_day(&w, "06-11-2026", "20", "1,068", Some("21,360")),
    );
    fixture(&w, "example", w.akram, "01-11-2026", "06-11-2026", "en");
    // The worked day and اکرم's later receipts: charges, receipts, a note, a balance paid to 0.
    let w = world_with_receipts();
    fixture(&w, "akram", w.akram, "01-10-2026", "05-12-2026", "en");
    fixture(&w, "akram_ur", w.akram, "01-10-2026", "05-12-2026", "ur");
    // A seller: the advance, two lines of Banana, his deductions, the نقد and a payment.
    fixture(&w, "rahim", w.rahim, "01-09-2026", "05-12-2026", "en");
    // A period with no entry: no days, the opening is the closing.
    fixture(&w, "empty", w.akram, "20-11-2026", "30-11-2026", "en");
}
