//! Plan 17b (PRD FR-19.11, E70, NFR-SPD-24, Code Standards B31): every list built before plan 17
//! loads one page at a time and brings its totals over the WHOLE filter. For each list:
//!
//! - the pages, read one after the other, hold every row once, in the order of the unpaged list;
//! - the count and the totals are the same on every page, and equal the sums of every row;
//! - an offset past the end gives no rows and the same count and totals;
//! - a limit of 0 is a page of 100.
//!
//! The last test writes (or checks) the request and answer fixtures of the lists that had none
//! (masters, rules, learned pairs) and of a second page. Write them again after a deliberate
//! change with `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test list_paging`, read the
//! diff and tell `frontend`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_services::core::bikri::{
    BikriDraft, BikriListInput, BikriSaveInput, BikriSaved, BikriStatus, CancelInput, PaymentType,
    PukhtaInput, PukhtaRegisterInput, PukhtaSide, StockListInput,
};
use mandi_services::core::masters::{MasterKind, MastersListInput};
use mandi_services::core::money::Paisa;
use mandi_services::core::rules::{
    ChargeMethod, ChargeSide, ChargeType, RuleInput, RulesFilter, RulesListInput,
};
use mandi_services::core::translit::Kind;
use mandi_services::core::vouchers::{
    AdvanceInput, RemovedPaymentFields, VoucherCancelInput, VoucherType, VouchersListInput,
};
use mandi_services::translit::NewPair;
use mandi_services::{bikri, masters, rules, translit, vouchers};
use serde::Serialize;
use serde_json::{Value, json};
use support::World;

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Writes the file (when asked to) or checks that it holds exactly this text.
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

/// One pair of files: the request and the real answer.
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

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

/// Reads every page of `size` rows one after the other. Each page says how many rows the whole
/// filter has; that count must be the same on every page and equal the rows read.
fn all_pages<T>(size: u32, mut fetch: impl FnMut(u32, u32) -> (Vec<T>, i64)) -> Vec<T> {
    let mut out: Vec<T> = Vec::new();
    let mut offset = 0_u32;
    let mut count = None;
    loop {
        let (rows, total_count) = fetch(offset, size);
        assert_eq!(
            *count.get_or_insert(total_count),
            total_count,
            "the count is the same on every page"
        );
        assert!(rows.len() <= usize::try_from(size).unwrap());
        if rows.is_empty() {
            break;
        }
        offset += u32::try_from(rows.len()).unwrap();
        out.extend(rows);
    }
    assert_eq!(
        i64::try_from(out.len()).unwrap(),
        count.unwrap(),
        "the pages hold every row once"
    );
    out
}

