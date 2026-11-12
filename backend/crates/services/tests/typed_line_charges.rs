//! Plan 72 P13 (PRD FR-11.10, FR-12.20): a commission or لاگا typed on a sale line, on a new بکری,
//! on an edit and on a later sale. The line is worked out again with what was typed (method
//! manual, source typed); nothing stored or ruled is kept for the typed charge.
//!
//! The numbers are PRD Event 1 (اکرم 20 × 600 = 12,000, ارشد 30 × 580 = 17,400) with the defaults
//! of 6% for both commissions and لاگا 8 per نگ, worked by hand:
//!
//! | Line 1 typed | Seller comm | Buyer comm | لاگا | Buyer pays | Shared income |
//! |---|---|---|---|---|---|
//! | seller 4% | 480 | 720 | 160 | 12,880 | 3,708 |
//! | buyer 4% | 720 | 480 | 160 | 12,640 | 3,708 |
//! | لاگا 10 per نگ | 720 | 720 | 200 | 12,920 | 3,988 |
//! | buyer 4% and لاگا 10 | 720 | 480 | 200 | 12,680 | 3,748 |
//!
//! Shared income is seller commission + buyer commission + لاگا + فون 20 over both lines.
//! Split at 40 / 30 / 30 with the leftover rupee to the largest share: 3,708 gives 1,484 / 1,112 /
//! 1,112; 3,988 gives 1,596 / 1,196 / 1,196; 3,748 gives 1,500 / 1,124 / 1,124.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriDraft, BikriPreview, ChargeSource, EditOpenInput, EditPreviewInput, LaterSaleInput,
    LineChargeInput, LineChargeMethod, StoredMethod,
};
use reports_world::*;
use support::{World, typed};

fn rupees_of(p: &BikriPreview) -> Vec<i64> {
    p.split.iter().map(|s| support::r(s.income_paisa)).collect()
}

/// The typed charges of line 1 of Event 1.
#[derive(Clone, Default)]
struct Typed {
    seller: Option<LineChargeInput>,
    buyer: Option<LineChargeInput>,
    laga: Option<LineChargeInput>,
}

fn with_typed(mut draft: BikriDraft, t: &Typed) -> BikriDraft {
    let row = &mut draft.lots[0].sales[0];
    row.seller_comm = t.seller.clone();
    row.buyer_comm = t.buyer.clone();
    row.laga = t.laga.clone();
    draft
}

/// What line 1 of a preview shows: (seller comm, buyer comm, لاگا, buyer total) in rupees.
fn line_1(p: &BikriPreview) -> Line {
    let c = support::calc(&p.lots[0], 0);
    (
        support::r(c.seller_comm.amount),
        support::r(c.buyer_comm.amount),
        support::r(c.laga.amount),
        support::r(c.buyer_total),
    )
}

/// A typed charge of a line is manual and typed; an untyped one is a default.
fn source_of(p: &BikriPreview) -> [(StoredMethod, ChargeSource); 3] {
    let c = support::calc(&p.lots[0], 0);
    [
        (c.seller_comm.method, c.seller_comm.source),
        (c.buyer_comm.method, c.buyer_comm.source),
        (c.laga.method, c.laga.source),
    ]
}

const MANUAL_TYPED: (StoredMethod, ChargeSource) = (StoredMethod::Manual, ChargeSource::Typed);

/// Seller comm, buyer comm, لاگا and buyer total of line 1, in rupees.
type Line = (i64, i64, i64, i64);

