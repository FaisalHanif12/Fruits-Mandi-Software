//! Faults and everyday behaviour of the backup (plan 18, PRD FR-20.1 to FR-20.10, NFR-BAK-1 to
//! NFR-BAK-15, NFR-SPD-15): a full disk, a USB drive pulled out half way, no internet for a while,
//! a damaged copy, a wrong key, retention, the schedule, the key and the settings.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use harness::*;
use mandi_backup::api::{
    BackupListInput, BackupRef, BackupSettingsInput, CloudLevel, RestoreInput, RestorePreviewInput,
    RowStatus, TestCheckName, TestRestoreInput,
};
use mandi_backup::error::BackupCode;
use mandi_backup::model::BackupKind;
use mandi_backup::run::Scope;
use mandi_backup::schedule::Job;
use mandi_backup::secrets::{SecretName, SecretStore};
use mandi_backup::settings::BackupSettings;
use mandi_services::vouchers;

fn names(rig: &Rig, kind: BackupKind) -> Vec<String> {
    rig.service
        .list(&BackupListInput {
            kind: Some(kind),
            offset: 0,
            limit: 1000,
        })
        .unwrap()
        .rows
        .into_iter()
        .map(|r| r.name)
        .collect()
}

fn files_under(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                found.extend(files_under(&path));
            } else {
                found.push(path);
            }
        }
    }
    found
}

fn no_partial_files(rig: &Rig) {
    let partial: Vec<_> = files_under(rig.root.path())
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "partial"))
        .collect();
    assert!(partial.is_empty(), "half-written files left: {partial:?}");
}

/// Adds `bytes` of random data that does not compress, so a copy is big enough to break half way.
fn add_ballast(rig: &Rig, bytes: i64) {
    rig.w
        .env
        .ctx
        .db()
        .write(|tx| {
            tx.execute_batch(&format!(
                "CREATE TABLE ballast (x BLOB); INSERT INTO ballast VALUES (randomblob({bytes}));"
            ))?;
            Ok::<(), mandi_services::db::DbError>(())
        })
        .unwrap();
}

fn restore_in(kind: BackupKind, name: &str, key: Option<&str>) -> RestoreInput {
    RestoreInput {
        backup: BackupRef {
            kind,
            name: name.to_owned(),
        },
        key: key.map(str::to_owned),
        confirmed: true,
    }
}

fn advance(rig: &Rig, seconds: i64) {
    rig.w.env.clock.advance_seconds(seconds);
}

fn flip_middle(path: &Path) {
    let mut bytes = fs::read(path).unwrap();
    let at = bytes.len() / 2;
    bytes[at] ^= 0xFF;
    fs::write(path, bytes).unwrap();
}

#[test]
fn a_full_disk_stops_the_copy_cleanly_and_the_old_copies_and_the_data_are_safe() {
    let rig = small_world();
    switch_everything_on(&rig);
    let first = rig.service.run(Scope::ALL).unwrap();
    assert!(first_ok(&first, BackupKind::Local));
    let before = fingerprint(&rig);
    let copies = names(&rig, BackupKind::Local);

    advance(&rig, 120);
    rig.disk.set(1024);
    let report = rig.service.run(Scope::ALL).unwrap();
    assert!(report.results.iter().all(|r| !r.ok));
    assert!(
        report
            .results
            .iter()
            .all(|r| r.code == Some(BackupCode::ErrDiskFull)),
        "{report:?}"
    );
    assert_eq!(
        names(&rig, BackupKind::Local),
        copies,
        "the old copy is still the only one"
    );
    no_partial_files(&rig);
    let status = rig.service.status().unwrap();
    assert!(
        status
            .errors
            .iter()
            .any(|e| e.code == BackupCode::ErrDiskFull)
    );
    // The live data is untouched and still works.
    assert_eq!(fingerprint(&rig), before);
    vouchers::save_receipt(
        &rig.w.env.ctx,
        &rig.w.owner,
        &rupees_input(rig.w.arshad, "100"),
    )
    .unwrap();

    rig.disk.set(10 * 1024 * 1024 * 1024);
    advance(&rig, 120);
    let again = rig.service.run(Scope::ALL).unwrap();
    assert!(again.results.iter().all(|r| r.ok), "{again:?}");
    assert!(
        rig.service.status().unwrap().errors.is_empty(),
        "a good copy clears the problem"
    );
}

