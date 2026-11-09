//! Fast reads of the buyers' and the sellers' books for the day summary, the dashboard and the
//! assistant (plan 48 P48-S3, NFR-SPD-2, NFR-SPD-10, NFR-AI-3, PRD FR-6.1).
//!
//! Every read here gives exactly the rows the slower query it replaced gave (the old SQL is kept
//! in the test `services/tests/speed_rewrites_equal.rs` and compared row by row). The only thing
//! that changed is HOW the rows are found, never which rows or which sums:
//!
//! - **Left out, not probed.** A balance before a day is every posted line less the few lines
//!   whose entry is not posted or is dated on or after the day. The all-time sum comes from the
//!   covering index `idx_journal_lines_account_amounts` (migration 0002) and the few entries to
//!   leave out are listed, instead of looking up the entry of every line. This is the idiom of
//!   `ledger::account_totals`. It took 10 seconds for 3.5 million lines and takes 0.2 seconds.
//! - **Driven by the entries.** The credits of the sellers dated on or after a day are found from
//!   the entries of those days (`idx_journal_entries_entry_date`), not from every line of every
//!   seller.
//!
//! All sums are `SUM()` on `INTEGER` paisa columns (money agent Section 9); every difference is
//! worked out by the caller with checked maths. Nothing is written (Code Standards B29).
//!
//! Example (the worked day after Events 1 to 5): the buyers' totals before 10-10-26 are empty,
//! before 11-10-26 اکرم has debit 12,880 and credit 10,000.

use std::collections::HashMap;

use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

/// The live-entry condition on `journal_entries e`: posted, not a reversal and not reversed by a
/// posted reversal. The same text as the one of `matching` and of the reports.
const LIVE: &str = "e.status = 'posted' AND e.reverses_entry_id IS NULL
    AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')";

/// Values for a statement, numbered `?1`, `?2` ... as they are added. The numbers are ours; a value
/// is only ever bound (Code Standards B14).
#[derive(Default)]
struct Binds {
    values: Vec<Value>,
}

impl Binds {
    /// Adds a value and gives its mark.
    fn add(&mut self, value: Value) -> String {
        self.values.push(value);
        format!("?{}", self.values.len())
    }

    /// Adds the texts and gives their marks, joined for an `IN (...)`.
    fn add_texts(&mut self, texts: &[&str]) -> String {
        texts
            .iter()
            .map(|t| self.add(Value::Text((*t).to_owned())))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Which party a read is for. Both `None` is every party of the controls.
#[derive(Debug, Clone, Copy, Default)]
pub struct PartyScope<'a> {
    /// Only this account.
    pub account_id: Option<i64>,
    /// Only this person of a چکتا کھاتہ account.
    pub person_key: Option<&'a str>,
}

/// What one party (an account, or a person of a shared چکتا کھاتہ account) has on each side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyTotals {
    /// The account.
    pub account_id: i64,
    /// The person key of a shared چکتا کھاتہ account; empty for every other account.
    pub person_key: String,
    /// The control code of the account (`0304` or `03011`).
    pub control_code: String,
    /// Σ debits, paisa.
    pub dr_paisa: i64,
    /// Σ credits, paisa.
    pub cr_paisa: i64,
}

/// The accounts of these controls as a sub-query of ids, with `shared` = the shared چکتا کھاتہ
/// accounts (`is_chukta_khata = 1`) or all the others. A subquery and not a list of ids: there
/// can be 9,000 buyers and more (money agent, plan 48 P48-S3, item 6).
fn accounts_of(binds: &mut Binds, controls: &[&str], shared: bool) -> String {
    let codes = binds.add_texts(controls);
    format!(
        "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ({codes}) AND a.is_chukta_khata = {}",
        i32::from(shared)
    )
}

/// The entries a balance before `before` leaves out: not posted, or dated on or after it.
fn left_out_from(binds: &mut Binds, before: &str) -> String {
    let day = binds.add(Value::Text(before.to_owned()));
    format!(
        "SELECT id FROM journal_entries WHERE status <> 'posted'
         UNION ALL SELECT id FROM journal_entries WHERE entry_date >= {day}"
    )
}