fn sum(values: impl Iterator<Item = Paisa>) -> Paisa {
    values.fold(Paisa::ZERO, |a, b| a.checked_add(b).unwrap())
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

// ---- بکری ----

/// A بکری of 10 نگ from رحیم of which 4 are sold to اکرم. Truck `TRK-<n>`.
fn small_bikri(w: &World, n: i64) -> BikriDraft {
    let item = if n % 2 == 0 { w.apple } else { w.banana };
    let lot = support::with_sales(
        support::bare_lot(w, "lot-1", w.rahim, item, "10"),
        vec![support::sale("s-1", w.akram, "4", &format!("{}", 600 + n))],
    );
    let mut draft = support::draft(vec![lot]);
    draft.header.truck_no = Some(format!("TRK-{n}"));
    draft
}

fn save_bikri(w: &World, draft: BikriDraft) -> BikriSaved {
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

/// Five بکری with 6 نگ left in each lot; the third is cancelled.
fn five_bikri(w: &World) {
    let saved: Vec<BikriSaved> = (1..=5).map(|n| save_bikri(w, small_bikri(w, n))).collect();
    let third = &saved[2];
    let detail = bikri::get(&w.env.ctx, third.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: third.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

fn bikri_page(
    w: &World,
    offset: u32,
    limit: u32,
    status: Option<BikriStatus>,
) -> mandi_services::core::bikri::BikriList {
    bikri::list(
        &w.env.ctx,
        &BikriListInput {
            offset,
            limit,
            status,
            // This test counts the cancelled one too.
            show_finished: true,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn the_list_of_bikri_pages_with_totals_over_the_whole_filter() {
    let w = support::world();
    five_bikri(&w);
    let whole = bikri_page(&w, 0, 0, None);
    assert_eq!(whole.total_count, 5, "the cancelled one is counted");
    assert_eq!(whole.rows.len(), 5);
    // The totals leave the cancelled بکری out, like the list's own sums.
    let live = |f: fn(&mandi_services::core::bikri::BikriListRow) -> Paisa| {
        sum(whole
            .rows
            .iter()
            .filter(|r| r.status == BikriStatus::Saved)
            .map(f))
    };
    assert_eq!(whole.totals.gross, live(|r| r.gross));
    assert_eq!(whole.totals.net, live(|r| r.net));
    assert_eq!(whole.totals.buyer_total, live(|r| r.buyer_total));
    assert_eq!(
        whole.totals.qty_sold,
        whole
            .rows
            .iter()
            .filter(|r| r.status == BikriStatus::Saved)
            .map(|r| r.qty_sold)
            .sum::<i64>()
    );
    assert_eq!(
        (whole.totals.qty_received, whole.totals.qty_sold),
        (40, 16),
        "four live بکری of 10 received and 4 sold"
    );
    // Fixed money: four live بکری, each 4 نگ sold at 601, 602, 604 and 605. Gross is the line
    // values; seller commission 6% of each line (144.24, 144.48, 144.96, 145.2 rounded to 144,
    // 144, 145, 145); buyer total adds the buyer's 6% and لاگا 8 per نگ on fruit (32).
    assert_eq!(whole.totals.gross, rupees(2_404 + 2_408 + 2_416 + 2_420));
    assert_eq!(
        whole.totals.net,
        rupees((2_404 - 144) + (2_408 - 144) + (2_416 - 145) + (2_420 - 145))
    );
    assert_eq!(
        whole.totals.buyer_total,
        rupees((2_404 + 144 + 32) + (2_408 + 144 + 32) + (2_416 + 145 + 32) + (2_420 + 145 + 32))
    );

    // Pages of 2: every row once, newest first, the same count and totals on every page.
    let paged = all_pages(2, |offset, limit| {
        let page = bikri_page(&w, offset, limit, None);
        assert_eq!(page.totals, whole.totals, "offset {offset}");
        (page.rows, page.total_count)
    });
    assert_eq!(
        paged.iter().map(|r| r.id).collect::<Vec<_>>(),
        whole.rows.iter().map(|r| r.id).collect::<Vec<_>>()
    );
    assert_eq!(
        paged.iter().map(|r| r.number).collect::<Vec<_>>(),
        [5, 4, 3, 2, 1]
    );
    // Past the end: no rows, the same count and totals.
    let past = bikri_page(&w, 50, 2, None);
    assert!(past.rows.is_empty());
    assert_eq!((past.total_count, past.totals), (5, whole.totals.clone()));
    // A filter changes the count and the totals with it.
    let saved_only = bikri_page(&w, 0, 3, Some(BikriStatus::Saved));
    assert_eq!((saved_only.total_count, saved_only.rows.len()), (4, 3));
    assert_eq!(saved_only.totals, whole.totals);
    let cancelled_only = bikri_page(&w, 0, 3, Some(BikriStatus::Cancelled));
    assert_eq!(cancelled_only.total_count, 1);
    assert_eq!(cancelled_only.totals.gross, Paisa::ZERO);
}

#[test]
fn a_bikri_with_two_lots_and_three_lines_is_counted_once_in_the_totals() {
    // The totals read lots and sale lines in separate queries; a بکری with two lots and three
    // lines must not be counted twice by a join.
    let w = support::world();
    let lot1 = support::with_sales(
        support::bare_lot(&w, "lot-1", w.rahim, w.banana, "10"),
        vec![
            support::sale("s-1", w.akram, "3", "600"),
            support::sale("s-2", w.arshad, "2", "500"),
        ],
    );
    let lot2 = support::with_sales(
        support::bare_lot(&w, "lot-2", w.karim, w.apple, "10"),
        vec![support::sale("s-3", w.akram, "5", "700")],
    );
    save_bikri(&w, support::draft(vec![lot1, lot2]));
    let list = bikri_page(&w, 0, 0, None);
    assert_eq!((list.total_count, list.rows.len()), (1, 1));
    assert_eq!(list.totals.qty_received, 20);
    assert_eq!(list.totals.qty_sold, 10);
    // Gross: 1,800 + 1,000 + 3,500. Seller commission 6%: 108 + 60 + 210 (lot nets 2,632 and
    // 3,290). Buyer total: value + 6% + لاگا 8 per نگ: 1,932 + 1,076 + 3,750.
    assert_eq!(list.totals.gross, rupees(6_300));
    assert_eq!(list.totals.net, rupees(2_632 + 3_290));
    assert_eq!(list.totals.buyer_total, rupees(1_932 + 1_076 + 3_750));
    let row = &list.rows[0];
    assert_eq!(
        (
            row.qty_received,
            row.qty_sold,
            row.gross,
            row.net,
            row.buyer_total
        ),
        (
            20,
            10,
            list.totals.gross,
            list.totals.net,
            list.totals.buyer_total
        )
    );
}

#[test]
fn the_stock_list_pages_with_the_balance_over_every_lot() {
    let w = support::world();
    five_bikri(&w);
    let page = |offset: u32, limit: u32| {
        bikri::stock_list(
            &w.env.ctx,
            &StockListInput {
                offset,
                limit,
                ..Default::default()
            },
        )
        .unwrap()
    };
    let whole = page(0, 0);
    // Four lots with 6 left (the cancelled بکری gives its goods back).
    assert_eq!((whole.total_count, whole.rows.len()), (4, 4));
    assert_eq!(whole.balance_total, 24);
    let paged = all_pages(3, |offset, limit| {
        let p = page(offset, limit);
        assert_eq!(p.balance_total, 24, "the balance is over every lot");
        assert!(p.own_rows.is_empty(), "no trip goods in this story");
        (p.rows, p.total_count)
    });
    assert_eq!(paged.len(), 4);
    let ids: Vec<i64> = paged.iter().map(|r| r.lot_id).collect();
    assert_eq!(ids, whole.rows.iter().map(|r| r.lot_id).collect::<Vec<_>>());
    let past = page(40, 3);
    assert!(past.rows.is_empty() && past.own_rows.is_empty());
    assert_eq!((past.total_count, past.balance_total), (4, 24));
}

#[test]
fn the_stock_list_can_ask_for_one_lot_and_a_master_list_for_one_record() {
    let w = support::world();
    five_bikri(&w);
    let lots: Vec<i64> = bikri::stock_list(&w.env.ctx, &StockListInput::default())
        .unwrap()
        .rows
        .iter()
        .map(|r| r.lot_id)
        .collect();
    assert_eq!(lots.len(), 4);
    // One lot: the same answer shape, the count and the balance over that one lot.
    let one = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            lot_id: lots.get(2).copied(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        one.rows.iter().map(|r| r.lot_id).collect::<Vec<_>>(),
        [lots[2]]
    );
    assert_eq!((one.total_count, one.balance_total), (1, 6));
    // A lot that is not in the list (the cancelled بکری's) gives an empty answer.
    let none = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            lot_id: Some(9_999),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(none.rows.is_empty() && none.own_rows.is_empty());
    assert_eq!((none.total_count, none.balance_total), (0, 0));
    // One master: the record asked for, active or not when the screen says so.
    let item = |id: Option<i64>, include_inactive: bool| {
        masters::list_page(
            &w.env.ctx,
            &MastersListInput {
                search: None,
                sort: None,
                kind: MasterKind::Item,
                include_inactive,
                id,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
    };
    let got = item(Some(w.apple), false);
    assert_eq!(
        got.rows.iter().map(|r| r.id()).collect::<Vec<_>>(),
        [w.apple]
    );
    assert_eq!((got.total_count, got.totals.active_count), (1, 1));
    masters::set_active(&w.env.ctx, &w.owner, MasterKind::Item, w.apple, false).unwrap();
    let hidden = item(Some(w.apple), false);
    assert!(hidden.rows.is_empty());
    assert_eq!((hidden.total_count, hidden.totals.active_count), (0, 0));
    let shown = item(Some(w.apple), true);
    assert_eq!(shown.rows.len(), 1);
    assert_eq!((shown.total_count, shown.totals.active_count), (1, 0));
}

// ---- Vouchers ----

fn advance(w: &World, amount: &str) -> AdvanceInput {
    AdvanceInput {
        confirm_cash_below_zero: false,
        date: "10-10-2026".into(),
        seller_account_id: w.rahim,
        amount: amount.into(),
        removed: RemovedPaymentFields::default(),
        expected_goods: String::new(),
        narration: String::new(),
    }
}

fn voucher_page(
    w: &World,
    voucher_type: Option<VoucherType>,
    offset: u32,
    limit: u32,
) -> mandi_services::core::vouchers::VouchersList {
    vouchers::list(
        &w.env.ctx,
        &VouchersListInput {
            sort: None,
            group_by_month: false,
            search: None,
            show_finished: true,
            voucher_type,
            from: String::new(),
            to: String::new(),
            account_id: None,
            person_key: None,
            offset,
            limit,
        },
    )
    .unwrap()
}

#[test]
fn the_list_of_vouchers_pages_with_the_total_over_the_whole_filter() {
    let w = support::world();
    let mut ids = Vec::new();
    for amount in ["1,000", "2,000", "3,000", "4,000", "5,000"] {
        ids.push(
            vouchers::save_advance(&w.env.ctx, &w.owner, &advance(&w, amount))
                .unwrap()
                .voucher_id,
        );
    }
    // The second one (2,000) is cancelled: it stays in the list and out of the total.
    let row = vouchers::get(&w.env.ctx, ids[1]).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: ids[1],
            version: row.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let kind = Some(VoucherType::Advance);
    let whole = voucher_page(&w, kind, 0, 0);
    assert_eq!((whole.total_count, whole.rows.len()), (5, 5));
    assert_eq!(whole.totals.cancelled_count, 1);
    assert_eq!(
        whole.totals.total_paisa,
        Some(rupees(1_000 + 3_000 + 4_000 + 5_000))
    );
    let paged = all_pages(2, |offset, limit| {
        let page = voucher_page(&w, kind, offset, limit);
        assert_eq!(page.totals, whole.totals, "offset {offset}");
        (page.rows, page.total_count)
    });
    assert_eq!(
        paged.iter().map(|r| r.id).collect::<Vec<_>>(),
        whole.rows.iter().map(|r| r.id).collect::<Vec<_>>()
    );
    // The total is the sum of the saved rows of every page.
    assert_eq!(
        whole.totals.total_paisa,
        Some(sum(paged
            .iter()
            .filter(|r| r.status == "saved")
            .map(|r| r.amount_paisa)))
    );
    // All types together have no single total; the count still holds.
    let all = voucher_page(&w, None, 0, 2);
    assert_eq!((all.total_count, all.rows.len()), (5, 2));
    assert_eq!(all.totals.total_paisa, None);
    let past = voucher_page(&w, kind, 99, 2);
    assert!(past.rows.is_empty());
    assert_eq!((past.total_count, past.totals), (5, whole.totals.clone()));
}

// ---- پختہ کھاتہ register ----

fn pukhta_bikri(w: &World, n: i64, name_ur: &str, name_en: &str, phone: &str) {
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
    draft.header.truck_no = Some(format!("PK-{n}"));
    save_bikri(w, draft);
}

#[test]
fn the_pukhta_register_pages_with_totals_over_every_line() {
    let w = support::world();
    pukhta_bikri(&w, 1, "شاہد", "Shahid", "0300-0000001");
    pukhta_bikri(&w, 2, "ناصر", "Nasir", "0300-0000002");
    pukhta_bikri(&w, 3, "بشیر", "Bashir", "0300-0000003");
    let page = |side: Option<PukhtaSide>, offset: u32, limit: u32| {
        bikri::register(
            &w.env.ctx,
            &PukhtaRegisterInput {
                side,
                offset,
                limit,
                ..Default::default()
            },
        )
        .unwrap()
    };
    // All three people are buyers on credit. With one side chosen the totals are given, over
    // every line of the filter.
    let buyers = Some(PukhtaSide::Buyer);
    let whole = page(buyers, 0, 0);
    assert_eq!((whole.total_count, whole.rows.len()), (3, 3));
    assert_eq!(whole.totals.qty, Some(6));
    assert_eq!(
        whole.totals.amount,
        Some(sum(whole.rows.iter().map(|r| r.amount)))
    );
    assert_eq!(
        whole.totals.commission,
        Some(sum(whole.rows.iter().map(|r| r.commission)))
    );
    assert_eq!(
        whole.totals.open,
        Some(sum(whole.rows.iter().map(|r| r.open)))
    );
    assert!(
        whole.totals.open > Some(Paisa::ZERO),
        "three credit buyers owe"
    );
    let paged = all_pages(2, |offset, limit| {
        let p = page(buyers, offset, limit);
        assert_eq!(p.totals, whole.totals, "offset {offset}");
        (p.rows, p.total_count)
    });
    assert_eq!(paged.len(), 3);
    assert_eq!(
        paged.iter().map(|r| r.bikri_number).collect::<Vec<_>>(),
        whole
            .rows
            .iter()
            .map(|r| r.bikri_number)
            .collect::<Vec<_>>()
    );
    let past = page(buyers, 30, 2);
    assert!(past.rows.is_empty());
    assert_eq!((past.total_count, past.totals), (3, whole.totals.clone()));
    // The other side has none of these lines: its count and its totals are 0, not null.
    let sellers = page(Some(PukhtaSide::Seller), 0, 0);
    assert_eq!(sellers.total_count, 0);
    assert_eq!(sellers.totals.qty, Some(0));
    assert_eq!(sellers.totals.open, Some(Paisa::ZERO));
    // No side chosen: the lines of buyers and sellers would be added together, so no total is
    // given at all; the rows and their count still page.
    let mixed = page(None, 0, 2);
    assert_eq!((mixed.total_count, mixed.rows.len()), (3, 2));
    assert_eq!(
        (
            mixed.totals.qty,
            mixed.totals.amount,
            mixed.totals.commission,
            mixed.totals.open
        ),
        (None, None, None, None)
    );
}

// ---- Masters, rules, learned pairs ----

fn masters_page(
    w: &World,
    include_inactive: bool,
    offset: u32,
    limit: u32,
) -> mandi_services::core::masters::MastersList {
    masters::list_page(
        &w.env.ctx,
        &MastersListInput {
            search: None,
            sort: None,
            kind: MasterKind::Item,
            include_inactive,
            id: None,
            offset,
            limit,
        },
    )
    .unwrap()
}

#[test]
fn a_master_list_pages_with_the_counts_over_the_whole_list() {
    let w = support::world();
    // مولی is switched off: it leaves the active list and stays in the full one.
    masters::set_active(&w.env.ctx, &w.owner, MasterKind::Item, w.mooli, false).unwrap();
    let active = masters_page(&w, false, 0, 0);
    let everything = masters_page(&w, true, 0, 0);
    assert_eq!(everything.total_count, active.total_count + 1);
    assert_eq!(everything.totals.active_count, active.total_count);
    assert_eq!(active.totals.active_count, active.total_count);
    let paged = all_pages(3, |offset, limit| {
        let p = masters_page(&w, true, offset, limit);
        assert_eq!(p.totals, everything.totals, "offset {offset}");
        (p.rows, p.total_count)
    });
    assert_eq!(
        paged.iter().map(|r| r.id()).collect::<Vec<_>>(),
        everything.rows.iter().map(|r| r.id()).collect::<Vec<_>>()
    );
    assert_eq!(
        i64::try_from(paged.iter().filter(|r| r.is_active()).count()).unwrap(),
        everything.totals.active_count
    );
    let past = masters_page(&w, true, 500, 3);
    assert!(past.rows.is_empty());
    assert_eq!(
        (past.total_count, past.totals),
        (everything.total_count, everything.totals.clone())
    );
}

fn rule(side_item: Option<i64>, account: Option<i64>, value: &str, active: bool) -> RuleInput {
    RuleInput {
        id: None,
        version: None,
        side: ChargeSide::Seller,
        charge_type: ChargeType::Commission,
        account_id: account,
        item_id: side_item,
        item_group: None,
        method: ChargeMethod::Percent,
        value: value.into(),
        from_date: "01-10-2026".into(),
        to_date: String::new(),
        is_active: active,
        note: String::new(),
    }
}

fn five_rules(w: &World) {
    for (item, account, value, active) in [
        (Some(w.banana), None, "5", true),
        (Some(w.apple), None, "4", true),
        (Some(w.mooli), None, "7", true),
        (None, Some(w.rahim), "3", true),
        (Some(w.weight_fruit), None, "8", false),
    ] {
        rules::save(&w.env.ctx, &w.owner, &rule(item, account, value, active)).unwrap();
    }
}

fn rules_page(w: &World, offset: u32, limit: u32) -> mandi_services::core::rules::RulesList {
    rules::list_page(
        &w.env.ctx,
        &RulesListInput {
            search: None,
            sort: None,
            filter: RulesFilter {
                side: None,
                charge_type: None,
                account_id: None,
                item_id: None,
                include_inactive: true,
            },
            offset,
            limit,
        },
    )
    .unwrap()
}

#[test]
fn the_rule_list_pages_with_the_counts_over_every_match() {
    let w = support::world();
    five_rules(&w);
    let whole = rules_page(&w, 0, 0);
    assert!(whole.total_count >= 5, "the default rules are listed too");
    assert_eq!(
        whole.totals.active_count,
        whole.total_count - 1,
        "one rule is inactive"
    );
    let paged = all_pages(2, |offset, limit| {
        let p = rules_page(&w, offset, limit);
        assert_eq!(p.totals, whole.totals, "offset {offset}");
        (p.rows, p.total_count)
    });
    assert_eq!(
        paged.iter().map(|r| r.id).collect::<Vec<_>>(),
        whole.rows.iter().map(|r| r.id).collect::<Vec<_>>()
    );
    let past = rules_page(&w, 900, 2);
    assert!(past.rows.is_empty());
    assert_eq!(
        (past.total_count, past.totals),
        (whole.total_count, whole.totals.clone())
    );
}

fn learn_five(w: &World) {
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
}

#[test]
fn the_learned_pairs_page_with_the_use_count_over_every_pair() {
    let w = support::world();
    learn_five(&w);
    let (whole, count, uses) = translit::list_page(&w.env.ctx, 0, 0).unwrap();
    assert_eq!(
        (count, whole.len(), uses),
        (5, 5, 6),
        "علی was chosen twice"
    );
    let paged = all_pages(2, |offset, limit| {
        let (rows, count, total_uses) = translit::list_page(&w.env.ctx, offset, limit).unwrap();
        assert_eq!(total_uses, 6, "offset {offset}");
        (rows, count)
    });
    assert_eq!(
        paged.iter().map(|r| r.id).collect::<Vec<_>>(),
        whole.iter().map(|r| r.id).collect::<Vec<_>>()
    );
    assert_eq!(
        paged.iter().map(|r| i64::from(r.use_count)).sum::<i64>(),
        uses
    );
    let (past, count, uses) = translit::list_page(&w.env.ctx, 70, 2).unwrap();
    assert!(past.is_empty());
    assert_eq!((count, uses), (5, 6));
}

// ---- Fixtures of the lists that had none, and of a second page ----

#[test]
fn the_paged_list_answers_are_the_fixtures() {
    let w = support::world();
    let ctx = &w.env.ctx;
    // مولی is switched off; five rules; five learned pairs.
    masters::set_active(ctx, &w.owner, MasterKind::Item, w.mooli, false).unwrap();
    save_bikri(&w, small_bikri(&w, 1));
    five_rules(&w);
    learn_five(&w);

    let request =
        json!({ "kind": "item", "include_inactive": false, "id": null, "offset": 0, "limit": 0 });
    let answer = masters::list_page(ctx, &typed::<MastersListInput>(&request)).unwrap();
    assert_eq!(
        answer.total_count,
        i64::try_from(answer.rows.len()).unwrap()
    );
    pair("masters_list_items", &request, &answer);
    let request =
        json!({ "kind": "item", "include_inactive": true, "id": null, "offset": 2, "limit": 2 });
    let answer = masters::list_page(ctx, &typed::<MastersListInput>(&request)).unwrap();
    assert_eq!(answer.rows.len(), 2);
    assert_eq!(answer.total_count, answer.totals.active_count + 1);
    pair("masters_list_items_page_2", &request, &answer);

    let request = json!({ "kind": "item", "include_inactive": true, "id": w.mooli,
        "offset": 0, "limit": 0 });
    let answer = masters::list_page(ctx, &typed::<MastersListInput>(&request)).unwrap();
    assert_eq!((answer.total_count, answer.rows.len()), (1, 1));
    pair("masters_list_one_item", &request, &answer);

    let lot_id = bikri::get(ctx, 1).unwrap().lots[0].id;
    let request = json!({ "lot_id": lot_id, "offset": 0, "limit": 0 });
    let answer = bikri::stock_list(ctx, &typed::<StockListInput>(&request)).unwrap();
    assert_eq!((answer.total_count, answer.balance_total), (1, 6));
    pair("bikri_stock_list_one_lot", &request, &answer);

    let filter = json!({ "side": null, "charge_type": null, "account_id": null, "item_id": null,
        "include_inactive": true });
    let request = json!({ "filter": filter, "offset": 0, "limit": 0 });
    let answer = rules::list_page(ctx, &typed::<RulesListInput>(&request)).unwrap();
    pair("rules_list_all", &request, &answer);
    let request = json!({ "filter": filter, "offset": 3, "limit": 3 });
    let answer = rules::list_page(ctx, &typed::<RulesListInput>(&request)).unwrap();
    assert_eq!(answer.rows.len(), 3);
    pair("rules_list_all_page_2", &request, &answer);

    for (name, offset, limit) in [
        ("translit_pairs_list", 0_u32, 0_u32),
        ("translit_pairs_list_page_2", 2, 2),
    ] {
        let request = json!({ "offset": offset, "limit": limit });
        let (rows, total_count, use_count) = translit::list_page(ctx, offset, limit).unwrap();
        let answer = json!({
            "rows": rows.iter().map(|r| json!({
                "id": r.id, "roman": r.roman, "urdu": r.urdu, "kind": r.kind,
                "use_count": r.use_count, "last_used_at": r.last_used_at
            })).collect::<Vec<_>>(),
            "total_count": total_count,
            "totals": { "use_count": use_count }
        });
        pair(name, &request, &answer);
    }
}
