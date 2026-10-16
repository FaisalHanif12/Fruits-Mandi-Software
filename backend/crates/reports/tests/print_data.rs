//! Print data of a saved بکری (plan 14, D3-A15). The input is the real answer of `bikri_get`
//! for the PRD worked day (the fixtures that the services tests keep true), so these tests also
//! prove the print shows the stored values and nothing else.
//!
//! The snapshots (`insta`) hold the whole print data of the three documents in both languages.
//! The tests around them check the worked numbers of PRD 4.4 and 4.5 by hand, so a wrong
//! snapshot can never be accepted by accident.
#![cfg(test)]

use chrono::{TimeZone, Utc};
use mandi_core::auth::Language;
use mandi_core::bikri::BikriDetail;
use mandi_core::money::Grouping;
use mandi_core::printing::{Paper, PrintDoc, PrintSetup};
use mandi_reports::print::{
    AmountRole, BuildOptions, FirmNames, PrintAmount, PrintData, PrintPage, TextDir, build,
};

const EVENT1: &str = include_str!("../../services/tests/fixtures/bikri_get_event1.json");
const EVENT2: &str = include_str!("../../services/tests/fixtures/bikri_get_event2.json");

fn detail(json: &str) -> BikriDetail {
    serde_json::from_str(json).unwrap()
}

fn firm() -> FirmNames {
    FirmNames {
        name_ur: "ماہر عدنان اینڈ برادرز".into(),
        name_en: "Maher Adnan and Brothers".into(),
        mandi_ur: "نیو سبزی منڈی کوٹ رادھا کشن".into(),
        mandi_en: "New Sabzi Mandi, Kot Radha Kishan".into(),
        address: "Kot Radha Kishan".into(),
        phone: "03001234567".into(),
    }
}

fn options(doc: PrintDoc, language: Language) -> BuildOptions {
    BuildOptions {
        doc,
        language,
        grouping: Grouping::International,
        setup: PrintSetup::default(),
        firm: firm(),
        // 09:00 UTC on 10-10-2026 is 14:00 in Pakistan.
        now_utc: Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap(),
        only_buyer_id: None,
    }
}

fn print(json: &str, doc: PrintDoc, language: Language) -> PrintData {
    build(&detail(json), &options(doc, language)).unwrap()
}

/// The amount lines of a page as (label id, amount, role).
fn amounts(page: &PrintPage) -> Vec<(&str, &str, AmountRole)> {
    page.amounts
        .iter()
        .map(|a: &PrintAmount| (a.label_id.as_str(), a.amount.as_str(), a.role))
        .collect()
}

/// Gives the first lot another net, with the gross moved by the same amount so the stored totals
/// still add up (gross less deductions is the net, C7).
fn set_net(d: &mut BikriDetail, net_rupees: i64) {
    let totals = &mut d.lots.first_mut().unwrap().totals;
    totals.net = mandi_core::money::Paisa::from_rupees(net_rupees).unwrap();
    totals.gross = totals.net.checked_add(totals.deductions).unwrap();
}

// ---- Snapshots: the three documents, both languages ----

#[test]
fn snapshot_seller_bill_event1_urdu() {
    insta::assert_json_snapshot!(print(EVENT1, PrintDoc::SellerBill, Language::Ur));
}

#[test]
fn snapshot_seller_bill_event1_english() {
    insta::assert_json_snapshot!(print(EVENT1, PrintDoc::SellerBill, Language::En));
}

#[test]
fn snapshot_buyer_slips_event1_urdu() {
    insta::assert_json_snapshot!(print(EVENT1, PrintDoc::BuyerSlips, Language::Ur));
}

#[test]
fn snapshot_buyer_slips_event1_english() {
    insta::assert_json_snapshot!(print(EVENT1, PrintDoc::BuyerSlips, Language::En));
}

#[test]
fn snapshot_pukhta_slip_event2_urdu() {
    insta::assert_json_snapshot!(print(EVENT2, PrintDoc::PukhtaSlip, Language::Ur));
}

#[test]
fn snapshot_pukhta_slip_event2_english() {
    insta::assert_json_snapshot!(print(EVENT2, PrintDoc::PukhtaSlip, Language::En));
}