/// How [`totals_before`] finds the lines before a day. Both give the same rows; only the time
/// differs, so a test can run each on the same data and compare (the same pattern as
/// `ledger::DayMethod`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeforeMethod {
    /// For each line of the controls, the entry is looked up and its status and day are checked.
    /// Slow for many lines (5 microseconds a line), exact for any party and any filter. Cheapest
    /// when few entries are dated before the day.
    PerLine,
    /// All-time sums from the covering index less the lines of the entries that are not posted or
    /// are dated on or after the day. 0.2 seconds for 3.5 million lines whatever the day.
    LeftOut,
}

/// With no more entries than this dated before the day, [`BeforeMethod::PerLine`] is cheaper: the
/// per-line read costs about 10 microseconds an entry, the left-out read about 0.2 seconds plus 0.6
/// microseconds for every entry dated on or after the day.
pub const EARLY_FROM_ENTRIES: i64 = 40_000;

/// True if more than `limit` entries are dated before `day`. Reads at most `limit + 1` index
/// entries of `idx_journal_entries_entry_date`.
fn more_entries_before(conn: &Connection, day: &str, limit: i64) -> rusqlite::Result<bool> {
    let counted: i64 = conn.query_row(
        "SELECT COUNT(*) FROM (SELECT 1 FROM journal_entries WHERE entry_date < ?1 LIMIT ?2)",
        rusqlite::params![day, limit.saturating_add(1)],
        |r| r.get(0),
    )?;
    Ok(counted > limit)
}

/// True if no ordinary account of these controls (`is_chukta_khata = 0`) has a line that carries a
/// person key. Nothing in the schema forbids such a line, but no writer makes one (money agent,
/// plan 48 P48-S3, item 1). The fast reads add the lines of an ordinary account into one sum, so
/// they check this first and fall back to the line-by-line read if it is false: a result is never
/// merged silently. One look at `idx_journal_lines_account_person` for each account.
///
/// # Errors
/// Any SQLite error.
pub fn person_keys_only_on_shared(conn: &Connection, controls: &[&str]) -> rusqlite::Result<bool> {
    if controls.is_empty() {
        return Ok(true);
    }
    let (sql, values) = stray_keys_sql(controls);
    let stray: bool = conn.query_row(&sql, params_from_iter(values.iter()), |r| r.get(0))?;
    Ok(!stray)
}

/// The SQL of [`person_keys_only_on_shared`] and its values. Public so a test can check its query
/// plan (Code Standards B18).
#[doc(hidden)]
pub fn stray_keys_sql(controls: &[&str]) -> (String, Vec<Value>) {
    let mut binds = Binds::default();
    let codes = binds.add_texts(controls);
    let sql = format!(
        "SELECT EXISTS (SELECT 1 FROM accounts a
                        JOIN account_controls k ON k.id = a.control_id
                        JOIN journal_lines l ON l.account_id = a.id
                                            AND l.chukta_person_key IS NOT NULL
                        WHERE k.code IN ({codes}) AND a.is_chukta_khata = 0)"
    );
    (sql, binds.values)
}

/// The method [`totals_before`] uses: [`BeforeMethod::LeftOut`] for the whole book (no account or
/// person asked for) when many entries are dated before the day and no ordinary account carries a
/// person key; otherwise [`BeforeMethod::PerLine`].
///
/// # Errors
/// Any SQLite error.
pub fn before_method(
    conn: &Connection,
    controls: &[&str],
    before: &str,
    scope: PartyScope<'_>,
) -> rusqlite::Result<BeforeMethod> {
    before_method_with(conn, controls, before, scope, EARLY_FROM_ENTRIES)
}

/// [`before_method`] with the number of earlier entries that makes the day "early" given, so a test
/// can reach both methods without writing forty thousand entries.
///
/// # Errors
/// Any SQLite error.
pub fn before_method_with(
    conn: &Connection,
    controls: &[&str],
    before: &str,
    scope: PartyScope<'_>,
    early_entries: i64,
) -> rusqlite::Result<BeforeMethod> {
    if scope.account_id.is_some() || scope.person_key.is_some() {
        return Ok(BeforeMethod::PerLine);
    }
    if !more_entries_before(conn, before, early_entries)? {
        return Ok(BeforeMethod::PerLine);
    }
    if !person_keys_only_on_shared(conn, controls)? {
        return Ok(BeforeMethod::PerLine);
    }
    Ok(BeforeMethod::LeftOut)
}

