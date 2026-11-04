//! The developer profile (plan 56, R5): a way to run the app on a developer's computer without
//! touching the real data.
//!
//! With the environment variable `MANDI_PROFILE=<name>` set, a DEBUG build uses
//!
//! - a separate data folder, `<app data>/profiles/<name>/`, instead of `<app data>/`, and
//! - separate Keychain or Credential Manager entries: the service name `pk.mandisoftware.app`
//!   becomes `pk.mandisoftware.app.<name>` (the backup key, the cloud keys, the chain key of the
//!   journal and audit and the OpenRouter key).
//!
//! A RELEASE build ignores the variable completely, so the client's installed app can never be
//! moved onto another folder or other keys by a stray variable.
//!
//! The name is checked: 1 to 32 characters of `a-z`, `0-9` and `-`, starting with a letter or a
//! digit. A name that fails the check never falls back to the real data: the app refuses to
//! start (`ProfileError`), and any code that asks for a service name anyway gets one that no real
//! entry has (`pk.mandisoftware.app.invalid-profile`).
//!
//! Example: `MANDI_PROFILE=r5` gives the folder `<app data>/profiles/r5` and the service
//! `pk.mandisoftware.app.r5`.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The environment variable that switches the profile on (debug builds only).
pub const PROFILE_VARIABLE: &str = "MANDI_PROFILE";
/// The Credential Manager (Keychain) service name of the real app (Tech Stack 7).
pub const KEYRING_BASE: &str = "pk.mandisoftware.app";
/// The folder inside the app data folder that holds the profiles.
pub const PROFILES_FOLDER: &str = "profiles";
/// The most characters of a profile name.
pub const NAME_MAX: usize = 32;
/// The service-name suffix used when the variable holds a bad name. No real entry has it.
pub const INVALID_SUFFIX: &str = "invalid-profile";

/// The variable holds a name that is not allowed. The app does not start.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "MANDI_PROFILE must be 1 to 32 characters of a-z, 0-9 and '-', starting with a letter or a digit"
)]
pub struct ProfileError;

/// Checks a profile name: 1 to [`NAME_MAX`] characters of `a-z`, `0-9` and `-`, starting with a
/// letter or a digit.
pub fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let starts_well = chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    starts_well
        && name.len() <= NAME_MAX
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The profile a value of the variable asks for. `dev_build` is false in a release build, which
/// ignores the variable. An unset or empty variable is no profile.
///
/// # Errors
/// [`ProfileError`] for a name that is not allowed (in a debug build).
pub fn parse(value: Option<&str>, dev_build: bool) -> Result<Option<String>, ProfileError> {
    if !dev_build {
        return Ok(None);
    }
    match value {
        None | Some("") => Ok(None),
        Some(name) if is_valid_name(name) => Ok(Some(name.to_owned())),
        Some(_) => Err(ProfileError),
    }
}

/// The service name of the Credential Manager for a profile: the base name, or the base name, a
/// dot and the profile.
pub fn service_for(base: &str, profile: Option<&str>) -> String {
    match profile {
        Some(name) => format!("{base}.{name}"),
        None => base.to_owned(),
    }
}

/// The data folder for a profile: the app data folder itself, or `<app data>/profiles/<name>`.
pub fn data_dir_for(app_data: &Path, profile: Option<&str>) -> PathBuf {
    match profile {
        Some(name) => app_data.join(PROFILES_FOLDER).join(name),
        None => app_data.to_path_buf(),
    }
}

/// What this process read from the environment, once.
fn read_environment() -> Result<Option<String>, ProfileError> {
    static CURRENT: OnceLock<Result<Option<String>, ProfileError>> = OnceLock::new();
    CURRENT
        .get_or_init(|| match std::env::var_os(PROFILE_VARIABLE) {
            None => Ok(None),
            Some(raw) => match raw.to_str() {
                Some(text) => parse(Some(text), cfg!(debug_assertions)),
                // Not text at all: in a release build the variable is ignored like any other.
                None if cfg!(debug_assertions) => Err(ProfileError),
                None => Ok(None),
            },
        })
        .clone()
}

