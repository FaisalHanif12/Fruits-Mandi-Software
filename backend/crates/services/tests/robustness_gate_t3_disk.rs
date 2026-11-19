//! Gate lane T3 (plan 80), part 3: the disk fills up in the middle of the BIG saves
//! (NFR-REL-10, NFR-REL-1, PRD FR-12.8, E20). Owner: `robustness`.
//!
//! `robustness_fault.rs` caps the number of pages SQLite may use (`PRAGMA max_page_count`) while a
//! voucher, a list of vouchers and an account are saved. This file does the same for the saves that
//! write the most: a 120-line بکری, its edit, its cancel, a later sale, a garden close and a month
//! close. For each:
//!
//! - the cap is set to the pages in use plus 0, 1, 2, 3 ... pages of room;
//! - a save that does not fit gives ERR_DISK_FULL (a clear code, not a crash and not a generic
//!   error) and leaves EVERY row of EVERY table as it was (a hash of all rows is compared);
//! - a save that fits is whole (the books balance, both chains verify);
//! - when the cap is lifted the same save works.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use common::*;
use mandi_services::core::bikri::{
    BikriSaveInput, CancelInput, EditOpenInput, EditSaveInput, LaterSaleSaveInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{
    GardenCloseInput, GardenEntryInput, GardenGetInput, GardenInput, GardenOtherSaleInput,
    GardenPaymentInput,
};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::db::rusqlite::{Connection, types::ValueRef};
use mandi_services::month::MonthInput;
use mandi_services::{ServiceError, bikri, day, gardens, ledger, month, vouchers};
use support::World;

fn tables(conn: &Connection) -> Vec<String> {
    let mut st = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap();
    st.query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn fingerprint(conn: &Connection) -> BTreeMap<String, (i64, u64)> {
    let mut out = BTreeMap::new();
    for table in tables(conn) {
        let mut st = conn.prepare(&format!("SELECT * FROM \"{table}\"")).unwrap();
        let columns = st.column_count();
        let mut rows = st.query([]).unwrap();
        let mut hasher = DefaultHasher::new();
        let mut n = 0_i64;
        while let Some(row) = rows.next().unwrap() {
            n += 1;
            for i in 0..columns {
                match row.get_ref(i).unwrap() {
                    ValueRef::Null => 0_u8.hash(&mut hasher),
                    ValueRef::Integer(v) => v.hash(&mut hasher),
                    ValueRef::Real(v) => v.to_bits().hash(&mut hasher),
                    ValueRef::Text(v) | ValueRef::Blob(v) => v.hash(&mut hasher),
                }
            }
        }
        out.insert(table, (n, hasher.finish()));
    }
    out
}

fn set_cap(w: &World, cap: i64) {
    w.env
        .ctx
        .db()
        .write(|tx| {
            tx.execute_batch(&format!("PRAGMA max_page_count = {cap}"))
                .map_err(ServiceError::from)
        })
        .unwrap();
}

fn pages(w: &World) -> i64 {
    w.env
        .ctx
        .db()
        .write(|tx| {
            tx.query_row("PRAGMA page_count", [], |r| r.get(0))
                .map_err(ServiceError::from)
        })
        .unwrap()
}

/// Runs `op` with room for 0, 1, 2 ... more pages. Returns (refused, headroom where it fitted).
fn with_little_room(
    w: &World,
    label: &str,
    mut op: impl FnMut() -> Result<(), ServiceError>,
) -> (usize, Option<i64>) {
    let conn = raw(&w.dir);
    let before = fingerprint(&conn);
    let base = pages(w);
    let mut refused = 0;
    let mut fitted = None;
    for headroom in [
        0_i64, 1, 2, 3, 4, 5, 7, 10, 14, 20, 30, 45, 70, 110, 170, 260, 400, 700,
    ] {
        set_cap(w, base + headroom);
        match op() {
            Err(error) => {
                assert!(
                    error.has_code(ErrorCode::ErrDiskFull),
                    "{label}: with {headroom} pages of room the save gave {error:?} instead of ERR_DISK_FULL"
                );
                let after = fingerprint(&conn);
                let diff: Vec<String> = after
                    .iter()
                    .filter(|(t, v)| before.get(*t) != Some(*v))
                    .map(|(t, v)| format!("{t}: {:?} -> {v:?}", before.get(t)))
                    .collect();
                assert!(
                    diff.is_empty(),
                    "{label}: a full disk ({headroom} pages of room) left rows behind: {diff:#?}"
                );
                refused += 1;
            }
            Ok(()) => {
                fitted = Some(headroom);
                break;
            }
        }
    }
    set_cap(w, 1_073_741_823);
    if fitted.is_none() {
        // The room is back: the same save works.
        op().unwrap_or_else(|e| panic!("{label}: the save does not work when room returns: {e:?}"));
    }
    // The books are whole either way.
    let imbalance: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0) - COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(imbalance, 0, "{label}: the books do not balance");
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok", "{label}");
    for (sql, what) in [
        (
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
            "a draft entry was left",
        ),
        (
            "SELECT COUNT(*) FROM edit_windows",
            "an edit window was left",
        ),
        (
            "SELECT COUNT(*) FROM pragma_foreign_key_check",
            "a foreign key is broken",
        ),
    ] {
        let n: i64 = conn.query_row(sql, [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0, "{label}: {what}");
    }
    ledger::verify_journal_chain(&w.env.ctx)
        .unwrap_or_else(|e| panic!("{label}: the journal chain is broken: {e:?}"));
    assert_chain_ok(&w.dir);
    println!(
        "{label}: {refused} tries refused for lack of room; fitted with {:?} pages of room",
        fitted
    );
    (refused, fitted)
}

fn big_draft(w: &World) -> mandi_services::core::bikri::BikriDraft {
    let sales = (0..120)
        .map(|i| {
            let buyer = if i % 2 == 0 { w.akram } else { w.arshad };
            support::sale(&format!("b-{i}"), buyer, "1", &(600 + (i % 5)).to_string())
        })
        .collect();
    support::draft(vec![support::with_sales(
        support::lot(w, "lot-big", w.rahim, w.banana, "130"),
        sales,
    )])
}

fn scalar(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn a_big_bikri_save_on_a_full_disk_gives_a_clear_error_and_leaves_nothing() {
    let w = support::world();
    let draft = big_draft(&w);
    let (refused, fitted) = with_little_room(&w, "120-line بکری save", || {
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft: draft.clone(),
                confirm_cash_below_zero: true,
            },
        )
        .map(|_| ())
    });
    assert!(
        refused >= 3,
        "the cap never bit, so the test shows nothing: {refused}"
    );
    assert!(fitted.is_some() || scalar(&w, "SELECT COUNT(*) FROM bikri_entries") == 1);
    // Exactly one بکری, with its 120 lines, however many tries were refused.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 1);
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_sales"), 120);
    assert_eq!(scalar(&w, "SELECT MAX(number) FROM bikri_entries"), 1);
}

#[test]
fn edit_later_sale_and_cancel_on_a_full_disk_give_a_clear_error_and_leave_nothing() {
    let w = support::world();
    let saved = bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::draft(vec![support::with_sales(
                support::lot(&w, "lot-1", w.rahim, w.banana, "50"),
                vec![
                    support::sale("s-1", w.akram, "20", "600"),
                    support::sale("s-2", w.arshad, "15", "580"),
                ],
            )]),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // Later sale of the 15 left.
    let lot_id = scalar(&w, "SELECT id FROM bikri_lots");
    let later = LaterSaleSaveInput {
        lot_id,
        date: "10-10-2026".into(),
        sales: vec![support::sale("s-8", w.arshad, "10", "560")],
        confirm_cash_below_zero: true,
    };
    let (r1, _) = with_little_room(&w, "later sale", || {
        bikri::later_sale_save(&w.env.ctx, &w.owner, &later).map(|_| ())
    });
    // Edit of the first posting.
    let entry = scalar(
        &w,
        "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first'",
    );
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: entry,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    draft.lots[0].sales[0].buyer_rate = Some("610".into());
    let edit = EditSaveInput {
        entry_id: opened.entry_id,
        version: opened.version,
        draft,
        reason: Some("rate".into()),
        confirm_cash_below_zero: true,
    };
    let (r2, _) = with_little_room(&w, "edit (reverse and post again)", || {
        bikri::edit_save(&w.env.ctx, &w.owner, &edit).map(|_| ())
    });
    // Cancel of the whole بکری.
    let version = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().version;
    let cancel = CancelInput {
        id: saved.bikri_id,
        version,
        reason: "gate".into(),
        confirm_cash_below_zero: true,
    };
    let (r3, _) = with_little_room(&w, "cancel", || {
        bikri::cancel(&w.env.ctx, &w.owner, &cancel).map(|_| ())
    });
    // A small save can reuse free pages and fit with no room; the cap must bite somewhere.
    assert!(r1 + r2 + r3 >= 1, "the cap never bit: {r1} {r2} {r3}");
}

#[test]
fn garden_close_month_close_and_day_close_on_a_full_disk_give_a_clear_error_and_leave_nothing() {
    let w = support::world();
    // A closed-able garden: first installment, sale in another shop, then the close.
    let [a, b, _] = w.partners;
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
            item_ids: Vec::new(),
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: vec![
                VenturePartnerInput {
                    partner_id: a,
                    share: "60".into(),
                    source: MoneySource::Shop,
                },
                VenturePartnerInput {
                    partner_id: b,
                    share: "40".into(),
                    source: MoneySource::Pocket,
                },
            ],
            paid_now: Some("30,000".into()),
            paid_now_reason: Some("first".into()),
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
    .row;
    gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::OtherSale(GardenOtherSaleInput {
            arrival_id: None,
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            shop: Some("Haji".into()),
            qty: "20".into(),
            amount: "10,000".into(),
            note: None,
        }),
    )
    .unwrap();
    let _ = GardenPaymentInput {
        garden_id: None,
        date: String::new(),
        amount: String::new(),
        reason: String::new(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    };
    let version = gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: garden.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
    .row
    .version;
    let close = GardenCloseInput {
        id: garden.id,
        version,
        date: "10-10-2026".into(),
    };
    let (r1, _) = with_little_room(&w, "garden close", || {
        gardens::close(&w.env.ctx, &w.owner, &close).map(|_| ())
    });
    // Income and an expense for the month close.
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::ReceiptInput {
            date: "10-10-2026".into(),
            account_id: w.akram,
            amount: "1,000".into(),
            removed: Default::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
    common::set_today(&w.env, 2026, 11, 2);
    let day_close = DayCloseInput {
        up_to: "31-10-2026".into(),
    };
    let (r2, _) = with_little_room(&w, "day close", || {
        day::close(&w.env.ctx, &w.owner, &day_close).map(|_| ())
    });
    let input = MonthInput {
        month: "2026-10".into(),
    };
    let (r3, _) = with_little_room(&w, "month close", || {
        month::close(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
    assert!(r1 + r2 + r3 >= 1, "the cap never bit: {r1} {r2} {r3}");
}
