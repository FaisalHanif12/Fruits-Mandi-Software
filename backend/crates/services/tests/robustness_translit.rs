//! Learned words under a crash (D2 attack list item 2; PRD FR-7.26, NFR-URD-26, NFR-REL-1).
//! Owner: `robustness`.
//!
//! The app process is killed (SIGKILL, no clean-up) straight after it has learned a word, many
//! times in a row on the same data file. After each restart:
//!
//! - every word the child reported as learned (it printed the line only after `learn` returned)
//!   is in the table and is offered first from the very next word typed (a restart loads the
//!   table again);
//! - a pair and its audit row are never apart, the audit chain verifies, the file checks out;
//! - no pair is half written (both spellings, a use count of at least 1).
//!
//! A kill of the process does not lose what the operating system already has, so this is not a
//! power cut. With `synchronous = FULL` and WAL (checked in `robustness_fault.rs`) a power cut
//! gives the same result; the real power-cut run is on the Windows list.
#![cfg(test)]

mod common;

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use common::*;
use mandi_services::core::translit::{Direction, Kind};
use mandi_services::testing::TestEnv;
use mandi_services::translit::{self, NewPair};

const CHILD_DIR: &str = "MANDI_ROBUST_TRANSLIT_DIR";
const CHILD_ROUND: &str = "MANDI_ROBUST_TRANSLIT_ROUND";
const ROUNDS: u64 = 15;

/// An English word of letters only that no real word and no earlier round uses.
fn roman_word(round: u64, n: u64) -> String {
    let mut value = round * 10_000 + n;
    let mut out = String::from("zq");
    loop {
        out.push(char::from(b'a' + u8::try_from(value % 26).unwrap_or(0)));
        value /= 26;
        if value == 0 {
            return out;
        }
    }
}

/// An Urdu word of letters only, different for every (round, n).
fn urdu_word(round: u64, n: u64) -> String {
    const LETTERS: [char; 16] = [
        'ب', 'ت', 'ج', 'د', 'ر', 'ز', 'س', 'ش', 'ع', 'ف', 'ق', 'ک', 'ل', 'م', 'ن', 'و',
    ];
    let mut value = round * 10_000 + n;
    let mut out = String::from("ٹ");
    loop {
        out.push(LETTERS[usize::try_from(value % 16).unwrap_or(0)]);
        value /= 16;
        if value == 0 {
            return out;
        }
    }
}

fn say(text: &str) {
    let mut out = std::io::stdout().lock();
    writeln!(out, "\n@@{text}").unwrap();
    out.flush().unwrap();
}

/// Not a test. The parent starts this same binary with `CHILD_DIR` set and kills it. Run on its
/// own it does nothing.
#[test]
fn child_process_for_the_learn_kill() {
    let Ok(dir) = std::env::var(CHILD_DIR) else {
        return;
    };
    let round: u64 = std::env::var(CHILD_ROUND)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let env = TestEnv::new(Path::new(&dir)).unwrap();
    login_owner(&env);
    let owner = mandi_services::auth::require(&env.ctx).unwrap();
    say("READY");
    let mut n = 0_u64;
    loop {
        n += 1;
        let pair = NewPair {
            roman: roman_word(round, n),
            urdu: urdu_word(round, n),
            kind: Kind::Name,
        };
        // Every third word is learned again, so a use count going up is hit by the kill too.
        let pair = if n.is_multiple_of(3) {
            NewPair {
                roman: roman_word(round, n - 1),
                urdu: urdu_word(round, n - 1),
                kind: Kind::Name,
            }
        } else {
            pair
        };
        translit::learn(&env.ctx, &owner, &pair).unwrap();
        say(&format!("ACK {} {}", pair.roman, pair.urdu));
    }
}

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        (z ^ (z >> 31)) % n.max(1)
    }
}

