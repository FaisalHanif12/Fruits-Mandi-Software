//! Plan 79 (PRD v1.44, FR-9.1, FR-9.6 to FR-9.8, FR-7.44): the pages that open from the dashboard
//! tiles. Each page's total is the tile's figure, read from the same posted records. Every expected
//! number is the PRD's worked day (4.4, 4.5, 4.9, E2E-28, E2E-30, E2E-44) or worked out in the
//! comments, and is never changed to suit the code.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::dashboard::{
    self, AttentionKind, BackupFacts, BuyerDuesInput, CashSalesDayInput, CashSalesMonthInput,
    LentOpenInput, SellerDuesInput, SellerDuesSide, TotalDuesSection,
};
use mandi_reports::reports::PrintCell;
use mandi_reports::reports::{ReportFilters, ReportId};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::CancelInput;
use mandi_services::core::dead::{DeadMarkInput, DeadReason};
use mandi_services::core::error::ErrorCode;
use mandi_services::{bikri, dead, vouchers};
use reports_world::*;

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

fn tiles(w: &support::World) -> dashboard::DashboardTiles {
    dashboard::get(w.env.ctx.db(), off()).unwrap().tiles
}

fn seller_page(
    w: &support::World,
    side: SellerDuesSide,
    party_id: Option<i64>,
) -> dashboard::SellerDuesPage {
    dashboard::seller_dues(
        w.env.ctx.db(),
        &SellerDuesInput {
            side,
            party_id,
            ..SellerDuesInput::default()
        },
    )
    .unwrap()
}

fn lend(w: &support::World, amount: &str) -> i64 {
    let person = golden_aslam(w);
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::lent::MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(person),
            amount: amount.into(),
            reason: "گھریلو ضرورت".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    person
}

