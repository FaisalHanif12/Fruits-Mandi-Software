//! The sums of R20 and R24 for every seller (or one-time person) at once (Code Standards B22:
//! no database query inside a loop over rows).
//!
//! `mandi_db::vouchers::payable_totals` gives the five sums of the payable working
//! (PRD FR-7.59, money agent 8.2a) for ONE seller key, in three or more queries. R24 needs them
//! for every seller, so this module reads each figure once for all sellers with a `GROUP BY`
//! and gives back a map. The answer for a key is exactly what `payable_totals` gives:
//!
//! - **An ordinary seller (no person key).** His lines are many, so the sums come from the
//!   covering index of the amounts and the partial index of the advances, less the few lines that
//!   carry a person key or sit in an entry that is not live (a draft, a reversal, a reversed
//!   entry). The same method and the same SQL fragments as `live_sums_without_key`.
//! - **One person of the پختہ کھاتہ account.** His live lines, grouped by account and key.
//! - **Adjusted sums.** The active rows of `advance_allocations` whose line is live, grouped by
//!   account and key, once for the credit end and once for the advance end.
//!
//! All sums are `SUM()` on `INTEGER` paisa columns; the differences are checked in Rust. The
//! test `services/tests/reports_grouped.rs` compares every key with `payable_totals` on 500
//! sellers and checks that the number of queries does not grow with the number of sellers.
//!
//! The names of the one-time people come from one query too ([`pukhta_names`]).
//!
//! Example (after Event 2, PRD 4.9): the map has `(کریم, None) -> credit 10,160`; the payable
//! working turns that into 10,160 open.

use std::collections::HashMap;

use mandi_core::bikri::SavedPukhta;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use crate::error::ReportsError;
use crate::reports::common::{LIVE, placeholders};

/// The seller accounts: the control 03011 (PRD FR-10.2).
const SELLER_ACCOUNTS: &str = "SELECT a.id FROM accounts a JOIN account_controls k
    ON k.id = a.control_id WHERE k.code = '03011'";

/// Entries that are not live: drafts, reversal entries and the entries a posted reversal
/// reverses. The same set as `NOT_LIVE_ENTRIES` in `mandi_db::vouchers`, the exact complement of
/// the live condition.
const NOT_LIVE_ENTRIES: &str = "nonlive(id) AS (
    SELECT id FROM journal_entries WHERE reverses_entry_id IS NOT NULL
    UNION SELECT reverses_entry_id FROM journal_entries
          WHERE reverses_entry_id IS NOT NULL AND status = 'posted'
    UNION SELECT id FROM journal_entries WHERE status <> 'posted')";

/// The five sums of one seller key, in paisa. Zero for a key with no lines.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeySums {
    /// Σ of his credit lines.
    pub credit: i64,
    /// Σ of his debit lines that are not advances.
    pub payments: i64,
    /// Σ of his advance lines.
    pub advances_given: i64,
    /// What active advance rows take from his credit lines.
    pub adjusted_on_credits: i64,
    /// What active advance rows take from his advance lines.
    pub adjusted_on_advances: i64,
}

/// The sums of every seller key, and how many queries it took.
#[derive(Debug, Default)]
pub struct SellerSums {
    by_key: HashMap<(i64, Option<String>), KeySums>,
    /// The number of queries run. It does not depend on the number of sellers (B22), and a test
    /// checks that.
    pub queries: usize,
}

impl SellerSums {
    /// The sums of a key; zeros when it has no lines (what `payable_totals` gives too).
    pub fn of(&self, account_id: i64, person_key: Option<&str>) -> KeySums {
        self.by_key
            .get(&(account_id, person_key.map(str::to_owned)))
            .copied()
            .unwrap_or_default()
    }
}

/// `a - b`, checked. The sums are whole-rupee totals of a ledger, so a failure means broken data.
fn minus(a: i64, b: i64) -> Result<i64, ReportsError> {
    a.checked_sub(b)
        .ok_or_else(|| ReportsError::Data("a ledger sum does not fit".to_owned()))
}

/// `column = ?` for one account, or `column IN (the sellers)` for all of them. A fragment of our
/// own and a bound value, never user text (Code Standards B14).
fn scope(column: &str, account: Option<i64>) -> (String, Vec<Value>) {
    match account {
        Some(id) => (format!("{column} = ?"), vec![Value::Integer(id)]),
        None => (format!("{column} IN ({SELLER_ACCOUNTS})"), Vec::new()),
    }
}

