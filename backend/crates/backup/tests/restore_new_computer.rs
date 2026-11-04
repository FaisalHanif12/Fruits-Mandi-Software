//! Plan 56 (PRD FR-20.11, NFR-INS-6, NFR-REL-4): the flows of "restore on a new computer, from
//! the first screen". An old shop with the PRD worked day backs up to a cloud (the real adapter
//! on memory), to a USB folder and to a file. A NEW computer (empty data file, empty key store,
//! empty credential store) lists the copies, previews one and restores it. Then it is checked
//! table by table, both hash chains are verified with the key that travelled in the copy, the
//! Owner signs in with the old password and with the recovery code, and the new computer makes
//! its own next copy, which restores again.
//!
//! The failures are in `restore_new_computer_faults.rs`.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
mod new_computer;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use mandi_backup::api::{
    BackupRef, FirstRunCheckName, FirstRunPreviewInput, FirstRunRestoreInput, RestoreSource,
    RestoreStage, RowStatus,
};
use mandi_backup::model::BackupKind;
use mandi_backup::run::Scope;
use mandi_backup::secrets::SecretName;
use mandi_backup::snapshot::{inspect, read_totals};
use mandi_services::db::audit::verify_audit_chain;
use mandi_services::health::{self, ChainState};
use mandi_services::{Secret, auth, ledger, vouchers};
use new_computer::*;

fn preview_input(source: RestoreSource, copy: Option<&str>, key: &str) -> FirstRunPreviewInput {
    FirstRunPreviewInput {
        source,
        copy: copy.map(str::to_owned),
        key: key.to_owned(),
    }
}

fn restore_input(source: RestoreSource, copy: &str, key: &str) -> FirstRunRestoreInput {
    FirstRunRestoreInput {
        source,
        copy: copy.to_owned(),
        key: key.to_owned(),
    }
}

/// Both hash chains of the new computer's data file verify with the key it holds now.
fn assert_chains_verify(new: &New) {
    ledger::verify_journal_chain(&new.ctx).expect("the journal chain verifies");
    let key = new
        .keys
        .current()
        .expect("the new computer has a chain key");
    new.ctx
        .db()
        .read(|conn| verify_audit_chain(conn, &key).map_err(|e| e.to_string()))
        .expect("the audit chain verifies");
    let check = health::check_books(&new.ctx).unwrap();
    assert!(matches!(check.journal_chain, ChainState::Ok), "{check:?}");
    assert!(matches!(check.audit_chain, ChainState::Ok), "{check:?}");
    assert_eq!(check.difference.as_paisa(), 0, "the books balance");
}

/// The restored data file holds the same rows as the copy, table by table. The audit log has the
/// rows of the copy first and then the rows the restore wrote; the settings differ only in the
/// backup row.
fn assert_same_as_copy(new: &New, plain: &std::path::Path) {
    let copy = table_hashes(plain, &["audit_log", "settings"]);
    let live = table_hashes(&new.db_path(), &["audit_log", "settings"]);
    assert_eq!(live, copy, "every other table is exactly the copy");

    let copy_conn = rusqlite_read(plain);
    let live_conn = rusqlite_read(&new.db_path());
    let last: i64 = copy_conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM audit_log", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(last > 0);
    let filter = format!("WHERE id <= {last}");
    assert_eq!(
        hash_rows(&live_conn, "audit_log", &filter),
        hash_rows(&copy_conn, "audit_log", &filter),
        "the audit rows of the copy are untouched; only new rows follow"
    );
    let not_backup = "WHERE key <> 'backup'";
    assert_eq!(
        hash_rows(&live_conn, "settings", not_backup),
        hash_rows(&copy_conn, "settings", not_backup),
        "every setting but the backup row is the copy's"
    );
}

