//! The support information file (plan 22 early, NFR-OBS-4, NFR-OBS-2): one plain text file with
//! the app version, the system, the state of the books, the backups and the last lines of the
//! log, that the client can send to support.
//!
//! It never holds a password, a key, a recovery code, a token or a balance:
//!
//! - the facts in it are versions, counts, dates and yes/no states only;
//! - the log lines come from the log files, where such fields are already written as
//!   `[hidden]` (`logging`); every line is checked again here ([`scrub_line`]) and any such
//!   field that still has a value is hidden.
//!
//! Example: the line `login ok password=hunter2 user=owner` becomes
//! `login ok password=[hidden] user=owner`.

use std::fs;
use std::io;
use std::path::Path;

use crate::logging::{HIDDEN, LOG_FILE_PREFIX, is_secret_field};

/// Most log lines in the file.
pub const MAX_LOG_LINES: usize = 2_000;
/// Most bytes read from the log files.
pub const MAX_LOG_BYTES: u64 = 400 * 1024;

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.'
}

/// One log line with the value of every secret field hidden (a name that holds `password`,
/// `key`, `secret`, `token`, `recovery_code` or `balance`, as in `logging::is_secret_field`).
pub fn scrub_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0_usize;
    while let Some(&c) = chars.get(i) {
        if !is_ident(c) {
            out.push(c);
            i += 1;
            continue;
        }
        let start = i;
        while chars.get(i).is_some_and(|c| is_ident(*c)) {
            i += 1;
        }
        let word: String = chars.get(start..i).unwrap_or_default().iter().collect();
        out.push_str(&word);
        if chars.get(i) == Some(&'=') && is_secret_field(&word) {
            out.push('=');
            i += 1;
            if chars.get(i) == Some(&'"') {
                // A quoted value, with \" inside.
                i += 1;
                while let Some(&inner) = chars.get(i) {
                    i += 1;
                    if inner == '\\' {
                        i += 1;
                    } else if inner == '"' {
                        break;
                    }
                }
            } else {
                while chars.get(i).is_some_and(|c| !c.is_whitespace()) {
                    i += 1;
                }
            }
            out.push_str(HIDDEN);
        }
    }
    out
}

/// The last `max_lines` lines of the log files in `dir`, oldest first, each scrubbed. Reads at
/// most `max_bytes` from the end of the newest files. A missing folder gives no lines.
///
/// # Errors
/// An I/O error when the folder exists but cannot be read.
pub fn log_tail(dir: &Path, max_lines: usize, max_bytes: u64) -> io::Result<Vec<String>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.metadata().is_ok_and(|m| m.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(LOG_FILE_PREFIX))
        .collect();
    // `mandi.log.YYYY-MM-DD` sorts by day; the newest comes last.
    names.sort();
    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut lines = 0_usize;
    let mut budget = max_bytes;
    for name in names.iter().rev() {
        if lines >= max_lines || budget == 0 {
            break;
        }
        let bytes = fs::read(dir.join(name))?;
        let len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        let skip = usize::try_from(len.saturating_sub(budget)).unwrap_or(0);
        budget = budget.saturating_sub(len);
        let text = String::from_utf8_lossy(bytes.get(skip..).unwrap_or_default()).into_owned();
        let mut file_lines: Vec<String> = text.lines().map(scrub_line).collect();
        // Cut at a line start when the file was cut in the middle of a line.
        if skip > 0 && !file_lines.is_empty() {
            file_lines.remove(0);
        }
        lines = lines.saturating_add(file_lines.len());
        chunks.push(file_lines);
    }
    chunks.reverse();
    let mut all: Vec<String> = chunks.into_iter().flatten().collect();
    if all.len() > max_lines {
        all.drain(..all.len() - max_lines);
    }
    Ok(all)
}

/// One group of facts of the file.
pub struct Section {
    /// The heading.
    pub title: String,
    /// The facts, name then value.
    pub facts: Vec<(String, String)>,
}

/// Everything the file holds.
pub struct SupportParts {
    /// The groups of facts, in order.
    pub sections: Vec<Section>,
    /// The log lines, oldest first, already scrubbed.
    pub log_lines: Vec<String>,
}

/// The text of the file.
pub fn render(parts: &SupportParts) -> String {
    let mut out = String::new();
    out.push_str("Mandi Software support information\n");
    out.push_str("Passwords, keys, recovery codes and balances are never in this file.\n");
    for section in &parts.sections {
        out.push_str(&format!("\n[{}]\n", section.title));
        for (name, value) in &section.facts {
            // A value is one line.
            let value = value.replace(['\r', '\n'], " ");
            out.push_str(&format!("{name}: {value}\n"));
        }
    }
    out.push_str(&format!(
        "\n[log] the last {} lines, oldest first\n",
        parts.log_lines.len()
    ));
    for line in &parts.log_lines {
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_field_with_a_value_is_hidden() {
        assert_eq!(
            scrub_line("login ok password=hunter2 user=owner"),
            "login ok password=[hidden] user=owner"
        );
        assert_eq!(
            scrub_line(r#"saved audit_key="a b \" c" next=1 balance_paisa=120000"#),
            "saved audit_key=[hidden] next=1 balance_paisa=[hidden]"
        );
        // Already hidden stays hidden; other fields and plain words are untouched.
        assert_eq!(
            scrub_line("x token=[hidden] rows=5"),
            "x token=[hidden] rows=5"
        );
        assert_eq!(
            scrub_line("password is not a field here"),
            "password is not a field here"
        );
        assert_eq!(scrub_line("رحیم کا بل safe=1"), "رحیم کا بل safe=1");
    }

    #[test]
    fn the_tail_is_the_newest_lines_in_order_and_scrubbed() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("mandi.log.2026-10-09"), "a1\na2 secret=x\n").unwrap();
        fs::write(dir.path().join("mandi.log.2026-10-10"), "b1\nb2\nb3\n").unwrap();
        fs::write(dir.path().join("other.txt"), "not a log\n").unwrap();
        let all = log_tail(dir.path(), 100, 1_000_000).unwrap();
        assert_eq!(all, ["a1", "a2 secret=[hidden]", "b1", "b2", "b3"]);
        let last3 = log_tail(dir.path(), 3, 1_000_000).unwrap();
        assert_eq!(last3, ["b1", "b2", "b3"]);
        assert!(
            log_tail(&dir.path().join("none"), 10, 100)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn the_byte_limit_cuts_old_files_and_half_lines() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("mandi.log.2026-10-09"), "old1\nold2\n").unwrap();
        fs::write(dir.path().join("mandi.log.2026-10-10"), "new1\nnew2\n").unwrap();
        // Room for the newest file and 3 bytes of the older one: the half line is dropped.
        let tail = log_tail(dir.path(), 100, 11).unwrap();
        assert_eq!(tail, ["new1", "new2"]);
    }

    #[test]
    fn the_file_has_the_sections_and_the_log() {
        let text = render(&SupportParts {
            sections: vec![Section {
                title: "app".into(),
                facts: vec![("version".into(), "0.1.0".into())],
            }],
            log_lines: vec!["one".into(), "two".into()],
        });
        assert!(text.contains("[app]\nversion: 0.1.0\n"));
        assert!(text.contains("[log] the last 2 lines, oldest first\none\ntwo\n"));
    }
}