/// Runs one grouped query and counts it.
fn rows<T>(
    conn: &Connection,
    queries: &mut usize,
    sql: &str,
    args: &[Value],
    read: impl FnMut(&mandi_db::rusqlite::Row<'_>) -> mandi_db::rusqlite::Result<T>,
) -> Result<Vec<T>, ReportsError> {
    *queries = queries.saturating_add(1);
    let mut statement = conn.prepare(sql)?;
    let found = statement
        .query_map(params_from_iter(args.iter()), read)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(found)
}

/// The sums of every seller key (or of the keys of one account), read with a fixed number of
/// grouped queries.
///
/// # Errors
/// A database error; `ReportsError::Data` if a sum does not fit.
pub fn payable_sums(conn: &Connection, account: Option<i64>) -> Result<SellerSums, ReportsError> {
    let mut loaded = SellerSums::default();
    let mut queries = 0_usize;

    // 1. Every line of each seller: credit and debit.
    let (in_scope, args) = scope("account_id", account);
    let all: HashMap<i64, (i64, i64)> = rows(
        conn,
        &mut queries,
        &format!(
            "SELECT account_id, COALESCE(SUM(cr_paisa), 0), COALESCE(SUM(dr_paisa), 0)
             FROM journal_lines WHERE {in_scope} GROUP BY account_id"
        ),
        &args,
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                (r.get::<_, i64>(1)?, r.get::<_, i64>(2)?),
            ))
        },
    )?
    .into_iter()
    .collect();

    // 2. Every advance line of each seller.
    let given_all: HashMap<i64, i64> = rows(
        conn,
        &mut queries,
        &format!(
            "SELECT account_id, COALESCE(SUM(dr_paisa), 0) FROM journal_lines
             WHERE is_advance = 1 AND {in_scope} GROUP BY account_id"
        ),
        &args,
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
    )?
    .into_iter()
    .collect();

    // 3. The lines to take off: lines that carry a person key, and lines of entries that are not
    // live. (A line in both sets is counted once, as `UNION` takes the line itself.)
    let (in_scope_a, args_a) = scope("account_id", account);
    let (in_scope_b, args_b) = scope("account_id", account);
    let mut args_out = args_a;
    args_out.extend(args_b);
    let out: HashMap<i64, (i64, i64, i64)> = rows(
        conn,
        &mut queries,
        &format!(
            "WITH {NOT_LIVE_ENTRIES}
             SELECT account_id,
                    COALESCE(SUM(CASE WHEN cr_paisa > 0 THEN cr_paisa END), 0),
                    COALESCE(SUM(CASE WHEN dr_paisa > 0 AND is_advance = 0 THEN dr_paisa END), 0),
                    COALESCE(SUM(CASE WHEN is_advance = 1 THEN dr_paisa END), 0)
             FROM (SELECT id, account_id, cr_paisa, dr_paisa, is_advance FROM journal_lines
                   WHERE {in_scope_a} AND pukhta_person_key IS NOT NULL
                   UNION
                   SELECT id, account_id, cr_paisa, dr_paisa, is_advance FROM journal_lines
                   WHERE {in_scope_b} AND entry_id IN (SELECT id FROM nonlive))
             GROUP BY account_id"
        ),
        &args_out,
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                (
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ),
            ))
        },
    )?
    .into_iter()
    .collect();

    // The keyless sums of each account: all lines, less the lines taken off.
    for (account_id, (all_credit, all_debit)) in all {
        let all_given = given_all.get(&account_id).copied().unwrap_or(0);
        let (out_credit, out_payments, out_given) =
            out.get(&account_id).copied().unwrap_or((0, 0, 0));
        let sums = loaded.by_key.entry((account_id, None)).or_default();
        sums.credit = minus(all_credit, out_credit)?;
        sums.payments = minus(minus(all_debit, all_given)?, out_payments)?;
        sums.advances_given = minus(all_given, out_given)?;
    }

    // 4. The live lines of each person key of a پختہ کھاتہ account.
    let (in_scope_l, args_l) = scope("l.account_id", account);
    let keyed = rows(
        conn,
        &mut queries,
        &format!(
            "SELECT l.account_id, l.pukhta_person_key,
                    COALESCE(SUM(CASE WHEN l.cr_paisa > 0 THEN l.cr_paisa END), 0),
                    COALESCE(SUM(CASE WHEN l.dr_paisa > 0 AND l.is_advance = 0 THEN l.dr_paisa END), 0),
                    COALESCE(SUM(CASE WHEN l.is_advance = 1 THEN l.dr_paisa END), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE {in_scope_l} AND l.pukhta_person_key IS NOT NULL AND {LIVE}
             GROUP BY l.account_id, l.pukhta_person_key"
        ),
        &args_l,
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                [r.get::<_, i64>(2)?, r.get::<_, i64>(3)?, r.get::<_, i64>(4)?],
            ))
        },
    )?;
    for (account_id, key, [credit, payments, given]) in keyed {
        let sums = loaded.by_key.entry((account_id, Some(key))).or_default();
        sums.credit = credit;
        sums.payments = payments;
        sums.advances_given = given;
    }

    // 5 and 6. What the active advance rows take from the credit lines and from the advance
    // lines, over live lines only.
    for end in ["credit_line_id", "advance_line_id"] {
        let (in_scope_r, args_r) = scope("a.account_id", account);
        let adjusted = rows(
            conn,
            &mut queries,
            &format!(
                "SELECT a.account_id, l.pukhta_person_key, COALESCE(SUM(a.amount_paisa), 0)
                 FROM advance_allocations a
                 JOIN journal_lines l ON l.id = a.{end}
                 JOIN journal_entries e ON e.id = l.entry_id
                 WHERE {in_scope_r} AND a.is_active = 1 AND {LIVE}
                 GROUP BY a.account_id, l.pukhta_person_key"
            ),
            &args_r,
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            },
        )?;
        for (account_id, key, amount) in adjusted {
            let sums = loaded.by_key.entry((account_id, key)).or_default();
            if end == "credit_line_id" {
                sums.adjusted_on_credits = amount;
            } else {
                sums.adjusted_on_advances = amount;
            }
        }
    }

    loaded.queries = queries;
    Ok(loaded)
}

