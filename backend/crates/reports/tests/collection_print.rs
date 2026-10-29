//! Print data of the collection list (اگراہی فہرست, PRD FR-15.6, FR-19.13; plan 17e, D4R-A11 and
//! D4R-A12). The input is a `CollectionList` as the screen reads it. The worked numbers are the
//! buyers of the PRD worked day after Events 1 to 5: اکرم 2,880 and ارشد 18,684, together 21,564.
//!
//! The snapshots (`insta`) hold the whole print data in both languages, with Urdu digits and with
//! the firm's contacts and trade mark. The tests around them check the numbers by hand, and that a
//! list that does not add up is not printed.
#![cfg(test)]

use chrono::{TimeZone, Utc};
use mandi_core::accounts::RecordRef;
use mandi_core::auth::Language;
use mandi_core::firm::FirmContact;
use mandi_core::money::{Grouping, Paisa};
use mandi_core::printing::{CollectionFilterNames, Paper, PrintDoc, PrintSetup};
use mandi_core::vouchers::{CollectionBuyer, CollectionGroup, CollectionList};
use mandi_reports::ReportsError;
use mandi_reports::print::{
    CollectionBuildOptions, CollectionPrintData, FirmNames, TextDir, build_collection,
};

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn record(id: i64, code: &str, ur: &str, en: &str, tag: Option<&str>) -> RecordRef {
    RecordRef {
        id,
        code: code.into(),
        name_ur: ur.into(),
        name_en: en.into(),
        tag: tag.map(str::to_owned),
    }
}

fn buyer(
    account: RecordRef,
    phone: &str,
    dues: i64,
    oldest: Option<(&str, i64)>,
) -> CollectionBuyer {
    CollectionBuyer {
        account,
        phone: phone.into(),
        dues_paisa: rupees(dues),
        oldest_bill_date: oldest.map(|(day, _)| day.to_owned()),
        oldest_days: oldest.map(|(_, days)| days),
    }
}

/// After Events 1 to 5: اکرم 2,880 and ارشد 18,684 (شاہد, a one-time buyer, owes nothing and is not
/// on the list). No collector and no city.
fn golden_list() -> CollectionList {
    let buyers = vec![
        buyer(
            record(1, "0020", "اکرم ٹول ٹیکس", "Akram Toll Tax", None),
            "03001234567",
            2_880,
            Some(("2026-10-10", 0)),
        ),
        buyer(
            record(2, "0021", "ارشد ملتانی", "Arshad Multani", None),
            "",
            18_684,
            Some(("2026-10-10", 0)),
        ),
    ];
    CollectionList {
        group_totals: Vec::new(),
        date: "2026-10-10".into(),
        weekday: 5,
        total_count: 2,
        offset: 0,
        limit: 100,
        groups: vec![CollectionGroup {
            collector: None,
            city: None,
            buyers,
            total_paisa: rupees(21_564),
        }],
        total_paisa: rupees(21_564),
    }
}

/// Two groups: a collector in a city, and the buyers with neither. One buyer has a tag.
fn grouped_list() -> CollectionList {
    let group_one = vec![
        buyer(
            record(3, "0022", "عاشق ظفر", "Ashiq Zafar", Some("K")),
            "03211234567",
            5_000,
            Some(("2026-09-30", 10)),
        ),
        buyer(
            record(4, "00212", "طارق محمود", "Tariq Mehmood", None),
            "03001112223",
            1_200,
            None,
        ),
    ];
    let group_two = vec![buyer(
        record(5, "0027", "ناصر", "Nasir", None),
        "",
        800,
        Some(("2026-10-01", 9)),
    )];
    CollectionList {
        group_totals: Vec::new(),
        date: "2026-10-10".into(),
        weekday: 5,
        total_count: 3,
        offset: 0,
        limit: 100,
        groups: vec![
            CollectionGroup {
                collector: None,
                city: None,
                buyers: group_two,
                total_paisa: rupees(800),
            },
            CollectionGroup {
                collector: Some(record(0, "", "آصف", "Asif", None)),
                city: Some(record(9, "01", "ملتان", "Multan", None)),
                buyers: group_one,
                total_paisa: rupees(6_200),
            },
        ],
        total_paisa: rupees(7_000),
    }
}