#[test]
fn snapshot_seller_bill_event2_english() {
    insta::assert_json_snapshot!(print(EVENT2, PrintDoc::SellerBill, Language::En));
}

// ---- The worked numbers of PRD 4.4, checked by hand ----

#[test]
fn seller_bill_of_event1_has_the_worked_numbers_in_both_languages() {
    for language in [Language::Ur, Language::En] {
        let data = print(EVENT1, PrintDoc::SellerBill, language);
        assert_eq!(data.pages.len(), 1);
        let page = data.pages.first().unwrap();
        assert_eq!(
            amounts(page),
            vec![
                ("print-gross-sale", "29,400", AmountRole::Subtotal),
                ("print-freight", "3,000", AmountRole::Line),
                ("print-commission", "1,764", AmountRole::Line),
                ("print-labour", "250", AmountRole::Line),
                ("print-munshiana", "50", AmountRole::Line),
                ("print-cash-paid", "5,000", AmountRole::Line),
                ("print-store-rent", "0", AmountRole::Line),
                ("print-phone-charge", "20", AmountRole::Line),
                ("print-total-deductions", "10,084", AmountRole::Subtotal),
                ("print-net-sale", "19,316", AmountRole::Total),
            ],
            "{language:?}"
        );
        let words = &page.words.as_ref().unwrap().text;
        match language {
            Language::Ur => assert_eq!(words, "انیس ہزار تین سو سولہ روپے"),
            Language::En => {
                assert_eq!(words, "Nineteen thousand three hundred sixteen rupees only")
            }
        }
        // The lines: the buyers with the seller's rates and amounts.
        let lines: Vec<_> = page
            .lines
            .iter()
            .map(|l| (l.qty.as_str(), l.rate.as_str(), l.amount.as_str()))
            .collect();
        assert_eq!(
            lines,
            vec![("20", "600", "12,000"), ("30", "580", "17,400")]
        );
    }
}

#[test]
fn seller_bill_names_follow_the_print_language() {
    let ur = print(EVENT1, PrintDoc::SellerBill, Language::Ur);
    let en = print(EVENT1, PrintDoc::SellerBill, Language::En);
    assert_eq!(ur.dir, TextDir::Rtl);
    assert_eq!(en.dir, TextDir::Ltr);
    let (ur_page, en_page) = (ur.pages.first().unwrap(), en.pages.first().unwrap());
    assert_eq!(ur_page.party.name, "رحیم");
    assert_eq!(en_page.party.name, "Rahim");
    assert_eq!(ur_page.party.code.as_deref(), Some("030110001"));
    assert_eq!(
        ur_page.lines.first().unwrap().name.as_deref(),
        Some("اکرم ٹول ٹیکس")
    );
    assert_eq!(
        en_page.lines.first().unwrap().name.as_deref(),
        Some("Akram Toll Tax")
    );
    assert_eq!(en_page.lines.first().unwrap().item, "Banana");
    assert_eq!(en_page.lines.first().unwrap().packing, "Box");
    assert_eq!(ur.firm.name, "ماہر عدنان اینڈ برادرز");
    assert_eq!(en.firm.name, "Maher Adnan and Brothers");
    assert_eq!(en.firm.mandi, "New Sabzi Mandi, Kot Radha Kishan");
}

#[test]
fn header_facts_and_dates_follow_the_plan() {
    let data = print(EVENT1, PrintDoc::SellerBill, Language::En);
    let facts: Vec<_> = data
        .facts
        .iter()
        .map(|f| (f.label_id.as_str(), f.value.as_str()))
        .collect();
    // The bill date equals the posting day here, so it is not repeated. The truck is as typed.
    assert_eq!(
        facts,
        vec![
            ("print-fact-number", "1"),
            ("print-fact-date", "10-10-2026"),
            ("print-fact-truck", "LES-1234"),
        ]
    );
    assert_eq!(data.printed_at, "10-10-2026 14:00");
    assert_eq!(data.title_id, "print-seller-bill-title");
    assert!(!data.cancelled);
    assert_eq!(
        data.pages.first().unwrap().lines.first().unwrap().date,
        "10-10-2026"
    );

    let mut later = detail(EVENT1);
    later.bill_date = "2026-10-12".into();
    let data = build(&later, &options(PrintDoc::SellerBill, Language::En)).unwrap();
    let ids: Vec<_> = data.facts.iter().map(|f| f.label_id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "print-fact-number",
            "print-fact-date",
            "print-fact-bill-date",
            "print-fact-truck"
        ]
    );
    assert_eq!(data.facts.get(2).unwrap().value, "12-10-2026");
}