/// What each buyer and seller (the accounts of `controls`) had before `before` (`YYYY-MM-DD`):
/// every posted line dated earlier, by account and, for the two shared چکتا کھاتہ accounts, by
/// person. An account with no such line is not in the answer. The rows are sorted by account and
/// person. [`before_method`] picks the way of finding them; the answer is the same either way.
///
/// The person key exists only on the shared accounts (`is_chukta_khata = 1`): every other account
/// has no key, so its person is empty and one sum covers it.
///
/// # Errors
/// Any SQLite error (a `SUM` that overflows is an error, never a float).
pub fn totals_before(
    conn: &Connection,
    controls: &[&str],
    before: &str,
    scope: PartyScope<'_>,
) -> rusqlite::Result<Vec<PartyTotals>> {
    let method = before_method(conn, controls, before, scope)?;
    totals_before_by(conn, method, controls, before, scope)
}

/// [`totals_before`] by a chosen method. [`BeforeMethod::LeftOut`] is for the whole book; with an
/// account or a person asked for it is the same as [`BeforeMethod::PerLine`].
///
/// # Errors
/// Any SQLite error.
pub fn totals_before_by(
    conn: &Connection,
    method: BeforeMethod,
    controls: &[&str],
    before: &str,
    scope: PartyScope<'_>,
) -> rusqlite::Result<Vec<PartyTotals>> {
    if controls.is_empty() {
        return Ok(Vec::new());
    }
    let whole_book = scope.account_id.is_none() && scope.person_key.is_none();
    let mut out = if method == BeforeMethod::LeftOut && whole_book {
        totals_before_left_out(conn, controls, before)?
    } else {
        totals_before_per_line(conn, controls, before, scope)?
    };
    out.sort_by(|a, b| (a.account_id, &a.person_key).cmp(&(b.account_id, &b.person_key)));
    Ok(out)
}

