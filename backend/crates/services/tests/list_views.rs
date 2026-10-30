//! Plan 38 B2 slice (a) (PRD FR-19.11, FR-7.45, Code Standards B31, NFR-SPD-24): the lists of
//! the app can be SORTED by a column over the whole filter, hide their finished rows unless asked,
//! be searched, and give month headings with subtotals over the whole filter. One test for each
//! list; the rules they share:
//!
//! - a sort is done before the page is cut, so pages of a sorted list hold every row once;
//! - an unknown sort column is `ERR_OUT_OF_RANGE` on `sort.column`;
//! - `show_finished` false leaves finished rows out of the rows, the count AND the totals;
//! - months come only for a list in date order, and their subtotals add up to the totals.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::accounts::{AccountFilter, AccountListKind, AccountsListInput};
use mandi_services::core::arrivals::{
    ArrivalCancelInput, ArrivalInput, ArrivalStateFilter, ArrivalsList, ArrivalsListInput,
};
use mandi_services::core::bikri::{
    BikriDraft, BikriListInput, BikriListRow, BikriSaveInput, BikriSaved, BikriStatus, CancelInput,
    PaymentType, PukhtaInput, PukhtaRegister, PukhtaRegisterInput, PukhtaSide, StockListInput,
};
use mandi_services::core::capital::{CapitalHistoryInput, CapitalInput};
use mandi_services::core::cashbook::{Cashbook, CashbookInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::error::ErrorValue;
use mandi_services::core::expenses::{
    PartnerTakesInput, PersonalExpenseInput, PersonalReturnInput,
};
use mandi_services::core::fruit::{
    FruitCancelInput, FruitInput, FruitList, FruitListInput, FruitSaved, FruitSource,
};
use mandi_services::core::gardens::{GardenInput, GardensListInput};
use mandi_services::core::khata::{KhataCreateInput, KhataOutInput, KhataStatementInput};
use mandi_services::core::lent::{
    LentListInput, LentPersonCreateInput, LentReportInput, LentReturnMode, MoneyLentReturnInput,
};
use mandi_services::core::masters::{MasterKind, MastersList, MastersListInput};
use mandi_services::core::paging::{MonthSubtotal, SortDirection, SortInput};
use mandi_services::core::purchases::{
    PurchaseCancelInput, PurchaseInput, PurchaseStateFilter, PurchasesList, PurchasesListInput,
};
use mandi_services::core::rules::{
    ChargeMethod, ChargeSide, ChargeType, RuleInput, RulesFilter, RulesListInput,
};
use mandi_services::core::statement::PartnersStatementInput;
use mandi_services::core::translit::Kind;
use mandi_services::core::trips::{TripInput, TripsListInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesReportInput, ReceiptInput, RemovedPaymentFields, VoucherCancelInput,
    VoucherType, VouchersList, VouchersListInput,
};
use mandi_services::translit::NewPair;
use mandi_services::{
    ServiceError, accounts, bikri, cashbook, fruit, gardens, khata, lent, masters, partners,
    purchases, rules, translit, trips, vouchers,
};
use support::World;

fn sort(column: &str, direction: SortDirection) -> Option<SortInput> {
    Some(SortInput {
        language: None,
        column: column.into(),
        direction,
    })
}

fn month_figure(month: &MonthSubtotal, key: &str) -> i64 {
    month
        .figures
        .iter()
        .find(|f| f.key == key)
        .unwrap_or_else(|| panic!("no figure {key}"))
        .value
}

fn rejected_column(error: &ServiceError) -> bool {
    matches!(error, ServiceError::Rejected(r)
        if r.iter().any(|x| x.code == ErrorCode::ErrOutOfRange && x.field.as_deref() == Some("sort.column")))
}

// ---- بکری ----

/// A بکری on a day: 10 نگ from رحیم, 4 sold to اکرم at `600 + n`. Truck `TRK-<n>`.
fn bikri_on(w: &World, n: i64, date: &str) -> BikriSaved {
    let item = if n % 2 == 0 { w.apple } else { w.banana };
    let lot = support::with_sales(
        support::bare_lot(w, "lot-1", w.rahim, item, "10"),
        vec![support::sale("s-1", w.akram, "4", &format!("{}", 600 + n))],
    );
    let mut draft: BikriDraft = support::draft(vec![lot]);
    draft.header.entry_date = date.into();
    draft.header.truck_no = Some(format!("TRK-{n}"));
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
}

fn cancel(w: &World, saved: &BikriSaved) {
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: saved.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

fn bikri_list(w: &World, input: BikriListInput) -> mandi_services::core::bikri::BikriList {
    bikri::list(&w.env.ctx, &input).unwrap()
}

fn numbers(rows: &[BikriListRow]) -> Vec<i64> {
    rows.iter().map(|r| r.number).collect()
}

/// Five بکری: 1, 2, 3 on 10-10-26 and 4, 5 on 05-11-26; the third is cancelled.
fn bikri_world() -> World {
    let w = support::world();
    let saved: Vec<BikriSaved> = (1..=3).map(|n| bikri_on(&w, n, "10-10-2026")).collect();
    set_today(&w.env, 2026, 11, 5);
    bikri_on(&w, 4, "05-11-2026");
    bikri_on(&w, 5, "05-11-2026");
    cancel(&w, &saved[2]);
    w
}

#[test]
fn the_list_of_bikri_hides_the_cancelled_sorts_searches_and_gives_month_subtotals() {
    let w = bikri_world();
    // The default opens newest first and leaves the cancelled one out of rows, count and totals.
    let list = bikri_list(&w, BikriListInput::default());
    assert_eq!(numbers(&list.rows), [5, 4, 2, 1]);
    assert_eq!(list.total_count, 4);
    // With the tick, the cancelled one is back in the rows and the count; the sums still leave it
    // out (a cancelled بکری is not business).
    let all = bikri_list(
        &w,
        BikriListInput {
            show_finished: true,
            ..BikriListInput::default()
        },
    );
    assert_eq!(numbers(&all.rows), [5, 4, 3, 2, 1]);
    assert_eq!(all.total_count, 5);
    assert_eq!(all.totals, list.totals);
    // A status that is asked for decides, whatever the tick says.
    let cancelled = bikri_list(
        &w,
        BikriListInput {
            status: Some(BikriStatus::Cancelled),
            ..BikriListInput::default()
        },
    );
    assert_eq!(numbers(&cancelled.rows), [3]);

    // Sorting is over the whole filter: pages of a sorted list hold every row once, in order.
    for (column, direction, want) in [
        ("number", SortDirection::Asc, vec![1, 2, 4, 5]),
        ("number", SortDirection::Desc, vec![5, 4, 2, 1]),
        ("buyer_total", SortDirection::Desc, vec![5, 4, 2, 1]),
        ("buyer_total", SortDirection::Asc, vec![1, 2, 4, 5]),
        ("gross", SortDirection::Asc, vec![1, 2, 4, 5]),
        ("truck_no", SortDirection::Desc, vec![5, 4, 2, 1]),
        ("entry_date", SortDirection::Asc, vec![1, 2, 4, 5]),
    ] {
        let mut paged = Vec::new();
        for offset in [0, 1, 2, 3] {
            let page = bikri_list(
                &w,
                BikriListInput {
                    sort: sort(column, direction),
                    offset,
                    limit: 1,
                    ..BikriListInput::default()
                },
            );
            assert_eq!(page.total_count, 4, "{column}");
            paged.extend(numbers(&page.rows));
        }
        assert_eq!(paged, want, "sort by {column} {direction:?}");
    }
    let bad = bikri::list(
        &w.env.ctx,
        &BikriListInput {
            sort: sort("password", SortDirection::Asc),
            ..BikriListInput::default()
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad), "{bad:?}");

    // A search finds a number or the start of a truck number.
    let by_number = bikri_list(
        &w,
        BikriListInput {
            search: Some("4".into()),
            ..BikriListInput::default()
        },
    );
    assert_eq!(numbers(&by_number.rows), [4]);
    let by_truck = bikri_list(
        &w,
        BikriListInput {
            search: Some("trk".into()),
            ..BikriListInput::default()
        },
    );
    assert_eq!(by_truck.total_count, 4);

    // Month headings: October (1 and 2) and November (4 and 5), newest first, over the whole
    // filter, and they add up to the totals. The page does not change them.
    let months = |input: BikriListInput| bikri_list(&w, input).months;
    let grouped = months(BikriListInput {
        group_by_month: true,
        limit: 1,
        offset: 2,
        ..BikriListInput::default()
    });
    assert_eq!(
        grouped
            .iter()
            .map(|m| (m.month.as_str(), m.count))
            .collect::<Vec<_>>(),
        [("2026-11", 2), ("2026-10", 2)]
    );
    for key in ["gross", "net", "buyer_total", "qty_received", "qty_sold"] {
        let sum: i64 = grouped.iter().map(|m| month_figure(m, key)).sum();
        let total = match key {
            "gross" => list.totals.gross.as_paisa(),
            "net" => list.totals.net.as_paisa(),
            "buyer_total" => list.totals.buyer_total.as_paisa(),
            "qty_received" => list.totals.qty_received,
            _ => list.totals.qty_sold,
        };
        assert_eq!(sum, total, "months add up to the totals: {key}");
    }
    // Oldest first when the list is sorted by the day, ascending; none when sorted by another
    // column or when not asked for.
    let oldest = months(BikriListInput {
        group_by_month: true,
        sort: sort("entry_date", SortDirection::Asc),
        ..BikriListInput::default()
    });
    assert_eq!(
        oldest.iter().map(|m| m.month.as_str()).collect::<Vec<_>>(),
        ["2026-10", "2026-11"]
    );
    assert!(
        months(BikriListInput {
            group_by_month: true,
            sort: sort("gross", SortDirection::Desc),
            ..BikriListInput::default()
        })
        .is_empty()
    );
    assert!(months(BikriListInput::default()).is_empty());
}

// ---- Vouchers ----

fn advance_on(w: &World, date: &str, seller: i64, amount: &str, narration: &str) -> i64 {
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: false,
            date: date.into(),
            seller_account_id: seller,
            amount: amount.into(),
            removed: RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: narration.into(),
        },
    )
    .unwrap()
    .voucher_id
}

fn voucher_input(kind: Option<VoucherType>) -> VouchersListInput {
    VouchersListInput {
        voucher_type: kind,
        from: String::new(),
        to: String::new(),
        account_id: None,
        person_key: None,
        offset: 0,
        limit: 0,
        show_finished: false,
        search: None,
        sort: None,
        group_by_month: false,
    }
}

fn voucher_numbers(list: &VouchersList) -> Vec<i64> {
    list.rows.iter().map(|r| r.number).collect()
}

/// Every row of a sorted voucher list, read in pages of one: the sort is of the whole filter.
fn voucher_numbers_in_pages(w: &World, input: &VouchersListInput) -> Vec<i64> {
    let total = vouchers::list(&w.env.ctx, input).unwrap().total_count;
    (0..total)
        .map(|offset| {
            let page = vouchers::list(
                &w.env.ctx,
                &VouchersListInput {
                    offset: u32::try_from(offset).unwrap(),
                    limit: 1,
                    ..input.clone()
                },
            )
            .unwrap();
            assert_eq!(page.rows.len(), 1);
            page.rows[0].number
        })
        .collect()
}