#[test]
fn buyer_slips_print_all_gives_one_page_per_buyer() {
    for language in [Language::Ur, Language::En] {
        let data = print(EVENT1, PrintDoc::BuyerSlips, language);
        assert_eq!(data.pages.len(), 2, "one slip per buyer");
        let akram = data.pages.first().unwrap();
        let arshad = data.pages.get(1).unwrap();
        assert_eq!(
            amounts(akram),
            vec![
                ("print-items-value", "12,000", AmountRole::Subtotal),
                ("print-buyer-commission", "720", AmountRole::Line),
                ("print-laga", "160", AmountRole::Line),
                ("print-buyer-pays", "12,880", AmountRole::Total),
            ]
        );
        assert_eq!(
            amounts(arshad),
            vec![
                ("print-items-value", "17,400", AmountRole::Subtotal),
                ("print-buyer-commission", "1,044", AmountRole::Line),
                ("print-laga", "240", AmountRole::Line),
                ("print-buyer-pays", "18,684", AmountRole::Total),
            ]
        );
        let line = akram.lines.first().unwrap();
        assert_eq!(
            (line.qty.as_str(), line.rate.as_str(), line.amount.as_str()),
            ("20", "600", "12,000")
        );
        let line = arshad.lines.first().unwrap();
        assert_eq!(
            (line.qty.as_str(), line.rate.as_str(), line.amount.as_str()),
            ("30", "580", "17,400")
        );
        // A slip is for the buyer: the lines carry no buyer name, the page does.
        assert!(line.name.is_none());
        assert_eq!(
            akram.party.name,
            if language == Language::Ur {
                "اکرم ٹول ٹیکس"
            } else {
                "Akram Toll Tax"
            }
        );
        assert_eq!(akram.payment_id.as_deref(), Some("print-payment-credit"));
        // The slips of buyers on credit are on A5 by default (A21).
        assert_eq!(data.paper, Paper::A5);
        assert_eq!(data.title_id, "print-buyer-slip-title");
    }
}

#[test]
fn buyer_slip_words_in_both_languages() {
    let ur = print(EVENT1, PrintDoc::BuyerSlips, Language::Ur);
    let en = print(EVENT1, PrintDoc::BuyerSlips, Language::En);
    assert_eq!(
        ur.pages.first().unwrap().words.as_ref().unwrap().text,
        "بارہ ہزار آٹھ سو اسی روپے"
    );
    assert_eq!(
        en.pages.first().unwrap().words.as_ref().unwrap().text,
        "Twelve thousand eight hundred eighty rupees only"
    );
    assert_eq!(
        en.pages.get(1).unwrap().words.as_ref().unwrap().text,
        "Eighteen thousand six hundred eighty-four rupees only"
    );
}

#[test]
fn only_one_buyer_gives_only_his_slip() {
    let mut opts = options(PrintDoc::BuyerSlips, Language::En);
    let detail = detail(EVENT1);
    let arshad_id = detail
        .lots
        .first()
        .and_then(|l| l.sales.get(1))
        .map(|s| s.buyer.id)
        .unwrap();
    opts.only_buyer_id = Some(arshad_id);
    let data = build(&detail, &opts).unwrap();
    assert_eq!(data.pages.len(), 1);
    assert_eq!(data.pages.first().unwrap().party.name, "Arshad Multani");
    // A buyer who is not in this بکری gives no page.
    opts.only_buyer_id = Some(999_999);
    assert!(build(&detail, &opts).unwrap().pages.is_empty());
}

// ---- The worked numbers of PRD 4.5: the one-time buyer ----