fn rusqlite_read(path: &std::path::Path) -> mandi_services::db::rusqlite::Connection {
    mandi_services::db::rusqlite::Connection::open_with_flags(
        path,
        mandi_services::db::rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap()
}

fn stages(events: &Events) -> Vec<RestoreStage> {
    let mut seen: Vec<RestoreStage> = Vec::new();
    for event in events.all() {
        if seen.last() != Some(&event.stage) {
            seen.push(event.stage);
        }
    }
    seen
}

#[test]
fn restore_from_the_cloud_on_a_new_computer_brings_back_every_table_and_both_chains_verify() {
    let old = old_computer();
    let name = old.back_up();
    let (_keep, plain) = old.unpack_cloud(&name);
    let live_totals = {
        let conn = rusqlite_read(old.rig.w.dir.path().join("data").join("mandi.db").as_path());
        read_totals(&conn).unwrap()
    };

    let new = new_computer(old.cloud.clone());
    assert!(new.is_fresh(), "a new computer has no Owner");
    assert!(new.keys.current().is_none(), "and no chain key");
    let before_key = new.keys.current();

    // The list: one cloud copy, newest first.
    let copies = new
        .service
        .first_run_list(&list_input(cloud_source(&old)))
        .unwrap();
    assert_eq!(copies.total_count, 1);
    let row = copies.rows.first().unwrap();
    assert_eq!(row.kind, BackupKind::Cloud);
    assert_eq!(row.name, name);
    assert_eq!(row.status, RowStatus::Good);
    assert!(row.encrypted);

    // The preview: the newest copy, typed key in small letters with spaces. Its books are the
    // old shop's books at the time of the copy.
    let typed = old.printed_key.to_lowercase().replace('-', " ");
    let events = Events::default();
    let preview = new
        .service
        .first_run_preview(
            &preview_input(cloud_source(&old), None, &typed),
            &events.sink(),
        )
        .unwrap();
    assert_eq!(
        preview.copy,
        BackupRef {
            kind: BackupKind::Cloud,
            name: name.clone()
        }
    );
    assert!(!preview.needs_upgrade);
    assert_eq!(preview.totals, inspect(&plain).unwrap().totals);
    assert_eq!(preview.totals.debit_paisa, preview.totals.credit_paisa);
    assert_eq!(preview.totals.accounts, live_totals.accounts);
    assert_eq!(preview.totals.bikri_entries, 2);
    assert_eq!(preview.totals.journal_entries, live_totals.journal_entries);
    assert_eq!(preview.totals.debit_paisa, live_totals.debit_paisa);
    assert_eq!(preview.totals.cash_paisa, live_totals.cash_paisa);
    assert_eq!(preview.totals.last_entry_date, live_totals.last_entry_date);
    let names: Vec<FirstRunCheckName> = preview.checks.iter().map(|c| c.name).collect();
    assert_eq!(
        names,
        [
            FirstRunCheckName::Opens,
            FirstRunCheckName::Checksum,
            FirstRunCheckName::Integrity,
            FirstRunCheckName::Counts,
            FirstRunCheckName::Balanced,
            FirstRunCheckName::Chains,
        ]
    );
    assert!(preview.checks.iter().all(|c| c.ok));
    // A preview changes nothing on the new computer.
    assert!(new.is_fresh());
    assert_eq!(new.keys.current().is_some(), before_key.is_some());
    assert_eq!(new.secrets.peek(SecretName::BackupKey), None);
    let tmp = new.root.path().join("backups").join("tmp");
    assert_eq!(
        std::fs::read_dir(tmp).map_or(0, std::iter::Iterator::count),
        0
    );

    // The restore. The old laptop's local folder is not on this computer.
    std::fs::remove_dir_all(&old.local_dir).unwrap();
    let events = Events::default();
    let done = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &typed),
            &events.sink(),
        )
        .unwrap();
    assert_eq!(done.restored_from, preview.created_at);
    assert_eq!(done.totals.accounts, preview.totals.accounts);
    assert_eq!(done.totals.journal_entries, preview.totals.journal_entries);
    assert_eq!(done.upgraded_from, None);
    assert!(done.chain_key_restored, "the key came from the copy");
    assert!(!done.cloud_needs_keys, "the cloud keys were typed");
    assert!(!done.assistant_was_on);
    let seen = stages(&events);
    let at = |s: RestoreStage| seen.iter().position(|x| *x == s).unwrap();
    assert!(at(RestoreStage::Downloading) < at(RestoreStage::Decrypting));
    assert!(at(RestoreStage::Decrypting) < at(RestoreStage::Checking));
    assert!(at(RestoreStage::Checking) < at(RestoreStage::Keys));
    assert!(at(RestoreStage::Keys) < at(RestoreStage::PuttingInPlace));
    let download: Vec<u8> = events
        .all()
        .iter()
        .filter(|e| e.stage == RestoreStage::Downloading)
        .filter_map(|e| e.percent)
        .collect();
    assert_eq!(download.last(), Some(&100), "{download:?}");
    assert!(download.windows(2).all(|w| w[0] <= w[1]));

    // The data: every table is the copy; the books and both chains are right on the new key store.
    assert_same_as_copy(&new, &plain);
    assert!(new.keys.current().unwrap().same_as(&old_chain_key()));
    assert_chains_verify(&new);
    assert!(!new.is_fresh());

    // The keys are on the new computer: the backup key and the cloud keys the Owner typed.
    assert_eq!(
        new.secrets.peek(SecretName::BackupKey).as_deref(),
        Some(old.printed_key.replace('-', "").as_str())
    );
    assert_eq!(
        new.secrets.peek(SecretName::CloudAccessKeyId).as_deref(),
        Some("AKIANEWKEYID")
    );
    assert_eq!(
        new.secrets
            .peek(SecretName::CloudSecretAccessKey)
            .as_deref(),
        Some("new-secret-value-2")
    );

    // The settings: the cloud the Owner typed, so the daily copy goes there; a local folder that
    // is not on this computer is back to the default; the USB folder is kept.
    let settings = new.service.settings().unwrap();
    assert!(settings.cloud_enabled);
    assert_eq!(settings.cloud_endpoint, "https://new.example.invalid");
    assert_eq!(settings.cloud_bucket, "new-bucket");
    assert_eq!(settings.local_dir, "");
    assert_eq!(settings.usb_folder, old.rig.usb.display().to_string());
    assert!(settings.usb_enabled);

    // The status: the key is known and confirmed (he typed it from the printed sheet).
    let status = new.service.status().unwrap();
    assert!(status.key_set && status.key_confirmed);
    assert!(status.cloud_ready);

    // Sign in with the old password (case-free user name).
    let session = auth::login(&new.ctx, "owner", &Secret::new(common::OWNER_PASSWORD)).unwrap();
    assert_eq!(session.username, "owner");
    // The learned word and the preference came back.
    let pairs: i64 = new
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT COUNT(*) FROM translit_pairs WHERE roman = 'arshad'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
        })
        .unwrap();
    assert_eq!(pairs, 1);
    let text_size: String = new
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT value_json FROM user_prefs WHERE key = 'text_size'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
        })
        .unwrap();
    assert_eq!(text_size.trim_matches('"'), "120");

    // A new entry on the new computer continues the same chains.
    let actor = auth::require(&new.ctx).unwrap();
    vouchers::save_receipt(&new.ctx, &actor, &rupees_input(old.rig.w.arshad, "1,000")).unwrap();
    assert_chains_verify(&new);
    assert!(new.keys.current().unwrap().same_as(&old_chain_key()));

    // The restore itself is in the audit log, once.
    let restores: i64 = new
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT COUNT(*) FROM audit_log WHERE action = 'restore' AND entity = 'backup' \
                 AND new_json LIKE '%\"new_computer\":true%'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
        })
        .unwrap();
    assert_eq!(restores, 1);

    // The scratch folder is gone.
    let tmp = new.root.path().join("backups").join("tmp");
    let left = std::fs::read_dir(tmp).map_or(0, std::iter::Iterator::count);
    assert_eq!(left, 0, "no scratch files are left");
}

