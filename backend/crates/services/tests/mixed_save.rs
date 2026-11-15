//! Plan 76 point 1 (PRD FR-12.17 v1.41): carried lots and new lots in one save.
//!
//! The state is E2E-16: رحیم's 50 کیلا, 45 sold (اکرم 20 at 600, ارشد 25 at 580), cash 45,000, 5 نگ
//! left. On 11-10-26 one screen sells the 5 to ارشد at 560 (a later posting of بکری 1: value 2,800,
//! entry 3,008, shares 152 / 112 / 112) and adds کریم's 10 سیب at 1,200 to a چکتا buyer شاہد (a new
//! بکری 2: entry 25,600, shares 616 / 462 / 462). Together: shared income 1,916, partners
//! 768 / 574 / 574, cash 45,000 + 12,800 = 57,800.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::{raw, set_today};
use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriFindInput, BikriListInput, BikriSaveInput, CarriedLotInput, ChuktaInput,
    LaterSaleSaveInput, LotInput, MixedPartKind, MixedSaleInput, MixedSaleSaveInput,
    MixedUndoInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::ledger;
use support::{World, rupees};

/// E2E-16: Event 1 with only 45 of the 50 sold. Returns the carried lot's id.
fn e2e16(w: &World) -> i64 {
    let draft = support::draft(vec![support::with_sales(
        support::lot(w, "lot-1", w.rahim, w.banana, "50"),
        vec![
            support::sale("s-1", w.akram, "20", "600"),
            support::sale("s-2", w.arshad, "25", "580"),
        ],
    )]);
    let saved = bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    bikri::get(&w.env.ctx, saved.bikri_id)
        .unwrap()
        .lots
        .first()
        .unwrap()
        .id
}

fn carried(w: &World, lot_id: i64) -> CarriedLotInput {
    CarriedLotInput {
        key: String::new(),
        lot_id,
        sales: vec![support::sale("c-1", w.arshad, "5", "560")],
    }
}

/// کریم's 10 سیب, freight 1,000, sold to the چکتا buyer شاہد at 1,200.
fn new_lot(w: &World) -> LotInput {
    let mut shahid = support::sale("n-1", w.chukta_khata, "10", "1200");
    shahid.chukta = Some(ChuktaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = support::lot(w, "new-1", w.karim, w.apple, "10");
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    support::with_sales(lot, vec![shahid])
}

fn mixed(w: &World, lot_id: i64) -> MixedSaleInput {
    MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![carried(w, lot_id)],
        new_lots: vec![new_lot(w)],
    }
}

fn save_input(input: &MixedSaleInput) -> MixedSaleSaveInput {
    MixedSaleSaveInput {
        date: input.date.clone(),
        carried: input.carried.clone(),
        new_lots: input.new_lots.clone(),
        confirm_cash_below_zero: false,
    }
}

/// The lines of an entry as (account code, debit, credit), sorted.
fn lines_of(w: &World, entry_id: i64) -> Vec<(String, i64, i64)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT a.code, l.dr_paisa, l.cr_paisa FROM journal_lines l
             JOIN accounts a ON a.id = l.account_id WHERE l.entry_id = ?1
             ORDER BY a.code, l.dr_paisa, l.cr_paisa",
        )
        .unwrap();
    statement
        .query_map([entry_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// The share rows of an entry as (partner id, kind, amount), sorted.
fn shares_of(w: &World, entry_id: i64) -> Vec<(i64, String, i64)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT partner_id, kind, amount_paisa FROM partner_share_rows
             WHERE journal_entry_id = ?1 ORDER BY partner_id, kind, amount_paisa",
        )
        .unwrap();
    statement
        .query_map([entry_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn total_of(w: &World, entry_id: i64) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT total_paisa FROM journal_entries WHERE id = ?1",
            [entry_id],
            |r| r.get(0),
        )
        .unwrap()
}

fn cash_now(w: &World) -> i64 {
    // The preview of an empty change reads cash in hand.
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![],
        new_lots: vec![],
    };
    let p = bikri::mixed_preview(&w.env.ctx, &input).unwrap();
    p.preview.cash.cash_before_paisa.as_paisa()
}

