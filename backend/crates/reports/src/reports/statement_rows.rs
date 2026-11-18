//! One party's statement with a row for each entry (plan 78 A7, B1, B2, B3; PRD FR-12.47).
//!
//! The statement screen, the statement print and the history pages read this one list, so they
//! cannot differ. A row is one live journal ENTRY of the party, not one journal line:
//!
//! * A بکری posts one buyer debit for each sale line and one credit for each line paid on the spot,
//!   and one seller credit for each lot. They are one row: the debits and credits are summed, the
//!   items are the row's `parts`, and what was paid on the spot is `on_spot_paisa` in the same row.
//! * An edit or a cancel writes a reversal and, for an edit, a new entry, all on the date of the
//!   entry (PRD FR-12.19, FR-12.22). The entry and its reversal cancel each other on the party, so
//!   they are hidden, and the new entry shows once with its edited mark. A reversal dated another
//!   day than its entry is NOT hidden: the balance after each row would not be true without it. The
//!   test is the one of the R35 slips and of the party ledgers (`SAME_DAY_PAIR`).
//! * An opening balance is its own row, "ابتدائی بقایا", first among the entries of its date (PRD
//!   FR-10.9).
//!
//! The balance after a row is the running sum over the rows shown, from the balance before the first
//! day. A hidden pair nets to zero, so the opening, the closing and the balance after every shown
//! row are what the books give; before anything is returned the shown rows are held to the same
//! closing as every line of the period (money agent rule 3). `include_history` shows every entry,
//! the hidden ones marked, and its totals are those of the party ledger R13.
//!
//! Example (plan 78 A7, احسان, with the figures of s10): 06-11-2026 ابتدائی بقایا 20,000 جمع (balance 20,000 جمع); 06-11-2026
//! بکری 1, credit 160,280 (balance 180,280 جمع); 16-11-2026 ادائیگی 1, debit 100,000 (balance
//! 80,280 جمع). Totals: debit 100,000, credit 180,280; closing 80,280 جمع.

use std::collections::{BTreeSet, HashMap};

use mandi_core::accounts::{BUYER_CONTROL, Balance, SELLER_CONTROL};
use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_core::party_ledgers::{LedgerSide, PartyLedgerHead};
use mandi_db::Db;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::accounts::ledger_items::{DetailQuery, load};
use super::accounts::lines::{self, LineFilter, RawLine};
use super::common::{add, in_snapshot, paisa, parse_day, source_of, today as today_of};
use super::ids;
use super::model::{DetailPart, Names, RowSource};
use crate::error::ReportsError;

/// The most rows a page of the screen holds when no limit is sent.
const PAGE: usize = 100;

/// Input of `party_ledgers_statement_rows`.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct StatementRowsInput {
    /// The party: a buyer or a seller.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The first day, `DD-MM-YYYY` or a shortcut (PRD FR-7.19). Missing: from the first entry.
    #[serde(default)]
    #[ts(optional)]
    pub from: Option<String>,
    /// The last day. Missing: up to now.
    #[serde(default)]
    #[ts(optional)]
    pub to: Option<String>,
    /// True shows every entry, also an entry that an edit or a cancel replaced and its reversal. They
    /// are marked, and the totals are those of the party ledger R13.
    #[serde(default)]
    pub include_history: bool,
    /// True gives the newest row first. The balances are those of the day order either way.
    #[serde(default)]
    pub newest_first: bool,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives 100.
    #[serde(default)]
    pub limit: u32,
}

/// One item of a row in short: the item (a lot's name on a seller) and its quantity, with the lines of
/// the same item added up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementShortItem {
    /// The item, or the lot's name on a seller's row.
    pub item: Names,
    /// The quantity in نگ.
    #[ts(type = "number")]
    pub qty: i64,
}

/// The last edit of the entry a row belongs to (PRD FR-7.71).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementEdited {
    /// The time, UTC, as stored.
    pub at: String,
    /// The display name of the user who made it.
    pub by: Option<String>,
    /// How many edits the entry has.
    #[ts(type = "number")]
    pub count: i64,
    /// The reason typed with it.
    pub reason: Option<String>,
    /// The Owner's sign saved with it, when it was made in a reopened day (PRD FR-21.4).
    pub sign: Option<String>,
}

