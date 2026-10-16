//! Money review of plan 14 (money-calculations agent): printed totals of REAL saved بکری shapes
//! that the fixture tests do not reach. Freight paid by the seller (PRD E6), a buyer rate that
//! differs (ریٹ فرق, FR-12.3), one buyer on two lots (E7), two sellers on one truck (E1), and a
//! seller who owes the firm (FR-12.7, E28). Every printed total must equal `bikri_get` and the
//! posted journal, and every page must add up inside itself.
#![cfg(test)]

#[path = "common/mod.rs"]
mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::print::{BikriPrintDataInput, PrintData, PrintPage, bikri_print_data};
use mandi_services::bikri;
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{BikriDraft, BikriSaveInput, FreightPaidBy};
use mandi_services::core::printing::PrintDoc;

use common::raw;
use support::*;

fn save(w: &World, draft: &BikriDraft) -> i64 {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
    .bikri_id
}

fn data(w: &World, id: i64, doc: PrintDoc, language: Language) -> PrintData {
    bikri_print_data(
        w.env.ctx.db(),
        &BikriPrintDataInput {
            bikri_id: id,
            doc,
            language,
            only_buyer_id: None,
        },
    )
    .unwrap()
}

fn n(text: &str) -> i64 {
    text.replace(',', "").parse().unwrap()
}

fn amount(page: &PrintPage, label: &str) -> Option<i64> {
    page.amounts
        .iter()
        .find(|a| a.label_id == label)
        .map(|a| n(&a.amount))
}

fn get(page: &PrintPage, label: &str) -> i64 {
    amount(page, label).unwrap_or_else(|| panic!("no {label}"))
}

/// (debit, credit) on one account in the posted entries of this بکری, in rupees.
fn journal(w: &World, bikri_id: i64, code: &str) -> (i64, i64) {
    raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_lines l
             JOIN journal_entries e ON e.id = l.entry_id
             JOIN accounts a ON a.id = l.account_id
             WHERE e.source_type = 'bikri' AND e.source_id = ?1 AND e.status = 'posted'
               AND a.code = ?2",
            (bikri_id, code),
            |r| Ok((r.get::<_, i64>(0)? / 100, r.get::<_, i64>(1)? / 100)),
        )
        .unwrap()
}

/// A seller page adds up inside itself: the seven deductions make the total deductions, and
/// gross − deductions is the net (or minus what the seller owes).
fn assert_seller_page_adds_up(page: &PrintPage) {
    let seven = [
        "print-freight",
        "print-commission",
        "print-labour",
        "print-munshiana",
        "print-cash-paid",
        "print-store-rent",
        "print-phone-charge",
    ]
    .iter()
    .map(|l| get(page, l))
    .sum::<i64>();
    assert_eq!(seven, get(page, "print-total-deductions"));
    let net = match amount(page, "print-net-sale") {
        Some(net) => net,
        None => -get(page, "print-seller-owes"),
    };
    assert_eq!(
        get(page, "print-gross-sale") - get(page, "print-total-deductions"),
        net
    );
}

/// A slip adds up inside itself (C5): value + commission + لاگا = what he pays.
fn assert_slip_adds_up(page: &PrintPage) {
    assert_eq!(
        get(page, "print-items-value")
            + get(page, "print-buyer-commission")
            + get(page, "print-laga"),
        get(page, "print-buyer-pays")
    );
}

/// Two sellers on one truck. رحیم paid the driver himself and اکرم's line has a buyer rate of
/// 610 over a seller rate of 600. کریم's lot has no charges. اکرم buys on both lots.
fn mixed(w: &World) -> BikriDraft {
    let mut akram = sale("s-1", w.akram, "20", "600");
    akram.rate_differs = true;
    akram.buyer_rate = Some("610".into());
    let mut rahim = with_sales(
        lot(w, "lot-1", w.rahim, w.banana, "50"),
        vec![akram, sale("s-2", w.arshad, "30", "580")],
    );
    rahim.freight_paid_by = FreightPaidBy::Seller;
    let karim = with_sales(
        bare_lot(w, "lot-2", w.karim, w.apple, "10"),
        vec![sale("s-3", w.akram, "10", "1200")],
    );
    draft(vec![rahim, karim])
}

