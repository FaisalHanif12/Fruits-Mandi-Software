//! E2E-18 (PRD 27.2, Code Standards 9.2): a backup, then a restore, gives back every balance and
//! the journal and audit chains exactly (plan 18, D5-A11, D5-A12, D5-A13). The books are the PRD
//! worked day (Events 1 to 4).
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use std::time::{Duration, Instant};

use harness::*;
use mandi_backup::api::{BackupListInput, BackupRef, RestoreInput, RestorePreviewInput};
use mandi_backup::error::BackupCode;
use mandi_backup::model::BackupKind;
use mandi_backup::run::Scope;
use mandi_backup::secrets::{SecretName, SecretStore};
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_services::db::audit::verify_audit_chain;
use mandi_services::{auth, vouchers};
use std::sync::Arc;

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
fn e2e_18_backup_then_restore_gives_identical_balances_and_chains_from_every_place() {
    for restore_from in [BackupKind::Local, BackupKind::Usb, BackupKind::Cloud] {
        let rig = worked_day();
        switch_everything_on(&rig);
        let report = rig.service.run(Scope::ALL).unwrap();
        for kind in [BackupKind::Local, BackupKind::Usb, BackupKind::Cloud] {
            assert!(first_ok(&report, kind), "{kind:?}: {report:?}");
        }
        assert!(report.results.iter().all(|r| !r.queued));
        let before = fingerprint(&rig);
        assert!(before.lines > 0 && before.bikri == 2);

        // The books move on after the backup (a receipt of 1,000 from ارشد).
        rig.w.env.clock.advance_seconds(60);
        vouchers::save_receipt(
            &rig.w.env.ctx,
            &rig.w.owner,
            &rupees_input(rig.w.arshad, "1,000"),
        )
        .unwrap();
        assert_ne!(fingerprint(&rig), before);

        let name = first_name(&rig, restore_from);
        let preview = rig
            .service
            .restore_preview(&RestorePreviewInput {
                backup: BackupRef {
                    kind: restore_from,
                    name: name.clone(),
                },
                key: None,
            })
            .unwrap();
        assert_eq!(preview.lost.journal_entries, 1, "the receipt would be lost");
        assert_eq!(preview.totals.debit_paisa, preview.totals.credit_paisa);
        assert!(!preview.needs_upgrade);

        let done = rig
            .service
            .restore(
                Some(rig.w.owner.user_id),
                &restore_input(restore_from, &name, None),
            )
            .unwrap();
        assert!(done.safety_copy.starts_with("mandi-local-"));
        // The restore adds one audit row (its own); everything else is the copy.
        let mut shown = done.totals.clone();
        shown.audit_rows -= 1;
        assert_eq!(shown, preview.totals);

        let after = fingerprint(&rig);
        assert_eq!(after.balances, before.balances, "{restore_from:?}");
        assert_eq!(after.journal, before.journal);
        assert_eq!((after.lines, after.bikri), (before.lines, before.bikri));
        // The audit log has the old rows unchanged, then one row for the restore.
        assert_eq!(
            after.audit.get(..before.audit.len()),
            Some(before.audit.as_slice())
        );
        assert_eq!(after.audit.len(), before.audit.len() + 1);
        let conn = raw(&rig);
        verify_audit_chain(&conn, &common::audit_key()).unwrap();

        // The app works on the restored data: the cash is the PRD 4.9 cash before Event 5.
        let cash: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(l.dr_paisa),0) - COALESCE(SUM(l.cr_paisa),0) FROM journal_lines l JOIN accounts a ON a.id = l.account_id WHERE a.code = '01010001'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cash, (50_000 - 5_000 + 12_800 + 10_000 - 4_316) * 100);
        rig.w.env.ctx.invalidate_caches();
        vouchers::save_receipt(
            &rig.w.env.ctx,
            &rig.w.owner,
            &rupees_input(rig.w.arshad, "500"),
        )
        .unwrap();
    }
}