/// The line-by-line read: the SQL that the day summary used before plan 48 P48-S3, kept verbatim
/// in meaning (grouped by the stored key, so a line with no key is its own group).
fn totals_before_per_line(
    conn: &Connection,
    controls: &[&str],
    before: &str,
    scope: PartyScope<'_>,
) -> rusqlite::Result<Vec<PartyTotals>> {
    let mut binds = Binds::default();
    let day = binds.add(Value::Text(before.to_owned()));
    let codes = binds.add_texts(controls);
    let mut filter = String::new();
    if let Some(account) = scope.account_id {
        filter.push_str(&format!(
            " AND l.account_id = {}",
            binds.add(Value::Integer(account))
        ));
    }
    if let Some(person) = scope.person_key {
        filter.push_str(&format!(
            " AND l.chukta_person_key = {}",
            binds.add(Value::Text(person.to_owned()))
        ));
    }
    let sql = format!(
        "SELECT l.account_id, COALESCE(l.chukta_person_key, ''), k.code,
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ({codes}) AND e.status = 'posted'
           AND e.entry_date < {day}{filter}
         GROUP BY l.account_id, l.chukta_person_key, k.code"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(binds.values.iter()), |r| {
        Ok(PartyTotals {
            account_id: r.get(0)?,
            person_key: r.get(1)?,
            control_code: r.get(2)?,
            dr_paisa: r.get(3)?,
            cr_paisa: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// The SQL of the left-out read for the accounts with one key, and its values. Public so a test can
/// check its query plan (Code Standards B18).
#[doc(hidden)]
pub fn left_out_single_key_sql(controls: &[&str], before: &str) -> (String, Vec<Value>) {
    let mut binds = Binds::default();
    let accounts = accounts_of(&mut binds, controls, false);
    let left_out = left_out_from(&mut binds, before);
    let sql = format!(
        "SELECT account_id, COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0)
         FROM journal_lines
         WHERE account_id IN ({accounts}) AND entry_id NOT IN ({left_out})
         GROUP BY account_id"
    );
    (sql, binds.values)
}

/// The left-out read of the whole book (see [`BeforeMethod::LeftOut`]).
fn totals_before_left_out(
    conn: &Connection,
    controls: &[&str],
    before: &str,
) -> rusqlite::Result<Vec<PartyTotals>> {
    // The control code of each account.
    let mut binds = Binds::default();
    let codes = binds.add_texts(controls);
    let meta_sql = format!(
        "SELECT a.id, k.code FROM accounts a JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ({codes})"
    );
    let mut meta: HashMap<i64, String> = HashMap::new();
    {
        let mut statement = conn.prepare(&meta_sql)?;
        let rows = statement.query_map(params_from_iter(binds.values.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (id, code) = row?;
            meta.insert(id, code);
        }
    }
    let mut out: Vec<PartyTotals> = Vec::new();
    // Accounts with one key (no person): all-time sums less the entries to leave out.
    {
        let (sql, values) = left_out_single_key_sql(controls, before);
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(values.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (account_id, dr, cr) = row?;
            out.push(PartyTotals {
                account_id,
                person_key: String::new(),
                control_code: meta.get(&account_id).cloned().unwrap_or_default(),
                dr_paisa: dr,
                cr_paisa: cr,
            });
        }
    }
    // The shared چکتا کھاتہ accounts: person by person (grouped by the stored key, so a line with
    // no key is its own group, as the line-by-line read has it).
    {
        let mut binds = Binds::default();
        let accounts = accounts_of(&mut binds, controls, true);
        let left_out = left_out_from(&mut binds, before);
        let sql = format!(
            "SELECT account_id, COALESCE(chukta_person_key, ''), COALESCE(SUM(dr_paisa), 0),
                    COALESCE(SUM(cr_paisa), 0)
             FROM journal_lines
             WHERE account_id IN ({accounts}) AND entry_id NOT IN ({left_out})
             GROUP BY account_id, chukta_person_key"
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(binds.values.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (account_id, person_key, dr, cr) = row?;
            out.push(PartyTotals {
                account_id,
                person_key,
                control_code: meta.get(&account_id).cloned().unwrap_or_default(),
                dr_paisa: dr,
                cr_paisa: cr,
            });
        }
    }
    Ok(out)
}

/// One credit line of a seller that matters for the age of what is payable to him.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldCredit {
    /// The seller's account.
    pub account_id: i64,
    /// The person key of an old one-time seller.
    pub person_key: Option<String>,
    /// True for an opening balance.
    pub is_opening: bool,
    /// The day of the entry, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The credit, paisa.
    pub cr_paisa: i64,
    /// What active advance rows take from this credit, paisa.
    pub adjusted_paisa: i64,
}

/// The SQL of [`held_credits`]: `?1` the seller control, `?2` the cutoff day. Public so a test can
/// check its query plan (Code Standards B18).
///
/// The join order and the two indexes are fixed in the text (`CROSS JOIN`, `INDEXED BY`). The
/// app's file has no table statistics (only the benchmark's file has), and without them SQLite
/// starts from the seller accounts and probes every entry for each of them (3 seconds instead of
/// 4 milliseconds).
#[doc(hidden)]
pub const HELD_CREDITS_SQL: &str =
    "SELECT l.account_id, l.chukta_person_key, e.posting_type = 'opening', e.entry_date,
                l.cr_paisa,
                COALESCE((SELECT SUM(a.amount_paisa) FROM advance_allocations a
                          WHERE a.credit_line_id = l.id AND a.is_active = 1), 0)
         FROM journal_entries e
         CROSS JOIN journal_lines l ON l.entry_id = e.id
         CROSS JOIN accounts ac ON ac.id = l.account_id
         CROSS JOIN account_controls k ON k.id = ac.control_id
         WHERE e.id IN (SELECT id FROM journal_entries INDEXED BY idx_journal_entries_entry_date
                         WHERE entry_date >= ?2
                        UNION SELECT id FROM journal_entries INDEXED BY idx_journal_entries_opening
                         WHERE posting_type = 'opening')
           AND k.code = ?1 AND l.cr_paisa > 0
           AND e.status = 'posted' AND e.reverses_entry_id IS NULL
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')";

/// The live credit lines of the accounts of the seller control that matter for the age: the ones
/// dated on or after `cutoff` and the opening credits of any date, each with the advance adjusted
/// on it (active rows only). Driven by the entries of those days and by the opening entries
/// (`idx_journal_entries_opening`, migration 0016), not by every line of every seller.
///
/// # Errors
/// Any SQLite error.
pub fn held_credits(
    conn: &Connection,
    seller_control: &str,
    cutoff: &str,
) -> rusqlite::Result<Vec<HeldCredit>> {
    let mut statement = conn.prepare(HELD_CREDITS_SQL)?;
    let rows = statement.query_map(rusqlite::params![seller_control, cutoff], |r| {
        Ok(HeldCredit {
            account_id: r.get(0)?,
            person_key: r.get(1)?,
            is_opening: r.get(2)?,
            entry_date: r.get(3)?,
            cr_paisa: r.get(4)?,
            adjusted_paisa: r.get(5)?,
        })
    })?;
    rows.collect()
}

// ---- the open bills of the buyers (R20 ages) -------------------------------------------------

/// A debit line of a buyer that the active matching rows do not settle in full (PRD FR-15.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenBill {
    /// The journal line.
    pub line_id: i64,
    /// The buyer's account.
    pub account_id: i64,
    /// The person key of a چکتا buyer.
    pub person_key: Option<String>,
    /// The entry of the line.
    pub entry_id: i64,
    /// The day of the entry, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What is still open: the debit less what active matching rows settle, paisa.
    pub open_paisa: i64,
}

/// At most this many ids in one `IN (...)` (SQLite allows 32,766 values; the reports use 500).
const IN_CHUNK: usize = 500;

/// The debit lines of the requested ordinary buyers above which the bulk read is used. Below it,
/// the lines are looked at one by one: 5.8 microseconds a line (50,000 lines is 0.29 s), while the
/// bulk read costs about 0.35 s whatever the number of buyers.
pub const BULK_FROM_DEBIT_LINES: i64 = 50_000;

/// The bulk read is for many buyers: a report for one buyer or one city must not read every buyer
/// (money agent, plan 48 P48-S3, item 11).
pub const BULK_FROM_ACCOUNTS: usize = 200;

/// The entry ids the bulk read can hold dates for: a date takes 10 bytes, so this is 50 MB (the
/// whole app may use 400 MB, NFR-SPD-16). A file with more entries than this (NFR-SCL-1 has 1.2
/// million, 12 MB) uses the line-by-line read.
const MAX_DENSE_ENTRY_ID: i64 = 5_000_000;

/// A hasher for integer keys: ids are small and spread, so one multiplication is enough. A map of
/// 300,000 matched lines is built and read many times in one report, and the default hasher is made
/// to resist an attacker who picks the keys, which nobody does here.
#[derive(Default)]
struct IdHasher(u64);

impl std::hash::Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(*byte)).wrapping_mul(0x517c_c1b7_2722_0a95);
        }
    }

    fn write_i64(&mut self, value: i64) {
        self.0 = u64::from_ne_bytes(value.to_ne_bytes()).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
}

type IdMap<V> = HashMap<i64, V, std::hash::BuildHasherDefault<IdHasher>>;

/// A broken row (an entry that is not there): the database is not what the schema promises.
fn broken(text: &str) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(text.to_owned())))
}

