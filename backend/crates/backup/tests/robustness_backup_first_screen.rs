//! The laptop is lost. A NEW computer starts with an empty data folder and nothing else
//! (PRD FR-20.11, NFR-INS-2, NFR-INS-6, NFR-REL-4, NFR-BAK-6, NFR-SEC-6, E2E-18).
//!
//! The old shop has the PRD worked day, a learned word, a preference, a question in the question
//! log, closed days, a USB folder and a cloud (the real `object_store` adapter on memory, the
//! stand-in the backup crate's own tests use, because real Cloudflare keys are not available). It
//! makes two copies, a day apart. The new computers then bring the books back from the FIRST screen:
//!
//! * N1: from the USB folder and from one file, with the printed key (wrong key, damaged file and a
//!   full disk first; each refusal changes nothing; then the same screen works).
//! * N2: from the CLOUD, with the address, bucket, region, folder and the two cloud keys typed, the
//!   internet gone half way first; then the list (newest first), the check, the totals before
//!   anything is restored, the restore, the old sign-in, the recovery code, and the next daily copy
//!   going to the same place and restoring on a third computer.
//!
//! "The books equal the source" is `robustness_backup_support::differences`: every table's rows,
//! every account balance (so every party, the cash and the partners), the trial balance, the
//! partner share rows, both hash chains row by row and verified with the key, the settings, the
//! learned words, the days and the question log. The result of every check is printed as a table
//! (`--nocapture`).
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
mod new_computer;
mod robustness_backup_support;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use mandi_backup::api::{
    BackupListInput, BackupRef, FirstRunCheckName, FirstRunPreviewInput, FirstRunRestoreInput,
    RestoreSource, RowStatus,
};
use mandi_backup::cloud::{CloudError, CloudStore};
use mandi_backup::error::BackupCode;
use mandi_backup::key::BackupKey;
use mandi_backup::model::BackupKind;
use mandi_backup::run::Scope;
use mandi_backup::secrets::SecretName;
use mandi_services::assistant::{self, QuestionSpec};
use mandi_services::core::auth::Language;
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::{Secret, auth, day, vouchers};
use new_computer::*;
use robustness_backup_support::{Report, differences, fingerprint, flip_a_byte};

/// The settings rows that may differ on a new computer: the backup row (the cloud the Owner typed,
/// a local folder that is not on this computer) and the assistant switch (it is off until the
/// OpenRouter key is typed again).
const NEW_COMPUTER_SETTINGS: &[&str] = &["backup", "assistant_enabled"];

fn question() -> QuestionSpec {
    QuestionSpec {
        question: "ikram ke total bakaya kitne hai".to_owned(),
        language: Language::Ur,
        conversation_id: "c-lost-laptop".to_owned(),
        parent_id: None,
        source: "model",
        counted: true,
        sources_json: "[]".to_owned(),
        outcome: "answered",
        tool_calls_json: "[]".to_owned(),
        answer_json: Some("{}".to_owned()),
        model: Some("openai/gpt-6-luna".to_owned()),
        model_calls: Some(3),
        tokens_in: Some(100),
        tokens_cached: Some(60),
        tokens_out: Some(20),
        tokens_reasoning: Some(5),
        finish_reason: Some("stop".to_owned()),
        total_millis: None,
        model_millis: None,
        tool_millis: None,
        calls_json: None,
    }
}

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

/// A new computer that has not changed at all: no Owner, the same rows, no chain key, no secret,
/// no scratch file.
fn untouched(new: &New, before: &std::collections::BTreeMap<String, (i64, String)>) -> bool {
    let tmp = new.root.path().join("backups").join("tmp");
    new.is_fresh()
        && &new.fingerprint() == before
        && new.keys.current().is_none()
        && new.secrets.peek(SecretName::BackupKey).is_none()
        && new.secrets.peek(SecretName::CloudAccessKeyId).is_none()
        && new.secrets.peek(SecretName::CloudSecretAccessKey).is_none()
        && std::fs::read_dir(tmp).map_or(0, std::iter::Iterator::count) == 0
}

fn key_of(new: &New) -> mandi_services::db::audit::AuditKey {
    new.keys
        .current()
        .expect("the new computer has a chain key")
}

fn one(path: &std::path::Path, sql: &str) -> String {
    robustness_backup_support::open_ro(path)
        .query_row(sql, [], |r| r.get::<_, String>(0))
        .unwrap_or_else(|e| format!("({e})"))
}

