//! Printing a saved بکری (plan 14, PRD FR-12.12, FR-7.36, FR-7.57, FR-19.3 to FR-19.7, D3-A15).
//!
//! The print data is built by `mandi-reports` from a REAL saved بکری of the PRD worked day. The
//! tests prove that every printed total equals `bikri_get` and the posted journal to the rupee
//! (a print never has a sum of its own), that a print raises the print count once with its audit
//! row, and that the Printing settings are saved with an audit row and used by the print data.
#![cfg(test)]

#[path = "common/mod.rs"]
mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::print::{BikriPrintDataInput, PrintData, bikri_print_data};
use mandi_services::bikri;
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{BikriDetail, BikriSaveInput, PukhtaInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::printing::{Paper, PrintDoc, PrintInput, PrintTo, SetPrintingInput};
use mandi_services::printing;

use common::raw;
use support::*;

/// Saves Event 1 (رحیم's 50 کیلا, اکرم 20 × 600 and ارشد 30 × 580) and gives its id.
fn save_event1(w: &World) -> i64 {
    save(w, &event1(w))
}

/// Saves Event 2: کریم's 10 سیب sold to the one-time buyer شاہد in cash, freight 1,000, no نقد.
fn save_event2(w: &World) -> i64 {
    let mut line = sale("s-1", w.pukhta_buyer, "10", "1200");
    line.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = with_sales(lot(w, "lot-1", w.karim, w.apple, "10"), vec![line]);
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    let mut d = draft(vec![lot]);
    d.header.truck_no = Some("LHR-0077".into());
    save(w, &d)
}

fn save(w: &World, draft: &mandi_services::core::bikri::BikriDraft) -> i64 {
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

/// A printed amount as whole rupees.
fn n(text: &str) -> i64 {
    text.replace(',', "").parse().unwrap()
}

/// The printed amount of one label on a page.
fn amount(page: &mandi_reports::print::PrintPage, label: &str) -> i64 {
    n(&page
        .amounts
        .iter()
        .find(|a| a.label_id == label)
        .unwrap_or_else(|| panic!("no {label}"))
        .amount)
}

/// What the posted journal of a بکری says: the credits and the debits on one account, in rupees.
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

fn dr(w: &World, id: i64, code: &str) -> i64 {
    journal(w, id, code).0
}

fn cr(w: &World, id: i64, code: &str) -> i64 {
    journal(w, id, code).1
}

// ---- Plan 17e: the shared header, the tag and the short code ----

/// Sets the firm's contacts and trade mark through the Settings action.
fn set_contacts(w: &World) {
    use mandi_services::core::firm::FirmContact;
    use mandi_services::settings::{self, FirmPrintInfo};
    let firm = settings::get(&w.env.ctx).unwrap().firm;
    let info = FirmPrintInfo {
        firm_contacts: vec![
            FirmContact {
                name_ur: "علی".into(),
                name_en: "Ali".into(),
                phone: "0300-4058860".into(),
            },
            FirmContact {
                name_ur: "اکرم".into(),
                name_en: "Akram".into(),
                phone: "03211234567".into(),
            },
        ],
        trade_mark: "RH".into(),
    };
    settings::set_firm_with_print_info(&w.env.ctx, &w.owner, &firm, &info, &[]).unwrap();
}

/// True if any string in this JSON holds an Arabic-Indic or Urdu digit.
fn has_urdu_digit(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(text) => text.chars().any(|c| {
            ('\u{0660}'..='\u{0669}').contains(&c) || ('\u{06F0}'..='\u{06F9}').contains(&c)
        }),
        serde_json::Value::Array(items) => items.iter().any(has_urdu_digit),
        serde_json::Value::Object(map) => map.values().any(has_urdu_digit),
        _ => false,
    }
}

#[test]
fn a_default_urdu_print_of_the_real_world_has_only_plain_digits_even_for_a_phone_typed_in_urdu_digits()
 {
    // Umar, 26-10-2026: the digits of every print are 0-9; the Urdu digits are an option, off by
    // default. A one-time buyer's phone typed with Urdu digits is saved, and printed, as 0-9.
    let w = world();
    let id1 = save_event1(&w);
    let mut line = sale("s-1", w.pukhta_buyer, "10", "1200");
    line.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "۰۳۰۰۰۰۰۰۰۰۰".into(),
    });
    let mut lot = with_sales(lot(&w, "lot-1", w.karim, w.apple, "10"), vec![line]);
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    let id2 = save(&w, &draft(vec![lot]));
    set_contacts(&w);
    for (id, doc) in [
        (id1, PrintDoc::SellerBill),
        (id1, PrintDoc::BuyerSlips),
        (id2, PrintDoc::PukhtaSlip),
    ] {
        let value = serde_json::to_value(data(&w, id, doc, Language::Ur)).unwrap();
        assert!(!has_urdu_digit(&value), "{doc:?}: {value}");
    }
    let slip = data(&w, id2, PrintDoc::PukhtaSlip, Language::Ur);
    assert_eq!(slip.pages[0].party.phone.as_deref(), Some("0300-0000000"));
}

