//! The developer profile (plan 56, R5): a run with `MANDI_PROFILE=r5` must leave the real data
//! folder and the real Keychain or Credential Manager entries untouched.
//!
//! The proof is in names and paths, not in touching the real store:
//!
//! 1. A child process (this test binary again) runs with `MANDI_PROFILE=r5`. It checks the
//!    profile, the service name of every store that keeps a secret, the data folder, and that a
//!    data file opened there lands only below `profiles/r5/`.
//! 2. A child without the variable gets the real names, and a child with a bad name gets a
//!    refusal and a service name no real entry has.
//! 3. A scan of the source: every `Entry::new(` asks for the service name from one function,
//!    no source file but the profile module holds the real service name, and no code but the
//!    profile helper asks Tauri for the app data folder. A new store that forgets the profile
//!    fails here.
//!
//! A release build ignores the variable: `mandi_core::profile::parse(.., false)` (its unit test).
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mandi_app_lib::profile;
use mandi_assistant::keys::KeyringStore;
use mandi_backup::secrets::KeyringSecretStore;
use mandi_services::core::profile as names;
use mandi_services::db::audit::KeyringKeyStore;

const CHILD_FLAG: &str = "MANDI_PROFILE_CHILD";
const REAL: &str = "pk.mandisoftware.app";

/// Runs one test of this binary again as a child, with or without a profile.
fn run_child(test: &str, profile: Option<&str>) -> String {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(CHILD_FLAG, "1");
    match profile {
        Some(value) => command.env("MANDI_PROFILE", value),
        None => command.env_remove("MANDI_PROFILE"),
    };
    let output = command.output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "the child failed:\n{text}");
    assert!(
        text.contains("CHILD CHECKED"),
        "the child did not run its checks:\n{text}"
    );
    text
}

fn is_child() -> bool {
    std::env::var_os(CHILD_FLAG).is_some()
}

// ---- the three children ----

#[test]
fn child_with_profile_r5() {
    if !is_child() {
        return;
    }
    assert_eq!(profile::current().unwrap().as_deref(), Some("r5"));
    // Every store that keeps a secret uses the service of the profile.
    let service = format!("{REAL}.r5");
    assert_eq!(names::keyring_service(), service);
    assert_eq!(KeyringKeyStore::service(), service, "the chain key");
    assert_eq!(
        KeyringSecretStore::service(),
        service,
        "backup and cloud keys"
    );
    assert_ne!(names::keyring_service(), REAL);

    // The data folder is below `profiles/r5` and never the real one.
    let real = tempfile::tempdir().unwrap();
    let folder = profile::data_dir_in(real.path()).unwrap();
    assert_eq!(folder, real.path().join("profiles").join("r5"));
    assert_ne!(folder, real.path());
    // A data file opened there lands only below the profile; the real folder gets nothing but
    // the `profiles` folder that holds it.
    let db_path = folder.join("data").join("mandi.db");
    let options = mandi_services::db::DbOptions {
        key_store: std::sync::Arc::new(mandi_services::db::audit::MemoryKeyStore::new(
            mandi_services::db::audit::AuditKey::from_bytes([1; 32]),
        )),
        ..mandi_services::db::DbOptions::new(folder.join("backups"))
    };
    let _db = mandi_services::db::Db::open_checked(&db_path, options).unwrap();
    assert!(db_path.exists());
    assert!(!real.path().join("data").exists(), "the real data folder");
    assert!(!real.path().join("backups").exists());
    let top: Vec<String> = fs::read_dir(real.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        top,
        ["profiles"],
        "only the profiles folder is in the real folder"
    );

    // The window position and the single instance do not touch the real app's either.
    assert_eq!(
        profile::window_state_file(Some("r5")),
        ".window-state-r5.json"
    );
    assert!(!profile::uses_single_instance(Some("r5")));
    println!("CHILD CHECKED");
}