#[test]
fn a_usb_drive_pulled_out_half_way_leaves_no_half_copy_and_the_other_places_are_not_stopped() {
    let rig = small_world();
    switch_everything_on(&rig);
    add_ballast(&rig, 3_000_000);
    rig.fault.arm(BackupKind::Usb, 1_000_000);
    let report = rig.service.run(Scope::ALL).unwrap();
    let usb = report
        .results
        .iter()
        .find(|r| r.kind == BackupKind::Usb)
        .unwrap();
    assert!(!usb.ok);
    assert_eq!(usb.code, Some(BackupCode::ErrBackupUsbFailed));
    assert!(first_ok(&report, BackupKind::Local) && first_ok(&report, BackupKind::Cloud));
    assert!(names(&rig, BackupKind::Usb).is_empty());
    assert!(
        files_under(&rig.usb).is_empty(),
        "nothing left on the drive"
    );
    no_partial_files(&rig);

    rig.fault.disarm();
    advance(&rig, 120);
    let again = rig.service.run(Scope::ALL).unwrap();
    assert!(first_ok(&again, BackupKind::Usb), "{again:?}");
    assert_eq!(names(&rig, BackupKind::Usb).len(), 1);
}

#[test]
fn a_drive_that_is_not_plugged_in_is_reported_not_skipped() {
    let rig = small_world();
    switch_everything_on(&rig);
    fs::remove_dir_all(rig.usb.parent().unwrap()).unwrap();
    let report = rig.service.run(Scope::ALL).unwrap();
    let usb = report
        .results
        .iter()
        .find(|r| r.kind == BackupKind::Usb)
        .unwrap();
    assert_eq!(usb.code, Some(BackupCode::ErrBackupUsbNotPresent));
    assert!(first_ok(&report, BackupKind::Local));
    assert!(!rig.service.status().unwrap().usb_present);
}

#[test]
fn no_internet_queues_the_cloud_copy_and_it_catches_up_when_the_internet_is_back() {
    let rig = small_world();
    switch_everything_on(&rig);
    rig.cloud.set_offline(true);
    let report = rig.service.run(Scope::ALL).unwrap();
    let cloud = report
        .results
        .iter()
        .find(|r| r.kind == BackupKind::Cloud)
        .unwrap();
    assert!(!cloud.ok && cloud.queued);
    assert_eq!(cloud.code, Some(BackupCode::ErrBackupCloudFailed));
    assert!(first_ok(&report, BackupKind::Local) && first_ok(&report, BackupKind::Usb));
    let status = rig.service.status().unwrap();
    assert_eq!(status.queue_len, 1);
    assert_eq!(status.cloud_level, CloudLevel::Red, "never uploaded");
    assert_eq!(
        files_under(&rig.backups.join("outbox")).len(),
        2,
        "bundle and manifest wait"
    );

    // The scheduler tries at once the first time, then every 10 minutes.
    assert!(rig.service.due_jobs().unwrap().contains(&Job::RetryQueue));
    rig.service.tick().unwrap();
    assert_eq!(rig.service.status().unwrap().queue_len, 1);
    advance(&rig, 9 * 60);
    assert!(!rig.service.due_jobs().unwrap().contains(&Job::RetryQueue));
    advance(&rig, 2 * 60);
    assert!(rig.service.due_jobs().unwrap().contains(&Job::RetryQueue));

    rig.cloud.set_offline(false);
    rig.service.tick().unwrap();
    let status = rig.service.status().unwrap();
    assert_eq!(status.queue_len, 0);
    assert!(status.last_cloud_at.is_some());
    assert!(files_under(&rig.backups.join("outbox")).is_empty());
    assert_eq!(names(&rig, BackupKind::Cloud).len(), 1);
    assert!(status.errors.iter().all(|e| e.kind != BackupKind::Cloud));
}

