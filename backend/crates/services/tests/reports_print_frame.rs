//! The report and analytics prints on the shared header and footer, the tag after party names, and
//! English digits by default (plan d5-late round 14 with D4R 17e; PRD FR-19.13, 10.2, FR-7.32,
//! NFR-URD-14).
//!
//! * Every report print and analytics print gives its title and facts to `reports_print_header`
//!   (`print_frame`); the header holds the firm, the title and exactly those facts, and no print
//!   data carries a header of its own.
//! * A tagged account (عاشق ظفر K) shows its tag in every report, analytics view, filter fact and
//!   dashboard list; the same name without a tag shows none.
//! * In both languages every number of a print is English digits (0 to 9). The Urdu digits setting
//!   is gone (plan 63); an old stored value of it is ignored.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::analytics::{
    AnalyticsFilters, AnalyticsPrintInput, AnalyticsRunInput, AnalyticsView, print_data as a_print,
    run as a_run,
};
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::print::{PrintFrameInput, print_frame};
use mandi_reports::reports::{
    Cell, PartyFilter, ReportFilters, ReportId, ReportPage, ReportPrintInput,
    buyer_bills_print_data, catalog, print_data,
};
use mandi_services::core::accounts::AccountType;
use mandi_services::core::auth::Language;
use reports_world::*;
use support::World;

/// The world after Events 1 to 5, plus a third بکری on 10-10: the seller منظور (tag D.P) sells 12
/// کیلا, 4 to عاشق ظفر (tag K) and 6 to another عاشق ظفر with no tag, both on credit at 500, and 2 to the cash buyer.
fn tagged_world() -> (World, i64, i64, i64) {
    let w = full_day();
    let seller = support::add_account_tagged(
        &w.env,
        &w.owner,
        AccountType::Seller,
        ("منظور", "Manzoor"),
        (true, ""),
        "D.P",
    );
    let with_tag = support::add_account_tagged(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        ("عاشق ظفر", "Ashiq Zafar"),
        (true, ""),
        "K",
    );
    let without = support::add_account_tagged(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        ("عاشق ظفر", "Ashiq Zafar"),
        (true, ""),
        "",
    );
    let lot = support::with_sales(
        support::bare_lot(&w, "lot-t", seller, w.banana, "12"),
        vec![
            support::sale("s-t1", with_tag, "4", "500"),
            support::sale("s-t2", without, "6", "500"),
            // The cash buyer with no name (serial 0): known by his full code, not a short code.
            support::sale("s-t3", w.naqdi, "2", "500"),
        ],
    );
    save(&w, &support::draft(vec![lot]));
    (w, with_tag, without, seller)
}

/// Every account a page shows, as (English name, tag).
fn parties(page: &ReportPage) -> Vec<(String, Option<String>)> {
    let mut found = Vec::new();
    for cells in page
        .rows
        .iter()
        .map(|r| &r.cells)
        .chain(std::iter::once(&page.totals))
    {
        for cell in cells {
            if let Cell::Party { party } = cell {
                found.push((party.name_en.clone(), party.tag.clone()));
            }
        }
    }
    found
}

fn ashiq(found: &[(String, Option<String>)]) -> Vec<Option<String>> {
    let mut tags: Vec<Option<String>> = found
        .iter()
        .filter(|(name, _)| name == "Ashiq Zafar")
        .map(|(_, tag)| tag.clone())
        .collect();
    tags.sort();
    tags
}

fn both_tags() -> Vec<Option<String>> {
    vec![None, Some("K".to_owned())]
}

fn day10() -> ReportFilters {
    ReportFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        ..ReportFilters::default()
    }
}

// ---- The tag -----------------------------------------------------------------------------------

