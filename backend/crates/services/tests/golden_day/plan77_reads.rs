//! Plan 77 on the PRD worked day: the commission page for one party and for a period (PRD FR-9.3
//! v1.42), and the analytics views L (buyer by item) and M (expenses by head) (PRD 18.2, FR-18.9).
//! Every expected number is the PRD's (4.4, 4.5, 4.9, 18.4) and is never changed to suit the code.
use super::day_money::{event5, events_1_to_4, expense_head, shop_input};
use super::*;
use mandi_reports::analytics::{
    AnalyticsFilters, AnalyticsGroupBy, AnalyticsKey, AnalyticsLinesInput, AnalyticsPage,
    AnalyticsRunInput, AnalyticsView, lines as analytics_lines, run as analytics_run,
    screen_catalog,
};
use mandi_reports::dashboard::{
    self, CommissionDayInput, CommissionMonthInput, CommissionParty, CommissionPeriod,
    CommissionPeriodKind, CommissionPrintInput, CommissionSide,
};
use mandi_reports::reports::{Cell, PartyFilter};
use mandi_services::core::auth::Language;

fn rupees_of(p: Paisa) -> i64 {
    p.as_paisa() / 100
}

fn party(side: CommissionSide, account_id: i64) -> Option<CommissionParty> {
    Some(CommissionParty {
        side,
        account_id,
        chukta_person_key: None,
    })
}

fn month_for(
    w: &support::World,
    party: Option<CommissionParty>,
    period: Option<CommissionPeriod>,
) -> dashboard::CommissionMonth {
    dashboard::commission_month(
        w.env.ctx.db(),
        &CommissionMonthInput {
            month: None,
            period,
            party,
            goods: None,
            offset: None,
            limit: None,
        },
    )
    .unwrap()
}