#[test]
fn a_lost_laptop_comes_back_from_the_first_screen_from_a_usb_file_and_from_the_cloud() {
    let mut rep = Report::default();

    // ---- the old shop: worked day, a question, closed days, two copies a day apart ----
    let old = old_computer();
    let env = &old.rig.w.env;
    let owner = &old.rig.w.owner;
    let old_db = old.rig.w.dir.path().join("data").join("mandi.db");
    let old_key = old_chain_key();
    common::set_today(env, 2026, 10, 10);
    assistant::record_question(&env.ctx, owner, &question()).expect("a question in the log");
    day::close(
        &env.ctx,
        owner,
        &DayCloseInput {
            up_to: "09-10-2026".into(),
        },
    )
    .expect("close the days up to 09-10-2026");
    let first = old.back_up();
    vouchers::save_receipt(
        &env.ctx,
        owner,
        &harness::rupees_input(old.rig.w.akram, "1,000"),
    )
    .expect("a receipt after the first copy");
    common::set_today(env, 2026, 10, 11);
    let second = old.back_up();
    let usb_newest = old.latest(BackupKind::Usb);
    let (_keep, plain) = old.unpack_cloud(&second);
    let fp_copy = fingerprint(&plain, Some(&old_key));
    let fp_live = fingerprint(&old_db, Some(&old_key));
    rep.check(
        "S0 the source: two copies a day apart; the newest holds the receipt, the closed days and the question; its own chains verify",
        first != second
            && fp_copy.chains_ok == Some(true)
            && fp_copy.questions.len() == 1
            && !fp_copy.days.is_empty()
            && fp_copy.questions == fp_live.questions
            && fp_copy.days == fp_live.days
            && fp_copy.debit == fp_copy.credit,
        format!(
            "copies {first} then {second}; {} tables, {} journal entries, {} audit rows, trial {}/{}, closed up to {}, {} question row",
            fp_copy.counts.len(),
            fp_copy.journal_chain.len(),
            fp_copy.audit_chain.len(),
            fp_copy.debit,
            fp_copy.credit,
            one(&plain, "SELECT closed_up_to_date FROM day_state"),
            fp_copy.questions.len()
        ),
    );

    // =========================================================================================
    // N1: from a USB folder and a file, with the printed key
    // =========================================================================================
    let n1 = new_computer(old.cloud.clone());
    let fresh_n1 = n1.fingerprint();
    rep.check(
        "N1-0 the first screen: a new data folder has no Owner, no chain key and no key anywhere (NFR-INS-2)",
        n1.is_fresh() && n1.keys.current().is_none() && n1.secrets.peek(SecretName::BackupKey).is_none(),
        "first_run true, no Owner, the key store and the credential store are empty",
    );
    let folder = RestoreSource::Folder(old.rig.usb.display().to_string());
    let list = n1
        .service
        .first_run_list(&list_input(folder.clone()))
        .unwrap();
    let newest_first = list
        .rows
        .windows(2)
        .all(|w| w[0].created_at >= w[1].created_at);
    rep.check(
        "N1-1 the USB folder lists both copies, newest first, good and encrypted (FR-20.11)",
        list.total_count == 2
            && newest_first
            && list.rows.first().map(|r| r.name.as_str()) == Some(usb_newest.as_str())
            && list
                .rows
                .iter()
                .all(|r| r.status == RowStatus::Good && r.encrypted),
        format!(
            "{:?}",
            list.rows
                .iter()
                .map(|r| (&r.name, &r.created_at))
                .collect::<Vec<_>>()
        ),
    );
    let file = old.rig.usb.join(&usb_newest);
    let file_source = RestoreSource::File(file.display().to_string());
    let one_file = n1
        .service
        .first_run_list(&list_input(file_source.clone()))
        .unwrap();
    rep.check(
        "N1-2 one picked .mbk file lists as one copy",
        one_file.total_count == 1
            && one_file.rows.first().map(|r| r.name.as_str()) == Some(usb_newest.as_str()),
        format!(
            "{:?}",
            one_file.rows.iter().map(|r| &r.name).collect::<Vec<_>>()
        ),
    );

    // refusals that change nothing
    let wrong = BackupKey::generate().unwrap().display();
    let wrong_key = n1
        .service
        .first_run_restore(
            &restore_input(file_source.clone(), &usb_newest, &wrong),
            &|_| {},
        )
        .unwrap_err();
    rep.check(
        "N1-3 a wrong key is refused, the first screen stays as it was (NFR-BAK-6)",
        wrong_key.code == BackupCode::ErrBackupKeyWrong && untouched(&n1, &fresh_n1),
        format!(
            "{:?}; no Owner, same rows, no key, no secret, no scratch file",
            wrong_key.code
        ),
    );
    let damaged_dir = n1.root.path().join("damaged");
    std::fs::create_dir_all(&damaged_dir).unwrap();
    let damaged = damaged_dir.join(&usb_newest);
    flip_a_byte(&file, &damaged);
    let bad = n1
        .service
        .first_run_restore(
            &restore_input(
                RestoreSource::File(damaged.display().to_string()),
                &usb_newest,
                &old.printed_key,
            ),
            &|_| {},
        )
        .unwrap_err();
    rep.check(
        "N1-4 a damaged copy file is refused with the right key, the first screen stays as it was (NFR-BAK-5)",
        bad.code != BackupCode::ErrInternal && untouched(&n1, &fresh_n1),
        format!("{:?}", bad.code),
    );
    n1.service_disk.set(1024);
    let full = n1
        .service
        .first_run_restore(
            &restore_input(file_source.clone(), &usb_newest, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    n1.service_disk.set(10 * 1024 * 1024 * 1024);
    rep.check(
        "N1-5 a full disk is refused before anything is unpacked, the first screen stays as it was (NFR-REL-10)",
        full.code == BackupCode::ErrDiskFull && untouched(&n1, &fresh_n1),
        format!("{:?}", full.code),
    );

    // the preview, then the restore
    let typed = old.printed_key.to_lowercase().replace('-', " ");
    let preview = n1
        .service
        .first_run_preview(&preview_input(file_source.clone(), None, &typed), &|_| {})
        .unwrap();
    let names: Vec<FirstRunCheckName> = preview.checks.iter().map(|c| c.name).collect();
    rep.check(
        "N1-6 the preview shows the date, the totals and six passed checks before anything is restored, and changes nothing",
        preview.checks.iter().all(|c| c.ok)
            && names
                == [
                    FirstRunCheckName::Opens,
                    FirstRunCheckName::Checksum,
                    FirstRunCheckName::Integrity,
                    FirstRunCheckName::Counts,
                    FirstRunCheckName::Balanced,
                    FirstRunCheckName::Chains,
                ]
            && preview.totals.debit_paisa == preview.totals.credit_paisa
            && preview.totals.journal_entries == i64::try_from(fp_copy.journal_chain.len()).unwrap()
            && preview.totals.audit_rows == i64::try_from(fp_copy.audit_chain.len()).unwrap()
            && !preview.needs_upgrade
            && untouched(&n1, &fresh_n1),
        format!(
            "made {}, {} accounts, {} entries, {} audit rows, newest entry {:?}, debit = credit = {:?}",
            preview.created_at,
            preview.totals.accounts,
            preview.totals.journal_entries,
            preview.totals.audit_rows,
            preview.totals.last_entry_date,
            preview.totals.debit_paisa
        ),
    );
    let done = n1
        .service
        .first_run_restore(
            &restore_input(file_source.clone(), &usb_newest, &typed),
            &|_| {},
        )
        .expect("restore from the file");
    let fp_n1 = fingerprint(&n1.db_path(), Some(&key_of(&n1)));
    let extra = fp_n1
        .audit_chain
        .len()
        .saturating_sub(fp_copy.audit_chain.len());
    let diff = differences(&fp_copy, &fp_n1, extra, NEW_COMPUTER_SETTINGS);
    rep.check(
        "N1-7 restore from a file: every table, balance, the trial balance, partner rows, both chains, settings, learned words, days and the question log equal the copy (E2E-18)",
        diff.is_empty() && extra == 1 && fp_n1.chains_ok == Some(true),
        format!(
            "{} tables, {} accounts, {} journal hashes, {} audit hashes + 1 restore row, chains verify {:?}, differences {diff:?}",
            fp_n1.counts.len(),
            fp_n1.balances.len(),
            fp_n1.journal_chain.len(),
            fp_copy.audit_chain.len(),
            fp_n1.chains_ok
        ),
    );
    let n1_status = n1.service.status().unwrap();
    rep.check(
        "N1-8 the chain key travelled in the copy; the backup key is kept; a file restore does not have the cloud keys, so it says to ask for them (FR-20.11)",
        done.chain_key_restored
            && key_of(&n1).same_as(&old_key)
            && n1.secrets.peek(SecretName::BackupKey).as_deref() == Some(old.printed_key.replace('-', "").as_str())
            && done.cloud_needs_keys
            && n1.secrets.peek(SecretName::CloudAccessKeyId).is_none()
            && n1.secrets.peek(SecretName::CloudSecretAccessKey).is_none()
            && n1_status.key_set
            && n1_status.key_confirmed,
        format!(
            "chain_key_restored {}, cloud_needs_keys {}, no cloud key on this computer, key set {} and confirmed {}",
            done.chain_key_restored, done.cloud_needs_keys, n1_status.key_set, n1_status.key_confirmed
        ),
    );
    let session = auth::login(&n1.ctx, "owner", &Secret::new(common::OWNER_PASSWORD));
    let wrong_pw = auth::login(&n1.ctx, "owner", &Secret::new("not-the-password"));
    rep.check(
        "N1-9 the Owner signs in with the OLD user name and password; a wrong password is refused",
        session.as_ref().is_ok_and(|s| s.username == "owner") && wrong_pw.is_err(),
        "owner / the old password: signed in",
    );
    let closed = one(&n1.db_path(), "SELECT closed_up_to_date FROM day_state");
    let asked = one(&n1.db_path(), "SELECT question FROM assistant_questions");
    rep.check(
        "N1-10 the closed days and the question log are back as they were",
        closed == one(&old_db, "SELECT closed_up_to_date FROM day_state")
            && asked == question().question,
        format!("closed up to {closed}, question {asked:?}"),
    );
    let actor = auth::require(&n1.ctx).unwrap();
    vouchers::save_receipt(
        &n1.ctx,
        &actor,
        &harness::rupees_input(old.rig.w.arshad, "500"),
    )
    .expect("the app works on the restored books");
    let after_new_entry = fingerprint(&n1.db_path(), Some(&key_of(&n1)));
    rep.check(
        "N1-11 a new entry on the new computer continues the same chains with the same key (NFR-SEC-6)",
        after_new_entry.chains_ok == Some(true) && key_of(&n1).same_as(&old_key),
        "a receipt of 500 posted; both chains verify",
    );
    let again = n1.service.first_run_list(&list_input(file_source));
    rep.check(
        "N1-12 once a shop exists the first-screen restore is closed (nothing can overwrite the books)",
        again.is_err(),
        format!("{:?}", again.err().map(|e| e.code)),
    );

    // =========================================================================================
    // N2: from the CLOUD, with the typed address, bucket and keys; the internet goes half way
    // =========================================================================================
    let flaky = Flaky::new(old.cloud.clone());
    let n2 = new_computer(flaky.clone());
    let fresh_n2 = n2.fingerprint();
    let account = old.account();
    let cloud_source = RestoreSource::Cloud(account.clone());
    let cloud_list = n2
        .service
        .first_run_list(&list_input(cloud_source.clone()))
        .unwrap();
    let cloud_newest_first = cloud_list
        .rows
        .windows(2)
        .all(|w| w[0].created_at >= w[1].created_at);
    rep.check(
        "N2-1 the cloud (address, bucket, region, folder and both keys typed) lists both copies newest first, good and encrypted",
        cloud_list.total_count == 2
            && cloud_newest_first
            && cloud_list.rows.first().map(|r| r.name.as_str()) == Some(second.as_str())
            && cloud_list.rows.iter().all(|r| r.kind == BackupKind::Cloud && r.status == RowStatus::Good && r.encrypted),
        format!(
            "{:?}",
            cloud_list.rows.iter().map(|r| (&r.name, &r.created_at)).collect::<Vec<_>>()
        ),
    );
    let cloud_preview = n2
        .service
        .first_run_preview(
            &preview_input(cloud_source.clone(), None, &old.printed_key),
            &|_| {},
        )
        .unwrap();
    rep.check(
        "N2-2 the preview picks the newest cloud copy, shows its totals and passes every check; nothing is restored yet",
        cloud_preview.copy
            == BackupRef {
                kind: BackupKind::Cloud,
                name: second.clone(),
            }
            && cloud_preview.checks.iter().all(|c| c.ok)
            && cloud_preview.totals == preview.totals
            && n2.is_fresh()
            && n2.fingerprint() == fresh_n2,
        format!("{} entries, made {}", cloud_preview.totals.journal_entries, cloud_preview.created_at),
    );
    // The internet goes while the copy is being fetched.
    flaky.fail_with(Some(CloudError::Offline("reset".into())));
    let cut = n2
        .service
        .first_run_restore(
            &restore_input(cloud_source.clone(), &second, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    rep.check(
        "N2-3 the internet goes half way: a clear cloud error, and the first screen is ready to try again with nothing changed",
        cut.code == BackupCode::ErrBackupCloudFailed && untouched(&n2, &fresh_n2),
        format!("{:?}; no Owner, same rows, no key, no secret, no scratch file", cut.code),
    );
    flaky.fail_with(None);
    let done2 = n2
        .service
        .first_run_restore(
            &restore_input(cloud_source.clone(), &second, &old.printed_key),
            &|_| {},
        )
        .expect("the same screen works once the internet is back");
    let fp_n2 = fingerprint(&n2.db_path(), Some(&key_of(&n2)));
    let extra2 = fp_n2
        .audit_chain
        .len()
        .saturating_sub(fp_copy.audit_chain.len());
    let diff2 = differences(&fp_copy, &fp_n2, extra2, NEW_COMPUTER_SETTINGS);
    let audit_rows_added: Vec<String> = {
        let conn = robustness_backup_support::open_ro(&n2.db_path());
        let mut st = conn
            .prepare("SELECT action || ' ' || entity FROM audit_log WHERE id > ?1 ORDER BY id")
            .unwrap();
        st.query_map([i64::try_from(fp_copy.audit_chain.len()).unwrap()], |r| {
            r.get::<_, String>(0)
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    rep.check(
        "N2-4 restore from the cloud: every table, balance, the trial balance, partner rows, both chains, settings, learned words, days and the question log equal the copy (E2E-18)",
        diff2.is_empty() && fp_n2.chains_ok == Some(true) && done2.chain_key_restored && key_of(&n2).same_as(&old_key) && extra2 >= 1,
        format!(
            "{} tables, {} accounts, chains verify {:?}, differences {diff2:?}, audit rows added by the restore {audit_rows_added:?}",
            fp_n2.counts.len(),
            fp_n2.balances.len(),
            fp_n2.chains_ok
        ),
    );
    let settings = n2.service.settings().unwrap();
    let status = n2.service.status().unwrap();
    rep.check(
        "N2-5 the cloud settings and all three keys are kept, so the next daily copy goes to the same place (FR-20.11)",
        settings.cloud_enabled
            && settings.cloud_endpoint == account.endpoint
            && settings.cloud_bucket == account.bucket
            && settings.cloud_prefix == account.prefix
            && n2.secrets.peek(SecretName::CloudAccessKeyId).as_deref() == Some(account.key_id.as_str())
            && n2.secrets.peek(SecretName::CloudSecretAccessKey).as_deref() == Some(account.secret.as_str())
            && n2.secrets.peek(SecretName::BackupKey).as_deref() == Some(old.printed_key.replace('-', "").as_str())
            && status.key_set
            && status.key_confirmed
            && status.cloud_ready
            && !done2.cloud_needs_keys,
        format!(
            "{} / {} / {}, key set and confirmed {}, cloud ready {}",
            settings.cloud_endpoint, settings.cloud_bucket, settings.cloud_prefix, status.key_confirmed, status.cloud_ready
        ),
    );
    rep.check(
        "N2-6 the Owner signs in with the old user name and password",
        auth::login(&n2.ctx, "owner", &Secret::new(common::OWNER_PASSWORD)).is_ok(),
        "owner / the old password: signed in",
    );

    // the recovery code, on this same computer, from the sign-in screen (nobody is signed in)
    auth::logout(&n2.ctx).expect("sign out");
    let recovered = auth::recover(
        &n2.ctx,
        &auth::RecoverRaw {
            username: "owner".into(),
            recovery_code: Secret::new(old.recovery_code.clone()),
            new_password: Secret::new("brand-new-1"),
            new_password_repeat: Secret::new("brand-new-1"),
        },
    );
    let new_login = auth::login(&n2.ctx, "owner", &Secret::new("brand-new-1"));
    let old_login = auth::login(&n2.ctx, "owner", &Secret::new(common::OWNER_PASSWORD));
    let chains_after_recover = fingerprint(&n2.db_path(), Some(&key_of(&n2))).chains_ok;
    rep.check(
        "N2-7 the printed recovery code sets a new password: the new one works, the old one is refused, a new code is given, the chains still verify (FR-8.5)",
        recovered.as_ref().is_ok_and(|code| !code.is_empty())
            && new_login.is_ok()
            && old_login.is_err()
            && chains_after_recover == Some(true),
        format!(
            "recover {:?}, sign in with the new password {:?}, with the old one {:?}, chains {:?}",
            recovered.as_ref().map(|c| !c.is_empty()),
            new_login.as_ref().map(|s| s.username.clone()),
            old_login.as_ref().map(|s| s.username.clone()),
            chains_after_recover
        ),
    );

    // the next daily copy goes to the same place and restores on a third computer
    let actor2 = auth::require(&n2.ctx).unwrap();
    vouchers::save_receipt(
        &n2.ctx,
        &actor2,
        &harness::rupees_input(old.rig.w.arshad, "2,000"),
    )
    .expect("an entry on the new computer");
    let before_objects = old.cloud.list("backups/").unwrap().len();
    let report = n2
        .service
        .run(Scope {
            local: true,
            usb: false,
            cloud: true,
        })
        .expect("the next daily copy");
    let after_objects = old.cloud.list("backups/").unwrap().len();
    let listed = n2
        .service
        .list(&BackupListInput {
            kind: Some(BackupKind::Cloud),
            offset: 0,
            limit: 10,
        })
        .unwrap();
    let third_name = listed
        .rows
        .first()
        .map(|r| r.name.clone())
        .unwrap_or_default();
    rep.check(
        "N2-8 the next daily copy of the new computer goes to the same cloud and folder, newest first, checked",
        report.results.iter().all(|r| r.ok)
            && after_objects > before_objects
            && listed.rows.len() == 3
            && third_name != second
            && third_name > second,
        format!(
            "objects {before_objects} -> {after_objects}, cloud copies {}, newest {third_name}",
            listed.rows.len()
        ),
    );
    let n3 = new_computer(old.cloud.clone());
    let (_keep3, plain3) = old.unpack_cloud(&third_name);
    let fp_copy3 = fingerprint(&plain3, Some(&old_key));
    let done3 = n3
        .service
        .first_run_restore(
            &restore_input(cloud_source, &third_name, &old.printed_key),
            &|_| {},
        )
        .expect("restore the copy the new computer made");
    let fp_n3 = fingerprint(&n3.db_path(), Some(&key_of(&n3)));
    let extra3 = fp_n3
        .audit_chain
        .len()
        .saturating_sub(fp_copy3.audit_chain.len());
    let diff3 = differences(&fp_copy3, &fp_n3, extra3, NEW_COMPUTER_SETTINGS);
    rep.check(
        "N2-9 that copy restores on a third computer: the books equal it and both chains verify with the key that travelled again",
        diff3.is_empty() && extra3 == 1 && fp_n3.chains_ok == Some(true) && done3.chain_key_restored && key_of(&n3).same_as(&old_key),
        format!(
            "{} journal hashes, {} audit hashes, the receipts of the old day, the one after the first copy and the new one are all there, differences {diff3:?}",
            fp_n3.journal_chain.len(),
            fp_copy3.audit_chain.len()
        ),
    );

    // a copy in the cloud that is damaged: refused, then the same screen takes another place
    let bad_cloud = memory_cloud();
    let scratch = tempfile::tempdir().unwrap();
    let good_bundle = scratch.path().join(&second);
    old.cloud
        .get_file(&format!("backups/{second}"), &good_bundle)
        .unwrap();
    let bad_bundle = scratch.path().join("flipped.mbk");
    flip_a_byte(&good_bundle, &bad_bundle);
    bad_cloud
        .put_file(
            &format!("backups/{second}"),
            &bad_bundle,
            &mandi_backup::io::NoHook,
        )
        .unwrap();
    let n4 = new_computer(bad_cloud);
    let fresh_n4 = n4.fingerprint();
    let bad_restore = n4
        .service
        .first_run_restore(
            &restore_input(RestoreSource::Cloud(account), &second, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    let still_fresh = untouched(&n4, &fresh_n4);
    let retry = n4
        .service
        .first_run_restore(
            &restore_input(
                RestoreSource::File(file.display().to_string()),
                &usb_newest,
                &old.printed_key,
            ),
            &|_| {},
        )
        .is_ok();
    rep.check(
        "N4 a damaged cloud copy is refused and the first screen stays ready; the Owner then picks the USB file on the same screen and it works",
        bad_restore.code != BackupCode::ErrInternal && still_fresh && retry,
        format!("{:?}, then the USB file restored", bad_restore.code),
    );

    rep.finish();
}
