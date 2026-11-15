//! Plan 76 points 7 and 9 on the PRD worked day: the commission page and the simple buyer dues
//! page. Every expected number is the PRD's (4.4, 4.5, 4.9) and is never changed to suit the code.
use super::day_money::events_1_to_4;
use super::*;
use mandi_reports::dashboard::{
    self, BackupFacts, BuyerDuesInput, CommissionDayInput, CommissionMonthInput,
    CommissionPrintInput,
};
use mandi_services::core::auth::Language;
use mandi_services::core::paging::{SortDirection, SortInput};

fn month_of(w: &support::World, month: Option<&str>) -> dashboard::CommissionMonth {
    dashboard::commission_month(
        w.env.ctx.db(),
        &CommissionMonthInput {
            month: month.map(str::to_owned),
            ..CommissionMonthInput::default()
        },
    )
    .unwrap()
}

fn rupees_of(p: Paisa) -> i64 {
    p.as_paisa() / 100
}

#[test]
fn the_commission_page_of_the_worked_day() {
    let w = worked_day();
    events_1_to_4(&w);
    // Today is in October 2026, so no month means October.
    let m = month_of(&w, None);
    assert_eq!(m.month, "2026-10");
    assert_eq!(m.current_month, "2026-10");
    assert_eq!(m.first_month.as_deref(), Some("2026-10"));
    assert_eq!(m.rows.len(), 1);
    let r = &m.rows[0];
    assert_eq!(r.date, "2026-10-10");
    // PRD 4.9: seller 2,484, buyer 2,484, لاگا 480, total 5,448, shared 5,488 (5,448 + فون 40).
    assert_eq!(
        (
            rupees_of(r.seller_comm_paisa),
            rupees_of(r.buyer_comm_paisa),
            rupees_of(r.laga_paisa),
            rupees_of(r.total_comm_paisa),
            rupees_of(r.shared_income_paisa),
        ),
        (2_484, 2_484, 480, 5_448, 5_488)
    );
    // The partners add up to the shared income, not to the total commission.
    let shares: Vec<i64> = r
        .partner_shares
        .as_ref()
        .unwrap()
        .iter()
        .map(|s| rupees_of(s.share_paisa))
        .collect();
    assert_eq!(shares, vec![2_196, 1_646, 1_646]);
    assert_eq!(shares.iter().sum::<i64>(), rupees_of(r.shared_income_paisa));
    // The month totals row is the sum of the days.
    assert_eq!(m.totals.total_comm_paisa, r.total_comm_paisa);
    assert_eq!(m.totals.shared_income_paisa, r.shared_income_paisa);
    // The tile آج کا کمیشن agrees (PRD FR-6.1).
    let tiles = dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles;
    assert_eq!(tiles.commission_paisa, r.total_comm_paisa);
}

#[test]
fn the_commission_page_hides_partner_shares_and_refuses_bad_months() {
    let w = worked_day();
    events_1_to_4(&w);
    let mut m = month_of(&w, Some("2026-10"));
    m.hide_partners();
    assert!(m.partners_locked);
    assert!(m.rows.iter().all(|r| r.partner_shares.is_none()));
    // The shared income stays: it is the shop's.
    assert_eq!(rupees_of(m.rows[0].shared_income_paisa), 5_488);
    // Names are always sent.
    assert_eq!(m.partners.len(), 3);
    // A month with nothing is an empty page, a future month and a bad text are refused.
    let september = month_of(&w, Some("2026-09"));
    assert!(september.rows.is_empty());
    assert_eq!(rupees_of(september.totals.total_comm_paisa), 0);
    for (text, code) in [
        ("2026-11", ErrorCode::ErrDateInFuture),
        ("10-2026", ErrorCode::ErrFormatInvalid),
    ] {
        let err = dashboard::commission_month(
            w.env.ctx.db(),
            &CommissionMonthInput {
                month: Some(text.into()),
                ..CommissionMonthInput::default()
            },
        )
        .unwrap_err();
        assert!(
            format!("{err:?}").contains(&format!("{code:?}")),
            "{text}: {err:?}"
        );
    }
}

