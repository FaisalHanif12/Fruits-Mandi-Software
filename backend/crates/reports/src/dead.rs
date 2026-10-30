//! R38 ڈوبی رقم رپورٹ, the dead amount (PRD 19.2 R38, FR-10.10, FR-19.9, plan 40) and the tile
//! ڈوبی رقم of the dashboard (PRD 9.1).
//!
//! One row for each move of a person to the dead amount. Nothing is stored here except the move
//! itself (`dead_marks`, migration 0011): what he owes now is his balance, read from the journal
//! (PRD FR-6.1). The money of a row is worked out by `mandi_core::dead::dead_money` from the sums
//! `mandi_db::dead::figures` reads, so the page, the tile and the report print can never differ:
//!
//! * **owed then** = what he owed on the day of the move (kept in the move);
//! * **recovered** = what a وصولی or an ادھار رقم واپسی put on his credit since, less a recovery
//!   that was cancelled or edited (cancelled 4,000 shows 0, edited to 3,000 shows 3,000);
//! * **corrections** = every other movement since: an edit, cancel or undo of an earlier posting
//!   (PRD A67), also a recovery saved before the move;
//! * **still owed** = his balance now, never below 0. The total of the still owed with the filter
//!   `owed` is the tile ڈوبی رقم.
//!
//! Example (PRD E2E-44): ارشد, moved on 11-10-26 with 18,684, a وصولی of 4,000 on 12-10-26: owed then
//! 18,684, recovered 4,000, corrections 0, still owed 14,684, last recovery 12-10-26.

use std::collections::HashMap;

use mandi_core::accounts::{BUYER_CONTROL, MONEY_LENT_CONTROL, SELLER_CONTROL};
use mandi_core::dead::{
    DeadAmountInput, DeadAmountPage, DeadFilter, DeadMoney, DeadPartyType, DeadReason, DeadRow,
    DeadStatus, DeadTotals, dead_money, dead_status,
};
use mandi_core::money::Paisa;
use mandi_core::text::normalize_urdu;
use mandi_db::dead::{self as repo, Figures, MarkRow};
use mandi_db::rusqlite::Connection;
use mandi_db::{Db, rusqlite};

use crate::error::ReportsError;
use crate::reports::PartyRef;
use crate::reports::collections::pukhta_names;
use crate::reports::common::{add, in_snapshot, load_parties, paisa};

/// The most rows of one page (Code Standards B31).
const MAX_LIMIT: i64 = 100;

fn reason_of(text: &str) -> DeadReason {
    match text {
        "died" => DeadReason::Died,
        "left" => DeadReason::Left,
        _ => DeadReason::Other,
    }
}

/// The control code of each account of these moves, to say whether a person is a buyer, a seller
/// or a person with ادھار رقم.
fn controls(
    conn: &Connection,
    ids: &[i64],
) -> Result<HashMap<i64, (String, bool)>, rusqlite::Error> {
    let mut found = HashMap::new();
    let mut statement = conn.prepare(
        "SELECT a.id, k.code, a.is_pukhta_khata FROM accounts a
         JOIN account_controls k ON k.id = a.control_id",
    )?;
    let wanted: std::collections::HashSet<i64> = ids.iter().copied().collect();
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)? == 1,
        ))
    })?;
    for row in rows {
        let (id, code, pukhta) = row?;
        if wanted.contains(&id) {
            found.insert(id, (code, pukhta));
        }
    }
    Ok(found)
}

fn party_type(control: &str, pukhta: bool) -> DeadPartyType {
    if pukhta {
        DeadPartyType::Pukhta
    } else if control == SELLER_CONTROL {
        DeadPartyType::Seller
    } else if control == MONEY_LENT_CONTROL {
        DeadPartyType::MoneyLent
    } else {
        // A buyer. A control a move is never made for cannot be here (the move refuses it).
        DeadPartyType::Buyer
    }
}

fn figures_money(mark: &MarkRow, f: &Figures) -> Result<DeadMoney, ReportsError> {
    Ok(dead_money(
        paisa(mark.owed_paisa)?,
        paisa(f.balance_dr)?,
        paisa(f.balance_cr)?,
        paisa(f.recovery_dr)?,
        paisa(f.recovery_cr)?,
    )?)
}