fn rupees_input(account: i64, amount: &str) -> mandi_services::core::vouchers::ReceiptInput {
    harness::rupees_input(account, amount)
}

#[test]
fn the_recovery_code_works_on_the_new_computer() {
    let old = old_computer();
    let name = old.back_up();
    let new = new_computer(old.cloud.clone());
    new.service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap();
    let recovered = auth::recover(
        &new.ctx,
        &auth::RecoverRaw {
            username: "owner".into(),
            recovery_code: Secret::new(old.recovery_code.clone()),
            new_password: Secret::new("brand-new-1"),
            new_password_repeat: Secret::new("brand-new-1"),
        },
    )
    .unwrap();
    assert!(!recovered.is_empty(), "a new code is given");
    auth::login(&new.ctx, "owner", &Secret::new("brand-new-1")).unwrap();
    assert_chains_verify(&new);
}

#[test]
fn a_usb_folder_and_a_single_file_restore_too_and_the_cloud_keys_are_asked_for() {
    let old = old_computer();
    let _cloud_name = old.back_up();
    let usb_name = old.latest(BackupKind::Usb);

    // The folder: the copies in it, with the kind usb.
    let new = new_computer(old.cloud.clone());
    let folder = RestoreSource::Folder(old.rig.usb.display().to_string());
    let copies = new
        .service
        .first_run_list(&list_input(folder.clone()))
        .unwrap();
    assert_eq!(row_names(&copies.rows), std::slice::from_ref(&usb_name));
    assert_eq!(copies.rows.first().unwrap().kind, BackupKind::Usb);
    let (_keep, plain) = {
        // The USB copy and the cloud copy are the same snapshot: compare with the unpacked one.
        old.unpack_cloud(&old.latest(BackupKind::Cloud))
    };
    let done = new
        .service
        .first_run_restore(&restore_input(folder, &usb_name, &old.printed_key), &|_| {})
        .unwrap();
    assert!(done.cloud_needs_keys, "no cloud keys travel in a copy");
    assert!(done.chain_key_restored);
    assert_same_as_copy(&new, &plain);
    assert_chains_verify(&new);
    // The cloud settings are those of the old computer; no keys were typed.
    let settings = new.service.settings().unwrap();
    assert_eq!(settings.cloud_bucket, "old-bucket");
    assert_eq!(new.secrets.peek(SecretName::CloudAccessKeyId), None);
    assert_eq!(
        new.secrets.peek(SecretName::BackupKey).as_deref(),
        Some(old.printed_key.replace('-', "").as_str())
    );
    assert_eq!(settings.local_dir, old.local_dir.display().to_string());

    // One file picked by hand: its own row, and the copy argument is ignored.
    let new2 = new_computer(old.cloud.clone());
    let file = old.rig.usb.join(&usb_name);
    let source = RestoreSource::File(file.display().to_string());
    let one = new2
        .service
        .first_run_list(&list_input(source.clone()))
        .unwrap();
    assert_eq!(row_names(&one.rows), std::slice::from_ref(&usb_name));
    let preview = new2
        .service
        .first_run_preview(
            &preview_input(source.clone(), Some("ignored"), &old.printed_key),
            &|_| {},
        )
        .unwrap();
    assert_eq!(preview.copy.name, usb_name);
    new2.service
        .first_run_restore(&restore_input(source, "ignored", &old.printed_key), &|_| {})
        .unwrap();
    assert_chains_verify(&new2);
}

