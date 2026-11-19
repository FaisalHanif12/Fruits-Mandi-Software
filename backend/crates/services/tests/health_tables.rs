//! The daily books check compares the money tables with the posted journal (NFR-SEC-6 v1.21,
//! NFR-OBS-5, plan 80 Q-T3-3). Expected numbers are the PRD's (Section 4.4 and 4.5, E2E-1 to
//! E2E-5, E2E-13, E2E-16).
//!
//! Two kinds of test: a healthy book is never flagged (after each event, an edit, a cancel, a
//! later sale and a share change on the day of a posting), and every kind of change made outside
//! the app, with the triggers dropped and the journal left alone, is found and named.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, CancelInput, ChuktaInput, EditOpenInput, EditPreviewInput,
    EditSaveInput, HeaderInput, LaterSaleSaveInput,
};
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::db::rusqlite::Connection;
use mandi_services::health::{self, TablesState};
use mandi_services::{bikri, partners};
use support::World;

/// The comparisons of the tables, on a raw connection, as the daily check runs them.
fn found(w: &World) -> health::Found {
    health::compare_connection(&raw(&w.dir)).unwrap()
}

/// The whole books check: balance, both chains and the tables.
fn books(w: &World) -> health::BooksCheck {
    health::check_books(&w.env.ctx).unwrap()
}

