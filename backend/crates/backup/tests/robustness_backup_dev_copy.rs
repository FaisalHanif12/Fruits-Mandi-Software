//! Robustness of backup and restore on a COPY of the developer's own database (PRD 20, FR-20.1 to
//! FR-20.11, NFR-BAK-1 to NFR-BAK-15, NFR-SEC-6, NFR-REL-4, E2E-18).
//!
//! It runs only when `MANDI_BACKUP_DEV_DB` names a COPY of `mandi.db` (never the file the app has
//! open) and `MANDI_BACKUP_DEV_KEYCHAIN=1` allows the one Keychain read below. A plain `cargo test`
//! does nothing. The hash chains of that file were written with the
//! chain key of the computer that made it, which is in the Keychain, so the test reads that one
//! entry (`pk.mandisoftware.app`, `audit-chain-key`) read-only, keeps it in memory and never prints
//! or writes it. Nothing else of the Keychain is touched, and a missing entry is never created.
//!
//! What it does, with the app's own functions and no cloud (a cloud upload needs real keys):
//!
//! 1. Opens the copy as computer A. Both hash chains verify with the real key before anything.
//! 2. Sets the USB folder and the backup key, runs a local and a USB (encrypted) backup, lists them,
//!    and checks that the USB file holds no plain text of the books and no key.
//! 3. Restores the USB file on a NEW computer (a fresh data folder, an empty key store, the key
//!    typed from the printed sheet), refusing a wrong key and a damaged file first, and compares.
//! 4. Restores the local copy into a fresh data folder of the same computer and compares.
//! 5. Moves A on (a receipt), refuses a wrong key, a damaged copy and an unconfirmed restore with
//!    nothing changed, restores the USB copy over A, and checks the safety copy taken before it.
//! 6. Runs the test restore of NFR-BAK-11.
//!
//! "Compares" means: the row count of every table, the trial balance, every account balance (so
//! every party, the cash and the partner accounts), the partner share rows, the journal and audit
//! hash chains row by row (and that both verify with the key), the settings and the learned words.
//! The result of every check is printed as a table (`--nocapture`).
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod robustness_backup_support;

use robustness_backup_support::{Report, contains, differences, fingerprint, flip_a_byte, open_ro};

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mandi_backup::api::{
    BackupListInput, BackupRef, BackupSettingsInput, FirstRunPreviewInput, FirstRunRestoreInput,
    RestoreInput, RestorePreviewInput, RestoreProgress, RestoreSource, RowStatus, TestRestoreInput,
};
use mandi_backup::cloud::{FixedCloud, LocalFolderCloud};
use mandi_backup::daily_pack::PendingPack;
use mandi_backup::error::{BackupCode, BackupError};
use mandi_backup::io::NoHook;
use mandi_backup::key::BackupKey;
use mandi_backup::model::BackupKind;
use mandi_backup::run::Scope;
use mandi_backup::secrets::MemorySecretStore;
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_backup::settings::BackupSettings;
use mandi_services::auth::Actor;
use mandi_services::core::vouchers::ReceiptInput;
use mandi_services::db::audit::{AuditKey, MemoryKeyStore};
use mandi_services::testing::TestEnv;
use mandi_services::vouchers;

// ---- computers ----------------------------------------------------------------------------------

struct Computer {
    root: tempfile::TempDir,
    env: TestEnv,
    service: Arc<BackupService>,
    path: PathBuf,
}

fn service_for(root: &Path, env: &TestEnv) -> Arc<BackupService> {
    Arc::new(BackupService::new(
        Arc::clone(env.ctx.db()),
        BackupConfig {
            backups_dir: root.join("backups"),
            app_version: "0.0.0-robustness".to_owned(),
        },
        BackupParts {
            secrets: Arc::new(MemorySecretStore::default()),
            cloud: Arc::new(FixedCloud(Arc::new(LocalFolderCloud::new(
                root.join("cloud"),
            )))),
            disk: env.disk.clone(),
            hook: Arc::new(NoHook),
            pack: Arc::new(PendingPack),
        },
    ))
}