#[test]
fn the_new_computer_makes_the_next_copy_and_that_one_restores_on_another_computer() {
    let old = old_computer();
    let first = old.back_up();
    let new = new_computer(old.cloud.clone());
    new.service
        .first_run_restore(
            &restore_input(cloud_source(&old), &first, &old.printed_key),
            &|_| {},
        )
        .unwrap();
    auth::login(&new.ctx, "owner", &Secret::new(common::OWNER_PASSWORD)).unwrap();
    let actor = auth::require(&new.ctx).unwrap();
    vouchers::save_receipt(&new.ctx, &actor, &rupees_input(old.rig.w.arshad, "2,000")).unwrap();

    // The daily copy goes on to the place the Owner typed (the same memory cloud here).
    let report = new
        .service
        .run(Scope {
            local: true,
            usb: false,
            cloud: true,
        })
        .unwrap();
    assert!(report.results.iter().all(|r| r.ok), "{report:?}");
    let listed = new.service.first_run_list(&list_input(cloud_source(&old)));
    // The new computer has an Owner now: the first-run commands are closed.
    assert!(listed.is_err());
    let second = new
        .service
        .list(&mandi_backup::api::BackupListInput {
            kind: Some(BackupKind::Cloud),
            offset: 0,
            limit: 10,
        })
        .unwrap();
    assert_eq!(second.rows.len(), 2, "the old copy and the new one");
    let newest = second.rows.first().unwrap().name.clone();
    assert_ne!(newest, first);

    // Another new computer restores the NEWEST copy: the key travelled again, and the receipt
    // entered on the new computer is in it.
    let third = new_computer(old.cloud.clone());
    let done = third
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &newest, &old.printed_key),
            &|_| {},
        )
        .unwrap();
    assert!(done.chain_key_restored);
    assert_chains_verify(&third);
    assert!(third.keys.current().unwrap().same_as(&old_chain_key()));
    let receipts: i64 = third
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT COUNT(*) FROM vouchers WHERE type = 'receipt'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
        })
        .unwrap();
    assert_eq!(receipts, 2, "the receipt of the old day and the new one");
}