#[test]
fn the_preview_has_both_parts_and_the_numbers_of_the_plan() {
    let w = support::world();
    let lot_id = e2e16(&w);
    assert_eq!(cash_now(&w), 45_000 * 100);
    let p = bikri::mixed_preview(&w.env.ctx, &mixed(&w, lot_id)).unwrap();
    assert!(p.preview.errors.is_empty(), "{:?}", p.preview.errors);
    // Carried first, then new; each entry has its own figures.
    assert_eq!(p.parts.len(), 2);
    assert_eq!(p.parts[0].kind, MixedPartKind::Carried);
    assert_eq!(p.parts[0].old_number, Some(1));
    assert_eq!(p.parts[1].kind, MixedPartKind::New);
    assert_eq!(p.preview.lots.len(), 2);
    assert_eq!(p.preview.lots[0].key, format!("lot:{lot_id}"));
    assert_eq!(p.preview.lots[1].key, "new-1");
    let shares = |s: &[mandi_services::core::bikri::SplitPart]| -> Vec<i64> {
        s.iter().map(|x| x.income_paisa.as_paisa() / 100).collect()
    };
    assert_eq!(shares(&p.parts[0].split), vec![152, 112, 112]);
    assert_eq!(shares(&p.parts[1].split), vec![616, 462, 462]);
    assert_eq!(shares(&p.preview.split), vec![768, 574, 574]);
    assert_eq!(p.preview.summary.shared_income, rupees(1_916));
    assert_eq!(p.parts[0].summary.buyer_total, rupees(3_008));
    assert_eq!(p.parts[1].summary.buyer_total, rupees(12_800));
    // Cash: only the new part moves it (12,800 paid on the spot).
    assert_eq!(p.preview.cash.cash_after_paisa, rupees(57_800));
    assert_eq!(p.preview.cash.change_paisa, rupees(12_800));
}

