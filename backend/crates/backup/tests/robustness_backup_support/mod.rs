//! Shared by the `robustness_backup_*.rs` tests: the result table and the fingerprint of a data
//! file. Equal fingerprints mean the same books: every table's row count, every account balance,
//! the trial balance, the partner share rows, both hash chains row by row, the settings, the
//! learned words, the days (closed up to and the close history) and the question log.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use mandi_services::db::audit::{AuditKey, verify_audit_chain};
use mandi_services::db::ledger::verify_journal_chain;
use mandi_services::db::rusqlite::{Connection, OpenFlags};

// ---- the result table --------------------------------------------------------------------------

#[derive(Default)]
pub struct Report(pub Vec<(String, bool, String)>);

impl Report {
    pub fn check(&mut self, name: &str, ok: bool, evidence: impl Into<String>) {
        let evidence = evidence.into();
        println!(
            "| {name} | {} | {evidence} |",
            if ok { "PASS" } else { "FAIL" }
        );
        self.0.push((name.to_owned(), ok, evidence));
    }

    pub fn finish(&self) {
        println!("\n| check | result | evidence |\n|---|---|---|");
        for (name, ok, evidence) in &self.0 {
            println!(
                "| {name} | {} | {evidence} |",
                if *ok { "PASS" } else { "FAIL" }
            );
        }
        let bad: Vec<&str> = self
            .0
            .iter()
            .filter(|c| !c.1)
            .map(|c| c.0.as_str())
            .collect();
        assert!(bad.is_empty(), "checks that failed: {bad:?}");
    }
}

// ---- the fingerprint of a database file --------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fp {
    pub counts: BTreeMap<String, i64>,
    pub balances: BTreeMap<String, i64>,
    pub debit: i64,
    pub credit: i64,
    pub partner_rows: BTreeMap<String, i64>,
    pub journal_chain: Vec<(i64, String)>,
    pub audit_chain: Vec<(i64, String)>,
    pub settings: BTreeMap<String, String>,
    pub learned: Vec<(String, String, i64)>,
    /// The rows of the day tables and of the question log, as text.
    pub days: Vec<String>,
    pub questions: Vec<String>,
    pub chains_ok: Option<bool>,
}

pub fn open_ro(path: &Path) -> Connection {
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("open the data file read-only")
}

fn rows_as_text(conn: &Connection, table: &str) -> Vec<String> {
    let mut st = conn
        .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY rowid"))
        .unwrap();
    let columns = st.column_count();
    let mut rows = st.query([]).unwrap();
    let mut out = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let mut text = String::new();
        for i in 0..columns {
            let v: mandi_services::db::rusqlite::types::Value = row.get(i).unwrap();
            text.push_str(&format!("{v:?}|"));
        }
        out.push(text);
    }
    out
}

pub fn fingerprint(path: &Path, key: Option<&AuditKey>) -> Fp {
    let conn = open_ro(path);
    let tables: Vec<String> = {
        let mut s = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap();
        s.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let mut counts = BTreeMap::new();
    for table in &tables {
        let n: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM \"{table}\""), [], |r| r.get(0))
            .unwrap();
        counts.insert(table.clone(), n);
    }
    let pairs = |sql: &str| -> BTreeMap<String, i64> {
        let mut s = conn.prepare(sql).unwrap();
        s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let balances = pairs(
        "SELECT a.code, COALESCE((SELECT SUM(l.dr_paisa) - SUM(l.cr_paisa) FROM journal_lines l \
         JOIN journal_entries e ON e.id = l.entry_id WHERE l.account_id = a.id AND e.status = 'posted'), 0) \
         FROM accounts a ORDER BY a.code",
    );
    let (debit, credit): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l \
             JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let partner_rows = pairs(
        "SELECT partner_id || ':' || kind, SUM(amount_paisa) FROM partner_share_rows GROUP BY partner_id, kind",
    );
    let chain = |sql: &str| -> Vec<(i64, String)> {
        let mut s = conn.prepare(sql).unwrap();
        s.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let journal_chain = chain("SELECT id, COALESCE(chain_hash, '') FROM journal_entries ORDER BY id");
    let audit_chain = chain("SELECT id, COALESCE(chain_hash, '') FROM audit_log ORDER BY id");
    let settings: BTreeMap<String, String> = {
        let mut s = conn
            .prepare("SELECT key, value_json FROM settings ORDER BY key")
            .unwrap();
        s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let learned: Vec<(String, String, i64)> = {
        let mut s = conn
            .prepare("SELECT roman, urdu, use_count FROM translit_pairs ORDER BY roman, urdu")
            .unwrap();
        s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let mut days = rows_as_text(&conn, "day_state");
    days.extend(rows_as_text(&conn, "day_closes"));
    let questions = rows_as_text(&conn, "assistant_questions");
    let chains_ok = key.map(|k| {
        verify_audit_chain(&conn, k).is_ok() && verify_journal_chain(&conn, k).is_ok()
    });
    Fp {
        counts,
        balances,
        debit,
        credit,
        partner_rows,
        journal_chain,
        audit_chain,
        settings,
        learned,
        days,
        questions,
        chains_ok,
    }
}

/// What differs between the books before and after. `audit_extra` is how many audit rows the
/// restore itself adds (its own row); the old rows must be unchanged. `skip_settings` names
/// settings rows that may legitimately differ (the backup row of a new computer).
pub fn differences(
    before: &Fp,
    after: &Fp,
    audit_extra: usize,
    skip_settings: &[&str],
) -> Vec<String> {
    let mut out = Vec::new();
    for table in before.counts.keys().chain(after.counts.keys()) {
        let (a, b) = (before.counts.get(table), after.counts.get(table));
        let want = if table == "audit_log" {
            a.map(|n| n + i64::try_from(audit_extra).unwrap())
        } else {
            a.copied()
        };
        if want != b.copied() {
            out.push(format!("rows of {table}: {a:?} before, {b:?} after"));
        }
    }
    out.sort();
    out.dedup();
    if before.balances != after.balances {
        out.push("an account balance differs".to_owned());
    }
    if (before.debit, before.credit) != (after.debit, after.credit) {
        out.push(format!(
            "trial balance {}/{} before, {}/{} after",
            before.debit, before.credit, after.debit, after.credit
        ));
    }
    if before.partner_rows != after.partner_rows {
        out.push("partner share rows differ".to_owned());
    }
    if before.journal_chain != after.journal_chain {
        out.push("the journal chain hashes differ".to_owned());
    }
    if after.audit_chain.len() != before.audit_chain.len() + audit_extra
        || after.audit_chain.get(..before.audit_chain.len()) != Some(before.audit_chain.as_slice())
    {
        out.push("the old audit rows are not the same".to_owned());
    }
    let strip = |m: &BTreeMap<String, String>| -> BTreeMap<String, String> {
        m.iter()
            .filter(|(k, _)| !skip_settings.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    };
    if strip(&before.settings) != strip(&after.settings) {
        out.push("settings differ".to_owned());
    }
    if before.learned != after.learned {
        out.push("learned words differ".to_owned());
    }
    if before.days != after.days {
        out.push("the days (closed up to, close history) differ".to_owned());
    }
    if before.questions != after.questions {
        out.push("the question log differs".to_owned());
    }
    out
}

pub fn flip_a_byte(from: &Path, to: &Path) {
    let mut bytes = fs::read(from).unwrap();
    let at = bytes.len() / 2;
    bytes[at] ^= 0xFF;
    fs::write(to, bytes).unwrap();
}

pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}
