//! Plan 71 (PRD FR-12.45, FR-12.12, E2E-70, decisions L1 to L8): the name of a lot is its seller, its
//! item and a counter that belongs to the seller and the item for his whole life.
//!
//! - E2E-70: lots typed آم, کیلا, آم show 1, 1, 2 before and after the save; his next بکری's آم lot is
//!   3; the seller bill (R1) prints آم-1, آم-2, کیلا-1 in Urdu, and Banana, Mango in English.
//! - L2: a cancel never frees a number.
//! - L5: an edit that moves a lot to another seller gives it his next number; the old number stays
//!   unused.
//! - L7: the seller's day slip (R35) is sorted the same way, with the lot's name on each line.
//! - L8: the stock list, the list of بکری, R6 and R10 carry the counter.
//! - A later sale keeps the lot's number.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::print::{BikriPrintDataInput, bikri_print_data};
use mandi_reports::reports::{Cell, ReportFilters, ReportId, ReportRunInput, run as run_report};
use mandi_services::bikri;
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, BikriSaved, CancelInput, EditOpenInput, EditPreviewInput,
    EditSaveInput, LotInput,
};
use mandi_services::core::printing::PrintDoc;
use support::{World, add_item, draft, lot, sale, with_sales, world};

/// A lot with one sale line of its whole quantity, so that every lot of a draft has a line.
fn sold(w: &World, key: &str, seller: i64, item: i64) -> LotInput {
    with_sales(
        lot(w, key, seller, item, "10"),
        vec![sale(&format!("s-{key}"), w.akram, "10", "600")],
    )
}

fn save(w: &World, d: &BikriDraft) -> BikriSaved {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: d.clone(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
}

/// The counters of the lots of a draft, as the preview shows them.
fn previewed(w: &World, d: &BikriDraft) -> Vec<Option<i64>> {
    bikri::preview(&w.env.ctx, d)
        .unwrap()
        .lots
        .iter()
        .map(|l| l.lot_counter)
        .collect()
}

/// The counters of the lots of a saved بکری, in lot order.
fn stored(w: &World, bikri_id: i64) -> Vec<i64> {
    bikri::get(&w.env.ctx, bikri_id)
        .unwrap()
        .lots
        .iter()
        .map(|l| l.lot_counter)
        .collect()
}

/// The items of the lines of a printed seller bill, in the order printed.
fn bill_items(w: &World, bikri_id: i64, language: Language) -> Vec<String> {
    let data = bikri_print_data(
        w.env.ctx.db(),
        &BikriPrintDataInput {
            bikri_id,
            doc: PrintDoc::SellerBill,
            language,
            only_buyer_id: None,
        },
    )
    .unwrap();
    assert_eq!(data.pages.len(), 1, "one seller, one page");
    data.pages[0].lines.iter().map(|l| l.item.clone()).collect()
}

#[test]
fn e2e_70_the_counter_belongs_to_the_seller_and_the_item_and_the_seller_bill_is_sorted() {
    let w = world();
    let mango = add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "unit");
    // One بکری of رحیم with lots typed آم, کیلا, آم.
    let d = draft(vec![
        sold(&w, "a", w.rahim, mango),
        sold(&w, "b", w.rahim, w.banana),
        sold(&w, "c", w.rahim, mango),
    ]);
    assert_eq!(
        previewed(&w, &d),
        [Some(1), Some(1), Some(2)],
        "before the save"
    );
    let first = save(&w, &d);
    assert_eq!(stored(&w, first.bikri_id), [1, 1, 2], "after the save");
    // The preview of the save is the saved lots' numbers (Code Standards B5).
    assert_eq!(
        first
            .preview
            .lots
            .iter()
            .map(|l| l.lot_counter)
            .collect::<Vec<_>>(),
        [Some(1), Some(1), Some(2)]
    );
    // His next بکری's آم lot is آم-3, his next کیلا lot کیلا-2, and کریم starts at 1.
    let next = draft(vec![
        sold(&w, "d", w.rahim, mango),
        sold(&w, "e", w.rahim, w.banana),
        sold(&w, "f", w.karim, mango),
    ]);
    assert_eq!(previewed(&w, &next), [Some(3), Some(2), Some(1)]);
    let second = save(&w, &next);
    assert_eq!(stored(&w, second.bikri_id), [3, 2, 1]);

    // R1: sorted by item (Urdu: آم before کیلا; English: Banana before Mango), then by counter.
    assert_eq!(
        bill_items(&w, first.bikri_id, Language::Ur),
        ["آم-1", "آم-2", "کیلا-1"]
    );
    assert_eq!(
        bill_items(&w, first.bikri_id, Language::En),
        ["Banana-1", "Mango-1", "Mango-2"]
    );
}