#[test]
fn the_save_writes_a_later_entry_and_a_new_bikri_like_the_standalone_paths() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let input = mixed(&w, lot_id);
    let before = bikri::mixed_preview(&w.env.ctx, &input).unwrap();
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap();
    // The preview just before is what the save wrote (Code Standards B5).
    assert_eq!(saved.preview, before.preview);
    assert_eq!(saved.new_bikri_number, Some(2));
    assert_eq!(saved.carried.len(), 1);
    assert_eq!(saved.carried[0].old_number, 1);
    assert_eq!(saved.date, "2026-10-11");
    assert_eq!(saved.entry_ids.len(), 2);
    let (a, b) = (saved.entry_ids[0], saved.entry_ids[1]);
    assert_eq!((total_of(&w, a), total_of(&w, b)), (300_800, 2_560_000));
    // Shares per partner: 152 / 112 / 112 and 616 / 462 / 462.
    let amounts = |entry| -> Vec<i64> {
        shares_of(&w, entry)
            .iter()
            .map(|(_, _, amount)| amount / 100)
            .collect()
    };
    assert_eq!(amounts(a), vec![152, 112, 112]);
    assert_eq!(amounts(b), vec![616, 462, 462]);
    // The books balance and cash is 57,800.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    assert_eq!(cash_now(&w), 57_800 * 100);

    // The same two saves done one after the other on a fresh world write the same lines.
    let w2 = support::world();
    let lot2 = e2e16(&w2);
    let later = bikri::later_sale_save(
        &w2.env.ctx,
        &w2.owner,
        &LaterSaleSaveInput {
            lot_id: lot2,
            date: "11-10-2026".into(),
            sales: vec![support::sale("c-1", w2.arshad, "5", "560")],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let mut first = support::draft(vec![new_lot(&w2)]);
    first.header.entry_date = "11-10-2026".into();
    let fresh = bikri::save(
        &w2.env.ctx,
        &w2.owner,
        &BikriSaveInput {
            draft: first,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(lines_of(&w, a), lines_of(&w2, later.entry_id));
    assert_eq!(lines_of(&w, b), lines_of(&w2, fresh.entry_id));
    let strip = |v: Vec<(i64, String, i64)>| -> Vec<(String, i64)> {
        v.into_iter().map(|(_, k, a)| (k, a)).collect()
    };
    assert_eq!(
        strip(shares_of(&w, a)),
        strip(shares_of(&w2, later.entry_id))
    );
    assert_eq!(
        strip(shares_of(&w, b)),
        strip(shares_of(&w2, fresh.entry_id))
    );
}

#[test]
fn a_carried_only_save_makes_no_new_number() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![carried(&w, lot_id)],
        new_lots: vec![],
    };
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap();
    assert_eq!(saved.new_bikri_number, None);
    assert_eq!(saved.new_bikri_id, None);
    assert_eq!(saved.entry_ids.len(), 1);
    // Only carried lots: the print covers the old بکری of the lot, and nothing else.
    assert_eq!(saved.print_bikri_ids, vec![saved.carried[0].old_bikri_id]);
    // Entry A of E2E-16b: 3,008, shares 152 / 112 / 112; nothing left in the lot.
    assert_eq!(total_of(&w, saved.entry_ids[0]), 300_800);
    let detail = bikri::get(&w.env.ctx, saved.carried[0].old_bikri_id).unwrap();
    assert_eq!(detail.lots[0].qty_sold, 50);
}

#[test]
fn a_new_only_save_is_a_plain_new_bikri() {
    let w = support::world();
    e2e16(&w);
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![],
        new_lots: vec![new_lot(&w)],
    };
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap();
    assert_eq!(saved.new_bikri_number, Some(2));
    assert!(saved.carried.is_empty());
    assert_eq!(saved.entry_ids.len(), 1);
}

#[test]
fn a_carried_lot_without_lines_writes_nothing() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![CarriedLotInput {
            key: String::new(),
            lot_id,
            sales: vec![support::blank_sale("c-1", "5")],
        }],
        new_lots: vec![new_lot(&w)],
    };
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap();
    assert!(saved.carried.is_empty());
    assert_eq!(saved.entry_ids.len(), 1);
}

#[test]
fn mistakes_come_back_at_their_fields() {
    let w = support::world();
    let lot_id = e2e16(&w);
    // The same lot twice, a quantity over what is left, a lot that does not exist.
    let over = CarriedLotInput {
        key: String::new(),
        lot_id,
        sales: vec![support::sale("c-1", w.arshad, "6", "560")],
    };
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![
            over.clone(),
            over,
            CarriedLotInput {
                key: String::new(),
                lot_id: 99_999,
                sales: vec![support::sale("c-2", w.arshad, "1", "560")],
            },
        ],
        new_lots: vec![],
    };
    let p = bikri::mixed_preview(&w.env.ctx, &input).unwrap();
    assert!(support::has(
        &p.preview.errors,
        ErrorCode::ErrQtyExceedsLot,
        "carried[0].sales[0].qty"
    ));
    assert!(support::has(
        &p.preview.errors,
        ErrorCode::ErrDuplicateLot,
        "carried[1].lot_id"
    ));
    assert!(support::has(
        &p.preview.errors,
        ErrorCode::ErrLotNotFound,
        "carried[2].lot_id"
    ));
    // The save returns the same errors and writes nothing.
    let before = ledger::trial_balance(&w.env.ctx, None).unwrap().total_dr;
    assert!(bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).is_err());
    assert_eq!(
        ledger::trial_balance(&w.env.ctx, None).unwrap().total_dr,
        before
    );
    // Nothing at all: one clear error on `carried`.
    let empty = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![],
        new_lots: vec![],
    };
    let p = bikri::mixed_preview(&w.env.ctx, &empty).unwrap();
    assert!(support::has(
        &p.preview.errors,
        ErrorCode::ErrNoSaleLines,
        "carried"
    ));
    // A new lot's mistake carries the `new_lots[k]` path.
    let mut bad = new_lot(&w);
    bad.sales[0].qty = "11".into();
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![],
        new_lots: vec![bad],
    };
    let p = bikri::mixed_preview(&w.env.ctx, &input).unwrap();
    assert!(support::has(
        &p.preview.errors,
        ErrorCode::ErrQtyExceedsLot,
        "new_lots[0].sales[0].qty"
    ));
}