#[test]
fn a_copy_from_an_older_version_is_upgraded_while_it_is_put_in_place() {
    // A copy whose data file has the schema of two versions ago, with the key in the bundle.
    let old = old_computer();
    let new = new_computer(old.cloud.clone());
    let app = new.ctx.db().migration().to;
    let older = i64::try_from(app).unwrap() - 2;
    let dir = tempfile::tempdir().unwrap();
    let plain = dir.path().join("older.db");
    {
        let mut conn = mandi_services::db::rusqlite::Connection::open(&plain).unwrap();
        mandi_db::migrate::migrations()
            .to_version(&mut conn, usize::try_from(older).unwrap())
            .unwrap();
        // A copy of a shop that was set up holds its Owner. A copy with none is refused
        // (ERR_BACKUP_NO_OWNER), so this one has the row any set-up shop has.
        conn.execute(
            "INSERT INTO users (username, password_hash, display_name, role) \
             VALUES ('owner', 'not-a-real-hash', 'Owner', 'owner')",
            [],
        )
        .unwrap();
        let _: String = conn
            .pragma_update_and_check(None, "journal_mode", "DELETE", |row| row.get(0))
            .unwrap();
    }
    let key = mandi_backup::key::BackupKey::parse(&old.printed_key).unwrap();
    let name = put_crafted_copy(
        &old.cloud,
        &key,
        &plain,
        "20261011T090000000Z",
        Some(&old_chain_key()),
        true,
    );

    let preview = new
        .service
        .first_run_preview(
            &preview_input(cloud_source(&old), Some(&name), &old.printed_key),
            &|_| {},
        )
        .unwrap();
    assert!(preview.needs_upgrade);
    assert_eq!(preview.schema_version, older);
    assert_eq!(preview.app_schema_version, i64::try_from(app).unwrap());

    let done = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap();
    assert_eq!(done.upgraded_from, Some(older));
    let version: i64 = new
        .ctx
        .db()
        .read(|c| {
            c.query_row("PRAGMA user_version", [], |r| r.get(0))
                .map_err(|e| e.to_string())
        })
        .unwrap();
    assert_eq!(version, i64::try_from(app).unwrap());
    assert!(new.keys.current().unwrap().same_as(&old_chain_key()));
    health::check_books(&new.ctx).unwrap();
}

#[test]
fn the_signed_in_restore_also_installs_the_key_of_the_copy() {
    // A computer where the wizard was already run (so it has an Owner and a key of its own) and
    // then restores a copy of another computer: the books of the copy need the copy's key.
    let old = old_computer();
    let name = old.back_up();
    let new = new_computer(old.cloud.clone());
    auth::first_run(&new.ctx, &common::first_run_input()).unwrap();
    auth::login(&new.ctx, "owner", &Secret::new(common::OWNER_PASSWORD)).unwrap();
    let own_key = new.keys.current().unwrap();
    assert!(!own_key.same_as(&old_chain_key()));
    let actor = auth::require(&new.ctx).unwrap();
    // The cloud and the typed key, as the signed-in screen would have them.
    new.service
        .settings_set(
            Some(actor.user_id),
            &mandi_backup::api::BackupSettingsInput {
                settings: mandi_backup::settings::BackupSettings {
                    cloud_enabled: true,
                    cloud_endpoint: "https://new.example.invalid".into(),
                    cloud_bucket: "new-bucket".into(),
                    ..mandi_backup::settings::BackupSettings::default()
                },
                cloud_access_key_id: Some("AKIANEWKEYID".into()),
                cloud_secret_access_key: Some("new-secret-value-2".into()),
            },
        )
        .unwrap();
    new.service
        .restore(
            Some(actor.user_id),
            &mandi_backup::api::RestoreInput {
                backup: BackupRef {
                    kind: BackupKind::Cloud,
                    name,
                },
                key: Some(old.printed_key.clone()),
                confirmed: true,
            },
        )
        .unwrap();
    assert!(new.keys.current().unwrap().same_as(&old_chain_key()));
    assert_chains_verify(&new);
}

#[test]
fn without_the_key_of_the_copy_the_chains_of_a_restored_shop_would_show_as_broken() {
    // The control for the tests above: the check really sees a wrong key. Put a different key
    // in the new computer after a good restore, as an empty key store would make by itself, and
    // both chains fail from the first row.
    let old = old_computer();
    let name = old.back_up();
    let new = new_computer(old.cloud.clone());
    new.service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap();
    assert_chains_verify(&new);
    let stranger = mandi_services::db::audit::AuditKey::from_bytes([42; 32]);
    new.ctx.db().auditor().install_key(stranger).unwrap();
    let check = health::check_books(&new.ctx).unwrap();
    assert!(
        matches!(check.journal_chain, ChainState::Broken(_)),
        "{check:?}"
    );
    assert!(
        matches!(check.audit_chain, ChainState::Broken(_)),
        "{check:?}"
    );
}