/// Makes the person اسلم (staff type none: a plain person) and returns his account id.
fn golden_aslam(w: &support::World) -> i64 {
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = mandi_services::core::lent::LentPersonCreateInput {
            name_ur: "اسلم".into(),
            name_en: "Aslam".into(),
            phone: Some("0300-1234567".into()),
            staff_id: None,
            confirmed_rules_words: words.clone(),
        };
        match mandi_services::lent::person_create(&w.env.ctx, &w.owner, &input) {
            Ok(person) => return person.account.id,
            Err(mandi_services::ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(text)) =
                        item.values.get("words")
                    {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

// ---- آج کی نقدی (FR-9.8) ----------------------------------------------------------------------

#[test]
fn the_cash_sales_tile_month_and_day_agree_on_the_worked_day() {
    // PRD 4.5 and 4.9: شاہد (چکتا) paid all 12,800 on the spot; اکرم and ارشد paid nothing.
    let w = full_day();
    let t = tiles(&w);
    assert_eq!(
        (t.cash_sales_today_paisa.rupees(), t.cash_sales_today_buyers),
        (12_800, 1)
    );
    let month =
        dashboard::cash_sales_month(w.env.ctx.db(), &CashSalesMonthInput::default()).unwrap();
    assert_eq!(month.month, "2026-10");
    assert_eq!(month.rows.len(), 1);
    assert_eq!(month.rows[0].date, "2026-10-10");
    assert_eq!(
        (month.rows[0].buyer_count, month.rows[0].cash_paisa.rupees()),
        (1, 12_800)
    );
    assert_eq!(
        (month.buyer_days_total, month.total_paisa.rupees()),
        (1, 12_800)
    );
    let day = dashboard::cash_sales_day(
        w.env.ctx.db(),
        &CashSalesDayInput {
            date: "10-10-2026".into(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!((day.total_count, day.buyer_count), (1, 1));
    let row = &day.rows[0];
    assert!(row.is_chukta);
    assert_eq!(row.party.name_en, "Shahid");
    assert!(row.party.person_key.is_some());
    assert_eq!(
        (
            row.bikri_number,
            row.bill_paisa.rupees(),
            row.cash_paisa.rupees()
        ),
        (2, 12_800, 12_800)
    );
    assert_eq!(
        (day.bill_total_paisa.rupees(), day.cash_total_paisa.rupees()),
        (12_800, 12_800)
    );
    // The page total of the day is the tile and the R3 paid total.
    assert_eq!(day.cash_total_paisa, t.cash_sales_today_paisa);
    let r3 = report(&w, ReportId::R3, ReportFilters::default(), 0, 100);
    let paid_col = r3.columns.iter().position(|c| c.key == "paid").unwrap();
    match &r3.totals[paid_col] {
        mandi_reports::reports::Cell::Money { paisa } => {
            assert_eq!(*paisa, t.cash_sales_today_paisa)
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_cancelled_sale_leaves_the_cash_sales_and_a_future_month_or_day_is_refused() {
    let w = full_day();
    let detail = bikri::get(&w.env.ctx, 2).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: 2,
            version: detail.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let t = tiles(&w);
    assert_eq!(
        (t.cash_sales_today_paisa.rupees(), t.cash_sales_today_buyers),
        (0, 0)
    );
    let month =
        dashboard::cash_sales_month(w.env.ctx.db(), &CashSalesMonthInput::default()).unwrap();
    assert!(month.rows.is_empty());
    assert_eq!(month.total_paisa.rupees(), 0);
    let future = dashboard::cash_sales_month(
        w.env.ctx.db(),
        &CashSalesMonthInput {
            month: Some("2026-11".into()),
        },
    )
    .unwrap_err();
    assert_eq!(future.code(), ErrorCode::ErrDateInFuture);
    let bad = dashboard::cash_sales_month(
        w.env.ctx.db(),
        &CashSalesMonthInput {
            month: Some("2026-13".into()),
        },
    )
    .unwrap_err();
    assert_eq!(bad.code(), ErrorCode::ErrFormatInvalid);
    let later = dashboard::cash_sales_day(
        w.env.ctx.db(),
        &CashSalesDayInput {
            date: "11-10-2026".into(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap_err();
    assert_eq!(later.code(), ErrorCode::ErrDateInFuture);
}

// ---- Seller dues (FR-9.6) -----------------------------------------------------------------------

#[test]
fn the_seller_page_totals_are_the_two_seller_tiles() {
    // After Events 1 to 5: کریم is owed 10,160 (جمع); رحیم is paid up and has no advance left.
    let w = full_day();
    let t = tiles(&w);
    let all = seller_page(&w, SellerDuesSide::All, None);
    assert_eq!(all.total_count, 1);
    assert_eq!(all.rows[0].party.name_en, "Karim");
    assert_eq!(all.rows[0].payable_paisa.rupees(), 10_160);
    assert_eq!(all.rows[0].balance.side, BalanceSide::Cr);
    assert_eq!(all.shop_owes_paisa, t.sellers_payable_paisa);
    assert_eq!(all.seller_owes_paisa, t.sellers_dues_paisa);
    assert_eq!(all.advances_open_paisa, t.advances_open_paisa);
    assert_eq!(all.shop_owes_paisa.rupees(), 10_160);
    assert_eq!(all.seller_owes_paisa.rupees(), 0);
    assert_eq!(
        (all.total.amount_paisa.rupees(), all.total.side),
        (10_160, BalanceSide::Cr)
    );
    // The side filters cut the rows, not the totals of both sides.
    let shop = seller_page(&w, SellerDuesSide::ShopOwes, None);
    assert_eq!(shop.total_count, 1);
    assert_eq!(shop.shop_owes_paisa.rupees(), 10_160);
    let owes = seller_page(&w, SellerDuesSide::SellerOwes, None);
    assert_eq!(owes.total_count, 0);
    assert_eq!(owes.shop_owes_paisa.rupees(), 10_160);
    // One seller (party_id) and سب (null).
    let one = seller_page(&w, SellerDuesSide::All, Some(w.karim));
    assert_eq!(one.total_count, 1);
    let other = seller_page(&w, SellerDuesSide::All, Some(w.rahim));
    assert_eq!(other.total_count, 0);
    assert_eq!(other.shop_owes_paisa.rupees(), 0);
}

#[test]
fn a_seller_with_an_open_advance_shows_all_three_figures() {
    // E2E-28: the advance tick is off. رحیم's payable is 24,316 - 5,000 = 19,316, his advance
    // 15,000 stays open, his ledger is 4,316 جمع.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save(&w, &draft);
    let t = tiles(&w);
    let page = seller_page(&w, SellerDuesSide::All, None);
    assert_eq!(page.total_count, 1);
    let row = &page.rows[0];
    assert_eq!(
        (
            row.payable_paisa.rupees(),
            row.advance_open_paisa.rupees(),
            row.owes_paisa.rupees(),
            row.balance.amount_paisa.rupees(),
            row.balance.side
        ),
        (19_316, 15_000, 0, 4_316, BalanceSide::Cr)
    );
    assert_eq!(page.shop_owes_paisa, t.sellers_payable_paisa);
    assert_eq!(page.advances_open_paisa, t.advances_open_paisa);
    // He is paid the 19,316: payable 0, ledger 15,000 بنام, the advance 15,000 still open, and he
    // owes the shop nothing beyond it. He is still a row (a balance), on سب and in no side.
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "19,316")).unwrap();
    let after = seller_page(&w, SellerDuesSide::All, None);
    assert_eq!(after.total_count, 1);
    assert_eq!(
        (
            after.total.amount_paisa.rupees(),
            after.total.side,
            after.shop_owes_paisa.rupees(),
            after.seller_owes_paisa.rupees(),
            after.advances_open_paisa.rupees()
        ),
        (15_000, BalanceSide::Dr, 0, 0, 15_000)
    );
    assert_eq!(
        seller_page(&w, SellerDuesSide::ShopOwes, None).total_count,
        0
    );
    assert_eq!(
        seller_page(&w, SellerDuesSide::SellerOwes, None).total_count,
        0
    );
    let t = tiles(&w);
    assert_eq!(after.advances_open_paisa, t.advances_open_paisa);
}

#[test]
fn the_seller_history_opens_with_his_opening_balance_and_ends_at_his_page_balance() {
    // PRD 6 s01: احسان is made with an opening balance of 20,000 جمع. The history of a seller is
    // `statement_rows`: ابتدائی بقایا first, and the last balance is the balance of his row on the
    // seller page and of his party card.
    use mandi_services::core::accounts::{AccountType, OpeningLineInput, OpeningsInput};
    let w = worked_day();
    let ehsan = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "احسان",
        "Ehsan",
        true,
        "",
    );
    mandi_services::accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: ehsan,
                version: mandi_services::accounts::get(&w.env.ctx, ehsan)
                    .unwrap()
                    .version,
                opening_date: "10-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "20,000".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let rows = mandi_reports::reports::statement_rows(
        w.env.ctx.db(),
        &mandi_reports::reports::StatementRowsInput {
            account_id: ehsan,
            from: None,
            to: None,
            include_history: false,
            newest_first: false,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let first = rows.rows.first().unwrap();
    assert_eq!(first.type_id, "report-doc-opening");
    assert_eq!(first.date, "2026-10-10");
    assert_eq!(
        (
            first.cr_paisa.rupees(),
            first.balance.amount_paisa.rupees(),
            first.balance.side
        ),
        (20_000, 20_000, BalanceSide::Cr)
    );
    assert_eq!(rows.closing, rows.rows.last().unwrap().balance);
    // His row on the seller page: the opening credit is payable, ledger 20,000 جمع.
    let page = seller_page(&w, SellerDuesSide::All, Some(ehsan));
    assert_eq!(page.total_count, 1);
    assert_eq!(page.rows[0].balance, rows.closing);
    assert_eq!(page.rows[0].payable_paisa.rupees(), 20_000);
    assert_eq!(page.shop_owes_paisa, tiles(&w).sellers_payable_paisa);
}

// ---- Buyer dues, with a party (plan 79 item 1) --------------------------------------------------

#[test]
fn the_buyer_page_takes_one_buyer_or_all() {
    let w = full_day();
    let all = dashboard::buyer_dues(w.env.ctx.db(), &BuyerDuesInput::default()).unwrap();
    assert_eq!((all.total_count, all.total_paisa.rupees()), (2, 21_564));
    assert_eq!(all.total_paisa, tiles(&w).buyers_dues_paisa);
    let one = dashboard::buyer_dues(
        w.env.ctx.db(),
        &BuyerDuesInput {
            party_id: Some(w.akram),
            ..BuyerDuesInput::default()
        },
    )
    .unwrap();
    assert_eq!((one.total_count, one.total_paisa.rupees()), (1, 2_880));
}

// ---- کل اگراہی and ادھار رقم باقی (FR-9.7) --------------------------------------------------------

fn part(total: &dashboard::TotalDues, section: TotalDuesSection) -> (i64, i64) {
    let p = total.parts.iter().find(|p| p.section == section).unwrap();
    (p.count, p.total_paisa.rupees())
}

#[test]
fn total_dues_has_four_parts_and_the_grand_total_is_the_tile() {
    let w = full_day();
    let total = dashboard::total_dues_parts(w.env.ctx.db()).unwrap();
    assert_eq!(total.parts.len(), 4);
    assert_eq!(part(&total, TotalDuesSection::Buyers), (2, 21_564));
    assert_eq!(part(&total, TotalDuesSection::Sellers), (0, 0));
    assert_eq!(part(&total, TotalDuesSection::Dead), (0, 0));
    assert_eq!(part(&total, TotalDuesSection::Others), (0, 0));
    assert_eq!(total.grand_total_paisa.rupees(), 21_564);
    assert_eq!(total.grand_total_paisa, tiles(&w).total_dues_paisa);

    // E2E-30: اسلم is lent 5,000 and has returned nothing. Open money lent is the fourth part.
    let person = lend(&w, "5,000");
    let total = dashboard::total_dues_parts(w.env.ctx.db()).unwrap();
    assert_eq!(part(&total, TotalDuesSection::Others), (1, 5_000));
    assert_eq!(total.grand_total_paisa.rupees(), 26_564);
    let t = tiles(&w);
    assert_eq!(total.grand_total_paisa, t.total_dues_paisa);
    let page = dashboard::lent_open(w.env.ctx.db(), &LentOpenInput::default()).unwrap();
    assert_eq!((page.total_count, page.total_paisa.rupees()), (1, 5_000));
    assert_eq!(page.total_paisa, t.lent_open_paisa);
    assert_eq!(page.rows[0].party.account_id, person);
    assert_eq!(page.rows[0].open_paisa.rupees(), 5_000);
    assert_eq!(page.rows[0].oldest_open_date.as_deref(), Some("2026-10-10"));

    // He leaves the business: his 5,000 moves to the dead amount. کل اگراہی does not change.
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: person,
            entry_date: "10-10-2026".into(),
            reason: DeadReason::Left,
            note: String::new(),
        },
    )
    .unwrap();
    let total = dashboard::total_dues_parts(w.env.ctx.db()).unwrap();
    assert_eq!(part(&total, TotalDuesSection::Others), (0, 0));
    assert_eq!(part(&total, TotalDuesSection::Dead), (1, 5_000));
    assert_eq!(total.grand_total_paisa.rupees(), 26_564);
    assert_eq!(total.grand_total_paisa, tiles(&w).total_dues_paisa);
}

// ---- The health check on the bell (NFR-OBS-5) ---------------------------------------------------

fn set_health(w: &support::World, json: &str) {
    w.env
        .ctx
        .db()
        .write(|tx| {
            tx.execute(
                "INSERT INTO settings (key, value_json, created_by, updated_by, version)
                 VALUES ('health_last', ?1, 1, 1, 1)
                 ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
                [json],
            )
            .map_err(mandi_services::db::DbError::from)
        })
        .unwrap();
}

fn has_health_item(w: &support::World) -> bool {
    dashboard::attention(w.env.ctx.db(), off())
        .unwrap()
        .items
        .iter()
        .any(|i| i.kind == AttentionKind::HealthCheck)
}

#[test]
fn a_failed_health_check_is_an_attention_item_and_a_good_one_is_not() {
    let w = full_day();
    assert!(!has_health_item(&w), "no result yet");
    let good = r#"{"checked_at":"2026-10-10T12:00:00Z","ok":true,
        "books":{"ran":true,"balanced":true,"difference_paisa":0,
                 "journal_chain":{"ok":true,"at":null},"audit_chain":{"ok":true,"at":null},
                 "journal_entries":7,"audit_rows":20},
        "data":{"state":"ok","reference":null},"backup":null}"#;
    set_health(&w, good);
    assert!(!has_health_item(&w));
    let unbalanced = good.replace(r#""balanced":true"#, r#""balanced":false"#);
    set_health(&w, &unbalanced);
    assert!(has_health_item(&w));
    let broken = good.replace(
        r#""journal_chain":{"ok":true,"at":null}"#,
        r#""journal_chain":{"ok":false,"at":"je7"}"#,
    );
    set_health(&w, &broken);
    assert!(has_health_item(&w));
    let data_failed = good.replace(r#""state":"ok""#, r#""state":"failed""#);
    set_health(&w, &data_failed);
    assert!(has_health_item(&w));
    // An old cloud copy alone is the item BackupOld, not this one.
    let cloud_only = good.replace(r#""ok":true,"books""#, r#""ok":false,"books""#);
    set_health(&w, &cloud_only);
    assert!(!has_health_item(&w));
    // The bell counts it once.
    set_health(&w, &unbalanced);
    let list = dashboard::attention(w.env.ctx.db(), off()).unwrap();
    let item = list
        .items
        .iter()
        .find(|i| i.kind == AttentionKind::HealthCheck)
        .unwrap();
    assert_eq!(item.count, 1);
}

// ---- Query plans (Code Standards B18, NFR-SPD-24) -----------------------------------------------

#[test]
fn the_cash_sales_reads_use_the_date_index_of_the_posting_and_no_scan_of_the_sale_lines() {
    // The month and the day are found through `journal_entries(entry_date)`, then the lines of those
    // few postings through `bikri_sales(journal_entry_id)`: ten years of sale lines are never read.
    let w = full_day();
    for (sql, args) in [
        (dashboard::cash_sales_plans::days(), 2),
        (dashboard::cash_sales_plans::day_buyers(), 1),
    ] {
        let plan = w
            .env
            .ctx
            .db()
            .read(|conn| {
                let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}"))?;
                let text = if args == 2 {
                    statement
                        .query_map(["2016-10-11", "2026-10-10"], |r| r.get::<_, String>(3))?
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    statement
                        .query_map(["2026-10-10"], |r| r.get::<_, String>(3))?
                        .collect::<Result<Vec<_>, _>>()?
                };
                Ok::<_, mandi_services::db::DbError>(text.join(" | "))
            })
            .unwrap();
        assert!(plan.contains("idx_journal_entries_entry_date"), "{plan}");
        assert!(
            !plan.contains("SCAN s") && !plan.contains("SCAN bikri_sales"),
            "{plan}"
        );
    }
}

// ---- Print data (FR-19.10): every row, the totals of the whole filter ---------------------------

fn money_text(cell: &PrintCell) -> String {
    match cell {
        PrintCell::Money { text } | PrintCell::Number { text } => text.clone(),
        other => panic!("not an amount: {other:?}"),
    }
}

#[test]
fn the_total_dues_print_has_the_four_parts_and_the_grand_total_of_the_tile() {
    let w = full_day();
    lend(&w, "5,000");
    let print = dashboard::total_dues_print_data(
        w.env.ctx.db(),
        &dashboard::TotalDuesPrintInput {
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(print.rows.len(), 4);
    let totals: Vec<String> = print.rows.iter().map(|r| money_text(&r.cells[2])).collect();
    assert_eq!(totals, ["21,564", "0", "0", "5,000"]);
    assert_eq!(money_text(&print.totals[2]), "26,564");
    assert_eq!(
        tiles(&w).total_dues_paisa.rupees(),
        26_564,
        "the print total is the tile"
    );
}

#[test]
fn the_lent_open_print_lists_every_person_with_the_total_of_the_filter() {
    let w = full_day();
    let person = lend(&w, "5,000");
    let print = dashboard::lent_open_print_data(
        w.env.ctx.db(),
        &dashboard::LentOpenPrintInput {
            search: None,
            sort: None,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(print.rows.len(), 1);
    assert_eq!(money_text(&print.rows[0].cells[2]), "5,000");
    assert_eq!(money_text(&print.totals[2]), "5,000");
    let page = dashboard::lent_open(w.env.ctx.db(), &LentOpenInput::default()).unwrap();
    assert_eq!(page.rows[0].party.account_id, person);
    // A search that matches nobody prints an empty table with a total of 0.
    let none = dashboard::lent_open_print_data(
        w.env.ctx.db(),
        &dashboard::LentOpenPrintInput {
            search: Some("zzzz".into()),
            sort: None,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(none.rows.len(), 0);
}

#[test]
fn the_cash_sales_prints_hold_the_month_and_the_day_totals() {
    let w = full_day();
    let month = dashboard::cash_sales_month_print_data(
        w.env.ctx.db(),
        &dashboard::CashSalesMonthPrintInput {
            month: None,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(month.rows.len(), 1);
    assert_eq!(money_text(&month.rows[0].cells[2]), "12,800");
    assert_eq!(money_text(&month.totals[1]), "1");
    assert_eq!(money_text(&month.totals[2]), "12,800");
    let day = dashboard::cash_sales_day_print_data(
        w.env.ctx.db(),
        &dashboard::CashSalesDayPrintInput {
            date: "10-10-2026".into(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(day.rows.len(), 1);
    assert_eq!(money_text(&day.totals[2]), "12,800");
    assert_eq!(money_text(&day.totals[3]), "12,800");
    // A future day is refused as on the page.
    let later = dashboard::cash_sales_day_print_data(
        w.env.ctx.db(),
        &dashboard::CashSalesDayPrintInput {
            date: "11-10-2026".into(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap_err();
    assert_eq!(later.code(), ErrorCode::ErrDateInFuture);
}

#[test]
fn the_advances_print_follows_the_page_filter_and_only_open() {
    use mandi_services::core::vouchers::AdvancesReportInput;
    // E2E-28: the advance tick is off, رحیم is paid 19,316: his 15,000 advance is still open.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save(&w, &draft);
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "19,316")).unwrap();
    let report = |only_open: bool| {
        vouchers::advances_report(
            &w.env.ctx,
            &AdvancesReportInput {
                seller_id: None,
                as_of: String::new(),
                offset: 0,
                limit: u32::MAX,
                search: None,
                only_open,
            },
        )
        .unwrap()
    };
    let open = report(true);
    assert_eq!((open.total_count, open.open_paisa.rupees()), (1, 15_000));
    assert_eq!(open.open_paisa, tiles(&w).advances_open_paisa);
    let print =
        dashboard::advances_print_data(w.env.ctx.db(), &open, None, Language::En, None).unwrap();
    assert_eq!(print.rows.len(), 1);
    assert_eq!(money_text(&print.rows[0].cells[1]), "15,000");
    assert_eq!(money_text(&print.totals[4]), "15,000");
    // Pay nothing more, adjust nothing: the same seller on the page of all sellers has the same row.
    assert_eq!(report(false).total_count, 1);
}