/// The `?` marks of a chunk, for an `IN (...)`.
fn placeholders(count: usize) -> String {
    vec!["?"; count].join(", ")
}

/// How [`open_buyer_bills_by`] finds the open bills of the ordinary buyers. Both give the same
/// rows; only the time differs, so a test can run each on the same data and compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMethod {
    /// The bulk read when many buyers with many lines are asked for, else line by line.
    Auto,
    /// For each debit line, the entry is looked up and the matching rows are summed.
    PerLine,
    /// One pass over the debit lines, one over the matched sums, and the dates and the entries
    /// that are not live joined in memory. Used only if no ordinary buyer carries a person key;
    /// otherwise it is the same as `PerLine`.
    Bulk,
}

/// The open bills of these accounts, in no particular order: every debit line of a live entry that
/// the active rows of `bill_allocations` do not settle in full.
///
/// A line is open when its debit is more than what the active matching rows take from it, and its
/// entry is live (posted, not a reversal and not reversed by a posted reversal). The answer is the
/// very set of rows the line-by-line SQL gives ([`open_bills_per_line`]); only the way of finding
/// them differs:
///
/// - the shared چکتا کھاتہ accounts and any account that is not an ordinary buyer are read line by
///   line (they carry the person key);
/// - ordinary buyers are read line by line when they are few or have few debit lines
///   ([`BULK_FROM_ACCOUNTS`], [`BULK_FROM_DEBIT_LINES`]);
/// - otherwise one pass over the debit lines of the ordinary buyers (the covering index), one pass
///   over the matched sums (`idx_bill_allocations_open_sum`), the dates of the entries and the
///   entries that are not live are joined here in memory. The database no longer looks up the entry
///   and sums the matching rows for each of 600,000 lines.
///
/// # Errors
/// Any SQLite error.
pub fn open_buyer_bills(conn: &Connection, account_ids: &[i64]) -> rusqlite::Result<Vec<OpenBill>> {
    open_buyer_bills_by(conn, account_ids, OpenMethod::Auto)
}