#[track_caller]
fn assert_clean(w: &World, at: &str) {
    let f = found(w);
    assert_eq!(
        f.count,
        0,
        "{at}: a healthy book was flagged: {:?}",
        f.reasons
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let b = books(w);
    assert!(b.is_ok(), "{at}: {b:?}");
    assert_eq!(b.tables, TablesState::Ok);
}

fn save(w: &World, draft: &BikriDraft) -> mandi_services::core::bikri::BikriSaved {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
}

/// Event 2 of PRD 4.5: کریم's 10 سیب to the چکتا buyer شاہد, paid in full on the spot.
fn event2(w: &World) -> BikriDraft {
    let mut line = support::sale("s-1", w.chukta_khata, "10", "1200");
    line.chukta = Some(ChuktaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = support::lot(w, "lot-2", w.karim, w.apple, "10");
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    lot.sales = vec![line];
    BikriDraft {
        header: HeaderInput {
            truck_no: Some("LHR-0077".into()),
            ..support::header()
        },
        lots: vec![lot],
    }
}

/// A connection with every trigger dropped and foreign keys off: what someone with the data file
/// and no key can do.
fn attacker(w: &World) -> Connection {
    let conn = raw(&w.dir);
    let names: Vec<String> = {
        let mut st = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
            .unwrap();
        st.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    for name in names {
        conn.execute_batch(&format!("DROP TRIGGER IF EXISTS \"{name}\""))
            .unwrap();
    }
    conn.pragma_update(None, "foreign_keys", "OFF").unwrap();
    conn
}

#[test]
fn a_healthy_book_is_not_flagged_after_each_event_of_the_worked_day() {
    let w = support::world();
    assert_clean(&w, "the world before any posting");
    save(&w, &support::event1(&w));
    assert_clean(&w, "after Event 1");
    save(&w, &event2(&w));
    assert_clean(&w, "after Event 2 (a چکتا buyer paid in full)");
}

#[test]
fn a_healthy_book_is_not_flagged_after_an_edit_and_after_it_is_edited_back() {
    let w = support::world();
    let saved = save(&w, &support::event1(&w));
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let mut entry_id = saved.entry_id;
    let mut version = opened.version;
    let mut draft = opened.draft.clone();
    for (rate, at) in [
        ("610", "E2E-13: 600 to 610"),
        ("600", "E2E-14: back to 600"),
    ] {
        draft.lots[0].sales[0].seller_rate = rate.into();
        bikri::edit_preview(
            &w.env.ctx,
            &EditPreviewInput {
                entry_id,
                draft: draft.clone(),
            },
        )
        .unwrap();
        let done = bikri::edit_save(
            &w.env.ctx,
            &w.owner,
            &EditSaveInput {
                entry_id,
                version,
                draft: draft.clone(),
                reason: Some("ریٹ کی درستی".into()),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        entry_id = done.entry_id;
        version = done.version;
        assert_clean(&w, at);
        // The edited posting is reopened with the ids of its new lines.
        draft = bikri::open_for_edit(
            &w.env.ctx,
            &EditOpenInput {
                bikri_id: saved.bikri_id,
                entry_id,
            },
        )
        .unwrap()
        .draft;
    }
}

#[test]
fn a_share_change_on_the_day_of_a_posting_does_not_flag_the_earlier_posting() {
    // PRD FR-17.2: a change may take effect today. Event 1 of 10-10-26 was split 40 / 30 / 30; then
    // the shares become 50 / 25 / 25 from the same day. Postings saved before keep their split.
    let w = support::world();
    save(&w, &support::event1(&w));
    assert_clean(&w, "after Event 1");
    let [a, b, c] = w.partners;
    partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "10-10-2026".into(),
            shares: [(a, "50"), (b, "25"), (c, "25")]
                .iter()
                .map(|&(partner_id, share)| ShareInput {
                    partner_id,
                    share: share.to_owned(),
                })
                .collect(),
        },
    )
    .unwrap();
    assert_clean(&w, "after the shares changed on the same day");
    // A posting saved after the change is split with the new shares and is clean too.
    save(&w, &event2(&w));
    assert_clean(&w, "after Event 2 with the new shares");
}

/// Every kind of change outside the app that the books check must find, with the table it names.
/// Each is made on the worked day after Event 1 and Event 2, with the journal left as it is.
const TAMPERING: &[(&str, &str, &str)] = &[
    (
        "a sale line: the buyer rate raised by one rupee, with its value and total",
        "UPDATE bikri_sales SET buyer_rate_paisa = buyer_rate_paisa + 100, buyer_value_paisa = buyer_value_paisa + qty * 100, buyer_total_paisa = buyer_total_paisa + qty * 100, rate_diff_paisa = rate_diff_paisa + qty * 100 WHERE id = (SELECT MIN(id) FROM bikri_sales)",
        "bikri_sales",
    ),
    (
        "a sale line: the seller rate raised by one rupee, with its amount",
        "UPDATE bikri_sales SET seller_rate_paisa = seller_rate_paisa + 100, seller_amount_paisa = seller_amount_paisa + qty * 100, rate_diff_paisa = rate_diff_paisa - qty * 100 WHERE id = (SELECT MIN(id) FROM bikri_sales)",
        "bikri_sales",
    ),
    (
        "a sale line: the money paid on the spot raised",
        "UPDATE bikri_sales SET paid_paisa = paid_paisa + 100 WHERE id = (SELECT MIN(id) FROM bikri_sales)",
        "bikri_sales",
    ),
    (
        "a sale line: the buyer commission and total lowered by one rupee",
        "UPDATE bikri_sales SET buyer_comm_paisa = buyer_comm_paisa - 100, buyer_total_paisa = buyer_total_paisa - 100, buyer_comm_method = 'manual', buyer_comm_value_bps = NULL, buyer_comm_value_paisa = NULL WHERE id = (SELECT MIN(id) FROM bikri_sales)",
        "bikri_sales",
    ),
    (
        "a sale line deleted",
        "DELETE FROM bikri_sales WHERE id = (SELECT MAX(id) FROM bikri_sales)",
        "bikri_sales",
    ),
    (
        "a lot: the kept gross and net raised by one rupee",
        "UPDATE bikri_lots SET gross_paisa = gross_paisa + 100, net_paisa = net_paisa + 100 WHERE id = (SELECT MIN(id) FROM bikri_lots)",
        "bikri_lots",
    ),
    (
        "a lot: the cash given to the seller lowered by one rupee, with its deductions and net",
        "UPDATE bikri_lots SET cash_paid_paisa = cash_paid_paisa - 100, deductions_paisa = deductions_paisa - 100, net_paisa = net_paisa + 100 WHERE id = (SELECT MIN(id) FROM bikri_lots) AND cash_paid_paisa >= 100",
        "bikri_sales",
    ),
    (
        "a lot: the quantity sold raised",
        "UPDATE bikri_lots SET qty_sold = qty_sold - 1 WHERE id = (SELECT MIN(id) FROM bikri_lots)",
        "bikri_lots",
    ),
    (
        "a partner share row raised by one rupee",
        "UPDATE partner_share_rows SET amount_paisa = amount_paisa + 100 WHERE id = (SELECT MIN(id) FROM partner_share_rows)",
        "partner_share_rows",
    ),
    (
        "a partner share row moved to another partner",
        "UPDATE partner_share_rows SET partner_id = (SELECT MAX(id) FROM partners) WHERE id = (SELECT MIN(id) FROM partner_share_rows)",
        "partner_share_rows",
    ),
    (
        "a partner share row deleted",
        "DELETE FROM partner_share_rows WHERE id = (SELECT MIN(id) FROM partner_share_rows)",
        "partner_share_rows",
    ),
    (
        "an analytics fact raised by one rupee",
        "UPDATE analytics_facts SET seller_amount_paisa = seller_amount_paisa + 100 WHERE id = (SELECT MIN(id) FROM analytics_facts)",
        "analytics_facts",
    ),
    (
        "an analytics fact deleted",
        "DELETE FROM analytics_facts WHERE id = (SELECT MIN(id) FROM analytics_facts)",
        "analytics_facts",
    ),
    (
        "an analytics fact moved to another buyer",
        "UPDATE analytics_facts SET buyer_account_id = (SELECT MAX(id) FROM accounts) WHERE id = (SELECT MIN(id) FROM analytics_facts)",
        "analytics_facts",
    ),
    (
        "an analytics fact moved to another item",
        "UPDATE analytics_facts SET item_id = (SELECT MAX(id) FROM items) WHERE id = (SELECT MIN(id) FROM analytics_facts)",
        "analytics_facts",
    ),
    (
        "an analytics fact moved to another seller",
        "UPDATE analytics_facts SET seller_account_id = (SELECT MAX(id) FROM accounts) WHERE id = (SELECT MIN(id) FROM analytics_facts)",
        "analytics_facts",
    ),
    (
        "an analytics fact put on a trip or garden",
        "UPDATE analytics_facts SET venture_id = 1 WHERE id = (SELECT MIN(id) FROM analytics_facts)",
        "analytics_facts",
    ),
    (
        "a چکتا buyer's line moved to another person, with its fact",
        "UPDATE bikri_sales SET chukta_person_key = 'moved' WHERE id = (SELECT MAX(id) FROM bikri_sales); UPDATE analytics_facts SET chukta_person_key = 'moved' WHERE sale_id = (SELECT MAX(id) FROM bikri_sales)",
        "bikri_sales",
    ),
    (
        "a stock move changed by one",
        "UPDATE stock_moves SET qty = qty + 1 WHERE id = (SELECT MIN(id) FROM stock_moves)",
        "stock_moves",
    ),
    (
        "a stock move deleted",
        "DELETE FROM stock_moves WHERE id = (SELECT MAX(id) FROM stock_moves)",
        "stock_moves",
    ),
];

#[test]
fn every_kind_of_change_outside_the_app_is_found_and_names_its_table() {
    for (label, sql, table) in TAMPERING {
        let w = support::world();
        save(&w, &support::event1(&w));
        save(&w, &event2(&w));
        assert_clean(&w, "before the change");
        let conn = attacker(&w);
        conn.execute_batch(sql).unwrap();
        if conn.changes() == 0 {
            // A change that does not apply to this book (a condition in its WHERE) is skipped.
            continue;
        }
        let f = found(&w);
        assert!(f.count > 0, "not found: {label}");
        assert!(
            f.reasons.iter().any(|d| d.table == *table),
            "{label}: no reason names {table}: {:?}",
            f.reasons
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        // The full check fails too, with the first reason, and no amount in it.
        let b = books(&w);
        assert!(!b.is_ok(), "{label}");
        match &b.tables {
            TablesState::Differs { count, first } => {
                assert_eq!(*count, f.count);
                // A table and a record, no amount (NFR-OBS-2).
                assert!(first.contains(':') && first.len() < 200, "{label}: {first}");
            }
            TablesState::Ok => panic!("{label}: the tables were not flagged"),
        }
        // The summary kept as the latest result tells the same.
        let json = b.summary_json();
        assert_eq!(json["tables"]["ok"], false, "{label}");
    }
}

#[test]
fn a_retired_sale_line_of_an_edited_posting_is_still_compared() {
    let w = support::world();
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
    draft.lots[0].sales[0].seller_rate = "610".into();
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: saved.entry_id,
            version: opened.version,
            draft,
            reason: Some("ریٹ".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_clean(&w, "after the edit");
    // The old line (replaced) is changed with its value and total: the old posting's journal
    // lines no longer match it.
    let conn = attacker(&w);
    conn.execute_batch(
        "UPDATE bikri_sales SET buyer_value_paisa = buyer_value_paisa + 100 * qty,
            buyer_total_paisa = buyer_total_paisa + 100 * qty, buyer_rate_paisa = buyer_rate_paisa + 100,
            rate_diff_paisa = rate_diff_paisa + 100 * qty
         WHERE is_active = 0 AND id = (SELECT MIN(id) FROM bikri_sales)",
    )
    .unwrap();
    assert_eq!(conn.changes(), 1);
    let f = found(&w);
    assert!(f.count > 0, "an edited retired line was not compared");
}

#[test]
fn an_edit_to_the_journal_is_still_found_by_the_chain_and_the_trial_balance() {
    // The control: the tables check does not replace the chain.
    let w = support::world();
    save(&w, &support::event1(&w));
    let conn = attacker(&w);
    conn.execute_batch(
        "UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE id = (SELECT MIN(id) FROM journal_lines WHERE dr_paisa > 0)",
    )
    .unwrap();
    assert!(!books(&w).is_ok());
}

#[test]
fn the_check_is_read_only_and_writes_nothing_by_itself() {
    let w = support::world();
    save(&w, &support::event1(&w));
    let before = audit_total(&w.dir);
    let _ = books(&w);
    let _ = books(&w);
    assert_eq!(
        audit_total(&w.dir),
        before,
        "check_books wrote an audit row"
    );
}

#[test]
fn a_healthy_book_is_not_flagged_after_a_later_sale_and_after_a_cancel() {
    // E2E-16 and E2E-16b: 45 of 50 sold, the last 5 sold on 11-10-26; E2E-15: بکری 2 cancelled.
    let w = support::world();
    let mut part = support::event1(&w);
    part.lots[0].sales[1].qty = "25".into();
    save(&w, &part);
    assert_clean(&w, "after the part-sold بکری");
    set_today(&w.env, 2026, 10, 11);
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: 1,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_clean(&w, "after the later sale");
    let two = save(&w, &event2(&w));
    assert_clean(&w, "after Event 2");
    let detail = bikri::get(&w.env.ctx, two.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: two.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_clean(&w, "after the cancel of بکری 2");
}

/// 2 نگ of grapes, 12.345 kg at 100 a kilogram, sold to اکرم (PRD C1: the line is worth 1,235).
fn weighed(w: &World) -> BikriDraft {
    let mut lot = support::bare_lot(w, "lot-g", w.rahim, w.weight_fruit, "2");
    lot.weight_received = Some("12.345".into());
    let mut sale = support::sale("s-1", w.akram, "2", "100");
    sale.weight = Some("12.345".into());
    lot.sales = vec![sale];
    support::draft(vec![lot])
}

#[test]
fn a_weighed_line_is_the_weight_times_the_rate_and_a_changed_rate_is_found() {
    let w = support::world();
    save(&w, &weighed(&w));
    assert_clean(&w, "after a weighed line");
    // The rate is raised and nothing else: the table cannot check weight times rate, the journal
    // is untouched, and only the line's own identity (PRD C1) shows it.
    let conn = attacker(&w);
    conn.execute_batch("UPDATE bikri_sales SET seller_rate_paisa = seller_rate_paisa + 100")
        .unwrap();
    assert_eq!(conn.changes(), 1);
    let f = found(&w);
    assert!(
        f.reasons
            .iter()
            .any(|d| d.what.contains("weight times its rate")),
        "{:?}",
        f.reasons
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_retired_line_with_a_changed_seller_amount_is_found_by_its_first_fact() {
    let w = support::world();
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
    draft.lots[0].sales[0].seller_rate = "610".into();
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: saved.entry_id,
            version: opened.version,
            draft,
            reason: Some("ریٹ".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_clean(&w, "after the edit");
    // The retired line's seller side (amount, rate and the rate difference) changed together.
    let conn = attacker(&w);
    conn.execute_batch(
        "UPDATE bikri_sales SET seller_amount_paisa = seller_amount_paisa + 100 * qty,
            seller_rate_paisa = seller_rate_paisa + 100, rate_diff_paisa = rate_diff_paisa - 100 * qty
         WHERE is_active = 0 AND id = (SELECT MIN(id) FROM bikri_sales)",
    )
    .unwrap();
    assert_eq!(conn.changes(), 1);
    let f = found(&w);
    assert!(
        f.reasons.iter().any(|d| d.table == "analytics_facts"),
        "{:?}",
        f.reasons
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}