#[test]
fn a_damaged_copy_is_found_and_never_restored_silently() {
    let rig = worked_day();
    switch_everything_on(&rig);
    rig.service.run(Scope::ALL).unwrap();
    let before = fingerprint(&rig);
    let local = names(&rig, BackupKind::Local).remove(0);
    let usb = names(&rig, BackupKind::Usb).remove(0);
    let cloud = names(&rig, BackupKind::Cloud).remove(0);
    let local_path = rig.backups.join("local").join(&local);
    let usb_path = rig.usb.join(&usb);
    let cloud_path = rig.root.path().join("cloud").join("backups").join(&cloud);
    for path in [&local_path, &usb_path, &cloud_path] {
        flip_middle(path);
    }
    // A cut file too.
    let bytes = fs::read(&usb_path).unwrap();
    fs::write(&usb_path, &bytes[..bytes.len() - 100]).unwrap();

    for (kind, name) in [
        (BackupKind::Local, &local),
        (BackupKind::Usb, &usb),
        (BackupKind::Cloud, &cloud),
    ] {
        let preview = rig
            .service
            .restore_preview(&RestorePreviewInput {
                backup: BackupRef {
                    kind,
                    name: name.clone(),
                },
                key: None,
            })
            .unwrap_err();
        assert_eq!(preview.code, BackupCode::ErrBackupCorrupt, "{kind:?}");
        let error = rig
            .service
            .restore(None, &restore_in(kind, name, None))
            .unwrap_err();
        assert_eq!(error.code, BackupCode::ErrBackupCorrupt, "{kind:?}");
    }
    assert_eq!(fingerprint(&rig), before, "nothing was restored");
    assert!(
        !rig.service.safety_dir().exists(),
        "no safety copy: nothing was touched"
    );
    no_partial_files(&rig);

    // Without a manifest the encryption itself still refuses a damaged bundle.
    fs::remove_file(
        rig.root
            .path()
            .join("cloud")
            .join("backups")
            .join(mandi_backup::names::manifest_name(&cloud)),
    )
    .unwrap();
    let error = rig
        .service
        .restore(None, &restore_in(BackupKind::Cloud, &cloud, None))
        .unwrap_err();
    assert!(
        matches!(
            error.code,
            BackupCode::ErrBackupCorrupt | BackupCode::ErrBackupKeyWrong
        ),
        "{error:?}"
    );
    assert_eq!(fingerprint(&rig), before);

    // The test restore says the same without failing: the checks show what is wrong.
    let result = rig
        .service
        .test_restore(&TestRestoreInput {
            backup: Some(BackupRef {
                kind: BackupKind::Local,
                name: local,
            }),
            key: None,
        })
        .unwrap();
    assert!(!result.ok);
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == TestCheckName::Checksum && !c.ok)
    );
}

#[test]
fn the_wrong_key_changes_nothing_and_the_missing_key_is_asked_for() {
    let rig = worked_day();
    let printed = switch_everything_on(&rig);
    rig.service.run(Scope::ALL).unwrap();
    let before = fingerprint(&rig);
    let cloud = names(&rig, BackupKind::Cloud).remove(0);

    let other = mandi_backup::key::BackupKey::generate().unwrap().display();
    assert_ne!(other, printed);
    let error = rig
        .service
        .restore(None, &restore_in(BackupKind::Cloud, &cloud, Some(&other)))
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrBackupKeyWrong);
    assert_eq!(error.field.as_deref(), Some("key"));
    let error = rig
        .service
        .restore(None, &restore_in(BackupKind::Cloud, &cloud, Some("ABC")))
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrBackupKeyFormat);
    assert_eq!(fingerprint(&rig), before);
    assert!(!rig.service.safety_dir().exists());

    rig.secrets.delete(SecretName::BackupKey).unwrap();
    let error = rig
        .service
        .restore(None, &restore_in(BackupKind::Cloud, &cloud, None))
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrBackupKeyRequired);
    // The right key typed from the printed sheet works, in any case.
    rig.service
        .restore(
            None,
            &restore_in(BackupKind::Cloud, &cloud, Some(&printed.to_lowercase())),
        )
        .unwrap();
}

#[test]
fn a_restore_needs_the_confirmation_and_a_name_cannot_leave_the_folder() {
    let rig = small_world();
    switch_everything_on(&rig);
    rig.service.run(Scope::ALL).unwrap();
    let local = names(&rig, BackupKind::Local).remove(0);
    let mut input = restore_in(BackupKind::Local, &local, None);
    input.confirmed = false;
    let error = rig.service.restore(None, &input).unwrap_err();
    assert_eq!(error.code, BackupCode::ErrBackupNotConfirmed);
    assert_eq!(error.field.as_deref(), Some("confirmed"));
    for bad in [
        "../../etc/passwd",
        "mandi-local-x/../y.db",
        "mandi-local-20261010T090000000Z.db/..",
    ] {
        let error = rig
            .service
            .restore(None, &restore_in(BackupKind::Local, bad, None))
            .unwrap_err();
        assert_eq!(error.code, BackupCode::ErrOutOfRange, "{bad}");
    }
    let missing = restore_in(
        BackupKind::Local,
        "mandi-local-20200101T000000000Z.db",
        None,
    );
    assert_eq!(
        rig.service.restore(None, &missing).unwrap_err().code,
        BackupCode::ErrBackupNotFound
    );
}