#[test]
fn the_contacts_and_the_trade_mark_print_in_the_header_of_every_bikri_document() {
    // D4R-A11: up to 3 contacts and the trade mark show in the header; none set, no empty box.
    let w = world();
    let id1 = save_event1(&w);
    let id2 = save_event2(&w);
    let docs = [
        (id1, PrintDoc::SellerBill),
        (id1, PrintDoc::BuyerSlips),
        (id2, PrintDoc::PukhtaSlip),
    ];
    for (id, doc) in docs {
        for language in [Language::Ur, Language::En] {
            let none = data(&w, id, doc, language);
            assert!(none.header.contacts.is_empty(), "{doc:?}");
            assert_eq!(none.header.trade_mark, None, "{doc:?}");
        }
    }
    set_contacts(&w);
    for (id, doc) in docs {
        let ur = data(&w, id, doc, Language::Ur);
        let en = data(&w, id, doc, Language::En);
        assert_eq!(ur.header.contacts.len(), 2, "{doc:?}");
        assert_eq!(ur.header.contacts[0].name, "علی");
        assert_eq!(en.header.contacts[0].name, "Ali");
        assert_eq!(en.header.contacts[0].phone, "0300-4058860");
        assert_eq!(ur.header.trade_mark.as_deref(), Some("RH"));
        assert_eq!(en.header.trade_mark.as_deref(), Some("RH"));
        // The footer and the numbers are the ones they were.
        assert_eq!(en.footer.page_label_id, "print-page-label");
    }
    let bill = data(&w, id1, PrintDoc::SellerBill, Language::En);
    assert_eq!(amount(&bill.pages[0], "print-gross-sale"), 29_400);
    assert_eq!(amount(&bill.pages[0], "print-net-sale"), 19_316);
}