/// [`open_buyer_bills`] by a chosen method.
///
/// # Errors
/// Any SQLite error.
pub fn open_buyer_bills_by(
    conn: &Connection,
    account_ids: &[i64],
    method: OpenMethod,
) -> rusqlite::Result<Vec<OpenBill>> {
    open_buyer_bills_with(
        conn,
        account_ids,
        method,
        BULK_FROM_ACCOUNTS,
        BULK_FROM_DEBIT_LINES,
    )
}

/// [`open_buyer_bills_by`] with the two numbers that make the bulk read the choice of
/// [`OpenMethod::Auto`] given, so a test can reach both ways on a small file.
///
/// # Errors
/// Any SQLite error.
pub fn open_buyer_bills_with(
    conn: &Connection,
    account_ids: &[i64],
    method: OpenMethod,
    bulk_from_accounts: usize,
    bulk_from_lines: i64,
) -> rusqlite::Result<Vec<OpenBill>> {
    let mut unique: Vec<i64> = account_ids.to_vec();
    unique.sort_unstable();
    unique.dedup();
    let (chosen, bulk_ids, line_ids) =
        choose_open_method(conn, &unique, method, bulk_from_accounts, bulk_from_lines)?;
    if chosen == OpenMethod::PerLine {
        return open_bills_per_line(conn, &unique);
    }
    let mut found = open_bills_per_line(conn, &line_ids)?;
    found.extend(open_bills_bulk(conn, &bulk_ids)?);
    Ok(found)
}

/// The method [`open_buyer_bills_with`] uses for these accounts (never `Auto`), and the accounts
/// the bulk read takes and the ones read line by line. The bulk read is used when it is asked for,
/// or when `Auto` finds enough ordinary buyers with enough debit lines, and only if no ordinary
/// buyer carries a person key.
///
/// # Errors
/// Any SQLite error.
pub fn choose_open_method(
    conn: &Connection,
    unique_ids: &[i64],
    method: OpenMethod,
    bulk_from_accounts: usize,
    bulk_from_lines: i64,
) -> rusqlite::Result<(OpenMethod, Vec<i64>, Vec<i64>)> {
    if method == OpenMethod::PerLine {
        return Ok((OpenMethod::PerLine, Vec::new(), unique_ids.to_vec()));
    }
    // Ordinary buyers (the bulk read's accounts) and all the others.
    let ordinary = ordinary_buyers(conn, unique_ids)?;
    let (bulk_ids, line_ids): (Vec<i64>, Vec<i64>) = unique_ids
        .iter()
        .copied()
        .partition(|id| ordinary.contains(id));
    let bulk = !bulk_ids.is_empty()
        && (method == OpenMethod::Bulk
            || (bulk_ids.len() >= bulk_from_accounts
                && debit_lines_more_than(conn, &bulk_ids, bulk_from_lines)?))
        && person_keys_only_on_shared(conn, &[mandi_core::accounts::BUYER_CONTROL])?;
    if bulk {
        Ok((OpenMethod::Bulk, bulk_ids, line_ids))
    } else {
        Ok((OpenMethod::PerLine, Vec::new(), unique_ids.to_vec()))
    }
}

