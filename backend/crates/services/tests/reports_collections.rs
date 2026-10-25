//! The Collections group of the report engine on the PRD worked day (plan 20 step B3, PRD 19.2,
//! FR-7.59, FR-15.4, FR-15.8, FR-19.9, FR-19.10): R20, R21, R22, R24 and R26, and the search
//! for one-time people (PRD FR-5.4).
//!
//! The expected numbers are the PRD's: R20 closes at 21,564 after Events 1 to 5 and at 24,564
//! after E2E-32, R24 shows کریم with 10,160 and, after E2E-28, رحیم with 19,316 payable and
//! 15,000 advance, R26 shows E2E-7 and E2E-31. Every total is also checked against the services
//! (the voucher screen's workings and the ledger balances), so a report can never disagree with
//! the books (money agent Section 9).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::people::{FindPeopleInput, find_people};
use mandi_reports::reports::{
    Cell, PartyFilter, PrintCell, ReportFilters, ReportId, ReportPage, ReportPrintInput, print_data,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::OpenPayablesInput;
use mandi_services::{ledger, vouchers};

use reports_world::*;

fn names_en(cell: &Cell) -> String {
    match cell {
        Cell::Names { names } => names.en.clone(),
        Cell::Empty => String::new(),
        other => panic!("not names: {other:?}"),
    }
}

fn label(cell: &Cell) -> String {
    match cell {
        Cell::Label { id } => id.clone(),
        other => panic!("not a label: {other:?}"),
    }
}

fn date_of(cell: &Cell) -> Option<String> {
    match cell {
        Cell::Date { iso } => Some(iso.clone()),
        Cell::Empty => None,
        other => panic!("not a date: {other:?}"),
    }
}

fn person_name(cell: &Cell) -> String {
    match cell {
        Cell::Party { party } => party.name_en.clone(),
        other => panic!("not a party: {other:?}"),
    }
}

fn as_of(text: &str) -> ReportFilters {
    ReportFilters {
        as_of: text.into(),
        ..ReportFilters::default()
    }
}

/// The balances of every buyer account (control 0304) that stands on the debit side, from the
/// services ledger: the shop's total اگراہی (PRD FR-19.9).
fn buyers_debit_total(w: &support::World) -> i64 {
    let ids: Vec<i64> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare(
                "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
                 WHERE k.code = '0304' AND a.is_pukhta_khata = 0",
            )
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let mut total = 0;
    for id in ids {
        let b = ledger::account_balance(&w.env.ctx, id).unwrap();
        if b.side == BalanceSide::Dr {
            total += b.amount_paisa.rupees();
        }
    }
    // The one-time buyers by person.
    let keys: Vec<String> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare(
                "SELECT DISTINCT pukhta_person_key FROM journal_lines
                 WHERE account_id = ?1 AND pukhta_person_key IS NOT NULL",
            )
            .unwrap();
        statement
            .query_map([w.pukhta_buyer], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    for key in keys {
        let b = ledger::person_balance(&w.env.ctx, w.pukhta_buyer, &key).unwrap();
        if b.side == BalanceSide::Dr {
            total += b.amount_paisa.rupees();
        }
    }
    total
}

// --- R20 -----------------------------------------------------------------------------------------

#[test]
fn r20_lists_the_buyers_who_owe_and_its_total_is_the_shops_total_agrahi() {
    let w = full_day();
    let page = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(
        page.columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        vec![
            "buyer",
            "city",
            "phone",
            "balance",
            "oldest_date",
            "days",
            "age_0_7",
            "age_8_30",
            "age_31_90",
            "age_91"
        ]
    );
    // PRD 4.9: اکرم 2,880 and ارشد 18,684. شاہد paid on the spot and رحیم, کریم are sellers.
    assert_eq!(page.total_count, 2);
    let akram = &page.rows.first().unwrap().cells;
    assert_eq!(code_of(&akram[0]), "03040001");
    assert_eq!(money(&akram[3]), 2_880);
    // His one open bill is بکری 1 of 10-10-26, 0 days old, in the first band (E2E-6, E2E-24).
    assert_eq!(date_of(&akram[4]).as_deref(), Some("2026-10-10"));
    assert_eq!(akram[5], Cell::Number { value: 0 });
    assert_eq!(
        akram[6..].iter().map(money).collect::<Vec<_>>(),
        vec![2_880, 0, 0, 0]
    );
    let arshad = &page.rows.get(1).unwrap().cells;
    assert_eq!(code_of(&arshad[0]), "03040002");
    assert_eq!(money(&arshad[3]), 18_684);
    // The total row is 21,564 (PRD 4.9, FR-19.9) and equals the books.
    assert_eq!(page.totals.get(3).map(money), Some(21_564));
    assert_eq!(buyers_debit_total(&w), 21_564);
    // The bands add up to the balance (the matching rows are canonical).
    assert_eq!(page.totals[6..].iter().map(money).sum::<i64>(), 21_564);
    assert_eq!(page.title_id, "report-r20-title");
}

#[test]
fn r20_after_cash_lent_to_a_buyer_is_24564_e2e_32() {
    let w = full_day();
    let mut lent = payment(w.akram, "3,000");
    lent.reason = "نقد ادھار".into();
    vouchers::save_payment(&w.env.ctx, &w.owner, &lent).unwrap();
    let page = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    // اکرم 2,880 + 3,000 = 5,880: two open bills, both of 10-10-26.
    assert_eq!(money(&page.rows.first().unwrap().cells[3]), 5_880);
    assert_eq!(page.totals.get(3).map(money), Some(24_564));
    assert_eq!(buyers_debit_total(&w), 24_564);
    assert_eq!(money(&page.rows.first().unwrap().cells[6]), 5_880);
    assert_eq!(page.totals[6..].iter().map(money).sum::<i64>(), 24_564);
}

#[test]
fn r20_ages_the_open_bills_in_four_bands_and_leaves_the_ages_empty_for_a_past_day() {
    let w = full_day();
    // 41 days later: both bills are in the band of 31 to 90 days.
    set_today(&w.env, 2026, 11, 20);
    let page = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    let akram = &page.rows.first().unwrap().cells;
    assert_eq!(akram[5], Cell::Number { value: 41 });
    assert_eq!(
        akram[6..].iter().map(money).collect::<Vec<_>>(),
        vec![0, 0, 2_880, 0]
    );
    assert_eq!(
        page.totals[6..].iter().map(money).collect::<Vec<_>>(),
        vec![0, 0, 21_564, 0]
    );
    // 100 days after: the fourth band.
    set_today(&w.env, 2027, 1, 18);
    let old = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(
        old.totals[6..].iter().map(money).collect::<Vec<_>>(),
        vec![0, 0, 0, 21_564]
    );
    // A report as of a day before today gives the balance of that day and no ages: the matching
    // rows are not dated, so nothing is guessed.
    let past = report(&w, ReportId::R20, as_of("10-10-2026"), 0, 0);
    assert_eq!(past.total_count, 2);
    assert_eq!(past.totals.get(3).map(money), Some(21_564));
    let row = &past.rows.first().unwrap().cells;
    assert_eq!((row[4].clone(), row[5].clone()), (Cell::Empty, Cell::Empty));
    assert!(row[6..].iter().all(|c| *c == Cell::Empty));
    // Before the sales there is nothing owed.
    let before = report(&w, ReportId::R20, as_of("09-10-2026"), 0, 0);
    assert_eq!(
        (before.total_count, before.totals.get(3).map(money)),
        (0, Some(0))
    );
}

#[test]
fn r20_keeps_one_time_buyers_apart_by_person_e2e_21() {
    use mandi_services::core::bikri::{LineChargeMethod, PaymentType, PukhtaInput};
    let w = worked_day();
    let line = |key: &str, qty: &str, ur: &str, en: &str, phone: &str| {
        let mut l = support::sale(key, w.pukhta_buyer, qty, "1000");
        l.pukhta = Some(PukhtaInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.into(),
        });
        l.payment_type = Some(PaymentType::Credit);
        l.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        l.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        l.laga = support::typed(LineChargeMethod::PerUnit, "0");
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "3");
    lot.sales = vec![
        line("s-1", "1", "شاہد", "Shahid", "0300-0000000"),
        line("s-2", "2", "ناصر", "Nasir", "0300-1111111"),
    ];
    save(&w, &support::draft(vec![lot]));
    let before = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(before.total_count, 2);
    let by_name = |page: &ReportPage, name: &str| -> Option<Vec<Cell>> {
        page.rows
            .iter()
            .find(|r| person_name(&r.cells[0]) == name)
            .map(|r| r.cells.clone())
    };
    // Each person has his own row, his own phone and his own balance.
    let shahid = by_name(&before, "Shahid").unwrap();
    assert_eq!(money(&shahid[3]), 1_000);
    assert_eq!(
        shahid[2],
        Cell::Text {
            text: "03000000000".into()
        }
    );
    let nasir = by_name(&before, "Nasir").unwrap();
    assert_eq!(money(&nasir[3]), 2_000);
    match &nasir[0] {
        Cell::Party { party } => {
            assert_eq!(party.code, "03049999");
            assert!(party.person_key.is_some());
            assert_eq!(party.short_code, None);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(before.totals.get(3).map(money), Some(3_000));

    // The receipt of 1,000 for شاہد: شاہد 0 (no row), ناصر 2,000 still due.
    let key = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    let mut input = receipt(w.pukhta_buyer, "1,000");
    input.person_key = Some(key);
    vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    let after = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(after.total_count, 1);
    assert_eq!(person_name(&after.rows.first().unwrap().cells[0]), "Nasir");
    assert_eq!(after.totals.get(3).map(money), Some(2_000));
    assert_eq!(buyers_debit_total(&w), 2_000);
    // The party filter takes one person.
    let nasir_key = mandi_services::core::text::person_key("ناصر", "0300-1111111");
    let one = report(
        &w,
        ReportId::R20,
        ReportFilters {
            party: Some(PartyFilter {
                account_id: w.pukhta_buyer,
                person_key: Some(nasir_key),
            }),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(one.total_count, 1);
}

// --- R21 and R22 ---------------------------------------------------------------------------------

/// Gives اکرم a city and a collector (a plain update of the test data) and returns their ids.
fn give_akram_a_city_and_a_collector(w: &support::World) -> (i64, i64) {
    let conn = raw(&w.dir);
    conn.execute(
        "INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('1', 'کوٹ رادھا کشن', 'Kot Radha Kishan', 'kot', 'kot radha kishan', 'kt')",
        [],
    )
    .unwrap();
    let city = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO staff (name_ur, name_en, search_ur, search_en, search_phonetic, role)
         VALUES ('اسلم', 'Aslam', 'aslam', 'aslam', 'aslm', 'collector')",
        [],
    )
    .unwrap();
    let staff = conn.last_insert_rowid();
    conn.execute(
        "UPDATE accounts SET city_id = ?1, collector_staff_id = ?2 WHERE id = ?3",
        [city, staff, w.akram],
    )
    .unwrap();
    (city, staff)
}

#[test]
fn r21_groups_the_buyers_by_city_with_a_total_for_each_city() {
    let w = full_day();
    let (city, _) = give_akram_a_city_and_a_collector(&w);
    let page = report(&w, ReportId::R21, ReportFilters::default(), 0, 0);
    assert_eq!(
        page.columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>()[..4],
        ["group", "buyer", "phone", "balance"]
    );
    // City heading, اکرم, the heading of buyers with no city, ارشد.
    assert_eq!(page.total_count, 4);
    let kinds: Vec<String> = page.rows.iter().map(|r| format!("{:?}", r.kind)).collect();
    assert_eq!(kinds, vec!["Group", "Normal", "Group", "Normal"]);
    let first = &page.rows[0].cells;
    assert_eq!(names_en(&first[0]), "Kot Radha Kishan");
    assert_eq!(money(&first[3]), 2_880);
    assert_eq!(code_of(&page.rows[1].cells[1]), "03040001");
    let no_city = &page.rows[2].cells;
    assert_eq!(no_city[0], Cell::Empty);
    assert_eq!(money(&no_city[3]), 18_684);
    // The total row is the shop's total, not the sum of the headings twice, and it is the one of
    // R20 and R22 (the same buyers, only arranged differently).
    assert_eq!(page.totals.get(3).map(money), Some(21_564));
    let flat = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(flat.totals.get(3), page.totals.get(3));
    assert_eq!(flat.totals[6..], page.totals[6..]);
    let by_collector = report(&w, ReportId::R22, ReportFilters::default(), 0, 0);
    assert_eq!(by_collector.totals.get(4), page.totals.get(3));
    assert_eq!(by_collector.totals[7..], page.totals[6..]);
    // A page cuts through the headings too, and every page has the same totals.
    let second = report(&w, ReportId::R21, ReportFilters::default(), 2, 2);
    assert_eq!(second.rows.len(), 2);
    assert_eq!(second.totals, page.totals);
    // The city filter keeps one city.
    let only = report(
        &w,
        ReportId::R21,
        ReportFilters {
            city_id: Some(city),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(only.total_count, 2);
    assert_eq!(only.totals.get(3).map(money), Some(2_880));
}

#[test]
fn r22_groups_the_buyers_by_collector() {
    let w = full_day();
    give_akram_a_city_and_a_collector(&w);
    let page = report(&w, ReportId::R22, ReportFilters::default(), 0, 0);
    assert_eq!(
        page.columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>()[..5],
        ["group", "buyer", "city", "phone", "balance"]
    );
    assert_eq!(page.total_count, 4);
    let first = &page.rows[0].cells;
    assert_eq!(names_en(&first[0]), "Aslam");
    assert_eq!(money(&first[4]), 2_880);
    // The buyer's own city shows on his row.
    assert_eq!(names_en(&page.rows[1].cells[2]), "Kot Radha Kishan");
    assert_eq!(page.rows[2].cells[0], Cell::Empty);
    assert_eq!(money(&page.rows[2].cells[4]), 18_684);
    assert_eq!(page.totals.get(4).map(money), Some(21_564));
}

// --- R24 -----------------------------------------------------------------------------------------

#[test]
fn r24_lists_karim_with_10160_and_not_rahim_who_is_paid_up() {
    let w = full_day();
    let page = report(&w, ReportId::R24, ReportFilters::default(), 0, 0);
    assert_eq!(
        page.columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        vec![
            "section",
            "seller",
            "payable",
            "advance_open",
            "owes_firm",
            "balance",
            "last_date"
        ]
    );
    // A heading for the sellers the firm owes, then کریم (E2E-27); رحیم is paid up.
    assert_eq!(page.total_count, 2);
    assert_payable_identity(&page);
    assert_eq!(label(&page.rows[0].cells[0]), "report-section-payable");
    assert_eq!(money(&page.rows[0].cells[2]), 10_160);
    let karim = &page.rows[1].cells;
    assert_eq!(code_of(&karim[1]), "030110002");
    assert_eq!(money(&karim[2]), 10_160);
    assert_eq!(karim[3], Cell::Empty);
    assert_eq!(karim[4], Cell::Empty);
    assert_eq!(balance(&karim[5]), (BalanceSide::Cr, 10_160));
    assert_eq!(date_of(&karim[6]).as_deref(), Some("2026-10-10"));
    // The total row: payable 10,160 (the tile ادائیگی باقی), advance 0, balance 10,160 جمع.
    assert_eq!(page.totals.get(2).map(money), Some(10_160));
    assert_eq!(page.totals.get(3).map(money), Some(0));
    assert_eq!(page.totals.get(4).map(money), Some(0));
    assert_eq!(
        balance(page.totals.get(5).unwrap()),
        (BalanceSide::Cr, 10_160)
    );
    // The voucher screen says the same.
    let open = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.karim,
            person_key: None,
            include_settled: false,
        },
    )
    .unwrap();
    assert_eq!(open.open_paisa.rupees(), 10_160);
    assert_eq!(page.title_id, "report-r24-title");
}

/// The report row of a seller: payable, advance open, owes the firm, balance, section.
type SellerLine = (i64, i64, i64, (BalanceSide, i64), String);

/// The identity of the payable working (money agent Section 8.2a): for every seller row, his
/// ledger balance (جمع positive) is the payable less the open advance less what he owes beyond
/// it. Checked on every row and on the total row.
fn assert_payable_identity(page: &ReportPage) {
    let signed = |cell: &Cell| -> i64 {
        let (side, amount) = balance(cell);
        match side {
            BalanceSide::Cr => amount,
            BalanceSide::Dr => -amount,
            BalanceSide::Zero => 0,
        }
    };
    let rows = page
        .rows
        .iter()
        .filter(|r| matches!(r.kind, mandi_reports::reports::RowKind::Normal))
        .map(|r| &r.cells)
        .chain(std::iter::once(&page.totals));
    for cells in rows {
        assert_eq!(
            signed(&cells[5]),
            money(&cells[2]) - money(&cells[3]) - money(&cells[4]),
            "balance = payable - open advance - owed beyond it"
        );
    }
}

/// The report row of a seller: payable, advance open, owes the firm, balance.
fn seller_row(page: &ReportPage, code: &str) -> Option<SellerLine> {
    page.rows.iter().find_map(|r| match r.cells.get(1) {
        Some(Cell::Party { party }) if party.code == code => Some((
            money(&r.cells[2]),
            money(&r.cells[3]),
            money(&r.cells[4]),
            balance(&r.cells[5]),
            label_of_section(page, r),
        )),
        _ => None,
    })
}

/// The section heading above a row.
fn label_of_section(page: &ReportPage, row: &mandi_reports::reports::Row) -> String {
    let at = page
        .rows
        .iter()
        .position(|r| std::ptr::eq(r, row))
        .unwrap_or(0);
    page.rows[..=at]
        .iter()
        .rev()
        .find_map(|r| match (&r.kind, r.cells.first()) {
            (mandi_reports::reports::RowKind::Group, Some(Cell::Label { id })) => Some(id.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn r24_after_e2e_28_shows_the_payable_and_the_advance_that_stays_open() {
    // E2E-28: the tick "پیشگی سے کاٹیں" is off. Payable 24,316 − 5,000 = 19,316; advance open
    // 15,000; ledger 4,316 جمع.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save(&w, &draft);
    let page = report(&w, ReportId::R24, ReportFilters::default(), 0, 0);
    assert_eq!(
        seller_row(&page, "030110001"),
        Some((
            19_316,
            15_000,
            0,
            (BalanceSide::Cr, 4_316),
            "report-section-payable".to_owned()
        ))
    );
    assert_eq!(page.totals.get(2).map(money), Some(19_316));
    assert_eq!(page.totals.get(3).map(money), Some(15_000));
    assert_payable_identity(&page);
    // The identity of the working: balance = payable − open advance − owed beyond advances.
    // After a payment of 19,316: payable 0, the advance still open, his ledger 15,000 بنام, and
    // he is not in the list of sellers who owe the firm.
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "19,316")).unwrap();
    let after = report(&w, ReportId::R24, ReportFilters::default(), 0, 0);
    assert_eq!(
        seller_row(&after, "030110001"),
        Some((
            0,
            15_000,
            0,
            (BalanceSide::Dr, 15_000),
            "report-section-payable".to_owned()
        ))
    );
    // A payment beyond what is payable makes him owe the firm beyond his advance (PRD FR-15.7):
    // the row moves to the list "بیوپاری کے ذمے".
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "700")).unwrap();
    let owing = report(&w, ReportId::R24, ReportFilters::default(), 0, 0);
    assert_eq!(
        seller_row(&owing, "030110001"),
        Some((
            0,
            15_000,
            700,
            (BalanceSide::Dr, 15_700),
            "report-section-owes-firm".to_owned()
        ))
    );
    assert_eq!(owing.totals.get(4).map(money), Some(700));
    assert_payable_identity(&owing);
    assert_payable_identity(&after);
    // The voucher screen says the same.
    let open = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.rahim,
            person_key: None,
            include_settled: false,
        },
    )
    .unwrap();
    assert_eq!(
        (
            open.open_paisa.rupees(),
            open.advances_open_paisa.rupees(),
            open.owes_firm_paisa.rupees()
        ),
        (0, 15_000, 700)
    );
}

#[test]
fn r24_for_a_past_day_gives_the_ledger_balance_only() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 12);
    let past = report(&w, ReportId::R24, as_of("10-10-2026"), 0, 0);
    // No sections: the workings are only known for today.
    assert_eq!(past.total_count, 1);
    let karim = &past.rows[0].cells;
    assert_eq!(karim[0], Cell::Empty);
    assert_eq!(
        (karim[2].clone(), karim[3].clone(), karim[4].clone()),
        (Cell::Empty, Cell::Empty, Cell::Empty)
    );
    assert_eq!(balance(&karim[5]), (BalanceSide::Cr, 10_160));
    assert_eq!(past.totals.get(2), Some(&Cell::Empty));
}