#[test]
fn the_list_of_vouchers_hides_the_cancelled_sorts_searches_and_gives_month_subtotals() {
    let w = support::world();
    let a1 = advance_on(&w, "10-10-2026", w.rahim, "1,000", "بیج کی پیشگی");
    let a2 = advance_on(&w, "10-10-2026", w.karim, "2,000", "seed advance");
    advance_on(&w, "10-10-2026", w.rahim, "3,000", "");
    set_today(&w.env, 2026, 11, 5);
    advance_on(&w, "05-11-2026", w.karim, "4,000", "");
    advance_on(&w, "05-11-2026", w.rahim, "5,000", "season advance");
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "05-11-2026".into(),
            account_id: w.akram,
            person_key: None,
            amount: "700".into(),
            removed: RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: "cash received".into(),
        },
    )
    .unwrap();
    let row = vouchers::get(&w.env.ctx, a2).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: a2,
            version: row.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let _ = a1;
    let advances = Some(VoucherType::Advance);

    // The cancelled one steps aside from the rows, the count and the total; it is counted apart.
    let hidden = vouchers::list(&w.env.ctx, &voucher_input(advances)).unwrap();
    assert_eq!(voucher_numbers(&hidden), [1, 3, 4, 5]);
    assert_eq!(hidden.total_count, 4);
    assert_eq!(hidden.totals.cancelled_count, 1);
    let shown_input = VouchersListInput {
        show_finished: true,
        ..voucher_input(advances)
    };
    let shown = vouchers::list(&w.env.ctx, &shown_input).unwrap();
    assert_eq!(voucher_numbers(&shown), [1, 2, 3, 4, 5]);
    assert_eq!(shown.total_count, 5);
    assert_eq!(shown.totals.total_paisa, hidden.totals.total_paisa);
    let all = vouchers::list(&w.env.ctx, &voucher_input(None)).unwrap();
    assert_eq!(all.total_count, 5);
    assert_eq!(all.totals.total_paisa, None);

    // Sorts of the whole filter, read in pages of one.
    let sorted = |column: &str, direction| {
        voucher_numbers_in_pages(
            &w,
            &VouchersListInput {
                sort: sort(column, direction),
                ..voucher_input(advances)
            },
        )
    };
    assert_eq!(sorted("amount_paisa", SortDirection::Desc), [5, 4, 3, 1]);
    assert_eq!(sorted("amount_paisa", SortDirection::Asc), [1, 3, 4, 5]);
    // Rahim's three, then Karim's; equal accounts keep the natural order.
    assert_eq!(sorted("account", SortDirection::Asc), [1, 3, 5, 4]);
    assert_eq!(sorted("account", SortDirection::Desc), [4, 1, 3, 5]);
    assert_eq!(sorted("entry_date", SortDirection::Desc), [5, 4, 3, 1]);
    assert_eq!(sorted("entry_date", SortDirection::Asc), [1, 3, 4, 5]);
    assert_eq!(sorted("number", SortDirection::Desc), [5, 4, 3, 1]);
    let by_status_input = VouchersListInput {
        sort: sort("status", SortDirection::Asc),
        ..shown_input.clone()
    };
    assert_eq!(
        voucher_numbers_in_pages(&w, &by_status_input),
        [2, 1, 3, 4, 5]
    );
    // An unknown column is refused with its field.
    let bad = vouchers::list(
        &w.env.ctx,
        &VouchersListInput {
            sort: sort("narration", SortDirection::Asc),
            ..voucher_input(advances)
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad));

    // Search: a number, a word of the narration, the account name.
    let found = |text: &str, input: VouchersListInput| {
        let list = vouchers::list(
            &w.env.ctx,
            &VouchersListInput {
                search: Some(text.into()),
                ..input
            },
        )
        .unwrap();
        (voucher_numbers(&list), list.total_count)
    };
    assert_eq!(found("season", voucher_input(advances)), (vec![5], 1));
    assert_eq!(found("بیج", voucher_input(advances)), (vec![1], 1));
    assert_eq!(found("3", voucher_input(advances)), (vec![3], 1));
    assert_eq!(found("rahim", voucher_input(advances)), (vec![1, 3, 5], 3));
    assert_eq!(found("karim", shown_input.clone()), (vec![2, 4], 2));
    assert_eq!(found("karim", voucher_input(advances)), (vec![4], 1));
    assert_eq!(found("nothing here", voucher_input(None)), (vec![], 0));
    // A typed `%` is a plain character, not a wild card.
    assert_eq!(found("%", voucher_input(None)), (vec![], 0));

    // Months over the whole filter, oldest first; the amounts add up to the total.
    let monthly = |input: VouchersListInput| {
        vouchers::list(
            &w.env.ctx,
            &VouchersListInput {
                group_by_month: true,
                ..input
            },
        )
        .unwrap()
    };
    let list = monthly(voucher_input(advances));
    let months: Vec<(&str, i64)> = list
        .months
        .iter()
        .map(|m| (m.month.as_str(), m.count))
        .collect();
    assert_eq!(months, [("2026-10", 2), ("2026-11", 2)]);
    let month_total: i64 = list
        .months
        .iter()
        .map(|m| month_figure(m, "total_paisa"))
        .sum();
    assert_eq!(
        Some(month_total),
        list.totals.total_paisa.map(|p| p.as_paisa())
    );
    assert_eq!(month_figure(&list.months[0], "total_paisa"), 400_000);
    // The cancelled one is hidden from the rows and the count, but the month still says how many
    // it holds, and the months add up to the total.
    assert_eq!(month_figure(&list.months[0], "cancelled_count"), 1);
    assert_eq!(
        list.months
            .iter()
            .map(|m| month_figure(m, "cancelled_count"))
            .sum::<i64>(),
        list.totals.cancelled_count
    );
    // With the cancelled shown the October month counts it, and its money still leaves it out.
    let list = monthly(shown_input.clone());
    assert_eq!(list.months[0].count, 3);
    assert_eq!(month_figure(&list.months[0], "cancelled_count"), 1);
    assert_eq!(month_figure(&list.months[0], "total_paisa"), 400_000);
    // All types: no money figure for a month, only the counts.
    let list = monthly(voucher_input(None));
    assert_eq!(list.months.len(), 2);
    assert!(
        list.months[1]
            .figures
            .iter()
            .all(|f| f.key != "total_paisa")
    );
    assert_eq!(list.months[1].count, 3);
    // Newest first when sorted by the day, descending; no months for another sort.
    let list = monthly(VouchersListInput {
        sort: sort("entry_date", SortDirection::Desc),
        ..voucher_input(advances)
    });
    assert_eq!(list.months[0].month, "2026-11");
    let list = monthly(VouchersListInput {
        sort: sort("amount_paisa", SortDirection::Asc),
        ..voucher_input(advances)
    });
    assert!(list.months.is_empty());
    assert!(
        vouchers::list(&w.env.ctx, &voucher_input(advances))
            .unwrap()
            .months
            .is_empty()
    );
}

// ---- Purchases ----

fn purchase_on(
    w: &World,
    trip: i64,
    date: &str,
    item: i64,
    supplier: &str,
    qty: &str,
    rate: &str,
) -> mandi_services::core::purchases::PurchaseSaved {
    purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip),
            date: date.into(),
            supplier: Some(supplier.into()),
            item_id: Some(item),
            packing_id: Some(w.box_),
            qty: qty.into(),
            weight: None,
            rate: rate.into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
}

fn purchase_input_list(state: Option<PurchaseStateFilter>) -> PurchasesListInput {
    PurchasesListInput {
        state,
        ..PurchasesListInput::default()
    }
}

fn purchase_numbers(list: &PurchasesList) -> Vec<i64> {
    list.rows.iter().map(|r| r.number).collect()
}

fn purchase_numbers_in_pages(w: &World, input: &PurchasesListInput) -> Vec<i64> {
    let total = purchases::list(&w.env.ctx, input).unwrap().total_count;
    (0..total)
        .map(|offset| {
            let page = purchases::list(
                &w.env.ctx,
                &PurchasesListInput {
                    offset,
                    limit: 1,
                    ..input.clone()
                },
            )
            .unwrap();
            assert_eq!(page.rows.len(), 1);
            page.rows[0].number
        })
        .collect()
}

