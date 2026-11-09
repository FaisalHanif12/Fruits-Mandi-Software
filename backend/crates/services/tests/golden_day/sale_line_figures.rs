//! Plan 61 T5 (D8 Bug 1), E2E-64: the sale lines of the client's screenshot of 08-11-2026, worked
//! on the real services. The lot is 200 آم received from the seller احسان, کرایہ 10,000 paid by the
//! firm, مزدوری 1,000. Three lines: مانی 20 at 1,000, انس 50 at 900 and ہارون 130 at 900. The rate
//! of line 3 is the same as the last-rate hint that the empty rate cell shows (FR-7.53).
//!
//! The expected numbers are the PRD's (D8 Bug 1, plan 61 Section 3) and are never changed to
//! suit the code. The test passes on the first run: the money rules were never the problem, a
//! complete line is worked out and counted. (The screen lost line 3 before the backend saw it.)
//!
//! | Buyer | Qty | Rate | Value | Seller comm | Buyer comm | لاگا | Buyer total |
//! |---|---|---|---|---|---|---|---|
//! | مانی | 20 | 1,000 | 20,000 | 1,200 | 1,200 | 160 | 21,360 |
//! | انس | 50 | 900 | 45,000 | 2,700 | 2,700 | 400 | 48,100 |
//! | ہارون | 130 | 900 | 117,000 | 7,020 | 7,020 | 1,040 | 125,060 |
//! | Total | 200 | | 182,000 | 10,920 | 10,920 | 1,600 | 194,520 |
//!
//! خام بکری 182,000; کل اخراجات 10,000 + 10,920 + 1,000 = 21,920; صافی بکری 182,000 - 21,920 =
//! 160,080; shared income 10,920 + 10,920 + 1,600 = 23,440, split 9,376 / 7,032 / 7,032.
//!
//! The preview and the answer of the two cases the frontend tests use are written as fixtures
//! (`bikri_preview_d8_line3`, `bikri_preview_d8_line3_rate_empty`).

use mandi_services::core::bikri::{BikriDraft, BikriPreview, RateHintSource};

use super::day_close::write_or_check;
use super::*;

/// The lot of the screenshot and its parties.
struct Case {
    w: support::World,
    seller: i64,
    mani: i64,
    anas: i64,
    haroon: i64,
    mango: i64,
}

fn case() -> Case {
    let w = worked_day();
    let seller = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "احسان",
        "Ehsan",
        true,
        "",
    );
    let buyer = |ur: &str, en: &str| {
        support::add_account(&w.env, &w.owner, AccountType::Buyer, ur, en, true, "")
    };
    let mani = buyer("مانی", "Mani");
    let anas = buyer("انس", "Anas");
    let haroon = buyer("ہارون", "Haroon");
    let mango = support::add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "unit");
    Case {
        w,
        seller,
        mani,
        anas,
        haroon,
        mango,
    }
}

impl Case {
    /// The lot of 200 with کرایہ 10,000 (paid by the firm) and مزدوری 1,000, no نقد.
    fn draft(&self, rows: Vec<mandi_services::core::bikri::SaleInput>) -> BikriDraft {
        let mut lot = support::bare_lot(&self.w, "lot-1", self.seller, self.mango, "200");
        lot.freight = Some("10,000".into());
        lot.labour = support::manual("1,000");
        lot.sales = rows;
        support::draft(vec![lot])
    }

    fn line1(&self) -> mandi_services::core::bikri::SaleInput {
        support::sale("line-1", self.mani, "20", "1000")
    }

    fn line2(&self) -> mandi_services::core::bikri::SaleInput {
        support::sale("line-2", self.anas, "50", "900")
    }

    /// The three lines of the screenshot, with the rate of line 3 typed.
    fn three_lines(&self) -> Vec<mandi_services::core::bikri::SaleInput> {
        vec![
            self.line1(),
            self.line2(),
            support::sale("line-3", self.haroon, "130", "900"),
        ]
    }
}

/// `{ "draft": ... }`, what `bikri_preview` takes inside `input`.
fn request_of(draft: &BikriDraft) -> serde_json::Value {
    serde_json::json!({ "draft": draft })
}

/// Checks (or, with MANDI_WRITE_FIXTURES, writes) the request and the real answer of a preview.
fn preview_fixture(name: &str, draft: &BikriDraft, answer: &BikriPreview) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(&request_of(draft)).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

/// The figures of every line, as (value, seller comm, buyer comm, لاگا, buyer total) in rupees.
fn line_figures(preview: &BikriPreview, line: usize) -> (i64, i64, i64, i64, i64) {
    let c = support::calc(&preview.lots[0], line);
    (
        support::r(c.buyer_value),
        support::r(c.seller_comm.amount),
        support::r(c.buyer_comm.amount),
        support::r(c.laga.amount),
        support::r(c.buyer_total),
    )
}