#[test]
fn killing_the_process_right_after_a_word_is_learned_never_loses_it() {
    let (dir, env) = env();
    set_up(&env);
    drop(env);
    let mut rng = Rng(0x7E57_0001);
    let mut all_acked: Vec<(String, String)> = Vec::new();
    let mut kills_in_flight = 0_u32;
    for round in 0..ROUNDS {
        let mut process = Command::new(std::env::current_exe().unwrap())
            .args([
                "child_process_for_the_learn_kill",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_DIR, dir.path())
            .env(CHILD_ROUND, round.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let out = process.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if let Some(line) = line.strip_prefix("@@")
                    && tx.send(line.to_owned()).is_err()
                {
                    break;
                }
            }
        });
        // Wait for the child to be ready, then for a few learned words, then kill at once.
        let want = 1 + rng.below(25);
        let mut acked: Vec<(String, String)> = Vec::new();
        let mut seen = 0_u64;
        while seen < want {
            match lines.recv_timeout(Duration::from_secs(60)) {
                Ok(line) => {
                    if let Some(rest) = line.strip_prefix("ACK ") {
                        let mut parts = rest.split(' ');
                        if let (Some(r), Some(u)) = (parts.next(), parts.next()) {
                            acked.push((r.to_owned(), u.to_owned()));
                        }
                        seen += 1;
                    }
                }
                Err(_) => panic!("round {round}: the child printed nothing for 60 seconds"),
            }
        }
        let _ = process.kill();
        let _ = process.wait();
        // Whatever it printed before it died was learned before it was printed.
        let mut late = 0;
        while let Ok(line) = lines.recv_timeout(Duration::from_millis(300)) {
            if let Some(rest) = line.strip_prefix("ACK ") {
                let mut parts = rest.split(' ');
                if let (Some(r), Some(u)) = (parts.next(), parts.next()) {
                    acked.push((r.to_owned(), u.to_owned()));
                    late += 1;
                }
            }
        }
        if late > 0 {
            kills_in_flight += 1;
        }

        // Restart: a new context on the same file, as the app does at start-up.
        let env = TestEnv::new(dir.path()).unwrap_or_else(|e| {
            panic!("round {round}: the file does not open after a kill: {e:?}")
        });
        login_owner(&env);
        let rows = translit::list(&env.ctx).unwrap();
        for (roman, urdu) in &acked {
            let row = rows
                .iter()
                .find(|r| &r.roman == roman && &r.urdu == urdu)
                .unwrap_or_else(|| {
                    panic!("round {round}: {roman} -> {urdu} was learned but is not in the table")
                });
            assert!(
                row.use_count >= 1,
                "round {round}: {roman} has no use count"
            );
            // Offered first from the very next word typed after the restart.
            let first = translit::suggest(&env.ctx, roman, Direction::ToUrdu).unwrap();
            assert_eq!(
                first.first().map(|s| s.text.as_str()),
                Some(urdu.as_str()),
                "round {round}: {roman} is not offered first after the restart"
            );
        }
        // No pair without its audit row, none half written.
        let pairs: i64 = raw(&dir)
            .query_row("SELECT COUNT(*) FROM translit_pairs", [], |r| r.get(0))
            .unwrap();
        let broken: i64 = raw(&dir)
            .query_row(
                "SELECT COUNT(*) FROM translit_pairs WHERE roman = '' OR urdu = '' OR use_count < 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(broken, 0, "round {round}: a pair is half written");
        let created: i64 = raw(&dir)
            .query_row(
                "SELECT COUNT(DISTINCT entity_id) FROM audit_log WHERE action = 'learn' AND entity = 'translit_pair'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            created, pairs,
            "round {round}: {pairs} pairs but {created} of them have an audit row"
        );
        assert_chain_ok(&dir);
        let integrity: String = raw(&dir)
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok", "round {round}");
        all_acked.extend(acked);
    }
    println!(
        "{ROUNDS} kills, {} learned words reported, {kills_in_flight} kills with a word in flight",
        all_acked.len()
    );
    // Every word of every round is still there at the end.
    let env = TestEnv::new(dir.path()).unwrap();
    let rows = translit::list(&env.ctx).unwrap();
    for (roman, urdu) in &all_acked {
        assert!(
            rows.iter().any(|r| &r.roman == roman && &r.urdu == urdu),
            "{roman} -> {urdu} is gone at the end"
        );
    }
}