#[test]
fn pukhta_slip_of_event2_shows_12800_paid_and_its_words() {
    for language in [Language::Ur, Language::En] {
        let data = print(EVENT2, PrintDoc::PukhtaSlip, language);
        assert_eq!(data.pages.len(), 1);
        let page = data.pages.first().unwrap();
        assert_eq!(
            amounts(page),
            vec![
                ("print-items-value", "12,000", AmountRole::Subtotal),
                ("print-buyer-commission", "720", AmountRole::Line),
                ("print-laga", "80", AmountRole::Line),
                ("print-buyer-pays", "12,800", AmountRole::Total),
            ]
        );
        assert_eq!(page.payment_id.as_deref(), Some("print-payment-cash"));
        assert_eq!(page.party.role_id, "print-party-onetime-buyer");
        assert_eq!(page.party.code, None);
        assert_eq!(page.party.phone.as_deref(), Some("03000000000"));
        let words = &page.words.as_ref().unwrap().text;
        match language {
            Language::Ur => {
                assert_eq!(words, "بارہ ہزار آٹھ سو روپے");
                assert_eq!(page.party.name, "شاہد");
            }
            Language::En => {
                assert_eq!(words, "Twelve thousand eight hundred rupees only");
                assert_eq!(page.party.name, "Shahid");
            }
        }
        let line = page.lines.first().unwrap();
        assert_eq!(
            (line.qty.as_str(), line.rate.as_str(), line.amount.as_str()),
            ("10", "1,200", "12,000")
        );
        assert_eq!(data.title_id, "print-pukhta-slip-title");
    }
}

#[test]
fn the_other_documents_leave_the_one_time_buyer_out_and_the_reverse() {
    // Event 2 has only a one-time buyer: no buyer slip, one پختہ slip.
    assert!(
        print(EVENT2, PrintDoc::BuyerSlips, Language::En)
            .pages
            .is_empty()
    );
    // Event 1 has no one-time buyer: nothing to print for the پختہ slip.
    assert!(
        print(EVENT1, PrintDoc::PukhtaSlip, Language::En)
            .pages
            .is_empty()
    );
    // The seller bill of Event 2 lists the one-time buyer by his own name.
    let bill = print(EVENT2, PrintDoc::SellerBill, Language::En);
    let page = bill.pages.first().unwrap();
    assert_eq!(page.lines.first().unwrap().name.as_deref(), Some("Shahid"));
    assert_eq!(page.party.name, "Karim");
    assert_eq!(
        amounts(page).last().copied(),
        Some(("print-net-sale", "10,160", AmountRole::Total))
    );
}

// ---- Other shapes of a بکری ----

#[test]
fn one_seller_with_two_lots_gets_one_page_and_two_sellers_get_two() {
    let mut d = detail(EVENT1);
    let mut second = d.lots.first().unwrap().clone();
    second.id = 2;
    second.lot_no = 2;
    d.lots.push(second.clone());
    d.totals = d.totals.plus(&second.totals).unwrap();
    let bill = build(&d, &options(PrintDoc::SellerBill, Language::En)).unwrap();
    assert_eq!(bill.pages.len(), 1, "same seller, one bill");
    let page = bill.pages.first().unwrap();
    assert_eq!(page.lines.len(), 4);
    assert_eq!(amounts(page).first().unwrap().1, "58,800");
    assert_eq!(amounts(page).last().unwrap().1, "38,632");

    // A second seller on the same truck gets his own page.
    d.lots.last_mut().unwrap().seller.id = 99;
    d.lots.last_mut().unwrap().seller.code = "030110099".into();
    let bill = build(&d, &options(PrintDoc::SellerBill, Language::En)).unwrap();
    assert_eq!(bill.pages.len(), 2);
    assert_eq!(
        bill.pages.get(1).unwrap().party.code.as_deref(),
        Some("030110099")
    );
    assert_eq!(
        amounts(bill.pages.first().unwrap()).first().unwrap().1,
        "29,400"
    );
}