#[test]
fn e2e_64_a_complete_third_line_is_worked_out_and_counted_when_its_rate_equals_the_hint() {
    let c = case();
    let w = &c.w;

    // The screen after line 2: line 3 has its buyer and quantity, the rate cell is empty and shows
    // the hint, the last rate of this item and packing in this بکری, 900 (FR-7.53). The line is
    // not complete, so it has no figures and the summary counts lines 1 and 2 only.
    let waiting = c.draft(vec![
        c.line1(),
        c.line2(),
        support::sale("line-3", c.haroon, "130", ""),
    ]);
    let p = bikri::preview(&w.env.ctx, &waiting).unwrap();
    assert!(p.lots[0].lines[2].calc.is_none());
    let hint = p.lots[0].lines[2].rate_hint.as_ref().unwrap();
    assert_eq!(support::r(hint.rate_paisa), 900);
    assert_eq!(hint.source, RateHintSource::ThisBikri);
    assert_eq!(support::r(p.summary.gross), 65_000, "lines 1 and 2 only");
    preview_fixture("bikri_preview_d8_line3_rate_empty", &waiting, &p);

    // Umar types 900, the same number as the hint. The line is complete: it has its figures and
    // the lot is exactly sold, 200 of 200, with no warning and no error. A fourth, empty row (the
    // screen keeps one at the end) is left out.
    let mut rows = c.three_lines();
    rows.push(support::blank_sale("line-4", ""));
    let typed = c.draft(rows);
    let preview = bikri::preview(&w.env.ctx, &typed).unwrap();
    assert!(preview.errors.is_empty(), "errors: {:?}", preview.errors);
    assert!(
        preview.warnings.is_empty(),
        "an exactly sold lot has no warning: {:?}",
        preview.warnings
    );
    assert_eq!(preview.lots[0].sold_qty, 200);
    assert_eq!(preview.lots[0].balance_qty, Some(0));
    assert!(preview.lots[0].lines[3].is_blank);
    assert!(preview.lots[0].lines[3].calc.is_none());
    // Line by line: value, seller comm, buyer comm, لاگا, buyer total.
    assert_eq!(
        line_figures(&preview, 0),
        (20_000, 1_200, 1_200, 160, 21_360)
    );
    assert_eq!(
        line_figures(&preview, 1),
        (45_000, 2_700, 2_700, 400, 48_100)
    );
    assert_eq!(
        line_figures(&preview, 2),
        (117_000, 7_020, 7_020, 1_040, 125_060)
    );
    // The whole بکری: the three buyer totals and every line of the seller's summary.
    let s = &preview.summary;
    assert_eq!(support::r(s.buyer_total), 194_520);
    assert_eq!(support::r(s.gross), 182_000);
    assert_eq!(support::r(s.freight), 10_000);
    assert_eq!(support::r(s.seller_comm), 10_920);
    assert_eq!(support::r(s.labour), 1_000);
    assert_eq!(support::r(s.deductions), 21_920);
    assert_eq!(support::r(s.net), 160_080);
    assert_eq!(support::r(s.buyer_comm), 10_920);
    assert_eq!(support::r(s.laga), 1_600);
    assert_eq!(support::r(s.shared_income), 23_440);
    let split: Vec<i64> = preview
        .split
        .iter()
        .map(|p| support::r(p.income_paisa))
        .collect();
    assert_eq!(split, [9_376, 7_032, 7_032]);
    preview_fixture("bikri_preview_d8_line3", &typed, &preview);

    // Save: the same numbers (preview and save are one calculation, Code Standards B5). The empty
    // fourth row is dropped.
    let saved = save_ok(w, &typed, false);
    assert_eq!(saved.preview, preview);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let lot = &detail.lots[0];
    assert_eq!(lot.sales.len(), 3, "the empty row is not saved");
    assert_eq!(
        (
            lot.gross.rupees(),
            lot.seller_comm.rupees(),
            lot.deductions.rupees(),
            lot.net.rupees()
        ),
        (182_000, 10_920, 21_920, 160_080)
    );
    let totals: Vec<i64> = lot.sales.iter().map(|s| s.buyer_total.rupees()).collect();
    assert_eq!(totals, [21_360, 48_100, 125_060]);

    // The journal: three buyers, the seller, the three income accounts, labour and freight.
    let journal = entry_lines(w, saved.entry_id);
    assert_eq!(
        journal,
        vec![
            (balance_code(&c, c.mani), 21_360, 0, None),
            (balance_code(&c, c.anas), 48_100, 0, None),
            (balance_code(&c, c.haroon), 125_060, 0, None),
            (balance_code(&c, c.seller), 0, 160_080, None),
            ("04010001".to_owned(), 0, 10_920, None),
            ("04010002".to_owned(), 0, 10_920, None),
            ("04010003".to_owned(), 0, 1_600, None),
            ("02010001".to_owned(), 0, 1_000, None),
            ("02010002".to_owned(), 0, 10_000, None),
        ]
    );
    assert_eq!(journal.iter().map(|l| l.1).sum::<i64>(), 194_520);
    assert_eq!(journal.iter().map(|l| l.2).sum::<i64>(), 194_520);
    // The partner split: 9,376 / 7,032 / 7,032, a total of 23,440.
    assert_eq!(shares_of(w, saved.entry_id), [9_376, 7_032, 7_032]);
    // Each buyer owes his bill, the seller is owed 160,080, cash did not move.
    assert_eq!(balance(w, c.mani), (BalanceSide::Dr, 21_360));
    assert_eq!(balance(w, c.anas), (BalanceSide::Dr, 48_100));
    assert_eq!(balance(w, c.haroon), (BalanceSide::Dr, 125_060));
    assert_eq!(balance(w, c.seller), (BalanceSide::Cr, 160_080));
    assert_eq!(
        balance(w, support::id_of(&w.env, "01010001")),
        (BalanceSide::Dr, 50_000)
    );
    // The lot is exactly sold: 200 received and 20 + 50 + 130 sold, so one move in and three out
    // and nothing left; three facts.
    assert_eq!(scalar(w, "SELECT COUNT(*) FROM stock_moves"), 4);
    assert_eq!(
        scalar(
            w,
            "SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE kind = 'received'"
        ),
        200
    );
    assert_eq!(
        scalar(
            w,
            "SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE kind = 'sold'"
        ),
        -200
    );
    assert_eq!(
        scalar(w, "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"),
        0
    );
    assert_eq!(scalar(w, "SELECT COUNT(*) FROM analytics_facts"), 3);
    // The books balance: the opening 65,000 (cash 50,000 and رحیم's advance 15,000) plus this
    // entry's 194,520 on each side.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    assert_eq!(trial.total_dr, rupees(65_000 + 194_520));
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

/// The code of an account, for the journal lines.
fn balance_code(c: &Case, id: i64) -> String {
    raw(&c.w.dir)
        .query_row("SELECT code FROM accounts WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap()
}

#[test]
fn e2e_64_the_same_numbers_come_whichever_order_the_lines_are_typed() {
    // The rate of line 3 typed before the others, or the last line first: the sum of complete
    // lines is the same (a complete line is never lost because of its place).
    let c = case();
    let forward = bikri::preview(&c.w.env.ctx, &c.draft(c.three_lines())).unwrap();
    let mut reversed_rows = c.three_lines();
    reversed_rows.reverse();
    // Reversed, the 130 comes first: it takes the lot to 130 of 200, then 50, then 20: still 200.
    let reversed = bikri::preview(&c.w.env.ctx, &c.draft(reversed_rows)).unwrap();
    assert!(reversed.errors.is_empty(), "{:?}", reversed.errors);
    for (a, b) in [
        (support::r(forward.summary.gross), 182_000),
        (support::r(forward.summary.buyer_total), 194_520),
        (support::r(forward.summary.net), 160_080),
        (support::r(forward.summary.shared_income), 23_440),
    ] {
        assert_eq!(a, b);
    }
    assert_eq!(support::r(reversed.summary.gross), 182_000);
    assert_eq!(support::r(reversed.summary.buyer_total), 194_520);
    assert_eq!(support::r(reversed.summary.net), 160_080);
    assert_eq!(support::r(reversed.summary.shared_income), 23_440);
}

#[test]
fn e2e_64_over_quantity_is_refused_at_once_and_an_incomplete_line_is_not_counted() {
    // FR-12.6: ہارون typed 131 is one more than the lot has left. The line has no figures, the
    // error is under its quantity, and the lot totals leave it out.
    let c = case();
    let over = c.draft(vec![
        c.line1(),
        c.line2(),
        support::sale("line-3", c.haroon, "131", "900"),
    ]);
    let p = bikri::preview(&c.w.env.ctx, &over).unwrap();
    assert!(p.lots[0].lines[2].calc.is_none());
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[0].sales[2].qty"
    ));
    assert_eq!(support::r(p.summary.gross), 65_000);
    // A line with a rate and a quantity but no buyer is incomplete: it has no figures and the
    // summary leaves it out.
    let mut no_buyer = support::sale("line-3", 0, "130", "900");
    no_buyer.buyer_account_id = None;
    let incomplete = c.draft(vec![c.line1(), c.line2(), no_buyer]);
    let p = bikri::preview(&c.w.env.ctx, &incomplete).unwrap();
    assert!(p.lots[0].lines[2].calc.is_none());
    assert!(!p.lots[0].lines[2].is_blank);
    assert_eq!(support::r(p.summary.gross), 65_000);
}