fn refused(
    w: &support::World,
    party: Option<CommissionParty>,
    period: Option<CommissionPeriod>,
) -> Vec<(ErrorCode, String)> {
    match dashboard::commission_month(
        w.env.ctx.db(),
        &CommissionMonthInput {
            month: None,
            period,
            party,
            goods: None,
            offset: None,
            limit: None,
        },
    ) {
        Err(mandi_reports::ReportsError::Rejected(found)) => {
            found.into_iter().map(|e| (e.code, e.field)).collect()
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// The figures of the one row of a party page: sale, seller commission, buyer commission, لاگا,
/// total, goods commission.
fn figures(row: &dashboard::CommissionDayRow) -> (i64, i64, i64, i64, i64, Option<i64>) {
    (
        rupees_of(row.sale_paisa.unwrap()),
        rupees_of(row.seller_comm_paisa),
        rupees_of(row.buyer_comm_paisa),
        rupees_of(row.laga_paisa),
        rupees_of(row.total_comm_paisa),
        row.goods_comm_paisa.map(rupees_of),
    )
}

/// The key of the چکتا buyer شاہد (his names and phone, PRD FR-13.2).
fn shahid_key(w: &support::World) -> String {
    raw(&w.dir)
        .query_row(
            "SELECT chukta_person_key FROM analytics_facts WHERE chukta_person_key IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn a_seller_page_shows_his_sale_and_the_commission_on_his_goods() {
    let w = worked_day();
    events_1_to_4(&w);
    // رحیم, 10-10-26: sale 29,400; commission charged to him 1,764; on his goods 3,928.
    let rahim = month_for(&w, party(CommissionSide::Seller, w.rahim), None);
    assert_eq!(rahim.rows.len(), 1);
    assert_eq!(rahim.rows[0].date, "2026-10-10");
    assert_eq!(
        figures(&rahim.rows[0]),
        (29_400, 1_764, 1_764, 400, 3_928, Some(3_928))
    );
    assert_eq!(figures(&rahim.totals), figures(&rahim.rows[0]));
    // A party page has no partner columns and no shared income.
    assert!(rahim.partners.is_empty());
    assert!(rahim.rows[0].partner_shares.is_none());
    assert_eq!(rupees_of(rahim.rows[0].shared_income_paisa), 0);
    assert_eq!(rahim.party.as_ref().unwrap().party.code, "0010");
    // کریم: sale 12,000; charged to him 720; on his goods 1,520.
    let karim = month_for(&w, party(CommissionSide::Seller, w.karim), None);
    assert_eq!(
        figures(&karim.rows[0]),
        (12_000, 720, 720, 80, 1_520, Some(1_520))
    );
    // Both sellers add up to the day's total commission of the whole shop: 3,928 + 1,520 = 5,448.
    let shop = month_for(&w, None, None);
    assert_eq!(rupees_of(shop.rows[0].total_comm_paisa), 5_448);
    assert_eq!(
        rupees_of(rahim.totals.total_comm_paisa) + rupees_of(karim.totals.total_comm_paisa),
        5_448
    );
    // The page without a party is today's page, unchanged (FR-9.3: 5,448; shared income 5,488).
    assert!(shop.party.is_none() && shop.rows[0].sale_paisa.is_none());
    assert_eq!(rupees_of(shop.rows[0].shared_income_paisa), 5_488);
    assert_eq!(shop.partners.len(), 3);
}

#[test]
fn a_buyer_page_shows_his_purchase_and_his_charges() {
    let w = worked_day();
    events_1_to_4(&w);
    let akram = month_for(&w, party(CommissionSide::Buyer, w.akram), None);
    // اکرم: value 12,000; buyer commission 720; لاگا 160; total 880.
    assert_eq!(akram.rows.len(), 1);
    assert_eq!(figures(&akram.rows[0]), (12_000, 0, 720, 160, 880, None));
    let arshad = month_for(&w, party(CommissionSide::Buyer, w.arshad), None);
    assert_eq!(
        figures(&arshad.rows[0]),
        (17_400, 0, 1_044, 240, 1_284, None)
    );
    // شاہد, a چکتا buyer: 720 + 80 = 800.
    let shahid = CommissionParty {
        side: CommissionSide::Buyer,
        account_id: w.chukta_khata,
        chukta_person_key: Some(shahid_key(&w)),
    };
    let shahid = month_for(&w, Some(shahid), None);
    assert_eq!(figures(&shahid.rows[0]), (12_000, 0, 720, 80, 800, None));
    assert_eq!(shahid.party.as_ref().unwrap().party.name_en, "Shahid");
    // The whole چکتا کھاتہ account is the same person here.
    let khata = month_for(&w, party(CommissionSide::Buyer, w.chukta_khata), None);
    assert_eq!(figures(&khata.rows[0]), figures(&shahid.rows[0]));
    // The three buyers add up to the day's buyer commission 2,484 + لاگا 480 = 2,964.
    assert_eq!(
        rupees_of(akram.totals.total_comm_paisa)
            + rupees_of(arshad.totals.total_comm_paisa)
            + rupees_of(shahid.totals.total_comm_paisa),
        2_964
    );
    // A buyer who bought nothing has an empty page, not an error.
    let naqdi = month_for(&w, party(CommissionSide::Buyer, w.naqdi), None);
    assert!(naqdi.rows.is_empty());
    assert_eq!(rupees_of(naqdi.totals.total_comm_paisa), 0);
}

#[test]
fn a_party_of_the_wrong_side_or_a_missing_one_is_refused() {
    let w = worked_day();
    events_1_to_4(&w);
    assert_eq!(
        refused(&w, party(CommissionSide::Seller, w.akram), None),
        vec![(ErrorCode::ErrWrongPartyType, "party.account_id".to_owned())]
    );
    assert_eq!(
        refused(&w, party(CommissionSide::Buyer, w.rahim), None),
        vec![(ErrorCode::ErrWrongPartyType, "party.account_id".to_owned())]
    );
    assert_eq!(
        refused(&w, party(CommissionSide::Buyer, 99_999), None),
        vec![(ErrorCode::ErrNotFound, "party.account_id".to_owned())]
    );
    // A person key belongs to the چکتا کھاتہ account only.
    let keyed = CommissionParty {
        side: CommissionSide::Buyer,
        account_id: w.akram,
        chukta_person_key: Some("x|0300".into()),
    };
    assert_eq!(
        refused(&w, Some(keyed), None),
        vec![(
            ErrorCode::ErrOutOfRange,
            "party.chukta_person_key".to_owned()
        )]
    );
    // The old closed account 0031 is not a seller to read.
    let old = id_of_code(&w, "0031");
    assert_eq!(
        refused(&w, party(CommissionSide::Seller, old), None),
        vec![(ErrorCode::ErrWrongPartyType, "party.account_id".to_owned())]
    );
}

fn id_of_code(w: &support::World, code: &str) -> i64 {
    support::id_of(&w.env, code)
}

fn period(kind: CommissionPeriodKind) -> CommissionPeriod {
    CommissionPeriod {
        kind,
        ..CommissionPeriod::default()
    }
}

#[test]
fn a_day_a_year_and_a_range_give_their_rows_and_totals() {
    let w = worked_day();
    events_1_to_4(&w);
    // One day: its one row, the day's totals; with and without a party.
    let day = CommissionPeriod {
        date: Some("10-10-2026".into()),
        ..period(CommissionPeriodKind::Day)
    };
    let shop_day = month_for(&w, None, Some(day.clone()));
    assert_eq!(shop_day.rows.len(), 1);
    assert_eq!(rupees_of(shop_day.totals.total_comm_paisa), 5_448);
    assert_eq!(shop_day.period.kind, CommissionPeriodKind::Day);
    assert_eq!(
        (shop_day.period.from.as_str(), shop_day.period.to.as_str()),
        ("2026-10-10", "2026-10-10")
    );
    let rahim_day = month_for(&w, party(CommissionSide::Seller, w.rahim), Some(day));
    assert_eq!(rupees_of(rahim_day.totals.total_comm_paisa), 3_928);

    // The year 2026: one row for each month, key `YYYY-MM`; with رحیم one October row.
    let year = CommissionPeriod {
        year: Some(2026),
        ..period(CommissionPeriodKind::Year)
    };
    let shop_year = month_for(&w, None, Some(year.clone()));
    assert_eq!(shop_year.rows.len(), 1);
    assert_eq!(shop_year.rows[0].date, "2026-10");
    assert_eq!(rupees_of(shop_year.rows[0].total_comm_paisa), 5_448);
    assert_eq!(rupees_of(shop_year.rows[0].shared_income_paisa), 5_488);
    assert_eq!(shop_year.year, 2026);
    let rahim_year = month_for(&w, party(CommissionSide::Seller, w.rahim), Some(year));
    assert_eq!(rahim_year.rows.len(), 1);
    assert_eq!(rahim_year.rows[0].date, "2026-10");
    assert_eq!(
        figures(&rahim_year.rows[0]),
        (29_400, 1_764, 1_764, 400, 3_928, Some(3_928))
    );
    assert_eq!(figures(&rahim_year.totals), figures(&rahim_year.rows[0]));

    // A range: the days that have a figure; the same totals.
    let range = CommissionPeriod {
        from: Some("01-10-2026".into()),
        to: Some("10-10-2026".into()),
        ..period(CommissionPeriodKind::Range)
    };
    let shop_range = month_for(&w, None, Some(range));
    assert_eq!(shop_range.rows.len(), 1);
    assert_eq!(shop_range.total_count, 1);
    assert_eq!(rupees_of(shop_range.totals.total_comm_paisa), 5_448);
    // An empty range is an empty page.
    let before = CommissionPeriod {
        from: Some("01-09-2026".into()),
        to: Some("30-09-2026".into()),
        ..period(CommissionPeriodKind::Range)
    };
    assert!(month_for(&w, None, Some(before)).rows.is_empty());
}

#[test]
fn a_future_date_a_reversed_range_and_missing_fields_are_refused() {
    let w = worked_day();
    events_1_to_4(&w);
    let day = |date: &str| CommissionPeriod {
        date: Some(date.into()),
        ..period(CommissionPeriodKind::Day)
    };
    assert_eq!(
        refused(&w, None, Some(day("11-10-2026"))),
        vec![(ErrorCode::ErrDateInFuture, "period.date".to_owned())]
    );
    let reversed = CommissionPeriod {
        from: Some("10-10-2026".into()),
        to: Some("01-10-2026".into()),
        ..period(CommissionPeriodKind::Range)
    };
    assert_eq!(
        refused(&w, None, Some(reversed)),
        vec![(ErrorCode::ErrDateRange, "period.to".to_owned())]
    );
    assert_eq!(
        refused(&w, None, Some(period(CommissionPeriodKind::Year))),
        vec![(ErrorCode::ErrRequired, "period.year".to_owned())]
    );
    let next_year = CommissionPeriod {
        year: Some(2027),
        ..period(CommissionPeriodKind::Year)
    };
    assert_eq!(
        refused(&w, None, Some(next_year)),
        vec![(ErrorCode::ErrDateInFuture, "period.year".to_owned())]
    );
    // Both problems of a range come back together.
    let both = CommissionPeriod {
        from: Some("31-02-2026".into()),
        to: None,
        ..period(CommissionPeriodKind::Range)
    };
    assert_eq!(refused(&w, None, Some(both)).len(), 2);
    // The day detail refuses a day after today the way the month does (robustness finding 8).
    for party in [None, party(CommissionSide::Seller, w.rahim)] {
        let error = dashboard::commission_day(
            w.env.ctx.db(),
            &CommissionDayInput {
                date: "2026-10-11".into(),
                offset: 0,
                limit: 0,
                party,
                goods: None,
            },
        )
        .unwrap_err();
        match error {
            mandi_reports::ReportsError::Rejected(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].code, ErrorCode::ErrDateInFuture);
                assert_eq!(errors[0].field, "date");
            }
            other => panic!("{other:?}"),
        }
    }
    // Today itself is read.
    dashboard::commission_day(
        w.env.ctx.db(),
        &CommissionDayInput {
            date: "2026-10-10".into(),
            offset: 0,
            limit: 0,
            party: None,
            goods: None,
        },
    )
    .unwrap();
}

#[test]
fn the_commission_sums_of_a_range_are_read_from_covering_indexes_in_day_order() {
    // Code Standards B18, robustness finding 3: ten years of the whole shop read in about a second
    // because each day's sums come from a covering index (migration 24), with no sort.
    let w = worked_day();
    events_1_to_4(&w);
    let conn = raw(&w.dir);
    for ((sql, args), index) in [
        (
            dashboard::commission_plans::facts_by_day("2016-10-11", "2026-10-10"),
            "idx_analytics_facts_date_sums",
        ),
        (
            dashboard::commission_plans::shares_by_day("2016-10-11", "2026-10-10"),
            "idx_partner_share_rows_kind_date",
        ),
    ] {
        let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let plan = statement
            .query_map(
                mandi_services::db::rusqlite::params_from_iter(args.iter()),
                |r| r.get::<_, String>(3),
            )
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(plan.contains(&format!("COVERING INDEX {index}")), "{plan}");
        assert!(!plan.contains("TEMP B-TREE"), "{plan}");
    }
}

#[test]
fn a_range_is_paged_and_its_totals_cover_the_whole_range() {
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    // The second بکری is of 11-10-26.
    set_today(&w.env, 2026, 10, 11);
    let mut second = event2(&w);
    second.header.entry_date = "11-10-2026".into();
    save_ok(&w, &second, false);
    let range = |offset: u32, limit: u32| {
        dashboard::commission_month(
            w.env.ctx.db(),
            &CommissionMonthInput {
                month: None,
                period: Some(CommissionPeriod {
                    from: Some("01-10-2026".into()),
                    to: Some("11-10-2026".into()),
                    ..period(CommissionPeriodKind::Range)
                }),
                party: None,
                goods: None,
                offset: Some(offset),
                limit: Some(limit),
            },
        )
        .unwrap()
    };
    let first = range(0, 1);
    assert_eq!(first.total_count, 2);
    assert_eq!(first.rows.len(), 1);
    assert_eq!(first.rows[0].date, "2026-10-10");
    assert_eq!((first.offset, first.limit), (0, 1));
    // The totals are the whole range, never the page: 3,928 + 1,520.
    assert_eq!(rupees_of(first.totals.total_comm_paisa), 5_448);
    let next = range(1, 1);
    assert_eq!(next.rows.len(), 1);
    assert_eq!(next.rows[0].date, "2026-10-11");
    assert_eq!(rupees_of(next.rows[0].total_comm_paisa), 1_520);
    assert_eq!(next.totals, first.totals);
    // A month gives a row for each day.
    let month = month_for(&w, None, None);
    assert_eq!(month.rows.len(), 2);
    assert_eq!(month.total_count, 2);
}

#[test]
fn the_day_detail_lists_only_the_postings_of_the_party() {
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    save_ok(&w, &event2(&w), false);
    let day = |party: Option<CommissionParty>| {
        dashboard::commission_day(
            w.env.ctx.db(),
            &CommissionDayInput {
                date: "2026-10-10".into(),
                offset: 0,
                limit: 0,
                party,
                goods: None,
            },
        )
        .unwrap()
    };
    let whole = day(None);
    assert_eq!(whole.total_count, 2);
    // رحیم: his بکری only, with his figures and only his lines.
    let rahim = day(party(CommissionSide::Seller, w.rahim));
    assert_eq!(rahim.total_count, 1);
    assert_eq!(rahim.entries[0].bikri_number, Some(1));
    assert_eq!(rahim.entries[0].sellers.len(), 1);
    assert_eq!(rahim.entries[0].buyers.len(), 2);
    assert_eq!(
        figures(&rahim.entries[0].figures),
        (29_400, 1_764, 1_764, 400, 3_928, Some(3_928))
    );
    assert_eq!(rahim.totals, rahim.entries[0].figures);
    assert!(rahim.partners.is_empty());
    // اکرم: the one line of بکری 1 that is his; ارشد is not listed among his buyers.
    let akram = day(party(CommissionSide::Buyer, w.akram));
    assert_eq!(akram.total_count, 1);
    assert_eq!(akram.entries[0].buyers.len(), 1);
    assert_eq!(
        figures(&akram.entries[0].figures),
        (12_000, 0, 720, 160, 880, None)
    );
    // A buyer who is in neither posting has an empty day.
    let naqdi = day(party(CommissionSide::Buyer, w.naqdi));
    assert_eq!(naqdi.total_count, 0);
    // Locked partner section: a party page has nothing to hide.
    let mut locked = day(party(CommissionSide::Seller, w.rahim));
    locked.hide_partners();
    assert!(!locked.partners_locked);
}

#[test]
fn the_print_names_the_party_and_the_period() {
    let w = worked_day();
    events_1_to_4(&w);
    let input = |party: Option<CommissionParty>, period: Option<CommissionPeriod>, language| {
        CommissionPrintInput {
            month: None,
            period,
            party,
            goods: None,
            language,
            paper: None,
        }
    };
    // A seller: date, sale, commission charged, commission on his goods; the header names him.
    let spec = dashboard::commission_month_spec(
        w.env.ctx.db(),
        &input(party(CommissionSide::Seller, w.rahim), None, Language::En),
        true,
    )
    .unwrap();
    assert_eq!(spec.columns.len(), 4);
    let keys: Vec<&str> = spec.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["date", "sale", "seller_comm", "goods_comm"]);
    let seller_fact = spec
        .facts
        .iter()
        .find(|f| f.label_id == "print-fact-seller")
        .unwrap();
    assert!(seller_fact.value.contains("Rahim"), "{seller_fact:?}");
    assert!(seller_fact.value.contains("0010"), "{seller_fact:?}");
    assert_eq!(spec.rows.len(), 1);
    // A buyer: date, purchase value, buyer commission, لاگا, total.
    let buyer = dashboard::commission_month_print_data(
        w.env.ctx.db(),
        &input(party(CommissionSide::Buyer, w.akram), None, Language::Ur),
        true,
    )
    .unwrap();
    assert_eq!(buyer.columns.len(), 5);
    assert_eq!(buyer.rows.len(), 1);
    // The partner columns are for the whole shop only.
    let shop = dashboard::commission_month_print_data(
        w.env.ctx.db(),
        &input(None, None, Language::En),
        true,
    )
    .unwrap();
    assert_eq!(shop.columns.len(), 9);
    // A year prints its months, and its header says the year.
    let year = CommissionPeriod {
        year: Some(2026),
        ..period(CommissionPeriodKind::Year)
    };
    let spec = dashboard::commission_month_spec(
        w.env.ctx.db(),
        &input(None, Some(year), Language::En),
        false,
    )
    .unwrap();
    assert_eq!(spec.title_id, "print-commission-title-period");
    assert!(
        spec.facts
            .iter()
            .any(|f| f.label_id == "print-fact-year" && f.value == "2026")
    );
    assert!(matches!(
        spec.rows[0].cells[0],
        Cell::Month {
            year: 2026,
            month: 10
        }
    ));
}

// ---- Analytics view L: buyer by item ----

fn view(w: &support::World, v: AnalyticsView, filters: AnalyticsFilters) -> AnalyticsPage {
    analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: v,
            filters,
            offset: 0,
            limit: 0,
            sort: None,
        },
    )
    .unwrap()
}