#[test]
fn a_buyer_with_a_tag_prints_with_it_and_every_party_has_its_short_code() {
    // D4R-A12: "اکرم ٹول ٹیکس K" with short code 1; the seller رحیم has short code 1 too.
    let w = world();
    let id = save_event1(&w);
    raw(&w.dir)
        .execute("UPDATE accounts SET tag = 'K' WHERE id = ?1", [w.akram])
        .unwrap();
    for language in [Language::Ur, Language::En] {
        let slips = data(&w, id, PrintDoc::BuyerSlips, language);
        let akram = &slips.pages[0].party;
        assert_eq!(akram.tag.as_deref(), Some("K"));
        assert_eq!(akram.short_code.as_deref(), Some("1"));
        assert_eq!(akram.code.as_deref(), Some("03040001"));
        let arshad = &slips.pages[1].party;
        assert_eq!(
            (arshad.tag.as_deref(), arshad.short_code.as_deref()),
            (None, Some("2"))
        );
        // The seller bill: the seller's short code, and the tag on the buyer's line.
        let bill = data(&w, id, PrintDoc::SellerBill, language);
        let page = &bill.pages[0];
        assert_eq!(page.party.short_code.as_deref(), Some("1"));
        assert_eq!(page.party.tag, None);
        let tags: Vec<_> = page.lines.iter().map(|l| l.name_tag.as_deref()).collect();
        assert_eq!(tags, vec![Some("K"), None]);
    }
    // The R5 columns: "(item qty . rate)", the value and the amount after لاگا, the row number.
    let en = data(&w, id, PrintDoc::BuyerSlips, Language::En);
    assert_eq!(en.pages[0].lines[0].detail, "(Banana 20 . 600)");
    assert_eq!(en.pages[1].lines[0].detail, "(Banana 30 . 580)");
    assert_eq!(
        (en.pages[0].serial.as_str(), en.pages[1].serial.as_str()),
        ("1", "2")
    );
    assert_eq!(amount(&en.pages[0], "print-items-value"), 12_000);
    assert_eq!(amount(&en.pages[0], "print-buyer-pays"), 12_880);
    assert_eq!(amount(&en.pages[1], "print-items-value"), 17_400);
    assert_eq!(amount(&en.pages[1], "print-buyer-pays"), 18_684);
    let ur = data(&w, id, PrintDoc::BuyerSlips, Language::Ur);
    assert_eq!(ur.pages[0].lines[0].detail, "(کیلا 20 . 600)");
}

// ---- Every printed total equals the screen and the journal ----

#[test]
fn event1_printed_totals_equal_bikri_get_and_the_journal() {
    let w = world();
    let id = save_event1(&w);
    let detail: BikriDetail = bikri::get(&w.env.ctx, id).unwrap();

    for language in [Language::Ur, Language::En] {
        // The seller bill: the same numbers as bikri_get.
        let bill = data(&w, id, PrintDoc::SellerBill, language);
        let page = bill.pages.first().unwrap();
        let t = &detail.totals;
        assert_eq!(amount(page, "print-gross-sale"), t.gross.rupees());
        assert_eq!(amount(page, "print-freight"), t.freight.rupees());
        assert_eq!(amount(page, "print-commission"), t.seller_comm.rupees());
        assert_eq!(amount(page, "print-labour"), t.labour.rupees());
        assert_eq!(amount(page, "print-munshiana"), t.munshiana.rupees());
        assert_eq!(amount(page, "print-cash-paid"), t.cash_paid.rupees());
        assert_eq!(amount(page, "print-store-rent"), t.store_rent.rupees());
        assert_eq!(amount(page, "print-phone-charge"), t.phone.rupees());
        assert_eq!(
            amount(page, "print-total-deductions"),
            t.deductions.rupees()
        );
        assert_eq!(amount(page, "print-net-sale"), t.net.rupees());
        assert_eq!(
            (t.gross.rupees(), t.deductions.rupees(), t.net.rupees()),
            (29_400, 10_084, 19_316)
        );

        // The same numbers, from the posted journal. Each deduction is a credit on its own
        // account, and the net is what the seller is credited less the cash he was given.
        assert_eq!(amount(page, "print-commission"), cr(&w, id, "04010001"));
        assert_eq!(amount(page, "print-freight"), cr(&w, id, "02010002"));
        assert_eq!(amount(page, "print-labour"), cr(&w, id, "02010001"));
        assert_eq!(amount(page, "print-munshiana"), cr(&w, id, "02010004"));
        assert_eq!(amount(page, "print-store-rent"), cr(&w, id, "02010003"));
        assert_eq!(amount(page, "print-phone-charge"), cr(&w, id, "04010004"));
        assert_eq!(amount(page, "print-cash-paid"), cr(&w, id, "01010001"));
        let seller_net = cr(&w, id, "030110001") - dr(&w, id, "030110001");
        assert_eq!(amount(page, "print-net-sale"), seller_net);
        // Gross = what the seller is credited plus everything taken off it in the same entry.
        let credited = cr(&w, id, "030110001");
        let taken = [
            "04010001", "02010002", "02010001", "02010004", "02010003", "04010004",
        ]
        .iter()
        .map(|code| cr(&w, id, code))
        .sum::<i64>();
        assert_eq!(amount(page, "print-gross-sale"), credited + taken);

        // The buyer slips: each total is the debit on that buyer's account.
        let slips = data(&w, id, PrintDoc::BuyerSlips, language);
        assert_eq!(slips.pages.len(), 2);
        let akram = slips.pages.first().unwrap();
        let arshad = slips.pages.get(1).unwrap();
        assert_eq!(amount(akram, "print-buyer-pays"), dr(&w, id, "03040001"));
        assert_eq!(amount(arshad, "print-buyer-pays"), dr(&w, id, "03040002"));
        assert_eq!(
            (
                amount(akram, "print-buyer-pays"),
                amount(arshad, "print-buyer-pays")
            ),
            (12_880, 18_684)
        );
        // All slips together are the buyer bills of bikri_get.
        assert_eq!(
            amount(akram, "print-buyer-pays") + amount(arshad, "print-buyer-pays"),
            detail.totals.buyer_total.rupees()
        );
        // The slips carry the buyer commission and لاگا of the journal.
        assert_eq!(
            amount(akram, "print-buyer-commission") + amount(arshad, "print-buyer-commission"),
            cr(&w, id, "04010002")
        );
        assert_eq!(
            amount(akram, "print-laga") + amount(arshad, "print-laga"),
            cr(&w, id, "04010003")
        );
        // Nothing for the one-time slip.
        assert!(
            data(&w, id, PrintDoc::PukhtaSlip, language)
                .pages
                .is_empty()
        );
    }
}