/// The profile of this process: from `MANDI_PROFILE` in a debug build, none in a release build.
/// The app calls this first thing and stops on an error.
///
/// # Errors
/// [`ProfileError`] when the variable holds a name that is not allowed.
pub fn current() -> Result<Option<String>, ProfileError> {
    read_environment()
}

/// The Credential Manager (Keychain) service name every secret of this process uses. Each crate
/// that stores a secret asks for it here and never writes the name itself.
pub fn keyring_service() -> String {
    match current() {
        Ok(profile) => service_for(KEYRING_BASE, profile.as_deref()),
        Err(ProfileError) => service_for(KEYRING_BASE, Some(INVALID_SUFFIX)),
    }
}

/// The data folder of this process below the app data folder.
///
/// # Errors
/// [`ProfileError`] when the variable holds a name that is not allowed.
pub fn data_dir(app_data: &Path) -> Result<PathBuf, ProfileError> {
    Ok(data_dir_for(app_data, current()?.as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_names_pass_and_bad_names_do_not() {
        for good in ["r5", "demo", "a", "0", "r-5", "a-b-c", &"x".repeat(32)] {
            assert!(is_valid_name(good), "{good}");
        }
        for bad in [
            "",
            "R5",
            "-r5",
            "r 5",
            "r5/",
            "../x",
            "r5\u{0}",
            "\u{627}",
            "r_5",
            "r.5",
            &"x".repeat(33),
        ] {
            assert!(!is_valid_name(bad), "{bad:?}");
        }
    }

    #[test]
    fn a_debug_build_takes_the_name_and_a_release_build_ignores_it() {
        assert_eq!(parse(Some("r5"), true), Ok(Some("r5".to_owned())));
        assert_eq!(parse(Some("r5"), false), Ok(None));
        // Even a bad name does nothing in a release build.
        assert_eq!(parse(Some("../../etc"), false), Ok(None));
        assert_eq!(parse(None, true), Ok(None));
        assert_eq!(parse(Some(""), true), Ok(None));
        assert_eq!(parse(Some("../../etc"), true), Err(ProfileError));
        assert_eq!(parse(Some("R5"), true), Err(ProfileError));
    }

    #[test]
    fn a_profile_has_other_service_names_than_the_real_app() {
        assert_eq!(service_for(KEYRING_BASE, None), "pk.mandisoftware.app");
        assert_eq!(
            service_for(KEYRING_BASE, Some("r5")),
            "pk.mandisoftware.app.r5"
        );
        // No profile can ever make the real name, whatever it is called.
        for name in ["r5", "a", "invalid-profile", "0"] {
            assert_ne!(service_for(KEYRING_BASE, Some(name)), KEYRING_BASE);
        }
        // The name made for a bad variable is not one a valid profile can make either, except
        // by being called exactly that: it is still never the real name.
        assert_ne!(
            service_for(KEYRING_BASE, Some(INVALID_SUFFIX)),
            KEYRING_BASE
        );
    }

    #[test]
    fn a_profile_has_its_own_data_folder_below_the_real_one() {
        let base = Path::new("/app-data/pk.mandisoftware.app");
        assert_eq!(data_dir_for(base, None), base);
        let folder = data_dir_for(base, Some("r5"));
        assert_eq!(folder, base.join("profiles").join("r5"));
        assert!(folder.starts_with(base));
        assert_ne!(folder, base);
        // The real data file lives at `<base>/data/mandi.db`; the profile's does not.
        assert_ne!(
            folder.join("data").join("mandi.db"),
            base.join("data").join("mandi.db")
        );
    }

    #[test]
    fn the_variable_is_read_once_and_the_default_is_the_real_app() {
        // The test run itself has no MANDI_PROFILE (the child test in the app crate sets it).
        if std::env::var_os(PROFILE_VARIABLE).is_none() {
            assert_eq!(current(), Ok(None));
            assert_eq!(keyring_service(), KEYRING_BASE);
            let base = Path::new("/x");
            assert_eq!(data_dir(base), Ok(base.to_path_buf()));
        }
    }
}