/// A money cell in rupees, 0 for an empty cell.
fn rupees_cell(cell: &Cell) -> i64 {
    match cell {
        Cell::Money { paisa } => rupees_of(*paisa),
        Cell::Empty => 0,
        other => panic!("not an amount: {other:?}"),
    }
}

#[test]
fn view_l_is_each_buyers_own_commission_by_item() {
    let w = worked_day();
    events_1_to_4(&w);
    let page = view(&w, AnalyticsView::L, AnalyticsFilters::default());
    let keys: Vec<&str> = page.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "buyer".to_owned(),
            format!("i_{}", w.banana),
            format!("i_{}", w.apple),
            "total_comm".to_owned()
        ]
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
    );
    // ارشد 1,284 and اکرم 880 on کیلا, شاہد 800 on سیب; the biggest buyer first.
    let rows: Vec<(String, i64, i64, i64)> = page
        .rows
        .iter()
        .map(|r| {
            let name = match &r.cells[0] {
                Cell::Party { party } => party.name_en.clone(),
                other => panic!("{other:?}"),
            };
            (
                name,
                rupees_cell(&r.cells[1]),
                rupees_cell(&r.cells[2]),
                rupees_cell(&r.cells[3]),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("Arshad Multani".to_owned(), 1_284, 0, 1_284),
            ("Akram Toll Tax".to_owned(), 880, 0, 880),
            ("Shahid".to_owned(), 0, 800, 800),
        ]
    );
    // The totals row: کیلا 2,164, سیب 800, 2,964 (the buyer commission 2,484 + لاگا 480).
    let totals: Vec<i64> = page.totals[1..].iter().map(rupees_cell).collect();
    assert_eq!(totals, vec![2_164, 800, 2_964]);
    assert_eq!(page.total_count, 3);
    // A cell opens the lines behind it, and they add up to the cell: اکرم × کیلا.
    let akram_row = page
        .rows
        .iter()
        .find(|r| matches!(&r.cells[0], Cell::Party { party } if party.account_id == w.akram))
        .unwrap();
    let banana = page.columns[1].drill.clone();
    let opened = analytics_lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::L,
            filters: AnalyticsFilters::default(),
            row_key: akram_row.key.clone().unwrap(),
            column_key: banana,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(opened.total_count, 1);
    // A buyer filter narrows to him; the side filter is refused (it is not his).
    let one = view(
        &w,
        AnalyticsView::L,
        AnalyticsFilters {
            buyer: Some(PartyFilter {
                account_id: w.arshad,
                person_key: None,
            }),
            ..AnalyticsFilters::default()
        },
    );
    assert_eq!(one.rows.len(), 1);
    let side = analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::L,
            filters: AnalyticsFilters {
                side: mandi_reports::analytics::AnalyticsSide::Seller,
                ..AnalyticsFilters::default()
            },
            offset: 0,
            limit: 0,
            sort: None,
        },
    );
    assert!(side.is_err());
}