#[test]
fn undo_takes_back_the_later_posting_and_the_new_bikri_in_one_go() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let input = mixed(&w, lot_id);
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap();
    let undone = bikri::mixed_undo(
        &w.env.ctx,
        &w.owner,
        &MixedUndoInput {
            later_entry_ids: vec![saved.carried[0].entry_id],
            new_bikri_id: saved.new_bikri_id,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(undone.reversal_entry_ids.len(), 2);
    // Back to the E2E-16 state: cash 45,000, 5 نگ left in the lot, بکری 2 cancelled.
    assert_eq!(cash_now(&w), 45_000 * 100);
    let detail = bikri::get(&w.env.ctx, saved.carried[0].old_bikri_id).unwrap();
    assert_eq!(detail.lots[0].qty_sold, 45);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    // The draft gives the lines back for a new try.
    assert_eq!(undone.draft.carried.len(), 1);
    assert_eq!(undone.draft.carried[0].lot_id, lot_id);
    assert_eq!(undone.draft.carried[0].sales.len(), 1);
    assert!(undone.draft.new_draft.is_some());
    // The same screen can be saved again after the undo (the number 2 is not used again).
    let again = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap();
    assert_eq!(again.new_bikri_number, Some(3));
}

#[test]
fn the_cash_question_is_asked_once_over_all_entries() {
    // Cash 2,000 after E2E-16. The carried line is paid 3,008 on the spot (cash in) and the new lot
    // gives its seller 20,000 نقد while شاہد pays 12,800 (net -7,200). Together: 2,000 + 3,008
    // - 7,200 = -2,192, the one figure the question shows (the new part alone would say -5,200).
    let w = support::world();
    let lot_id = e2e16(&w);
    // The opening is 7,000, so cash in hand is 7,000 - 5,000 (نقد of E2E-16) = 2,000.
    support::set_cash(&w, 7_000);
    let mut paid = carried(&w, lot_id);
    paid.sales[0].paid = Some("3,008".into());
    let mut lot = new_lot(&w);
    lot.cash_paid = Some("20,000".into());
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![paid],
        new_lots: vec![lot],
    };
    let p = bikri::mixed_preview(&w.env.ctx, &input).unwrap();
    assert!(p.preview.errors.is_empty(), "{:?}", p.preview.errors);
    assert_eq!(p.preview.cash.cash_after_paisa, rupees(-2_192));
    assert!(p.preview.cash.needs_confirm);
    // Without the yes nothing is written; the error carries the combined figure.
    let refused = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&input)).unwrap_err();
    let (code, value) = common::code_and_value(&refused, "cash_after_paisa");
    assert_eq!(code, ErrorCode::ErrCashBelowZeroNotConfirmed);
    assert_eq!(value, Some(-2_192 * 100));
    assert_eq!(cash_now(&w), 2_000 * 100);
    // With the yes it saves, and the undo asks the same question over its own change.
    let mut yes = save_input(&input);
    yes.confirm_cash_below_zero = true;
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &yes).unwrap();
    assert_eq!(cash_now(&w), -2_192 * 100);
    let undo = MixedUndoInput {
        later_entry_ids: vec![saved.carried[0].entry_id],
        new_bikri_id: saved.new_bikri_id,
        confirm_cash_below_zero: false,
    };
    // Undo gives back 20,000 less 12,800 and 3,008: cash goes UP, so no question.
    bikri::mixed_undo(&w.env.ctx, &w.owner, &undo).unwrap();
    assert_eq!(cash_now(&w), 2_000 * 100);
}