#[test]
fn a_seller_who_owes_the_firm_is_printed_without_a_minus() {
    let mut d = detail(EVENT1);
    // Net -1,200 (PRD FR-12.7, E28): the stored lot values say so; the print shows what is stored.
    set_net(&mut d, -1_200);
    let bill = build(&d, &options(PrintDoc::SellerBill, Language::En)).unwrap();
    let page = bill.pages.first().unwrap();
    assert_eq!(
        amounts(page).last().copied(),
        Some(("print-seller-owes", "1,200", AmountRole::Total))
    );
    assert_eq!(
        page.words.as_ref().unwrap().text,
        "One thousand two hundred rupees only"
    );
}

#[test]
fn mixed_payment_types_give_no_single_mark() {
    let mut d = detail(EVENT1);
    let sale = d.lots.first_mut().unwrap().sales.first_mut().unwrap();
    let buyer = sale.buyer.id;
    // A second line of the same buyer, paid in cash.
    let mut cash = sale.clone();
    cash.id = 99;
    cash.payment_type = mandi_core::bikri::PaymentType::Cash;
    d.lots.first_mut().unwrap().sales.push(cash);
    let mut opts = options(PrintDoc::BuyerSlips, Language::En);
    opts.only_buyer_id = Some(buyer);
    let page = build(&d, &opts).unwrap().pages.into_iter().next().unwrap();
    assert_eq!(page.lines.len(), 2);
    assert_eq!(page.payment_id, None);
}

#[test]
fn replaced_lines_are_not_printed_and_a_cancelled_bikri_is_flagged() {
    let mut d = detail(EVENT1);
    d.lots
        .first_mut()
        .unwrap()
        .sales
        .first_mut()
        .unwrap()
        .is_active = false;
    d.status = mandi_core::bikri::BikriStatus::Cancelled;
    let slips = build(&d, &options(PrintDoc::BuyerSlips, Language::En)).unwrap();
    assert!(slips.cancelled);
    assert_eq!(slips.pages.len(), 1, "the replaced line has no slip");
    assert_eq!(slips.pages.first().unwrap().party.name, "Arshad Multani");
}

// ---- Digits, grouping, paper ----

#[test]
fn urdu_digits_only_on_an_urdu_print_with_the_setting_on() {
    let mut opts = options(PrintDoc::SellerBill, Language::Ur);
    opts.setup.urdu_digits = true;
    let ur = build(&detail(EVENT1), &opts).unwrap();
    assert!(ur.urdu_digits);
    assert_eq!(ur.number, "۱");
    assert_eq!(ur.printed_at, "۱۰-۱۰-۲۰۲۶ ۱۴:۰۰");
    let page = ur.pages.first().unwrap();
    assert_eq!(amounts(page).first().unwrap().1, "۲۹,۴۰۰");
    assert_eq!(page.lines.first().unwrap().qty, "۲۰");
    // The truck number and the code stay as typed.
    assert_eq!(ur.facts.last().unwrap().value, "LES-1234");
    assert_eq!(page.party.code.as_deref(), Some("030110001"));
    // The setting does nothing on an English print.
    opts.language = Language::En;
    let en = build(&detail(EVENT1), &opts).unwrap();
    assert!(!en.urdu_digits);
    assert_eq!(
        amounts(en.pages.first().unwrap()).first().unwrap().1,
        "29,400"
    );
}

#[test]
fn lakh_grouping_changes_the_numbers_and_the_english_words() {
    let mut opts = options(PrintDoc::SellerBill, Language::En);
    opts.grouping = Grouping::Lakh;
    let mut d = detail(EVENT1);
    // A net of 1,316,040: the PRD FR-19.7 example for both groupings.
    set_net(&mut d, 1_316_040);
    let page = build(&d, &opts).unwrap().pages.into_iter().next().unwrap();
    assert_eq!(amounts(&page).last().unwrap().1, "13,16,040");
    assert_eq!(
        page.words.unwrap().text,
        "Thirteen lakh sixteen thousand forty rupees only"
    );
    opts.grouping = Grouping::International;
    let page = build(&d, &opts).unwrap().pages.into_iter().next().unwrap();
    assert_eq!(amounts(&page).last().unwrap().1, "1,316,040");
    assert_eq!(
        page.words.unwrap().text,
        "One million three hundred sixteen thousand forty rupees only"
    );
}

