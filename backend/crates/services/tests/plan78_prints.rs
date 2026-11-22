//! Plan 78 Part A and B on the PRD worked day: the ادھاری تفصیل (A5), the day slip opening line
//! (A6), the whole sale print (A4) and the statement rows (A7, B1 to B3). Every expected number is
//! the PRD's (4.4, 4.5, 4.9) or the plan's example, and is never changed to suit the code.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_reports::reports::{
    AdhariSheetInput, AdhariSheetPrintInput, adhari_sheet, adhari_sheet_print_data,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::core::money::Paisa;
use reports_world::full_day;

fn rupees_of(p: Paisa) -> i64 {
    p.as_paisa() / 100
}

fn adhari(w: &support::World, date: &str) -> mandi_reports::reports::AdhariSheet {
    adhari_sheet(
        w.env.ctx.db(),
        &AdhariSheetInput {
            date: date.into(),
            ..AdhariSheetInput::default()
        },
    )
    .unwrap()
}

// ---- A5: the ادھاری تفصیل ----

#[test]
fn the_adhari_sheet_of_the_worked_day_has_todays_buyers_and_the_whole_amount_owed() {
    let w = full_day();
    let sheet = adhari(&w, "10-10-2026");
    // Rows are today's buyers only, with the plan's example figures: bill, today's ادھار, پچھلا بقایا
    // and کل ادھار. There is no نقد column (PRD R5 v1.57, plan 95 item 17).
    let rows: Vec<(String, [i64; 4])> = sheet
        .rows
        .iter()
        .map(|r| {
            (
                r.party.name_en.clone(),
                [
                    rupees_of(r.bill_paisa),
                    rupees_of(r.today_credit_paisa),
                    rupees_of(r.previous_due.amount_paisa),
                    rupees_of(r.total_due.amount_paisa),
                ],
            )
        })
        .collect();
    // PRD R5 v1.57 (plan 95 item 12): a چکتا buyer (شاہد, paid 12,800 on the spot) is not a row: he
    // has no ادھار and no khata.
    assert_eq!(sheet.total_count, 2);
    assert!(rows.contains(&("Akram Toll Tax".into(), [12_880, 12_880, 0, 12_880])));
    assert!(rows.contains(&("Arshad Multani".into(), [18_684, 18_684, 0, 18_684])));
    assert!(rows.iter().all(|(name, _)| name != "Shahid"));
    let totals = |t: &mandi_reports::reports::AdhariSheetTotals| {
        [
            rupees_of(t.bill_paisa),
            rupees_of(t.today_credit_paisa),
            rupees_of(t.previous_due.amount_paisa),
            rupees_of(t.total_due.amount_paisa),
        ]
    };
    assert_eq!(totals(&sheet.totals), [31_564, 31_564, 0, 31_564]);
    // Nobody else owes, so کل ادھار is the same, and there is no "others" total.
    assert!(sheet.others.is_none() && sheet.others_count == 0);
    assert_eq!(totals(&sheet.all_totals), [31_564, 31_564, 0, 31_564]);
    assert!(sheet.dead_totals.is_none() && sheet.credit_totals.is_none());
}

#[test]
fn the_next_day_with_no_sale_has_no_rows_and_the_others_are_the_total_of_r20() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    let sheet = adhari(&w, "11-10-2026");
    // اکرم 2,880 and ارشد 18,684 owe, and bought nothing: they are the others, 21,564 = R20.
    assert!(sheet.rows.is_empty());
    assert_eq!(sheet.total_count, 0);
    assert_eq!(sheet.others_count, 2);
    let others = sheet.others.as_ref().unwrap();
    assert_eq!(rupees_of(others.previous_due.amount_paisa), 21_564);
    assert_eq!(rupees_of(others.total_due.amount_paisa), 21_564);
    assert_eq!(rupees_of(sheet.all_totals.total_due.amount_paisa), 21_564);
    assert_eq!(sheet.all_totals.total_due.side, BalanceSide::Dr);
    let tile = mandi_reports::dashboard::get(
        w.env.ctx.db(),
        mandi_reports::dashboard::BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles
    .buyers_dues_paisa;
    assert_eq!(sheet.all_totals.total_due.amount_paisa, tile);
}