/// The later posting a mixed save writes on an OLD بکری counts under the day of that بکری, exactly
/// as a later sale did before (plan 72: a heading is the sum of its rows), and the find follows the
/// same rule. The new بکری of the same save is on the day of the save.
#[test]
fn the_list_and_the_find_show_both_kinds_the_way_a_later_sale_always_did() {
    let w = support::world();
    let lot_id = e2e16(&w);
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&mixed(&w, lot_id))).unwrap();
    assert_eq!(saved.new_bikri_number, Some(2));
    let window = BikriListInput {
        from: Some("10-10-2026".into()),
        to: Some("11-10-2026".into()),
        group_by_day: true,
        ..BikriListInput::default()
    };
    let list = bikri::list(&w.env.ctx, &window).unwrap();
    let row = |number: i64| list.rows.iter().find(|r| r.number == number).unwrap();
    // بکری 1 is dated 10-10 and now holds the carried sale too: 29,400 sold gross (E2E-16b).
    assert_eq!(
        (row(1).entry_date.as_str(), row(1).qty_sold, row(1).gross),
        ("2026-10-10", 50, rupees(29_300))
    );
    // بکری 2 is the new one, dated the day of the save: کریم's 10 سیب at 1,200.
    assert_eq!(
        (row(2).entry_date.as_str(), row(2).qty_sold, row(2).gross),
        ("2026-10-11", 10, rupees(12_000))
    );
    // Each day heading is the sum of its rows.
    for day in &list.days {
        let sum = list
            .rows
            .iter()
            .filter(|r| r.entry_date == day.date)
            .try_fold(mandi_services::core::money::Paisa::ZERO, |a, r| {
                a.checked_add(r.gross)
            })
            .unwrap();
        assert_eq!(day.gross_paisa, sum, "{}", day.date);
    }
    // The find: by day it finds the بکری dated that day (the new one); by number and by a buyer of
    // the carried sale it finds the old one.
    let find = |input: BikriFindInput| -> Vec<i64> {
        bikri::find(&w.env.ctx, &input)
            .unwrap()
            .iter()
            .map(|r| r.number)
            .collect()
    };
    assert_eq!(
        find(BikriFindInput {
            date: Some("11-10-2026".into()),
            ..BikriFindInput::default()
        }),
        vec![2]
    );
    assert_eq!(
        find(BikriFindInput {
            number: Some(1),
            ..BikriFindInput::default()
        }),
        vec![1]
    );
    assert_eq!(
        find(BikriFindInput {
            buyer_account_id: Some(w.arshad),
            ..BikriFindInput::default()
        }),
        vec![1]
    );
    // The undo takes both back: بکری 2 is cancelled and بکری 1 has its 5 نگ again.
    bikri::mixed_undo(
        &w.env.ctx,
        &w.owner,
        &MixedUndoInput {
            later_entry_ids: vec![saved.carried[0].entry_id],
            new_bikri_id: saved.new_bikri_id,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let after = bikri::list(&w.env.ctx, &window).unwrap();
    assert_eq!(
        after.rows.iter().map(|r| r.number).collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(after.rows[0].qty_sold, 45);
}

/// Reviewer finding C1 (plan 76 point 3, FR-12.48): after a mixed save the sale print and the buyer
/// parchi cover everything that save sold: the new بکری and the old بکری of each carried lot. State
/// E2E-16b: ارشد's last 5 کیلا (بکری 1, later posting of 11-10-26, 2,800) and شاہد's 10 سیب (the new
/// بکری 2, 12,000). The new بکری alone has only شاہد; with the old one too, ارشد (who bought only
/// carried goods) gets his parchi as well; the whole day has both.
#[test]
fn the_sale_print_of_a_mixed_save_covers_the_new_bikri_and_the_old_one_of_each_carried_lot() {
    use mandi_reports::reports::{
        DaySlipsInput, PrintCell, ReportPrintData, SlipSections, day_slips,
    };
    use mandi_services::core::auth::Language;
    let w = support::world();
    let lot_id = e2e16(&w);
    let saved = bikri::mixed_save(&w.env.ctx, &w.owner, &save_input(&mixed(&w, lot_id))).unwrap();
    let new_id = saved.new_bikri_id.expect("a new بکری");
    let old_id = saved.carried[0].old_bikri_id;
    // The save itself says which بکری to print: the new one, then the old one of each carried lot.
    assert_eq!(saved.print_bikri_ids, vec![new_id, old_id]);
    let slips = |bikri_id: Option<i64>, bikri_ids: Option<Vec<i64>>| {
        day_slips(
            w.env.ctx.db(),
            &DaySlipsInput {
                date: "11-10-2026".into(),
                bikri_id,
                bikri_ids,
                buyer_ids: None,
                sections: SlipSections::Both,
                language: Language::En,
                paper: None,
            },
            "print-sale-title",
        )
        .unwrap()
    };
    let text = |slip: &ReportPrintData, id: &str| -> String {
        match &slip
            .summary
            .iter()
            .find(|l| l.label_id == id)
            .unwrap_or_else(|| panic!("no line {id}"))
            .cell
        {
            PrintCell::Money { text } | PrintCell::Balance { text, .. } => text.clone(),
            other => panic!("{id}: {other:?}"),
        }
    };
    // The new بکری alone (the single id still works): only شاہد, one seller (کریم).
    let only_new = slips(Some(new_id), None);
    assert_eq!((only_new.buyers.len(), only_new.sellers.len()), (1, 1));
    assert_eq!(text(&only_new.buyers[0], "report-r35-goods"), "12,000");
    // The new and the old بکری of the carried lot: both buyers and both sellers (کریم, رحیم).
    let both = slips(None, Some(saved.print_bikri_ids.clone()));
    assert_eq!((both.buyers.len(), both.sellers.len()), (2, 2));
    // The whole day is the same two buyers, and each slip of the save equals the day's slip, since
    // nothing else was sold that day.
    let day = slips(None, None);
    assert_eq!((day.buyers.len(), day.sellers.len()), (2, 2));
    let goods: Vec<String> = both
        .buyers
        .iter()
        .map(|s| text(s, "report-r35-goods"))
        .collect();
    assert!(
        goods.contains(&"2,800".to_owned()) && goods.contains(&"12,000".to_owned()),
        "{goods:?}"
    );
    // ارشد bought only carried goods: 5 x 560 = 2,800, today's total 2,800 + 168 + 40 = 3,008, and
    // his balances are the day's.
    let arshad = both
        .buyers
        .iter()
        .find(|s| text(s, "report-r35-goods") == "2,800")
        .unwrap();
    assert_eq!(text(arshad, "report-r35-today-total"), "3,008");
    let of_day = day
        .buyers
        .iter()
        .find(|s| text(s, "report-r35-goods") == "2,800")
        .unwrap();
    assert_eq!(
        text(arshad, "report-r35-new-balance"),
        text(of_day, "report-r35-new-balance")
    );
    // The old بکری alone: ارشد and رحیم, not شاہد.
    let only_old = slips(None, Some(vec![old_id]));
    assert_eq!((only_old.buyers.len(), only_old.sellers.len()), (1, 1));
    assert_eq!(text(&only_old.buyers[0], "report-r35-goods"), "2,800");
}