#[test]
fn restore_on_a_fresh_computer_with_the_printed_key_brings_back_identical_balances() {
    let rig = worked_day();
    let printed = switch_everything_on(&rig);
    rig.service.run(Scope::ALL).unwrap();
    let before = fingerprint(&rig);
    let cloud_copy = first_name(&rig, BackupKind::Cloud);

    // A new computer: an empty data file with only the first-run wizard done, no key, no state.
    let fresh = rig_for_fresh(&rig);
    assert!(fresh.secrets.get(SecretName::BackupKey).unwrap().is_none());
    // The Owner types the cloud account details, then the printed key (the key is not stored).
    fresh
        .service
        .settings_set(
            Some(fresh.w.owner.user_id),
            &mandi_backup::api::BackupSettingsInput {
                settings: settings_on(&fresh),
                cloud_access_key_id: Some("AKIATEST".into()),
                cloud_secret_access_key: Some("secret-test-value".into()),
            },
        )
        .unwrap();
    let no_key = fresh
        .service
        .restore_preview(&RestorePreviewInput {
            backup: BackupRef {
                kind: BackupKind::Cloud,
                name: cloud_copy.clone(),
            },
            key: None,
        })
        .unwrap_err();
    assert_eq!(no_key.code, BackupCode::ErrBackupKeyRequired);

    let started = Instant::now();
    fresh
        .service
        .restore(
            None,
            &restore_input(
                BackupKind::Cloud,
                &cloud_copy,
                Some(&printed.to_lowercase()),
            ),
        )
        .unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "NFR-REL-4 leaves 30 minutes in all"
    );
    let after = fingerprint(&fresh);
    assert_eq!(after.balances, before.balances);
    assert_eq!(after.journal, before.journal);
    assert_eq!(&after.audit[..before.audit.len()], &before.audit[..]);
    // The new computer keeps the key, so its next copies are locked with it.
    let status = fresh.service.status().unwrap();
    assert!(status.key_set && status.key_confirmed);
    assert_eq!(
        fresh.secrets.get(SecretName::BackupKey).unwrap().unwrap(),
        printed.replace('-', "")
    );
}

/// A second environment that shares the cloud folder of `first` but has its own data file.
fn rig_for_fresh(first: &Rig) -> Rig {
    let (dir, env) = common::env();
    common::set_up(&env);
    common::login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let w = support::World {
        dir,
        env,
        owner,
        rahim: 0,
        karim: 0,
        akram: 0,
        arshad: 0,
        naqdi: 0,
        pukhta_buyer: 0,
        pukhta_seller: 0,
        banana: 0,
        apple: 0,
        mooli: 0,
        weight_fruit: 0,
        box_: 0,
        partners: [0; 3],
    };
    let mut fresh = rig_for(w, None);
    // Same cloud folder and the same bucket settings.
    fresh.cloud = first.cloud.clone();
    fresh.service = Arc::new(BackupService::new(
        Arc::clone(fresh.w.env.ctx.db()),
        BackupConfig {
            backups_dir: fresh.backups.clone(),
            app_version: "0.0.0-test".to_owned(),
        },
        BackupParts {
            secrets: fresh.secrets.clone(),
            cloud: Arc::new(mandi_backup::cloud::FixedCloud(Arc::new(
                first.cloud.clone(),
            ))),
            disk: fresh.disk.clone(),
            hook: fresh.fault.clone(),
            pack: Arc::new(mandi_backup::daily_pack::PendingPack),
        },
    ));
    fresh
}

/// The totals of a backup count posted lines only, as `model.rs` says. A draft entry that was
/// never posted (a crash in the middle of a save leaves none in a committed file, but a copy of
/// an older file may hold one) must not change the debit, credit or cash totals.
#[test]
fn backup_totals_count_posted_lines_only() {
    let rig = worked_day();
    let conn = raw(&rig);
    let before = mandi_backup::snapshot::read_totals(&conn).unwrap();
    assert!(before.debit_paisa.as_paisa() > 0);
    assert_eq!(before.debit_paisa, before.credit_paisa);

    let cash: i64 = conn
        .query_row("SELECT id FROM accounts WHERE code = '01010001'", [], |r| {
            r.get(0)
        })
        .unwrap();
    conn.execute(
        "INSERT INTO journal_entries (created_at, created_by, updated_at, updated_by, entry_date,
             posting_type, source_type, source_id, narration, status, total_paisa)
         VALUES ('2026-10-10T00:00:00Z', 1, '2026-10-10T00:00:00Z', 1, '2026-10-10',
             'general', 'voucher', 0, NULL, 'draft', 0)",
        [],
    )
    .unwrap();
    let draft = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO journal_lines (created_at, created_by, updated_at, updated_by, entry_id, line_no,
             account_id, dr_paisa, cr_paisa, is_advance)
         VALUES ('2026-10-10T00:00:00Z', 1, '2026-10-10T00:00:00Z', 1, ?1, 1, ?2, 700000, 0, 0)",
        [draft, cash],
    )
    .unwrap();

    let after = mandi_backup::snapshot::read_totals(&conn).unwrap();
    assert_eq!(
        after.debit_paisa, before.debit_paisa,
        "a draft line is not counted"
    );
    assert_eq!(after.credit_paisa, before.credit_paisa);
    assert_eq!(
        after.cash_paisa, before.cash_paisa,
        "a draft line is not in cash"
    );
}