/// One entry of the statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementRowsRow {
    /// The day of the entry, `YYYY-MM-DD`.
    pub date: String,
    /// The journal entry.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// Fluent message ID of the kind (`report-doc-bikri-first`, `report-doc-opening` ...).
    pub type_id: String,
    /// The بکری or voucher number. Null for an entry with none (an opening).
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The record the row opens.
    pub source: RowSource,
    /// Σ debits of the party in the entry.
    #[ts(type = "Paisa")]
    pub dr_paisa: Paisa,
    /// Σ credits of the party in the entry, what was paid on the spot included.
    #[ts(type = "Paisa")]
    pub cr_paisa: Paisa,
    /// The part of `cr_paisa` a buyer paid on the spot (PRD FR-12.44). 0 on any other row.
    #[ts(type = "Paisa")]
    pub on_spot_paisa: Paisa,
    /// The balance after this row.
    pub balance: Balance,
    /// What the row is about in one line: the items and their quantities.
    pub short: Vec<StatementShortItem>,
    /// The items in full: a buyer's sale lines, a seller's lots, cash lent, the reason. Without the
    /// money paid on the spot, which is `on_spot_paisa`.
    pub parts: Vec<DetailPart>,
    /// The narration or note as typed. Null if none.
    pub narration: Option<String>,
    /// Set when the entry was edited.
    pub edited: Option<StatementEdited>,
    /// True for a reversal, which is shown only when it is dated another day than its entry or when
    /// `include_history` is on.
    pub is_reversal: bool,
    /// True for an entry that an edit or a cancel replaced, or its reversal (same day). Shown only
    /// with `include_history`.
    pub is_replaced: bool,
}

/// What `party_ledgers_statement_rows` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementRows {
    /// Who the statement is for.
    pub head: PartyLedgerHead,
    /// The first day of the period, `YYYY-MM-DD`. Null for the first entry.
    pub from: Option<String>,
    /// The last day of the period, `YYYY-MM-DD`. Null for now.
    pub to: Option<String>,
    /// The balance before the first day: everything dated earlier (0 with no `from`).
    pub brought_forward: Balance,
    /// One page of the entries.
    pub rows: Vec<StatementRowsRow>,
    /// How many entries the whole period has.
    #[ts(type = "number")]
    pub count: i64,
    /// Σ debits of all the entries of the period (never of the page).
    #[ts(type = "Paisa")]
    pub dr_total_paisa: Paisa,
    /// Σ credits of all the entries of the period.
    #[ts(type = "Paisa")]
    pub cr_total_paisa: Paisa,
    /// Σ of the money paid on the spot in the period.
    #[ts(type = "Paisa")]
    pub on_spot_total_paisa: Paisa,
    /// The balance at the end of the period: the brought forward balance, plus the debits, less the
    /// credits. With no `to` it is his balance now (the party card, R13).
    pub closing: Balance,
}

fn reject(code: ErrorCode, field: &str) -> ReportsError {
    ReportsError::Rejected(vec![FieldError::new(code, field)])
}

/// One journal entry of the party, with his lines in it.
struct Entry {
    id: i64,
    date: String,
    posting_type: String,
    source_type: String,
    source_id: i64,
    reverses: Option<i64>,
    doc_number: Option<i64>,
    venture: (Option<String>, Option<i64>),
    voucher: Option<(String, Option<String>)>,
    narration: Option<String>,
    /// (line number, debit, credit) in line order.
    lines: Vec<(i64, i64, i64)>,
    dr: i64,
    cr: i64,
}

fn overflow() -> ReportsError {
    ReportsError::Data("the statement sums do not fit".to_owned())
}

/// The entries of the lines, in the order of the journal. A line of an entry follows the line before
/// it, so one pass groups them.
fn entries_of(raw: Vec<RawLine>) -> Result<Vec<Entry>, ReportsError> {
    let mut out: Vec<Entry> = Vec::new();
    for line in raw {
        let narration = line
            .narration
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_owned);
        match out.last_mut().filter(|e| e.id == line.entry_id) {
            Some(entry) => {
                entry.dr = entry.dr.checked_add(line.dr).ok_or_else(overflow)?;
                entry.cr = entry.cr.checked_add(line.cr).ok_or_else(overflow)?;
                entry.lines.push((line.line_no, line.dr, line.cr));
                if entry.narration.is_none() {
                    entry.narration = narration;
                }
            }
            None => out.push(Entry {
                id: line.entry_id,
                date: line.entry_date.clone(),
                posting_type: line.posting_type.clone(),
                source_type: line.source_type.clone(),
                source_id: line.source_id,
                reverses: line.reverses,
                doc_number: line.doc_number,
                venture: (line.venture_kind.clone(), line.venture_id),
                voucher: (line.source_type == "voucher")
                    .then(|| {
                        line.voucher_type
                            .clone()
                            .map(|t| (t, line.voucher_reason.clone()))
                    })
                    .flatten(),
                narration,
                lines: vec![(line.line_no, line.dr, line.cr)],
                dr: line.dr,
                cr: line.cr,
            }),
        }
    }
    Ok(out)
}