#[test]
fn the_tag_shows_after_the_name_in_every_report_and_the_same_name_without_a_tag_shows_none() {
    let (w, with_tag, _, _) = tagged_world();
    // R5 (the buyer bills), R2/R4 are lines, R20 (dues), R18 (the trial balance) and R35 (the day
    // sheet, all parties) list the two buyers.
    for (id, filters) in [
        (ReportId::R5, day10()),
        (ReportId::R20, ReportFilters::default()),
        (ReportId::R18, ReportFilters::default()),
        (
            ReportId::R35,
            ReportFilters {
                as_of: "10-10-2026".into(),
                ..ReportFilters::default()
            },
        ),
    ] {
        let page = report(&w, id, filters, 0, 0);
        assert_eq!(ashiq(&parties(&page)), both_tags(), "{id:?}");
    }
    // The table of the day sheet: two people with the same name are told apart by their tag
    // (the one without a tag first), and the cash buyer shows his full code, not the serial 0.
    let table = report(
        &w,
        ReportId::R35,
        ReportFilters {
            as_of: "10-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    let order: Vec<Option<String>> = table
        .rows
        .iter()
        .flat_map(|r| r.cells.iter())
        .filter_map(|c| match c {
            Cell::Party { party } if party.name_en == "Ashiq Zafar" => Some(party.tag.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec![None, Some("K".to_owned())]);
    let naqdi = table
        .rows
        .iter()
        .flat_map(|r| r.cells.iter())
        .find_map(|c| match c {
            Cell::Party { party } if party.code == "03040000" => Some(party.clone()),
            _ => None,
        })
        .expect("the cash buyer has a row");
    assert_eq!(naqdi.short_code, None);
    // The slip of one party names him with his tag: in the filter echoed and in the header fact.
    let mut one = ReportFilters {
        party: Some(PartyFilter {
            account_id: with_tag,
            person_key: None,
        }),
        ..ReportFilters::default()
    };
    one.as_of = "10-10-2026".into();
    let slip = report(&w, ReportId::R35, one.clone(), 0, 0);
    assert_eq!(
        slip.filters.party.as_ref().and_then(|p| p.tag.clone()),
        Some("K".to_owned())
    );
    let printed = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            group_by_month: false,
            sort: None,
            id: ReportId::R35,
            filters: one,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    let fact = printed
        .filters
        .iter()
        .find(|f| f.label_id == "report-filter-party")
        .unwrap();
    assert!(fact.value.ends_with(" K"), "{}", fact.value);
    assert!(fact.value.contains("Ashiq Zafar"), "{}", fact.value);
    // The ledger of the tagged buyer names the party with the tag too.
    let ledger = report(
        &w,
        ReportId::R13,
        ReportFilters {
            party: Some(PartyFilter {
                account_id: with_tag,
                person_key: None,
            }),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        ledger.filters.party.as_ref().and_then(|p| p.tag.clone()),
        Some("K".to_owned())
    );
}

#[test]
fn the_tag_shows_in_the_analytics_views_the_buyer_bills_slips_and_the_dashboard() {
    let (w, _, _, _) = tagged_world();
    // Analytics: the buyers (view C) and the sellers (view B) with their tags.
    let c = a_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: AnalyticsView::C,
            filters: AnalyticsFilters::default(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let mut tags: Vec<Option<String>> = c
        .rows
        .iter()
        .filter_map(|r| match r.cells.first() {
            Some(Cell::Party { party }) if party.name_en == "Ashiq Zafar" => {
                Some(party.tag.clone())
            }
            _ => None,
        })
        .collect();
    tags.sort();
    assert_eq!(tags, both_tags());
    let b = a_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: AnalyticsView::B,
            filters: AnalyticsFilters::default(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert!(b.rows.iter().any(|r| matches!(
        r.cells.first(),
        Some(Cell::Party { party }) if party.name_en == "Manzoor" && party.tag.as_deref() == Some("D.P")
    )));
    // The slips of "Print All" (R5).
    let slips = buyer_bills_print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            group_by_month: false,
            sort: None,
            id: ReportId::R5,
            filters: day10(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    let mut slip_tags: Vec<Option<String>> = slips
        .slips
        .iter()
        .filter(|s| s.party.name == "Ashiq Zafar")
        .map(|s| s.party.tag.clone())
        .collect();
    slip_tags.sort();
    assert_eq!(slip_tags, both_tags());
    // The dashboard: the oldest dues name both buyers with their tags, and today's بکری names the
    // seller of the third بکری with his tag.
    let dash = dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap();
    let mut due_tags: Vec<Option<String>> = dash
        .oldest_dues
        .iter()
        .filter(|d| d.party.name_en == "Ashiq Zafar")
        .map(|d| d.party.tag.clone())
        .collect();
    due_tags.sort();
    assert_eq!(due_tags, both_tags());
    let seller_row = dash
        .bikri_today
        .iter()
        .find(|b| b.seller.as_ref().is_some_and(|s| s.en == "Manzoor"))
        .unwrap();
    assert_eq!(seller_row.seller_tag.as_deref(), Some("D.P"));
}

// ---- The shared header and footer --------------------------------------------------------------

fn minimal_filters(w: &World, id: ReportId, party_required: bool) -> ReportFilters {
    let mut filters = ReportFilters::default();
    if id == ReportId::R1 {
        filters.bikri_id = Some(
            w.env
                .ctx
                .db()
                .read(|conn| {
                    conn.query_row("SELECT id FROM bikri_entries WHERE number = 1", [], |r| {
                        r.get(0)
                    })
                })
                .unwrap(),
        );
    } else if party_required {
        filters.party = Some(PartyFilter {
            account_id: w.akram,
            person_key: None,
        });
    }
    filters
}

#[test]
fn every_report_and_analytics_print_gets_its_header_and_footer_from_the_one_command() {
    let (w, _, _, _) = tagged_world();
    for language in [Language::Ur, Language::En] {
        for entry in catalog() {
            // R31 comes from the services: the app builds its page (reports_round7).
            if entry.id == ReportId::R31 {
                continue;
            }
            let data = print_data(
                w.env.ctx.db(),
                &ReportPrintInput {
                    group_by_month: false,
                    sort: None,
                    id: entry.id,
                    filters: minimal_filters(&w, entry.id, entry.party_required),
                    language,
                    paper: None,
                },
            )
            .unwrap_or_else(|e| panic!("{:?}: {e:?}", entry.id));
            let frame = print_frame(
                w.env.ctx.db(),
                &PrintFrameInput {
                    language,
                    title_id: data.title_id.clone(),
                    facts: data.filters.clone(),
                },
            )
            .unwrap();
            assert_eq!(frame.header.title_id, data.title_id, "{:?}", entry.id);
            assert_eq!(frame.header.facts, data.filters, "{:?}", entry.id);
            assert_eq!(frame.language, language);
            assert_eq!(frame.dir, data.dir);
            assert_eq!(
                frame.header.firm.name,
                match language {
                    Language::Ur => "ماہر عدنان اینڈ برادرز",
                    Language::En => "Maher Adnan and Brothers",
                }
            );
        }
        for view in [
            AnalyticsView::A,
            AnalyticsView::B,
            AnalyticsView::C,
            AnalyticsView::D,
            AnalyticsView::E,
            AnalyticsView::F,
            AnalyticsView::G,
            AnalyticsView::H,
            AnalyticsView::I,
            AnalyticsView::J,
            AnalyticsView::K,
        ] {
            let data = a_print(
                w.env.ctx.db(),
                &AnalyticsPrintInput {
                    sort: None,
                    view,
                    filters: AnalyticsFilters {
                        from: "10-10-2026".into(),
                        to: "10-10-2026".into(),
                        ..AnalyticsFilters::default()
                    },
                    language,
                    paper: None,
                },
            )
            .unwrap_or_else(|e| panic!("{view:?}: {e:?}"));
            let frame = print_frame(
                w.env.ctx.db(),
                &PrintFrameInput {
                    language,
                    title_id: data.title_id.clone(),
                    facts: data.filters.clone(),
                },
            )
            .unwrap();
            assert_eq!(frame.header.title_id, data.title_id, "{view:?}");
            assert_eq!(frame.header.facts, data.filters, "{view:?}");
        }
    }
    // The footer carries the print time as DD-MM-YYYY HH:MM, in English digits.
    let frame = print_frame(
        w.env.ctx.db(),
        &PrintFrameInput {
            language: Language::Ur,
            title_id: "report-r18-title".into(),
            facts: Vec::new(),
        },
    )
    .unwrap();
    assert_eq!(frame.footer.printed_at, "10-10-2026 14:00");
}

// ---- English digits ----------------------------------------------------------------------------

/// True for an Arabic-Indic digit (U+0660 to U+0669) or an Eastern Arabic-Indic one (U+06F0 to
/// U+06F9).
fn has_urdu_digit(text: &str) -> bool {
    text.chars()
        .any(|c| ('\u{0660}'..='\u{0669}').contains(&c) || ('\u{06F0}'..='\u{06F9}').contains(&c))
}

#[test]
fn a_print_has_english_digits_in_both_languages_and_an_old_urdu_digits_setting_changes_nothing() {
    let (w, _, _, _) = tagged_world();
    let report_json = |w: &World, id: ReportId, language: Language| -> String {
        let entry = catalog().into_iter().find(|e| e.id == id).unwrap();
        let data = print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                group_by_month: false,
                sort: None,
                id,
                filters: minimal_filters(w, id, entry.party_required),
                language,
                paper: None,
            },
        )
        .unwrap();
        serde_json::to_string(&data).unwrap()
    };
    // Every report, every analytics view and the slips, in both languages, with the setting off.
    for language in [Language::Ur, Language::En] {
        for entry in catalog() {
            if entry.id == ReportId::R31 {
                continue;
            }
            let json = report_json(&w, entry.id, language);
            assert!(!has_urdu_digit(&json), "{:?} {language:?}", entry.id);
        }
        for view in [
            AnalyticsView::A,
            AnalyticsView::D,
            AnalyticsView::F,
            AnalyticsView::I,
            AnalyticsView::J,
            AnalyticsView::K,
        ] {
            let data = a_print(
                w.env.ctx.db(),
                &AnalyticsPrintInput {
                    sort: None,
                    view,
                    filters: AnalyticsFilters::default(),
                    language,
                    paper: None,
                },
            )
            .unwrap();
            assert!(
                !has_urdu_digit(&serde_json::to_string(&data).unwrap()),
                "{view:?} {language:?}"
            );
        }
        let slips = buyer_bills_print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                group_by_month: false,
                sort: None,
                id: ReportId::R5,
                filters: day10(),
                language,
                paper: None,
            },
        )
        .unwrap();
        assert!(!has_urdu_digit(&serde_json::to_string(&slips).unwrap()));
    }

    // Plan 63: the Urdu digits setting is gone. A computer that still holds the old stored value
    // `urdu_digits: true` prints exactly as before: 0 to 9, in both languages, for each print type.
    common::raw(&w.dir)
        .execute(
            "INSERT INTO settings (key, value_json) VALUES ('printing', ?1)
             ON CONFLICT (key) DO UPDATE SET value_json = excluded.value_json",
            [r#"{"copies":1,"paper_seller_bill":"a4","urdu_digits":true}"#],
        )
        .unwrap();
    for language in [Language::Ur, Language::En] {
        assert!(!has_urdu_digit(&report_json(&w, ReportId::R18, language)));
    }
    let analytics = |language| {
        serde_json::to_string(
            &a_print(
                w.env.ctx.db(),
                &AnalyticsPrintInput {
                    sort: None,
                    view: AnalyticsView::A,
                    filters: AnalyticsFilters::default(),
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        )
        .unwrap()
    };
    assert!(!has_urdu_digit(&analytics(Language::Ur)));
    assert!(!has_urdu_digit(&analytics(Language::En)));
    let slips = |language| {
        serde_json::to_string(
            &buyer_bills_print_data(
                w.env.ctx.db(),
                &ReportPrintInput {
                    group_by_month: false,
                    sort: None,
                    id: ReportId::R5,
                    filters: day10(),
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        )
        .unwrap()
    };
    assert!(!has_urdu_digit(&slips(Language::Ur)));
    assert!(!has_urdu_digit(&slips(Language::En)));
}

// ---- Phones in the dash form, and a چکتا buyer's phone in his party cell --------------------

/// Two چکتا buyers with the same name and different phones, and a third, all paid in full.
fn two_shahids() -> World {
    use mandi_services::core::bikri::ChuktaInput;
    let w = worked_day();
    let line = |key: &str, qty: &str, ur: &str, en: &str, phone: &str| {
        let mut l = support::sale(key, w.chukta_khata, qty, "200");
        l.chukta = Some(ChuktaInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.into(),
        });
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "17");
    lot.sales = vec![
        line("s-1", "5", "شاہد", "Shahid", "03000000000"),
        line("s-2", "2", "شاہد", "Shahid", "0301-2222222"),
        line("s-3", "10", "ناصر", "Nasir", "0300-1111111"),
    ];
    save(&w, &support::draft(vec![lot]));
    w
}

fn person_phones(page: &ReportPage, name: &str) -> Vec<Option<String>> {
    let mut phones: Vec<Option<String>> = page
        .rows
        .iter()
        .flat_map(|r| r.cells.iter())
        .filter_map(|c| match c {
            Cell::Party { party } if party.name_en == name => Some(party.phone.clone()),
            _ => None,
        })
        .collect();
    phones.sort();
    phones
}

#[test]
fn a_chukta_buyer_shows_his_phone_in_the_dash_form_in_his_party_cell_and_in_the_day_sheet() {
    let w = two_shahids();
    let wanted = vec![
        Some("0300-0000000".to_owned()),
        Some("0301-2222222".to_owned()),
    ];
    // R35 (the day sheet): two people with the same name are two rows told apart by the phone,
    // whatever way the phone was typed.
    let filters = ReportFilters {
        as_of: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    let table = report(&w, ReportId::R35, filters.clone(), 0, 0);
    assert_eq!(person_phones(&table, "Shahid"), wanted);
    // They paid in full, so nobody owes: R20 has no row for any of them (PRD FR-13.6).
    let r20 = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(r20.total_count, 0);
    // The print: the party of the row carries the phone. Digits are 0-9 in an Urdu print too.
    let data = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            group_by_month: false,
            sort: None,
            id: ReportId::R35,
            filters,
            language: Language::Ur,
            paper: None,
        },
    )
    .unwrap();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("0300-0000000") && json.contains("0301-2222222"));
    assert!(!has_urdu_digit(&json));
}

/// Plan 75 Section 7.9: the print header reads the firm from the settings at each print. After the
/// Owner changes the firm name, the very next header carries the new names in both languages, and
/// the mandi name changes with it. No copy of the firm is kept in the app, a cache or the build.
#[test]
fn the_next_print_header_shows_the_firm_the_owner_saved_last() {
    let w = worked_day();
    let header_firm = |language: Language| {
        print_frame(
            w.env.ctx.db(),
            &PrintFrameInput {
                language,
                title_id: "report-r11-title".to_owned(),
                facts: Vec::new(),
            },
        )
        .unwrap()
        .header
        .firm
    };
    // The firm of the worked day, as the wizard saved it.
    let before = (header_firm(Language::Ur), header_firm(Language::En));
    assert_eq!(before.0.name, "ماہر عدنان اینڈ برادرز");
    assert_eq!(before.1.name, "Maher Adnan and Brothers");

    let mut firm = mandi_services::settings::get(&w.env.ctx).unwrap().firm;
    firm.name_ur = "ماہر عدنان اینڈ سنز".to_owned();
    firm.name_en = "Maher Adnan and Sons".to_owned();
    firm.mandi_ur = "نئی سبزی منڈی لاہور".to_owned();
    firm.mandi_en = "New Sabzi Mandi Lahore".to_owned();
    // Words that come only from the spelling rules are confirmed, as the Settings screen does
    // (PRD FR-7.66).
    let confirmed: Vec<String> = ["سنز", "Sons", "لاہور", "Lahore"]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    mandi_services::settings::set_firm_confirmed(&w.env.ctx, &w.owner, &firm, &confirmed).unwrap();

    let (ur, en) = (header_firm(Language::Ur), header_firm(Language::En));
    assert_eq!(ur.name, "ماہر عدنان اینڈ سنز");
    assert_eq!(en.name, "Maher Adnan and Sons");
    assert_eq!(ur.mandi, "نئی سبزی منڈی لاہور");
    assert_eq!(en.mandi, "New Sabzi Mandi Lahore");

    // And again: a second change shows at once too.
    firm.name_en = "Maher Adnan Traders".to_owned();
    firm.name_ur = "ماہر عدنان ٹریڈرز".to_owned();
    let confirmed: Vec<String> = ["ٹریڈرز", "Traders"]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    mandi_services::settings::set_firm_confirmed(&w.env.ctx, &w.owner, &firm, &confirmed).unwrap();
    assert_eq!(header_firm(Language::En).name, "Maher Adnan Traders");
    assert_eq!(header_firm(Language::Ur).name, "ماہر عدنان ٹریڈرز");
}