#[test]
fn a_copy_from_a_newer_app_is_refused() {
    let rig = small_world();
    switch_everything_on(&rig);
    rig.service.run(Scope::LOCAL_ONLY).unwrap();
    let local = names(&rig, BackupKind::Local).remove(0);
    let path = rig.backups.join("local").join(&local);
    let conn = mandi_services::db::rusqlite::Connection::open(&path).unwrap();
    conn.pragma_update(None, "user_version", 99).unwrap();
    drop(conn);
    // No manifest: nothing records the old checksum, so the file itself is what is judged.
    fs::remove_file(
        rig.backups
            .join("local")
            .join(mandi_backup::names::manifest_name(&local)),
    )
    .unwrap();
    let before = fingerprint(&rig);
    let error = rig
        .service
        .restore(None, &restore_in(BackupKind::Local, &local, None))
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrBackupNewerThanApp);
    assert_eq!(fingerprint(&rig), before);
}

#[test]
fn retention_keeps_the_newest_48_local_and_30_usb_copies() {
    let rig = small_world();
    switch_everything_on(&rig);
    for _ in 0..50 {
        advance(&rig, 61);
        assert!(first_ok(
            &rig.service.run(Scope::LOCAL_ONLY).unwrap(),
            BackupKind::Local
        ));
    }
    let local = names(&rig, BackupKind::Local);
    assert_eq!(local.len(), 48);
    let manifests = files_under(&rig.backups.join("local"))
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .count();
    assert_eq!(manifests, 48);
    for _ in 0..32 {
        advance(&rig, 61);
        assert!(first_ok(
            &rig.service.run(Scope::USB_ONLY).unwrap(),
            BackupKind::Usb
        ));
    }
    assert_eq!(names(&rig, BackupKind::Usb).len(), 30);
}

#[test]
fn the_cloud_keeps_thirty_days_and_a_copy_for_each_older_month() {
    let rig = small_world();
    switch_everything_on(&rig);
    let cloud_only = Scope {
        local: false,
        usb: false,
        cloud: true,
    };
    for _ in 0..75 {
        advance(&rig, 24 * 3600);
        let report = rig.service.run(cloud_only).unwrap();
        assert!(first_ok(&report, BackupKind::Cloud), "{report:?}");
    }
    let kept = names(&rig, BackupKind::Cloud);
    // 30 daily copies, plus the newest copy of each older month (75 days reach back 3 months).
    assert!(kept.len() >= 30 && kept.len() <= 30 + 3, "{}", kept.len());
    let objects = files_under(&rig.root.path().join("cloud").join("backups"));
    assert_eq!(
        objects.len(),
        kept.len() * 2,
        "each copy has its manifest, and no strays"
    );
}

#[test]
fn a_run_in_progress_says_busy_and_the_status_says_running() {
    let spy = Arc::new(Spy(OnceLock::new()));
    let rig = rig_for(support::world(), Some(spy.clone()));
    switch_everything_on(&rig);
    let service = Arc::clone(&rig.service);
    let seen = Arc::new(std::sync::Mutex::new(None));
    let seen_in = Arc::clone(&seen);
    spy.0
        .set(Box::new(move || {
            let nested = service.run(Scope::LOCAL_ONLY).err().map(|e| e.code);
            let running = service.status().map(|s| s.running).unwrap_or(false);
            let restore = service
                .restore(
                    None,
                    &restore_in(
                        BackupKind::Local,
                        "mandi-local-20200101T000000000Z.db",
                        None,
                    ),
                )
                .err()
                .map(|e| e.code);
            *seen_in.lock().unwrap() = Some((nested, running, restore));
        }))
        .ok();
    rig.service.run(Scope::ALL).unwrap();
    let seen = seen.lock().unwrap().take().unwrap();
    assert_eq!(
        seen,
        (
            Some(BackupCode::ErrBackupBusy),
            true,
            Some(BackupCode::ErrBackupBusy)
        )
    );
    assert!(!rig.service.status().unwrap().running);
}