/// The names and phone of the one-time people of some پختہ کھاتہ accounts, as each was last
/// typed (the same answer as `mandi_db::vouchers::pukhta_person` for each key), from ONE query:
/// the latest line of every key that has a line on these accounts.
///
/// # Errors
/// A database error.
pub fn pukhta_names(
    conn: &Connection,
    account_ids: &[i64],
) -> Result<HashMap<String, SavedPukhta>, ReportsError> {
    if account_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let sql = format!(
        "SELECT person_key, name_ur, name_en, phone FROM (
             SELECT person_key, name_ur, name_en, phone,
                    ROW_NUMBER() OVER (PARTITION BY person_key
                                       ORDER BY entry_date DESC, row_id DESC) AS rn
             FROM (
                 SELECT s.pukhta_person_key AS person_key, s.pukhta_name_ur AS name_ur,
                        s.pukhta_name_en AS name_en, s.pukhta_phone AS phone,
                        s.entry_date AS entry_date, s.id AS row_id
                 FROM bikri_sales s WHERE s.pukhta_person_key IN (SELECT DISTINCT pukhta_person_key
                     FROM journal_lines WHERE account_id IN ({marks}) AND pukhta_person_key IS NOT NULL)
                 UNION ALL
                 SELECT l.pukhta_person_key, l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone,
                        e.entry_date, l.id
                 FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
                 WHERE l.pukhta_person_key IN (SELECT DISTINCT pukhta_person_key
                     FROM journal_lines WHERE account_id IN ({marks}) AND pukhta_person_key IS NOT NULL)))
         WHERE rn = 1",
        marks = placeholders(account_ids.len())
    );
    let args: Vec<Value> = account_ids
        .iter()
        .chain(account_ids.iter())
        .map(|id| Value::Integer(*id))
        .collect();
    let mut statement = conn.prepare(&sql)?;
    let found = statement
        .query_map(params_from_iter(args.iter()), |r| {
            let key: String = r.get(0)?;
            Ok((
                key.clone(),
                SavedPukhta {
                    name_ur: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    name_en: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    phone: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    person_key: key,
                },
            ))
        })?
        .collect::<Result<HashMap<_, _>, _>>()?;
    Ok(found)
}