#[test]
fn a_party_and_an_item_filter_combine_in_view_a() {
    let w = worked_day();
    events_1_to_4(&w);
    // The seller رحیم and the item کیلا: one row.
    let seller = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            seller_id: Some(w.rahim),
            item_id: Some(w.banana),
            ..AnalyticsFilters::default()
        },
    );
    assert_eq!(seller.rows.len(), 1);
    // اکرم and سیب: none (he bought only کیلا).
    let buyer = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            buyer: Some(PartyFilter {
                account_id: w.akram,
                person_key: None,
            }),
            item_id: Some(w.apple),
            ..AnalyticsFilters::default()
        },
    );
    assert!(buyer.rows.is_empty());
}

// ---- Analytics view M: expenses by head ----

#[test]
fn view_m_is_the_shop_expense_of_each_head_by_period() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    let page = view(&w, AnalyticsView::M, AnalyticsFilters::default());
    // October 2026: بجلی 3,000; the personal 2,000 of الف is not a shop expense.
    assert_eq!(page.rows.len(), 1);
    assert_eq!(rupees_cell(&page.rows[0].cells[1]), 3_000);
    assert_eq!(rupees_cell(&page.rows[0].cells[2]), 3_000);
    let totals: Vec<i64> = page.totals[1..].iter().map(rupees_cell).collect();
    assert_eq!(totals, vec![3_000, 3_000]);
    // By day and by year.
    for (group, headings) in [(AnalyticsGroupBy::Day, 1), (AnalyticsGroupBy::Year, 1)] {
        let grouped = view(
            &w,
            AnalyticsView::M,
            AnalyticsFilters {
                group_by: Some(group),
                ..AnalyticsFilters::default()
            },
        );
        assert_eq!(grouped.columns.len(), headings + 2, "{group:?}");
        assert_eq!(rupees_cell(grouped.totals.last().unwrap()), 3_000);
    }
    // The cell opens the voucher line behind it, and it adds up to the cell.
    let opened = analytics_lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::M,
            filters: AnalyticsFilters::default(),
            row_key: page.rows[0].key.clone().unwrap(),
            column_key: page.columns[1].drill.clone(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(opened.total_count, 1);
    assert_eq!(rupees_cell(opened.totals.last().unwrap()), 3_000);
    assert!(opened.rows[0].source.is_some());
    // Every head of the totals row.
    let all = analytics_lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::M,
            filters: AnalyticsFilters::default(),
            row_key: AnalyticsKey::default(),
            column_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(rupees_cell(all.totals.last().unwrap()), 3_000);
    // No sale filter applies to an expense view.
    let refused = analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::M,
            filters: AnalyticsFilters {
                item_id: Some(w.banana),
                ..AnalyticsFilters::default()
            },
            offset: 0,
            limit: 0,
            sort: None,
        },
    );
    assert!(refused.is_err());
}

#[test]
fn a_cancelled_expense_nets_out_of_view_m() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    let saved = vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &shop_input(electricity, "3,000", "10-10-2026"),
    )
    .unwrap();
    let before = view(&w, AnalyticsView::M, AnalyticsFilters::default());
    assert_eq!(rupees_cell(before.totals.last().unwrap()), 3_000);
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::VoucherCancelInput {
            id: saved.voucher_id,
            version: row.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // The reversal is dated like the voucher, so the head nets to 0 inside its own day and has no
    // row.
    let after = view(&w, AnalyticsView::M, AnalyticsFilters::default());
    assert!(after.rows.is_empty(), "{:?}", after.rows);
    assert_eq!(rupees_cell(after.totals.last().unwrap()), 0);
}

// ---- View M, the partner part: the kharch each partner spent himself ----

fn partner_filters(partner: Option<i64>) -> AnalyticsFilters {
    AnalyticsFilters {
        from: "01-10-2026".into(),
        to: "31-10-2026".into(),
        partner_id: partner,
        expense_part: Some(mandi_reports::analytics::AnalyticsExpensePart::Partner),
        ..AnalyticsFilters::default()
    }
}

#[test]
fn view_m_partner_part_is_the_kharch_each_partner_spent_himself() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    let page = view(&w, AnalyticsView::M, partner_filters(None));
    let keys: Vec<&str> = page.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["partner", "own"]);
    // After Events 1 to 5, October 2026: الف 2,000, ب 0, ج 0, total 2,000. No share of the shop kharch.
    let rows: Vec<i64> = page.rows.iter().map(|r| rupees_cell(&r.cells[1])).collect();
    assert_eq!(rows, vec![2_000, 0, 0]);
    assert_eq!(rupees_cell(&page.totals[1]), 2_000);
    // A ranked chart of the same figures: one point for each partner, keyed by the partner.
    assert_eq!(page.charts.len(), 1);
    assert_eq!(page.charts[0].series[0].key, "own");
    let points: Vec<Option<i64>> = page.charts[0]
        .points
        .iter()
        .map(|p| p.key.as_ref().and_then(|k| k.partner_id))
        .collect();
    assert_eq!(points.len(), 3);
    assert!(points.iter().all(Option::is_some));
    // The shop part of the same period is the shop's kharch: بجلی 3,000.
    let shop = view(&w, AnalyticsView::M, AnalyticsFilters::default());
    assert_eq!(rupees_cell(shop.totals.last().unwrap()), 3_000);
    assert_eq!(
        page.filters.expense_part,
        Some(mandi_reports::analytics::AnalyticsExpensePart::Partner)
    );
    // One partner with the partner filter.
    let one = view(&w, AnalyticsView::M, partner_filters(Some(w.partners[0])));
    assert_eq!(one.rows.len(), 1);
    assert_eq!(rupees_cell(&one.rows[0].cells[1]), 2_000);
    let none = view(&w, AnalyticsView::M, partner_filters(Some(w.partners[1])));
    assert_eq!(rupees_cell(&none.rows[0].cells[1]), 0);
    // The partner filter belongs to the partner part only, and the part to view M only.
    let run = |view: AnalyticsView, filters: AnalyticsFilters| {
        analytics_run(
            w.env.ctx.db(),
            &AnalyticsRunInput {
                view,
                filters,
                offset: 0,
                limit: 0,
                sort: None,
            },
        )
    };
    assert!(
        run(
            AnalyticsView::M,
            AnalyticsFilters {
                partner_id: Some(w.partners[0]),
                ..AnalyticsFilters::default()
            }
        )
        .is_err()
    );
    assert!(run(AnalyticsView::A, partner_filters(None)).is_err());
}

#[test]
fn view_m_partner_part_by_day_and_its_entries_add_up_to_the_cell() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    // A second take of الف on 11-10-26.
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &super::day_money::personal_input(w.partners[0], "500", "11-10-2026"),
    )
    .unwrap();
    let page = view(&w, AnalyticsView::M, partner_filters(Some(w.partners[0])));
    assert_eq!(rupees_cell(&page.rows[0].cells[1]), 2_500);
    // By day: a row for each day with a take, and the totals row is the whole period.
    let mut by_day = partner_filters(Some(w.partners[0]));
    by_day.group_by = Some(AnalyticsGroupBy::Day);
    let days = view(&w, AnalyticsView::M, by_day);
    assert_eq!(days.columns[0].key, "period");
    let amounts: Vec<i64> = days.rows.iter().map(|r| rupees_cell(&r.cells[2])).collect();
    assert_eq!(amounts, vec![2_000, 500]);
    assert_eq!(rupees_cell(&days.totals[2]), 2_500);
    // The entries behind the cell: date, voucher, partner, reason and amount; they add up.
    let entries = analytics_lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::M,
            filters: partner_filters(Some(w.partners[0])),
            row_key: AnalyticsKey {
                partner_id: Some(w.partners[0]),
                ..AnalyticsKey::default()
            },
            column_key: page.columns[1].drill.clone().map(|d| AnalyticsKey {
                partner_id: None,
                ..d
            }),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let keys: Vec<&str> = entries.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["date", "doc", "name", "reason", "amount"]);
    assert_eq!(entries.total_count, 2);
    assert_eq!(rupees_cell(entries.totals.last().unwrap()), 2_500);
    assert!(entries.rows.iter().all(|r| r.source.is_some()));
    // The typed reason of the take shows (the worked day uses «گھر کا خرچ»).
    assert!(matches!(&entries.rows[0].cells[3], Cell::Text { text } if !text.is_empty()));
    // A partner's entries are not read under the shop part.
    let leak = analytics_lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::M,
            filters: AnalyticsFilters::default(),
            row_key: AnalyticsKey {
                partner_id: Some(w.partners[0]),
                money: Some(mandi_reports::analytics::AnalyticsPartnerMoney::Taken),
                ..AnalyticsKey::default()
            },
            column_key: None,
            offset: 0,
            limit: 0,
        },
    );
    assert!(leak.is_err());
}

