//! A rig for the backup tests: the PRD worked day in a real database, a backup service with a
//! folder for the cloud, a folder for the USB drive, memory secrets and switches for faults.
#![allow(dead_code)]

use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use mandi_backup::api::{BackupSettingsInput, RunReport};
use mandi_backup::cloud::{FixedCloud, LocalFolderCloud};
use mandi_backup::io::IoHook;
use mandi_backup::model::BackupKind;
use mandi_backup::secrets::MemorySecretStore;
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_backup::settings::BackupSettings;
use mandi_db::disk::DiskProbe;
use mandi_services::core::bikri::{BikriDraft, HeaderInput, PukhtaInput};
use mandi_services::core::vouchers::{PaymentInput, ReceiptInput};
use mandi_services::db::rusqlite::Connection;
use mandi_services::{bikri, vouchers};
use tempfile::TempDir;

use crate::support;

/// Free disk space the backup sees; a test sets it small.
#[derive(Debug)]
pub struct Disk(AtomicU64);

impl Disk {
    pub fn set(&self, bytes: u64) {
        self.0.store(bytes, Ordering::SeqCst);
    }
}

impl DiskProbe for Disk {
    fn available_bytes(&self, _: &std::path::Path) -> io::Result<u64> {
        Ok(self.0.load(Ordering::SeqCst))
    }
}

/// Breaks writes to one place once more than `after` bytes of a file are written.
#[derive(Debug, Default)]
pub struct Fault {
    armed: AtomicBool,
    after: AtomicU64,
    kind: Mutex<Option<BackupKind>>,
}

impl Fault {
    pub fn arm(&self, kind: BackupKind, after: u64) {
        *self.kind.lock().unwrap_or_else(PoisonError::into_inner) = Some(kind);
        self.after.store(after, Ordering::SeqCst);
        self.armed.store(true, Ordering::SeqCst);
    }

    pub fn disarm(&self) {
        self.armed.store(false, Ordering::SeqCst);
    }
}

impl IoHook for Fault {
    fn before_write(&self, target: BackupKind, written: u64) -> io::Result<()> {
        let kind = *self.kind.lock().unwrap_or_else(PoisonError::into_inner);
        if self.armed.load(Ordering::SeqCst)
            && kind == Some(target)
            && written >= self.after.load(Ordering::SeqCst)
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "the drive was pulled out",
            ));
        }
        Ok(())
    }
}

/// Calls a function while a copy is being written (to look at the service in the middle).
pub struct Spy(pub OnceLock<Box<dyn Fn() + Send + Sync>>);

impl IoHook for Spy {
    fn before_write(&self, target: BackupKind, written: u64) -> io::Result<()> {
        if target == BackupKind::Usb
            && written == 0
            && let Some(look) = self.0.get()
        {
            look();
        }
        Ok(())
    }
}

pub struct Rig {
    pub w: support::World,
    pub service: Arc<BackupService>,
    pub cloud: LocalFolderCloud,
    pub secrets: Arc<MemorySecretStore>,
    pub disk: Arc<Disk>,
    pub fault: Arc<Fault>,
    pub root: TempDir,
    pub usb: PathBuf,
    pub backups: PathBuf,
}

/// Builds the service for an existing world, with the cloud and USB switched on.
pub fn rig_for(w: support::World, hook: Option<Arc<dyn IoHook>>) -> Rig {
    let root = tempfile::tempdir().unwrap();
    let backups = root.path().join("backups");
    let usb = root.path().join("usb").join("MandiBackup");
    let cloud = LocalFolderCloud::new(root.path().join("cloud"));
    let secrets = Arc::new(MemorySecretStore::default());
    let disk = Arc::new(Disk(AtomicU64::new(10 * 1024 * 1024 * 1024)));
    let fault = Arc::new(Fault::default());
    let hook: Arc<dyn IoHook> = hook.unwrap_or_else(|| fault.clone());
    std::fs::create_dir_all(root.path().join("usb")).unwrap();
    let service = Arc::new(BackupService::new(
        Arc::clone(w.env.ctx.db()),
        BackupConfig {
            backups_dir: backups.clone(),
            app_version: "0.0.0-test".to_owned(),
        },
        BackupParts {
            secrets: secrets.clone(),
            cloud: Arc::new(FixedCloud(Arc::new(cloud.clone()))),
            disk: disk.clone(),
            hook,
            pack: Arc::new(mandi_backup::daily_pack::PendingPack),
        },
    ));
    Rig {
        w,
        service,
        cloud,
        secrets,
        disk,
        fault,
        root,
        usb,
        backups,
    }
}