/// Every move as a row, oldest first, with its money worked out.
fn build(conn: &Connection, marks: &[MarkRow]) -> Result<Vec<DeadRow>, ReportsError> {
    let figures = repo::figures(conn)?;
    let last = repo::last_recovery_dates(conn)?;
    let account_ids: Vec<i64> = marks.iter().map(|m| m.account_id).collect();
    let parties: HashMap<i64, PartyRef> = load_parties(conn, &account_ids)?;
    let kinds = controls(conn, &account_ids)?;
    let pukhta_ids: Vec<i64> = marks
        .iter()
        .filter(|m| m.person_key.is_some())
        .map(|m| m.account_id)
        .collect();
    let names = pukhta_names(conn, &pukhta_ids)?;

    let mut out = Vec::with_capacity(marks.len());
    for mark in marks {
        let party = parties
            .get(&mark.account_id)
            .ok_or_else(|| ReportsError::Data(format!("account {} is missing", mark.account_id)))?;
        let (control, pukhta) = kinds
            .get(&mark.account_id)
            .cloned()
            .unwrap_or_else(|| (BUYER_CONTROL.to_owned(), false));
        let f = figures.get(&mark.id).copied().unwrap_or_default();
        let money = figures_money(mark, &f)?;
        let status = dead_status(mark.is_active, &money);
        // An undone move is no longer owed: its figures stop on the day it was undone and its
        // still owed is 0, so the total of the still owed is the tile under every filter.
        let still_owed = if status == DeadStatus::Undone {
            Paisa::ZERO
        } else {
            money.still_owed
        };
        let person = mark.person_key.as_deref().and_then(|k| names.get(k));
        out.push(DeadRow {
            mark_id: mark.id,
            account_id: mark.account_id,
            code: party.code.clone(),
            name_ur: party.name_ur.clone(),
            name_en: party.name_en.clone(),
            tag: party.tag.clone(),
            party_type: party_type(&control, pukhta),
            pukhta_person_key: mark.person_key.clone(),
            pukhta_name_ur: person.map(|p| p.name_ur.clone()),
            pukhta_name_en: person.map(|p| p.name_en.clone()),
            reason: reason_of(&mark.reason),
            note: mark.note.clone(),
            entry_date: mark.entry_date.clone(),
            owed_then_paisa: paisa(mark.owed_paisa)?,
            recovered_paisa: money.recovered,
            corrections_paisa: money.corrections.as_paisa(),
            still_owed_paisa: still_owed,
            last_recovery_date: last.get(&mark.id).cloned(),
            status,
        });
    }
    Ok(out)
}

/// True when the row matches the text the user typed: code, tag, either name, the names of a
/// one-time person, or his phone (PRD FR-19.11 search).
fn matches(row: &DeadRow, search: &str) -> bool {
    let needle = search.trim();
    if needle.is_empty() {
        return true;
    }
    let lower = needle.to_lowercase();
    let urdu = normalize_urdu(needle);
    let en = |text: &str| text.to_lowercase().contains(&lower);
    let ur = |text: &str| !urdu.is_empty() && normalize_urdu(text).contains(&urdu);
    en(&row.code)
        || row.tag.as_deref().is_some_and(en)
        || en(&row.name_en)
        || ur(&row.name_ur)
        || row.pukhta_name_en.as_deref().is_some_and(en)
        || row.pukhta_name_ur.as_deref().is_some_and(ur)
        || row.pukhta_person_key.as_deref().is_some_and(en)
}