/// The entries that an edit or a cancel of the same day cancelled: an entry whose reversal is dated
/// the same day, and a reversal of an entry of the same day (the test of `SAME_DAY_PAIR`). Both
/// halves have the party's lines in the same day, so the day's entries are enough to tell.
fn replaced_ids(entries: &[Entry]) -> BTreeSet<i64> {
    let date_of: HashMap<i64, &str> = entries.iter().map(|e| (e.id, e.date.as_str())).collect();
    let mut out = BTreeSet::new();
    for entry in entries {
        if let Some(original) = entry.reverses
            && date_of.get(&original) == Some(&entry.date.as_str())
        {
            out.insert(entry.id);
            out.insert(original);
        }
    }
    out
}

/// The items of a row in short: each item (a lot's name for a seller) with its quantity, the lines of
/// one item added up, in the order they are first seen.
fn short_of(parts: &[DetailPart]) -> Result<Vec<StatementShortItem>, ReportsError> {
    let mut out: Vec<StatementShortItem> = Vec::new();
    for part in parts {
        let (item, qty) = match part {
            DetailPart::Sale { item, qty, .. } | DetailPart::Lot { item, qty, .. } => (item, *qty),
            _ => continue,
        };
        match out.iter_mut().find(|s| s.item == *item) {
            Some(found) => found.qty = found.qty.checked_add(qty).ok_or_else(overflow)?,
            None => out.push(StatementShortItem {
                item: item.clone(),
                qty,
            }),
        }
    }
    Ok(out)
}