#[test]
fn the_key_is_shown_once_confirmed_by_typing_and_never_stored_in_the_data_file() {
    let rig = small_world();
    rig.service
        .settings_set(
            Some(rig.w.owner.user_id),
            &BackupSettingsInput {
                settings: settings_on(&rig),
                cloud_access_key_id: Some("AKIA-SECRET-ID".into()),
                cloud_secret_access_key: Some("cloud-secret-VALUE".into()),
            },
        )
        .unwrap();
    let status = rig.service.status().unwrap();
    assert!(!status.key_set);
    let first = rig.service.key_setup(Some(rig.w.owner.user_id)).unwrap();
    assert_eq!(first.key.len(), 29);
    // Not used yet and not confirmed: the wizard can be run again and the key is replaced.
    let second = rig.service.key_setup(None).unwrap();
    assert_ne!(first.key, second.key);
    let wrong = rig.service.key_confirm(None, &first.key).unwrap_err();
    assert_eq!(wrong.code, BackupCode::ErrBackupKeyWrong);
    assert_eq!(wrong.field.as_deref(), Some("key"));
    assert_eq!(
        rig.service.key_confirm(None, "nope").unwrap_err().code,
        BackupCode::ErrBackupKeyFormat
    );
    assert!(!rig.service.status().unwrap().key_confirmed);
    rig.service
        .key_confirm(Some(rig.w.owner.user_id), &second.key.to_lowercase())
        .unwrap();
    assert!(rig.service.status().unwrap().key_confirmed);
    // Confirmed: it is kept for good.
    assert_eq!(
        rig.service.key_setup(None).unwrap_err().code,
        BackupCode::ErrBackupKeyAlreadySet
    );

    // Neither the key nor the cloud keys are in the data file, its log, or the state file.
    rig.service.run(Scope::ALL).unwrap();
    let mut hay = Vec::new();
    for entry in ["data/mandi.db", "data/mandi.db-wal"] {
        if let Ok(bytes) = fs::read(rig.w.dir.path().join(entry)) {
            hay.extend(bytes);
        }
    }
    for path in files_under(&rig.backups) {
        if path.extension().is_some_and(|e| e == "json" || e == "db") {
            hay.extend(fs::read(path).unwrap());
        }
    }
    let text = String::from_utf8_lossy(&hay);
    for secret in [
        second.key.replace('-', ""),
        second.key.clone(),
        "AKIA-SECRET-ID".to_owned(),
        "cloud-secret-VALUE".to_owned(),
    ] {
        assert!(!text.contains(&secret), "{secret} found on disk");
    }
    // The key that was replaced locked nothing.
    assert!(rig.service.settings_view().unwrap().cloud_secret_set);
}

#[test]
fn an_encrypted_copy_without_a_key_fails_loudly_and_local_copies_still_run() {
    let rig = small_world();
    rig.service
        .settings_set(
            Some(rig.w.owner.user_id),
            &BackupSettingsInput {
                settings: settings_on(&rig),
                cloud_access_key_id: None,
                cloud_secret_access_key: None,
            },
        )
        .unwrap();
    let report = rig.service.run(Scope::ALL).unwrap();
    assert!(first_ok(&report, BackupKind::Local));
    for kind in [BackupKind::Usb, BackupKind::Cloud] {
        let result = report.results.iter().find(|r| r.kind == kind).unwrap();
        assert_eq!(
            result.code,
            Some(BackupCode::ErrBackupKeyNotSet),
            "{kind:?}"
        );
    }
}