/// Which of these accounts are ordinary buyers: control 0304 and not a shared چکتا کھاتہ account.
fn ordinary_buyers(
    conn: &Connection,
    ids: &[i64],
) -> rusqlite::Result<std::collections::HashSet<i64>> {
    let mut out = std::collections::HashSet::new();
    for chunk in ids.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
             WHERE k.code = '0304' AND a.is_chukta_khata = 0 AND a.id IN ({})",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| r.get::<_, i64>(0))?;
        for row in rows {
            out.insert(row?);
        }
    }
    Ok(out)
}

/// True if these accounts have more than `limit` debit lines. Reads at most `limit + 1` index
/// entries in all.
fn debit_lines_more_than(conn: &Connection, ids: &[i64], limit: i64) -> rusqlite::Result<bool> {
    let mut counted = 0_i64;
    for chunk in ids.chunks(IN_CHUNK) {
        let room = limit.saturating_add(1).saturating_sub(counted);
        if room <= 0 {
            break;
        }
        let sql = format!(
            "SELECT COUNT(*) FROM (SELECT 1 FROM journal_lines
               WHERE account_id IN ({}) AND dr_paisa > 0 LIMIT ?)",
            placeholders(chunk.len())
        );
        let mut args: Vec<Value> = chunk.iter().map(|id| Value::Integer(*id)).collect();
        args.push(Value::Integer(room));
        let here: i64 = conn.query_row(&sql, params_from_iter(args.iter()), |r| r.get(0))?;
        counted = counted.saturating_add(here);
    }
    Ok(counted > limit)
}

/// The open bills of these accounts, looked at line by line: for each debit line the entry is
/// looked up and the active matching rows are summed. Slow for many lines, exact for any account
/// (it carries the person key). The SQL the bulk read must equal.
///
/// # Errors
/// Any SQLite error.
pub fn open_bills_per_line(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<OpenBill>> {
    let mut found = Vec::new();
    for chunk in ids.chunks(IN_CHUNK) {
        let matched = "COALESCE((SELECT SUM(a.amount_paisa) FROM bill_allocations a
                                WHERE a.bill_line_id = l.id AND a.is_active = 1), 0)";
        let sql = format!(
            "SELECT l.id, l.account_id, l.chukta_person_key, e.id, e.entry_date, l.dr_paisa, {matched}
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id IN ({marks}) AND l.dr_paisa > 0 AND {LIVE}
               AND l.dr_paisa > {matched}
             ORDER BY e.entry_date, e.id, l.line_no",
            marks = placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?;
        for row in rows {
            let (line_id, account_id, person_key, entry_id, entry_date, dr, matched) = row?;
            found.push(OpenBill {
                line_id,
                account_id,
                person_key,
                entry_id,
                entry_date,
                open_paisa: dr
                    .checked_sub(matched)
                    .ok_or_else(|| broken("a bill is matched for more than it holds"))?,
            });
        }
    }
    Ok(found)
}

/// What the active matching rows take from each bill line (`idx_bill_allocations_open_sum`). Public
/// so a test can check its query plan (Code Standards B18).
#[doc(hidden)]
pub const MATCHED_SUMS_SQL: &str = "SELECT bill_line_id, SUM(amount_paisa) FROM bill_allocations
     WHERE is_active = 1 GROUP BY bill_line_id";