#[test]
fn mixed_bikri_prints_the_stored_totals_and_the_journal() {
    let w = world();
    let id = save(&w, &mixed(&w));
    let detail = bikri::get(&w.env.ctx, id).unwrap();

    for language in [Language::Ur, Language::En] {
        let bill = data(&w, id, PrintDoc::SellerBill, language);
        assert_eq!(bill.pages.len(), 2, "two sellers, two bills");
        let (rahim, karim) = (bill.pages.first().unwrap(), bill.pages.get(1).unwrap());
        bill.pages.iter().for_each(assert_seller_page_adds_up);

        // رحیم: freight paid by himself is not deducted (PRD E6).
        assert_eq!(get(rahim, "print-gross-sale"), 29_400);
        assert_eq!(get(rahim, "print-freight"), 0);
        assert_eq!(get(rahim, "print-total-deductions"), 7_084);
        assert_eq!(get(rahim, "print-net-sale"), 22_316);
        let (dr, cr) = journal(&w, id, "030110001");
        assert_eq!((dr, cr), (5_000, 27_316));
        assert_eq!(get(rahim, "print-net-sale"), cr - dr);
        // No freight was posted to the transporters' account.
        assert_eq!(journal(&w, id, "02010002"), (0, 0));

        // کریم: only his commission is deducted.
        assert_eq!(get(karim, "print-gross-sale"), 12_000);
        assert_eq!(get(karim, "print-total-deductions"), 720);
        assert_eq!(get(karim, "print-net-sale"), 11_280);
        let (dr, cr) = journal(&w, id, "030110002");
        assert_eq!(get(karim, "print-net-sale"), cr - dr);

        // The two bills together are the بکری totals of bikri_get.
        let both = |label: &str| bill.pages.iter().map(|p| get(p, label)).sum::<i64>();
        assert_eq!(both("print-gross-sale"), detail.totals.gross.rupees());
        assert_eq!(
            both("print-total-deductions"),
            detail.totals.deductions.rupees()
        );
        assert_eq!(both("print-net-sale"), detail.totals.net.rupees());
        assert_eq!(both("print-commission"), journal(&w, id, "04010001").1);

        // The slips: اکرم's one page covers both lots, at the buyer rate.
        let slips = data(&w, id, PrintDoc::BuyerSlips, language);
        assert_eq!(slips.pages.len(), 2);
        slips.pages.iter().for_each(assert_slip_adds_up);
        let akram = slips.pages.first().unwrap();
        assert_eq!(akram.lines.len(), 2);
        assert_eq!(n(&akram.lines.first().unwrap().rate), 610);
        assert_eq!(n(&akram.lines.first().unwrap().amount), 12_200);
        assert_eq!(get(akram, "print-items-value"), 24_200);
        assert_eq!(get(akram, "print-buyer-commission"), 1_452);
        assert_eq!(get(akram, "print-laga"), 240);
        assert_eq!(get(akram, "print-buyer-pays"), 25_892);
        assert_eq!(
            get(akram, "print-buyer-pays"),
            journal(&w, id, "03040001").0
        );
        assert_eq!(
            akram.words.as_ref().unwrap().text,
            if language == Language::Ur {
                "پچیس ہزار آٹھ سو بانوے روپے"
            } else {
                "Twenty-five thousand eight hundred ninety-two rupees only"
            }
        );
        let arshad = slips.pages.get(1).unwrap();
        assert_eq!(get(arshad, "print-buyer-pays"), 18_684);
        assert_eq!(
            get(arshad, "print-buyer-pays"),
            journal(&w, id, "03040002").0
        );
        let slips_total = slips
            .pages
            .iter()
            .map(|p| get(p, "print-buyer-pays"))
            .sum::<i64>();
        assert_eq!(slips_total, detail.totals.buyer_total.rupees());
        // The seller bill keeps the seller rate: 20 × 600 = 12,000 for اکرم's line.
        assert_eq!(n(&rahim.lines.first().unwrap().rate), 600);
        assert_eq!(n(&rahim.lines.first().unwrap().amount), 12_000);
        // The rate difference of 200 is firm income in the journal, on no printed page.
        assert_eq!(journal(&w, id, "04010005"), (0, 200));
    }
}

#[test]
fn a_seller_who_owes_the_firm_matches_his_journal_debit() {
    let w = world();
    let id = save(
        &w,
        &draft(vec![with_sales(
            lot(&w, "lot-1", w.rahim, w.banana, "1"),
            vec![sale("s-1", w.akram, "1", "600")],
        )]),
    );
    for language in [Language::Ur, Language::En] {
        let bill = data(&w, id, PrintDoc::SellerBill, language);
        let page = bill.pages.first().unwrap();
        assert_seller_page_adds_up(page);
        assert_eq!(get(page, "print-gross-sale"), 600);
        assert_eq!(get(page, "print-total-deductions"), 8_111);
        assert_eq!(amount(page, "print-net-sale"), None);
        assert_eq!(get(page, "print-seller-owes"), 7_511);
        // The journal: the negative seller credit 2,511 and the نقد 5,000 are both debits.
        assert_eq!(journal(&w, id, "030110001"), (7_511, 0));
        // No minus sign anywhere on the page.
        assert!(page.amounts.iter().all(|a| !a.amount.contains('-')));
        assert_eq!(
            page.words.as_ref().unwrap().text,
            if language == Language::Ur {
                "سات ہزار پانچ سو گیارہ روپے"
            } else {
                "Seven thousand five hundred eleven rupees only"
            }
        );
    }
}