#[test]
fn settings_are_checked_audited_and_the_secrets_stay_out_of_the_log() {
    let rig = small_world();
    let bad = BackupSettings {
        cloud_enabled: true,
        closing_time: "9pm".into(),
        ..BackupSettings::default()
    };
    let error = rig
        .service
        .settings_set(
            Some(1),
            &BackupSettingsInput {
                settings: bad,
                cloud_access_key_id: None,
                cloud_secret_access_key: None,
            },
        )
        .unwrap_err();
    let fields: std::collections::BTreeSet<String> =
        error.all().into_iter().filter_map(|e| e.field).collect();
    assert!(
        ["closing_time", "cloud_endpoint", "cloud_bucket"]
            .iter()
            .all(|f| fields.contains(*f)),
        "{fields:?}"
    );

    let audit_before: i64 = raw(&rig)
        .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
        .unwrap();
    let view = rig
        .service
        .settings_set(
            Some(rig.w.owner.user_id),
            &BackupSettingsInput {
                settings: settings_on(&rig),
                cloud_access_key_id: Some("id-123".into()),
                cloud_secret_access_key: Some("sec-456".into()),
            },
        )
        .unwrap();
    assert!(view.cloud_access_key_set && view.cloud_secret_set);
    assert_eq!(view.settings.closing_time, "21:00");
    let rows: Vec<(String, Option<String>, Option<String>)> = raw(&rig)
        .prepare("SELECT entity, old_json, new_json FROM audit_log WHERE id > ?1 ORDER BY id")
        .unwrap()
        .query_map([audit_before], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert!(rows.iter().any(|r| r.0 == "backup_settings"));
    let all = format!("{rows:?}");
    assert!(!all.contains("id-123") && !all.contains("sec-456"));
    // An empty text removes a key.
    let view = rig
        .service
        .settings_set(
            Some(1),
            &BackupSettingsInput {
                settings: settings_on(&rig),
                cloud_access_key_id: None,
                cloud_secret_access_key: Some(String::new()),
            },
        )
        .unwrap();
    assert!(view.cloud_access_key_set && !view.cloud_secret_set);
}

#[test]
fn the_cloud_colour_follows_the_age_of_the_last_good_copy() {
    let rig = small_world();
    assert_eq!(rig.service.status().unwrap().cloud_level, CloudLevel::Off);
    switch_everything_on(&rig);
    assert_eq!(
        rig.service.status().unwrap().cloud_level,
        CloudLevel::Red,
        "none yet"
    );
    rig.service.run(Scope::ALL).unwrap();
    let hours = |h: i64| advance(&rig, h * 3600);
    let level = || rig.service.status().unwrap().cloud_level;
    assert_eq!(level(), CloudLevel::Green);
    hours(23);
    assert_eq!(level(), CloudLevel::Green);
    hours(2);
    assert_eq!(level(), CloudLevel::Red);
    hours(24);
    assert_eq!(level(), CloudLevel::Alert);
    assert_eq!(rig.service.status().unwrap().cloud_age_hours, Some(49));
}

#[test]
fn a_test_restore_checks_the_latest_copy_without_touching_the_data_and_reminds_monthly() {
    let rig = worked_day();
    switch_everything_on(&rig);
    rig.service.run(Scope::ALL).unwrap();
    assert!(
        rig.service.status().unwrap().test_restore_due,
        "a cloud copy exists and no test yet"
    );
    let before = fingerprint(&rig);
    let result = rig
        .service
        .test_restore(&TestRestoreInput {
            backup: None,
            key: None,
        })
        .unwrap();
    assert_eq!(
        result.backup.kind,
        BackupKind::Cloud,
        "the newest cloud copy"
    );
    assert!(result.ok, "{result:?}");
    assert_eq!(result.checks.len(), 5);
    assert_eq!(result.restored.debit_paisa, result.restored.credit_paisa);
    assert_eq!(
        result.restored.cash_paisa.rupees(),
        50_000 - 5_000 + 12_800 + 10_000 - 4_316
    );
    assert_eq!(fingerprint(&rig), before);
    assert!(!rig.service.status().unwrap().test_restore_due);
    advance(&rig, 31 * 24 * 3600);
    assert!(
        rig.service.status().unwrap().test_restore_due,
        "monthly reminder (NFR-BAK-11)"
    );
    assert!(
        files_under(&rig.backups.join("tmp")).is_empty(),
        "the test copy is thrown away"
    );
}

#[test]
fn the_schedule_runs_every_15_minutes_while_data_changes_and_daily_at_closing_time() {
    let rig = small_world();
    switch_everything_on(&rig);
    // Shop time is UTC+5; the test clock starts at 09:00 UTC = 14:00 in the shop.
    // The app just started: a local copy is due, and so is one for a drive that is already in.
    assert_eq!(
        rig.service.due_jobs().unwrap(),
        vec![Job::Local, Job::UsbPlugged]
    );
    rig.service.tick().unwrap();
    assert!(rig.service.due_jobs().unwrap().is_empty());
    advance(&rig, 16 * 60);
    assert!(
        rig.service.due_jobs().unwrap().is_empty(),
        "nothing changed, so no copy"
    );
    vouchers::save_receipt(
        &rig.w.env.ctx,
        &rig.w.owner,
        &rupees_input(rig.w.akram, "100"),
    )
    .unwrap();
    assert_eq!(rig.service.due_jobs().unwrap(), vec![Job::Local]);
    advance(&rig, 14 * 60 + 30);
    // Still the same moment of day, 14:30 shop time: not yet 21:00.
    rig.service.tick().unwrap();
    assert!(rig.service.due_jobs().unwrap().is_empty());

    // 16:00 UTC is 21:00 in the shop: the daily copy to every place.
    let to_closing = 16 * 3600 - (9 * 3600 + 16 * 60 + 14 * 60 + 30);
    advance(&rig, to_closing);
    assert!(rig.service.due_jobs().unwrap().contains(&Job::Daily));
    let reports = rig.service.tick().unwrap();
    assert_eq!(reports.len(), 1);
    for kind in [BackupKind::Local, BackupKind::Usb, BackupKind::Cloud] {
        assert!(first_ok(&reports[0], kind), "{kind:?}");
    }
    assert!(
        !rig.service.due_jobs().unwrap().contains(&Job::Daily),
        "once a day"
    );
    advance(&rig, 3600);
    assert!(!rig.service.due_jobs().unwrap().contains(&Job::Daily));
    // The next shop day, after closing time, it runs again.
    advance(&rig, 23 * 3600);
    assert!(rig.service.due_jobs().unwrap().contains(&Job::Daily));
}

#[test]
fn a_usb_drive_that_is_plugged_in_gets_a_copy() {
    let rig = small_world();
    switch_everything_on(&rig);
    let parent = rig.usb.parent().unwrap().to_path_buf();
    fs::remove_dir_all(&parent).unwrap();
    rig.service.tick().unwrap();
    fs::create_dir_all(&parent).unwrap();
    advance(&rig, 60);
    assert!(rig.service.due_jobs().unwrap().contains(&Job::UsbPlugged));
}

#[test]
fn the_list_pages_newest_first_and_shows_what_was_checked() {
    let rig = small_world();
    switch_everything_on(&rig);
    for _ in 0..3 {
        advance(&rig, 120);
        rig.service.run(Scope::ALL).unwrap();
    }
    let all = rig
        .service
        .list(&BackupListInput {
            kind: None,
            offset: 0,
            limit: 0,
        })
        .unwrap();
    // Three local, three USB, and one cloud copy: the cloud keeps the newest copy of each day.
    assert_eq!(all.total_count, 7);
    assert!(all.cloud_reachable);
    assert!(
        all.rows
            .windows(2)
            .all(|w| w[0].created_at >= w[1].created_at)
    );
    assert!(all.rows.iter().all(|r| r.status == RowStatus::Good));
    assert!(
        all.rows
            .iter()
            .all(|r| r.encrypted == (r.kind != BackupKind::Local))
    );
    let page = rig
        .service
        .list(&BackupListInput {
            kind: None,
            offset: 4,
            limit: 3,
        })
        .unwrap();
    assert_eq!((page.rows.len(), page.total_count), (3, 7));
    assert_eq!(page.rows[0], all.rows[4]);

    rig.cloud.set_offline(true);
    let offline = rig
        .service
        .list(&BackupListInput {
            kind: None,
            offset: 0,
            limit: 100,
        })
        .unwrap();
    assert!(!offline.cloud_reachable);
    assert_eq!(offline.total_count, 6, "local and USB still show");
}

#[test]
fn every_run_writes_backup_log_rows_and_the_snapshot_is_fast_and_consistent_under_writes() {
    let rig = small_world();
    switch_everything_on(&rig);
    add_ballast(&rig, 40_000_000);
    // A writer keeps saving while the copy is taken.
    let db = Arc::clone(rig.w.env.ctx.db());
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_in = Arc::clone(&stop);
    let writer = std::thread::spawn(move || {
        let mut n = 0_i64;
        while !stop_in.load(std::sync::atomic::Ordering::SeqCst) && n < 200 {
            db.write(|tx| {
                tx.execute(
                    "INSERT INTO user_prefs (user_id, key, value_json) VALUES (1, ?1, '1')",
                    [format!("k{n}")],
                )?;
                Ok::<(), mandi_services::db::DbError>(())
            })
            .ok();
            n += 1;
        }
    });
    let started = Instant::now();
    let report = rig.service.run(Scope::ALL).unwrap();
    stop.store(true, std::sync::atomic::Ordering::SeqCst);
    writer.join().unwrap();
    assert!(report.results.iter().all(|r| r.ok), "{report:?}");
    assert!(
        report.snapshot_ms < 5_000,
        "NFR-SPD-15: the snapshot of a big file took {} ms",
        report.snapshot_ms
    );
    eprintln!(
        "snapshot {} ms, check {} ms, whole run {:?}",
        report.snapshot_ms,
        report.check_ms,
        started.elapsed()
    );
    let rows: Vec<(String, i64, i64)> = raw(&rig)
        .prepare("SELECT kind, integrity_ok, size_bytes FROM backup_log ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let kinds: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(kinds, ["local", "usb", "cloud"]);
    assert!(rows.iter().all(|r| r.1 == 1 && r.2 > 0));
    // The copy is a whole database: it passes the integrity check and balances.
    let name = names(&rig, BackupKind::Local).remove(0);
    let inspected = mandi_backup::snapshot::inspect(&rig.backups.join("local").join(name)).unwrap();
    assert_eq!(inspected.totals.debit_paisa, inspected.totals.credit_paisa);
}

#[test]
fn a_run_with_nothing_switched_on_but_local_makes_one_local_copy_and_asks_nobody() {
    let rig = small_world();
    let report = rig.service.run(Scope::ALL).unwrap();
    assert_eq!(report.results.len(), 1);
    assert!(first_ok(&report, BackupKind::Local));
    assert!(!rig.service.status().unwrap().cloud_ready);
}

/// A pack source that hands over one PDF, like the report engine will (plan 20, R11).
struct OnePdf;

impl mandi_backup::daily_pack::PackSource for OnePdf {
    fn build(
        &self,
        day: &str,
        out_dir: &Path,
    ) -> Result<Vec<mandi_backup::daily_pack::PackFile>, mandi_backup::BackupError> {
        let path = out_dir.join(format!("journal-{day}.pdf"));
        fs::write(&path, b"%PDF-1.7 journal").unwrap();
        Ok(vec![mandi_backup::daily_pack::PackFile {
            part: mandi_backup::daily_pack::PackPart::JournalPdf,
            name: format!("journal-{day}.pdf"),
            path,
        }])
    }

    fn pending(&self) -> bool {
        false
    }
}

#[test]
fn the_readable_pack_goes_to_the_cloud_encrypted_once_a_day() {
    use std::io::Read;
    let rig = small_world();
    let printed = switch_everything_on(&rig);
    let service = Arc::new(mandi_backup::service::BackupService::new(
        Arc::clone(rig.w.env.ctx.db()),
        mandi_backup::service::BackupConfig {
            backups_dir: rig.backups.clone(),
            app_version: "t".into(),
        },
        mandi_backup::service::BackupParts {
            secrets: rig.secrets.clone(),
            cloud: Arc::new(mandi_backup::cloud::FixedCloud(Arc::new(rig.cloud.clone()))),
            disk: rig.disk.clone(),
            hook: rig.fault.clone(),
            pack: Arc::new(OnePdf),
        },
    ));
    assert!(!service.status().unwrap().pack_pending);
    service.run(Scope::ALL).unwrap();
    advance(&rig, 60);
    service.run(Scope::ALL).unwrap();
    let packed: Vec<_> = files_under(&rig.root.path().join("cloud").join("pack"));
    assert_eq!(packed.len(), 1, "one pack for the day: {packed:?}");
    // It opens with any age tool and the printed key, so the books can be read without the app.
    let key = mandi_backup::key::BackupKey::parse(&printed).unwrap();
    let decryptor =
        age::Decryptor::new(std::io::BufReader::new(fs::File::open(&packed[0]).unwrap())).unwrap();
    let identity = age::scrypt::Identity::new(age::secrecy::SecretString::from(
        key.display().replace('-', ""),
    ));
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .unwrap();
    let mut text = Vec::new();
    reader.read_to_end(&mut text).unwrap();
    assert_eq!(text, b"%PDF-1.7 journal");
}