/// A computer whose data file is a COPY of `source` (or a fresh empty one), whose key store holds
/// `chain_key` (or the test key of a new computer when it is `None`).
fn computer(source: Option<&Path>, chain_key: Option<&AuditKey>, day: (i32, u32, u32)) -> Computer {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("data").join("mandi.db");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    if let Some(source) = source {
        fs::copy(source, &path).expect("copy the database");
    }
    let env = match chain_key {
        Some(key) => {
            TestEnv::with_key_store(root.path(), Arc::new(MemoryKeyStore::new(key.clone())))
        }
        None => TestEnv::new(root.path()),
    }
    .expect("open the data file");
    common::set_today(&env, day.0, day.1, day.2);
    let service = service_for(root.path(), &env);
    Computer {
        root,
        env,
        service,
        path,
    }
}

/// The one Keychain entry the real app wrote its chain key to. Read only, never created. macOS may
/// ask the user to allow it; if nobody answers within two minutes the test stops with a clear
/// message instead of waiting for ever.
fn real_chain_key() -> Option<AuditKey> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let key = keyring::Entry::new(&mandi_core::profile::keyring_service(), "audit-chain-key")
            .ok()
            .and_then(|entry| entry.get_secret().ok())
            .and_then(|bytes| <[u8; 32]>::try_from(bytes.as_slice()).ok())
            .map(AuditKey::from_bytes);
        let _ = tx.send(key);
    });
    match rx.recv_timeout(std::time::Duration::from_secs(120)) {
        Ok(key) => key,
        Err(_) => panic!(
            "no answer from the Keychain in two minutes: a macOS prompt for the entry \
             pk.mandisoftware.app / audit-chain-key is probably waiting for a click (choose Allow)"
        ),
    }
}

fn code_of(result: Result<impl std::fmt::Debug, BackupError>) -> String {
    match result {
        Ok(v) => format!("accepted: {v:?}").chars().take(80).collect(),
        Err(e) => format!("{:?}", e.code),
    }
}