/// What the buyer paid on the spot in each entry, from the value stored on his sale lines of it
/// (PRD FR-12.44): Σ `paid_paisa` by posting. A line replaced by an edit keeps its old posting, so
/// an entry shown with the history has its own sum. A چکتا buyer has no statement.
fn paid_on_spot(
    conn: &mandi_db::rusqlite::Connection,
    account_id: i64,
    entry_ids: &[i64],
) -> Result<HashMap<i64, i64>, ReportsError> {
    let mut out = HashMap::new();
    for chunk in entry_ids.chunks(super::common::IN_CHUNK) {
        let sql = format!(
            "SELECT journal_entry_id, COALESCE(SUM(paid_paisa), 0) FROM bikri_sales
             WHERE buyer_account_id = ?1 AND chukta_person_key IS NULL
               AND journal_entry_id IN ({}) GROUP BY journal_entry_id",
            super::common::placeholders(chunk.len())
        );
        let mut args: Vec<mandi_db::rusqlite::types::Value> =
            vec![mandi_db::rusqlite::types::Value::Integer(account_id)];
        args.extend(
            chunk
                .iter()
                .map(|id| mandi_db::rusqlite::types::Value::Integer(*id)),
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement
            .query_map(mandi_db::rusqlite::params_from_iter(args.iter()), |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        out.extend(rows);
    }
    Ok(out)
}

/// The whole period, every entry in the day order.
pub(crate) struct Whole {
    pub(crate) head: PartyLedgerHead,
    pub(crate) from: Option<String>,
    pub(crate) to: Option<String>,
    pub(crate) brought_forward: Balance,
    /// Oldest first, with the running balance.
    pub(crate) rows: Vec<StatementRowsRow>,
    pub(crate) dr_total: Paisa,
    pub(crate) cr_total: Paisa,
    pub(crate) on_spot_total: Paisa,
    pub(crate) closing: Balance,
}

/// Reads every entry of the period (the page limits of the input are not used).
///
/// # Errors
/// `ERR_NOT_FOUND` on `account_id` for an account that is not a buyer or a seller;
/// `ERR_DATE_INVALID` on `from` or `to`; `ERR_DATE_RANGE` on `to` before `from`;
/// `ReportsError::Data` when the entries do not add up to the ledger.
pub(crate) fn whole(db: &Db, input: &StatementRowsInput) -> Result<Whole, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let day = |text: &Option<String>, field: &str| {
                text.as_deref()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(|t| parse_day(t, today).map_err(|code| reject(code, field)))
                    .transpose()
            };
            let from = day(&input.from, "from")?;
            let to = day(&input.to, "to")?;
            if let (Some(first), Some(last)) = (from, to)
                && first > last
            {
                return Err(reject(ErrorCode::ErrDateRange, "to"));
            }
            let (from, to) = (from.map(BusinessDate::to_iso), to.map(BusinessDate::to_iso));

            let not_found = || reject(ErrorCode::ErrNotFound, "account_id");
            let head_row = mandi_db::party_ledgers::head(conn, input.account_id)?
                .filter(|h| !h.is_system)
                .ok_or_else(not_found)?;
            let kind = match head_row.control_code.as_str() {
                BUYER_CONTROL => LedgerSide::Buyer,
                SELLER_CONTROL => LedgerSide::Seller,
                _ => return Err(not_found()),
            };
            let is_dead = mandi_db::dead::dead_set(conn, None)?.has_account(input.account_id);
            let head = PartyLedgerHead {
                account_id: input.account_id,
                code: head_row.party.code,
                name_ur: head_row.party.name_ur,
                name_en: head_row.party.name_en,
                tag: head_row.party.tag,
                kind,
                is_active: head_row.party.is_active,
                is_dead,
            };

            // What stood before the first day, and every line of the period.
            let filter = LineFilter {
                account: Some(input.account_id),
                person: None,
                from: from.clone(),
                to: to.clone(),
            };
            let (open_dr, open_cr) = match from.as_deref() {
                Some(first) => {
                    let (dr, cr) = lines::before_day(conn, &filter, first)?;
                    (paisa(dr)?, paisa(cr)?)
                }
                None => (Paisa::ZERO, Paisa::ZERO),
            };
            let brought_forward = Balance::from_totals(open_dr, open_cr)?;
            let mut entries = entries_of(lines::page(conn, &filter, (0, -1))?)?;
            // An opening is the first entry of its day (plan 78 B1); the rest keep the journal order.
            entries.sort_by_key(|e| (e.date.clone(), e.posting_type != "opening", e.id));

            let replaced = replaced_ids(&entries);
            let (mut all_dr, mut all_cr) = (0_i64, 0_i64);
            for entry in &entries {
                all_dr = all_dr.checked_add(entry.dr).ok_or_else(overflow)?;
                all_cr = all_cr.checked_add(entry.cr).ok_or_else(overflow)?;
            }
            let shown: Vec<&Entry> = entries
                .iter()
                .filter(|e| input.include_history || !replaced.contains(&e.id))
                .collect();

            // The items of the shown entries.
            let queries: Vec<DetailQuery> = shown
                .iter()
                .flat_map(|e| {
                    e.lines.iter().map(|(line_no, _, _)| DetailQuery {
                        entry_id: e.id,
                        line_no: *line_no,
                        bikri_base_entry: (e.source_type == "bikri")
                            .then(|| e.reverses.unwrap_or(e.id)),
                        voucher: e.voucher.clone(),
                    })
                })
                .collect();
            let mut details = load(conn, &queries, kind == LedgerSide::Buyer)?;

            // The last edit of the records behind the entries.
            let ids_of = |source: &str| -> Vec<i64> {
                let mut ids: Vec<i64> = shown
                    .iter()
                    .filter(|e| e.source_type == source)
                    .map(|e| e.source_id)
                    .collect();
                ids.sort_unstable();
                ids.dedup();
                ids
            };
            let mut edits: HashMap<(String, i64), mandi_db::edit_stamps::LastEdit> = HashMap::new();
            for (source, entity) in [("bikri", "bikri"), ("voucher", "voucher")] {
                let ids = ids_of(source);
                for (id, last) in mandi_db::edit_stamps::last_edits(conn, entity, &ids)? {
                    edits.insert((source.to_owned(), id), last);
                }
            }

            let paid_of = if kind == LedgerSide::Buyer {
                paid_on_spot(
                    conn,
                    input.account_id,
                    &shown.iter().map(|e| e.id).collect::<Vec<_>>(),
                )?
            } else {
                HashMap::new()
            };
            let (mut run_dr, mut run_cr) = (open_dr, open_cr);
            let (mut sum_dr, mut sum_cr, mut sum_spot) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
            let mut rows = Vec::with_capacity(shown.len());
            for entry in shown {
                let (dr, cr) = (paisa(entry.dr)?, paisa(entry.cr)?);
                let mut parts: Vec<DetailPart> = Vec::new();
                for (line_no, _, _) in &entry.lines {
                    for part in details.remove(&(entry.id, *line_no)).unwrap_or_default() {
                        // The money paid on the spot is `on_spot`, not a part of the row.
                        if !matches!(part, DetailPart::PaidOnSpot { .. }) {
                            parts.push(part);
                        }
                    }
                }
                // What a buyer paid on the spot is the value stored on his sale lines of the entry
                // (PRD FR-12.44), never worked out from the pairing of lines.
                let on_spot = paisa(paid_of.get(&entry.id).copied().unwrap_or(0))?;
                if on_spot > cr {
                    return Err(ReportsError::Data(format!(
                        "an entry of account {} was paid on the spot more than it credits",
                        input.account_id
                    )));
                }
                add(&mut run_dr, dr)?;
                add(&mut run_cr, cr)?;
                add(&mut sum_dr, dr)?;
                add(&mut sum_cr, cr)?;
                add(&mut sum_spot, on_spot)?;
                let edited = edits
                    .get(&(entry.source_type.clone(), entry.source_id))
                    .map(|last| StatementEdited {
                        at: last.at.clone(),
                        by: last.by.clone(),
                        count: last.count,
                        reason: last.reason.clone(),
                        sign: last.sign.clone(),
                    });
                rows.push(StatementRowsRow {
                    date: entry.date.clone(),
                    entry_id: entry.id,
                    type_id: ids::doc_id(&entry.posting_type),
                    number: entry.doc_number,
                    source: source_of(
                        &entry.source_type,
                        entry.source_id,
                        entry.id,
                        (entry.venture.0.as_deref(), entry.venture.1),
                    ),
                    dr_paisa: dr,
                    cr_paisa: cr,
                    on_spot_paisa: on_spot,
                    balance: Balance::from_totals(run_dr, run_cr)?,
                    short: short_of(&parts)?,
                    parts,
                    narration: entry.narration.clone(),
                    edited,
                    is_reversal: entry.reverses.is_some(),
                    is_replaced: replaced.contains(&entry.id),
                });
            }

            // The shown rows carry the party to the same closing as every line of the period: the
            // entries left out are pairs that net to zero. Anything else is an error, never a wrong
            // statement.
            let closing = Balance::from_totals(run_dr, run_cr)?;
            let whole = Balance::from_totals(
                open_dr.checked_add(paisa(all_dr)?)?,
                open_cr.checked_add(paisa(all_cr)?)?,
            )?;
            if closing != whole {
                return Err(ReportsError::Data(format!(
                    "the statement of account {} does not add up to his ledger",
                    input.account_id
                )));
            }
            Ok(Whole {
                head,
                from,
                to,
                brought_forward,
                rows,
                dr_total: sum_dr,
                cr_total: sum_cr,
                on_spot_total: sum_spot,
                closing,
            })
        })
    })
}

/// One page of the statement of a party, one row for each entry, with the totals over the whole
/// period (PRD FR-19.11). Read only.
///
/// # Errors
/// As [`whole`].
pub fn statement_rows(db: &Db, input: &StatementRowsInput) -> Result<StatementRows, ReportsError> {
    let mut whole = whole(db, input)?;
    let count = i64::try_from(whole.rows.len()).unwrap_or(i64::MAX);
    if input.newest_first {
        whole.rows.reverse();
    }
    let limit = match input.limit as usize {
        0 => PAGE,
        n => n.min(PAGE),
    };
    let rows = whole
        .rows
        .into_iter()
        .skip(input.offset as usize)
        .take(limit)
        .collect();
    Ok(StatementRows {
        head: whole.head,
        from: whole.from,
        to: whole.to,
        brought_forward: whole.brought_forward,
        rows,
        count,
        dr_total_paisa: whole.dr_total,
        cr_total_paisa: whole.cr_total,
        on_spot_total_paisa: whole.on_spot_total,
        closing: whole.closing,
    })
}