#[test]
fn child_without_a_profile() {
    if !is_child() {
        return;
    }
    assert_eq!(profile::current().unwrap(), None);
    assert_eq!(names::keyring_service(), REAL);
    assert_eq!(KeyringKeyStore::service(), REAL);
    assert_eq!(KeyringSecretStore::service(), REAL);
    assert_eq!(KeyringStore::service(), REAL);
    let real = Path::new("/app-data/pk.mandisoftware.app");
    assert_eq!(profile::data_dir_in(real).unwrap(), real);
    assert_eq!(profile::window_state_file(None), ".window-state.json");
    assert!(profile::uses_single_instance(None));
    println!("CHILD CHECKED");
}

#[test]
fn child_with_a_bad_profile_name() {
    if !is_child() {
        return;
    }
    // The app refuses to start; nothing falls back to the real data.
    assert!(profile::current().is_err());
    assert!(profile::data_dir_in(Path::new("/app-data")).is_err());
    // Any code that asks for a service name anyway gets one that no real entry has.
    let service = names::keyring_service();
    assert_eq!(service, format!("{REAL}.invalid-profile"));
    assert_ne!(service, REAL);
    assert_eq!(KeyringKeyStore::service(), service);
    assert_eq!(KeyringSecretStore::service(), service);
    assert_eq!(KeyringStore::service(), service);
    println!("CHILD CHECKED");
}

// ---- the parents ----

#[test]
fn a_profile_run_has_its_own_names_and_folders() {
    if is_child() {
        return;
    }
    run_child("child_with_profile_r5", Some("r5"));
}

#[test]
fn a_run_without_a_profile_uses_the_real_names() {
    if is_child() {
        return;
    }
    run_child("child_without_a_profile", None);
    // An empty variable is no profile either.
    run_child("child_without_a_profile", Some(""));
}

#[test]
fn a_bad_profile_name_stops_the_app_and_reaches_no_real_entry() {
    if is_child() {
        return;
    }
    for bad in ["../real", "R5", "r 5", "r5/..", "-x"] {
        run_child("child_with_a_bad_profile_name", Some(bad));
    }
}

// ---- the scan ----

fn backend_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Every non-test Rust source file of the workspace: `crates/*/src` and `app/src`.
fn source_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    let backend = backend_dir();
    walk(&backend.join("app").join("src"), &mut out);
    if let Ok(crates) = fs::read_dir(backend.join("crates")) {
        for krate in crates.flatten() {
            walk(&krate.path().join("src"), &mut out);
        }
    }
    out
}

/// The code of a file without its `#[cfg(test)] mod tests` tail and without comment lines.
fn real_code(text: &str) -> String {
    let cut = text.find("#[cfg(test)]").unwrap_or(text.len());
    text[..cut]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_keychain_entry_takes_its_service_name_from_the_profile_function() {
    let mut seen = 0;
    for path in source_files() {
        let text = fs::read_to_string(&path).unwrap();
        let code = real_code(&text);
        // `Entry::new(` may span lines: look at the text up to the closing bracket's line.
        for (at, _) in code.match_indices("Entry::new(") {
            seen += 1;
            let call: String = code[at..].chars().take(120).collect();
            assert!(
                call.contains("service()") || call.contains("keyring_service()"),
                "{} makes a Keychain entry with a service name that is not from the profile \
                 function: {call}",
                path.display()
            );
        }
    }
    assert!(seen >= 3, "the scan found the Keychain entries ({seen})");
}

#[test]
fn only_the_profile_module_holds_the_real_service_name_and_the_app_data_folder() {
    let allowed_name = backend_dir()
        .join("crates")
        .join("core")
        .join("src")
        .join("profile.rs");
    let allowed_dir = backend_dir().join("app").join("src").join("profile.rs");
    for path in source_files() {
        let text = fs::read_to_string(&path).unwrap();
        let code = real_code(&text);
        if path != allowed_name {
            assert!(
                !code.contains(&format!("\"{REAL}\"")),
                "{} holds the real service name; use mandi_core::profile",
                path.display()
            );
        }
        if path != allowed_dir {
            assert!(
                !code.contains("app_data_dir()"),
                "{} asks Tauri for the app data folder; use crate::profile::app_data_dir",
                path.display()
            );
        }
    }
}