pub fn settings_on(rig: &Rig) -> BackupSettings {
    BackupSettings {
        usb_enabled: true,
        usb_folder: rig.usb.display().to_string(),
        cloud_enabled: true,
        cloud_endpoint: "https://example.invalid".into(),
        cloud_bucket: "mandi-test".into(),
        ..BackupSettings::default()
    }
}

/// Saves the settings with the cloud and USB on, and makes the key. Returns the printed key.
pub fn switch_everything_on(rig: &Rig) -> String {
    rig.service
        .settings_set(
            Some(rig.w.owner.user_id),
            &BackupSettingsInput {
                settings: settings_on(rig),
                cloud_access_key_id: Some("AKIATEST".into()),
                cloud_secret_access_key: Some("secret-test-value".into()),
            },
        )
        .unwrap();
    rig.service
        .key_setup(Some(rig.w.owner.user_id))
        .unwrap()
        .key
}

pub fn rupees_input(account: i64, amount: &str) -> ReceiptInput {
    ReceiptInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: amount.into(),
        removed: Default::default(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

fn event2(w: &support::World) -> BikriDraft {
    let mut line = support::sale("s-1", w.pukhta_buyer, "10", "1200");
    line.pukhta = Some(PukhtaInput {
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

fn save(w: &support::World, draft: &BikriDraft) {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// PRD 4.4 to 4.7: Events 1 and 2 (the two بکری), Event 3 (اکرم pays 10,000) and Event 4
/// (رحیم is paid 4,316).
pub fn worked_day() -> Rig {
    let w = support::world();
    save(&w, &support::event1(&w));
    save(&w, &event2(&w));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &rupees_input(w.akram, "10,000")).unwrap();
    vouchers::save_payment(
        &w.env.ctx,
        &w.owner,
        &PaymentInput {
            date: "10-10-2026".into(),
            account_id: w.rahim,
            person_key: None,
            amount: "4,316".into(),
            removed: Default::default(),
            reason: String::new(),
            narration: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    rig_for(w, None)
}

/// A quick world with one receipt for tests that need a book but not the whole day.
pub fn small_world() -> Rig {
    rig_for(support::world(), None)
}

/// A raw read connection to the live data file.
pub fn raw(rig: &Rig) -> Connection {
    Connection::open(rig.w.dir.path().join("data").join("mandi.db")).unwrap()
}

/// Everything a restore must give back exactly: the balance of every account, every journal
/// entry with its chain hash, and the whole audit log with its chain.
#[derive(Debug, PartialEq, Eq)]
pub struct Fingerprint {
    pub balances: Vec<(String, i64, i64)>,
    pub journal: Vec<(i64, String, String)>,
    pub audit: Vec<(i64, String)>,
    pub lines: i64,
    pub bikri: i64,
}

pub fn fingerprint(rig: &Rig) -> Fingerprint {
    let conn = raw(rig);
    let balances = conn
        .prepare(
            "SELECT a.code, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0) \
             FROM accounts a LEFT JOIN journal_lines l ON l.account_id = a.id GROUP BY a.id ORDER BY a.code",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let journal = conn
        .prepare("SELECT id, entry_date, COALESCE(chain_hash, '') FROM journal_entries ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let audit = conn
        .prepare("SELECT id, chain_hash FROM audit_log ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    Fingerprint {
        balances,
        journal,
        audit,
        lines: conn
            .query_row("SELECT COUNT(*) FROM journal_lines", [], |r| r.get(0))
            .unwrap(),
        bikri: conn
            .query_row("SELECT COUNT(*) FROM bikri_entries", [], |r| r.get(0))
            .unwrap(),
    }
}

pub fn first_ok(report: &RunReport, kind: BackupKind) -> bool {
    report.results.iter().any(|r| r.kind == kind && r.ok)
}