fn firm() -> FirmNames {
    FirmNames {
        name_ur: "ماہر عدنان اینڈ برادرز".into(),
        name_en: "Maher Adnan and Brothers".into(),
        mandi_ur: "نیو سبزی منڈی کوٹ رادھا کشن".into(),
        mandi_en: "New Sabzi Mandi, Kot Radha Kishan".into(),
        address: "Kot Radha Kishan".into(),
        phone: "03001234567".into(),
        contacts: Vec::new(),
        trade_mark: String::new(),
    }
}

fn options(language: Language) -> CollectionBuildOptions {
    CollectionBuildOptions {
        language,
        grouping: Grouping::International,
        setup: PrintSetup::default(),
        firm: firm(),
        // 09:00 UTC on 10-10-2026 is 14:00 in Pakistan.
        now_utc: Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap(),
        filters: CollectionFilterNames::default(),
    }
}

fn print(list: &CollectionList, language: Language) -> CollectionPrintData {
    build_collection(list, &options(language)).unwrap()
}

fn data_error(list: &CollectionList) -> String {
    match build_collection(list, &options(Language::En)) {
        Err(ReportsError::Data(message)) => message,
        other => panic!("not a data error: {other:?}"),
    }
}

// ---- Snapshots ----

#[test]
fn snapshot_golden_urdu() {
    insta::assert_json_snapshot!(print(&golden_list(), Language::Ur));
}

#[test]
fn snapshot_golden_english() {
    insta::assert_json_snapshot!(print(&golden_list(), Language::En));
}

#[test]
fn snapshot_golden_urdu_with_urdu_digits() {
    let mut set = options(Language::Ur);
    set.setup.urdu_digits = true;
    insta::assert_json_snapshot!(build_collection(&golden_list(), &set).unwrap());
}

#[test]
fn snapshot_grouped_with_contacts_trade_mark_and_filters_urdu() {
    let mut set = options(Language::Ur);
    set.firm.contacts = vec![
        FirmContact {
            name_ur: "منشی آصف بھٹی".into(),
            name_en: "Munshi Asif Bhatti".into(),
            phone: "03004058860".into(),
        },
        FirmContact {
            name_ur: "اکرم".into(),
            name_en: "Akram".into(),
            phone: "03211234567".into(),
        },
    ];
    set.firm.trade_mark = "RH".into();
    set.filters = CollectionFilterNames {
        collector: Some(("آصف".into(), "Asif".into())),
        city: Some(("ملتان".into(), "Multan".into())),
    };
    insta::assert_json_snapshot!(build_collection(&grouped_list(), &set).unwrap());
}

#[test]
fn snapshot_grouped_english() {
    insta::assert_json_snapshot!(print(&grouped_list(), Language::En));
}

// ---- The numbers, by hand ----

#[test]
fn the_golden_list_prints_2880_18684_and_21564() {
    for language in [Language::Ur, Language::En] {
        let data = print(&golden_list(), language);
        assert_eq!(data.doc, PrintDoc::CollectionList);
        assert_eq!(data.paper, Paper::A4);
        let rows = &data.groups[0].rows;
        assert_eq!(
            rows.iter().map(|r| r.dues.as_str()).collect::<Vec<_>>(),
            vec!["2,880", "18,684"]
        );
        assert_eq!(data.groups[0].total, "21,564");
        assert_eq!(data.total.amount, "21,564");
        assert_eq!(data.total.label_id, "print-collection-total");
        assert_eq!(data.buyer_count, "2");
        // The short code (PRD FR-7.39) and the row numbers.
        assert_eq!(
            rows.iter().map(|r| r.code.as_str()).collect::<Vec<_>>(),
            vec!["0", "1"]
        );
        assert_eq!(
            rows.iter().map(|r| r.serial.as_str()).collect::<Vec<_>>(),
            vec!["1", "2"]
        );
        assert_eq!(rows[0].oldest_bill_date.as_deref(), Some("10-10-2026"));
        assert_eq!(rows[0].oldest_days.as_deref(), Some("0"));
        assert_eq!(rows[0].phone, "0300-1234567");
    }
    let ur = print(&golden_list(), Language::Ur);
    assert_eq!(ur.dir, TextDir::Rtl);
    assert_eq!(ur.groups[0].rows[0].name, "اکرم ٹول ٹیکس");
    assert_eq!(
        print(&golden_list(), Language::En).groups[0].rows[0].name,
        "Akram Toll Tax"
    );
}

