//! Lists the Fluent files of the screens so the Excel export can use the same words (Tech Stack
//! 4.2: the backend reads the same `.ftl` files, embedded at build time from
//! `frontend/src/i18n/`). It writes `ftl_files.rs` into the build folder: one `include_str!` for
//! every file of `ur/` and of `en/`. A new message file is picked up on the next build, with no
//! list to keep in step.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

fn files_of(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "ftl"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

fn list(name: &str, dir: &Path) -> String {
    let mut text = format!("pub(crate) const {name}: &[(&str, &str)] = &[\n");
    for path in files_of(dir) {
        let file = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let absolute = path.to_string_lossy().replace('\\', "/");
        let _ = writeln!(text, "    ({file:?}, include_str!({absolute:?})),");
    }
    text.push_str("];\n");
    text
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    // backend/crates/reports -> the repository root is three folders up.
    let i18n = manifest.join("../../../frontend/src/i18n");
    // Cargo reads these lines from standard output. `writeln!` keeps the print lints quiet.
    let mut cargo = std::io::stdout();
    let _ = writeln!(cargo, "cargo:rerun-if-changed=build.rs");
    let _ = writeln!(cargo, "cargo:rerun-if-changed={}", i18n.display());
    let mut out = String::new();
    out.push_str(&list("UR", &i18n.join("ur")));
    out.push_str(&list("EN", &i18n.join("en")));
    let target = PathBuf::from(env::var("OUT_DIR").unwrap_or_default()).join("ftl_files.rs");
    if let Err(error) = fs::write(&target, out) {
        let _ = writeln!(
            cargo,
            "cargo:warning=cannot write {}: {error}",
            target.display()
        );
    }
}