#[test]
fn paper_comes_from_the_printing_settings_per_document() {
    let mut opts = options(PrintDoc::SellerBill, Language::En);
    assert_eq!(build(&detail(EVENT1), &opts).unwrap().paper, Paper::A4);
    opts.setup.paper_seller_bill = Paper::A5;
    assert_eq!(build(&detail(EVENT1), &opts).unwrap().paper, Paper::A5);
    opts.doc = PrintDoc::BuyerSlips;
    opts.setup.paper_buyer_slips = Paper::Mm80;
    assert_eq!(build(&detail(EVENT1), &opts).unwrap().paper, Paper::Mm80);
}

#[test]
fn a_weight_item_prints_its_kilos_and_a_note_prints_as_typed() {
    let mut d = detail(EVENT1);
    let sale = d.lots.first_mut().unwrap().sales.first_mut().unwrap();
    sale.weight_grams = Some(12_345);
    sale.note = Some("  ضروری  ".into());
    let bill = build(&d, &options(PrintDoc::SellerBill, Language::En)).unwrap();
    let line = bill.pages.first().unwrap().lines.first().unwrap();
    assert_eq!(line.weight_kg.as_deref(), Some("12.345"));
    assert_eq!(line.note.as_deref(), Some("ضروری"));
}

#[test]
fn the_print_data_json_has_the_snake_case_shape_the_template_reads() {
    let json = serde_json::to_value(print(EVENT1, PrintDoc::BuyerSlips, Language::En)).unwrap();
    assert_eq!(json["doc"], "buyer_slips");
    assert_eq!(json["language"], "en");
    assert_eq!(json["dir"], "ltr");
    assert_eq!(json["paper"], "a5");
    assert_eq!(json["title_id"], "print-buyer-slip-title");
    assert_eq!(json["pages"][0]["amounts"][3]["role"], "total");
    assert_eq!(json["pages"][0]["words"]["label_id"], "print-in-words");
}

// ---- Stored totals that do not add up stop the print ----

#[test]
fn a_bill_whose_stored_totals_do_not_add_up_is_not_printed() {
    let build_with = |change: &dyn Fn(&mut BikriDetail)| {
        let mut d = detail(EVENT1);
        change(&mut d);
        build(&d, &options(PrintDoc::SellerBill, Language::En))
    };
    let money = |n: i64| mandi_core::money::Paisa::from_rupees(n).unwrap();
    assert!(build_with(&|_| {}).is_ok());
    // The net is not gross less deductions (C7).
    let wrong_net = build_with(&|d| d.lots.first_mut().unwrap().totals.net = money(19_317));
    assert!(matches!(
        wrong_net,
        Err(mandi_reports::ReportsError::Data(_))
    ));
    // The seven deductions do not make the total deductions (C6).
    let wrong_sum = build_with(&|d| d.lots.first_mut().unwrap().totals.labour = money(251));
    assert!(matches!(
        wrong_sum,
        Err(mandi_reports::ReportsError::Data(_))
    ));
}

#[test]
fn a_slip_line_that_does_not_add_up_is_not_printed() {
    let mut d = detail(EVENT1);
    // 12,000 + 720 + 160 = 12,880 (C5); 12,881 is not.
    d.lots
        .first_mut()
        .unwrap()
        .sales
        .first_mut()
        .unwrap()
        .buyer_total = mandi_core::money::Paisa::from_rupees(12_881).unwrap();
    let slips = build(&d, &options(PrintDoc::BuyerSlips, Language::En));
    assert!(matches!(slips, Err(mandi_reports::ReportsError::Data(_))));
}

#[test]
fn a_total_too_big_for_words_is_a_clear_refusal() {
    let mut d = detail(EVENT1);
    // More than 10,00,00,00,000 rupees on one page: the words cannot be written.
    set_net(&mut d, 10_000_000_001);
    let error = build(&d, &options(PrintDoc::SellerBill, Language::En)).unwrap_err();
    assert_eq!(error.code(), mandi_core::error::ErrorCode::ErrOverflow);
}