#[test]
fn event2_printed_totals_equal_bikri_get_and_the_journal() {
    let w = world();
    let id = save_event2(&w);
    let detail = bikri::get(&w.env.ctx, id).unwrap();

    for language in [Language::Ur, Language::En] {
        let slip = data(&w, id, PrintDoc::PukhtaSlip, language);
        assert_eq!(slip.pages.len(), 1);
        let page = slip.pages.first().unwrap();
        // 12,800: the debit on the پختہ کھاتہ buyer account, paid at once (a credit of the same
        // amount in the cash part).
        assert_eq!(amount(page, "print-buyer-pays"), 12_800);
        assert_eq!(amount(page, "print-buyer-pays"), dr(&w, id, "03049999"));
        assert_eq!(cr(&w, id, "03049999"), 12_800);
        assert_eq!(
            amount(page, "print-buyer-pays"),
            detail.totals.buyer_total.rupees()
        );
        assert_eq!(page.payment_id.as_deref(), Some("print-payment-cash"));
        assert_eq!(
            page.words.as_ref().unwrap().text,
            if language == Language::Ur {
                "بارہ ہزار آٹھ سو روپے"
            } else {
                "Twelve thousand eight hundred rupees only"
            }
        );
        // His phone is saved as 11 digits and printed as 03XX-XXXXXXX.
        assert_eq!(page.party.phone.as_deref(), Some("0300-0000000"));

        let bill = data(&w, id, PrintDoc::SellerBill, language);
        let page = bill.pages.first().unwrap();
        assert_eq!(amount(page, "print-net-sale"), 10_160);
        assert_eq!(amount(page, "print-net-sale"), detail.totals.net.rupees());
        assert_eq!(
            amount(page, "print-net-sale"),
            cr(&w, id, "030110002") - dr(&w, id, "030110002")
        );
        // The buyer slips leave the one-time buyer to his own slip.
        assert!(
            data(&w, id, PrintDoc::BuyerSlips, language)
                .pages
                .is_empty()
        );
    }
}