/// One page of R38 with the totals over the whole filter (never a sum of the page, Code Standards
/// B31). Reads in one snapshot.
///
/// # Errors
/// `ReportsError::Rejected` for an `offset` below 0 or a `limit` outside 1 to 100;
/// `ReportsError::Money` if a stored amount is not whole rupees or a total does not fit.
pub fn page(db: &Db, input: &DeadAmountInput) -> Result<DeadAmountPage, ReportsError> {
    if input.offset < 0 || input.limit < 1 || input.limit > MAX_LIMIT {
        let mut problems = Vec::new();
        if input.offset < 0 {
            problems.push(mandi_core::error::FieldError::new(
                mandi_core::error::ErrorCode::ErrOutOfRange,
                "offset",
            ));
        }
        if input.limit < 1 || input.limit > MAX_LIMIT {
            problems.push(mandi_core::error::FieldError::new(
                mandi_core::error::ErrorCode::ErrOutOfRange,
                "limit",
            ));
        }
        return Err(ReportsError::Rejected(problems));
    }
    db.read(|conn| in_snapshot(conn, |conn| page_in(conn, input)))
}

/// [`page`] on a connection that is already in a snapshot.
pub(crate) fn page_in(
    conn: &Connection,
    input: &DeadAmountInput,
) -> Result<DeadAmountPage, ReportsError> {
    let (rows, totals) = filtered(conn, input.filter, &input.search)?;
    let row_count = i64::try_from(rows.len())
        .map_err(|_| ReportsError::Data("the number of rows does not fit".to_owned()))?;
    let start = usize::try_from(input.offset).unwrap_or(usize::MAX);
    let take = usize::try_from(input.limit).unwrap_or(0);
    Ok(DeadAmountPage {
        rows: rows.into_iter().skip(start).take(take).collect(),
        row_count,
        totals,
    })
}

/// Every row of the filter, oldest move first, and the totals over all of them. The engine (print
/// and Excel) reads this too.
///
/// # Errors
/// As [`page`].
pub(crate) fn filtered(
    conn: &Connection,
    filter: DeadFilter,
    search: &str,
) -> Result<(Vec<DeadRow>, DeadTotals), ReportsError> {
    let marks = match filter {
        DeadFilter::Owed => repo::active_marks(conn)?,
        DeadFilter::All => repo::all_marks(conn)?,
    };
    let built = build(conn, &marks)?;
    let mut rows = Vec::new();
    let mut totals = DeadTotals {
        owed_then_paisa: Paisa::ZERO,
        recovered_paisa: Paisa::ZERO,
        corrections_paisa: 0,
        still_owed_paisa: Paisa::ZERO,
    };
    for row in built {
        // "Still owed" lists only the people who still owe something.
        if filter == DeadFilter::Owed && row.status != DeadStatus::Owing {
            continue;
        }
        if !matches(&row, search) {
            continue;
        }
        add(&mut totals.owed_then_paisa, row.owed_then_paisa)?;
        add(&mut totals.recovered_paisa, row.recovered_paisa)?;
        totals.corrections_paisa = totals
            .corrections_paisa
            .checked_add(row.corrections_paisa)
            .ok_or(mandi_core::money::MoneyError::Overflow)?;
        add(&mut totals.still_owed_paisa, row.still_owed_paisa)?;
        rows.push(row);
    }
    Ok((rows, totals))
}

/// The tile ڈوبی رقم (PRD 9.1): what the people in the dead amount still owe, and how many of them
/// still owe something. The same sums as R38 with the filter `owed`, so the tile equals the R38
/// total (PRD FR-19.9).
///
/// # Errors
/// `ReportsError::Money` if a stored amount is not whole rupees or a sum does not fit.
pub(crate) fn tile(conn: &Connection) -> Result<(Paisa, i64), ReportsError> {
    let marks = repo::active_marks(conn)?;
    if marks.is_empty() {
        return Ok((Paisa::ZERO, 0));
    }
    let figures = repo::figures(conn)?;
    let mut total = Paisa::ZERO;
    let mut count = 0_i64;
    for mark in &marks {
        let f = figures.get(&mark.id).copied().unwrap_or_default();
        let money = figures_money(mark, &f)?;
        if money.still_owed > Paisa::ZERO {
            add(&mut total, money.still_owed)?;
            count = count
                .checked_add(1)
                .ok_or_else(|| ReportsError::Data("the dead count does not fit".to_owned()))?;
        }
    }
    Ok((total, count))
}