#[test]
fn a_cancel_never_frees_a_number() {
    let w = world();
    let first = save(&w, &draft(vec![sold(&w, "a", w.rahim, w.banana)]));
    assert_eq!(stored(&w, first.bikri_id), [1]);
    let version = bikri::get(&w.env.ctx, first.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: first.bikri_id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // The cancelled lot keeps its number 1; the next lot of the same seller and item is 2.
    assert_eq!(stored(&w, first.bikri_id), [1]);
    let d = draft(vec![sold(&w, "b", w.rahim, w.banana)]);
    assert_eq!(previewed(&w, &d), [Some(2)]);
    assert_eq!(stored(&w, save(&w, &d).bikri_id), [2]);
}

#[test]
fn an_edit_that_moves_a_lot_gives_it_the_next_number_of_the_new_seller_and_leaves_the_old_one_unused()
 {
    let w = world();
    let mango = add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "unit");
    let first = save(
        &w,
        &draft(vec![
            sold(&w, "a", w.rahim, mango),
            sold(&w, "b", w.rahim, mango),
        ]),
    );
    assert_eq!(stored(&w, first.bikri_id), [1, 2]);
    // کریم already has two آم lots.
    let karims = save(
        &w,
        &draft(vec![
            sold(&w, "c", w.karim, mango),
            sold(&w, "d", w.karim, mango),
        ]),
    );
    assert_eq!(stored(&w, karims.bikri_id), [1, 2]);
    // The second lot of the first بکری is moved to کریم: it becomes his آم-3.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: first.bikri_id,
            entry_id: first.entry_id,
        },
    )
    .unwrap();
    let mut edited = opened.draft.clone();
    edited.lots[1].seller_account_id = Some(w.karim);
    let before = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: edited.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        before
            .lots
            .iter()
            .map(|l| l.lot_counter)
            .collect::<Vec<_>>(),
        [Some(1), Some(3)],
        "the lot that stays keeps its number, the lot that moves takes the next of کریم and آم"
    );
    let done = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft: edited,
            reason: Some("edit".into()),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_eq!(done.preview, before, "the preview is the save");
    assert_eq!(stored(&w, first.bikri_id), [1, 3]);
    // رحیم's old number 2 stays unused: his next آم lot is 3, not 2.
    let next = draft(vec![sold(&w, "e", w.rahim, mango)]);
    assert_eq!(previewed(&w, &next), [Some(3)]);
    assert_eq!(stored(&w, save(&w, &next).bikri_id), [3]);
}