#[test]
fn a_cancelled_bikri_nets_out_of_its_day() {
    let w = worked_day();
    let first = save_ok(&w, &support::event1(&w), false);
    let before = month_of(&w, None);
    assert_eq!(rupees_of(before.rows[0].total_comm_paisa), 3_928);
    let detail = bikri::get(&w.env.ctx, first.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: first.bikri_id,
            version: detail.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // Every column nets to 0, so the day is left out.
    let after = month_of(&w, None);
    assert!(after.rows.is_empty(), "{:?}", after.rows);
}

#[test]
fn the_day_detail_lists_the_bikri_that_made_it() {
    let w = worked_day();
    let first = save_ok(&w, &support::event1(&w), false);
    save_ok(&w, &event2(&w), false);
    let day = dashboard::commission_day(
        w.env.ctx.db(),
        &CommissionDayInput {
            date: "2026-10-10".into(),
            offset: 0,
            limit: 0,
            party: None,
        },
    )
    .unwrap();
    assert_eq!(day.total_count, 2);
    assert_eq!(day.entries.len(), 2);
    assert_eq!(day.entries[0].bikri_id, Some(first.bikri_id));
    assert_eq!(day.entries[0].bikri_number, Some(1));
    assert_eq!(day.entries[0].sellers.len(), 1);
    assert_eq!(day.entries[0].buyers.len(), 2);
    // Event 1: 1,764 + 1,764 + 400 = 3,928 and shared income 3,948; Event 2: 1,520 and 1,540.
    assert_eq!(rupees_of(day.entries[0].figures.total_comm_paisa), 3_928);
    assert_eq!(rupees_of(day.entries[0].figures.shared_income_paisa), 3_948);
    assert_eq!(rupees_of(day.entries[1].figures.total_comm_paisa), 1_520);
    // The postings add up to the day.
    assert_eq!(
        day.totals.total_comm_paisa,
        month_of(&w, None).rows[0].total_comm_paisa
    );
    // A page of one.
    let page = dashboard::commission_day(
        w.env.ctx.db(),
        &CommissionDayInput {
            date: "10-10-2026".into(),
            offset: 1,
            limit: 1,
            party: None,
        },
    )
    .unwrap();
    assert_eq!(page.entries.len(), 1);
    assert_eq!(page.total_count, 2);
}

#[test]
fn the_commission_print_has_every_row_and_the_partner_columns_when_open() {
    let w = worked_day();
    events_1_to_4(&w);
    let input = CommissionPrintInput {
        month: None,
        period: None,
        party: None,
        language: Language::En,
        paper: None,
    };
    let open = dashboard::commission_month_print_data(w.env.ctx.db(), &input, true).unwrap();
    assert_eq!(open.columns.len(), 9);
    assert_eq!(open.column_labels.len(), 9);
    assert_eq!(open.rows.len(), 1);
    let locked = dashboard::commission_month_print_data(w.env.ctx.db(), &input, false).unwrap();
    assert_eq!(locked.columns.len(), 6);
}

fn dues_page(w: &support::World, input: &BuyerDuesInput) -> dashboard::BuyerDuesPage {
    dashboard::buyer_dues(w.env.ctx.db(), input).unwrap()
}

#[test]
fn the_buyer_dues_page_equals_the_tile_and_r20() {
    let w = worked_day();
    events_1_to_4(&w);
    let page = dues_page(&w, &BuyerDuesInput::default());
    // After Events 1 to 4: اکرم 2,880 and ارشد 18,684, total 21,564 (PRD 4.9).
    let rows: Vec<(String, i64, Option<String>)> = page
        .rows
        .iter()
        .map(|r| {
            (
                r.party.name_en.clone(),
                rupees_of(r.balance.amount_paisa),
                r.last_change_date.clone(),
            )
        })
        .collect();
    assert_eq!(page.total_count, 2);
    assert_eq!(rupees_of(page.total_paisa), 21_564);
    assert!(rows.contains(&("Akram Toll Tax".into(), 2_880, Some("2026-10-10".into()))));
    assert!(rows.contains(&("Arshad Multani".into(), 18_684, Some("2026-10-10".into()))));
    // The tile and the total of R20 are the same figure by construction.
    let tile = dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles
    .buyers_dues_paisa;
    assert_eq!(page.total_paisa, tile);
    // Search and sort. The total follows the search.
    let akram = dues_page(
        &w,
        &BuyerDuesInput {
            search: Some("akram".into()),
            ..BuyerDuesInput::default()
        },
    );
    assert_eq!(
        (akram.total_count, rupees_of(akram.total_paisa)),
        (1, 2_880)
    );
    let by_balance = dues_page(
        &w,
        &BuyerDuesInput {
            sort: Some(SortInput {
                column: "balance".into(),
                direction: SortDirection::Desc,
                language: None,
            }),
            ..BuyerDuesInput::default()
        },
    );
    assert_eq!(by_balance.rows[0].party.name_en, "Arshad Multani");
    // A page of one, with the total over all.
    let first = dues_page(
        &w,
        &BuyerDuesInput {
            limit: 1,
            ..BuyerDuesInput::default()
        },
    );
    assert_eq!(first.rows.len(), 1);
    assert_eq!(rupees_of(first.total_paisa), 21_564);
    // A column that does not exist is refused.
    let bad = dashboard::buyer_dues(
        w.env.ctx.db(),
        &BuyerDuesInput {
            sort: Some(SortInput {
                column: "colour".into(),
                direction: SortDirection::Asc,
                language: None,
            }),
            ..BuyerDuesInput::default()
        },
    );
    assert!(bad.is_err());
}

/// Plan 76 speed review: the day of the last change of a page of buyers is read in the same
/// snapshot as the balances. A page of any size, from any place, shows the same day for a buyer as
/// the page that is sorted by that day (which reads it for every buyer), with the same count and
/// total.
#[test]
fn a_page_of_buyer_dues_shows_the_same_last_change_day_as_the_full_read() {
    let w = worked_day();
    events_1_to_4(&w);
    let sorted_by_day = dues_page(
        &w,
        &BuyerDuesInput {
            sort: Some(SortInput {
                column: "last_change_date".into(),
                direction: SortDirection::Asc,
                language: None,
            }),
            ..BuyerDuesInput::default()
        },
    );
    assert_eq!(sorted_by_day.total_count, 2);
    let day_of = |name: &str| -> Option<String> {
        sorted_by_day
            .rows
            .iter()
            .find(|r| r.party.name_en == name)
            .and_then(|r| r.last_change_date.clone())
    };
    assert_eq!(day_of("Akram Toll Tax"), Some("2026-10-10".into()));
    for offset in [0, 1] {
        let page = dues_page(
            &w,
            &BuyerDuesInput {
                offset,
                limit: 1,
                ..BuyerDuesInput::default()
            },
        );
        // The count and the total are over every buyer, whatever the page.
        assert_eq!(page.total_count, 2, "offset {offset}");
        assert_eq!(rupees_of(page.total_paisa), 21_564, "offset {offset}");
        assert_eq!(page.rows.len(), 1, "offset {offset}");
        let row = &page.rows[0];
        assert_eq!(
            row.last_change_date,
            day_of(&row.party.name_en),
            "{} at offset {offset}",
            row.party.name_en
        );
    }
    // A page past the end is empty, with the same count and total.
    let past = dues_page(
        &w,
        &BuyerDuesInput {
            offset: 5,
            limit: 1,
            ..BuyerDuesInput::default()
        },
    );
    assert!(past.rows.is_empty());
    assert_eq!((past.total_count, rupees_of(past.total_paisa)), (2, 21_564));
}

// ---- Plan 76 points 2 and 3: the اگراہی sheet, the parchi and the overall sale print ----

use mandi_reports::reports::{
    CollectionSheetInput, CollectionSheetPrintInput, DaySlipsInput, SlipSections, collection_sheet,
    collection_sheet_print_data, day_slips,
};

fn sheet_of(w: &support::World, date: &str) -> mandi_reports::reports::CollectionSheet {
    collection_sheet(
        w.env.ctx.db(),
        &CollectionSheetInput {
            date: date.into(),
            ..CollectionSheetInput::default()
        },
    )
    .unwrap()
}

fn figures(row: &mandi_reports::reports::CollectionSheetRow) -> (i64, i64, i64, i64) {
    (
        rupees_of(row.previous_due.amount_paisa),
        rupees_of(row.bill_paisa),
        rupees_of(row.paid_paisa),
        rupees_of(row.total_due.amount_paisa),
    )
}

#[test]
fn the_collection_sheet_of_the_worked_day_and_the_next_day() {
    let w = worked_day();
    events_1_to_4(&w);
    // 10-10-26: the 10,000 وصولی of Event 3 is NOT on the sheet; it lowers the next day's previous
    // due. Plan 76 point 2: اکرم 0 + 12,880 - 0, ارشد 0 + 18,684 - 0, شاہد 0 + 12,800 - 12,800.
    let sheet = sheet_of(&w, "10-10-2026");
    let by_name: Vec<(String, (i64, i64, i64, i64))> = sheet
        .rows
        .iter()
        .map(|r| (r.party.name_en.clone(), figures(r)))
        .collect();
    assert_eq!(sheet.total_count, 3);
    assert!(by_name.contains(&("Akram Toll Tax".into(), (0, 12_880, 0, 12_880))));
    assert!(by_name.contains(&("Arshad Multani".into(), (0, 18_684, 0, 18_684))));
    assert!(by_name.contains(&("Shahid".into(), (0, 12_800, 12_800, 0))));
    let shahid = sheet
        .rows
        .iter()
        .find(|r| r.party.name_en == "Shahid")
        .unwrap();
    assert!(shahid.party.person_key.is_some());
    assert_eq!(shahid.lines.len(), 1);
    assert_eq!(
        (
            rupees_of(sheet.totals.previous_due.amount_paisa),
            rupees_of(sheet.totals.bill_paisa),
            rupees_of(sheet.totals.paid_paisa),
            rupees_of(sheet.totals.total_due.amount_paisa),
        ),
        (0, 44_364, 12_800, 31_564)
    );
    assert!(sheet.dead_totals.is_none());
    // The next day, with no sale: اکرم 2,880, ارشد 18,684, total 21,564 = R20 = the tile. شاہد is gone.
    set_today(&w.env, 2026, 10, 11);
    let next = sheet_of(&w, "11-10-2026");
    let names: Vec<String> = next.rows.iter().map(|r| r.party.name_en.clone()).collect();
    assert_eq!(next.total_count, 2, "{names:?}");
    let akram = next
        .rows
        .iter()
        .find(|r| r.party.name_en == "Akram Toll Tax")
        .unwrap();
    assert_eq!(figures(akram), (2_880, 0, 0, 2_880));
    assert_eq!(rupees_of(next.totals.total_due.amount_paisa), 21_564);
    let tile = dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles
    .buyers_dues_paisa;
    assert_eq!(next.totals.total_due.amount_paisa, tile);
    // A future day is refused; the print has every row in both languages.
    assert!(
        collection_sheet(
            w.env.ctx.db(),
            &CollectionSheetInput {
                date: "12-10-2026".into(),
                ..CollectionSheetInput::default()
            },
        )
        .is_err()
    );
    for language in [Language::Ur, Language::En] {
        let print = collection_sheet_print_data(
            w.env.ctx.db(),
            &CollectionSheetPrintInput {
                date: "10-10-2026".into(),
                search: None,
                sort: None,
                language,
                paper: None,
            },
        )
        .unwrap();
        assert_eq!(print.rows.len(), 3);
        assert_eq!(print.columns.len(), 7);
    }
}

#[test]
fn the_parchi_and_the_sale_print_are_the_r35_slips_of_the_day() {
    let w = worked_day();
    let first = save_ok(&w, &support::event1(&w), false);
    let second = save_ok(&w, &event2(&w), false);
    let input = |bikri: Option<i64>, sections| DaySlipsInput {
        date: "10-10-2026".into(),
        bikri_id: bikri,
        bikri_ids: None,
        buyer_ids: None,
        sections,
        language: Language::Ur,
        paper: None,
    };
    // The whole day: three buyers (the چکتا buyer too), two sellers.
    let day = day_slips(
        w.env.ctx.db(),
        &input(None, SlipSections::Both),
        "print-sale-title",
    )
    .unwrap();
    assert_eq!((day.buyers.len(), day.sellers.len()), (3, 2));
    assert_eq!(day.date, "2026-10-10");
    // The parchi are buyers only; one بکری narrows to its parties.
    let parchi = day_slips(
        w.env.ctx.db(),
        &input(None, SlipSections::Buyers),
        "print-parchi-title",
    )
    .unwrap();
    assert_eq!((parchi.buyers.len(), parchi.sellers.len()), (3, 0));
    let one = day_slips(
        w.env.ctx.db(),
        &input(Some(second.bikri_id), SlipSections::Both),
        "print-sale-title",
    )
    .unwrap();
    assert_eq!((one.buyers.len(), one.sellers.len()), (1, 1));
    let of_first = day_slips(
        w.env.ctx.db(),
        &input(Some(first.bikri_id), SlipSections::Both),
        "print-sale-title",
    )
    .unwrap();
    assert_eq!((of_first.buyers.len(), of_first.sellers.len()), (2, 1));
    // Each slip is the R35 slip: it has the goods rows and the summary lines of the day.
    assert!(
        day.buyers
            .iter()
            .all(|slip| !slip.rows.is_empty() && !slip.summary.is_empty())
    );
    assert!(day.sellers.iter().all(|slip| !slip.rows.is_empty()));
    // Named buyers only.
    let akram_only = day_slips(
        w.env.ctx.db(),
        &DaySlipsInput {
            buyer_ids: Some(vec![w.akram]),
            ..input(None, SlipSections::Buyers)
        },
        "print-parchi-title",
    )
    .unwrap();
    assert_eq!(akram_only.buyers.len(), 1);
}

// ---- Plan 76 point 11: a partner's statement, one month at a time ----

use mandi_services::core::statement::{
    PartnersStatementDayInput, PartnersStatementInput, PartnersStatementMonthInput,
};
use mandi_services::partners;

fn month_input(partner: i64, month: Option<&str>) -> PartnersStatementMonthInput {
    PartnersStatementMonthInput {
        partner_id: partner,
        month: month.map(str::to_owned),
    }
}

#[test]
fn partner_b_october_is_the_example_of_the_plan() {
    // Events 1 to 5 and E2E-35: ب invests 50,000 on 11-10-26.
    let w = super::partner_money::events_1_to_5_and_capital();
    let b = w.partners[1];
    let m = partners::statement_month(&w.env.ctx, &month_input(b, Some("2026-10"))).unwrap();
    assert_eq!(m.month, "2026-10");
    assert_eq!(m.first_month, "2026-10");
    assert_eq!(rupees_of(m.opening_total_paisa), 0);
    assert_eq!(m.days.len(), 2);
    let (d10, d11) = (&m.days[0], &m.days[1]);
    assert_eq!(d10.date, "2026-10-10");
    // 10-10-26: commission 1,184 + 462 = 1,646, expense -900, day net 746, کل رقم 746.
    assert_eq!(
        (
            rupees_of(d10.columns.commission_paisa),
            rupees_of(d10.columns.expense_paisa),
            rupees_of(d10.columns.net_paisa),
            rupees_of(d10.running_paisa),
        ),
        (1_646, -900, 746, 746)
    );
    // 11-10-26: capital +50,000, کل رقم 50,746.
    assert_eq!(
        (
            rupees_of(d11.columns.capital_paisa),
            rupees_of(d11.columns.net_paisa),
            rupees_of(d11.running_paisa),
        ),
        (50_000, 50_000, 50_746)
    );
    assert_eq!(rupees_of(m.totals.commission_paisa), 1_646);
    assert_eq!(rupees_of(m.totals.expense_paisa), -900);
    assert_eq!(rupees_of(m.totals.capital_paisa), 50_000);
    assert_eq!(rupees_of(m.totals.net_paisa), 50_746);
    assert_eq!(rupees_of(m.closing_total_paisa), 50_746);
    // The closing is his کل رقم on the overview and the last running total of R31 (E2E-37).
    let overview = partners::overview(&w.env.ctx).unwrap();
    let row = overview.iter().find(|r| r.partner_id == b).unwrap();
    assert_eq!(m.closing_total_paisa, row.total_paisa);
    let r31 = partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            partner_id: b,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap();
    assert_eq!(m.closing_total_paisa, r31.closing_total_paisa);
    // The current month is the default, and the month before has nothing and carries the opening.
    let default = partners::statement_month(&w.env.ctx, &month_input(b, None)).unwrap();
    assert_eq!(default.month, "2026-10");
    let september =
        partners::statement_month(&w.env.ctx, &month_input(b, Some("2026-09"))).unwrap();
    assert!(september.days.is_empty());
    assert_eq!(rupees_of(september.closing_total_paisa), 0);
    // A future month and a bad one are refused.
    assert!(partners::statement_month(&w.env.ctx, &month_input(b, Some("2026-11"))).is_err());
    assert!(partners::statement_month(&w.env.ctx, &month_input(b, Some("26-10"))).is_err());
}

#[test]
fn the_day_card_lists_every_movement_with_its_source() {
    let w = super::partner_money::events_1_to_5_and_capital();
    let b = w.partners[1];
    let day = partners::statement_day(
        &w.env.ctx,
        &PartnersStatementDayInput {
            partner_id: b,
            date: "2026-10-10".into(),
        },
    )
    .unwrap();
    assert_eq!(day.date, "2026-10-10");
    assert_eq!(rupees_of(day.opening_total_paisa), 0);
    assert_eq!(rupees_of(day.after_total_paisa), 746);
    // بکری 1: his 30% of 3,948 = 1,184; بکری 2: 30% of 1,540 = 462; بجلی -900.
    let lines: Vec<(i64, Option<i64>, Option<i64>)> = day
        .lines
        .iter()
        .map(|l| {
            let value = |name: &str| match l.values.get(name) {
                Some(mandi_services::core::error::ErrorValue::Number(n)) => Some(*n),
                _ => None,
            };
            (
                rupees_of(l.amount_paisa.unwrap_or(Paisa::ZERO)),
                value("base_paisa").map(|v| v / 100),
                value("share_bps"),
            )
        })
        .collect();
    assert!(
        lines.contains(&(1_184, Some(3_948), Some(3_000))),
        "{lines:?}"
    );
    assert!(
        lines.contains(&(462, Some(1_540), Some(3_000))),
        "{lines:?}"
    );
    assert!(lines.iter().any(|l| l.0 == -900));
    // Every line keeps its entry id and its heading so the card can open the entry.
    assert!(day.lines.iter().all(|l| l.entry_id.is_some()));
    // A day with nothing is an empty card, not an error.
    let empty = partners::statement_day(
        &w.env.ctx,
        &PartnersStatementDayInput {
            partner_id: b,
            date: "2026-10-09".into(),
        },
    )
    .unwrap();
    assert!(empty.lines.is_empty());
}

#[test]
fn the_overview_counts_the_open_gardens_and_trips() {
    let w = super::partner_money::events_1_to_5_and_capital();
    let overview = partners::overview(&w.env.ctx).unwrap();
    assert!(
        overview
            .iter()
            .all(|r| r.open_gardens_count == 0 && r.open_trips_count == 0)
    );
    let cards = partners::open_ventures(&w.env.ctx, w.partners[0]).unwrap();
    assert!(cards.is_empty());
    assert!(partners::open_ventures(&w.env.ctx, 99_999).is_err());
}

// ---- Plan 76 point 8: the money lists take a range and total it ----

#[test]
fn the_money_lists_take_a_first_and_last_day() {
    use mandi_services::core::capital::CapitalHistoryInput;
    use mandi_services::core::expenses::PartnerTakesInput;
    use mandi_services::core::lent::LentListInput;
    use mandi_services::{lent, vouchers};
    // Events 1 to 5 (الف took 2,000 on 10-10-26) and ب's 50,000 on 11-10-26.
    let w = super::partner_money::events_1_to_5_and_capital();
    let (a, b) = (w.partners[0], w.partners[1]);

    // Money taken by الف: one take dated 10-10-26.
    let takes = |from: Option<&str>, to: Option<&str>| {
        partners::takes(
            &w.env.ctx,
            &PartnerTakesInput {
                from: from.map(str::to_owned),
                to: to.map(str::to_owned),
                partner_id: a,
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: false,
                show_finished: true,
            },
        )
    };
    let all = takes(None, None).unwrap();
    assert_eq!(
        (all.total_count, rupees_of(all.totals.taken_paisa)),
        (1, 2_000)
    );
    // An empty text is no limit, exactly as null.
    assert_eq!(takes(Some(""), Some(" ")).unwrap().total_count, 1);
    let in_window = takes(Some("01-10-2026"), Some("10-10-2026")).unwrap();
    assert_eq!(
        (
            in_window.total_count,
            rupees_of(in_window.totals.taken_paisa)
        ),
        (1, 2_000)
    );
    let after = takes(Some("11-10-2026"), None).unwrap();
    assert_eq!(
        (
            after.total_count,
            after.rows.len(),
            rupees_of(after.totals.taken_paisa)
        ),
        (0, 0, 0)
    );
    assert_eq!(
        takes(Some("32-10-2026"), None).unwrap_err().code(),
        mandi_services::core::error::ErrorCode::ErrDateInvalid
    );

    // Capital of ب: 50,000 on 11-10-26. The balance now never changes with the range.
    let capital = |from: Option<&str>, to: Option<&str>| {
        partners::capital_history(
            &w.env.ctx,
            &CapitalHistoryInput {
                from: from.map(str::to_owned),
                to: to.map(str::to_owned),
                partner_id: b,
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap()
    };
    let whole = capital(None, None);
    assert_eq!(
        (
            whole.total_count,
            rupees_of(whole.opening_paisa),
            rupees_of(whole.balance_paisa)
        ),
        (1, 0, 50_000)
    );
    let before = capital(None, Some("10-10-2026"));
    assert_eq!(
        (
            before.total_count,
            rupees_of(before.totals.invested_paisa),
            rupees_of(before.balance_paisa)
        ),
        (0, 0, 50_000)
    );
    let later = capital(Some("12-10-2026"), None);
    assert_eq!(
        (
            later.total_count,
            rupees_of(later.opening_paisa),
            rupees_of(later.balance_paisa)
        ),
        (0, 50_000, 50_000)
    );
    let exact = capital(Some("11-10-2026"), Some("11-10-2026"));
    assert_eq!(
        (
            exact.total_count,
            rupees_of(exact.totals.invested_paisa),
            rupees_of(exact.rows[0].running_paisa)
        ),
        (1, 50_000, 50_000)
    );

    // Whatever the range: the opening plus the amounts of its rows is the last running balance,
    // and the balance now never changes (money agent, plan 76).
    for (from, to) in [
        (None, None),
        (Some("11-10-2026"), None),
        (Some("10-10-2026"), Some("11-10-2026")),
        (Some("12-10-2026"), None),
        (None, Some("10-10-2026")),
    ] {
        let h = capital(from, to);
        let sum: i64 = h.rows.iter().map(|r| r.amount_paisa.as_paisa()).sum();
        let last = h
            .rows
            .last()
            .map_or(h.opening_paisa.as_paisa(), |r| r.running_paisa.as_paisa());
        assert_eq!(h.opening_paisa.as_paisa() + sum, last, "{from:?} {to:?}");
        assert_eq!(h.balance_paisa, whole.balance_paisa, "{from:?} {to:?}");
    }

    // Money lent to اسلم: 5,000 on 10-10-26 and 1,000 on 11-10-26.
    let person = super::day_money::aslam(&w, None);
    for (amount, date) in [("5,000", "10-10-2026"), ("1,000", "11-10-2026")] {
        vouchers::save_money_lent(
            &w.env.ctx,
            &w.owner,
            &super::day_money::lent_input(person, amount, date, None),
        )
        .unwrap();
    }
    let loans = |from: Option<&str>, to: Option<&str>| {
        lent::list(
            &w.env.ctx,
            &LentListInput {
                from: from.map(str::to_owned),
                to: to.map(str::to_owned),
                person_id: person,
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: false,
                show_finished: false,
            },
        )
        .unwrap()
    };
    let all = loans(None, None);
    assert_eq!(
        (all.total_count, rupees_of(all.totals.lent_paisa)),
        (2, 6_000)
    );
    let eleventh = loans(Some("11-10-2026"), Some("11-10-2026"));
    assert_eq!(
        (
            eleventh.total_count,
            rupees_of(eleventh.totals.lent_paisa),
            rupees_of(eleventh.totals.open_paisa)
        ),
        (1, 1_000, 1_000)
    );
    let tenth = loans(None, Some("10-10-2026"));
    assert_eq!(
        (tenth.total_count, rupees_of(tenth.totals.lent_paisa)),
        (1, 5_000)
    );
    // The balance is his true balance whatever the range.
    assert_eq!(tenth.balance, all.balance);
}

// ---- Plan 76 point 8: the open bills, payables, advances and the find take a page ----

#[test]
fn the_voucher_helpers_and_the_find_take_a_page() {
    use mandi_services::core::bikri::BikriFindInput;
    use mandi_services::core::paging::PageInput;
    use mandi_services::core::vouchers::{AdvanceInput, OpenBillsInput, RemovedPaymentFields};
    use mandi_services::{assist, bikri, vouchers};
    let w = worked_day();
    events_1_to_4(&w);

    // اکرم: بکری 1 bill 12,880 with 10,000 paid, then two cash-lent bills of 1,000 and 2,000.
    for amount in ["1,000", "2,000"] {
        let input = super::payment_input(w.akram, amount, "نقد ادھار");
        vouchers::save_payment(&w.env.ctx, &w.owner, &input).unwrap();
    }
    let bills = |offset: u32, limit: u32| {
        vouchers::open_bills_page(
            &w.env.ctx,
            &OpenBillsInput {
                account_id: w.akram,
            },
            PageInput { offset, limit },
        )
        .unwrap()
    };
    let whole = bills(0, 0);
    assert_eq!(whole.total_count, 3);
    assert!(!whole.truncated);
    assert_eq!(rupees_of(whole.open_paisa), 5_880);
    let first = bills(0, 2);
    assert_eq!((first.bills.len(), first.truncated), (2, true));
    let second = bills(2, 2);
    assert_eq!((second.bills.len(), second.truncated), (1, false));
    // The count, the open total and the age of the oldest bill are of all his bills, whatever the
    // page; the pages put together are the whole.
    assert_eq!(
        (second.total_count, second.open_paisa, second.oldest_days),
        (3, whole.open_paisa, whole.oldest_days)
    );
    let together: Vec<i64> = first
        .bills
        .iter()
        .chain(&second.bills)
        .map(|b| b.line_id)
        .collect();
    assert_eq!(
        together,
        whole.bills.iter().map(|b| b.line_id).collect::<Vec<_>>()
    );
    // A page past the end is empty, and the same call without a page is as before.
    assert!(bills(5, 2).bills.is_empty());
    assert_eq!(
        vouchers::open_bills(
            &w.env.ctx,
            &OpenBillsInput {
                account_id: w.akram
            }
        )
        .unwrap(),
        whole
    );

    // رحیم's advance of 15,000 was adjusted by بکری 1. Two new ones are open, so the card pages.
    for amount in ["500", "700"] {
        vouchers::save_advance(
            &w.env.ctx,
            &w.owner,
            &AdvanceInput {
                confirm_cash_below_zero: false,
                date: "10-10-2026".into(),
                seller_account_id: w.rahim,
                amount: amount.into(),
                removed: RemovedPaymentFields::default(),
                expected_goods: String::new(),
                narration: String::new(),
            },
        )
        .unwrap();
    }
    let all = assist::advances_page(&w.env.ctx, w.rahim, PageInput::default()).unwrap();
    assert_eq!(
        (all.total_count, all.open.len(), all.truncated),
        (2, 2, false)
    );
    assert_eq!(rupees_of(all.open_paisa), 1_200);
    let one = assist::advances_page(
        &w.env.ctx,
        w.rahim,
        PageInput {
            offset: 0,
            limit: 1,
        },
    )
    .unwrap();
    assert_eq!(
        (one.open.len(), one.truncated, one.total_count),
        (1, true, 2)
    );
    // The total is of all of them, not of the page.
    assert_eq!(rupees_of(one.open_paisa), 1_200);
    let rest = assist::advances_page(
        &w.env.ctx,
        w.rahim,
        PageInput {
            offset: 1,
            limit: 1,
        },
    )
    .unwrap();
    assert_eq!((rest.open.len(), rest.truncated), (1, false));
    assert_ne!(one.open[0].open_paisa, rest.open[0].open_paisa);

    // کریم's payable (بکری 2, 10,160): one posting.
    // Payables take the page too: a page past the end is empty and the count stays whole.
    let payables = |offset: u32, limit: u32, settled: bool| {
        vouchers::open_payables_page(
            &w.env.ctx,
            &mandi_services::core::vouchers::OpenPayablesInput {
                seller_id: w.karim,
                include_settled: settled,
            },
            PageInput { offset, limit },
        )
        .unwrap()
    };
    let one = payables(0, 0, false);
    assert_eq!(
        (one.total_count, one.postings.len(), one.truncated),
        (1, 1, false)
    );
    let past = payables(1, 5, false);
    assert_eq!(
        (past.total_count, past.postings.len(), past.truncated),
        (1, 0, false)
    );
    assert_eq!(past.open_paisa, one.open_paisa);
    let settled_all = payables(0, 0, true);
    let settled_page = payables(0, 1, true);
    assert_eq!(settled_page.postings.len(), 1);
    assert_eq!(settled_page.truncated, settled_all.total_count > 1);

    // The find: the newest first, a page at a time. Two بکری on the worked day.
    let find = |offset: u32, limit: u32| {
        bikri::find_page(
            &w.env.ctx,
            &BikriFindInput::default(),
            PageInput { offset, limit },
        )
        .unwrap()
    };
    let all: Vec<i64> = find(0, 0).iter().map(|r| r.id).collect();
    assert_eq!(all.len(), 2);
    let (p1, p2) = (find(0, 1), find(1, 1));
    assert_eq!(
        (p1.len(), p2.len(), vec![p1[0].id, p2[0].id]),
        (1, 1, all.clone())
    );
    assert_eq!(
        bikri::find(&w.env.ctx, &BikriFindInput::default())
            .unwrap()
            .iter()
            .map(|r| r.id)
            .collect::<Vec<_>>(),
        all
    );
}

/// Reviewer finding 2 (plan 76 point 2, FR-19.9): a buyer who starts the day in credit (جمع) and buys
/// less than it ends the day in credit. He stays on the sheet with his side word, but the main total
/// is what buyers owe, so it stays R20 and the tile; his total is in a second total row.
#[test]
fn a_buyer_who_ends_the_day_in_credit_is_listed_apart_and_the_main_total_stays_r20() {
    let w = worked_day();
    events_1_to_4(&w);
    // 10-10-26: ارشد (owes 18,684) pays 28,684, so he is 10,000 جمع. اکرم still owes 2,880.
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &super::receipt_input(w.arshad, "28,684"),
    )
    .unwrap();
    // 11-10-26: he buys 10 کیلا at 400 on credit: 4,000 + commission 240 + لاگا 80 = 4,320.
    set_today(&w.env, 2026, 10, 11);
    let mut draft = support::draft(vec![support::with_sales(
        support::bare_lot(&w, "lot-1", w.rahim, w.banana, "10"),
        vec![support::sale("s-1", w.arshad, "10", "400")],
    )]);
    draft.header.entry_date = "11-10-2026".into();
    save_ok(&w, &draft, false);
    let sheet = sheet_of(&w, "11-10-2026");
    let arshad = sheet
        .rows
        .iter()
        .find(|r| r.party.name_en == "Arshad Multani")
        .expect("he stays on the sheet");
    // previous 10,000 جمع + bill 4,320 - paid 0 = 5,680 جمع, with his side word.
    assert_eq!(
        (
            arshad.previous_due.side,
            rupees_of(arshad.previous_due.amount_paisa),
            rupees_of(arshad.bill_paisa),
            arshad.total_due.side,
            rupees_of(arshad.total_due.amount_paisa),
        ),
        (BalanceSide::Cr, 10_000, 4_320, BalanceSide::Cr, 5_680)
    );
    // The main total leaves him out: only اکرم (2,880). It is what the buyers owe, R20 and the tile.
    assert_eq!(
        (
            sheet.totals.total_due.side,
            rupees_of(sheet.totals.total_due.amount_paisa),
            rupees_of(sheet.totals.previous_due.amount_paisa),
        ),
        (BalanceSide::Dr, 2_880, 2_880)
    );
    let tile = dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles
    .buyers_dues_paisa;
    assert_eq!(sheet.totals.total_due.amount_paisa, tile);
    // His own total row: previous 10,000 جمع, bill 4,320, total due 5,680 جمع.
    let credit = sheet.credit_totals.as_ref().expect("a credit total row");
    assert_eq!(
        (
            credit.previous_due.side,
            rupees_of(credit.previous_due.amount_paisa),
            rupees_of(credit.bill_paisa),
            credit.total_due.side,
            rupees_of(credit.total_due.amount_paisa),
        ),
        (BalanceSide::Cr, 10_000, 4_320, BalanceSide::Cr, 5_680)
    );
    assert!(sheet.dead_totals.is_none());
    // The print has the second total row too, labelled for it, and every buyer is still a row.
    let print = collection_sheet_print_data(
        w.env.ctx.db(),
        &CollectionSheetPrintInput {
            date: "11-10-2026".into(),
            search: None,
            sort: None,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(print.rows.len(), 2);
    assert_eq!(
        print
            .extra_totals
            .iter()
            .map(|t| t.label_id.as_str())
            .collect::<Vec<_>>(),
        vec!["print-collection-sheet-credit-total"]
    );
    // A day with no one in credit has no such row (the worked day, as before).
}

/// Reviewer finding 1 (plan 76 point 3, FR-19.15): the slip of "this بکری" has only that بکری's lines.
/// Two بکری on one day for the same seller and the same buyer: بکری 1 has اکرم's 20 @ 600 (12,000), بکری 2
/// his 10 @ 500 (5,000). The slip of بکری 1 shows 12,000 and 29,400 (the seller's gross of بکری 1), the
/// whole-day slip 17,000 and 34,400. The previous and the new balance are the day's in both.
#[test]
fn the_slip_of_one_bikri_has_only_its_lines_and_the_balances_of_the_day() {
    use mandi_reports::reports::{PrintCell, ReportPrintData};
    let w = worked_day();
    let first = save_ok(&w, &support::event1(&w), false);
    let draft = support::draft(vec![support::with_sales(
        support::bare_lot(&w, "lot-2", w.rahim, w.banana, "10"),
        vec![support::sale("s-1", w.akram, "10", "500")],
    )]);
    let second = save_ok(&w, &draft, false);
    let slips = |bikri: Option<i64>| {
        day_slips(
            w.env.ctx.db(),
            &DaySlipsInput {
                date: "10-10-2026".into(),
                bikri_id: bikri,
                bikri_ids: None,
                buyer_ids: Some(vec![w.akram]),
                sections: SlipSections::Both,
                language: Language::En,
                paper: None,
            },
            "print-sale-title",
        )
        .unwrap()
    };
    let text = |slip: &ReportPrintData, id: &str| -> String {
        let line = slip
            .summary
            .iter()
            .find(|l| l.label_id == id)
            .unwrap_or_else(|| panic!("no line {id}"));
        match &line.cell {
            PrintCell::Money { text } | PrintCell::Balance { text, .. } => text.clone(),
            other => panic!("{id}: {other:?}"),
        }
    };
    let day = slips(None);
    let of_first = slips(Some(first.bikri_id));
    let of_second = slips(Some(second.bikri_id));
    // The buyer: his lines of that بکری only (rows are the goods rows), goods and total of it.
    let (d, f, s2) = (&day.buyers[0], &of_first.buyers[0], &of_second.buyers[0]);
    assert_eq!(text(d, "report-r35-goods"), "17,000");
    assert_eq!(text(f, "report-r35-goods"), "12,000");
    assert_eq!(text(s2, "report-r35-goods"), "5,000");
    assert!(f.rows.len() < d.rows.len() || d.rows.len() == 1);
    // Today's total of بکری 1: 12,000 + 720 + 160 = 12,880. The rest of the day is "other entries".
    assert_eq!(text(f, "report-r35-today-total"), "12,880");
    // Commission, لاگا and the quantity of the two بکری add up to the day's.
    for (id, first_part, second_part, day_total) in [
        ("report-r35-commission", "720", "300", "1,020"),
        ("report-r35-laga", "160", "80", "240"),
    ] {
        assert_eq!(text(f, id), first_part, "{id}");
        assert_eq!(text(s2, id), second_part, "{id}");
        assert_eq!(text(d, id), day_total, "{id}");
    }
    // The balances are the day's, the same on every slip of the buyer.
    for slip in [f, s2] {
        assert_eq!(
            text(slip, "report-r35-previous"),
            text(d, "report-r35-previous")
        );
        assert_eq!(
            text(slip, "report-r35-new-balance"),
            text(d, "report-r35-new-balance")
        );
        assert_eq!(
            text(slip, "report-r35-today-left"),
            text(d, "report-r35-today-left")
        );
    }
    // A بکری in which the party has nothing is refused, not an empty slip.
    let none = day_slips(
        w.env.ctx.db(),
        &DaySlipsInput {
            date: "10-10-2026".into(),
            bikri_id: Some(second.bikri_id),
            bikri_ids: None,
            buyer_ids: Some(vec![w.arshad]),
            sections: SlipSections::Buyers,
            language: Language::En,
            paper: None,
        },
        "print-sale-title",
    )
    .unwrap();
    assert!(none.buyers.is_empty(), "ارشد has no line in بکری 2");
    // The seller: his lots of that بکری only. Gross 29,400 on بکری 1, 5,000 on بکری 2, 34,400 the day.
    let (sd, sf, ss) = (&day.sellers[0], &of_first.sellers[0], &of_second.sellers[0]);
    assert_eq!(text(sd, "report-r35-gross"), "34,400");
    assert_eq!(text(sf, "report-r35-gross"), "29,400");
    assert_eq!(text(ss, "report-r35-gross"), "5,000");
    for slip in [sf, ss] {
        assert_eq!(
            text(slip, "report-r35-new-balance"),
            text(sd, "report-r35-new-balance")
        );
        assert_eq!(
            text(slip, "report-r35-previous"),
            text(sd, "report-r35-previous")
        );
    }
}