/// The debit lines of the ordinary buyers (the covering index of the amounts). Public so a test can
/// check its query plan (Code Standards B18).
#[doc(hidden)]
pub const BUYER_DEBIT_LINES_SQL: &str =
    "SELECT l.account_id, l.entry_id, l.id, l.dr_paisa FROM journal_lines l
     WHERE l.account_id IN (SELECT a.id FROM accounts a
                            JOIN account_controls k ON k.id = a.control_id
                            WHERE k.code = '0304' AND a.is_chukta_khata = 0)
       AND l.dr_paisa > 0";

/// The bulk read of the open bills of ordinary buyers (see [`open_buyer_bills`]).
fn open_bills_bulk(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<OpenBill>> {
    let wanted: std::collections::HashSet<i64> = ids.iter().copied().collect();
    // What the active matching rows take from each line, once for all lines.
    let mut matched: IdMap<i64> = IdMap::default();
    {
        let mut statement = conn.prepare(MATCHED_SUMS_SQL)?;
        let rows = statement.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (line, sum) = row?;
            matched.insert(line, sum);
        }
    }
    // The entries that are not live: not posted, a reversal, or reversed by a posted reversal. The
    // exact complement of the live condition (the same set as `NOT_LIVE_ENTRIES` of the reports).
    let mut not_live: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for sql in [
        "SELECT id FROM journal_entries WHERE reverses_entry_id IS NOT NULL",
        "SELECT reverses_entry_id FROM journal_entries
           WHERE reverses_entry_id IS NOT NULL AND status = 'posted'",
        "SELECT id FROM journal_entries WHERE status <> 'posted'",
    ] {
        let mut statement = conn.prepare(sql)?;
        let rows = statement.query_map([], |r| r.get::<_, i64>(0))?;
        for row in rows {
            not_live.insert(row?);
        }
    }
    // The day of every entry, by entry id. A date is exactly 10 bytes (`CHECK (date(entry_date)
    // IS entry_date)`).
    let max_id: i64 = conn.query_row(
        "SELECT COALESCE(MAX(id), 0) FROM journal_entries",
        [],
        |r| r.get(0),
    )?;
    if max_id > MAX_DENSE_ENTRY_ID {
        return open_bills_per_line(conn, ids);
    }
    let size = usize::try_from(max_id.saturating_add(1)).map_err(|_| broken("entry ids"))?;
    let mut dates: Vec<[u8; 10]> = vec![[0; 10]; size];
    {
        let mut statement = conn.prepare("SELECT id, entry_date FROM journal_entries")?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let date = row
                .get_ref(1)?
                .as_str()
                .map_err(|_| broken("an entry date"))?;
            let slot = usize::try_from(id)
                .ok()
                .and_then(|i| dates.get_mut(i))
                .ok_or_else(|| broken("an entry id"))?;
            let bytes: [u8; 10] = date
                .as_bytes()
                .try_into()
                .map_err(|_| broken("an entry date is not 10 characters"))?;
            *slot = bytes;
        }
    }
    // The debit lines of the ordinary buyers, from the covering index; the ones asked for are joined.
    let mut statement = conn.prepare(BUYER_DEBIT_LINES_SQL)?;
    let mut rows = statement.query([])?;
    let mut found = Vec::new();
    while let Some(row) = rows.next()? {
        let account_id: i64 = row.get(0)?;
        if !wanted.contains(&account_id) {
            continue;
        }
        let (entry_id, line_id, dr): (i64, i64, i64) = (row.get(1)?, row.get(2)?, row.get(3)?);
        let taken = matched.get(&line_id).copied().unwrap_or(0);
        if dr <= taken || not_live.contains(&entry_id) {
            continue;
        }
        let date = usize::try_from(entry_id)
            .ok()
            .and_then(|i| dates.get(i))
            .and_then(|d| std::str::from_utf8(d).ok())
            .filter(|d| !d.starts_with('\0'))
            .ok_or_else(|| broken("a line is on an entry that is not there"))?;
        found.push(OpenBill {
            line_id,
            account_id,
            person_key: None,
            entry_id,
            entry_date: date.to_owned(),
            open_paisa: dr
                .checked_sub(taken)
                .ok_or_else(|| broken("a bill is matched for more than it holds"))?,
        });
    }
    Ok(found)
}
