//! Gate lane T3 (plan 80), part 4: backup and restore on a RICH book (E2E-18, NFR-BAK-4,
//! NFR-BAK-5, NFR-BAK-6, NFR-REL-3, PRD FR-20.8, FR-20.9, FR-20.10, NFR-SEC-6). Owner: `robustness`.
//!
//! `round_trip.rs` proves that the balances, the journal and audit chains and two counts come back
//! after a restore on the PRD worked day. These tests go further:
//!
//! 1. A book with every kind of record (بکری, an edit, a cancel, a trip, a garden, capital, a
//!    separate khata, the dead amount, a closed day) is backed up to the local folder, the USB
//!    drive and the cloud, the book moves on, and a restore from each place gives back EVERY ROW OF
//!    EVERY TABLE exactly (the audit log keeps its old rows and adds the one row of the restore).
//! 2. A copy cut or changed at many places is refused by the signed-in restore, from each place,
//!    and the data file and the safety folder are untouched.
//! 3. The safety copy made before a restore is a whole, checkable copy of the data as it was, with
//!    both chains verifying, and a second restore of the same copy gives the same books.
//! 4. A file changed outside the app is found by the chains, and a restore of a copy made from it
//!    is refused (NFR-SEC-6).
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use harness::*;
use mandi_backup::api::{
    BackupListInput, BackupRef, RestoreInput, RestorePreviewInput, TestRestoreInput,
};
use mandi_backup::model::BackupKind;
use mandi_backup::run::Scope;
use mandi_services::core::bikri::{BikriSaveInput, CancelInput, EditOpenInput, EditSaveInput};
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::dead::{DeadMarkInput, DeadReason};
use mandi_services::core::gardens::{GardenEntryInput, GardenInput, GardenPaymentInput};
use mandi_services::core::khata::{KhataCreateInput, KhataOutInput};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::{TripCostInput, TripInput};
use mandi_services::core::ventures::{EntryPartInput, MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::db::audit::verify_audit_chain;
use mandi_services::db::rusqlite::{Connection, OpenFlags, types::ValueRef};
use mandi_services::{bikri, day, gardens, khata, purchases, trips, vouchers};

// ---------------------------------------------------------------------------------------------
// A book with every kind of record.
// ---------------------------------------------------------------------------------------------

fn shop(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn pocket(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    }
}

/// The worked day (Events 1 to 4) and then every other kind of record.
fn rich_rig() -> Rig {
    let rig = worked_day();
    let w = &rig.w;
    let ctx = &w.env.ctx;
    // An edit of بکری 1 and a cancel of بکری 2.
    let entry: i64 = raw(&rig)
        .query_row(
            "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first' ORDER BY id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let opened = bikri::open_for_edit(
        ctx,
        &EditOpenInput {
            bikri_id: 1,
            entry_id: entry,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    draft.lots[0].sales[0].buyer_rate = Some("610".into());
    bikri::edit_save(
        ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("rate".into()),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let version = bikri::get(ctx, 2).unwrap().version;
    bikri::cancel(
        ctx,
        &w.owner,
        &CancelInput {
            id: 2,
            version,
            reason: "gate".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // A new بکری after the cancel, part sold.
    bikri::save(
        ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::draft(vec![support::with_sales(
                support::lot(w, "lot-1", w.rahim, w.banana, "50"),
                vec![support::sale("s-1", w.akram, "20", "600")],
            )]),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // Capital.
    vouchers::save_capital_in(
        ctx,
        &w.owner,
        &CapitalInput {
            date: "10-10-2026".into(),
            partner_id: Some(w.partners[1]),
            amount: "50,000".into(),
            narration: None,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // A separate khata, paid out.
    let khata_row = khata::create(
        ctx,
        &w.owner,
        &KhataCreateInput {
            name_ur: "خاندانی کھاتہ".into(),
            name_en: "Family Khata".into(),
            note: None,
            confirmed_rules_words: vec![
                "خاندانی".into(),
                "کھاتہ".into(),
                "Family".into(),
                "Khata".into(),
            ],
        },
    )
    .unwrap();
    vouchers::save_khata_out(
        ctx,
        &w.owner,
        &KhataOutInput {
            date: "10-10-2026".into(),
            khata_account_id: Some(khata_row.account.id),
            amount: "10,000".into(),
            reason: "family".into(),
            narration: None,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // A trip with a purchase and a cost.
    let [a, b, c] = w.partners;
    let trip = trips::create(
        ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: vec![shop(a, "40"), shop(b, "30"), shop(c, "30")],
        },
    )
    .unwrap();
    let orange = support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit");
    purchases::save(
        ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            supplier: Some("Orange Traders".into()),
            item_id: Some(orange),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            rate: "500".into(),
            freight: Some("400".into()),
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::<EntryPartInput>::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    trips::cost_save(
        ctx,
        &w.owner,
        &TripCostInput {
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            amount: "4,000".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // A garden with an installment.
    let garden = gardens::create(
        ctx,
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
            partners: vec![shop(a, "60"), pocket(b, "40")],
            paid_now: Some("30,000".into()),
            paid_now_reason: Some("first".into()),
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
    .row;
    gardens::entry_save(
        ctx,
        &w.owner,
        &GardenEntryInput::Installment(GardenPaymentInput {
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            amount: "5,000".into(),
            reason: "installment".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        }),
    )
    .unwrap();
    // The dead amount.
    mandi_services::dead::mark(
        ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.arshad,
            entry_date: "10-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();
    // A day closed.
    common::set_today(&w.env, 2026, 10, 11);
    day::close(
        ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    rig
}

// ---------------------------------------------------------------------------------------------
// Every row of every table.
// ---------------------------------------------------------------------------------------------

type Rows = BTreeMap<String, Vec<String>>;

fn every_row(conn: &Connection) -> Rows {
    let names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut out = Rows::new();
    for table in names {
        let mut st = conn.prepare(&format!("SELECT * FROM \"{table}\"")).unwrap();
        let columns = st.column_count();
        let mut rows = st.query([]).unwrap();
        let mut list = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            let mut text = String::new();
            for i in 0..columns {
                match row.get_ref(i).unwrap() {
                    ValueRef::Null => text.push_str("NULL|"),
                    ValueRef::Integer(v) => text.push_str(&format!("{v}|")),
                    ValueRef::Real(v) => text.push_str(&format!("{v}|")),
                    ValueRef::Text(v) => text.push_str(&format!("{}|", String::from_utf8_lossy(v))),
                    ValueRef::Blob(v) => text.push_str(&format!("blob{}|", v.len())),
                }
            }
            list.push(text);
        }
        out.insert(table, list);
    }
    out
}

fn all_rows(rig: &Rig) -> Rows {
    every_row(&raw(rig))
}

/// The tables that may differ after a restore, and why.
const MAY_DIFFER: &[&str] = &[
    // The restore writes one audit row; the old rows must be the same.
    "audit_log",
    // The backup runs write their own log rows.
    "backup_log",
];

fn receipt_on(
    account: i64,
    amount: &str,
    date: &str,
) -> mandi_services::core::vouchers::ReceiptInput {
    mandi_services::core::vouchers::ReceiptInput {
        date: date.into(),
        ..rupees_input(account, amount)
    }
}

fn first_name(rig: &Rig, kind: BackupKind) -> String {
    rig.service
        .list(&BackupListInput {
            kind: Some(kind),
            offset: 0,
            limit: 10,
        })
        .unwrap()
        .rows
        .first()
        .unwrap_or_else(|| panic!("no {kind:?} copy"))
        .name
        .clone()
}

fn restore_input(kind: BackupKind, name: &str, key: Option<&str>) -> RestoreInput {
    RestoreInput {
        backup: BackupRef {
            kind,
            name: name.to_owned(),
        },
        key: key.map(str::to_owned),
        confirmed: true,
    }
}

#[test]
fn a_rich_book_comes_back_row_for_row_from_the_local_folder_the_usb_drive_and_the_cloud() {
    for restore_from in [BackupKind::Local, BackupKind::Usb, BackupKind::Cloud] {
        let rig = rich_rig();
        switch_everything_on(&rig);
        let report = rig.service.run(Scope::ALL).unwrap();
        for kind in [BackupKind::Local, BackupKind::Usb, BackupKind::Cloud] {
            assert!(first_ok(&report, kind), "{kind:?}: {report:?}");
        }
        let before = all_rows(&rig);
        let tables_with_rows = before.values().filter(|rows| !rows.is_empty()).count();
        let empty: Vec<&String> = before
            .iter()
            .filter(|(_, rows)| rows.is_empty())
            .map(|(t, _)| t)
            .collect();
        assert!(
            tables_with_rows >= 30,
            "the rich book should fill many tables: {tables_with_rows}; empty: {empty:?}"
        );
        let before_fp = fingerprint(&rig);

        // The books move on after the backup.
        rig.w.env.clock.advance_seconds(60);
        common::set_today(&rig.w.env, 2026, 10, 12);
        vouchers::save_receipt(
            &rig.w.env.ctx,
            &rig.w.owner,
            &receipt_on(rig.w.akram, "1,000", "12-10-2026"),
        )
        .unwrap();
        bikri::save(
            &rig.w.env.ctx,
            &rig.w.owner,
            &BikriSaveInput {
                draft: {
                    let mut d = support::draft(vec![support::with_sales(
                        support::lot(&rig.w, "lot-9", rig.w.karim, rig.w.apple, "10"),
                        vec![support::sale("s-9", rig.w.akram, "10", "900")],
                    )]);
                    d.header.entry_date = "12-10-2026".into();
                    d
                },
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
        assert_ne!(all_rows(&rig), before);

        let name = first_name(&rig, restore_from);
        let done = rig
            .service
            .restore(
                Some(rig.w.owner.user_id),
                &restore_input(restore_from, &name, None),
            )
            .unwrap();
        assert!(done.safety_copy.starts_with("mandi-local-"));
        rig.w.env.ctx.invalidate_caches();

        let after = all_rows(&rig);
        let mut differences = Vec::new();
        for (table, rows) in &before {
            if MAY_DIFFER.contains(&table.as_str()) {
                continue;
            }
            let now = after.get(table).map(Vec::as_slice).unwrap_or(&[]);
            if rows.as_slice() != now {
                differences.push(format!(
                    "{table}: {} rows before the restore, {} after",
                    rows.len(),
                    now.len()
                ));
            }
        }
        for table in after.keys() {
            assert!(
                before.contains_key(table),
                "{restore_from:?}: new table {table}"
            );
        }
        assert!(
            differences.is_empty(),
            "{restore_from:?}: tables that are not row for row the copy: {differences:#?}"
        );
        // The audit log: the old rows unchanged, then one row for the restore.
        let (old, new) = (&before["audit_log"], &after["audit_log"]);
        assert_eq!(
            new.get(..old.len()),
            Some(old.as_slice()),
            "{restore_from:?}"
        );
        assert_eq!(new.len(), old.len() + 1, "{restore_from:?}");
        let after_fp = fingerprint(&rig);
        assert_eq!(after_fp.balances, before_fp.balances);
        assert_eq!(after_fp.journal, before_fp.journal);
        verify_audit_chain(&raw(&rig), &common::audit_key()).unwrap();
        mandi_services::ledger::verify_journal_chain(&rig.w.env.ctx).unwrap();
        // The app works on the restored book.
        vouchers::save_receipt(
            &rig.w.env.ctx,
            &rig.w.owner,
            &receipt_on(rig.w.akram, "500", "12-10-2026"),
        )
        .unwrap();
        println!(
            "{restore_from:?}: {tables_with_rows} tables with rows, all identical after the restore"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Copies cut or changed at many places.
// ---------------------------------------------------------------------------------------------

#[test]
fn a_copy_cut_or_changed_at_many_places_is_refused_by_the_signed_in_restore_from_every_place() {
    let rig = worked_day();
    switch_everything_on(&rig);
    rig.service.run(Scope::ALL).unwrap();
    let before = all_rows(&rig);
    let local = first_name(&rig, BackupKind::Local);
    let usb = first_name(&rig, BackupKind::Usb);
    let cloud = first_name(&rig, BackupKind::Cloud);
    let places: Vec<(BackupKind, String, std::path::PathBuf)> = vec![
        (
            BackupKind::Local,
            local.clone(),
            rig.backups.join("local").join(&local),
        ),
        (BackupKind::Usb, usb.clone(), rig.usb.join(&usb)),
        (
            BackupKind::Cloud,
            cloud.clone(),
            rig.root.path().join("cloud").join("backups").join(&cloud),
        ),
    ];
    let mut refused = 0;
    let mut codes: BTreeMap<String, usize> = BTreeMap::new();
    for (kind, name, path) in &places {
        let original = fs::read(path).unwrap();
        let len = original.len();
        assert!(
            len > 1_000,
            "{kind:?}: the copy is too small to test ({len})"
        );
        let mut attacks: Vec<(String, Vec<u8>)> = Vec::new();
        for at in [
            0,
            1,
            7,
            15,
            31,
            100,
            len / 8,
            len / 4,
            len / 2,
            3 * len / 4,
            len - 40,
            len - 17,
            len - 2,
            len - 1,
        ] {
            let mut bytes = original.clone();
            bytes[at] ^= 0xFF;
            attacks.push((format!("byte {at} changed"), bytes));
        }
        for keep in [0, 1, 10, 16, 100, len / 2, len - 100, len - 1] {
            attacks.push((format!("cut to {keep} bytes"), original[..keep].to_vec()));
        }
        let mut extended = original.clone();
        extended.extend_from_slice(b"extra bytes after the end");
        attacks.push(("bytes added at the end".into(), extended));
        for (label, bytes) in attacks {
            fs::write(path, &bytes).unwrap();
            let preview = rig.service.restore_preview(&RestorePreviewInput {
                backup: BackupRef {
                    kind: *kind,
                    name: name.clone(),
                },
                key: None,
            });
            let restore = rig.service.restore(None, &restore_input(*kind, name, None));
            match (&preview, &restore) {
                (Err(p), Err(r)) => {
                    *codes.entry(format!("{kind:?}: {:?}", r.code)).or_default() += 1;
                    let _ = p;
                    refused += 1;
                }
                other => panic!("{kind:?}, {label}: a damaged copy was accepted: {other:?}"),
            }
            assert_eq!(
                all_rows(&rig),
                before,
                "{kind:?}, {label}: the data changed although the restore was refused"
            );
            assert!(
                !rig.service.safety_dir().exists(),
                "{kind:?}, {label}: a safety copy was made for a refused restore"
            );
        }
        fs::write(path, &original).unwrap();
    }
    println!("{refused} damaged copies refused; codes: {codes:?}");
    // With the files put back, the same restores work.
    for (kind, name, _) in &places {
        rig.service
            .restore(None, &restore_input(*kind, name, None))
            .unwrap_or_else(|e| panic!("{kind:?}: the good copy was refused: {e:?}"));
    }
}

// ---------------------------------------------------------------------------------------------
// The safety copy, and a second restore.
// ---------------------------------------------------------------------------------------------

#[test]
fn the_safety_copy_is_the_whole_data_before_the_restore_and_a_second_restore_gives_the_same_books()
{
    let rig = rich_rig();
    switch_everything_on(&rig);
    rig.service.run(Scope::ALL).unwrap();
    // The book moves on, so the safety copy differs from the copy that is restored.
    rig.w.env.clock.advance_seconds(60);
    vouchers::save_receipt(
        &rig.w.env.ctx,
        &rig.w.owner,
        &receipt_on(rig.w.akram, "1,000", "11-10-2026"),
    )
    .unwrap();
    let moved_on = all_rows(&rig);
    let name = first_name(&rig, BackupKind::Local);
    let done = rig
        .service
        .restore(
            Some(rig.w.owner.user_id),
            &restore_input(BackupKind::Local, &name, None),
        )
        .unwrap();
    rig.w.env.ctx.invalidate_caches();
    let safety = rig.service.safety_dir().join(&done.safety_copy);
    assert!(
        safety.exists(),
        "the safety copy is not in the safety folder"
    );
    // The safety copy opens, is whole and holds exactly the data as it was before the restore.
    let conn = Connection::open_with_flags(&safety, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let kept = every_row(&conn);
    for (table, rows) in &moved_on {
        assert_eq!(
            kept.get(table),
            Some(rows),
            "the safety copy differs from the data before the restore in {table}"
        );
    }
    verify_audit_chain(&conn, &common::audit_key()).unwrap();
    mandi_services::db::ledger::verify_journal_chain(&conn, &common::audit_key()).unwrap();
    // A second restore of the same copy gives the same books as the first (plus one audit row).
    let first = all_rows(&rig);
    // The clock is manual here; a safety copy is named by the moment it is made.
    rig.w.env.clock.advance_seconds(60);
    rig.service
        .restore(
            Some(rig.w.owner.user_id),
            &restore_input(BackupKind::Local, &name, None),
        )
        .unwrap();
    rig.w.env.ctx.invalidate_caches();
    let second = all_rows(&rig);
    for (table, rows) in &first {
        if MAY_DIFFER.contains(&table.as_str()) {
            continue;
        }
        assert_eq!(
            second.get(table),
            Some(rows),
            "{table} changed in the second restore"
        );
    }
    // The restore puts the copy back, so the audit row of the first restore is gone with the rest
    // of what came after the copy; the second restore writes its own. The count is the same.
    assert_eq!(second["audit_log"].len(), first["audit_log"].len());
    assert_eq!(
        second["audit_log"].get(..second["audit_log"].len() - 1),
        first["audit_log"].get(..first["audit_log"].len() - 1)
    );
    // The safety folder keeps both copies (nothing is overwritten).
    let kept_copies = fs::read_dir(rig.service.safety_dir())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with("mandi-local-"))
        .filter(|e| !e.file_name().to_string_lossy().ends_with(".json"))
        .count();
    assert!(kept_copies >= 2, "{kept_copies} safety copies");
}

// ---------------------------------------------------------------------------------------------
// A file changed outside the app, then backed up.
// ---------------------------------------------------------------------------------------------

fn tamper(path: &Path) {
    let conn = Connection::open(path).unwrap();
    let triggers: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for name in triggers {
        conn.execute_batch(&format!("DROP TRIGGER IF EXISTS \"{name}\""))
            .unwrap();
    }
    // Both sides of one entry raised by a rupee: the books still balance.
    conn.execute_batch(
        "UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE id = (SELECT MIN(id) FROM journal_lines WHERE dr_paisa > 0);
         UPDATE journal_lines SET cr_paisa = cr_paisa + 100 WHERE id = (SELECT MIN(id) FROM journal_lines WHERE cr_paisa > 0 AND entry_id = (SELECT entry_id FROM journal_lines WHERE dr_paisa > 0 ORDER BY id LIMIT 1));",
    )
    .unwrap();
}

#[test]
fn a_file_changed_outside_the_app_is_found_by_the_chains_and_a_copy_of_it_cannot_be_restored() {
    let rig = worked_day();
    switch_everything_on(&rig);
    // A good copy first.
    rig.service.run(Scope::ALL).unwrap();
    let good_local = first_name(&rig, BackupKind::Local);
    // Someone changes the live file with another tool.
    tamper(&rig.w.dir.path().join("data").join("mandi.db"));
    let found = mandi_services::health::check_books(&rig.w.env.ctx).unwrap();
    assert!(
        !found.journal_chain.is_ok(),
        "the chain still verifies after an edit outside the app"
    );
    assert!(!found.is_ok());
    // A new backup is taken of the changed file (the schedule does not know). Say what happens.
    rig.w.env.clock.advance_seconds(120);
    let report = rig.service.run(Scope::ALL);
    println!("backup of a file changed outside the app: {report:?}");
    // gate fix Q-T3-1 (NFR-BAK-5 v1.21): the check of a copy also verifies both hash chains, so a
    // snapshot of a file changed outside the app is marked bad, nothing is kept, and the alert shows.
    let report = report.unwrap();
    assert!(
        !report.results.is_empty() && report.results.iter().all(|r| !r.ok),
        "a copy of a file with a broken chain was marked good: {report:?}"
    );
    assert!(
        report
            .results
            .iter()
            .all(|r| r.code == Some(mandi_backup::error::BackupCode::ErrBackupCheckFailed)),
        "{report:?}"
    );
    let listed = rig
        .service
        .list(&BackupListInput {
            kind: Some(BackupKind::Local),
            offset: 0,
            limit: 10,
        })
        .unwrap();
    let newest = listed.rows.first().unwrap().name.clone();
    if newest != good_local {
        // A copy of the changed file exists: it must be refused, with the data left alone.
        let before = all_rows(&rig);
        let refused = rig
            .service
            .restore(None, &restore_input(BackupKind::Local, &newest, None));
        assert!(
            refused.is_err(),
            "a copy made from a file changed outside the app was restored"
        );
        assert_eq!(all_rows(&rig), before);
        // The test restore says why.
        let tested = rig
            .service
            .test_restore(&TestRestoreInput {
                backup: Some(BackupRef {
                    kind: BackupKind::Local,
                    name: newest,
                }),
                key: None,
            })
            .unwrap();
        // NFR-BAK-5 lists the data check, the record counts and the checksum, not the chains, so
        // the test restore may say all is well; the restore itself is what must refuse.
        println!(
            "the test restore of a copy of a changed file says ok = {}, checks {:?}",
            tested.ok, tested.checks
        );
    }
}
