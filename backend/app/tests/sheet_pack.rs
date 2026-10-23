//! The readable pack through the backup service (plan 22 round 10, PRD FR-20.5, NFR-BAK-13):
//! the app's `SheetPack` hands the five spreadsheets to the service, which encrypts each with
//! the backup key and uploads it once a day. The sheets are built from the SNAPSHOT of the copy
//! they travel with. The sheets themselves are checked cell by cell in
//! `crates/services/tests/excel_export.rs`; this file checks the hand-over.
#![cfg(test)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mandi_app_lib::sheet_pack::SheetPack;
use mandi_backup::api::BackupSettingsInput;
use mandi_backup::cloud::{FixedCloud, LocalFolderCloud};
use mandi_backup::daily_pack::{PackPart, PackSource};
use mandi_backup::io::NoHook;
use mandi_backup::run::Scope;
use mandi_backup::secrets::MemorySecretStore;
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_backup::settings::BackupSettings;
use mandi_services::db::disk::FixedDiskProbe;
use mandi_services::testing::TestEnv;

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                found.extend(files_under(&path));
            } else {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn the_sheet_source_hands_over_five_spreadsheets_and_says_the_pdf_is_still_to_come() {
    let dir = tempfile::tempdir().unwrap();
    let env = TestEnv::new(dir.path()).unwrap();
    let source = SheetPack::new(Arc::clone(env.ctx.db().clock()));
    // A consistent copy of the books, as a backup takes it.
    let snapshot = dir.path().join("snapshot.db");
    {
        let live = mandi_services::db::rusqlite::Connection::open(
            dir.path().join("data").join("mandi.db"),
        )
        .unwrap();
        let mut copy = mandi_services::db::rusqlite::Connection::open(&snapshot).unwrap();
        mandi_services::db::rusqlite::backup::Backup::new(&live, &mut copy)
            .unwrap()
            .run_to_completion(64, std::time::Duration::ZERO, None)
            .unwrap();
    }
    assert!(source.pending(), "the PDF part waits for Windows printing");
    let out = tempfile::tempdir().unwrap();
    let files = source.build("2026-10-10", &snapshot, out.path()).unwrap();
    let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "buyers-2026-10-10.xlsx",
            "sellers-2026-10-10.xlsx",
            "shop-expenses-2026-10-10.xlsx",
            "personal-expenses-2026-10-10.xlsx",
            "partner-shares-2026-10-10.xlsx"
        ]
    );
    for file in &files {
        assert_eq!(file.part, PackPart::Sheet);
        let bytes = std::fs::read(&file.path).unwrap();
        assert_eq!(&bytes[..2], b"PK", "{} is a real .xlsx", file.name);
    }
    // A day that is not a date is an error, and the backup itself is not stopped by it.
    assert!(source.build("not-a-day", &snapshot, out.path()).is_err());
    // A snapshot that is not there gives no pack.
    assert!(
        source
            .build("2026-10-10", &dir.path().join("none.db"), out.path())
            .is_err()
    );
}

#[test]
fn the_service_uploads_the_five_sheets_encrypted_once_a_day() {
    let dir = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let env = TestEnv::new(dir.path()).unwrap();
    let cloud = LocalFolderCloud::new(root.path().join("cloud"));
    let service = Arc::new(BackupService::new(
        Arc::clone(env.ctx.db()),
        BackupConfig {
            backups_dir: root.path().join("backups"),
            app_version: "0.0.0-test".to_owned(),
        },
        BackupParts {
            secrets: Arc::new(MemorySecretStore::default()),
            cloud: Arc::new(FixedCloud(Arc::new(cloud))),
            disk: Arc::new(FixedDiskProbe(10 * 1024 * 1024 * 1024)),
            hook: Arc::new(NoHook),
            pack: Arc::new(SheetPack::new(Arc::clone(env.ctx.db().clock()))),
        },
    ));
    service
        .settings_set(
            None,
            &BackupSettingsInput {
                settings: BackupSettings {
                    cloud_enabled: true,
                    cloud_endpoint: "https://example.invalid".into(),
                    cloud_bucket: "mandi-test".into(),
                    ..BackupSettings::default()
                },
                cloud_access_key_id: Some("AKIATEST".into()),
                cloud_secret_access_key: Some("secret-test-value".into()),
            },
        )
        .unwrap();
    service.key_setup(None).unwrap();
    assert!(service.status().unwrap().pack_pending);
    service.run(Scope::ALL).unwrap();
    service.run(Scope::ALL).unwrap();
    let packed: Vec<PathBuf> = files_under(&root.path().join("cloud").join("pack"));
    let names: Vec<String> = packed
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(packed.len(), 5, "one pack for the day: {names:?}");
    for (stem, name) in [
        "buyers",
        "partner-shares",
        "personal-expenses",
        "sellers",
        "shop-expenses",
    ]
    .iter()
    .zip(&names)
    {
        assert!(
            name.starts_with(stem) && name.ends_with(".xlsx.age"),
            "{name}"
        );
    }
    // Nothing readable leaves the computer: no uploaded file is a plain zip.
    for path in &packed {
        let bytes = std::fs::read(path).unwrap();
        assert_ne!(&bytes[..2], b"PK", "{} is encrypted", path.display());
    }
}