// ---- The coverage audit (plan 77 Section 4) ----

#[test]
fn every_view_of_the_screens_takes_a_period_and_a_custom_range() {
    let w = worked_day();
    events_1_to_4(&w);
    let catalog = screen_catalog();
    assert_eq!(catalog.len(), 13);
    for entry in &catalog {
        assert!(
            entry
                .filters
                .contains(&mandi_reports::analytics::AnalyticsFilterKind::Period),
            "{:?} has no period",
            entry.id
        );
        // A custom range, one day and one month, all read.
        for (from, to) in [
            ("01-10-2026", "10-10-2026"),
            ("10-10-2026", "10-10-2026"),
            ("01-10-2026", "31-10-2026"),
        ] {
            let page = view(
                &w,
                entry.id,
                AnalyticsFilters {
                    from: from.into(),
                    to: to.into(),
                    ..AnalyticsFilters::default()
                },
            );
            assert_eq!(page.view, entry.id);
        }
    }
}

#[test]
fn the_grouping_and_the_party_with_item_combinations_are_taken_where_the_views_have_them() {
    use mandi_reports::analytics::AnalyticsFilterKind as Kind;
    let w = worked_day();
    events_1_to_4(&w);
    // Every view takes the grouping by day, month or year, except the top 10 (H): D, G and M group
    // their columns, the others their rows.
    for entry in screen_catalog() {
        let groups = entry.filters.contains(&Kind::GroupBy);
        assert_eq!(groups, entry.id != AnalyticsView::H, "{:?}", entry.id);
        for group in [
            AnalyticsGroupBy::Day,
            AnalyticsGroupBy::Month,
            AnalyticsGroupBy::Year,
        ] {
            let result = analytics_run(
                w.env.ctx.db(),
                &AnalyticsRunInput {
                    view: entry.id,
                    filters: AnalyticsFilters {
                        group_by: Some(group),
                        ..AnalyticsFilters::default()
                    },
                    offset: 0,
                    limit: 0,
                    sort: None,
                },
            );
            assert_eq!(result.is_ok(), groups, "{:?} by {group:?}", entry.id);
        }
    }
    // A seller with an item, and a buyer with an item, in every view that counts sales by them.
    for v in [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::D,
        AnalyticsView::E,
        AnalyticsView::G,
        AnalyticsView::H,
        AnalyticsView::L,
    ] {
        let seller = view(
            &w,
            v,
            AnalyticsFilters {
                seller_id: Some(w.rahim),
                item_id: Some(w.banana),
                ..AnalyticsFilters::default()
            },
        );
        assert_eq!(seller.view, v);
        let buyer = view(
            &w,
            v,
            AnalyticsFilters {
                buyer: Some(PartyFilter {
                    account_id: w.akram,
                    person_key: None,
                }),
                item_id: Some(w.banana),
                ..AnalyticsFilters::default()
            },
        );
        assert_eq!(buyer.view, v);
    }
}

#[test]
fn the_rows_of_views_f_i_j_and_k_group_by_day_month_and_year() {
    use mandi_reports::analytics::AnalyticsRow;
    use mandi_reports::reports::ColumnKind;
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    // The second بکری is of 11-10-26, so the range has two days with shares.
    set_today(&w.env, 2026, 10, 11);
    let mut second = event2(&w);
    second.header.entry_date = "11-10-2026".into();
    save_ok(&w, &second, false);
    let range = AnalyticsFilters {
        from: "01-10-2026".into(),
        to: "11-10-2026".into(),
        ..AnalyticsFilters::default()
    };
    let money = |row: &AnalyticsRow, at: usize| match row.cells.get(at) {
        Some(Cell::Money { paisa }) => paisa.as_paisa(),
        _ => 0,
    };
    for v in [
        AnalyticsView::F,
        AnalyticsView::I,
        AnalyticsView::J,
        AnalyticsView::K,
    ] {
        let plain = view(&w, v, range.clone());
        for (group, buckets) in [
            (AnalyticsGroupBy::Day, 2),
            (AnalyticsGroupBy::Month, 1),
            (AnalyticsGroupBy::Year, 1),
        ] {
            let grouped = view(
                &w,
                v,
                AnalyticsFilters {
                    group_by: Some(group),
                    ..range.clone()
                },
            );
            // The period first, then the columns of the view; the totals are the whole period.
            assert_eq!(grouped.columns[0].key, "period", "{v:?} {group:?}");
            let keys = |page: &AnalyticsPage| -> Vec<String> {
                page.columns.iter().map(|c| c.key.clone()).collect()
            };
            assert_eq!(keys(&grouped)[1..], keys(&plain)[..], "{v:?} {group:?}");
            assert_eq!(grouped.totals[1..], plain.totals[..], "{v:?} {group:?}");
            assert_eq!(grouped.filters.group_by, Some(group));
            assert!(grouped.charts.is_empty());
            assert_eq!(
                grouped.total_count == 0,
                plain.total_count == 0,
                "{v:?} {group:?}"
            );
            if v != AnalyticsView::F {
                continue;
            }
            // Three partners in every bucket; each figure of a partner adds up over the buckets.
            assert_eq!(grouped.rows.len(), 3 * buckets, "{group:?}");
            for p in &plain.rows {
                let partner = p.key.as_ref().and_then(|k| k.partner_id);
                for at in 1..plain.columns.len() {
                    if plain.columns[at].kind != ColumnKind::Money {
                        continue;
                    }
                    let sum: i64 = grouped
                        .rows
                        .iter()
                        .filter(|r| r.key.as_ref().and_then(|k| k.partner_id) == partner)
                        .map(|r| money(r, at + 1))
                        .sum();
                    assert_eq!(sum, money(p, at), "{group:?} column {at}");
                }
            }
            // A row opens its lines inside its own bucket.
            assert!(grouped.rows.iter().all(|r| {
                r.key
                    .as_ref()
                    .is_some_and(|k| k.from.is_some() && k.to.is_some())
            }));
        }
    }
}