#[test]
fn backup_and_restore_on_a_copy_of_the_dev_database() {
    let Ok(dev_db) = std::env::var("MANDI_BACKUP_DEV_DB") else {
        eprintln!("skipped: MANDI_BACKUP_DEV_DB (a COPY of mandi.db) is not set");
        return;
    };
    if std::env::var("MANDI_BACKUP_DEV_KEYCHAIN").as_deref() != Ok("1") {
        eprintln!(
            "skipped: MANDI_BACKUP_DEV_KEYCHAIN=1 is needed to read the chain key of the dev \
             database from the Keychain (read only, macOS may ask to allow it)"
        );
        return;
    }
    let dev_db = PathBuf::from(dev_db);
    let mut rep = Report::default();
    let chain_key = real_chain_key().expect(
        "the chain key of the dev database is not in the Keychain (service pk.mandisoftware.app, \
         entry audit-chain-key): allow the access, or the chain checks cannot run",
    );

    // ---- 1. computer A: a copy of the dev database ----
    let a = computer(Some(&dev_db), Some(&chain_key), (2026, 11, 20));
    let fp_start = fingerprint(&a.path, Some(&chain_key));
    let tables = fp_start.counts.len();
    let rows: i64 = fp_start.counts.values().sum();
    rep.check(
        "A0 the copy opens and both hash chains verify with the app's real key",
        fp_start.chains_ok == Some(true) && fp_start.debit == fp_start.credit,
        format!(
            "{tables} tables, {rows} rows, {} journal entries, {} audit rows, trial balance {}/{}",
            fp_start.journal_chain.len(),
            fp_start.audit_chain.len(),
            fp_start.debit,
            fp_start.credit
        ),
    );
    assert_eq!(
        fp_start.chains_ok,
        Some(true),
        "the copy's chains must verify before the test starts"
    );

    let usb_dir = a.root.path().join("usb").join("MandiBackup");
    fs::create_dir_all(a.root.path().join("usb")).unwrap();
    let settings = BackupSettings {
        usb_enabled: true,
        usb_folder: usb_dir.display().to_string(),
        ..BackupSettings::default()
    };
    a.service
        .settings_set(
            Some(1),
            &BackupSettingsInput {
                settings: settings.clone(),
                cloud_access_key_id: None,
                cloud_secret_access_key: None,
            },
        )
        .expect("save the backup settings");
    let printed = a.service.key_setup(Some(1)).expect("make the key").key;
    a.service
        .key_confirm(Some(1), &printed)
        .expect("confirm the key");
    let fp_before = fingerprint(&a.path, Some(&chain_key));

    // ---- 2. local and USB (encrypted) backup ----
    let report = a
        .service
        .run(Scope {
            local: true,
            usb: true,
            cloud: false,
        })
        .expect("the backup run");
    let ok_of = |kind: BackupKind| {
        report
            .results
            .iter()
            .any(|r| r.kind == kind && r.ok && !r.queued && r.file.is_some())
    };
    rep.check(
        "B1 a local and a USB backup are made, checked and kept (NFR-BAK-4, NFR-BAK-5)",
        ok_of(BackupKind::Local) && ok_of(BackupKind::Usb),
        format!(
            "snapshot {} ms, check {} ms, results {:?}",
            report.snapshot_ms,
            report.check_ms,
            report
                .results
                .iter()
                .map(|r| (r.kind, r.ok, r.size_bytes))
                .collect::<Vec<_>>()
        ),
    );
    let list = a
        .service
        .list(&BackupListInput {
            kind: None,
            offset: 0,
            limit: 50,
        })
        .unwrap();
    let local_row = list
        .rows
        .iter()
        .find(|r| r.kind == BackupKind::Local)
        .expect("a local copy")
        .clone();
    let usb_row = list
        .rows
        .iter()
        .find(|r| r.kind == BackupKind::Usb)
        .expect("a USB copy")
        .clone();
    rep.check(
        "B2 the list shows both copies as good, the USB one encrypted, the local one plain",
        local_row.status == RowStatus::Good
            && usb_row.status == RowStatus::Good
            && usb_row.encrypted
            && !local_row.encrypted,
        format!(
            "local {} ({} bytes), USB {} ({} bytes)",
            local_row.name, local_row.size_bytes, usb_row.name, usb_row.size_bytes
        ),
    );
    let local_path = a.service.local_dir(&settings).join(&local_row.name);
    let usb_path = usb_dir.join(&usb_row.name);
    let usb_bytes = fs::read(&usb_path).unwrap();
    let printed_plain = printed.replace('-', "");
    let chain_hex = chain_key.to_hex();
    let plain = [
        b"SQLite format 3".as_slice(),
        "Haroon".as_bytes(),
        "ہارون".as_bytes(),
        b"journal_lines",
        printed_plain.as_bytes(),
        chain_hex.as_bytes(),
    ];
    let leaks: Vec<usize> = plain
        .iter()
        .enumerate()
        .filter(|(_, p)| contains(&usb_bytes, p))
        .map(|(i, _)| i)
        .collect();
    rep.check(
        "B3 the USB file is encrypted: no SQLite header, no account name, no table name, no key in it (NFR-BAK-6, NFR-SEC-7)",
        leaks.is_empty() && !contains(&fs::read(usb_dir.join(mandi_backup::names::manifest_name(&usb_row.name))).unwrap(), chain_key.to_hex().as_bytes()),
        format!("{} bytes, plain markers found: {leaks:?}", usb_bytes.len()),
    );

    // ---- 3. computer B: a NEW computer, the USB file and the printed key ----
    let b = computer(None, None, (2026, 11, 21));
    let fp_b_fresh = fingerprint(&b.path, None);
    let source = RestoreSource::File(usb_path.display().to_string());
    let quiet = |_: RestoreProgress| {};
    let wrong = BackupKey::generate().unwrap().display();
    let wrong_key = b.service.first_run_preview(
        &FirstRunPreviewInput {
            source: source.clone(),
            copy: None,
            key: wrong.clone(),
        },
        &quiet,
    );
    let damaged_dir = b.root.path().join("damaged");
    fs::create_dir_all(&damaged_dir).unwrap();
    let damaged = damaged_dir.join("mandi-20261121T090000000Z.mbk");
    flip_a_byte(&usb_path, &damaged);
    let damaged_try = b.service.first_run_preview(
        &FirstRunPreviewInput {
            source: RestoreSource::File(damaged.display().to_string()),
            copy: None,
            key: printed.clone(),
        },
        &quiet,
    );
    let (wk, dm) = (
        wrong_key.as_ref().err().map(|e| e.code),
        damaged_try.as_ref().err().map(|e| e.code),
    );
    rep.check(
        "C1 a wrong key is refused on a new computer and nothing is changed (NFR-BAK-6, NFR-REL-4)",
        wk == Some(BackupCode::ErrBackupKeyWrong) && fingerprint(&b.path, None) == fp_b_fresh,
        format!("{wk:?}, the fresh data file is byte-for-row the same"),
    );
    rep.check(
        "C2 a damaged backup file is refused on a new computer and nothing is changed (NFR-BAK-5)",
        dm.is_some() && fingerprint(&b.path, None) == fp_b_fresh,
        format!("one byte flipped in the middle: {dm:?}"),
    );
    let preview = b
        .service
        .first_run_preview(
            &FirstRunPreviewInput {
                source: source.clone(),
                copy: None,
                key: printed.clone(),
            },
            &quiet,
        )
        .expect("preview the copy with the printed key");
    rep.check(
        "C3 the preview shows the copy's date and totals, and every check passes (FR-20.11)",
        preview.checks.iter().all(|c| c.ok)
            && preview.totals.debit_paisa == preview.totals.credit_paisa
            && preview.totals.journal_entries == i64::try_from(fp_before.journal_chain.len()).unwrap()
            && !preview.needs_upgrade,
        format!(
            "{} entries, accounts {}, audit rows {}, newest entry {:?}, debit = credit = {:?}, schema {} of {}",
            preview.totals.journal_entries,
            preview.totals.accounts,
            preview.totals.audit_rows,
            preview.totals.last_entry_date,
            preview.totals.debit_paisa,
            preview.schema_version,
            preview.app_schema_version
        ),
    );
    let done = b
        .service
        .first_run_restore(
            &FirstRunRestoreInput {
                source,
                copy: usb_row.name.clone(),
                key: printed.clone(),
            },
            &quiet,
        )
        .expect("restore on the new computer");
    let fp_b = fingerprint(&b.path, Some(&chain_key));
    let extra_b = fp_b
        .audit_chain
        .len()
        .saturating_sub(fp_before.audit_chain.len());
    let diff_b = differences(&fp_before, &fp_b, extra_b, &["assistant_enabled"]);
    rep.check(
        "C4 restore on a new computer: every table's rows, trial balance, every account balance, the partner rows, both hash chains and the settings equal the copy (E2E-18, NFR-REL-4)",
        diff_b.is_empty() && extra_b <= 1 && fp_b.chains_ok == Some(true),
        format!(
            "{tables} tables compared, {} accounts, {} journal hashes, {} old audit hashes, {} audit row added by the restore, chains verify: {:?}, differences: {diff_b:?}",
            fp_b.balances.len(),
            fp_b.journal_chain.len(),
            fp_before.audit_chain.len(),
            extra_b,
            fp_b.chains_ok
        ),
    );
    let new_key = b.service.db().auditor().key().unwrap();
    rep.check(
        "C5 the chain key came back with the copy, so new rows continue the same chains (NFR-SEC-6)",
        done.chain_key_restored && new_key.same_as(&chain_key),
        format!(
            "chain_key_restored {}, cloud_needs_keys {}, assistant_was_on {}",
            done.chain_key_restored, done.cloud_needs_keys, done.assistant_was_on
        ),
    );

    // ---- 4. computer C: the local copy into a fresh data folder of the same computer ----
    let c = computer(None, Some(&chain_key), (2026, 11, 21));
    let c_settings = c.service.settings().unwrap();
    let c_local = c.service.local_dir(&c_settings);
    fs::create_dir_all(&c_local).unwrap();
    fs::copy(&local_path, c_local.join(&local_row.name)).unwrap();
    fs::copy(
        a.service
            .local_dir(&settings)
            .join(mandi_backup::names::manifest_name(&local_row.name)),
        c_local.join(mandi_backup::names::manifest_name(&local_row.name)),
    )
    .unwrap();
    let c_ref = BackupRef {
        kind: BackupKind::Local,
        name: local_row.name.clone(),
    };
    let c_preview = c
        .service
        .restore_preview(&RestorePreviewInput {
            backup: c_ref.clone(),
            key: None,
        })
        .expect("preview the local copy");
    let c_done = c
        .service
        .restore(
            None,
            &RestoreInput {
                backup: c_ref,
                key: None,
                confirmed: true,
            },
        )
        .expect("restore the local copy into the fresh data folder");
    let fp_c = fingerprint(&c.path, Some(&chain_key));
    let extra_c = fp_c
        .audit_chain
        .len()
        .saturating_sub(fp_before.audit_chain.len());
    let diff_c = differences(&fp_before, &fp_c, extra_c, &[]);
    rep.check(
        "D1 the local copy restored into a fresh data folder equals the copy: all tables, balances, chains, settings",
        diff_c.is_empty() && extra_c <= 1 && fp_c.chains_ok == Some(true) && c_preview.totals.debit_paisa == c_preview.totals.credit_paisa,
        format!(
            "{} entries restored, {} audit row added, safety copy {}, differences: {diff_c:?}",
            c_done.totals.journal_entries, extra_c, c_done.safety_copy
        ),
    );

    // ---- 5. computer A moves on, then is restored over from the USB copy ----
    common::set_today(&a.env, 2026, 11, 21);
    let mani: i64 = open_ro(&a.path)
        .query_row("SELECT id FROM accounts WHERE code = '0020'", [], |r| {
            r.get(0)
        })
        .expect("the buyer 0020 of the dev copy");
    let owner = Actor {
        user_id: 1,
        username: "owner".to_owned(),
        display_name: "Owner".to_owned(),
    };
    vouchers::save_receipt(
        &a.env.ctx,
        &owner,
        &ReceiptInput {
            date: "21-11-2026".into(),
            account_id: mani,
            amount: "1,000".into(),
            removed: Default::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .expect("a receipt after the backup");
    let fp_mutated = fingerprint(&a.path, Some(&chain_key));
    rep.check(
        "E0 the books move on after the backup and still verify (the receipt is after the copy)",
        fp_mutated != fp_before && fp_mutated.chains_ok == Some(true),
        format!(
            "journal entries {} -> {}",
            fp_before.journal_chain.len(),
            fp_mutated.journal_chain.len()
        ),
    );
    let usb_ref = BackupRef {
        kind: BackupKind::Usb,
        name: usb_row.name.clone(),
    };
    let safety_files = |service: &BackupService| -> usize {
        fs::read_dir(service.safety_dir()).map_or(0, |d| d.flatten().count())
    };
    let safety_before = safety_files(&a.service);
    let wrong_preview = a.service.restore_preview(&RestorePreviewInput {
        backup: usb_ref.clone(),
        key: Some(wrong.clone()),
    });
    let wrong_restore = a.service.restore(
        Some(1),
        &RestoreInput {
            backup: usb_ref.clone(),
            key: Some(wrong.clone()),
            confirmed: true,
        },
    );
    rep.check(
        "E1 a wrong key is refused by the preview and by the restore, the books and the safety folder are unchanged (NFR-BAK-6)",
        code_of(wrong_preview) == "ErrBackupKeyWrong"
            && code_of(wrong_restore) == "ErrBackupKeyWrong"
            && fingerprint(&a.path, Some(&chain_key)) == fp_mutated
            && safety_files(&a.service) == safety_before,
        "ERR_BACKUP_KEY_WRONG twice; every table, hash and setting the same as before the try",
    );
    let bad_name = "mandi-20261121T100000000Z.mbk";
    flip_a_byte(&usb_path, &usb_dir.join(bad_name));
    let bad_ref = BackupRef {
        kind: BackupKind::Usb,
        name: bad_name.to_owned(),
    };
    let bad_restore = code_of(a.service.restore(
        Some(1),
        &RestoreInput {
            backup: bad_ref,
            key: None,
            confirmed: true,
        },
    ));
    rep.check(
        "E2 a damaged USB copy is refused, the books and the safety folder are unchanged (NFR-BAK-5)",
        bad_restore.starts_with("ErrBackup") && fingerprint(&a.path, Some(&chain_key)) == fp_mutated && safety_files(&a.service) == safety_before,
        format!("one byte flipped: {bad_restore}"),
    );
    let local_plain_bad = a
        .service
        .local_dir(&settings)
        .join("mandi-local-20261121T100000000Z.db");
    flip_a_byte(&local_path, &local_plain_bad);
    // The manifest of the damaged copy is the one of the good copy: its checksum no longer fits.
    fs::copy(
        a.service
            .local_dir(&settings)
            .join(mandi_backup::names::manifest_name(&local_row.name)),
        a.service
            .local_dir(&settings)
            .join("mandi-local-20261121T100000000Z.json"),
    )
    .unwrap();
    let bad_local = code_of(a.service.restore(
        Some(1),
        &RestoreInput {
            backup: BackupRef {
                kind: BackupKind::Local,
                name: "mandi-local-20261121T100000000Z.db".to_owned(),
            },
            key: None,
            confirmed: true,
        },
    ));
    rep.check(
        "E3 a damaged local copy is refused, the books are unchanged",
        bad_local.starts_with("Err")
            && !bad_local.starts_with("accepted")
            && fingerprint(&a.path, Some(&chain_key)) == fp_mutated,
        format!("one byte flipped in the middle of the file: {bad_local}"),
    );
    let unconfirmed = code_of(a.service.restore(
        Some(1),
        &RestoreInput {
            backup: usb_ref.clone(),
            key: None,
            confirmed: false,
        },
    ));
    rep.check(
        "E4 a restore the Owner did not confirm is refused and changes nothing (FR-20.9)",
        unconfirmed == "ErrBackupNotConfirmed"
            && fingerprint(&a.path, Some(&chain_key)) == fp_mutated,
        unconfirmed,
    );
    let preview_a = a
        .service
        .restore_preview(&RestorePreviewInput {
            backup: usb_ref.clone(),
            key: None,
        })
        .expect("preview over the books");
    rep.check(
        "E5 the preview says what would be lost: the receipt made after the copy (FR-20.9)",
        preview_a.lost.journal_entries == 1 && preview_a.lost.vouchers == 1,
        format!(
            "lost {:?}; copy {} entries, live {} entries",
            preview_a.lost, preview_a.totals.journal_entries, preview_a.current.journal_entries
        ),
    );
    let restored = a
        .service
        .restore(
            Some(1),
            &RestoreInput {
                backup: usb_ref,
                key: None,
                confirmed: true,
            },
        )
        .expect("restore the USB copy over the books");
    let safety_path = a.service.safety_dir().join(&restored.safety_copy);
    let safety_ok = safety_path.exists() && {
        let fp_safety = fingerprint(&safety_path, Some(&chain_key));
        fp_safety.counts == fp_mutated.counts
            && fp_safety.balances == fp_mutated.balances
            && fp_safety.journal_chain == fp_mutated.journal_chain
            && fp_safety.chains_ok == Some(true)
    };
    rep.check(
        "E6 a safety copy of the books as they were is taken before the restore and holds them (FR-20.9)",
        safety_ok,
        format!(
            "{} ({} bytes): the books with the receipt, chains verify",
            restored.safety_copy,
            fs::metadata(&safety_path).map_or(0, |m| m.len())
        ),
    );
    let fp_a = fingerprint(&a.path, Some(&chain_key));
    let extra_a = fp_a
        .audit_chain
        .len()
        .saturating_sub(fp_before.audit_chain.len());
    let diff_a = differences(&fp_before, &fp_a, extra_a, &[]);
    rep.check(
        "E7 after the restore over the same computer every table, balance, chain and setting equals the copy, the receipt is gone (E2E-18)",
        diff_a.is_empty() && extra_a == 1 && fp_a.chains_ok == Some(true),
        format!(
            "{} audit row added, trial balance {}/{}, cash 01010001 {} paisa, chains verify {:?}, differences: {diff_a:?}",
            extra_a,
            fp_a.debit,
            fp_a.credit,
            fp_a.balances.get("01010001").copied().unwrap_or(0),
            fp_a.chains_ok
        ),
    );
    a.env.ctx.invalidate_caches();
    vouchers::save_receipt(
        &a.env.ctx,
        &owner,
        &ReceiptInput {
            date: "21-11-2026".into(),
            account_id: mani,
            amount: "500".into(),
            removed: Default::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .expect("the app works on the restored books");
    rep.check(
        "E8 the app keeps working on the restored books and the chains still verify",
        fingerprint(&a.path, Some(&chain_key)).chains_ok == Some(true),
        "a new receipt of 500 was posted and both chains verify with the same key",
    );

    // ---- 6. the monthly test restore ----
    // The planted damaged copies are still in the folders (the newest by name). The test restore of
    // exactly that copy must say it is damaged, and the live books must not change.
    let live_before_test = fingerprint(&a.path, Some(&chain_key));
    let damaged_test = a
        .service
        .test_restore(&TestRestoreInput {
            backup: Some(BackupRef {
                kind: BackupKind::Usb,
                name: bad_name.to_owned(),
            }),
            key: None,
        })
        .expect("the test restore of a damaged copy answers");
    rep.check(
        "F1 the test restore (NFR-BAK-11) of a damaged copy says so, with the check that failed, and the live books are untouched",
        !damaged_test.ok
            && damaged_test.checks.iter().any(|c| !c.ok)
            && fingerprint(&a.path, Some(&chain_key)) == live_before_test,
        format!("ok {}, checks {:?}", damaged_test.ok, damaged_test.checks),
    );
    let _ = fs::remove_file(usb_dir.join(bad_name));
    let _ = fs::remove_file(
        a.service
            .local_dir(&settings)
            .join("mandi-local-20261121T100000000Z.db"),
    );
    let _ = fs::remove_file(
        a.service
            .local_dir(&settings)
            .join("mandi-local-20261121T100000000Z.json"),
    );
    let test = a
        .service
        .test_restore(&TestRestoreInput {
            backup: None,
            key: None,
        })
        .expect("the test restore");
    rep.check(
        "F2 the test restore of the newest good copy restores it into a separate file, every check passes and the totals match the books (NFR-BAK-11)",
        test.ok
            && test.checks.iter().all(|c| c.ok)
            && test.restored.debit_paisa == test.restored.credit_paisa
            && fingerprint(&a.path, Some(&chain_key)) == live_before_test,
        format!(
            "{:?}: restored {} entries, live {} entries, checks {:?}",
            test.backup, test.restored.journal_entries, test.live.journal_entries, test.checks
        ),
    );

    rep.finish();
}
