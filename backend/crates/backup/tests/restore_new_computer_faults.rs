//! Plan 56 (PRD FR-20.11): what goes wrong on the first screen of a new computer, and the proof
//! that a restore is one safe operation. A wrong key, a damaged copy, a wrong bucket, wrong cloud
//! keys, no internet, a copy newer than the app, a copy made before the chain key travelled, a
//! copy whose chains do not verify, a shop that already exists, a locked credential store, a data
//! file that cannot be written. Each leaves the computer as a fresh install.
//!
//! The last tests prove that no key is ever written to a plain file or a log.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
mod new_computer;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use std::fmt::Write as _;
use std::sync::{Arc, Mutex, PoisonError};

use mandi_backup::api::{
    BackupRef, CloudAccountInput, FirstRunPreviewInput, FirstRunRestoreInput, RestoreInput,
    RestoreSource,
};
use mandi_backup::cloud::{CloudError, CloudStore};
use mandi_backup::error::BackupCode;
use mandi_backup::key::BackupKey;
use mandi_backup::model::BackupKind;
use mandi_backup::secrets::SecretName;
use mandi_backup::secrets::SecretStore;
use mandi_services::auth;
use mandi_services::db::rusqlite::Connection;
use new_computer::*;
use tracing::field::{Field, Visit};
use tracing::{Event, Metadata, span};

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

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = items
        .iter()
        .map(|(c, f)| ((*c).to_owned(), (*f).to_owned()))
        .collect();
    found.sort();
    found
}

/// The new computer is exactly as a fresh install: no Owner, no stored secret, no scratch file.
fn assert_still_fresh(new: &New, before: &std::collections::BTreeMap<String, (i64, String)>) {
    assert!(new.is_fresh(), "no Owner exists");
    assert_eq!(&new.fingerprint(), before, "the data file is the fresh one");
    for name in [
        SecretName::BackupKey,
        SecretName::CloudAccessKeyId,
        SecretName::CloudSecretAccessKey,
    ] {
        assert_eq!(new.secrets.peek(name), None, "{name:?} was not kept");
    }
    if let Some(key) = new.keys.current() {
        assert!(
            !key.same_as(&old_chain_key()),
            "the old chain key was not kept"
        );
    }
    let tmp = new.root.path().join("backups").join("tmp");
    let left = std::fs::read_dir(tmp).map_or(0, std::iter::Iterator::count);
    assert_eq!(left, 0, "the scratch folder is empty");
}

// ---- the key, the copy, the cloud ----