#[test]
fn the_list_of_purchases_hides_the_cancelled_sorts_searches_and_gives_month_subtotals() {
    let w = support::world();
    let [a, b, c] = w.partners;
    let share = |partner_id: i64, share: &str| VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    };
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: vec![share(a, "40"), share(b, "30"), share(c, "30")],
        },
    )
    .unwrap()
    .id;
    purchase_on(
        &w,
        trip,
        "10-10-2026",
        w.banana,
        "Orange Traders",
        "10",
        "500",
    );
    purchase_on(&w, trip, "10-10-2026", w.apple, "Ali Fruit", "20", "300");
    set_today(&w.env, 2026, 11, 5);
    purchase_on(
        &w,
        trip,
        "05-11-2026",
        w.banana,
        "Orange Traders",
        "5",
        "400",
    );
    let fourth = purchase_on(&w, trip, "05-11-2026", w.apple, "Ali Fruit", "8", "200");
    purchases::cancel(
        &w.env.ctx,
        &w.owner,
        &PurchaseCancelInput {
            id: fourth.id,
            version: fourth.version,
            reason: "wrong".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let plain = purchase_input_list(None);
    let shown = PurchasesListInput {
        show_finished: true,
        ..plain.clone()
    };

    // The cancelled one steps aside from the rows, the count and the totals, unless asked for.
    let hidden = purchases::list(&w.env.ctx, &plain).unwrap();
    assert_eq!(purchase_numbers(&hidden), [3, 2, 1]);
    assert_eq!(hidden.total_count, 3);
    let with = purchases::list(&w.env.ctx, &shown).unwrap();
    assert_eq!(purchase_numbers(&with), [4, 3, 2, 1]);
    assert_eq!(with.total_count, 4);
    assert_eq!(with.totals.total_cost_paisa, hidden.totals.total_cost_paisa);
    assert_eq!(hidden.totals.total_cost_paisa.as_paisa(), 1_300_000);
    // A chosen state wins over the default.
    let cancelled = purchases::list(
        &w.env.ctx,
        &purchase_input_list(Some(PurchaseStateFilter::Cancelled)),
    )
    .unwrap();
    assert_eq!(purchase_numbers(&cancelled), [4]);

    // Sorts of the whole filter, read in pages of one.
    let sorted = |column: &str, direction| {
        purchase_numbers_in_pages(
            &w,
            &PurchasesListInput {
                sort: sort(column, direction),
                ..plain.clone()
            },
        )
    };
    assert_eq!(sorted("total_cost_paisa", SortDirection::Asc), [3, 1, 2]);
    assert_eq!(sorted("total_cost_paisa", SortDirection::Desc), [2, 1, 3]);
    assert_eq!(sorted("qty", SortDirection::Desc), [2, 1, 3]);
    assert_eq!(sorted("entry_date", SortDirection::Asc), [1, 2, 3]);
    assert_eq!(sorted("entry_date", SortDirection::Desc), [3, 2, 1]);
    assert_eq!(sorted("supplier", SortDirection::Asc), [2, 3, 1]);
    let bad = purchases::list(
        &w.env.ctx,
        &PurchasesListInput {
            sort: sort("result_paisa", SortDirection::Asc),
            ..plain.clone()
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad));

    // Search: the supplier, the number, the item name.
    let found = |text: &str, input: &PurchasesListInput| {
        let list = purchases::list(
            &w.env.ctx,
            &PurchasesListInput {
                search: Some(text.into()),
                ..input.clone()
            },
        )
        .unwrap();
        (purchase_numbers(&list), list.total_count)
    };
    assert_eq!(found("orange", &plain), (vec![3, 1], 2));
    assert_eq!(found("ali", &plain), (vec![2], 1));
    assert_eq!(found("ali", &shown), (vec![4, 2], 2));
    // The trip's name holds the date, so a number is looked for among numbers that are not in it.
    assert_eq!(found("3", &plain), (vec![3], 1));
    assert_eq!(found("Apple", &plain), (vec![2], 1));
    assert_eq!(found("%", &plain), (vec![], 0));

    // Months over the whole filter, newest first like the list; they add up to the totals.
    let monthly = |input: &PurchasesListInput| {
        purchases::list(
            &w.env.ctx,
            &PurchasesListInput {
                group_by_month: true,
                ..input.clone()
            },
        )
        .unwrap()
    };
    let list = monthly(&plain);
    let months: Vec<(&str, i64)> = list
        .months
        .iter()
        .map(|m| (m.month.as_str(), m.count))
        .collect();
    assert_eq!(months, [("2026-11", 1), ("2026-10", 2)]);
    let month_total: i64 = list
        .months
        .iter()
        .map(|m| month_figure(m, "total_cost_paisa"))
        .sum();
    assert_eq!(month_total, list.totals.total_cost_paisa.as_paisa());
    let month_left: i64 = list
        .months
        .iter()
        .map(|m| month_figure(m, "qty_left"))
        .sum();
    assert_eq!(month_left, list.totals.qty_left);
    // With the cancelled shown, November counts it and its money still leaves it out.
    let list = monthly(&shown);
    assert_eq!(list.months[0].count, 2);
    assert_eq!(month_figure(&list.months[0], "total_cost_paisa"), 200_000);
    // Oldest first when sorted by the day, ascending; none for another sort.
    let list = monthly(&PurchasesListInput {
        sort: sort("entry_date", SortDirection::Asc),
        ..plain.clone()
    });
    assert_eq!(list.months[0].month, "2026-10");
    let list = monthly(&PurchasesListInput {
        sort: sort("qty", SortDirection::Asc),
        ..plain.clone()
    });
    assert!(list.months.is_empty());
}

// ---- Garden arrivals ----

fn arrive(
    w: &World,
    garden: i64,
    date: &str,
    item: i64,
    qty: &str,
    note: &str,
) -> mandi_services::core::arrivals::ArrivalRow {
    gardens::arrivals::save(
        &w.env.ctx,
        &w.owner,
        &ArrivalInput {
            garden_id: Some(garden),
            date: date.into(),
            sell_date: None,
            item_id: Some(item),
            packing_id: Some(w.box_),
            qty: qty.into(),
            weight: None,
            note: Some(note.into()),
        },
    )
    .unwrap()
    .row
}

fn arrival_numbers(list: &ArrivalsList) -> Vec<i64> {
    list.rows.iter().map(|r| r.number).collect()
}

fn arrival_numbers_in_pages(w: &World, input: &ArrivalsListInput) -> Vec<i64> {
    let total = gardens::arrivals::list(&w.env.ctx, input)
        .unwrap()
        .total_count;
    (0..total)
        .map(|offset| {
            let page = gardens::arrivals::list(
                &w.env.ctx,
                &ArrivalsListInput {
                    offset: u32::try_from(offset).unwrap(),
                    limit: 1,
                    ..input.clone()
                },
            )
            .unwrap();
            assert_eq!(page.rows.len(), 1);
            page.rows[0].number
        })
        .collect()
}

#[test]
fn the_list_of_garden_arrivals_hides_the_cancelled_sorts_searches_and_gives_month_subtotals() {
    let w = support::world();
    let [a, b, c] = w.partners;
    let garden = gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "Shahpur".into(),
                "Garden".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: [(a, "40"), (b, "30"), (c, "30")]
                .into_iter()
                .map(|(partner_id, share)| VenturePartnerInput {
                    partner_id,
                    share: share.into(),
                    source: MoneySource::Shop,
                })
                .collect(),
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row
    .id;
    arrive(&w, garden, "10-10-2026", w.banana, "10", "first load");
    arrive(&w, garden, "10-10-2026", w.apple, "20", "second load");
    set_today(&w.env, 2026, 11, 5);
    arrive(&w, garden, "05-11-2026", w.banana, "5", "third load");
    let fourth = arrive(&w, garden, "05-11-2026", w.apple, "8", "fourth load");
    gardens::arrivals::cancel(
        &w.env.ctx,
        &w.owner,
        &ArrivalCancelInput {
            id: fourth.id,
            version: fourth.version,
            reason: "wrong".into(),
        },
    )
    .unwrap();
    let plain = ArrivalsListInput::default();
    let shown = ArrivalsListInput {
        show_finished: true,
        ..plain.clone()
    };

    // The cancelled one steps aside from the rows, the count and the totals, unless asked for.
    let hidden = gardens::arrivals::list(&w.env.ctx, &plain).unwrap();
    assert_eq!(arrival_numbers(&hidden), [1, 2, 3]);
    assert_eq!(hidden.total_count, 3);
    assert_eq!(hidden.totals.qty, 35);
    let with = gardens::arrivals::list(&w.env.ctx, &shown).unwrap();
    assert_eq!(arrival_numbers(&with), [1, 2, 3, 4]);
    assert_eq!(with.total_count, 4);
    assert_eq!(with.totals.qty, 35);
    let cancelled = gardens::arrivals::list(
        &w.env.ctx,
        &ArrivalsListInput {
            state: Some(ArrivalStateFilter::Cancelled),
            ..plain.clone()
        },
    )
    .unwrap();
    assert_eq!(arrival_numbers(&cancelled), [4]);

    // Sorts of the whole filter, read in pages of one.
    let sorted = |column: &str, direction| {
        arrival_numbers_in_pages(
            &w,
            &ArrivalsListInput {
                sort: sort(column, direction),
                ..plain.clone()
            },
        )
    };
    assert_eq!(sorted("qty", SortDirection::Desc), [2, 1, 3]);
    assert_eq!(sorted("qty", SortDirection::Asc), [3, 1, 2]);
    assert_eq!(sorted("entry_date", SortDirection::Desc), [3, 2, 1]);
    assert_eq!(sorted("entry_date", SortDirection::Asc), [1, 2, 3]);
    assert_eq!(sorted("number", SortDirection::Desc), [3, 2, 1]);
    let bad = gardens::arrivals::list(
        &w.env.ctx,
        &ArrivalsListInput {
            sort: sort("commission_paisa", SortDirection::Asc),
            ..plain.clone()
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad));

    // Search: the note, the number, the item name.
    let found = |text: &str, input: &ArrivalsListInput| {
        let list = gardens::arrivals::list(
            &w.env.ctx,
            &ArrivalsListInput {
                search: Some(text.into()),
                ..input.clone()
            },
        )
        .unwrap();
        (arrival_numbers(&list), list.total_count)
    };
    assert_eq!(found("second", &plain), (vec![2], 1));
    assert_eq!(found("fourth", &plain), (vec![], 0));
    assert_eq!(found("fourth", &shown), (vec![4], 1));
    assert_eq!(found("3", &plain), (vec![3], 1));
    assert_eq!(found("Apple", &plain), (vec![2], 1));
    assert_eq!(found("%", &plain), (vec![], 0));

    // Months over the whole filter, oldest first like the list; they add up to the totals.
    let monthly = |input: &ArrivalsListInput| {
        gardens::arrivals::list(
            &w.env.ctx,
            &ArrivalsListInput {
                group_by_month: true,
                ..input.clone()
            },
        )
        .unwrap()
    };
    let list = monthly(&plain);
    let months: Vec<(&str, i64)> = list
        .months
        .iter()
        .map(|m| (m.month.as_str(), m.count))
        .collect();
    assert_eq!(months, [("2026-10", 2), ("2026-11", 1)]);
    let month_qty: i64 = list.months.iter().map(|m| month_figure(m, "qty")).sum();
    assert_eq!(month_qty, list.totals.qty);
    let month_left: i64 = list
        .months
        .iter()
        .map(|m| month_figure(m, "qty_left"))
        .sum();
    assert_eq!(month_left, list.totals.qty_left);
    // With the cancelled shown, November counts it and its qty still leaves it out.
    let list = monthly(&shown);
    assert_eq!(list.months[1].count, 2);
    assert_eq!(month_figure(&list.months[1], "qty"), 5);
    // Newest first when sorted by the day, descending; none for another sort.
    let list = monthly(&ArrivalsListInput {
        sort: sort("entry_date", SortDirection::Desc),
        ..plain.clone()
    });
    assert_eq!(list.months[0].month, "2026-11");
    let list = monthly(&ArrivalsListInput {
        sort: sort("qty", SortDirection::Asc),
        ..plain.clone()
    });
    assert!(list.months.is_empty());
}

// ---- Fruit taken ----

#[allow(clippy::too_many_arguments)]
fn fruit_on(
    w: &World,
    partner: Option<usize>,
    date: &str,
    item: i64,
    qty: &str,
    rate: &str,
    note: &str,
) -> FruitSaved {
    fruit::save(
        &w.env.ctx,
        &w.owner,
        &FruitInput {
            for_guests: partner.is_none(),
            partner_id: partner.map(|i| w.partners[i]),
            date: date.into(),
            source: FruitSource::OtherShop,
            lot_id: None,
            agent_account_id: Some(w.rahim),
            item_id: Some(item),
            packing_id: Some(w.box_),
            qty: qty.into(),
            weight: None,
            rate: rate.into(),
            note: if note.is_empty() {
                None
            } else {
                Some(note.into())
            },
            editing_id: None,
        },
    )
    .unwrap()
}

fn fruit_numbers(list: &FruitList) -> Vec<i64> {
    list.rows.iter().map(|r| r.number).collect()
}

fn fruit_numbers_in_pages(w: &World, input: &FruitListInput) -> Vec<i64> {
    let total = fruit::list(&w.env.ctx, input).unwrap().total_count;
    (0..total)
        .map(|offset| {
            let page = fruit::list(
                &w.env.ctx,
                &FruitListInput {
                    offset: u32::try_from(offset).unwrap(),
                    limit: 1,
                    ..input.clone()
                },
            )
            .unwrap();
            assert_eq!(page.rows.len(), 1);
            page.rows[0].number
        })
        .collect()
}

#[test]
fn the_list_of_fruit_hides_the_cancelled_sorts_searches_and_gives_month_subtotals() {
    let w = support::world();
    fruit_on(&w, Some(0), "10-10-2026", w.banana, "2", "580", "home");
    fruit_on(&w, None, "10-10-2026", w.apple, "3", "1,000", "");
    set_today(&w.env, 2026, 11, 5);
    fruit_on(&w, Some(1), "05-11-2026", w.banana, "4", "500", "");
    let fourth = fruit_on(&w, None, "05-11-2026", w.apple, "1", "900", "");
    fruit::cancel(
        &w.env.ctx,
        &w.owner,
        &FruitCancelInput {
            id: fourth.id,
            version: fourth.version,
            reason: "غلط اندراج".into(),
        },
    )
    .unwrap();
    let plain = FruitListInput::default();
    let shown = FruitListInput {
        show_finished: true,
        ..plain.clone()
    };

    // The cancelled one steps aside from the rows, the count and the totals, unless asked for.
    let hidden = fruit::list(&w.env.ctx, &plain).unwrap();
    assert_eq!(fruit_numbers(&hidden), [3, 2, 1]);
    assert_eq!(hidden.total_count, 3);
    assert_eq!(hidden.totals.qty, 9);
    assert_eq!(hidden.totals.value_paisa.as_paisa(), 616_000);
    let with = fruit::list(&w.env.ctx, &shown).unwrap();
    assert_eq!(fruit_numbers(&with), [4, 3, 2, 1]);
    assert_eq!(with.total_count, 4);
    assert_eq!(with.totals.value_paisa, hidden.totals.value_paisa);
    // A chosen status wins over the default.
    let cancelled = fruit::list(
        &w.env.ctx,
        &FruitListInput {
            status: Some(mandi_services::core::fruit::FruitStatus::Cancelled),
            ..plain.clone()
        },
    )
    .unwrap();
    assert_eq!(fruit_numbers(&cancelled), [4]);

    // Sorts of the whole filter, read in pages of one.
    let sorted = |column: &str, direction| {
        fruit_numbers_in_pages(
            &w,
            &FruitListInput {
                sort: sort(column, direction),
                ..plain.clone()
            },
        )
    };
    assert_eq!(sorted("value_paisa", SortDirection::Asc), [1, 3, 2]);
    assert_eq!(sorted("value_paisa", SortDirection::Desc), [2, 3, 1]);
    assert_eq!(sorted("qty", SortDirection::Desc), [3, 2, 1]);
    assert_eq!(sorted("entry_date", SortDirection::Asc), [1, 2, 3]);
    assert_eq!(sorted("entry_date", SortDirection::Desc), [3, 2, 1]);
    let bad = fruit::list(
        &w.env.ctx,
        &FruitListInput {
            sort: sort("lot", SortDirection::Asc),
            ..plain.clone()
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad));

    // Search: the note, the number, the item name, the credited account.
    let found = |text: &str, input: &FruitListInput| {
        let list = fruit::list(
            &w.env.ctx,
            &FruitListInput {
                search: Some(text.into()),
                ..input.clone()
            },
        )
        .unwrap();
        (fruit_numbers(&list), list.total_count)
    };
    assert_eq!(found("home", &plain), (vec![1], 1));
    assert_eq!(found("3", &plain), (vec![3], 1));
    assert_eq!(found("Apple", &plain), (vec![2], 1));
    assert_eq!(found("Apple", &shown), (vec![4, 2], 2));
    assert_eq!(found("Rahim", &plain), (vec![3, 2, 1], 3));
    assert_eq!(found("%", &plain), (vec![], 0));

    // Months over the whole filter, newest first like the list; they add up to the totals.
    let monthly = |input: &FruitListInput| {
        fruit::list(
            &w.env.ctx,
            &FruitListInput {
                group_by_month: true,
                ..input.clone()
            },
        )
        .unwrap()
    };
    let list = monthly(&plain);
    let months: Vec<(&str, i64)> = list
        .months
        .iter()
        .map(|m| (m.month.as_str(), m.count))
        .collect();
    assert_eq!(months, [("2026-11", 1), ("2026-10", 2)]);
    let month_qty: i64 = list.months.iter().map(|m| month_figure(m, "qty")).sum();
    assert_eq!(month_qty, list.totals.qty);
    let month_value: i64 = list
        .months
        .iter()
        .map(|m| month_figure(m, "value_paisa"))
        .sum();
    assert_eq!(month_value, list.totals.value_paisa.as_paisa());
    // With the cancelled shown, November counts it and its value still leaves it out.
    let list = monthly(&shown);
    assert_eq!(list.months[0].count, 2);
    assert_eq!(month_figure(&list.months[0], "value_paisa"), 200_000);
    // Oldest first when sorted by the day, ascending; none for another sort.
    let list = monthly(&FruitListInput {
        sort: sort("entry_date", SortDirection::Asc),
        ..plain.clone()
    });
    assert_eq!(list.months[0].month, "2026-10");
    let list = monthly(&FruitListInput {
        sort: sort("qty", SortDirection::Asc),
        ..plain.clone()
    });
    assert!(list.months.is_empty());
}

// ---- The one-time register (پختہ کھاتہ) ----

fn pukhta_on(
    w: &World,
    n: i64,
    date: &str,
    name_ur: &str,
    name_en: &str,
    phone: &str,
) -> BikriSaved {
    let mut line = support::sale("s-1", w.pukhta_buyer, "2", &format!("{}", 1_000 + n));
    line.pukhta = Some(PukhtaInput {
        name_ur: name_ur.into(),
        name_en: name_en.into(),
        phone: phone.into(),
    });
    line.payment_type = Some(PaymentType::Credit);
    let lot = support::with_sales(
        support::bare_lot(w, "lot-1", w.karim, w.apple, "2"),
        vec![line],
    );
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = date.into();
    draft.header.truck_no = Some(format!("PK-{n}"));
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
}

fn register_numbers(list: &PukhtaRegister) -> Vec<i64> {
    list.rows.iter().map(|r| r.bikri_number).collect()
}

#[test]
fn the_pukhta_register_hides_the_cancelled_sorts_searches_and_gives_month_subtotals() {
    let w = support::world();
    pukhta_on(&w, 1, "10-10-2026", "شاہد", "Shahid", "0300-0000001");
    pukhta_on(&w, 2, "10-10-2026", "ناصر", "Nasir", "0300-0000002");
    set_today(&w.env, 2026, 11, 5);
    pukhta_on(&w, 3, "05-11-2026", "بشیر", "Bashir", "0300-0000003");
    let fourth = pukhta_on(&w, 4, "05-11-2026", "زاہد", "Zahid", "0300-0000004");
    cancel(&w, &fourth);
    let buyers = PukhtaRegisterInput {
        side: Some(PukhtaSide::Buyer),
        ..PukhtaRegisterInput::default()
    };
    let shown = PukhtaRegisterInput {
        show_finished: true,
        ..buyers.clone()
    };
    let read = |input: &PukhtaRegisterInput| bikri::register(&w.env.ctx, input).unwrap();

    // The lines of the cancelled بکری step aside from the rows and the count, unless asked for.
    let hidden = read(&buyers);
    assert_eq!(register_numbers(&hidden), [1, 2, 3]);
    assert_eq!(hidden.total_count, 3);
    let with = read(&shown);
    assert_eq!(register_numbers(&with), [1, 2, 3, 4]);
    assert_eq!(with.total_count, 4);
    assert_eq!(with.totals, hidden.totals);

    // Sorts of the whole filter.
    let sorted = |column: &str, direction| {
        register_numbers(&read(&PukhtaRegisterInput {
            sort: sort(column, direction),
            ..buyers.clone()
        }))
    };
    assert_eq!(sorted("amount", SortDirection::Desc), [3, 2, 1]);
    assert_eq!(sorted("amount", SortDirection::Asc), [1, 2, 3]);
    assert_eq!(sorted("name_en", SortDirection::Asc), [3, 2, 1]);
    assert_eq!(sorted("entry_date", SortDirection::Desc), [3, 2, 1]);
    assert_eq!(sorted("entry_date", SortDirection::Asc), [1, 2, 3]);
    let bad = bikri::register(
        &w.env.ctx,
        &PukhtaRegisterInput {
            sort: sort("person_key", SortDirection::Asc),
            ..buyers.clone()
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad));
    // A page of a sorted register is a slice of the whole sorted list.
    let second = read(&PukhtaRegisterInput {
        sort: sort("amount", SortDirection::Desc),
        offset: 1,
        limit: 1,
        ..buyers.clone()
    });
    assert_eq!(register_numbers(&second), [2]);

    // Search: the name, the phone, the item.
    let found = |text: &str, input: &PukhtaRegisterInput| {
        register_numbers(&read(&PukhtaRegisterInput {
            search: Some(text.into()),
            ..input.clone()
        }))
    };
    assert_eq!(found("nasir", &buyers), [2]);
    assert_eq!(found("0300-0000003", &buyers), [3]);
    assert_eq!(found("Apple", &buyers), [1, 2, 3]);
    assert_eq!(found("zahid", &buyers), Vec::<i64>::new());
    assert_eq!(found("zahid", &shown), [4]);

    // Months over the whole filter, oldest first; they add up to the totals.
    let monthly = |input: &PukhtaRegisterInput| {
        read(&PukhtaRegisterInput {
            group_by_month: true,
            ..input.clone()
        })
    };
    let list = monthly(&buyers);
    let months: Vec<(&str, i64)> = list
        .months
        .iter()
        .map(|m| (m.month.as_str(), m.count))
        .collect();
    assert_eq!(months, [("2026-10", 2), ("2026-11", 1)]);
    let sum_of = |key: &str| -> i64 { list.months.iter().map(|m| month_figure(m, key)).sum() };
    assert_eq!(Some(sum_of("qty")), list.totals.qty);
    assert_eq!(
        Some(sum_of("amount")),
        list.totals.amount.map(|p| p.as_paisa())
    );
    assert_eq!(Some(sum_of("open")), list.totals.open.map(|p| p.as_paisa()));
    // With the cancelled shown, November counts it and its money still leaves it out.
    let list = monthly(&shown);
    assert_eq!(list.months[1].count, 2);
    let november_hidden = monthly(&buyers).months[1].clone();
    assert_eq!(
        month_figure(&list.months[1], "amount"),
        month_figure(&november_hidden, "amount")
    );
    // No side chosen: the lines are mixed, so a month has its count and no money figure.
    let list = monthly(&PukhtaRegisterInput::default());
    assert_eq!(list.months.len(), 2);
    assert!(list.months.iter().all(|m| m.figures.is_empty()));
    // Newest first when sorted by the day, descending; none for another sort.
    let list = monthly(&PukhtaRegisterInput {
        sort: sort("entry_date", SortDirection::Desc),
        ..buyers.clone()
    });
    assert_eq!(list.months[0].month, "2026-11");
    let list = monthly(&PukhtaRegisterInput {
        sort: sort("qty", SortDirection::Asc),
        ..buyers.clone()
    });
    assert!(list.months.is_empty());
}

// ---- Accounts ----

#[test]
fn the_list_of_accounts_sorts_the_whole_filter_by_a_column() {
    let w = support::world();
    // Rahim gets 3,000 of advance and Karim 1,000: their balances are debits.
    advance_on(&w, "10-10-2026", w.rahim, "3,000", "");
    advance_on(&w, "10-10-2026", w.karim, "1,000", "");
    let list = |sort_input: Option<SortInput>, offset: u32, limit: u32| {
        accounts::list(
            &w.env.ctx,
            &AccountsListInput {
                filter: AccountFilter {
                    kind: AccountListKind::default(),
                    text: String::new(),
                    include_inactive: false,
                    has_opening: None,
                },
                offset,
                limit,
                sort: sort_input,
            },
        )
        .unwrap()
    };
    let natural = list(None, 0, 0);
    let codes_of = |l: &mandi_services::core::accounts::AccountsList| -> Vec<String> {
        l.rows.iter().map(|r| r.code.clone()).collect()
    };
    let in_pages = |sort_input: Option<SortInput>| -> Vec<String> {
        let total = natural.count;
        (0..total)
            .map(|offset| {
                let page = list(sort_input.clone(), offset, 1);
                assert_eq!(page.rows.len(), 1);
                page.rows[0].code.clone()
            })
            .collect()
    };
    // Sorted by code, ascending, is the natural order; descending is its mirror (ties are none:
    // codes are unique).
    let asc = in_pages(sort("code", SortDirection::Asc));
    assert_eq!(asc, codes_of(&natural));
    let mut desc = in_pages(sort("code", SortDirection::Desc));
    desc.reverse();
    assert_eq!(desc, asc);
    // There is no sort by balance (it would read every journal line); the dues are ranked by R20.
    let balance = accounts::list(
        &w.env.ctx,
        &AccountsListInput {
            filter: AccountFilter {
                kind: AccountListKind::default(),
                text: String::new(),
                include_inactive: false,
                has_opening: None,
            },
            offset: 0,
            limit: 0,
            sort: sort("balance", SortDirection::Desc),
        },
    )
    .unwrap_err();
    assert!(rejected_column(&balance));
    // By English name: case does not matter, and every account is once in the answer.
    let names: Vec<String> = list(sort("name_en", SortDirection::Asc), 0, 0)
        .rows
        .iter()
        .map(|r| r.name_en.to_lowercase())
        .collect();
    let mut expected = names.clone();
    expected.sort();
    assert_eq!(names, expected);
    assert_eq!(names.len() as u32, natural.count);
    let error = accounts::list(
        &w.env.ctx,
        &AccountsListInput {
            filter: AccountFilter {
                kind: AccountListKind::default(),
                text: String::new(),
                include_inactive: false,
                has_opening: None,
            },
            offset: 0,
            limit: 0,
            sort: sort("note", SortDirection::Asc),
        },
    )
    .unwrap_err();
    assert!(rejected_column(&error));
}

// ---- Masters ----

fn masters_of(w: &World, kind: MasterKind, input: MastersListInput) -> MastersList {
    let _ = kind;
    masters::list_page(&w.env.ctx, &input).unwrap()
}

fn master_input(kind: MasterKind) -> MastersListInput {
    MastersListInput {
        kind,
        include_inactive: false,
        id: None,
        offset: 0,
        limit: 0,
        search: None,
        sort: None,
    }
}

fn master_codes(list: &MastersList) -> Vec<String> {
    list.rows
        .iter()
        .map(|row| match row {
            mandi_services::core::masters::MasterRow::Item(r) => r.name_en.clone(),
            other => panic!("not an item: {other:?}"),
        })
        .collect()
}

#[test]
fn the_list_of_masters_sorts_searches_and_hides_the_inactive() {
    let w = support::world();
    let items = |input: MastersListInput| masters_of(&w, MasterKind::Item, input);
    let natural = items(master_input(MasterKind::Item));
    let names = master_codes(&natural);
    assert!(names.len() >= 4, "the world has four items: {names:?}");
    // By English name, ascending and descending, in pages of one: every item once.
    let by_name = |direction| {
        (0..natural.total_count)
            .map(|offset| {
                let page = items(MastersListInput {
                    offset: u32::try_from(offset).unwrap(),
                    limit: 1,
                    sort: sort("name_en", direction),
                    ..master_input(MasterKind::Item)
                });
                master_codes(&page).remove(0)
            })
            .collect::<Vec<String>>()
    };
    let mut expected = names.clone();
    expected.sort_by_key(|n| n.to_lowercase());
    assert_eq!(by_name(SortDirection::Asc), expected);
    expected.reverse();
    assert_eq!(by_name(SortDirection::Desc), expected);
    // By code, descending is the mirror of the natural order (codes are unique).
    let mut mirrored = master_codes(&items(MastersListInput {
        sort: sort("code", SortDirection::Desc),
        ..master_input(MasterKind::Item)
    }));
    mirrored.reverse();
    assert_eq!(mirrored, names);
    // Search finds by name in either script, and a typed `%` is a plain character.
    let found = |text: &str, include_inactive: bool| {
        let list = items(MastersListInput {
            search: Some(text.into()),
            include_inactive,
            ..master_input(MasterKind::Item)
        });
        (master_codes(&list), list.total_count)
    };
    assert_eq!(found("Apple", false), (vec!["Apple".to_owned()], 1));
    assert_eq!(found("%", false), (vec![], 0));
    // An inactive record steps aside unless asked for; the active count stays honest.
    masters::set_active(&w.env.ctx, &w.owner, MasterKind::Item, w.apple, false).unwrap();
    assert_eq!(found("Apple", false), (vec![], 0));
    assert_eq!(found("Apple", true), (vec!["Apple".to_owned()], 1));
    let shown = items(MastersListInput {
        include_inactive: true,
        ..master_input(MasterKind::Item)
    });
    assert_eq!(shown.total_count, natural.total_count);
    assert_eq!(shown.totals.active_count, natural.total_count - 1);
    // Another kind has its own columns; one that the kind does not have is refused.
    let packings = masters::list_page(
        &w.env.ctx,
        &MastersListInput {
            sort: sort("approx_kg", SortDirection::Desc),
            ..master_input(MasterKind::Packing)
        },
    )
    .unwrap();
    assert!(packings.total_count >= 1);
    let error = masters::list_page(
        &w.env.ctx,
        &MastersListInput {
            sort: sort("approx_kg", SortDirection::Asc),
            ..master_input(MasterKind::City)
        },
    )
    .unwrap_err();
    assert!(rejected_column(&error));
}

// ---- The cash book ----

fn cash_book(w: &World, input: CashbookInput) -> Cashbook {
    cashbook::get(&w.env.ctx, &input).unwrap()
}

#[test]
fn the_cash_book_pages_in_day_order_with_the_true_running_cash_and_sorts_by_amount() {
    let w = support::world();
    advance_on(&w, "10-10-2026", w.rahim, "1,000", "");
    advance_on(&w, "10-10-2026", w.karim, "3,000", "");
    set_today(&w.env, 2026, 11, 5);
    advance_on(&w, "05-11-2026", w.rahim, "2,000", "");
    let base = CashbookInput::default();
    let natural = cash_book(&w, base.clone());
    assert_eq!(natural.rows.len() as i64, natural.total_count);
    assert!(natural.total_count >= 4);
    assert!(natural.rows.iter().all(|r| r.running_paisa.is_some()));
    assert_eq!(
        natural.rows.last().and_then(|r| r.running_paisa),
        Some(natural.closing_paisa)
    );
    // Pages of one, oldest first and newest first, from both ends of the book: each row has the
    // running cash it has in the one big page (the prefix is taken from the nearer end).
    for desc in [false, true] {
        let direction = if desc {
            SortDirection::Desc
        } else {
            SortDirection::Asc
        };
        let mut seen = Vec::new();
        for offset in 0..natural.total_count {
            let page = cash_book(
                &w,
                CashbookInput {
                    offset: u32::try_from(offset).unwrap(),
                    limit: 1,
                    sort: sort("entry_date", direction),
                    ..base.clone()
                },
            );
            assert_eq!(page.rows.len(), 1);
            let row = page.rows[0].clone();
            let same = natural
                .rows
                .iter()
                .find(|r| r.entry_id == row.entry_id)
                .expect("the same entry");
            assert_eq!(row.running_paisa, same.running_paisa, "desc {desc}");
            assert_eq!(
                (row.in_paisa, row.out_paisa),
                (same.in_paisa, same.out_paisa)
            );
            seen.push(row.entry_id);
        }
        let mut expected: Vec<i64> = natural.rows.iter().map(|r| r.entry_id).collect();
        if desc {
            expected.reverse();
        }
        assert_eq!(seen, expected);
    }
    // A date range: the opening is the cash before the first day, and the pages still add up.
    let november = cash_book(
        &w,
        CashbookInput {
            from: Some("01-11-2026".into()),
            ..base.clone()
        },
    );
    assert_eq!(november.total_count, 1);
    assert_eq!(november.closing_paisa, natural.closing_paisa);
    assert_eq!(
        november.rows[0].running_paisa,
        natural.rows.last().and_then(|r| r.running_paisa)
    );
    assert_eq!(
        november
            .opening_paisa
            .checked_add(november.totals.in_paisa)
            .and_then(|v| v.checked_sub(november.totals.out_paisa))
            .unwrap(),
        november.closing_paisa
    );
    // By the cash paid out, biggest first: every entry once, in order, with no running cash.
    let by_amount = |direction| {
        (0..natural.total_count)
            .map(|offset| {
                let page = cash_book(
                    &w,
                    CashbookInput {
                        offset: u32::try_from(offset).unwrap(),
                        limit: 1,
                        sort: sort("out_paisa", direction),
                        ..base.clone()
                    },
                );
                assert_eq!(page.rows.len(), 1);
                page.rows[0].clone()
            })
            .collect::<Vec<_>>()
    };
    let rows = by_amount(SortDirection::Desc);
    assert!(rows.iter().all(|r| r.running_paisa.is_none()));
    let outs: Vec<i64> = rows.iter().map(|r| r.out_paisa.as_paisa()).collect();
    let mut expected = outs.clone();
    expected.sort_by(|a, b| b.cmp(a));
    assert_eq!(outs, expected);
    assert_eq!(outs[0], 300_000);
    let ups: Vec<i64> = by_amount(SortDirection::Asc)
        .iter()
        .map(|r| r.out_paisa.as_paisa())
        .collect();
    let mut expected = ups.clone();
    expected.sort_unstable();
    assert_eq!(ups, expected);
    // Another column is sorted too (the slow path), without the running cash; the running cash is
    // the day order, so it is not a column to sort by.
    let by_type = cash_book(
        &w,
        CashbookInput {
            sort: sort("posting_type", SortDirection::Asc),
            ..base.clone()
        },
    );
    assert_eq!(by_type.rows.len(), natural.rows.len());
    assert!(by_type.rows.iter().all(|r| r.running_paisa.is_none()));
    for column in ["running_paisa", "account"] {
        let bad = cashbook::get(
            &w.env.ctx,
            &CashbookInput {
                sort: sort(column, SortDirection::Asc),
                ..base.clone()
            },
        )
        .unwrap_err();
        assert!(rejected_column(&bad), "{column}");
    }

    // Months: the months THIS PAGE touches, each whole month.
    let monthly = |input: CashbookInput| {
        cash_book(
            &w,
            CashbookInput {
                group_by_month: true,
                ..input
            },
        )
    };
    let list = monthly(base.clone());
    assert_eq!(
        list.months.last().map(|m| m.month.as_str()),
        Some("2026-11")
    );
    assert_eq!(
        list.months.iter().map(|m| m.count).sum::<i64>(),
        list.total_count
    );
    let sum_of = |key: &str| -> i64 { list.months.iter().map(|m| month_figure(m, key)).sum() };
    assert_eq!(sum_of("in_paisa"), list.totals.in_paisa.as_paisa());
    assert_eq!(sum_of("out_paisa"), list.totals.out_paisa.as_paisa());
    // The last row alone is in November: only that month, with the whole month's sums.
    let last_page = monthly(CashbookInput {
        offset: u32::try_from(natural.total_count - 1).unwrap(),
        limit: 1,
        ..base.clone()
    });
    assert_eq!(last_page.months.len(), 1);
    assert_eq!(last_page.months[0].month, "2026-11");
    assert_eq!(last_page.months[0].count, 1);
    // Newest first when sorted by the day, descending; none for another sort.
    let list = monthly(CashbookInput {
        sort: sort("entry_date", SortDirection::Desc),
        ..base.clone()
    });
    assert_eq!(list.months[0].month, "2026-11");
    let list = monthly(CashbookInput {
        sort: sort("in_paisa", SortDirection::Asc),
        ..base
    });
    assert!(list.months.is_empty());
}

#[test]
fn a_cash_book_page_over_a_gap_has_headings_only_for_months_with_entries() {
    let w = support::world();
    advance_on(&w, "10-10-2026", w.rahim, "1,000", "");
    set_today(&w.env, 2027, 1, 5);
    advance_on(&w, "05-01-2027", w.rahim, "2,000", "");
    // One page holds both rows: October and January, and no entry in the three months between.
    let list = cash_book(
        &w,
        CashbookInput {
            group_by_month: true,
            ..CashbookInput::default()
        },
    );
    let months: Vec<&str> = list.months.iter().map(|m| m.month.as_str()).collect();
    assert_eq!(months, ["2026-10", "2027-01"]);
    assert_eq!(
        list.months.iter().map(|m| m.count).sum::<i64>(),
        list.total_count
    );
}

// ---- Ledgers of one partner, khata, person ----

/// Repeats an attempt that asks to confirm the spelling of some words, with those words.
fn confirming<T>(mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

#[test]
fn the_ledger_lists_of_one_partner_khata_and_person_sort_and_give_months() {
    let w = support::world();
    let a = w.partners[0];
    let take = |amount: &str, date: &str, reason: &str| {
        vouchers::save_personal_expense(
            &w.env.ctx,
            &w.owner,
            &PersonalExpenseInput {
                date: date.into(),
                partner_id: Some(a),
                amount: amount.into(),
                reason: reason.into(),
                return_by: None,
                taken_by: None,
                narration: None,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    };
    let invest = |amount: &str, date: &str| {
        vouchers::save_capital_in(
            &w.env.ctx,
            &w.owner,
            &CapitalInput {
                date: date.into(),
                partner_id: Some(a),
                amount: amount.into(),
                narration: None,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    };
    let family = confirming(|words| {
        khata::create(
            &w.env.ctx,
            &w.owner,
            &KhataCreateInput {
                name_ur: "خاندانی کھاتہ".into(),
                name_en: "Family Khata".into(),
                note: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id;
    let khata_out = |amount: &str, date: &str| {
        vouchers::save_khata_out(
            &w.env.ctx,
            &w.owner,
            &KhataOutInput {
                date: date.into(),
                khata_account_id: Some(family),
                amount: amount.into(),
                reason: "خرچ".into(),
                narration: None,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    };
    let aslam = confirming(|words| {
        lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                phone: None,
                staff_id: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id;
    let lend = |amount: &str, date: &str| {
        vouchers::save_money_lent(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::lent::MoneyLentInput {
                date: date.into(),
                person_account_id: Some(aslam),
                amount: amount.into(),
                reason: "ضرورت".into(),
                return_by: None,
                narration: None,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    };
    take("2,000", "10-10-2026", "home");
    take("500", "10-10-2026", "tea");
    invest("9,000", "10-10-2026");
    khata_out("1,500", "10-10-2026");
    lend("4,000", "10-10-2026");
    set_today(&w.env, 2026, 11, 5);
    take("1,000", "05-11-2026", "wedding");
    invest("3,000", "05-11-2026");
    khata_out("700", "05-11-2026");
    lend("1,000", "05-11-2026");

    // ---- Money taken by a partner ----
    let takes = |input: PartnerTakesInput| partners::takes(&w.env.ctx, &input).unwrap();
    let take_base = PartnerTakesInput {
        show_finished: true,
        partner_id: a,
        offset: 0,
        limit: 0,
        sort: None,
        group_by_month: false,
    };
    let natural = takes(take_base.clone());
    assert_eq!(natural.rows.len(), 3);
    let by_amount: Vec<i64> = takes(PartnerTakesInput {
        sort: sort("amount_paisa", SortDirection::Desc),
        ..take_base.clone()
    })
    .rows
    .iter()
    .map(|r| r.amount_paisa.as_paisa())
    .collect();
    assert_eq!(by_amount, [200_000, 100_000, 50_000]);
    let second = takes(PartnerTakesInput {
        sort: sort("amount_paisa", SortDirection::Desc),
        offset: 1,
        limit: 1,
        ..take_base.clone()
    });
    assert_eq!(second.rows[0].amount_paisa.as_paisa(), 100_000);
    let bad = partners::takes(
        &w.env.ctx,
        &PartnerTakesInput {
            sort: sort("partner", SortDirection::Asc),
            ..take_base.clone()
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad));
    let months = takes(PartnerTakesInput {
        group_by_month: true,
        ..take_base.clone()
    })
    .months;
    let shape: Vec<(&str, i64, i64)> = months
        .iter()
        .map(|m| (m.month.as_str(), m.count, month_figure(m, "taken_paisa")))
        .collect();
    assert_eq!(shape, [("2026-10", 2, 250_000), ("2026-11", 1, 100_000)]);
    assert!(
        takes(PartnerTakesInput {
            group_by_month: true,
            sort: sort("amount_paisa", SortDirection::Asc),
            ..take_base.clone()
        })
        .months
        .is_empty()
    );
    assert_eq!(
        takes(PartnerTakesInput {
            group_by_month: true,
            sort: sort("entry_date", SortDirection::Desc),
            ..take_base
        })
        .months[0]
            .month,
        "2026-11"
    );

    // ---- Capital history: a sorted row keeps its true running balance ----
    let history =
        |input: CapitalHistoryInput| partners::capital_history(&w.env.ctx, &input).unwrap();
    let capital_base = CapitalHistoryInput {
        partner_id: a,
        offset: 0,
        limit: 0,
        sort: None,
        group_by_month: false,
    };
    let natural = history(capital_base.clone());
    assert_eq!(natural.rows.len(), 2);
    let sorted = history(CapitalHistoryInput {
        sort: sort("amount_paisa", SortDirection::Asc),
        ..capital_base.clone()
    });
    assert_eq!(sorted.rows[0].amount_paisa.as_paisa(), 300_000);
    assert_eq!(sorted.rows[0].running_paisa, natural.rows[1].running_paisa);
    let months = history(CapitalHistoryInput {
        group_by_month: true,
        ..capital_base.clone()
    })
    .months;
    assert_eq!(months.len(), 2);
    assert_eq!(month_figure(&months[0], "invested_paisa"), 900_000);
    assert_eq!(month_figure(&months[1], "invested_paisa"), 300_000);

    // ---- A khata's statement ----
    let statement = |input: KhataStatementInput| khata::statement(&w.env.ctx, &input).unwrap();
    let khata_base = KhataStatementInput {
        khata_account_id: family,
        from: None,
        to: None,
        offset: 0,
        limit: 0,
        sort: None,
        group_by_month: false,
    };
    let natural = statement(khata_base.clone());
    assert_eq!(natural.rows.len(), 2);
    let sorted = statement(KhataStatementInput {
        sort: sort("amount_paisa", SortDirection::Desc),
        ..khata_base.clone()
    });
    // Paid out is minus, so the smaller (more negative) amount is last when descending.
    assert_eq!(sorted.rows[0].amount_paisa.as_paisa(), -70_000);
    assert_eq!(sorted.rows[0].running_paisa, natural.rows[1].running_paisa);
    let months = statement(KhataStatementInput {
        group_by_month: true,
        ..khata_base
    })
    .months;
    assert_eq!(month_figure(&months[0], "paid_out_paisa"), 150_000);
    assert_eq!(month_figure(&months[1], "paid_out_paisa"), 70_000);

    // ---- The partner's statement ----
    let ledger = |input: PartnersStatementInput| partners::statement(&w.env.ctx, &input).unwrap();
    let statement_base = PartnersStatementInput {
        partner_id: a,
        from: None,
        to: None,
        offset: 0,
        limit: 0,
        sort: None,
        group_by_month: false,
    };
    let natural = ledger(statement_base.clone());
    let sorted = ledger(PartnersStatementInput {
        sort: sort("amount_paisa", SortDirection::Asc),
        ..statement_base.clone()
    });
    assert_eq!(sorted.total_count, natural.total_count);
    let amounts: Vec<Option<i64>> = sorted
        .lines
        .iter()
        .map(|l| l.amount_paisa.map(|p| p.as_paisa()))
        .collect();
    let mut expected = amounts.clone();
    expected.sort();
    assert_eq!(amounts, expected);
    // Every sorted line keeps the running total it has in the natural order.
    for line in &sorted.lines {
        let same = natural
            .lines
            .iter()
            .find(|l| {
                l.entry_date == line.entry_date
                    && l.amount_paisa == line.amount_paisa
                    && l.heading == line.heading
                    && l.entry_id == line.entry_id
            })
            .expect("the same line");
        assert_eq!(line.running_paisa, same.running_paisa);
    }
    let months = ledger(PartnersStatementInput {
        group_by_month: true,
        ..statement_base.clone()
    })
    .months;
    assert_eq!(months.len(), 2);
    assert_eq!(
        months.iter().map(|m| m.count).sum::<i64>(),
        natural.total_count
    );
    let bad = partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            sort: sort("heading", SortDirection::Asc),
            ..statement_base
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad));

    // ---- One person's loans ----
    let loans = |input: LentListInput| lent::list(&w.env.ctx, &input).unwrap();
    let lent_base = LentListInput {
        show_finished: true,
        person_id: aslam,
        offset: 0,
        limit: 0,
        sort: None,
        group_by_month: false,
    };
    let sorted = loans(LentListInput {
        sort: sort("amount_paisa", SortDirection::Asc),
        ..lent_base.clone()
    });
    let amounts: Vec<i64> = sorted
        .rows
        .iter()
        .map(|r| r.amount_paisa.as_paisa())
        .collect();
    assert_eq!(amounts, [100_000, 400_000]);
    let months = loans(LentListInput {
        group_by_month: true,
        ..lent_base.clone()
    })
    .months;
    assert_eq!(month_figure(&months[0], "lent_paisa"), 400_000);
    assert_eq!(month_figure(&months[1], "lent_paisa"), 100_000);
    let bad = lent::list(
        &w.env.ctx,
        &LentListInput {
            sort: sort("person", SortDirection::Asc),
            ..lent_base
        },
    )
    .unwrap_err();
    assert!(rejected_column(&bad));
}

// ---- Trips and gardens ----

#[test]
fn the_lists_of_trips_and_gardens_sort_and_give_month_subtotals() {
    let w = support::world();
    let [a, b, c] = w.partners;
    let partners = || -> Vec<VenturePartnerInput> {
        [(a, "40"), (b, "30"), (c, "30")]
            .into_iter()
            .map(|(partner_id, share)| VenturePartnerInput {
                partner_id,
                share: share.into(),
                source: MoneySource::Shop,
            })
            .collect()
    };
    let new_trip = |date: &str, market: &str| {
        trips::create(
            &w.env.ctx,
            &w.owner,
            &TripInput {
                date: date.into(),
                market: Some(market.into()),
                sent_person: None,
                note: None,
                partners: partners(),
            },
        )
        .unwrap()
        .number
    };
    let new_garden = |date: &str, ur: &str, en: &str, words: &[&str], rent: &str| {
        gardens::create(
            &w.env.ctx,
            &w.owner,
            &GardenInput {
                name_ur: ur.into(),
                name_en: en.into(),
                confirmed_rules_words: words.iter().map(|x| (*x).to_owned()).collect(),
                date: date.into(),
                item_id: Some(w.banana),
                rent: rent.into(),
                season_end: None,
                note: None,
                partners: partners(),
                paid_now: None,
                paid_now_reason: None,
                parts: Vec::new(),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap()
        .row
        .number
    };
    new_trip("10-10-2026", "Lahore");
    new_trip("10-10-2026", "Faisalabad");
    new_garden(
        "10-10-2026",
        "شاہ پور باغ",
        "Shahpur Garden",
        &["شاہ", "پور", "باغ", "Shahpur", "Garden"],
        "50,000",
    );
    set_today(&w.env, 2026, 11, 5);
    new_trip("05-11-2026", "Multan");
    new_garden(
        "05-11-2026",
        "ملتان باغ",
        "Multan Garden",
        &["ملتان", "باغ", "Multan", "Garden"],
        "20,000",
    );

    // ---- Trips: a sort by a column of the table, read in pages of one ----
    let trip_numbers = |input: TripsListInput| -> Vec<i64> {
        trips::list(&w.env.ctx, &input)
            .unwrap()
            .rows
            .iter()
            .map(|r| r.number)
            .collect()
    };
    let trips_base = TripsListInput::default();
    assert_eq!(trip_numbers(trips_base.clone()), [3, 2, 1]);
    let by = |column: &str, direction| TripsListInput {
        sort: sort(column, direction),
        ..trips_base.clone()
    };
    assert_eq!(trip_numbers(by("number", SortDirection::Asc)), [1, 2, 3]);
    assert_eq!(
        trip_numbers(by("entry_date", SortDirection::Asc)),
        [1, 2, 3]
    );
    assert_eq!(
        trip_numbers(by("entry_date", SortDirection::Desc)),
        [3, 2, 1]
    );
    // Faisalabad, Lahore, Multan.
    assert_eq!(trip_numbers(by("market", SortDirection::Asc)), [2, 1, 3]);
    let page = trips::list(
        &w.env.ctx,
        &TripsListInput {
            offset: 1,
            limit: 1,
            ..by("market", SortDirection::Asc)
        },
    )
    .unwrap();
    assert_eq!(page.rows[0].number, 1);
    let bad = trips::list(&w.env.ctx, &by("result_paisa", SortDirection::Asc)).unwrap_err();
    assert!(rejected_column(&bad));
    let months = trips::list(
        &w.env.ctx,
        &TripsListInput {
            group_by_month: true,
            ..trips_base.clone()
        },
    )
    .unwrap()
    .months;
    let shape: Vec<(&str, i64)> = months.iter().map(|m| (m.month.as_str(), m.count)).collect();
    assert_eq!(shape, [("2026-11", 1), ("2026-10", 2)]);
    assert!(months.iter().all(|m| m.figures.is_empty()));

    // ---- Gardens: sorted in memory, so the figures can be sorted too ----
    let garden_numbers = |input: GardensListInput| -> Vec<i64> {
        gardens::list(&w.env.ctx, &input)
            .unwrap()
            .rows
            .iter()
            .map(|r| r.number)
            .collect()
    };
    let gardens_base = GardensListInput::default();
    assert_eq!(garden_numbers(gardens_base.clone()), [2, 1]);
    let by = |column: &str, direction| GardensListInput {
        sort: sort(column, direction),
        ..gardens_base.clone()
    };
    assert_eq!(garden_numbers(by("rent_paisa", SortDirection::Asc)), [2, 1]);
    assert_eq!(
        garden_numbers(by("rent_paisa", SortDirection::Desc)),
        [1, 2]
    );
    assert_eq!(garden_numbers(by("name_en", SortDirection::Asc)), [2, 1]);
    // Neither garden has had a payment: equal balances keep the natural order.
    assert_eq!(
        garden_numbers(by("balance_paisa", SortDirection::Asc)),
        [2, 1]
    );
    assert_eq!(garden_numbers(by("entry_date", SortDirection::Asc)), [1, 2]);
    let bad = gardens::list(&w.env.ctx, &by("partners", SortDirection::Asc)).unwrap_err();
    assert!(rejected_column(&bad));
    let months = gardens::list(
        &w.env.ctx,
        &GardensListInput {
            group_by_month: true,
            ..gardens_base.clone()
        },
    )
    .unwrap()
    .months;
    let shape: Vec<(&str, i64, i64)> = months
        .iter()
        .map(|m| (m.month.as_str(), m.count, month_figure(m, "rent_paisa")))
        .collect();
    assert_eq!(
        shape,
        [("2026-11", 1, 2_000_000), ("2026-10", 1, 5_000_000)]
    );
}

// ---- Rules, learned pairs, the audit log and the lent report ----

fn rule_of(
    item: Option<i64>,
    account: Option<i64>,
    value: &str,
    active: bool,
    note: &str,
) -> RuleInput {
    RuleInput {
        id: None,
        version: None,
        side: ChargeSide::Seller,
        charge_type: ChargeType::Commission,
        account_id: account,
        item_id: item,
        item_group: None,
        method: ChargeMethod::Percent,
        value: value.into(),
        from_date: "01-10-2026".into(),
        to_date: String::new(),
        is_active: active,
        note: note.into(),
    }
}

#[test]
fn the_rule_list_searches_sorts_and_hides_the_inactive() {
    let w = support::world();
    for input in [
        rule_of(Some(w.banana), None, "5", true, "first"),
        rule_of(Some(w.apple), None, "4", true, ""),
        rule_of(None, Some(w.rahim), "3", true, ""),
        rule_of(Some(w.banana), None, "6", false, "old rate"),
    ] {
        rules::save(&w.env.ctx, &w.owner, &input).unwrap();
    }
    let filter = |include_inactive| RulesFilter {
        side: None,
        charge_type: None,
        account_id: None,
        item_id: None,
        include_inactive,
    };
    let read = |include_inactive: bool, search: Option<&str>, sort_input: Option<SortInput>| {
        rules::list_page(
            &w.env.ctx,
            &RulesListInput {
                filter: filter(include_inactive),
                offset: 0,
                limit: 0,
                search: search.map(str::to_owned),
                sort: sort_input,
            },
        )
        .unwrap()
    };
    // The inactive rule steps aside; the counts are over the rules the search names.
    let banana = read(false, Some("banana"), None);
    assert_eq!(banana.total_count, 1);
    assert_eq!(banana.totals.active_count, 1);
    let both = read(true, Some("banana"), None);
    assert_eq!(both.total_count, 2);
    assert_eq!(both.totals.active_count, 1);
    // A search finds a note and a party too, and a typed `%` is a plain character.
    assert_eq!(read(true, Some("old rate"), None).total_count, 1);
    assert_eq!(read(false, Some("rahim"), None).total_count, 1);
    assert_eq!(read(false, Some("%"), None).total_count, 0);
    // Sorted by the state, active first when descending, inactive first when ascending.
    let desc = read(true, Some("banana"), sort("is_active", SortDirection::Desc));
    assert_eq!(
        desc.rows.iter().map(|r| r.is_active).collect::<Vec<_>>(),
        [true, false]
    );
    let asc = read(true, Some("banana"), sort("is_active", SortDirection::Asc));
    assert_eq!(
        asc.rows.iter().map(|r| r.is_active).collect::<Vec<_>>(),
        [false, true]
    );
    // A page of a sorted list is a slice of the whole sorted list.
    let second = rules::list_page(
        &w.env.ctx,
        &RulesListInput {
            filter: filter(true),
            offset: 1,
            limit: 1,
            search: Some("banana".into()),
            sort: sort("is_active", SortDirection::Desc),
        },
    )
    .unwrap();
    assert!(!second.rows[0].is_active);
    let error = rules::list_page(
        &w.env.ctx,
        &RulesListInput {
            filter: filter(false),
            offset: 0,
            limit: 0,
            search: None,
            sort: sort("value_text", SortDirection::Asc),
        },
    )
    .unwrap_err();
    assert!(rejected_column(&error));
}

#[test]
fn the_learned_pairs_search_and_sort_over_the_whole_list() {
    let w = support::world();
    for (roman, urdu) in [
        ("ali", "علی"),
        ("ali", "علی"),
        ("ahmed", "احمد"),
        ("bilal", "بلال"),
        ("chand", "چاند"),
        ("dawood", "داؤد"),
    ] {
        translit::learn(
            &w.env.ctx,
            &w.owner,
            &NewPair {
                roman: roman.into(),
                urdu: urdu.into(),
                kind: Kind::Name,
            },
        )
        .unwrap();
    }
    let page = |offset, limit, search: Option<&str>, sort_input: Option<SortInput>| {
        translit::list_page_with(&w.env.ctx, offset, limit, search, sort_input.as_ref()).unwrap()
    };
    let romans = |rows: &[mandi_services::translit::LearnedRow]| -> Vec<String> {
        rows.iter().map(|r| r.roman.clone()).collect()
    };
    // The natural order is the most used first.
    let (rows, count, uses) = page(0, 0, None, None);
    assert_eq!((count, uses), (5, 6));
    assert_eq!(rows[0].roman, "ali");
    // Sorted by the English spelling, read in pages of one.
    let (rows, _, _) = page(0, 0, None, sort("roman", SortDirection::Desc));
    assert_eq!(romans(&rows), ["dawood", "chand", "bilal", "ali", "ahmed"]);
    let (second, _, _) = page(1, 1, None, sort("roman", SortDirection::Desc));
    assert_eq!(romans(&second), ["chand"]);
    // A search keeps the pairs it names, and the count and the uses are over them.
    let (rows, count, uses) = page(0, 0, Some("al"), None);
    assert_eq!(romans(&rows), ["ali", "bilal"]);
    assert_eq!((count, uses), (2, 3));
    let (rows, count, _) = page(0, 0, Some("احمد"), None);
    assert_eq!((romans(&rows), count), (vec!["ahmed".to_owned()], 1));
    let (rows, _, _) = page(0, 0, Some("%"), None);
    assert!(rows.is_empty());
    let error = translit::list_page_with(
        &w.env.ctx,
        0,
        0,
        None,
        sort("id", SortDirection::Asc).as_ref(),
    )
    .unwrap_err();
    assert!(rejected_column(&error));
}

#[test]
fn the_audit_log_sorts_and_counts_entries_by_month() {
    use mandi_reports::audit::{AuditFilters, AuditListInput, AuditPage};
    let w = support::world();
    rules::save(
        &w.env.ctx,
        &w.owner,
        &rule_of(Some(w.banana), None, "5", true, ""),
    )
    .unwrap();
    set_today(&w.env, 2026, 11, 5);
    rules::save(
        &w.env.ctx,
        &w.owner,
        &rule_of(Some(w.apple), None, "4", true, ""),
    )
    .unwrap();
    let read = |sort_input: Option<SortInput>, group: bool, offset: i64, limit: i64| -> AuditPage {
        mandi_reports::audit::list(
            w.env.ctx.db(),
            &AuditListInput {
                filters: AuditFilters::default(),
                offset,
                limit,
                sort: sort_input,
                group_by_month: group,
            },
        )
        .unwrap()
    };
    let natural = read(None, false, 0, 1000);
    assert!(natural.total_count >= 2);
    // Newest first by default; a sort by the time ascending is the mirror.
    let ids: Vec<i64> = natural.rows.iter().map(|r| r.id).collect();
    let asc = read(sort("at", SortDirection::Asc), false, 0, 1000);
    let mut asc_ids: Vec<i64> = asc.rows.iter().map(|r| r.id).collect();
    asc_ids.reverse();
    assert_eq!(asc_ids, ids);
    // By the action, every entry once, and a page is a slice of the whole sorted log.
    let by_action = read(sort("action", SortDirection::Asc), false, 0, 1000);
    let actions: Vec<&str> = by_action.rows.iter().map(|r| r.action.as_str()).collect();
    let mut sorted = actions.clone();
    sorted.sort_unstable();
    assert_eq!(actions, sorted);
    assert_eq!(by_action.total_count, natural.total_count);
    let one = read(sort("action", SortDirection::Asc), false, 1, 1);
    assert_eq!(one.rows[0].id, by_action.rows[1].id);
    // Months: the counts add up to the whole log, newest month first like the list.
    let months = read(None, true, 0, 1).months;
    assert!(months.len() >= 2, "{months:?}");
    assert_eq!(
        months.iter().map(|m| m.count).sum::<i64>(),
        natural.total_count
    );
    assert!(months.windows(2).all(|pair| pair[0].month > pair[1].month));
    let months = read(sort("at", SortDirection::Asc), true, 0, 1).months;
    assert!(months.windows(2).all(|pair| pair[0].month < pair[1].month));
    assert!(
        read(sort("action", SortDirection::Asc), true, 0, 1)
            .months
            .is_empty()
    );
    let error = mandi_reports::audit::list(
        w.env.ctx.db(),
        &AuditListInput {
            filters: AuditFilters::default(),
            offset: 0,
            limit: 0,
            sort: sort("old_json", SortDirection::Asc),
            group_by_month: false,
        },
    )
    .unwrap_err();
    assert!(matches!(error, mandi_reports::ReportsError::Rejected(items)
        if items.iter().any(|i| i.field == "sort.column")));
}

#[test]
fn the_lent_report_searches_and_sorts_over_every_person() {
    let w = support::world();
    let person = |ur: &str, en: &str| {
        confirming(|words| {
            lent::person_create(
                &w.env.ctx,
                &w.owner,
                &LentPersonCreateInput {
                    name_ur: ur.into(),
                    name_en: en.into(),
                    phone: None,
                    staff_id: None,
                    confirmed_rules_words: words.to_vec(),
                },
            )
        })
        .account
        .id
    };
    let aslam = person("اسلم", "Aslam");
    let bilal = person("بلال", "Bilal");
    for (who, amount) in [(aslam, "4,000"), (bilal, "1,000")] {
        vouchers::save_money_lent(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::lent::MoneyLentInput {
                date: "10-10-2026".into(),
                person_account_id: Some(who),
                amount: amount.into(),
                reason: "ضرورت".into(),
                return_by: None,
                narration: None,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    }
    let report = |search: Option<&str>, sort_input: Option<SortInput>| {
        lent::report(
            &w.env.ctx,
            &LentReportInput {
                as_of: None,
                offset: 0,
                limit: 0,
                search: search.map(str::to_owned),
                sort: sort_input,
            },
        )
        .unwrap()
    };
    let names = |r: &mandi_services::core::lent::LentReport| -> Vec<String> {
        r.rows.iter().map(|x| x.person.name_en.clone()).collect()
    };
    assert_eq!(names(&report(None, None)), ["Aslam", "Bilal"]);
    assert_eq!(
        names(&report(None, sort("lent_paisa", SortDirection::Asc))),
        ["Bilal", "Aslam"]
    );
    assert_eq!(
        names(&report(None, sort("person", SortDirection::Desc))),
        ["Bilal", "Aslam"]
    );
    // A search keeps the people it names, and the totals are over them.
    let only = report(Some("bilal"), None);
    assert_eq!(names(&only), ["Bilal"]);
    assert_eq!(only.total_count, 1);
    assert_eq!(only.totals.lent_paisa.as_paisa(), 100_000);
    assert_eq!(names(&report(Some("اسلم"), None)), ["Aslam"]);
    assert_eq!(report(None, None).totals.lent_paisa.as_paisa(), 500_000);
    let error = lent::report(
        &w.env.ctx,
        &LentReportInput {
            as_of: None,
            offset: 0,
            limit: 0,
            search: None,
            sort: sort("phone", SortDirection::Asc),
        },
    )
    .unwrap_err();
    assert!(rejected_column(&error));
}

// ---- Urdu alphabetical order (NFR-URD-19) ----

#[test]
fn urdu_names_sort_in_the_order_of_the_urdu_alphabet() {
    use mandi_services::core::accounts::AccountType;
    let w = support::world();
    // ا ب پ ت ٹ is the Urdu order. As code points the order is ا ب ت ٹ پ, which is wrong.
    let names = [
        ("ٹیپو", "Tipu"),
        ("پرویز", "Parvez"),
        ("تنویر", "Tanvir"),
        ("اسلم", "Aslam"),
        ("بلال", "Bilal"),
    ];
    for (ur, en) in names {
        support::add_account(&w.env, &w.owner, AccountType::Buyer, ur, en, true, "");
    }
    let mine: Vec<&str> = names.iter().map(|n| n.1).collect();
    let listed = |direction| -> Vec<String> {
        accounts::list(
            &w.env.ctx,
            &AccountsListInput {
                filter: AccountFilter {
                    kind: AccountListKind::Buyer,
                    text: String::new(),
                    include_inactive: false,
                    has_opening: None,
                },
                offset: 0,
                limit: 0,
                sort: sort("name_ur", direction),
            },
        )
        .unwrap()
        .rows
        .iter()
        .map(|r| r.name_en.clone())
        .filter(|n| mine.contains(&n.as_str()))
        .collect()
    };
    assert_eq!(
        listed(SortDirection::Asc),
        ["Aslam", "Bilal", "Parvez", "Tanvir", "Tipu"]
    );
    assert_eq!(
        listed(SortDirection::Desc),
        ["Tipu", "Tanvir", "Parvez", "Bilal", "Aslam"]
    );
    // The learned pairs are sorted in memory with the same order.
    for (ur, en) in names {
        translit::learn(
            &w.env.ctx,
            &w.owner,
            &NewPair {
                roman: en.to_lowercase(),
                urdu: ur.into(),
                kind: Kind::Name,
            },
        )
        .unwrap();
    }
    let (rows, _, _) = translit::list_page_with(
        &w.env.ctx,
        0,
        0,
        None,
        sort("urdu", SortDirection::Asc).as_ref(),
    )
    .unwrap();
    let urdu: Vec<&str> = rows.iter().map(|r| r.urdu.as_str()).collect();
    assert_eq!(urdu, ["اسلم", "بلال", "پرویز", "تنویر", "ٹیپو"]);
}

#[test]
fn a_fully_returned_take_or_loan_is_finished_and_the_totals_follow_the_rows_shown() {
    let w = support::world();
    let a = w.partners[0];
    // Partner الف takes 2,000 and 500 on 10-10 and returns 2,000 on 11-05: the first take is
    // returned in full (oldest first), the second is still open.
    for (amount, reason) in [("2,000", "home"), ("500", "tea")] {
        vouchers::save_personal_expense(
            &w.env.ctx,
            &w.owner,
            &PersonalExpenseInput {
                date: "10-10-2026".into(),
                partner_id: Some(a),
                amount: amount.into(),
                reason: reason.into(),
                return_by: None,
                taken_by: None,
                narration: None,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    }
    set_today(&w.env, 2026, 11, 5);
    vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &PersonalReturnInput {
            date: "05-11-2026".into(),
            partner_id: Some(a),
            amount: "2,000".into(),
            narration: None,
        },
    )
    .unwrap();
    let takes = |show_finished: bool| {
        partners::takes(
            &w.env.ctx,
            &PartnerTakesInput {
                partner_id: a,
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: true,
                show_finished,
            },
        )
        .unwrap()
    };
    let open_only = takes(false);
    assert_eq!(open_only.rows.len(), 1);
    assert_eq!(open_only.total_count, 1);
    assert_eq!(open_only.rows[0].amount_paisa.as_paisa(), 50_000);
    // The totals are over the rows listed: only the open take.
    assert_eq!(
        (
            open_only.totals.taken_paisa.as_paisa(),
            open_only.totals.returned_paisa.as_paisa(),
            open_only.totals.open_paisa.as_paisa()
        ),
        (50_000, 0, 50_000)
    );
    assert_eq!(
        open_only.months.iter().map(|m| m.count).sum::<i64>(),
        1,
        "the months follow the rows"
    );
    let everything = takes(true);
    assert_eq!(everything.rows.len(), 2);
    assert_eq!(
        (
            everything.totals.taken_paisa.as_paisa(),
            everything.totals.returned_paisa.as_paisa(),
            everything.totals.open_paisa.as_paisa()
        ),
        (250_000, 200_000, 50_000)
    );
    // In both views the totals equal the sums of the rows.
    for list in [&open_only, &everything] {
        let sum = |f: fn(&mandi_services::core::expenses::TakeRow) -> i64| -> i64 {
            list.rows.iter().map(f).sum()
        };
        assert_eq!(
            sum(|r| r.amount_paisa.as_paisa()),
            list.totals.taken_paisa.as_paisa()
        );
        assert_eq!(
            sum(|r| r.returned_paisa.as_paisa()),
            list.totals.returned_paisa.as_paisa()
        );
        assert_eq!(
            sum(|r| r.open_paisa.as_paisa()),
            list.totals.open_paisa.as_paisa()
        );
    }

    // The same for the loans of one person.
    let aslam = confirming(|words| {
        lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                phone: None,
                staff_id: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id;
    for (date, amount) in [("10-10-2026", "4,000"), ("05-11-2026", "1,000")] {
        vouchers::save_money_lent(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::lent::MoneyLentInput {
                date: date.into(),
                person_account_id: Some(aslam),
                amount: amount.into(),
                reason: "ضرورت".into(),
                return_by: None,
                narration: None,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    }
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &MoneyLentReturnInput {
            date: "05-11-2026".into(),
            person_account_id: Some(aslam),
            amount: "4,000".into(),
            mode: LentReturnMode::Cash,
            salary_head_id: None,
            narration: None,
        },
    )
    .unwrap();
    let loans = |show_finished: bool| {
        lent::list(
            &w.env.ctx,
            &LentListInput {
                person_id: aslam,
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: false,
                show_finished,
            },
        )
        .unwrap()
    };
    let open_loans = loans(false);
    assert_eq!(open_loans.rows.len(), 1);
    assert_eq!(
        (
            open_loans.totals.lent_paisa.as_paisa(),
            open_loans.totals.returned_paisa.as_paisa(),
            open_loans.totals.open_paisa.as_paisa()
        ),
        (100_000, 0, 100_000)
    );
    let all_loans = loans(true);
    assert_eq!(all_loans.rows.len(), 2);
    assert_eq!(
        (
            all_loans.totals.lent_paisa.as_paisa(),
            all_loans.totals.returned_paisa.as_paisa(),
            all_loans.totals.open_paisa.as_paisa()
        ),
        (500_000, 400_000, 100_000)
    );
    for list in [&open_loans, &all_loans] {
        let lent_sum: i64 = list.rows.iter().map(|r| r.amount_paisa.as_paisa()).sum();
        let returned_sum: i64 = list.rows.iter().map(|r| r.returned_paisa.as_paisa()).sum();
        let open_sum: i64 = list.rows.iter().map(|r| r.open_paisa.as_paisa()).sum();
        assert_eq!(
            (lent_sum, returned_sum, open_sum),
            (
                list.totals.lent_paisa.as_paisa(),
                list.totals.returned_paisa.as_paisa(),
                list.totals.open_paisa.as_paisa()
            )
        );
    }
}

// ---- The stock list and the advances report ----

#[test]
fn the_stock_list_searches_and_sorts_each_kind_and_the_totals_follow_the_search() {
    let w = support::world();
    // Three lots with goods left: 10 from Rahim (6 left), 20 from Karim (16), 15 from Rahim (11).
    for (n, seller, item, qty) in [
        (1, w.rahim, w.banana, "10"),
        (2, w.karim, w.apple, "20"),
        (3, w.rahim, w.apple, "15"),
    ] {
        let lot = support::with_sales(
            support::bare_lot(&w, "lot-1", seller, item, qty),
            vec![support::sale("s-1", w.akram, "4", &format!("{}", 600 + n))],
        );
        let mut draft: BikriDraft = support::draft(vec![lot]);
        draft.header.truck_no = Some(format!("STK-{n}"));
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    }
    let read = |search: Option<&str>, sort_input: Option<SortInput>| {
        bikri::stock_list(
            &w.env.ctx,
            &StockListInput {
                search: search.map(str::to_owned),
                sort: sort_input,
                ..StockListInput::default()
            },
        )
        .unwrap()
    };
    let balances = |list: &mandi_services::core::bikri::StockList| -> Vec<i64> {
        list.rows.iter().map(|r| r.balance).collect()
    };
    assert_eq!(balances(&read(None, None)), [6, 16, 11]);
    assert_eq!(
        balances(&read(None, sort("balance", SortDirection::Desc))),
        [16, 11, 6]
    );
    assert_eq!(
        balances(&read(None, sort("balance", SortDirection::Asc))),
        [6, 11, 16]
    );
    assert_eq!(
        balances(&read(None, sort("number", SortDirection::Desc))),
        [11, 16, 6]
    );
    // A search keeps the rows it names; the total follows it, and a number finds a بکری.
    let karim = read(Some("karim"), None);
    assert_eq!((karim.total_count, karim.balance_total), (1, 16));
    let rahim = read(Some("Rahim"), None);
    assert_eq!((rahim.total_count, rahim.balance_total), (2, 17));
    let apples = read(Some("Apple"), None);
    assert_eq!((apples.total_count, apples.balance_total), (2, 27));
    assert_eq!(read(Some("3"), None).balance_total, 11);
    assert_eq!(read(Some("%"), None).total_count, 0);
    let error = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            sort: sort("truck_no", SortDirection::Asc),
            ..StockListInput::default()
        },
    )
    .unwrap_err();
    assert!(rejected_column(&error));
}

#[test]
fn the_advances_report_searches_and_its_totals_follow() {
    let w = support::world();
    advance_on(&w, "10-10-2026", w.rahim, "3,000", "");
    advance_on(&w, "10-10-2026", w.karim, "1,000", "");
    let read = |search: Option<&str>| {
        vouchers::advances_report(
            &w.env.ctx,
            &AdvancesReportInput {
                seller_id: None,
                as_of: String::new(),
                offset: 0,
                limit: 0,
                search: search.map(str::to_owned),
            },
        )
        .unwrap()
    };
    let all = read(None);
    assert_eq!((all.total_count, all.given_paisa.as_paisa()), (2, 400_000));
    let karim = read(Some("karim"));
    assert_eq!(
        (karim.total_count, karim.given_paisa.as_paisa()),
        (1, 100_000)
    );
    assert_eq!(karim.open_paisa.as_paisa(), 100_000);
    assert_eq!(read(Some("رحیم")).given_paisa.as_paisa(), 300_000);
    assert_eq!(read(Some("nobody")).total_count, 0);
}