#[test]
fn the_adhari_print_has_every_buyer_the_others_row_and_the_whole_total() {
    let w = full_day();
    for language in [Language::Ur, Language::En] {
        let print = adhari_sheet_print_data(
            w.env.ctx.db(),
            &AdhariSheetPrintInput {
                date: "10-10-2026".into(),
                language,
                paper: None,
            },
        )
        .unwrap();
        // اکرم and ارشد; the چکتا buyer شاہد is not a row (PRD R5 v1.57).
        assert_eq!(print.rows.len(), 2);
        // party, goods, bill, today's ادھار, پچھلا بقایا, کل ادھار: no نقد column (PRD R5 v1.57).
        assert_eq!(print.columns.len(), 6);
        assert!(print.columns.iter().all(|c| c.key != "cash"));
        assert_eq!(print.frame.header.title_id, "print-adhari-sheet-title");
        // No others today: the total of today's buyers, then کل ادھار.
        assert_eq!(print.extra_totals.len(), 1);
        assert_eq!(print.extra_totals[0].label_id, "print-adhari-sheet-all");
        assert!(print.words.is_some());
    }
    // A future day is refused.
    assert!(
        adhari_sheet(
            w.env.ctx.db(),
            &AdhariSheetInput {
                date: "12-10-2026".into(),
                ..AdhariSheetInput::default()
            }
        )
        .is_err()
    );
}

// ---- A6: an opening balance on a day slip is its own line ----

fn id_named(w: &support::World, name_en: &str) -> i64 {
    w.env
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT id FROM accounts WHERE name_en = ?1",
                [name_en],
                |r| r.get::<_, i64>(0),
            )
        })
        .unwrap()
}

fn summary_of(
    page: &mandi_reports::reports::ReportPage,
    label: &str,
) -> Option<mandi_reports::reports::Cell> {
    page.summary
        .iter()
        .find(|s| s.label_id == label)
        .map(|s| s.cell.clone())
}