#[test]
fn a_wrong_or_badly_typed_key_changes_nothing_and_points_at_the_key() {
    let old = old_computer();
    let name = old.back_up();
    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();

    let wrong = BackupKey::generate().unwrap().display();
    for call in 0..2 {
        let error = if call == 0 {
            new.service
                .first_run_preview(
                    &preview_input(cloud_source(&old), Some(&name), &wrong),
                    &|_| {},
                )
                .unwrap_err()
        } else {
            new.service
                .first_run_restore(&restore_input(cloud_source(&old), &name, &wrong), &|_| {})
                .unwrap_err()
        };
        assert_eq!(codes(&error), pairs(&[("ERR_BACKUP_KEY_WRONG", "key")]));
    }
    // A key in the wrong form, together with a bad address: both come back at once.
    let mut account = old.account();
    account.endpoint = "http://not-secure".into();
    let error = new
        .service
        .first_run_preview(
            &preview_input(RestoreSource::Cloud(account), Some(&name), "ABCD"),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(
        codes(&error),
        pairs(&[
            ("ERR_BACKUP_KEY_FORMAT", "key"),
            ("ERR_OUT_OF_RANGE", "source.cloud.endpoint"),
        ])
    );
    assert_still_fresh(&new, &before);
}

#[test]
fn a_damaged_copy_is_refused_with_the_check_that_failed_and_nothing_changes() {
    let old = old_computer();
    let name = old.back_up();
    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();

    // 1. One byte changed in the cloud: the checksum recorded when it was made no longer fits.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("copy.mbk");
    old.cloud
        .get_file(&format!("backups/{name}"), &path)
        .unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0x55;
    let flipped = dir.path().join("flipped.mbk");
    std::fs::write(&flipped, &bytes).unwrap();
    old.cloud
        .put_file(
            &format!("backups/{name}"),
            &flipped,
            &mandi_backup::io::NoHook,
        )
        .unwrap();
    let error = new
        .service
        .first_run_preview(
            &preview_input(cloud_source(&old), Some(&name), &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(codes(&error), pairs(&[("ERR_BACKUP_CORRUPT", "copy")]));
    assert_eq!(value_of(&error, "check").as_deref(), Some("checksum"));

    // 2. Cut short, with no manifest to tell: the bundle cannot be read to its end.
    old.cloud
        .delete(&format!(
            "backups/{}",
            mandi_backup::names::manifest_name(&name)
        ))
        .unwrap();
    bytes = std::fs::read(&path).unwrap();
    bytes.truncate(bytes.len() / 2);
    let cut = dir.path().join("cut.mbk");
    std::fs::write(&cut, &bytes).unwrap();
    old.cloud
        .put_file(&format!("backups/{name}"), &cut, &mandi_backup::io::NoHook)
        .unwrap();
    let error = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrBackupCorrupt, "{error:?}");
    assert_eq!(error.field.as_deref(), Some("copy"));
    assert_still_fresh(&new, &before);
}

#[test]
fn a_wrong_bucket_wrong_cloud_keys_and_no_internet_each_name_their_fields() {
    let old = old_computer();
    old.back_up();
    let flaky = Flaky::new(old.cloud.clone());
    let new = new_computer(flaky.clone());
    let before = new.fingerprint();
    let list = |new: &New| {
        new.service
            .first_run_list(&list_input(cloud_source(&old)))
            .unwrap_err()
    };

    flaky.fail_with(Some(CloudError::Refused("403".into())));
    let error = list(&new);
    assert_eq!(
        codes(&error),
        pairs(&[
            ("ERR_BACKUP_CLOUD_FAILED", "source.cloud.key_id"),
            ("ERR_BACKUP_CLOUD_FAILED", "source.cloud.secret"),
        ])
    );
    assert_eq!(value_of(&error, "reason").as_deref(), Some("refused"));

    flaky.fail_with(Some(CloudError::NotFound));
    let error = list(&new);
    assert_eq!(
        codes(&error),
        pairs(&[("ERR_BACKUP_CLOUD_FAILED", "source.cloud.bucket")])
    );
    assert_eq!(value_of(&error, "reason").as_deref(), Some("not_found"));

    flaky.fail_with(Some(CloudError::Offline("no route".into())));
    let error = list(&new);
    assert_eq!(
        codes(&error),
        pairs(&[("ERR_BACKUP_CLOUD_FAILED", "source.cloud.endpoint")])
    );
    assert_eq!(value_of(&error, "reason").as_deref(), Some("offline"));

    // The cloud's own words never reach the log text of the error (they can hold a key id).
    assert!(!format!("{error:?}").contains("no route"));

    // The internet goes away while a copy is being fetched: the same words, nothing changed.
    let name = old.latest(BackupKind::Cloud);
    flaky.fail_with(Some(CloudError::Offline("reset".into())));
    let error = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(
        codes(&error),
        pairs(&[("ERR_BACKUP_CLOUD_FAILED", "source.cloud.endpoint")])
    );
    assert_still_fresh(&new, &before);

    // A copy that is not there: a copy error, not a bucket error.
    flaky.fail_with(None);
    let error = new
        .service
        .first_run_preview(
            &preview_input(
                cloud_source(&old),
                Some("mandi-20200101T000000000Z.mbk"),
                &old.printed_key,
            ),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(codes(&error), pairs(&[("ERR_BACKUP_NOT_FOUND", "copy")]));
    // A name that points outside the folder is refused before anything is asked.
    let error = new
        .service
        .first_run_preview(
            &preview_input(cloud_source(&old), Some("../mandi-x.mbk"), &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(codes(&error), pairs(&[("ERR_OUT_OF_RANGE", "copy")]));

    // An empty place is an empty list, and a preview with no copy says "not found".
    let empty = new_computer(memory_cloud());
    let copies = empty
        .service
        .first_run_list(&list_input(cloud_source(&old)))
        .unwrap();
    assert!(copies.rows.is_empty());
    assert_eq!(copies.total_count, 0);
    let error = empty
        .service
        .first_run_preview(
            &preview_input(cloud_source(&old), None, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(codes(&error), pairs(&[("ERR_BACKUP_NOT_FOUND", "copy")]));
}

#[test]
fn bad_fields_and_missing_places_name_their_fields_all_together() {
    let old = old_computer();
    let new = new_computer(old.cloud.clone());
    let blank = CloudAccountInput {
        endpoint: " ".into(),
        bucket: String::new(),
        region: String::new(),
        prefix: "a..b".into(),
        key_id: String::new(),
        secret: "x\u{7}y".into(),
    };
    let error = new
        .service
        .first_run_list(&list_input(RestoreSource::Cloud(blank)))
        .unwrap_err();
    assert_eq!(
        codes(&error),
        pairs(&[
            ("ERR_REQUIRED", "source.cloud.endpoint"),
            ("ERR_REQUIRED", "source.cloud.bucket"),
            ("ERR_REQUIRED", "source.cloud.region"),
            ("ERR_OUT_OF_RANGE", "source.cloud.prefix"),
            ("ERR_REQUIRED", "source.cloud.key_id"),
            ("ERR_OUT_OF_RANGE", "source.cloud.secret"),
        ])
    );
    let cases = [
        (
            RestoreSource::Folder("relative/folder".into()),
            "ERR_OUT_OF_RANGE",
            "source.folder",
        ),
        (
            RestoreSource::Folder(String::new()),
            "ERR_REQUIRED",
            "source.folder",
        ),
        (
            RestoreSource::Folder("/mandi-test-no-such-folder".into()),
            "ERR_BACKUP_USB_NOT_PRESENT",
            "source.folder",
        ),
        (
            RestoreSource::File("/mandi-test-no-such-file.mbk".into()),
            "ERR_BACKUP_USB_NOT_PRESENT",
            "source.file",
        ),
        (
            RestoreSource::File("/tmp/notes.txt".into()),
            "ERR_OUT_OF_RANGE",
            "source.file",
        ),
    ];
    for (source, code, field) in cases {
        let error = new.service.first_run_list(&list_input(source)).unwrap_err();
        assert_eq!(codes(&error), pairs(&[(code, field)]));
    }
    // A folder is not a file.
    let error = new
        .service
        .first_run_list(&list_input(RestoreSource::File(
            old.rig.usb.display().to_string() + ".mbk",
        )))
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrBackupUsbNotPresent);
}

#[test]
fn a_cloud_address_that_is_refused_says_why_on_the_first_screen_and_in_settings() {
    let old = old_computer();
    let new = new_computer(old.cloud.clone());
    for (address, reason) in [
        ("http://acct.r2.cloudflarestorage.com", "https"),
        ("acct.r2.cloudflarestorage.com", "https"),
    ] {
        // The first screen: the field is the one of the input.
        let mut account = old.account();
        account.endpoint = address.into();
        let error = new
            .service
            .first_run_list(&list_input(RestoreSource::Cloud(account)))
            .unwrap_err();
        assert_eq!(
            codes(&error),
            pairs(&[("ERR_OUT_OF_RANGE", "source.cloud.endpoint")]),
            "{address}"
        );
        assert_eq!(
            value_of(&error, "reason").as_deref(),
            Some(reason),
            "{address}"
        );

        // Settings: the same check and the same value, on the field of Settings.
        let owner = old.rig.w.owner.user_id;
        let error = old
            .service
            .settings_set(
                Some(owner),
                &mandi_backup::api::BackupSettingsInput {
                    settings: mandi_backup::settings::BackupSettings {
                        cloud_enabled: true,
                        cloud_endpoint: address.into(),
                        cloud_bucket: "mandi-backup".into(),
                        ..mandi_backup::settings::BackupSettings::default()
                    },
                    cloud_access_key_id: None,
                    cloud_secret_access_key: None,
                },
            )
            .unwrap_err();
        assert_eq!(
            codes(&error),
            pairs(&[("ERR_OUT_OF_RANGE", "cloud_endpoint")]),
            "{address}"
        );
        assert_eq!(
            value_of(&error, "reason").as_deref(),
            Some(reason),
            "{address}"
        );
    }
}

/// The bucket name put at the end of the address (plan 53 section 10) has its own code, on the
/// first screen and in Settings, with the address to use. Settings saves nothing.
#[test]
fn a_cloud_address_with_a_path_says_so_and_offers_the_address_without_it() {
    let old = old_computer();
    let new = new_computer(old.cloud.clone());
    let address = "https://acct.r2.cloudflarestorage.com/mandidemo";

    // The first screen: the field is the one of the input.
    let mut account = old.account();
    account.endpoint = address.into();
    let error = new
        .service
        .first_run_list(&list_input(RestoreSource::Cloud(account)))
        .unwrap_err();
    assert_eq!(
        codes(&error),
        pairs(&[("ERR_BACKUP_CLOUD_ADDRESS_PATH", "source.cloud.endpoint")])
    );
    assert_eq!(
        value_of(&error, "address").as_deref(),
        Some("https://acct.r2.cloudflarestorage.com")
    );
    assert_eq!(value_of(&error, "path").as_deref(), Some("/mandidemo"));
    assert_eq!(value_of(&error, "bucket").as_deref(), Some("mandidemo"));

    // Settings: the same code and values on the field of Settings, and nothing is saved.
    let owner = old.rig.w.owner.user_id;
    let before = old.service.settings().unwrap();
    let error = old
        .service
        .settings_set(
            Some(owner),
            &mandi_backup::api::BackupSettingsInput {
                settings: mandi_backup::settings::BackupSettings {
                    cloud_enabled: true,
                    cloud_endpoint: address.into(),
                    cloud_bucket: "mandidemo".into(),
                    ..mandi_backup::settings::BackupSettings::default()
                },
                cloud_access_key_id: None,
                cloud_secret_access_key: None,
            },
        )
        .unwrap_err();
    assert_eq!(
        codes(&error),
        pairs(&[("ERR_BACKUP_CLOUD_ADDRESS_PATH", "cloud_endpoint")])
    );
    assert_eq!(
        value_of(&error, "address").as_deref(),
        Some("https://acct.r2.cloudflarestorage.com")
    );
    assert_eq!(value_of(&error, "bucket").as_deref(), Some("mandidemo"));
    assert_eq!(old.service.settings().unwrap(), before, "nothing was saved");

    // The address the error offers is accepted.
    old.service
        .settings_set(
            Some(owner),
            &mandi_backup::api::BackupSettingsInput {
                settings: mandi_backup::settings::BackupSettings {
                    cloud_enabled: true,
                    cloud_endpoint: "https://acct.r2.cloudflarestorage.com".into(),
                    cloud_bucket: "mandidemo".into(),
                    ..mandi_backup::settings::BackupSettings::default()
                },
                cloud_access_key_id: None,
                cloud_secret_access_key: None,
            },
        )
        .unwrap();
}

#[test]
fn a_cloud_copy_that_does_not_fit_is_refused_before_the_download_starts() {
    let old = old_computer();
    let name = old.back_up();
    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();
    // 4 MiB free: the copy (and the 16 MiB margin) cannot fit.
    new.service_disk.set(4 * 1024 * 1024);
    for call in 0..2 {
        let error = if call == 0 {
            new.service
                .first_run_preview(
                    &preview_input(cloud_source(&old), Some(&name), &old.printed_key),
                    &|_| {},
                )
                .unwrap_err()
        } else {
            new.service
                .first_run_restore(
                    &restore_input(cloud_source(&old), &name, &old.printed_key),
                    &|_| {},
                )
                .unwrap_err()
        };
        assert_eq!(codes(&error), pairs(&[("ERR_DISK_FULL", "")]));
    }
    // Nothing was fetched into the scratch folder, and room again lets it through.
    assert_still_fresh(&new, &before);
    new.service_disk.set(10 * 1024 * 1024 * 1024);
    new.service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap();
}

#[test]
fn a_copy_that_holds_no_owner_is_refused_on_the_first_screen_and_in_the_signed_in_restore() {
    // A copy of a shop that was never set up: valid and current, but nobody can sign in to it.
    let old = old_computer();
    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();
    let dir = tempfile::tempdir().unwrap();
    let empty = dir.path().join("never-set-up.db");
    {
        let conn = Connection::open(new.db_path()).unwrap();
        conn.execute("VACUUM INTO ?1", [empty.to_str().unwrap()])
            .unwrap();
    }
    let key = BackupKey::parse(&old.printed_key).unwrap();
    let name = put_crafted_copy(
        &old.cloud,
        &key,
        &empty,
        "20261016T090000000Z",
        Some(&old_chain_key()),
        true,
    );
    let error = new
        .service
        .first_run_preview(
            &preview_input(cloud_source(&old), Some(&name), &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(codes(&error), pairs(&[("ERR_BACKUP_NO_OWNER", "copy")]));
    assert_still_fresh(&new, &before);

    // The signed-in restore would erase the Owner and leave nobody to sign in: refused too.
    let actor = auth::require(&old.rig.w.env.ctx).unwrap();
    let error = old
        .service
        .restore(
            Some(actor.user_id),
            &RestoreInput {
                backup: BackupRef {
                    kind: BackupKind::Cloud,
                    name,
                },
                key: None,
                confirmed: true,
            },
        )
        .unwrap_err();
    assert_eq!(codes(&error), pairs(&[("ERR_BACKUP_NO_OWNER", "backup")]));
    auth::require(&old.rig.w.env.ctx).expect("the Owner is still signed in");
}

// ---- the first-run state ----

#[test]
fn nothing_works_once_a_shop_exists_and_nothing_is_changed() {
    let old = old_computer();
    let name = old.back_up();
    let before_data = table_hashes(&old.rig.w.dir.path().join("data").join("mandi.db"), &[]);
    let before_key = old.secrets.get(SecretName::BackupKey).unwrap();
    let source = cloud_source(&old);

    let list = old
        .service
        .first_run_list(&list_input(source.clone()))
        .unwrap_err();
    let preview = old
        .service
        .first_run_preview(
            &preview_input(source.clone(), Some(&name), &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    let restore = old
        .service
        .first_run_restore(&restore_input(source, &name, &old.printed_key), &|_| {})
        .unwrap_err();
    for error in [list, preview, restore] {
        assert_eq!(error.code, BackupCode::ErrAlreadySetUp);
        assert!(error.also.is_empty());
    }
    assert_eq!(
        table_hashes(&old.rig.w.dir.path().join("data").join("mandi.db"), &[]),
        before_data,
        "no table changed"
    );
    assert_eq!(old.secrets.get(SecretName::BackupKey).unwrap(), before_key);

    // The same after a first-run restore: the second one is refused.
    let new = new_computer(old.cloud.clone());
    new.service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap();
    let again = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(again.code, BackupCode::ErrAlreadySetUp);
}

// ---- a restore is one safe operation ----

#[test]
fn a_locked_credential_store_half_way_leaves_the_computer_a_fresh_install() {
    let old = old_computer();
    let name = old.back_up();
    // The first key can be stored; the second is refused (a locked Keychain).
    let secrets = FlakySecrets::new(1);
    let new = new_computer_with(old.cloud.clone(), secrets);
    let before = new.fingerprint();
    let error = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrInternal);
    assert_still_fresh(&new, &before);

    // Nothing is stuck: with a working store the same restore goes through.
    let healthy = new_computer(old.cloud.clone());
    healthy
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap();
}

#[test]
fn a_data_file_that_cannot_be_written_after_the_swap_is_put_back_with_every_key() {
    let old = old_computer();
    let name = old.back_up();
    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();
    // The disk of the data file is full from now on: the data file is replaced (no save is
    // needed for that), then the cloud settings cannot be saved, and everything goes back.
    new.data_disk.set(0);
    let error = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrDiskFull, "{error:?}");
    assert_still_fresh(&new, &before);

    // Room again: it goes through, and the books and chains are right.
    new.data_disk.set(10 * 1024 * 1024 * 1024);
    let done = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &name, &old.printed_key),
            &|_| {},
        )
        .unwrap();
    assert!(done.chain_key_restored);
    assert!(new.keys.current().unwrap().same_as(&old_chain_key()));
    assert!(!new.is_fresh());
}

// ---- the chain key ----

#[test]
fn a_copy_made_before_the_key_travelled_is_refused_on_a_new_computer_but_not_on_its_own() {
    let old = old_computer();
    let real = old.back_up();
    let (_keep, plain) = old.unpack_cloud(&real);
    let key = BackupKey::parse(&old.printed_key).unwrap();
    let keyless = put_crafted_copy(&old.cloud, &key, &plain, "20261012T090000000Z", None, true);

    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();
    let error = new
        .service
        .first_run_preview(
            &preview_input(cloud_source(&old), Some(&keyless), &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(
        codes(&error),
        pairs(&[("ERR_BACKUP_CHAIN_KEY_MISSING", "copy")])
    );
    let error = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &keyless, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(error.code, BackupCode::ErrBackupChainKeyMissing);
    assert_still_fresh(&new, &before);

    // The newest copy that is not key-less still restores: the Owner picks that one.
    new.service
        .first_run_restore(
            &restore_input(cloud_source(&old), &real, &old.printed_key),
            &|_| {},
        )
        .unwrap();

    // On its own computer the key-less copy is fine: the key the computer holds verifies it.
    let actor = auth::require(&old.rig.w.env.ctx).unwrap();
    old.service
        .restore(
            Some(actor.user_id),
            &RestoreInput {
                backup: BackupRef {
                    kind: BackupKind::Cloud,
                    name: keyless,
                },
                key: None,
                confirmed: true,
            },
        )
        .unwrap();
}

#[test]
fn a_copy_whose_chains_do_not_verify_is_refused_even_with_its_own_key() {
    let old = old_computer();
    let real = old.back_up();
    let (_keep, plain) = old.unpack_cloud(&real);
    {
        // Someone changed an audit row outside the app: the triggers are removed in this copy.
        let conn = Connection::open(&plain).unwrap();
        let triggers: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'trigger' AND tbl_name = 'audit_log'",
            )
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for name in triggers {
            conn.execute_batch(&format!("DROP TRIGGER \"{name}\""))
                .unwrap();
        }
        conn.execute("UPDATE audit_log SET reason = 'changed' WHERE id = 2", [])
            .unwrap();
    }
    let key = BackupKey::parse(&old.printed_key).unwrap();
    let tampered = put_crafted_copy(
        &old.cloud,
        &key,
        &plain,
        "20261013T090000000Z",
        Some(&old_chain_key()),
        true,
    );
    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();
    let error = new
        .service
        .first_run_restore(
            &restore_input(cloud_source(&old), &tampered, &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(codes(&error), pairs(&[("ERR_BACKUP_CORRUPT", "copy")]));
    assert_eq!(value_of(&error, "check").as_deref(), Some("chains"));
    assert_still_fresh(&new, &before);
}

#[test]
fn a_copy_made_by_a_newer_app_is_refused() {
    let old = old_computer();
    let real = old.back_up();
    let (_keep, plain) = old.unpack_cloud(&real);
    {
        let conn = Connection::open(&plain).unwrap();
        conn.pragma_update(None, "user_version", 9_999).unwrap();
    }
    let key = BackupKey::parse(&old.printed_key).unwrap();
    let newer = put_crafted_copy(
        &old.cloud,
        &key,
        &plain,
        "20261014T090000000Z",
        Some(&old_chain_key()),
        true,
    );
    let new = new_computer(old.cloud.clone());
    let before = new.fingerprint();
    let error = new
        .service
        .first_run_preview(
            &preview_input(cloud_source(&old), Some(&newer), &old.printed_key),
            &|_| {},
        )
        .unwrap_err();
    assert_eq!(
        codes(&error),
        pairs(&[("ERR_BACKUP_NEWER_THAN_APP", "copy")])
    );
    assert_still_fresh(&new, &before);
}

// ---- no key on disk, none in a log ----

fn secrets_of(old: &Old) -> Vec<(&'static str, Vec<u8>)> {
    let key = old_chain_key();
    let printed = old.printed_key.clone();
    vec![
        ("chain key (hex)", key.to_hex().into_bytes()),
        ("chain key (HEX)", key.to_hex().to_uppercase().into_bytes()),
        ("chain key (raw)", vec![7_u8; 32]),
        ("backup key (dashes)", printed.clone().into_bytes()),
        ("backup key (plain)", printed.replace('-', "").into_bytes()),
        ("new cloud secret", b"new-secret-value-2".to_vec()),
        ("old cloud secret", b"old-secret-value-1".to_vec()),
    ]
}

#[test]
fn no_copy_file_next_to_the_data_holds_a_key_in_plain_form() {
    let old = old_computer();
    old.back_up();
    old.rig.w.env.clock.advance_seconds(60);
    old.back_up();
    // Every file the backup wrote on the old computer: the local copies (plain databases), the
    // manifests (.json) and the USB bundles. None holds a key. A bundle is locked, so it holds
    // none in plain form either.
    let mut files = Vec::new();
    files_under(&old.rig.backups, &mut files);
    files_under(&old.local_dir, &mut files);
    files_under(&old.rig.usb, &mut files);
    assert!(
        files
            .iter()
            .any(|(p, _)| p.extension().is_some_and(|e| e == "json"))
    );
    assert!(
        files
            .iter()
            .any(|(p, _)| p.extension().is_some_and(|e| e == "db"))
    );
    assert!(
        files
            .iter()
            .any(|(p, _)| p.extension().is_some_and(|e| e == "mbk"))
    );
    for (label, needle) in secrets_of(&old) {
        for (path, bytes) in &files {
            assert!(
                !contains(bytes, &needle),
                "{label} is in {}",
                path.display()
            );
        }
    }
    // The live data file of the old computer holds none either.
    let live = old.rig.w.dir.path().join("data");
    let mut data = Vec::new();
    files_under(&live, &mut data);
    for (label, needle) in secrets_of(&old) {
        for (path, bytes) in &data {
            assert!(
                !contains(bytes, &needle),
                "{label} is in {}",
                path.display()
            );
        }
    }
}

/// Collects every event a thread logs, at any level, as text.
struct Capture(Arc<Mutex<String>>);

struct Text<'a>(&'a mut String);

impl Visit for Text<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let _ = write!(self.0, "{}={:?} ", field.name(), value);
    }
}

impl tracing::Subscriber for Capture {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, attrs: &span::Attributes<'_>) -> span::Id {
        let mut text = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        attrs.record(&mut Text(&mut text));
        span::Id::from_u64(1)
    }
    fn record(&self, _: &span::Id, values: &span::Record<'_>) {
        let mut text = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        values.record(&mut Text(&mut text));
    }
    fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}
    fn event(&self, event: &Event<'_>) {
        let mut text = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = write!(text, "{} ", event.metadata().target());
        event.record(&mut Text(&mut text));
        text.push('\n');
    }
    fn enter(&self, _: &span::Id) {}
    fn exit(&self, _: &span::Id) {}
}

#[test]
fn a_restore_on_a_new_computer_writes_no_key_to_any_file_or_log_and_prints_none() {
    let old = old_computer();
    let name = old.back_up();
    let new = new_computer(old.cloud.clone());
    let log = Arc::new(Mutex::new(String::new()));
    let input = restore_input(cloud_source(&old), &name, &old.printed_key);
    let wrong = restore_input(
        cloud_source(&old),
        &name,
        &BackupKey::generate().unwrap().display(),
    );
    let (failed, done) = tracing::subscriber::with_default(Capture(log.clone()), || {
        // A probe: the capture sees what the thread logs.
        tracing::warn!("capture probe");
        // A failed restore first, then a good one, so the failure paths are logged too.
        let failed = new.service.first_run_restore(&wrong, &|_| {}).unwrap_err();
        let done = new.service.first_run_restore(&input, &|_| {}).unwrap();
        (failed, done)
    });

    // The inputs and the errors print no secret.
    let printed = format!("{input:?} {wrong:?} {failed:?} {done:?} {}", failed.detail);
    let logged = log.lock().unwrap_or_else(PoisonError::into_inner).clone();
    assert!(logged.contains("capture probe"), "the capture works");
    for (label, needle) in secrets_of(&old) {
        let text = String::from_utf8_lossy(&needle).into_owned();
        if needle.len() == 32 && needle.iter().all(|b| *b == 7) {
            continue;
        }
        assert!(!printed.contains(&text), "{label} is in a Debug text");
        assert!(!logged.contains(&text), "{label} is in the log: {logged}");
    }
    assert!(printed.contains("[hidden]"));

    // Every file of the new computer, the data file included: no key in plain form.
    let files = new.all_files();
    assert!(
        files.len() > 3,
        "the scan saw the data file and the backup files"
    );
    for (label, needle) in secrets_of(&old) {
        for (path, bytes) in &files {
            assert!(
                !contains(bytes, &needle),
                "{label} is in {}",
                path.display()
            );
        }
    }
}