// --- R26 -----------------------------------------------------------------------------------------

#[test]
fn r26_shows_rahims_advance_fully_adjusted_e2e_7() {
    let w = full_day();
    let page = report(&w, ReportId::R26, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 1);
    let row = &page.rows[0].cells;
    assert_eq!(code_of(&row[0]), "030110001");
    // Given 15,000, returned 0, adjusted 15,000, open 0, last advance 01-09-26.
    assert_eq!(
        row[1..5].iter().map(money).collect::<Vec<_>>(),
        vec![15_000, 0, 15_000, 0]
    );
    assert_eq!(date_of(&row[5]).as_deref(), Some("2026-09-01"));
    assert_eq!(
        page.totals[1..5].iter().map(money).collect::<Vec<_>>(),
        vec![15_000, 0, 15_000, 0]
    );
    // The voucher screen's report says the same.
    let screen = vouchers::advances_report(
        &w.env.ctx,
        &mandi_services::core::vouchers::AdvancesReportInput {
            as_of: String::new(),
            seller_id: None,
        },
    )
    .unwrap();
    assert_eq!(
        (
            screen.given_paisa.rupees(),
            screen.returned_paisa.rupees(),
            screen.adjusted_paisa.rupees(),
            screen.open_paisa.rupees()
        ),
        (15_000, 0, 15_000, 0)
    );
    // As of a day before the sale: given and still open. Before the advance: nobody.
    let early = report(&w, ReportId::R26, as_of("05-09-2026"), 0, 0);
    assert_eq!(
        early.rows[0].cells[1..5]
            .iter()
            .map(money)
            .collect::<Vec<_>>(),
        vec![15_000, 0, 0, 15_000]
    );
    let none = report(&w, ReportId::R26, as_of("31-08-2026"), 0, 0);
    assert_eq!(none.total_count, 0);
    // One seller.
    let karim = report(
        &w,
        ReportId::R26,
        ReportFilters {
            seller_id: Some(w.karim),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(karim.total_count, 0);
}

#[test]
fn r26_after_a_cash_return_shows_given_returned_adjusted_open_e2e_31() {
    // E2E-31: رحیم returns 10,000 of his advance, then بکری 1 adjusts the last 5,000.
    let w = worked_day();
    let back = receipt(w.rahim, "10,000");
    vouchers::save_receipt(&w.env.ctx, &w.owner, &back).unwrap();
    let mid = report(&w, ReportId::R26, ReportFilters::default(), 0, 0);
    assert_eq!(
        mid.rows[0].cells[1..5]
            .iter()
            .map(money)
            .collect::<Vec<_>>(),
        vec![15_000, 10_000, 0, 5_000]
    );
    save(&w, &support::event1(&w));
    let page = report(&w, ReportId::R26, ReportFilters::default(), 0, 0);
    assert_eq!(
        page.rows[0].cells[1..5]
            .iter()
            .map(money)
            .collect::<Vec<_>>(),
        vec![15_000, 10_000, 5_000, 0]
    );
}

// --- Search for one-time people ----------------------------------------------------------------

#[test]
fn find_people_finds_a_one_time_person_by_either_name_a_close_spelling_or_phone() {
    let w = full_day();
    let find = |text: &str| {
        find_people(
            w.env.ctx.db(),
            &FindPeopleInput {
                text: text.into(),
                limit: 0,
            },
        )
        .unwrap()
    };
    for text in ["shahid", "SHAHID", "شاہد", "shahd", "0300", "03-00-000"] {
        let hits = find(text);
        assert_eq!(hits.len(), 1, "{text}");
        let hit = &hits[0];
        assert_eq!(
            (
                hit.name_en.as_str(),
                hit.phone.as_str(),
                hit.last_date.as_str()
            ),
            ("Shahid", "03000000000", "2026-10-10")
        );
        // He is a one-time buyer: the account to open his ledger with.
        assert_eq!(hit.accounts.len(), 1);
        assert_eq!(hit.accounts[0].account_id, w.pukhta_buyer);
        assert!(hit.person_key.contains("03000000000"));
    }
    assert!(find("nasir").is_empty());
    // Empty text lists the newest people; the limit cuts the list.
    assert_eq!(find("").len(), 1);
    // The person key and the account open his ledger R13 as a party.
    let hit = find("shahid").remove(0);
    let page = report(
        &w,
        ReportId::R13,
        ReportFilters {
            party: Some(PartyFilter {
                account_id: hit.accounts[0].account_id,
                person_key: Some(hit.person_key.clone()),
            }),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(page.total_count, 2);
}

#[test]
fn find_people_checks_its_input_and_writes_nothing() {
    let w = full_day();
    let refuse = |text: &str, limit: u32| -> Vec<(ErrorCode, String)> {
        match find_people(
            w.env.ctx.db(),
            &FindPeopleInput {
                text: text.into(),
                limit,
            },
        ) {
            Err(mandi_reports::ReportsError::Rejected(items)) => {
                items.into_iter().map(|e| (e.code, e.field)).collect()
            }
            other => panic!("expected a rejection, got {other:?}"),
        }
    };
    assert_eq!(
        refuse("a\u{0}b", 51),
        vec![
            (ErrorCode::ErrOutOfRange, "text".to_owned()),
            (ErrorCode::ErrOutOfRange, "limit".to_owned())
        ]
    );
    assert_eq!(
        refuse(&"x".repeat(101), 10),
        vec![(ErrorCode::ErrOutOfRange, "text".to_owned())]
    );
    let count = |sql: &str| -> i64 { raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap() };
    let before = count("SELECT COUNT(*) FROM audit_log");
    find_people(
        w.env.ctx.db(),
        &FindPeopleInput {
            text: "shahid".into(),
            limit: 5,
        },
    )
    .unwrap();
    assert_eq!(count("SELECT COUNT(*) FROM audit_log"), before);
}

// --- Writes, print data ----------------------------------------------------------------------

#[test]
fn a_collections_report_writes_nothing() {
    let w = full_day();
    let count = |sql: &str| -> i64 { raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap() };
    let rows = || {
        (
            count("SELECT COUNT(*) FROM journal_entries"),
            count("SELECT COUNT(*) FROM journal_lines"),
            count("SELECT COUNT(*) FROM audit_log"),
            count("SELECT COUNT(*) FROM bill_allocations"),
            count("SELECT COUNT(*) FROM advance_allocations"),
        )
    };
    let start = rows();
    for id in [
        ReportId::R20,
        ReportId::R21,
        ReportId::R22,
        ReportId::R24,
        ReportId::R26,
    ] {
        report(&w, id, ReportFilters::default(), 0, 0);
    }
    assert_eq!(rows(), start);
}

#[test]
fn every_collections_report_prints_in_both_languages_with_the_totals_of_the_screen() {
    let w = full_day();
    give_akram_a_city_and_a_collector(&w);
    for id in [
        ReportId::R20,
        ReportId::R21,
        ReportId::R22,
        ReportId::R24,
        ReportId::R26,
    ] {
        let page = report(&w, id, ReportFilters::default(), 0, 0);
        for language in [Language::Ur, Language::En] {
            let data = print_data(
                w.env.ctx.db(),
                &ReportPrintInput {
                    id,
                    filters: ReportFilters::default(),
                    language,
                    paper: None,
                },
            )
            .unwrap();
            assert_eq!(data.rows.len(), page.rows.len(), "{id:?}");
            assert_eq!(data.totals.len(), page.columns.len(), "{id:?}");
            assert!(
                data.rows
                    .iter()
                    .all(|r| r.cells.len() == page.columns.len())
            );
        }
    }
    // The English R20 prints the dues with the buyer's short code and name, and 21,564 in the
    // total row.
    let en = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R20,
            filters: ReportFilters::default(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    match en.rows[0].cells.first().unwrap() {
        PrintCell::Party { party } => assert_eq!(
            (party.short_code.as_deref(), party.name.as_str()),
            (Some("1"), "Akram Toll Tax")
        ),
        other => panic!("{other:?}"),
    }
    match en.totals.get(3).unwrap() {
        PrintCell::Money { text } => assert_eq!(text, "21,564"),
        other => panic!("{other:?}"),
    }
    // The as-of day is in the header as DD-MM-YYYY.
    let dated = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R20,
            filters: as_of("10-10-2026"),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(
        dated
            .filters
            .iter()
            .map(|f| (f.label_id.as_str(), f.value.as_str()))
            .collect::<Vec<_>>(),
        vec![("report-filter-as-of", "10-10-2026")]
    );
    // R24 prints its section headings as labels.
    let r24 = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R24,
            filters: ReportFilters::default(),
            language: Language::Ur,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(
        r24.rows[0].cells[0],
        PrintCell::Label {
            id: "report-section-payable".into()
        }
    );
}

#[test]
fn the_collections_filters_not_taken_are_refused() {
    let w = full_day();
    // R26 takes the seller, not a party; R20 takes no dates.
    assert_eq!(
        rejected(&w, ReportId::R26, party(w.rahim), 0, 0),
        vec![(ErrorCode::ErrOutOfRange, "filters.party".to_owned())]
    );
    assert_eq!(
        rejected(
            &w,
            ReportId::R20,
            ReportFilters {
                from: "01-10-2026".into(),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(ErrorCode::ErrOutOfRange, "filters.from".to_owned())]
    );
    assert_eq!(
        rejected(&w, ReportId::R20, as_of("31-02-2026"), 0, 0),
        vec![(ErrorCode::ErrDateInvalid, "filters.as_of".to_owned())]
    );
}