#[test]
fn a_later_sale_keeps_the_number_of_its_lot_and_the_lists_carry_it() {
    let w = world();
    let mango = add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "unit");
    // Two lots, the second one only part sold, so it waits in the stock list.
    let a = sold(&w, "a", w.rahim, mango);
    let b = with_sales(
        lot(&w, "b", w.rahim, mango, "10"),
        vec![sale("s-b", w.akram, "4", "600")],
    );
    let first = save(&w, &draft(vec![a, b]));
    assert_eq!(stored(&w, first.bikri_id), [1, 2]);
    // The list of goods left names the lot آم-2.
    let stock = bikri::stock_list(
        &w.env.ctx,
        &mandi_services::core::bikri::StockListInput::default(),
    )
    .unwrap();
    assert_eq!(stock.rows.len(), 1);
    assert_eq!(stock.rows[0].lot_counter, 2);
    // The list of بکری names its lots.
    let list = bikri::list(
        &w.env.ctx,
        &mandi_services::core::bikri::BikriListInput::default(),
    )
    .unwrap();
    let counters: Vec<i64> = list.rows[0].lots.iter().map(|l| l.counter).collect();
    assert_eq!(counters, [1, 2]);
    // R6 and R10 write the item with the counter.
    let r6 = run_report(
        w.env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R6,
            filters: ReportFilters::default(),
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap();
    let items: Vec<String> = r6
        .rows
        .iter()
        .map(|r| match &r.cells[3] {
            Cell::Names { names } => names.en.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(items, ["Mango-1", "Mango-2"]);
}

fn run(w: &World, id: ReportId, filters: ReportFilters) -> mandi_reports::reports::ReportPage {
    run_report(
        w.env.ctx.db(),
        &ReportRunInput {
            id,
            filters,
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap()
}

#[test]
fn a_report_can_be_narrowed_to_one_lot_of_a_seller_and_an_item() {
    // Plan 71 L9: after "کون سا؟" the assistant reads the reports of the lot the Owner picked. R2 to
    // R6, R10 and R27 take `lot_id`; every other report refuses it.
    let w = world();
    let mango = add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "unit");
    // آم-1 sells 10 to اکرم; آم-2 sells 4 to اکرم and has 6 left.
    let a = sold(&w, "a", w.rahim, mango);
    let b = with_sales(
        lot(&w, "b", w.rahim, mango, "10"),
        vec![sale("s-b", w.akram, "4", "600")],
    );
    let first = save(&w, &draft(vec![a, b]));
    let lot_ids: Vec<i64> = bikri::find_lots(
        &w.env.ctx,
        &mandi_services::core::bikri::LotFindInput {
            seller_ids: vec![w.rahim],
            item_ids: vec![mango],
            limit: 10,
            ..Default::default()
        },
    )
    .unwrap()
    .0
    .iter()
    .map(|l| l.name.lot_id)
    .collect();
    assert_eq!(lot_ids.len(), 2, "{first:?}");
    let by_counter = |counter: i64| {
        bikri::find_lots(
            &w.env.ctx,
            &mandi_services::core::bikri::LotFindInput {
                seller_ids: vec![w.rahim],
                item_ids: vec![mango],
                counter: Some(counter),
                limit: 1,
                ..Default::default()
            },
        )
        .unwrap()
        .0[0]
            .name
            .lot_id
    };
    let (lot1, lot2) = (by_counter(1), by_counter(2));
    let filters = |lot_id: Option<i64>| ReportFilters {
        seller_id: Some(w.rahim),
        item_id: Some(mango),
        lot_id,
        ..ReportFilters::default()
    };
    // R6: one row for each lot, so the lot filter leaves one.
    assert_eq!(run(&w, ReportId::R6, filters(None)).rows.len(), 2);
    let r6 = run(&w, ReportId::R6, filters(Some(lot2)));
    assert_eq!(r6.rows.len(), 1);
    assert_eq!(
        r6.filters.lot.as_ref().map(|n| n.en.as_str()),
        Some("Rahim, Mango-2")
    );
    // R2: one line for each sale line.
    assert_eq!(run(&w, ReportId::R2, filters(None)).rows.len(), 2);
    assert_eq!(run(&w, ReportId::R2, filters(Some(lot1))).rows.len(), 1);
    // R5: the buyers of the lot, and the totals are of the lot only: 4 × 600 = 2,400 of value.
    let r5_all = run(&w, ReportId::R5, filters(None));
    let r5_lot = run(&w, ReportId::R5, filters(Some(lot2)));
    assert_eq!(r5_all.rows.len(), 1, "one buyer");
    assert_ne!(
        format!("{:?}", r5_all.totals),
        format!("{:?}", r5_lot.totals),
        "the totals follow the lot"
    );
    // R10: the stock of the lot with goods left; the other lot is sold out and is not listed either way.
    assert_eq!(run(&w, ReportId::R10, filters(Some(lot2))).rows.len(), 1);
    assert_eq!(run(&w, ReportId::R10, filters(Some(lot1))).rows.len(), 0);
    // A report with no lot filter refuses it, with its field; a lot that does not exist is not found.
    let refused = run_report(
        w.env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R7,
            filters: filters(Some(lot1)),
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
        },
    );
    assert!(
        matches!(&refused, Err(mandi_reports::ReportsError::Rejected(items))
            if items.iter().any(|i| i.field == "filters.lot_id")),
        "{refused:?}"
    );
    let missing = run_report(
        w.env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R6,
            filters: filters(Some(999_999)),
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
        },
    );
    assert!(
        matches!(&missing, Err(mandi_reports::ReportsError::Rejected(items))
            if items.iter().any(|i| i.field == "filters.lot_id")),
        "{missing:?}"
    );
}