#[test]
fn the_rows_of_views_a_b_c_e_and_l_group_by_day_month_and_year() {
    use mandi_reports::reports::ColumnKind;
    use std::collections::BTreeSet;
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    // The second بکری is of 11-10-26: every item, seller and buyer of the worked day sits on one day.
    set_today(&w.env, 2026, 10, 11);
    let mut second = event2(&w);
    second.header.entry_date = "11-10-2026".into();
    save_ok(&w, &second, false);
    let range = AnalyticsFilters {
        from: "01-10-2026".into(),
        to: "11-10-2026".into(),
        ..AnalyticsFilters::default()
    };
    let money_sum = |page: &AnalyticsPage, at: usize| -> i64 {
        page.rows.iter().map(|r| rupees_cell(&r.cells[at])).sum()
    };
    for v in [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::E,
        AnalyticsView::L,
    ] {
        let plain = view(&w, v, range.clone());
        for (group, buckets) in [
            (AnalyticsGroupBy::Day, 2),
            (AnalyticsGroupBy::Month, 1),
            (AnalyticsGroupBy::Year, 1),
        ] {
            let grouped = view(
                &w,
                v,
                AnalyticsFilters {
                    group_by: Some(group),
                    ..range.clone()
                },
            );
            // The period first, then the columns of the view; the totals are the whole period, never
            // a sum of the buckets.
            assert_eq!(grouped.columns[0].key, "period", "{v:?} {group:?}");
            let keys = |page: &AnalyticsPage| -> Vec<String> {
                page.columns.iter().map(|c| c.key.clone()).collect()
            };
            assert_eq!(keys(&grouped)[1..], keys(&plain)[..], "{v:?} {group:?}");
            assert_eq!(grouped.totals[1..], plain.totals[..], "{v:?} {group:?}");
            assert_eq!(grouped.filters.group_by, Some(group));
            assert!(grouped.charts.is_empty(), "{v:?} {group:?}");
            // Each row sits on one day, so the grouped view has the rows of the plain view, in
            // `buckets` periods, and every amount adds up to the plain view's.
            assert_eq!(grouped.total_count, plain.total_count, "{v:?} {group:?}");
            let periods: BTreeSet<String> = grouped
                .rows
                .iter()
                .map(|r| format!("{:?}", r.cells[0]))
                .collect();
            assert_eq!(periods.len(), buckets, "{v:?} {group:?}");
            for (at, column) in plain.columns.iter().enumerate() {
                if column.kind != ColumnKind::Money || column.key == "avg_rate" {
                    continue;
                }
                assert_eq!(
                    money_sum(&grouped, at + 1),
                    money_sum(&plain, at),
                    "{v:?} {group:?} {}",
                    column.key
                );
            }
            // A row opens its lines inside its own bucket.
            assert!(grouped.rows.iter().all(|r| {
                r.key
                    .as_ref()
                    .is_some_and(|k| k.from.is_some() && k.to.is_some())
            }));
        }
    }

    // View B by month, October 2026: one bucket, رحیم 3,928 and کریم 1,520 (the worked day).
    let by_month = view(
        &w,
        AnalyticsView::B,
        AnalyticsFilters {
            group_by: Some(AnalyticsGroupBy::Month),
            ..range.clone()
        },
    );
    let total_at = by_month
        .columns
        .iter()
        .position(|c| c.key == "total_comm")
        .unwrap();
    let mut rows: Vec<(String, i64)> = by_month
        .rows
        .iter()
        .map(|r| {
            assert!(matches!(
                &r.cells[0],
                Cell::Month {
                    year: 2026,
                    month: 10
                }
            ));
            let name = match &r.cells[1] {
                Cell::Party { party } => party.name_en.clone(),
                other => panic!("{other:?}"),
            };
            (name, rupees_cell(&r.cells[total_at]))
        })
        .collect();
    rows.sort();
    assert_eq!(
        rows,
        vec![("Karim".to_owned(), 1_520), ("Rahim".to_owned(), 3_928)]
    );
    // By day the two sellers fall on their own days: Rahim on 10-10, Karim on 11-10.
    let by_day = view(
        &w,
        AnalyticsView::B,
        AnalyticsFilters {
            group_by: Some(AnalyticsGroupBy::Day),
            ..range.clone()
        },
    );
    let days: Vec<(String, i64)> = by_day
        .rows
        .iter()
        .map(|r| {
            let day = match &r.cells[0] {
                Cell::Date { iso } => iso.clone(),
                other => panic!("{other:?}"),
            };
            (day, rupees_cell(&r.cells[total_at]))
        })
        .collect();
    assert_eq!(
        days,
        vec![
            ("2026-10-10".to_owned(), 3_928),
            ("2026-10-11".to_owned(), 1_520)
        ]
    );
    // Only the top 10 refuses the grouping.
    let top = analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::H,
            filters: AnalyticsFilters {
                group_by: Some(AnalyticsGroupBy::Month),
                ..range
            },
            offset: 0,
            limit: 0,
            sort: None,
        },
    );
    assert!(top.is_err());
}

/// One page of a view, as the screen asks for it.
fn page_of_view(
    w: &support::World,
    v: AnalyticsView,
    filters: AnalyticsFilters,
    offset: i64,
    limit: i64,
) -> AnalyticsPage {
    analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: v,
            filters,
            offset,
            limit,
            sort: None,
        },
    )
    .unwrap()
}

#[test]
fn a_page_of_a_grouped_sale_view_is_that_part_of_all_its_rows() {
    // Robustness finding 2: a page of a sale view grouped by period counts the rows of each bucket
    // and reads only the buckets of the page. Its rows, count and totals are those of the whole view.
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    set_today(&w.env, 2026, 10, 11);
    let mut second = event2(&w);
    second.header.entry_date = "11-10-2026".into();
    save_ok(&w, &second, false);
    // A third بکری in the next month, so a grouping by month has two buckets, a page can start in the
    // first and end in the second, and a grouping by day has three days.
    set_today(&w.env, 2026, 11, 3);
    let mut third = event2(&w);
    third.header.entry_date = "03-11-2026".into();
    save_ok(&w, &third, false);
    let range = AnalyticsFilters {
        from: "01-10-2026".into(),
        to: "03-11-2026".into(),
        ..AnalyticsFilters::default()
    };
    for v in [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::E,
        AnalyticsView::L,
    ] {
        for (group, buckets) in [(AnalyticsGroupBy::Day, 3), (AnalyticsGroupBy::Month, 2)] {
            let filters = AnalyticsFilters {
                group_by: Some(group),
                ..range.clone()
            };
            let all = page_of_view(&w, v, filters.clone(), 0, 0);
            assert!(all.total_count >= 3, "{v:?} {group:?}");
            // Every bucket has a row: the periods of the rows are the buckets.
            let periods: std::collections::BTreeSet<String> = all
                .rows
                .iter()
                .map(|r| format!("{:?}", r.cells[0]))
                .collect();
            assert_eq!(periods.len(), buckets, "{v:?} {group:?}");
            for limit in [1_i64, 2, 3] {
                let mut rows: Vec<String> = Vec::new();
                let mut offset = 0;
                while offset < all.total_count {
                    let page = page_of_view(&w, v, filters.clone(), offset, limit);
                    assert_eq!(page.total_count, all.total_count, "{v:?} {group:?} {limit}");
                    assert_eq!(page.totals, all.totals, "{v:?} {group:?} {limit}");
                    assert_eq!(page.columns.len(), all.columns.len());
                    assert!(page.rows.len() <= usize::try_from(limit).unwrap());
                    rows.extend(page.rows.iter().map(|r| format!("{r:?}")));
                    offset += limit;
                }
                let expected: Vec<String> = all.rows.iter().map(|r| format!("{r:?}")).collect();
                assert_eq!(rows, expected, "{v:?} {group:?} pages of {limit}");
            }
            // A page past the end has no row and the same count.
            let past = page_of_view(&w, v, filters.clone(), all.total_count + 5, 10);
            assert!(past.rows.is_empty());
            assert_eq!(past.total_count, all.total_count);
        }
    }
}

#[test]
fn view_d_refuses_a_closed_range_of_more_than_144_periods_from_its_dates() {
    // Robustness finding 2 (D): ten years by month was read in full and refused two seconds later. The
    // periods of a closed range are counted from its dates, so even a range with sales in one month
    // only is refused before a fact is read. Plan 80 gate: the cap is 144 periods (twelve years by
    // month, PRD 18.2 view D: one column for each month); it was 36, which refused five years.
    let w = worked_day();
    events_1_to_4(&w);
    let d = |from: &str, to: &str, group: Option<AnalyticsGroupBy>| {
        analytics_run(
            w.env.ctx.db(),
            &AnalyticsRunInput {
                view: AnalyticsView::D,
                filters: AnalyticsFilters {
                    from: from.into(),
                    to: to.into(),
                    group_by: group,
                    ..AnalyticsFilters::default()
                },
                offset: 0,
                limit: 0,
                sort: None,
            },
        )
    };
    let refused_at = |result: Result<AnalyticsPage, mandi_reports::ReportsError>| match result {
        Err(mandi_reports::ReportsError::Rejected(found)) => found
            .into_iter()
            .map(|e| (e.code, e.field))
            .collect::<Vec<_>>(),
        other => panic!("expected a refusal, got {other:?}"),
    };
    let group_by = vec![(ErrorCode::ErrOutOfRange, "filters.group_by".to_owned())];
    // Twelve years and a month by month (145 months), by default (month) and by day over 283 days.
    assert_eq!(
        refused_at(d("01-10-2014", "10-10-2026", Some(AnalyticsGroupBy::Month))),
        group_by
    );
    assert_eq!(refused_at(d("01-10-2014", "10-10-2026", None)), group_by);
    assert_eq!(
        refused_at(d("01-01-2026", "10-10-2026", Some(AnalyticsGroupBy::Day))),
        group_by
    );
    // 144 months are read (ten years and the 121 months of the 10-year volume, and five years by
    // month, which the cap of 36 used to refuse), 145 are refused; the same years by year are read.
    assert!(d("01-11-2014", "10-10-2026", Some(AnalyticsGroupBy::Month)).is_ok());
    assert!(d("11-10-2016", "10-10-2026", Some(AnalyticsGroupBy::Month)).is_ok());
    assert!(d("01-10-2021", "10-10-2026", Some(AnalyticsGroupBy::Month)).is_ok());
    assert!(d("01-01-2020", "10-10-2026", Some(AnalyticsGroupBy::Year)).is_ok());
    // 144 days are read, 145 are refused.
    assert!(d("20-05-2026", "10-10-2026", Some(AnalyticsGroupBy::Day)).is_ok());
    assert_eq!(
        refused_at(d("19-05-2026", "10-10-2026", Some(AnalyticsGroupBy::Day))),
        group_by
    );
}