/// A case: its name, the typed charges, line 1, the shared income and the split of it.
type Case = (&'static str, Typed, Line, i64, Vec<i64>);

fn cases() -> Vec<Case> {
    let seller_4 = typed(LineChargeMethod::Percent, "4");
    let buyer_4 = typed(LineChargeMethod::Percent, "4");
    let laga_10 = typed(LineChargeMethod::PerUnit, "10");
    vec![
        (
            "seller 4%",
            Typed {
                seller: seller_4,
                ..Typed::default()
            },
            (480, 720, 160, 12_880),
            3_708,
            vec![1_484, 1_112, 1_112],
        ),
        (
            "buyer 4%",
            Typed {
                buyer: buyer_4.clone(),
                ..Typed::default()
            },
            (720, 480, 160, 12_640),
            3_708,
            vec![1_484, 1_112, 1_112],
        ),
        (
            "laga 10 per nag",
            Typed {
                laga: laga_10.clone(),
                ..Typed::default()
            },
            (720, 720, 200, 12_920),
            3_988,
            vec![1_596, 1_196, 1_196],
        ),
        (
            "buyer 4% and laga 10 per nag",
            Typed {
                buyer: buyer_4,
                laga: laga_10,
                ..Typed::default()
            },
            (720, 480, 200, 12_680),
            3_748,
            vec![1_500, 1_124, 1_124],
        ),
    ]
}

fn marks(t: &Typed) -> [Option<(StoredMethod, ChargeSource)>; 3] {
    [
        t.seller.as_ref().map(|_| MANUAL_TYPED),
        t.buyer.as_ref().map(|_| MANUAL_TYPED),
        t.laga.as_ref().map(|_| MANUAL_TYPED),
    ]
}

/// Every typed charge shows as manual and typed; an untyped one does not.
fn assert_marks(p: &BikriPreview, t: &Typed, what: &str) {
    let shown = source_of(p);
    for (index, expected) in marks(t).iter().enumerate() {
        match expected {
            Some(mark) => assert_eq!(shown[index], *mark, "{what}: charge {index} is not typed"),
            None => assert_ne!(
                shown[index].1,
                ChargeSource::Typed,
                "{what}: charge {index} became typed"
            ),
        }
    }
}

#[test]
fn a_new_sale_works_out_a_typed_buyer_commission_and_laga() {
    // (1) and (2): the same four cases through the preview of a new بکری.
    let w = worked_day();
    for (what, t, line, shared, split) in cases() {
        let p = support::preview(&w, &with_typed(support::event1(&w), &t));
        assert!(p.errors.is_empty(), "{what}: {:?}", p.errors);
        assert_eq!(line_1(&p), line, "{what}");
        assert_marks(&p, &t, what);
        assert_eq!(support::r(p.summary.shared_income), shared, "{what}");
        assert_eq!(rupees_of(&p), split, "{what}");
        // The second line is not touched.
        let c = support::calc(&p.lots[0], 1);
        assert_eq!(
            (
                support::r(c.seller_comm.amount),
                support::r(c.buyer_comm.amount),
                support::r(c.laga.amount),
                support::r(c.buyer_total)
            ),
            (1_044, 1_044, 240, 18_684),
            "{what}"
        );
    }
}

#[test]
fn a_typed_charge_is_saved_as_typed_and_a_reopened_edit_gives_the_same_numbers() {
    // Save with the typed charges, open the saved بکری for an edit and preview it as it came:
    // the draft carries the typed values back, so nothing changes.
    for (what, t, line, shared, split) in cases() {
        let w2 = worked_day();
        let saved = save(&w2, &with_typed(support::event1(&w2), &t));
        assert_eq!(
            support::r(saved.preview.summary.shared_income),
            shared,
            "{what}"
        );
        let opened = bikri::open_for_edit(
            &w2.env.ctx,
            &EditOpenInput {
                bikri_id: saved.bikri_id,
                entry_id: saved.entry_id,
            },
        )
        .unwrap();
        let p = bikri::edit_preview(
            &w2.env.ctx,
            &EditPreviewInput {
                entry_id: opened.entry_id,
                draft: opened.draft.clone(),
            },
        )
        .unwrap();
        assert!(p.errors.is_empty(), "{what}: {:?}", p.errors);
        assert_eq!(line_1(&p), line, "{what}: reopened");
        assert_eq!(rupees_of(&p), split, "{what}: reopened");
    }
}

#[test]
fn an_edit_that_only_types_a_charge_on_a_saved_line_works_it_out_again() {
    // (3): the buyer and the seller are as saved, the line keeps its `sale_id`, and only a charge is
    // typed. The line is a changed line, so the stored rule charge is not kept.
    for (what, t, line, shared, split) in cases() {
        let w = worked_day();
        let saved = save(&w, &support::event1(&w));
        let opened = bikri::open_for_edit(
            &w.env.ctx,
            &EditOpenInput {
                bikri_id: saved.bikri_id,
                entry_id: saved.entry_id,
            },
        )
        .unwrap();
        let mut draft = opened.draft.clone();
        assert!(draft.lots[0].sales[0].sale_id.is_some());
        draft.lots[0].sales[0].seller_comm = t.seller.clone();
        draft.lots[0].sales[0].buyer_comm = t.buyer.clone();
        draft.lots[0].sales[0].laga = t.laga.clone();
        let p = bikri::edit_preview(
            &w.env.ctx,
            &EditPreviewInput {
                entry_id: opened.entry_id,
                draft: draft.clone(),
            },
        )
        .unwrap();
        assert!(p.errors.is_empty(), "{what}: {:?}", p.errors);
        assert_eq!(line_1(&p), line, "{what}");
        assert_marks(&p, &t, what);
        assert_eq!(support::r(p.summary.shared_income), shared, "{what}");
        assert_eq!(rupees_of(&p), split, "{what}");
        // The other line keeps what it had.
        let c = support::calc(&p.lots[0], 1);
        assert_eq!(
            (
                support::r(c.seller_comm.amount),
                support::r(c.buyer_comm.amount),
                support::r(c.laga.amount)
            ),
            (1_044, 1_044, 240),
            "{what}"
        );
        // The save writes what the preview showed (Code Standards B5).
        let edited = bikri::edit_save(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::bikri::EditSaveInput {
                entry_id: opened.entry_id,
                version: opened.version,
                draft,
                reason: Some("ریٹ درست کیا".into()),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        assert_eq!(edited.preview, p, "{what}");
    }
}

#[test]
fn a_typed_charge_that_is_changed_again_in_a_later_edit_follows_the_new_value() {
    // The line was saved with a typed 4% seller commission. A later edit types 5% on it: 600.
    let w = worked_day();
    let t = Typed {
        seller: typed(LineChargeMethod::Percent, "4"),
        ..Typed::default()
    };
    let saved = save(&w, &with_typed(support::event1(&w), &t));
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_comm = typed(LineChargeMethod::Percent, "5");
    let p = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft,
        },
    )
    .unwrap();
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert_eq!(line_1(&p).0, 600);
}

/// Event 1 with 45 of the 50 sold (ارشد's line cut to 25), saved: 5 نگ are left in the lot.
fn part_sold(w: &World) -> i64 {
    let mut draft = support::event1(w);
    draft.lots[0].sales[1].qty = "25".into();
    let saved = save(w, &draft);
    assert!(saved.preview.errors.is_empty());
    let stock = bikri::stock_list(
        &w.env.ctx,
        &mandi_services::core::bikri::StockListInput::default(),
    )
    .unwrap();
    assert_eq!(stock.rows[0].balance, 5);
    stock.rows[0].lot_id
}

#[test]
fn a_later_sale_works_out_typed_charges() {
    // (4): 5 نگ at 560 = 2,800. By the defaults: 168, 168 and 40 (E2E-16b). Typed: seller 4% = 112,
    // buyer 4% = 112, لاگا 10 per نگ = 50; the buyer pays 2,800 + 112 + 50 = 2,962.
    let w = worked_day();
    let lot_id = part_sold(&w);
    set_today(&w.env, 2026, 10, 11);
    let later = |seller: Option<LineChargeInput>, buyer: Option<LineChargeInput>, laga| {
        let mut line = support::sale("s-3", w.arshad, "5", "560");
        line.seller_comm = seller;
        line.buyer_comm = buyer;
        line.laga = laga;
        bikri::later_sale_preview(
            &w.env.ctx,
            &LaterSaleInput {
                lot_id,
                date: "11-10-2026".into(),
                sales: vec![line],
            },
        )
        .unwrap()
    };
    let figures = |p: &BikriPreview| {
        assert!(p.errors.is_empty(), "{:?}", p.errors);
        let c = support::calc(&p.lots[0], 0);
        (
            support::r(c.seller_comm.amount),
            support::r(c.buyer_comm.amount),
            support::r(c.laga.amount),
            support::r(c.buyer_total),
        )
    };
    assert_eq!(figures(&later(None, None, None)), (168, 168, 40, 3_008));
    let p = later(
        typed(LineChargeMethod::Percent, "4"),
        typed(LineChargeMethod::Percent, "4"),
        typed(LineChargeMethod::PerUnit, "10"),
    );
    assert_eq!(figures(&p), (112, 112, 50, 2_962));
    let c = support::calc(&p.lots[0], 0);
    for charge in [&c.seller_comm, &c.buyer_comm, &c.laga] {
        assert_eq!((charge.method, charge.source), MANUAL_TYPED);
    }
    // Each one alone.
    assert_eq!(
        figures(&later(typed(LineChargeMethod::Percent, "4"), None, None)),
        (112, 168, 40, 3_008)
    );
    assert_eq!(
        figures(&later(None, typed(LineChargeMethod::Percent, "4"), None)),
        (168, 112, 40, 2_952)
    );
    assert_eq!(
        figures(&later(None, None, typed(LineChargeMethod::PerUnit, "10"))),
        (168, 168, 50, 3_018)
    );
}
