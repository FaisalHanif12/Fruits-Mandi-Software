//! Plan 80 Q-R2-1 (PRD FR-7.70, FR-12.19, FR-6.3, A74): a saved lot can be removed in an edit of the first
//! posting of its بکری, unless a later sale, a write-off or fruit taken rests on it. The lot stays as a
//! row (Code Standards D3), marked removed, and every number is exactly what the بکری would have been
//! without it.
//!
//! The proof: two worlds. World A saves Event 1 alone (PRD 4.4: رحیم's 50 کیلا, اکرم 20 at 600, ارشد 30
//! at 580; خام 29,400, صافی 19,316, shared income 3,948, split 1,580 / 1,184 / 1,184). World B saves
//! Event 1 together with a second lot (کریم's 10 سیب at 1,200 to the چکتا buyer شاہد, freight 1,000), and
//! then removes the second lot in an edit. Every account, every partner share, every analytics sum and
//! every stock sum must be the same in both worlds, and the trial balance must balance.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use common::{assert_chain_ok, raw, set_today};
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::reports::{ReportFilters, ReportId, ReportRunInput, run as run_report};
use mandi_services::core::bikri::{
    BikriDraft, BikriListInput, BikriStatus, ChuktaInput, LaterSaleSaveInput, LotInput,
    StockListInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::{ServiceError, bikri, ledger};
use support::World;

/// کریم's 10 سیب, freight 1,000, sold to the چکتا buyer شاہد at 1,200 (PRD 4.5).
fn karim_lot(w: &World) -> LotInput {
    let mut shahid = support::sale("n-1", w.chukta_khata, "10", "1200");
    shahid.chukta = Some(ChuktaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = support::lot(w, "lot-2", w.karim, w.apple, "10");
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    support::with_sales(lot, vec![shahid])
}

/// Event 1 and کریم's lot in one بکری.
fn both_lots(w: &World) -> BikriDraft {
    let mut draft = support::event1(w);
    draft.lots.push(karim_lot(w));
    draft
}

/// Every figure that must be the same: the net of each account code, each partner's share rows, the
/// quantity and the money of the analytics facts by item, and the stock moves by item. Sorted text.
fn books(w: &World) -> Vec<String> {
    let conn = raw(&w.dir);
    let mut out = Vec::new();
    let mut ask = |label: &str, sql: &str| {
        let mut statement = conn.prepare(sql).unwrap();
        let rows: Vec<String> = statement
            .query_map([], |r| {
                let mut cells = Vec::new();
                for i in 0..r.as_ref().column_count() {
                    let v: mandi_services::db::rusqlite::types::Value = r.get(i)?;
                    cells.push(format!("{v:?}"));
                }
                Ok(cells.join("|"))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for row in rows {
            out.push(format!("{label}: {row}"));
        }
    };
    ask(
        "account",
        "SELECT a.code, SUM(l.dr_paisa) - SUM(l.cr_paisa) FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id GROUP BY a.code
         HAVING SUM(l.dr_paisa) - SUM(l.cr_paisa) <> 0 ORDER BY a.code",
    );
    ask(
        "share",
        "SELECT partner_id, kind, SUM(amount_paisa) FROM partner_share_rows
         GROUP BY partner_id, kind ORDER BY partner_id, kind",
    );
    ask(
        "facts",
        "SELECT item_id, SUM(sign * qty), SUM(sign * seller_amount_paisa), SUM(sign * seller_comm_paisa),
                SUM(sign * buyer_comm_paisa), SUM(sign * laga_paisa)
         FROM analytics_facts GROUP BY item_id
         HAVING SUM(sign * qty) <> 0 ORDER BY item_id",
    );
    ask(
        "stock",
        "SELECT l.item_id, SUM(m.qty) FROM stock_moves m JOIN bikri_lots l ON l.id = m.lot_id
         GROUP BY l.item_id HAVING SUM(m.qty) <> 0 ORDER BY l.item_id",
    );
    out
}

fn removed_lots(w: &World) -> Vec<(i64, bool)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare("SELECT lot_no, removed_at IS NOT NULL FROM bikri_lots ORDER BY id")
        .unwrap();
    statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn refusals(error: &ServiceError) -> Vec<(ErrorCode, Option<String>, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .map(|r| {
                let rule = match r.values.get("rule") {
                    Some(ErrorValue::Text(t)) => Some(t.clone()),
                    _ => None,
                };
                (r.code, r.field.clone(), rule)
            })
            .collect(),
        other => panic!("not a refusal: {other:?}"),
    }
}

#[test]
fn removing_a_lot_in_an_edit_gives_the_books_the_bikri_would_have_had_without_it() {
    // World A: Event 1 alone.
    let a = support::world();
    support::save_old(&a, &support::event1(&a), false);
    // World B: Event 1 and کریم's lot, then the second lot is removed in an edit of the first posting.
    let b = support::world();
    let saved = support::save_old(&b, &both_lots(&b), false);
    assert_ne!(
        books(&a),
        books(&b),
        "the two lots are really in the books before the edit"
    );
    let opened = tools::open(&b, saved.bikri_id, saved.entry_id);
    assert_eq!(opened.draft.lots.len(), 2);
    let mut draft = opened.draft.clone();
    draft.lots.truncate(1);
    // The review of the edit (a rehearsal of the save, PRD FR-7.70) shows the removal: changes at the lot
    // that is no longer there, and cash in hand 57,800 -> 45,000 (شاہد's 12,800 paid on the spot goes
    // back out of the drawer), and no mistake.
    let review = mandi_services::edit::bikri_review(
        &b.env.ctx,
        &mandi_services::core::bikri::EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(review.errors.is_empty(), "{:?}", review.errors);
    assert!(
        review
            .review
            .changes
            .iter()
            .any(|c| c.place.lot_no == Some(2)
                && c.after == mandi_services::core::edit::ChangeValue::Empty),
        "{:?}",
        review.review.changes
    );
    let cash = review.review.effect.cash.as_ref().unwrap();
    assert_eq!(
        (cash.before_paisa.rupees(), cash.after_paisa.rupees()),
        (57_800, 45_000)
    );
    // The rehearsal wrote nothing: the lot is still in force.
    assert_eq!(removed_lots(&b), [(1, false), (2, false)]);
    // The preview before the save is the save (Code Standards B5); the second lot is not in it.
    let edited = tools::edit(&b, &opened, &draft);
    assert_eq!(edited.preview.lots.len(), 1);
    // Event 1 alone, to the rupee (PRD 4.4): خام 29,400, کل اخراجات 10,084, صافی 19,316, shared 3,948.
    let s = &edited.preview.summary;
    assert_eq!(
        (
            s.gross.rupees(),
            s.deductions.rupees(),
            s.net.rupees(),
            s.shared_income.rupees()
        ),
        (29_400, 10_084, 19_316, 3_948)
    );
    // Every account, share, fact and stock sum is the same as in world A.
    assert_eq!(books(&a), books(&b));
    // FR-6.1: every screen that adds up lots agrees with world A. R2 (the register), R6 (the lot
    // summary), R10 (the stock) and R27 (the bill series) have the same totals and rows, پچھلی بکری has
    // the same day, and the dashboard tiles are the same.
    let report = |w: &World, id: ReportId| {
        run_report(
            w.env.ctx.db(),
            &ReportRunInput {
                id,
                filters: ReportFilters::default(),
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap()
    };
    for id in [ReportId::R2, ReportId::R6, ReportId::R10, ReportId::R27] {
        let (page_a, page_b) = (report(&a, id), report(&b, id));
        assert_eq!(
            format!("{:?}", page_a.totals),
            format!("{:?}", page_b.totals),
            "{id:?}"
        );
        assert_eq!(page_a.total_count, page_b.total_count, "{id:?}");
    }
    let day = |w: &World| {
        let page = bikri::list(
            &w.env.ctx,
            &BikriListInput {
                date: Some("10-10-2026".into()),
                ..BikriListInput::default()
            },
        )
        .unwrap();
        (
            page.total_count,
            page.totals.gross,
            page.totals.net,
            page.totals.buyer_total,
            page.totals.qty_received,
            page.totals.qty_sold,
        )
    };
    assert_eq!(day(&a), day(&b));
    let tiles = |w: &World| {
        dashboard::get(
            w.env.ctx.db(),
            BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        )
        .unwrap()
        .tiles
    };
    assert_eq!(tiles(&a), tiles(&b));
    // The second lot is a row, marked removed, never deleted; the first lot is in force.
    assert_eq!(removed_lots(&b), [(1, false), (2, true)]);
    // The بکری shows one lot, its totals are Event 1's, and nothing waits in باقی مال.
    let detail = bikri::get(&b.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(detail.lots.len(), 1);
    assert_eq!(
        (
            detail.totals.gross.rupees(),
            detail.totals.deductions.rupees(),
            detail.totals.net.rupees()
        ),
        (29_400, 10_084, 19_316)
    );
    assert!(
        bikri::stock_list(&b.env.ctx, &StockListInput::default())
            .unwrap()
            .rows
            .is_empty()
    );
    // The books balance and both chains hold.
    let trial = ledger::trial_balance(&b.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&b.env.ctx).unwrap();
    assert_chain_ok(&b.dir);
    // Editing the بکری again works on the lot that is left, and a new lot gets the next number, not the
    // number of the removed one.
    let opened = tools::open(&b, saved.bikri_id, edited.entry_id);
    let mut draft = opened.draft.clone();
    assert_eq!(draft.lots.len(), 1);
    draft.lots.push(karim_lot(&b));
    tools::edit(&b, &opened, &draft);
    assert_eq!(removed_lots(&b), [(1, false), (2, true), (3, false)]);
}

#[test]
fn a_lot_with_a_later_sale_a_write_off_or_fruit_taken_cannot_be_removed() {
    // E2E-16: 45 of the 50 are sold, 5 are left. The بکری has a second lot, so one lot can be left out.
    let setup = || -> (World, BikriDraft, mandi_services::core::bikri::BikriSaved, i64) {
        let w = support::world();
        let mut draft = support::event1(&w);
        draft.lots[0].sales[1].qty = "25".into();
        draft.lots.push(karim_lot(&w));
        let saved = support::save_old(&w, &draft, false);
        let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
        (w, draft, saved, lot_id)
    };
    let try_remove_first_lot = |w: &World, saved: &mandi_services::core::bikri::BikriSaved| {
        let opened = tools::open(w, saved.bikri_id, saved.entry_id);
        let mut draft = opened.draft.clone();
        draft.lots.remove(0);
        tools::try_edit(w, &opened, &draft, true)
    };
    // (the rule the refusal names, the change that makes the lot rest on something)
    for rule in ["lot_has_later_sale", "lot_has_write_off", "lot_has_fruit"] {
        let (w, _draft, saved, lot_id) = setup();
        match rule {
            "lot_has_later_sale" => {
                set_today(&w.env, 2026, 10, 11);
                bikri::later_sale_save(
                    &w.env.ctx,
                    &w.owner,
                    &LaterSaleSaveInput {
                        lot_id,
                        date: "11-10-2026".into(),
                        sales: vec![support::sale("c-1", w.arshad, "5", "560")],
                        confirm_cash_below_zero: false,
                    },
                )
                .unwrap();
            }
            "lot_has_write_off" => {
                raw(&w.dir)
                    .execute(
                        "UPDATE bikri_lots SET qty_written_off = 1 WHERE id = ?1",
                        [lot_id],
                    )
                    .unwrap();
            }
            _ => {
                raw(&w.dir)
                    .execute(
                        "UPDATE bikri_lots SET qty_taken = 2 WHERE id = ?1",
                        [lot_id],
                    )
                    .unwrap();
            }
        }
        // The first posting is still the one to edit; a later sale moves the version, so read it again.
        let before = books(&w);
        let error = try_remove_first_lot(&w, &saved).unwrap_err();
        assert_eq!(
            refusals(&error),
            [(
                ErrorCode::ErrEditNotAllowed,
                Some("lots".to_owned()),
                Some(rule.to_owned())
            )],
            "{rule}"
        );
        // Nothing was written: the books and the lot rows are as they were.
        assert_eq!(books(&w), before, "{rule}");
        assert_eq!(removed_lots(&w), [(1, false), (2, false)], "{rule}");
        assert_eq!(
            bikri::get(&w.env.ctx, saved.bikri_id).unwrap().status,
            BikriStatus::Saved
        );
    }
}

#[test]
fn the_last_lot_cannot_be_removed() {
    let w = support::world();
    let saved = support::save_old(&w, &support::event1(&w), false);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut draft = opened.draft.clone();
    draft.lots.clear();
    // A بکری needs a lot and a line (PRD FR-12.8): the edit is refused and nothing is removed.
    assert!(tools::try_edit(&w, &opened, &draft, true).is_err());
    assert_eq!(removed_lots(&w), [(1, false)]);
}

/// A removed lot takes nothing more: the services do not find it (a later sale is refused), and the
/// database refuses a sale line on it, a change of its goods and a second removal time (the guards of
/// migration 27).
#[test]
fn a_removed_lot_cannot_be_sold_or_changed_again() {
    let w = support::world();
    let saved = support::save_old(&w, &both_lots(&w), false);
    let removed_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[1].id;
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut draft = opened.draft.clone();
    draft.lots.truncate(1);
    tools::edit(&w, &opened, &draft);
    // A later sale on the removed lot: not found.
    let error = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: removed_id,
            date: "10-10-2026".into(),
            sales: vec![support::sale("c-1", w.akram, "1", "100")],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert!(
        matches!(&error, ServiceError::Rejected(items) if items.iter().any(|r| r.code == ErrorCode::ErrLotNotFound)),
        "{error:?}"
    );
    // The guards of the database, with plain SQL.
    let conn = raw(&w.dir);
    let refused = |sql: &str| {
        let error = conn.execute(sql, []).unwrap_err().to_string();
        assert!(error.contains("ERR_LOT_REMOVED"), "{sql}: {error}");
    };
    refused(&format!(
        "UPDATE bikri_lots SET removed_at = NULL WHERE id = {removed_id}"
    ));
    refused(&format!(
        "UPDATE bikri_lots SET qty_taken = 1 WHERE id = {removed_id}"
    ));
    refused(&format!(
        "UPDATE bikri_lots SET qty_written_off = 1 WHERE id = {removed_id}"
    ));
    // A sale line on the removed lot, copied from a line of the lot that is in force.
    refused(&format!(
        "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date,
                buyer_account_id, qty, seller_rate_paisa, seller_amount_paisa, seller_comm_method,
                seller_comm_value_bps, seller_comm_paisa, buyer_rate_paisa, buyer_value_paisa,
                buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa, laga_method,
                laga_value_paisa, laga_paisa, buyer_total_paisa, rate_diff_paisa, payment_type,
                is_active, paid_paisa)
         SELECT bikri_id, {removed_id}, 9, journal_entry_id, entry_date, buyer_account_id, qty,
                seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps,
                seller_comm_paisa, buyer_rate_paisa, buyer_value_paisa, buyer_comm_method,
                buyer_comm_value_bps, buyer_comm_paisa, laga_method, laga_value_paisa, laga_paisa,
                buyer_total_paisa, rate_diff_paisa, payment_type, is_active, paid_paisa
         FROM bikri_sales WHERE is_active = 1 LIMIT 1"
    ));
}