#[test]
fn the_firm_header_comes_from_settings_in_the_print_language() {
    let w = world();
    let id = save_event1(&w);
    let ur = data(&w, id, PrintDoc::SellerBill, Language::Ur);
    let en = data(&w, id, PrintDoc::SellerBill, Language::En);
    assert_eq!(ur.header.firm.name, "ماہر عدنان اینڈ برادرز");
    assert_eq!(en.header.firm.name, "Maher Adnan and Brothers");
    assert_eq!(en.header.firm.mandi, "New Sabzi Mandi Kot Radha Kishan");
    assert_eq!(ur.footer.printed_at, "10-10-2026 14:00");
}

#[test]
fn an_unknown_bikri_is_not_found() {
    let w = world();
    let error = bikri_print_data(
        w.env.ctx.db(),
        &BikriPrintDataInput {
            bikri_id: 99,
            doc: PrintDoc::SellerBill,
            language: Language::Ur,
            only_buyer_id: None,
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrNotFound);
}

// ---- The Printing settings ----

fn set_input() -> SetPrintingInput {
    SetPrintingInput {
        paper_seller_bill: Paper::A4,
        paper_buyer_slips: Paper::Mm80,
        paper_pukhta_slip: Paper::A5,
        paper_receipt_slip: Paper::A5,
        paper_collection_list: Paper::A4,
        printer: "  Office Laser  ".into(),
        copies: "2".into(),
        urdu_digits: true,
        print_after_save: vec![
            PrintDoc::SellerBill,
            PrintDoc::SellerBill,
            PrintDoc::BuyerSlips,
        ],
    }
}

fn audit_rows(w: &World, action: &str) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = ?1",
            [action],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn printing_settings_start_with_a21_and_are_saved_with_an_audit_row() {
    let w = world();
    let start = printing::get_settings(&w.env.ctx, &w.owner).unwrap();
    assert_eq!(start.paper_seller_bill, Paper::A4);
    assert_eq!(start.paper_buyer_slips, Paper::A5);
    assert_eq!(start.paper_pukhta_slip, Paper::A5);
    assert_eq!(start.copies, 1);
    assert!(!start.urdu_digits);
    assert!(start.printer.is_empty());
    assert_eq!(start.paper_receipt_slip, Paper::A5);
    assert_eq!(start.paper_collection_list, Paper::A4);
    assert_eq!(start.print_after_save, PrintDoc::BIKRI_DOCS.to_vec());

    let edits_before = audit_rows(&w, "edit");
    let saved = printing::set_settings(&w.env.ctx, &w.owner, &set_input()).unwrap();
    assert_eq!(saved.printer, "Office Laser", "trimmed");
    assert_eq!(saved.copies, 2);
    // A repeated document counts once, in the order first given.
    assert_eq!(
        saved.print_after_save,
        vec![PrintDoc::SellerBill, PrintDoc::BuyerSlips]
    );
    assert_eq!(printing::get_settings(&w.env.ctx, &w.owner).unwrap(), saved);
    assert_eq!(audit_rows(&w, "edit"), edits_before + 1);

    // The audit row holds the old and the new value.
    let (old, new): (String, String) = raw(&w.dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log
             WHERE action = 'edit' AND entity = 'settings' ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(old.contains("\"copies\":1"), "{old}");
    assert!(new.contains("\"copies\":2"), "{new}");
    assert!(new.contains("\"paper_buyer_slips\":\"mm80\""), "{new}");
    assert!(new.contains("Office Laser"), "{new}");

    // Saving the same values again changes nothing and writes no second row.
    printing::set_settings(&w.env.ctx, &w.owner, &set_input()).unwrap();
    assert_eq!(audit_rows(&w, "edit"), edits_before + 1);
}

#[test]
fn bad_printing_settings_are_refused_with_codes_at_their_fields() {
    let w = world();
    let edits_before = audit_rows(&w, "edit");
    let bad = |copies: &str, printer: &str| {
        let mut input = set_input();
        input.copies = copies.into();
        input.printer = printer.into();
        printing::set_settings(&w.env.ctx, &w.owner, &input).unwrap_err()
    };
    let code_at = |error: mandi_services::ServiceError, code: ErrorCode, field: &str| {
        let mandi_services::ServiceError::Rejected(items) = error else {
            panic!("not a rejection");
        };
        assert!(
            items
                .iter()
                .any(|i| i.code == code && i.field.as_deref() == Some(field)),
            "{items:?}"
        );
    };
    code_at(bad("", ""), ErrorCode::ErrRequired, "copies");
    code_at(bad("two", ""), ErrorCode::ErrNotANumber, "copies");
    code_at(bad("0", ""), ErrorCode::ErrOutOfRange, "copies");
    code_at(bad("100", ""), ErrorCode::ErrOutOfRange, "copies");
    code_at(
        bad("1", &"x".repeat(260)),
        ErrorCode::ErrTextTooLong,
        "printer",
    );
    code_at(
        bad("1", "Laser\u{0007}"),
        ErrorCode::ErrControlChars,
        "printer",
    );
    // Both are reported together.
    let mandi_services::ServiceError::Rejected(items) = bad("0", &"x".repeat(260)) else {
        panic!("not a rejection");
    };
    assert_eq!(items.len(), 2);
    // Nothing was saved by the refused calls.
    assert_eq!(audit_rows(&w, "edit"), edits_before);
}

#[test]
fn the_print_data_uses_the_paper_and_digits_of_the_settings() {
    let w = world();
    let id = save_event1(&w);
    assert_eq!(
        data(&w, id, PrintDoc::SellerBill, Language::Ur).paper,
        Paper::A4
    );
    printing::set_settings(&w.env.ctx, &w.owner, &set_input()).unwrap();
    let slips = data(&w, id, PrintDoc::BuyerSlips, Language::Ur);
    assert_eq!(slips.paper, Paper::Mm80);
    assert!(slips.urdu_digits);
    assert_eq!(
        slips.pages.first().unwrap().amounts.last().unwrap().amount,
        "۱۲,۸۸۰"
    );
    // The Urdu digits switch does nothing on an English print.
    let en = data(&w, id, PrintDoc::BuyerSlips, Language::En);
    assert!(!en.urdu_digits);
    assert_eq!(
        en.pages.first().unwrap().amounts.last().unwrap().amount,
        "12,880"
    );
}

// ---- A print: the checks, the count and the audit row ----

fn request(id: i64) -> PrintInput {
    PrintInput {
        bikri_id: id,
        docs: vec![PrintDoc::SellerBill, PrintDoc::BuyerSlips],
        language: Language::Ur,
        copies: None,
        to: PrintTo::Printer,
        only_buyer_id: None,
    }
}

#[test]
fn a_print_request_is_checked_before_anything_happens() {
    let w = world();
    let id = save_event1(&w);
    let code_of = |input: PrintInput| printing::plan(&w.env.ctx, &input).unwrap_err().code();

    let mut none = request(id);
    none.docs.clear();
    assert_eq!(code_of(none), ErrorCode::ErrRequired);
    assert_eq!(code_of(request(99)), ErrorCode::ErrNotFound);
    for bad in ["0", "100", "x"] {
        let mut input = request(id);
        input.copies = Some(bad.into());
        let error = printing::plan(&w.env.ctx, &input).unwrap_err();
        assert!(
            error.has_code(ErrorCode::ErrOutOfRange) || error.has_code(ErrorCode::ErrNotANumber),
            "{bad}"
        );
    }

    // A good request: the copies of the settings unless the request has its own.
    let plan = printing::plan(&w.env.ctx, &request(id)).unwrap();
    assert_eq!((plan.number, plan.copies), (1, 1));
    assert_eq!(plan.docs, vec![PrintDoc::SellerBill, PrintDoc::BuyerSlips]);
    printing::set_settings(&w.env.ctx, &w.owner, &set_input()).unwrap();
    assert_eq!(printing::plan(&w.env.ctx, &request(id)).unwrap().copies, 2);
    let mut own = request(id);
    own.copies = Some(" 5 ".into());
    own.docs = vec![PrintDoc::SellerBill, PrintDoc::SellerBill];
    let plan = printing::plan(&w.env.ctx, &own).unwrap();
    assert_eq!(plan.copies, 5);
    assert_eq!(plan.docs, vec![PrintDoc::SellerBill], "each document once");
    // Checking a request writes nothing.
    assert_eq!(audit_rows(&w, "print"), 0);
    assert_eq!(bikri::get(&w.env.ctx, id).unwrap().print_count, 0);
}

#[test]
fn a_print_raises_the_count_once_and_writes_one_audit_row() {
    let w = world();
    let id = save_event1(&w);
    assert_eq!(bikri::get(&w.env.ctx, id).unwrap().print_count, 0);
    let version_before: i64 = raw(&w.dir)
        .query_row(
            "SELECT version FROM bikri_entries WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();

    // One print of two documents raises the count by one.
    let plan = printing::plan(&w.env.ctx, &request(id)).unwrap();
    let count = printing::record(
        &w.env.ctx,
        &w.owner,
        &plan,
        &[PrintDoc::SellerBill, PrintDoc::BuyerSlips],
    )
    .unwrap();
    assert_eq!(count, 1);
    assert_eq!(bikri::get(&w.env.ctx, id).unwrap().print_count, 1);
    assert_eq!(audit_rows(&w, "print"), 1);
    let (entity, entity_id, new): (String, i64, String) = raw(&w.dir)
        .query_row(
            "SELECT entity, entity_id, new_json FROM audit_log WHERE action = 'print'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!((entity.as_str(), entity_id), ("bikri", id));
    assert!(
        new.contains("\"docs\":[\"seller_bill\",\"buyer_slips\"]"),
        "{new}"
    );
    assert!(new.contains("\"print_count\":1"), "{new}");
    assert!(new.contains("\"language\":\"ur\""), "{new}");

    // A second print makes it 2. A print changes no business value, so the row's version
    // (which a screen that edits the بکری compares) does not move.
    let again = printing::record(&w.env.ctx, &w.owner, &plan, &[PrintDoc::SellerBill]).unwrap();
    assert_eq!(again, 2);
    assert_eq!(audit_rows(&w, "print"), 2);
    let version_after: i64 = raw(&w.dir)
        .query_row(
            "SELECT version FROM bikri_entries WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(version_after, version_before);

    // The audit chain is still whole after the print rows (NFR-SEC-6).
    common::assert_chain_ok(&w.dir);
}

#[test]
fn recording_a_print_of_a_missing_bikri_writes_nothing() {
    let w = world();
    let id = save_event1(&w);
    let mut plan = printing::plan(&w.env.ctx, &request(id)).unwrap();
    plan.bikri_id = 99;
    let error = printing::record(&w.env.ctx, &w.owner, &plan, &[PrintDoc::SellerBill]).unwrap_err();
    assert!(error.has_code(ErrorCode::ErrNotFound));
    assert_eq!(audit_rows(&w, "print"), 0);
}

#[test]
fn a_damaged_printing_setting_never_stops_a_print() {
    let w = world();
    let id = save_event1(&w);
    // Someone (or a bad restore) left text that is not the settings shape.
    raw(&w.dir)
        .execute(
            "INSERT INTO settings (key, value_json) VALUES ('printing', '{\"copies\":\"many\"}')",
            [],
        )
        .unwrap();
    let start = printing::get_settings(&w.env.ctx, &w.owner).unwrap();
    assert_eq!((start.copies, start.paper_seller_bill), (1, Paper::A4));
    assert_eq!(
        data(&w, id, PrintDoc::SellerBill, Language::En).paper,
        Paper::A4
    );
    // A save replaces it with a good value.
    printing::set_settings(&w.env.ctx, &w.owner, &set_input()).unwrap();
    assert_eq!(
        printing::get_settings(&w.env.ctx, &w.owner).unwrap().copies,
        2
    );
}