#[test]
fn view_c_refuses_a_buyer_filter_that_is_not_a_buyer() {
    // Robustness finding 5: the account of a seller is not a buyer, and view C says so, as the
    // commission page does, instead of answering with no rows.
    let w = worked_day();
    events_1_to_4(&w);
    let asked = |account_id: i64| {
        analytics_run(
            w.env.ctx.db(),
            &AnalyticsRunInput {
                view: AnalyticsView::C,
                filters: AnalyticsFilters {
                    buyer: Some(PartyFilter {
                        account_id,
                        person_key: None,
                    }),
                    ..AnalyticsFilters::default()
                },
                offset: 0,
                limit: 0,
                sort: None,
            },
        )
    };
    match asked(w.rahim) {
        Err(mandi_reports::ReportsError::Rejected(found)) => {
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].code, ErrorCode::ErrWrongPartyType);
            assert_eq!(found[0].field, "filters.buyer.account_id");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(asked(w.akram).unwrap().rows.len(), 1);
    // The cash account is no buyer either; the cash buyer نقدی منڈی is one (a buyer with no sale in
    // the period answers with no rows, which is not a refusal).
    let cash = support::id_of(&w.env, "01010001");
    match asked(cash) {
        Err(mandi_reports::ReportsError::Rejected(found)) => {
            assert_eq!(found[0].code, ErrorCode::ErrWrongPartyType);
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert!(asked(w.naqdi).unwrap().rows.is_empty());
}

/// The count of rows of three reads kept in memory for their next pages: view B by seller, view B by
/// month, and the commission page of a range (robustness findings 2 and 3).
fn kept_counts(w: &support::World) -> (i64, i64, i64) {
    let range = AnalyticsFilters {
        from: "01-10-2026".into(),
        to: "10-10-2026".into(),
        ..AnalyticsFilters::default()
    };
    let plain = page_of_view(w, AnalyticsView::B, range.clone(), 0, 1);
    let by_month = page_of_view(
        w,
        AnalyticsView::B,
        AnalyticsFilters {
            group_by: Some(AnalyticsGroupBy::Month),
            ..range
        },
        0,
        1,
    );
    let commission = dashboard::commission_month(
        w.env.ctx.db(),
        &CommissionMonthInput {
            month: None,
            period: Some(CommissionPeriod {
                from: Some("01-10-2026".into()),
                to: Some("10-10-2026".into()),
                ..period(CommissionPeriodKind::Range)
            }),
            party: None,
            goods: None,
            offset: Some(0),
            limit: Some(1),
        },
    )
    .unwrap();
    (
        plain.total_count,
        by_month.total_count,
        commission.total_count,
    )
}

#[test]
fn a_save_between_page_1_and_page_2_is_seen_by_page_2() {
    // The next page of the same filter is served from memory while the books have not moved, and a
    // save moves them: page 2 after a save shows the save (tech-lead decision of 15-11-2026).
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    let range = AnalyticsFilters {
        from: "01-10-2026".into(),
        to: "10-10-2026".into(),
        ..AnalyticsFilters::default()
    };
    let first = page_of_view(&w, AnalyticsView::B, range.clone(), 0, 1);
    assert_eq!(first.total_count, 1);
    // The same page again, and a next page: nothing moved, so the answer is the same.
    assert_eq!(
        page_of_view(&w, AnalyticsView::B, range.clone(), 0, 1).totals,
        first.totals
    );
    assert!(
        page_of_view(&w, AnalyticsView::B, range.clone(), 1, 1)
            .rows
            .is_empty()
    );
    assert_eq!(kept_counts(&w), (1, 1, 1));
    // A save between page 1 and page 2: a second seller sells.
    let two = save_ok(&w, &event2(&w), false);
    let second = page_of_view(&w, AnalyticsView::B, range.clone(), 1, 1);
    assert_eq!(second.total_count, 2, "page 2 must see the save");
    assert_eq!(second.rows.len(), 1);
    assert_ne!(second.totals, first.totals);
    assert_eq!(kept_counts(&w), (2, 2, 1));
    // The commission figures of the day moved too.
    let commission = month_for(
        &w,
        None,
        Some(CommissionPeriod {
            from: Some("01-10-2026".into()),
            to: Some("10-10-2026".into()),
            ..period(CommissionPeriodKind::Range)
        }),
    );
    assert_eq!(rupees_of(commission.totals.total_comm_paisa), 5_448);
    // A cancel is a save too: cancelling بکری 2 takes the second seller out again.
    let detail = bikri::get(&w.env.ctx, two.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: two.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_eq!(kept_counts(&w).0, 1);
}

#[test]
fn a_change_that_writes_none_of_the_three_tables_is_seen_by_the_next_page_and_by_a_snapshot() {
    // A change of an item's group writes `items` only: no fact, no journal entry, no share row. The
    // kept results of the screens' connection must still miss (the data version moves), and a
    // snapshot of the assistant is another connection that never shares a result (money review of
    // the cache, B1).
    use mandi_services::core::rules::ItemGroup;
    let w = worked_day();
    events_1_to_4(&w);
    let by_group = |db: &mandi_services::db::Db, group: ItemGroup| {
        analytics_run(
            db,
            &AnalyticsRunInput {
                view: AnalyticsView::A,
                filters: AnalyticsFilters {
                    item_group: Some(group),
                    ..AnalyticsFilters::default()
                },
                offset: 0,
                limit: 1,
                sort: None,
            },
        )
        .unwrap()
        .total_count
    };
    // کیلا and سیب are fruit: two rows in the fruit group, none in the vegetable group.
    assert_eq!(by_group(w.env.ctx.db(), ItemGroup::Fruit), 2);
    assert_eq!(by_group(w.env.ctx.db(), ItemGroup::Vegetable), 0);
    let first_snapshot = w.env.ctx.snapshot().unwrap();
    assert_eq!(by_group(first_snapshot.db(), ItemGroup::Fruit), 2);
    // سیب is moved to the vegetables: only `items` changes.
    raw(&w.dir)
        .execute(
            "UPDATE items SET item_group = 'vegetable' WHERE id = ?1",
            [w.apple],
        )
        .unwrap();
    assert_eq!(
        by_group(w.env.ctx.db(), ItemGroup::Fruit),
        1,
        "the screens' connection"
    );
    assert_eq!(by_group(w.env.ctx.db(), ItemGroup::Vegetable), 1);
    let second_snapshot = w.env.ctx.snapshot().unwrap();
    assert_eq!(
        by_group(second_snapshot.db(), ItemGroup::Fruit),
        1,
        "a new snapshot"
    );
    assert_eq!(by_group(second_snapshot.db(), ItemGroup::Vegetable), 1);
    // The first snapshot is still the moment it was made at, and a later question is not given its
    // result or the other way round.
    assert_eq!(by_group(first_snapshot.db(), ItemGroup::Fruit), 2);
    assert_eq!(by_group(first_snapshot.db(), ItemGroup::Vegetable), 0);
}

#[test]
fn a_restored_file_is_never_answered_from_a_result_made_for_the_file_before() {
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    // The file as it is after بکری 1, kept aside.
    let kept = tempfile::tempdir().unwrap();
    let snapshot = kept.path().join("after_bikri_1.db");
    raw(&w.dir)
        .execute("VACUUM INTO ?1", [snapshot.to_string_lossy().as_ref()])
        .unwrap();
    // بکری 2 is saved and every kept result is made for the file with two sellers.
    save_ok(&w, &event2(&w), false);
    assert_eq!(kept_counts(&w), (2, 2, 1));
    // The file of the first بکری is put back: nothing made for the other is used.
    w.env.ctx.db().replace_contents_from(&snapshot).unwrap();
    assert_eq!(kept_counts(&w), (1, 1, 1));
}

// ---- Fixtures for the screens (plan 77) ----
//
// Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is read
// back into the real input type before it is used) and `<name>.json` is the answer the real engine
// gives for it on the worked day (Events 1 to 5). The frontend copies the files. The test fails if a
// file differs from the real answer; after a deliberate change, write them again with
// `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test golden_day plan77_reads::fixtures`.
mod fixtures {
    use std::path::{Path, PathBuf};

    use mandi_reports::analytics::AnalyticsLinesPage;
    use serde::Serialize;
    use serde_json::{Value, json};

    use super::*;

    const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

    fn folder() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    fn write_or_check(file: &str, text: &str) {
        let path = folder().join(file);
        if std::env::var_os(WRITE_VARIABLE).is_some() {
            std::fs::create_dir_all(folder()).unwrap();
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

    fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
        write_or_check(
            &format!("{name}_request.json"),
            &(serde_json::to_string_pretty(request).unwrap() + "\n"),
        );
        write_or_check(
            &format!("{name}.json"),
            &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
        );
    }

    fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
        serde_json::from_value(request.clone()).unwrap()
    }

    /// The analytics filters with every key, as the frontend sends them.
    fn afilters(extra: Value) -> Value {
        let mut base = json!({
            "period": null, "from": "", "to": "", "group_by": null, "item_id": null,
            "seller_id": null, "buyer": null, "city_id": null, "adda_id": null,
            "side": "both", "compare": "off", "partner_id": null, "trip_id": null,
            "goods": "all", "trip_view": null, "garden_id": null
        });
        if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
            for (k, v) in extra {
                base.insert(k.clone(), v.clone());
            }
        }
        base
    }

    fn analytics(w: &support::World, name: &str, request: Value) -> AnalyticsPage {
        let input: AnalyticsRunInput = typed(&request);
        let page = analytics_run(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &page);
        page
    }

    fn lines(w: &support::World, name: &str, request: Value) -> AnalyticsLinesPage {
        let input: AnalyticsLinesInput = typed(&request);
        let page = analytics_lines(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &page);
        page
    }

    fn month(w: &support::World, name: &str, request: Value) {
        let input: CommissionMonthInput = typed(&request);
        let page = dashboard::commission_month(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &page);
    }

    #[test]
    fn fixtures_are_the_real_answers() {
        let w = worked_day();
        events_1_to_4(&w);
        let electricity = expense_head(&w, "بجلی", "Electricity");
        event5(&w, electricity);
        let rahim = json!({ "side": "seller", "account_id": w.rahim });
        let akram = json!({ "side": "buyer", "account_id": w.akram });
        let shahid = json!({ "side": "buyer", "account_id": w.chukta_khata,
            "chukta_person_key": shahid_key(&w) });

        // The catalogue of the screens: thirteen views.
        write_or_check(
            "analytics_catalog.json",
            &(serde_json::to_string_pretty(&screen_catalog()).unwrap() + "\n"),
        );

        // Views L and M, their lines, and a grouped view F.
        let october = afilters(json!({ "from": "01-10-2026", "to": "31-10-2026" }));
        let l = analytics(
            &w,
            "analytics_run_l",
            json!({ "view": "L", "filters": october, "offset": 0, "limit": 0 }),
        );
        let banana = l.columns[1].drill.clone();
        let akram_row = l
            .rows
            .iter()
            .find(|r| matches!(&r.cells[0], Cell::Party { party } if party.account_id == w.akram))
            .and_then(|r| r.key.clone());
        lines(
            &w,
            "analytics_lines_l_akram_banana",
            json!({ "view": "L", "filters": october, "row_key": akram_row,
                "column_key": banana, "offset": 0, "limit": 0 }),
        );
        let m = analytics(
            &w,
            "analytics_run_m",
            json!({ "view": "M", "filters": october, "offset": 0, "limit": 0 }),
        );
        lines(
            &w,
            "analytics_lines_m_electricity_october",
            json!({ "view": "M", "filters": october, "row_key": m.rows[0].key,
                "column_key": m.columns[1].drill, "offset": 0, "limit": 0 }),
        );
        // The partner part: each partner's own kharch, and the entries behind it.
        let kharch = afilters(
            json!({ "from": "01-10-2026", "to": "31-10-2026", "expense_part": "partner" }),
        );
        let mp = analytics(
            &w,
            "analytics_run_m_partners",
            json!({ "view": "M", "filters": kharch, "offset": 0, "limit": 0 }),
        );
        lines(
            &w,
            "analytics_lines_m_partner_alif",
            json!({ "view": "M", "filters": kharch, "row_key": mp.rows[0].key,
                "column_key": mp.columns[1].drill, "offset": 0, "limit": 0 }),
        );
        analytics(
            &w,
            "analytics_run_m_by_day",
            json!({ "view": "M", "filters": afilters(json!({ "from": "01-10-2026",
                "to": "31-10-2026", "group_by": "day" })), "offset": 0, "limit": 0 }),
        );
        analytics(
            &w,
            "analytics_run_f_by_month",
            json!({ "view": "F", "filters": afilters(json!({ "from": "01-10-2026",
                "to": "31-10-2026", "group_by": "month" })), "offset": 0, "limit": 0 }),
        );
        // The sale views with their rows grouped by month: the period column comes first and the
        // item columns of E and L stay under their own headings.
        for view in ["A", "B", "C", "E", "L"] {
            analytics(
                &w,
                &format!("analytics_run_{}_by_month", view.to_lowercase()),
                json!({ "view": view, "filters": afilters(json!({ "from": "01-10-2026",
                    "to": "31-10-2026", "group_by": "month" })), "offset": 0, "limit": 0 }),
            );
        }

        // The commission page: a seller, a buyer, a چکتا buyer, a year and a day.
        month(
            &w,
            "dashboard_commission_month_seller_rahim",
            json!({ "month": null, "period": { "kind": "month", "month": "2026-10" },
                "party": rahim, "offset": null, "limit": null }),
        );
        month(
            &w,
            "dashboard_commission_month_buyer_akram",
            json!({ "month": null, "period": { "kind": "month", "month": "2026-10" },
                "party": akram, "offset": null, "limit": null }),
        );
        month(
            &w,
            "dashboard_commission_month_buyer_shahid",
            json!({ "month": null, "period": { "kind": "month", "month": "2026-10" },
                "party": shahid, "offset": null, "limit": null }),
        );
        month(
            &w,
            "dashboard_commission_month_year_rahim",
            json!({ "month": null, "period": { "kind": "year", "year": 2026 },
                "party": rahim, "offset": null, "limit": null }),
        );
        month(
            &w,
            "dashboard_commission_month_range_shop",
            json!({ "month": null, "period": { "kind": "range", "from": "01-10-2026",
                "to": "10-10-2026" }, "party": null, "offset": 0, "limit": 25 }),
        );
        let request = json!({ "date": "2026-10-10", "offset": 0, "limit": 0, "party": rahim });
        let input: CommissionDayInput = typed(&request);
        let day = dashboard::commission_day(w.env.ctx.db(), &input).unwrap();
        pair("dashboard_commission_day_seller_rahim", &request, &day);
        let request = json!({ "month": null, "period": { "kind": "month", "month": "2026-10" },
            "party": rahim, "language": "en", "paper": null });
        let input: CommissionPrintInput = typed(&request);
        let print = dashboard::commission_month_print_data(w.env.ctx.db(), &input, true).unwrap();
        pair(
            "dashboard_commission_print_data_seller_en",
            &request,
            &print,
        );
    }
}