#[test]
fn the_header_and_footer_are_the_shared_ones() {
    let data = print(&golden_list(), Language::En);
    assert_eq!(data.header.title_id, "print-collection-list-title");
    assert_eq!(data.header.firm.name, "Maher Adnan and Brothers");
    assert!(data.header.contacts.is_empty());
    assert_eq!(data.header.trade_mark, None, "no empty box");
    assert_eq!(data.header.facts.len(), 1, "only the date: no filter");
    assert_eq!(data.header.facts[0].label_id, "print-fact-date");
    assert_eq!(data.header.facts[0].value, "10-10-2026");
    assert_eq!(data.footer.printed_at, "10-10-2026 14:00");
    assert_eq!(data.footer.page_label_id, "print-page-label");
}

#[test]
fn groups_keep_their_order_and_totals_and_the_rows_are_numbered_through_the_list() {
    let data = print(&grouped_list(), Language::En);
    assert_eq!(data.groups.len(), 2);
    assert_eq!(
        (
            data.groups[0].collector.clone(),
            data.groups[0].city.clone()
        ),
        (None, None)
    );
    assert_eq!(
        (
            data.groups[1].collector.as_deref(),
            data.groups[1].city.as_deref()
        ),
        (Some("Asif"), Some("Multan"))
    );
    assert_eq!(data.groups[0].total, "800");
    assert_eq!(data.groups[1].total, "6,200");
    assert_eq!(data.total.amount, "7,000");
    assert_eq!(data.buyer_count, "3");
    let serials: Vec<_> = data
        .groups
        .iter()
        .flat_map(|g| g.rows.iter().map(|r| r.serial.as_str()))
        .collect();
    assert_eq!(serials, vec!["1", "2", "3"]);
    // Short codes: the running number of the code (00212 is 12).
    assert_eq!(data.groups[1].rows[1].code, "12");
    // The tag is its own field; the name does not hold it. No tag means no key in the JSON.
    let tagged = &data.groups[1].rows[0];
    assert_eq!(
        (tagged.name.as_str(), tagged.tag.as_deref()),
        ("Ashiq Zafar", Some("K"))
    );
    let json = serde_json::to_value(&data).unwrap();
    assert_eq!(json["groups"][1]["rows"][0]["tag"], "K");
    assert!(json["groups"][1]["rows"][1].get("tag").is_none());
    // A buyer with no open bill has no date and no days.
    assert_eq!(data.groups[1].rows[1].oldest_bill_date, None);
    assert_eq!(data.groups[1].rows[1].oldest_days, None);
}