fn balance_of(cell: &mandi_reports::reports::Cell) -> (i64, BalanceSide) {
    match cell {
        mandi_reports::reports::Cell::Balance { paisa, side } => (rupees_of(*paisa), *side),
        mandi_reports::reports::Cell::Money { paisa } => (rupees_of(*paisa), BalanceSide::Dr),
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_opening_dated_the_day_of_the_sale_is_its_own_line_on_the_slip() {
    // E2E-65 (D8): ہارون holds 10,000 جمع, an opening dated 10-10-26, and buys 125,060 that day.
    let (w, _bikri) = reports_world::d8_paid_world();
    let haroon = id_named(&w, "Haroon");
    let page = reports_world::report(
        &w,
        mandi_reports::reports::ReportId::R35,
        mandi_reports::reports::ReportFilters {
            as_of: "10-10-2026".into(),
            party: Some(mandi_reports::reports::PartyFilter {
                account_id: haroon,
                person_key: None,
            }),
            ..Default::default()
        },
        0,
        0,
    );
    // previous 0, opening 10,000 جمع, today's total 125,060, nothing paid: left 125,060, new
    // balance 115,060 بنام (the PRD number of E2E-65).
    assert_eq!(
        balance_of(&summary_of(&page, "report-r35-previous").unwrap()),
        (0, BalanceSide::Zero)
    );
    assert_eq!(
        balance_of(&summary_of(&page, "report-r35-opening").unwrap()),
        (10_000, BalanceSide::Cr)
    );
    assert_eq!(
        balance_of(&summary_of(&page, "report-r35-today-left").unwrap()),
        (125_060, BalanceSide::Dr)
    );
    assert_eq!(
        balance_of(&summary_of(&page, "report-r35-new-balance").unwrap()),
        (115_060, BalanceSide::Dr)
    );
    // The opening is not one of the "other entries".
    assert!(summary_of(&page, "report-r35-other").is_none());
    // A party with no opening that day has no opening line (اکرم of the worked day is unchanged).
    let w2 = full_day();
    let akram = id_named(&w2, "Akram Toll Tax");
    let page = reports_world::report(
        &w2,
        mandi_reports::reports::ReportId::R35,
        mandi_reports::reports::ReportFilters {
            as_of: "10-10-2026".into(),
            party: Some(mandi_reports::reports::PartyFilter {
                account_id: akram,
                person_key: None,
            }),
            ..Default::default()
        },
        0,
        0,
    );
    assert!(summary_of(&page, "report-r35-opening").is_none());
}

// ---- A4: the whole sale print ----

use mandi_reports::reports::{DaySlipsInput, SlipSections, sale_print, seller_bills};

fn slips_input(bikri: Option<i64>) -> DaySlipsInput {
    DaySlipsInput {
        date: "10-10-2026".into(),
        bikri_id: bikri,
        bikri_ids: None,
        buyer_ids: None,
        sections: SlipSections::Both,
        language: Language::En,
        paper: None,
    }
}

/// The printed text of a cell of a print row, with the grouping stripped.
fn text_of(cell: &mandi_reports::reports::PrintCell) -> String {
    format!("{cell:?}")
}

#[test]
fn the_sale_print_has_a_sellers_table_and_a_buyers_table_that_agree_with_the_prd() {
    let w = reports_world::worked_day();
    let first = reports_world::save(&w, &support::event1(&w));
    reports_world::save(&w, &reports_world::event2(&w));
    // The whole day: رحیم 10,084 of deductions and 19,316 net, کریم 1,840 and 10,160 (PRD 4.4, 4.5).
    let print = sale_print(w.env.ctx.db(), &slips_input(None)).unwrap();
    assert_eq!(print.date, "2026-10-10");
    assert_eq!(print.sellers.rows.len(), 2);
    assert!(print.sellers.tables.is_empty());
    let totals: Vec<String> = print.sellers.totals.iter().map(text_of).collect();
    let joined = totals.join(" ");
    for want in ["41,400", "11,924", "29,476"] {
        assert!(joined.contains(want), "{want} in {joined}");
    }
    let rahim = print.sellers.rows[0]
        .cells
        .iter()
        .map(text_of)
        .collect::<Vec<_>>()
        .join(" ");
    for want in [
        "29,400", "3,000", "1,764", "250", "5,000", "10,084", "19,316",
    ] {
        assert!(rahim.contains(want), "{want} in {rahim}");
    }
    // The buyers: اکرم 12,880, ارشد 18,684, شاہد 12,800 paid on the spot; total 44,364, نقد 12,800,
    // ادھار 31,564.
    assert_eq!(print.buyers.rows.len(), 3);
    let bt = print
        .buyers
        .totals
        .iter()
        .map(text_of)
        .collect::<Vec<_>>()
        .join(" ");
    for want in ["44,364", "12,800", "31,564"] {
        assert!(bt.contains(want), "{want} in {bt}");
    }
    // One بکری only: رحیم's lot and his two buyers.
    let one = sale_print(w.env.ctx.db(), &slips_input(Some(first.bikri_id))).unwrap();
    assert_eq!(one.sellers.rows.len(), 1);
    assert_eq!(one.buyers.rows.len(), 2);
    let bt = one
        .buyers
        .totals
        .iter()
        .map(text_of)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(bt.contains("31,564"), "{bt}");
    // A future day is refused.
    let mut future = slips_input(None);
    future.date = "12-10-2026".into();
    assert!(sale_print(w.env.ctx.db(), &future).is_err());
}

/// PRD FR-19.15 v1.57 (plan 95 item 13): the buyers table of the whole sale print shows the value of
/// the goods, the commission and the لاگا beside the bill, so value + commission + لاگا = bill.
/// Worked numbers PRD 4.4 and 4.5.
#[test]
fn the_sale_print_buyers_table_has_value_commission_laga_and_a_bill_that_adds_up() {
    let w = reports_world::worked_day();
    reports_world::save(&w, &support::event1(&w));
    reports_world::save(&w, &reports_world::event2(&w));
    let print = sale_print(w.env.ctx.db(), &slips_input(None)).unwrap();
    let keys: Vec<&str> = print
        .buyers
        .columns
        .iter()
        .map(|c| c.key.as_str())
        .collect();
    assert_eq!(
        keys,
        [
            "party",
            "detail",
            "value",
            "commission",
            "laga",
            "bill",
            "cash",
            "credit"
        ]
    );
    let money = |row: &mandi_reports::reports::PrintReportRow| -> Vec<String> {
        row.cells[2..].iter().map(text_of).collect()
    };
    // اکرم 12,000 + 720 + 160 = 12,880; ارشد 17,400 + 1,044 + 240 = 18,684; شاہد (چکتا) 12,000 + 720 +
    // 80 = 12,800, paid in full.
    let mut got: Vec<Vec<String>> = print.buyers.rows.iter().map(money).collect();
    got.sort();
    let mut want: Vec<Vec<&str>> = vec![
        vec!["12,000", "720", "160", "12,880", "0", "12,880"],
        vec!["17,400", "1,044", "240", "18,684", "0", "18,684"],
        vec!["12,000", "720", "80", "12,800", "12,800", "0"],
    ];
    want.sort();
    assert_eq!(got.len(), want.len());
    for (row, expected) in got.iter().zip(&want) {
        for (cell, figure) in row.iter().zip(expected) {
            assert!(cell.contains(figure), "{figure} in {cell}");
        }
    }
    // Totals: value 41,400, commission 2,484, لاگا 480, bill 44,364, نقد 12,800, ادھار 31,564.
    let totals: Vec<String> = print.buyers.totals.iter().map(text_of).collect();
    assert_eq!(totals.len(), 8);
    for (cell, figure) in totals[2..]
        .iter()
        .zip(["41,400", "2,484", "480", "44,364", "12,800", "31,564"])
    {
        assert!(cell.contains(figure), "{figure} in {cell}");
    }
}

#[test]
fn the_seller_bills_are_the_seller_slips_with_the_named_title() {
    let w = full_day();
    let bills = seller_bills(w.env.ctx.db(), &slips_input(None)).unwrap();
    assert!(bills.buyers.is_empty());
    assert_eq!(bills.sellers.len(), 2);
    for slip in &bills.sellers {
        assert_eq!(
            slip.named_title_id.as_deref(),
            Some("print-seller-bill-named-title")
        );
        assert!(slip.party.is_some());
    }
    // The buyer slips of the same day carry the buyer title and the party.
    let parchi = mandi_reports::reports::day_slips(
        w.env.ctx.db(),
        &DaySlipsInput {
            sections: SlipSections::Buyers,
            ..slips_input(None)
        },
        "print-parchi-title",
    )
    .unwrap();
    assert_eq!(parchi.buyers.len(), 3);
    assert!(parchi.buyers.iter().all(|s| s.named_title_id.as_deref()
        == Some("print-buyer-day-slip-title")
        && s.party.is_some()));
}

// ---- A7, B1, B2, B3: the statement with a row for each entry ----

use mandi_reports::reports::{StatementRowsInput, statement_rows};
use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::{accounts, vouchers};

fn rows_input(account: i64) -> StatementRowsInput {
    StatementRowsInput {
        account_id: account,
        from: None,
        to: None,
        include_history: false,
        newest_first: false,
        offset: 0,
        limit: 0,
    }
}

fn bal(b: &mandi_services::core::accounts::Balance) -> (i64, BalanceSide) {
    (rupees_of(b.amount_paisa), b.side)
}

#[test]
fn a_sellers_statement_has_his_opening_first_and_one_row_for_each_entry() {
    // E2E-64 (D8): 200 آم from احسان, کرایہ 10,000, مزدوری 1,000: his lot's net is 160,080. He is
    // given an opening of 20,000 جمع dated the day of the sale, and 100,000 is paid to him next day.
    let (w, _) = reports_world::d8_paid_world();
    let ehsan = id_named(&w, "Ehsan");
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: ehsan,
                version: accounts::get(&w.env.ctx, ehsan).unwrap().version,
                opening_date: "10-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "20,000".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    let mut pay = reports_world::payment(ehsan, "100,000");
    pay.date = "11-10-2026".into();
    pay.confirm_cash_below_zero = true;
    vouchers::save_payment(&w.env.ctx, &w.owner, &pay).unwrap();

    let s = statement_rows(w.env.ctx.db(), &rows_input(ehsan)).unwrap();
    assert_eq!(s.count, 3);
    let kinds: Vec<&str> = s.rows.iter().map(|r| r.type_id.as_str()).collect();
    // The opening is the first row even though its entry was posted after the sale.
    assert_eq!(
        kinds,
        [
            "report-doc-opening",
            "report-doc-bikri-first",
            "report-doc-payment"
        ]
    );
    let figures: Vec<(i64, i64, (i64, BalanceSide))> = s
        .rows
        .iter()
        .map(|r| {
            (
                rupees_of(r.dr_paisa),
                rupees_of(r.cr_paisa),
                bal(&r.balance),
            )
        })
        .collect();
    assert_eq!(
        figures,
        [
            (0, 20_000, (20_000, BalanceSide::Cr)),
            (0, 160_080, (180_080, BalanceSide::Cr)),
            (100_000, 0, (80_080, BalanceSide::Cr)),
        ]
    );
    // Totals over the whole period, and the closing is what the party card shows.
    assert_eq!(rupees_of(s.dr_total_paisa), 100_000);
    assert_eq!(rupees_of(s.cr_total_paisa), 180_080);
    assert_eq!(bal(&s.closing), (80_080, BalanceSide::Cr));
    assert_eq!(bal(&s.brought_forward), (0, BalanceSide::Zero));
    // The sale row has its lot inside, as the lot's name, and the quantity in short.
    let sale = &s.rows[1];
    assert_eq!(sale.number, Some(1));
    assert_eq!(sale.parts.len(), 1);
    assert_eq!(sale.short.len(), 1);
    assert_eq!(sale.short[0].qty, 200);
    assert_eq!(sale.short[0].item.en, "Mango-1");
    assert!(sale.edited.is_none());
    // A period that starts after the opening: the balance brought forward is the opening.
    let later = statement_rows(
        w.env.ctx.db(),
        &StatementRowsInput {
            from: Some("11-10-2026".into()),
            ..rows_input(ehsan)
        },
    )
    .unwrap();
    assert_eq!(later.count, 1);
    assert_eq!(bal(&later.brought_forward), (180_080, BalanceSide::Cr));
    assert_eq!(bal(&later.closing), (80_080, BalanceSide::Cr));
    // The newest row first, with the same balances.
    let newest = statement_rows(
        w.env.ctx.db(),
        &StatementRowsInput {
            newest_first: true,
            ..rows_input(ehsan)
        },
    )
    .unwrap();
    assert_eq!(newest.rows[0].type_id, "report-doc-payment");
    assert_eq!(bal(&newest.rows[0].balance), (80_080, BalanceSide::Cr));
    // An account that is not a party has no statement.
    let cash = w
        .env
        .ctx
        .db()
        .read(|c| {
            c.query_row("SELECT id FROM accounts WHERE code = '01010001'", [], |r| {
                r.get::<_, i64>(0)
            })
        })
        .unwrap();
    assert!(statement_rows(w.env.ctx.db(), &rows_input(cash)).is_err());
}

#[test]
fn a_buyer_who_paid_on_the_spot_has_it_in_the_same_row_as_his_bill() {
    // E2E-65 (D8): مانی's bill 21,360 paid in full; انس's 48,100 paid 10,000; ہارون's 125,060 with
    // an opening of 10,000 جمع before it.
    let (w, _) = reports_world::d8_paid_world();
    let row_of =
        |name: &str| statement_rows(w.env.ctx.db(), &rows_input(id_named(&w, name))).unwrap();
    let mani = row_of("Mani");
    assert_eq!(mani.rows.len(), 1);
    let r = &mani.rows[0];
    assert_eq!(
        (
            rupees_of(r.dr_paisa),
            rupees_of(r.cr_paisa),
            rupees_of(r.on_spot_paisa)
        ),
        (21_360, 21_360, 21_360)
    );
    assert_eq!(bal(&r.balance), (0, BalanceSide::Zero));
    // The paid-on-the-spot credit is not a part of the row: it is the on_spot figure.
    assert!(
        r.parts
            .iter()
            .all(|p| !matches!(p, mandi_reports::reports::DetailPart::PaidOnSpot { .. }))
    );
    let anas = row_of("Anas");
    let r = &anas.rows[0];
    assert_eq!(
        (
            rupees_of(r.dr_paisa),
            rupees_of(r.cr_paisa),
            rupees_of(r.on_spot_paisa)
        ),
        (48_100, 10_000, 10_000)
    );
    assert_eq!(bal(&r.balance), (38_100, BalanceSide::Dr));
    // ہارون: the opening first (10,000 جمع), then the sale 125,060, closing 115,060 بنام.
    let haroon = row_of("Haroon");
    assert_eq!(haroon.rows.len(), 2);
    assert_eq!(haroon.rows[0].type_id, "report-doc-opening");
    assert_eq!(bal(&haroon.rows[0].balance), (10_000, BalanceSide::Cr));
    assert_eq!(bal(&haroon.closing), (115_060, BalanceSide::Dr));
}

#[test]
fn an_edited_entry_shows_once_and_its_reversal_is_hidden_with_the_same_closing() {
    // E2E-13: اکرم's rate 600 → 610, so his bill is 13,092. The old bill and its reversal are one
    // pair on the same day: hidden. The new bill shows once, with the edited mark.
    let w = reports_world::worked_day();
    let saved = reports_world::save(&w, &support::event1(&w));
    let opened = mandi_services::bikri::open_for_edit(
        &w.env.ctx,
        &mandi_services::core::bikri::EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    mandi_services::bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("ریٹ کی درستی".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let akram = w.akram;
    let normal = statement_rows(w.env.ctx.db(), &rows_input(akram)).unwrap();
    assert_eq!(normal.count, 1);
    let row = &normal.rows[0];
    assert_eq!(
        (rupees_of(row.dr_paisa), rupees_of(row.cr_paisa)),
        (13_092, 0)
    );
    assert_eq!(bal(&row.balance), (13_092, BalanceSide::Dr));
    assert!(row.edited.is_some() && !row.is_replaced && !row.is_reversal);
    assert_eq!(bal(&normal.closing), (13_092, BalanceSide::Dr));
    // With the history: the old bill, its reversal and the new bill, the same closing, and the
    // totals are those of the party ledger R13 (12,880 + 13,092 in debit, 12,880 in credit).
    let full = statement_rows(
        w.env.ctx.db(),
        &StatementRowsInput {
            include_history: true,
            ..rows_input(akram)
        },
    )
    .unwrap();
    assert_eq!(full.count, 3);
    assert_eq!(bal(&full.closing), (13_092, BalanceSide::Dr));
    assert_eq!(rupees_of(full.dr_total_paisa), 12_880 + 13_092);
    assert_eq!(rupees_of(full.cr_total_paisa), 12_880);
    assert_eq!(full.rows.iter().filter(|r| r.is_replaced).count(), 2);
    let r13 = reports_world::report(
        &w,
        mandi_reports::reports::ReportId::R13,
        mandi_reports::reports::ReportFilters {
            party: Some(mandi_reports::reports::PartyFilter {
                account_id: akram,
                person_key: None,
            }),
            ..Default::default()
        },
        0,
        0,
    );
    let r13_closing = r13
        .summary
        .iter()
        .find(|s| s.label_id == "report-closing-balance")
        .map(|s| balance_of(&s.cell));
    assert_eq!(r13_closing, Some((13_092, BalanceSide::Dr)));
}

#[test]
fn the_statement_print_is_a_compact_table_with_the_opening_first_and_the_closing_in_words() {
    // The احسان example of plan 78 A7 on the E2E-64 sale: the opening, the sale and a payment.
    let (w, _) = reports_world::d8_paid_world();
    let ehsan = id_named(&w, "Ehsan");
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: ehsan,
                version: accounts::get(&w.env.ctx, ehsan).unwrap().version,
                opening_date: "10-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "20,000".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    let mut pay = reports_world::payment(ehsan, "100,000");
    pay.date = "11-10-2026".into();
    pay.confirm_cash_below_zero = true;
    vouchers::save_payment(&w.env.ctx, &w.owner, &pay).unwrap();

    for language in [Language::Ur, Language::En] {
        for items in [false, true] {
            let print = mandi_reports::reports::statement_rows_print_data(
                w.env.ctx.db(),
                &mandi_reports::reports::StatementRowsPrintInput {
                    account_id: ehsan,
                    from: None,
                    to: None,
                    items,
                    include_history: false,
                    language,
                    paper: None,
                },
            )
            .unwrap();
            // Three rows, not a page for the sale: opening, بکری 1, ادائیگی 1.
            assert_eq!(print.rows.len(), 3);
            assert_eq!(print.columns.len(), 6);
            assert_eq!(print.frame.header.title_id, "print-statement-rows-title");
            // The totals row: 100,000 debit, 180,080 credit; the closing 80,080 in words.
            let totals = print
                .totals
                .iter()
                .map(text_of)
                .collect::<Vec<_>>()
                .join(" ");
            for want in ["100,000", "180,080", "80,080"] {
                assert!(totals.contains(want), "{want} in {totals}");
            }
            assert!(print.words.is_some());
        }
    }
    // From the day after the opening: the balance brought forward is the first row.
    let later = mandi_reports::reports::statement_rows_print_data(
        w.env.ctx.db(),
        &mandi_reports::reports::StatementRowsPrintInput {
            account_id: ehsan,
            from: Some("11-10-2026".into()),
            to: None,
            items: false,
            include_history: false,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(later.rows.len(), 2);
    let first = later.rows[0]
        .cells
        .iter()
        .map(text_of)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        first.contains("print-statement-rows-brought-forward"),
        "{first}"
    );
    assert!(first.contains("180,080"), "{first}");
}

// ---- Extra cases from the money review ----

#[test]
fn the_adhari_sheet_counts_an_opening_dated_the_day_and_the_slip_of_one_bikri_keeps_it() {
    // E2E-65 (D8): ہارون holds 10,000 جمع (an opening dated 10-10-26) and buys 125,060 that day with
    // nothing paid: previous -10,000, today's ادھار 125,060, کل ادھار 115,060 بنام.
    let (w, bikri_id) = reports_world::d8_paid_world();
    let sheet = adhari(&w, "10-10-2026");
    let haroon = sheet
        .rows
        .iter()
        .find(|r| r.party.name_en == "Haroon")
        .unwrap();
    assert_eq!(
        (
            rupees_of(haroon.bill_paisa),
            rupees_of(haroon.today_credit_paisa)
        ),
        (125_060, 125_060)
    );
    assert_eq!(bal(&haroon.previous_due), (10_000, BalanceSide::Cr));
    assert_eq!(bal(&haroon.total_due), (115_060, BalanceSide::Dr));
    // The buyers of the sheet: مانی 0 left, انس 38,100, ہارون 115,060 (E2E-65): 153,160 in all.
    assert_eq!(rupees_of(sheet.all_totals.total_due.amount_paisa), 153_160);
    // The slip of this بکری only keeps the opening line and the day's balances.
    let slips = mandi_reports::reports::day_slips(
        w.env.ctx.db(),
        &DaySlipsInput {
            sections: SlipSections::Buyers,
            ..slips_input(Some(bikri_id))
        },
        "print-parchi-title",
    )
    .unwrap();
    let haroon_slip = slips
        .buyers
        .iter()
        .find(|s| s.party.as_ref().is_some_and(|p| p.name == "Haroon"))
        .unwrap();
    assert!(
        haroon_slip
            .summary
            .iter()
            .any(|l| l.label_id == "report-r35-opening")
    );
}