#[test]
fn the_filters_show_in_the_facts_in_the_print_language() {
    let mut set = options(Language::En);
    set.filters = CollectionFilterNames {
        collector: Some(("آصف".into(), "Asif".into())),
        city: Some(("ملتان".into(), "Multan".into())),
    };
    let data = build_collection(&grouped_list(), &set).unwrap();
    let facts: Vec<_> = data
        .header
        .facts
        .iter()
        .map(|f| (f.label_id.as_str(), f.value.as_str()))
        .collect();
    assert_eq!(
        facts,
        vec![
            ("print-fact-date", "10-10-2026"),
            ("print-fact-collector", "Asif"),
            ("print-fact-city", "Multan"),
        ]
    );
    set.language = Language::Ur;
    let ur = build_collection(&grouped_list(), &set).unwrap();
    assert_eq!(ur.header.facts[1].value, "آصف");
}

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
fn a_default_urdu_collection_print_has_only_plain_digits_everywhere() {
    // Amounts, dates, phones, short codes, serials and counts: 0-9 (PRD FR-7.32, NFR-URD-14).
    for list in [golden_list(), grouped_list()] {
        for language in [Language::Ur, Language::En] {
            let data = print(&list, language);
            assert!(!data.urdu_digits);
            let value = serde_json::to_value(&data).unwrap();
            assert!(!has_urdu_digit(&value), "{language:?}: {value}");
        }
    }
    // The setting on is the one case: then the numbers are Urdu digits, the phones and codes not
    // (a phone is plain digits and the dash of `03XX-XXXXXXX`).
    let mut set = options(Language::Ur);
    set.setup.urdu_digits = true;
    let on = serde_json::to_value(build_collection(&golden_list(), &set).unwrap()).unwrap();
    assert!(has_urdu_digit(&on));
    assert_eq!(on["groups"][0]["rows"][0]["phone"], "0300-1234567");
    assert_eq!(on["groups"][0]["rows"][0]["code"], "0");
}

#[test]
fn urdu_digits_only_on_an_urdu_print_with_the_setting_on() {
    let mut set = options(Language::Ur);
    set.setup.urdu_digits = true;
    let ur = build_collection(&golden_list(), &set).unwrap();
    assert!(ur.urdu_digits);
    assert_eq!(ur.total.amount, "۲۱,۵۶۴");
    set.language = Language::En;
    let en = build_collection(&golden_list(), &set).unwrap();
    assert!(!en.urdu_digits);
    assert_eq!(en.total.amount, "21,564");
}

// ---- A list that does not add up is not printed (money agent, plan 17e) ----

#[test]
fn a_group_total_that_is_off_by_100_paisa_is_not_printed() {
    let mut list = golden_list();
    // One rupee is 100 paisa: the group total says 21,565 while the buyers add up to 21,564.
    list.groups[0].total_paisa = rupees(21_565);
    list.total_paisa = rupees(21_565);
    assert!(data_error(&list).contains("do not add up to its total"));
}

#[test]
fn a_list_total_that_is_off_is_not_printed() {
    let mut list = golden_list();
    list.total_paisa = rupees(21_565);
    assert!(data_error(&list).contains("groups of the collection list"));
    // A second group changes nothing about the rule: the groups must add up.
    let mut grouped = grouped_list();
    grouped.total_paisa = rupees(7_100);
    assert!(data_error(&grouped).contains("groups of the collection list"));
}

#[test]
fn a_buyer_who_owes_nothing_is_not_printed() {
    let mut list = golden_list();
    list.groups[0].buyers[0].dues_paisa = Paisa::ZERO;
    list.groups[0].total_paisa = rupees(18_684);
    list.total_paisa = rupees(18_684);
    assert!(data_error(&list).contains("owes nothing"));
}

#[test]
fn a_buyer_with_negative_dues_is_not_printed() {
    // The list holds debit balances only, so a negative amount is a data error, never a minus sign.
    let mut list = golden_list();
    list.groups[0].buyers[0].dues_paisa = rupees(-1);
    list.groups[0].total_paisa = rupees(18_683);
    list.total_paisa = rupees(18_683);
    assert!(data_error(&list).contains("owes nothing"));
}

#[test]
fn an_empty_list_has_no_groups_and_a_zero_total() {
    let list = CollectionList {
        group_totals: Vec::new(),
        date: "2026-10-10".into(),
        weekday: 5,
        total_count: 0,
        offset: 0,
        limit: 100,
        groups: Vec::new(),
        total_paisa: Paisa::ZERO,
    };
    let data = print(&list, Language::En);
    assert!(data.groups.is_empty());
    assert_eq!(data.buyer_count, "0");
    assert_eq!(data.total.amount, "0");
}
