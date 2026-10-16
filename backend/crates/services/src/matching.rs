//! Matching of credits to bills and of seller credits to advances (PRD FR-15.4, FR-15.5,
//! FR-15.8, FR-12.23, plan 13). The oldest-first working is the pure
//! `mandi_domain::matching::match_bills`; this module reads the lines, decides what is matched
//! and writes the rows. Matching rows are never deleted: a change switches the old row off with
//! who and when (Code Standards B17).
//!
//! # Bill matching of a buyer key: a canonical working
//!
//! A key is an account, plus the person key for a پختہ کھاتہ account (PRD FR-13.5). [`rematch`]
//! works out, in memory, what the active rows of the key must be, from every live line of the key
//! (money agent review of plan 13, point 1):
//!
//! 1. A **live** line is a line of a posted entry that is not reversed and is not itself a
//!    reversal. A reversed entry and its reversal cancel each other, so neither is a bill or a
//!    credit (cancel a بکری while an older bill is open: the older bill stays open and keeps its
//!    age).
//! 2. A **pinned** row pairs the bill of a نقدی sale line with its own cash credit of the same
//!    بکری posting (PRD FR-12.9). It is written by the save ([`pin_cash_sale`]) and a re-match
//!    never changes it; only the reversal of its entry switches it off. Its amounts come out of
//!    both pools first, so the cash of today's sale never pays an older bill.
//! 3. What is left goes through `match_bills`: each credit, oldest first, settles the oldest
//!    open bills; a credit that is more than all open bills stays unmatched and settles his next
//!    bills (PRD FR-15.5).
//! 4. The rows that differ from the active rows are switched off and the missing ones inserted.
//!    For an append (a new بکری or a receipt) that is inserts only. After an edit or a cancel it
//!    gives the same result as working the key from the start, so aging is right.
//!
//! Example (money agent 8.1): bills 12,880 (09-10) and 5,000 (10-10); a receipt of 15,000
//! settles 12,880 and 2,120.
//!
//! # Appending to a key: the fast paths
//!
//! A new بکری or a receipt only adds lines at the end of a key. When the rows of the key are
//! already the canonical ones and no entry of the key is dated later than the new one, the
//! canonical result of the longer key is the old result plus the matching of the new lines, and
//! only the unmatched side has to be read: [`match_new_bills`] reads the unmatched credits
//! (credit left over means every old bill is full, so the new bills are the only open ones) and
//! [`match_new_credits`] reads the open bills (bill left over means no old credit is unmatched).
//! A buyer with 55,000 lines costs a few milliseconds instead of reading all of them (NFR-SPD-8,
//! NFR-SPD-9). If an entry of the key is dated later (a back-dated posting), the new lines are
//! not the newest, and both fall back to the full [`rematch`]. The test
//! `after_any_sequence_of_sales_and_receipts_the_rows_are_the_canonical_ones` runs the fast paths
//! and then the full working, which must change nothing.
//!
//! The fast paths rely on the rows being canonical before. Every writer of this crate keeps them
//! so. Lines that come from outside (an import of old data) must be matched once with
//! [`rematch`] for each key before the app is used.
//!
//! # Advances
//!
//! [`adjust_advances`] matches ONE seller credit line (C8, not the نقد, which is a separate debit
//! line) to the seller's open advances, oldest first. The save calls it for a lot whose tick
//! "پیشگی سے کاٹیں" is on (PRD FR-12.23). [`rematch_advances`] is the canonical full working for
//! a seller, for when an advance or a credit changes behind the rows (an opening balance that is
//! corrected, plan 16's cancel): it knows the credits of a first posting (with the tick of its
//! lot) and of a seller's receipt; a row on any other credit (a later sale, plan 16) is kept as
//! it is until that plan teaches it.

use std::collections::{BTreeMap, HashMap};

use mandi_db::matching as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::money::Paisa;
use mandi_domain::matching::{Allocation, Open, Ranked, match_before, match_bills};

use crate::error::{ServiceError, money_error};

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// What a re-match changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rematched {
    /// Rows switched off.
    pub switched_off: usize,
    /// Rows added.
    pub added: usize,
}

/// The bill of a نقدی sale line and its own cash credit.
#[derive(Debug, Clone, Copy)]
pub struct CashSalePair<'a> {
    /// The buyer account.
    pub account_id: i64,
    /// The person key, for a پختہ کھاتہ buyer.
    pub person_key: Option<&'a str>,
    /// The buyer's debit line of the sale.
    pub bill_line_id: i64,
    /// The buyer's credit line of the cash part.
    pub payment_line_id: i64,
    /// The buyer total of the line.
    pub amount: Paisa,
}

/// Writes the pinned pair of a نقدی sale line: its bill and its own cash credit, for the whole
/// buyer total (PRD FR-12.9, FR-15.4). Called by the save, once for each cash line, before
/// [`rematch`].
///
/// # Errors
/// `ServiceError::Db`: the triggers refuse lines of the wrong direction, account or person key
/// and an amount above a line (a bug signal).
pub fn pin_cash_sale(
    tx: &Connection,
    user_id: i64,
    now: &str,
    pair: &CashSalePair<'_>,
) -> Result<(), ServiceError> {
    repo::insert_bill_allocation(
        tx,
        &repo::NewBillAllocation {
            bill_line_id: pair.bill_line_id,
            payment_line_id: pair.payment_line_id,
            account_id: pair.account_id,
            person_key: pair.person_key,
            amount_paisa: pair.amount.as_paisa(),
        },
        user_id,
        now,
    )?;
    Ok(())
}

/// Works out the canonical matching of a buyer key and makes the active rows equal to it (see
/// the module text). Only buyer keys are matched; the caller does not pass a seller.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` if a pinned row is bigger than its line (a bug
/// signal); `ERR_OVERFLOW`.
pub fn rematch(
    tx: &Connection,
    user_id: i64,
    now: &str,
    account_id: i64,
    person_key: Option<&str>,
) -> Result<Rematched, ServiceError> {
    let lines = repo::live_key_lines(tx, account_id, person_key)?;
    let active = repo::active_bill_rows(tx, account_id, person_key)?;
    let live: std::collections::HashSet<i64> = lines.iter().map(|l| l.line_id).collect();

    // Pinned rows take their amounts out of both pools. A pinned row that points at a line that
    // is no longer live is stale and is switched off with the others.
    let mut taken: HashMap<i64, Paisa> = HashMap::new();
    let mut keep: Vec<repo::ActiveRow> = Vec::new();
    let mut stale: Vec<i64> = Vec::new();
    for row in &active {
        if row.pinned && live.contains(&row.bill_line_id) && live.contains(&row.payment_line_id) {
            let amount = paisa(row.amount_paisa)?;
            for line in [row.bill_line_id, row.payment_line_id] {
                let slot = taken.entry(line).or_insert(Paisa::ZERO);
                *slot = slot.checked_add(amount).map_err(money_error)?;
            }
            keep.push(*row);
        } else if row.pinned {
            stale.push(row.id);
        }
    }
    let mut bills: Vec<Open> = Vec::new();
    let mut credits: Vec<Open> = Vec::new();
    for line in &lines {
        let used = taken.get(&line.line_id).copied().unwrap_or(Paisa::ZERO);
        if line.dr_paisa > 0 {
            let open = paisa(line.dr_paisa)?
                .checked_sub(used)
                .map_err(money_error)?;
            if open.is_negative() {
                return Err(ServiceError::Data(format!(
                    "pinned rows are bigger than bill line {}",
                    line.line_id
                )));
            }
            bills.push(Open {
                line_id: line.line_id,
                open,
            });
        } else if line.cr_paisa > 0 {
            let open = paisa(line.cr_paisa)?
                .checked_sub(used)
                .map_err(money_error)?;
            if open.is_negative() {
                return Err(ServiceError::Data(format!(
                    "pinned rows are bigger than credit line {}",
                    line.line_id
                )));
            }
            credits.push(Open {
                line_id: line.line_id,
                open,
            });
        }
    }
    let wanted = match_bills(&bills, &credits).map_err(money_error)?;

    // The rows that are not pinned are compared pair by pair with what is wanted.
    let mut wanted_by_pair: BTreeMap<(i64, i64), Allocation> = BTreeMap::new();
    for row in &wanted {
        wanted_by_pair.insert((row.bill_line_id, row.credit_line_id), *row);
    }
    let mut done = Rematched::default();
    let mut kept_pairs: std::collections::HashSet<(i64, i64)> = std::collections::HashSet::new();
    let mut to_switch_off: Vec<i64> = stale;
    for row in active.iter().filter(|r| !r.pinned) {
        let pair = (row.bill_line_id, row.payment_line_id);
        let same = wanted_by_pair
            .get(&pair)
            .is_some_and(|w| w.amount.as_paisa() == row.amount_paisa)
            && !kept_pairs.contains(&pair);
        if same {
            kept_pairs.insert(pair);
        } else {
            to_switch_off.push(row.id);
        }
    }
    // Switch off first, so the amount checks of the database see only the rows that stay.
    for id in to_switch_off {
        if repo::deactivate_bill_allocation(tx, id, user_id, now)? {
            done.switched_off = done.switched_off.saturating_add(1);
        }
    }
    for row in wanted {
        if kept_pairs.contains(&(row.bill_line_id, row.credit_line_id)) {
            continue;
        }
        repo::insert_bill_allocation(
            tx,
            &repo::NewBillAllocation {
                bill_line_id: row.bill_line_id,
                payment_line_id: row.credit_line_id,
                account_id,
                person_key,
                amount_paisa: row.amount.as_paisa(),
            },
            user_id,
            now,
        )?;
        done.added = done.added.saturating_add(1);
    }
    Ok(done)
}

/// A new line of a key, with its amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewLine {
    /// The journal line just posted.
    pub line_id: i64,
    /// Its amount.
    pub amount: Paisa,
}

fn open_of(lines: Vec<repo::LineMatch>) -> Result<Vec<Open>, ServiceError> {
    lines
        .into_iter()
        .map(|line| {
            Ok(Open {
                line_id: line.line_id,
                open: paisa(line.amount_paisa)?
                    .checked_sub(paisa(line.matched_paisa)?)
                    .map_err(money_error)?,
            })
        })
        .collect()
}

fn insert_rows(
    tx: &Connection,
    user_id: i64,
    now: &str,
    account_id: i64,
    person_key: Option<&str>,
    rows: &[Allocation],
) -> Result<Rematched, ServiceError> {
    for row in rows {
        repo::insert_bill_allocation(
            tx,
            &repo::NewBillAllocation {
                bill_line_id: row.bill_line_id,
                payment_line_id: row.credit_line_id,
                account_id,
                person_key,
                amount_paisa: row.amount.as_paisa(),
            },
            user_id,
            now,
        )?;
    }
    Ok(Rematched {
        switched_off: 0,
        added: rows.len(),
    })
}

/// Matches new bills (debit lines just posted, dated `entry_date`) of a key to the credits that
/// are still unmatched, oldest first (PRD FR-15.5): a prepayment settles the next bill. Gives
/// what [`rematch`] would give, without reading the history of the key (see the module text).
///
/// # Errors
/// As [`rematch`].
pub fn match_new_bills(
    tx: &Connection,
    user_id: i64,
    now: &str,
    account_id: i64,
    person_key: Option<&str>,
    entry_date: &str,
    bills: &[NewLine],
) -> Result<Rematched, ServiceError> {
    if repo::has_later_entry(tx, account_id, person_key, entry_date)? {
        return rematch(tx, user_id, now, account_id, person_key);
    }
    let credits = open_of(repo::open_credits(tx, account_id, person_key)?)?;
    if credits.is_empty() {
        return Ok(Rematched::default());
    }
    let bills: Vec<Open> = bills
        .iter()
        .map(|b| Open {
            line_id: b.line_id,
            open: b.amount,
        })
        .collect();
    let rows = match_bills(&bills, &credits).map_err(money_error)?;
    insert_rows(tx, user_id, now, account_id, person_key, &rows)
}

/// Matches new credits (credit lines just posted, dated `entry_date`) of a key to the bills that
/// are still open, oldest first (PRD FR-15.4): a receipt settles the oldest bills first. Gives
/// what [`rematch`] would give, without reading the history of the key (see the module text).
///
/// # Errors
/// As [`rematch`].
pub fn match_new_credits(
    tx: &Connection,
    user_id: i64,
    now: &str,
    account_id: i64,
    person_key: Option<&str>,
    entry_date: &str,
    credits: &[NewLine],
) -> Result<Rematched, ServiceError> {
    if repo::has_later_entry(tx, account_id, person_key, entry_date)? {
        return rematch(tx, user_id, now, account_id, person_key);
    }
    let bills = open_of(repo::open_bills(tx, account_id, person_key)?)?;
    if bills.is_empty() {
        return Ok(Rematched::default());
    }
    let credits: Vec<Open> = credits
        .iter()
        .map(|c| Open {
            line_id: c.line_id,
            open: c.amount,
        })
        .collect();
    let rows = match_bills(&bills, &credits).map_err(money_error)?;
    insert_rows(tx, user_id, now, account_id, person_key, &rows)
}

/// What the active advance rows take from one credit line, paisa.
fn adjusted_by(
    tx: &Connection,
    account_id: i64,
    credit_line_id: i64,
) -> Result<Paisa, ServiceError> {
    repo::active_advance_rows(tx, account_id)?
        .iter()
        .filter(|row| row.credit_line_id == credit_line_id)
        .try_fold(Paisa::ZERO, |sum, row| {
            sum.checked_add(paisa(row.amount_paisa)?)
                .map_err(money_error)
        })
}

/// Adjusts a seller's open advances, oldest first, by one seller credit line of a بکری posting
/// (PRD FR-15.8, FR-12.23): the amount adjusted is `min(credit, open advances)` and goes into
/// `advance_allocations` row by row. Only the advances that come before the credit are used.
/// Returns the total adjusted (0 when he has no open advance).
///
/// # Errors
/// `ServiceError::Db`: the triggers refuse a credit that is not a seller credit of a بکری or a
/// receipt, and an amount above a line (a bug signal); `ERR_OVERFLOW`.
pub fn adjust_advances(
    tx: &Connection,
    user_id: i64,
    now: &str,
    account_id: i64,
    credit_line_id: i64,
    credit: Paisa,
    entry_date: &str,
) -> Result<Paisa, ServiceError> {
    // A credit adjusts only the advances before it. When an entry of the seller is dated later
    // (a back-dated بکری) the new credit is not the newest line, so what it may take is worked
    // out again from the start; the answer is the same whenever it is worked out.
    if repo::has_later_entry(tx, account_id, None, entry_date)? {
        rematch_advances(tx, user_id, now, account_id)?;
        return adjusted_by(tx, account_id, credit_line_id);
    }
    let mut advances = Vec::new();
    for line in repo::open_advances(tx, account_id)? {
        let open = paisa(line.amount_paisa)?
            .checked_sub(paisa(line.matched_paisa)?)
            .map_err(money_error)?;
        advances.push(Open {
            line_id: line.line_id,
            open,
        });
    }
    let credits = [Open {
        line_id: credit_line_id,
        open: credit,
    }];
    let rows = match_bills(&advances, &credits).map_err(money_error)?;
    let mut total = Paisa::ZERO;
    for row in rows {
        repo::insert_advance_allocation(
            tx,
            &repo::NewAdvanceAllocation {
                advance_line_id: row.bill_line_id,
                credit_line_id: row.credit_line_id,
                account_id,
                person_key: None,
                amount_paisa: row.amount.as_paisa(),
            },
            user_id,
            now,
        )?;
        total = total.checked_add(row.amount).map_err(money_error)?;
    }
    Ok(total)
}

/// The credit of a lot's first posting (PRD C8): the seller amounts of its lines less their
/// commissions and the lot charges that come off the seller's credit. نقد is a separate line.
fn first_credit(lot: &repo::FirstLot) -> Result<Paisa, ServiceError> {
    let freight = if lot.freight_by_firm {
        paisa(lot.freight_paisa)?
    } else {
        Paisa::ZERO
    };
    let mut charges = freight;
    for part in [
        lot.labour_paisa,
        lot.munshiana_paisa,
        lot.store_rent_paisa,
        lot.phone_paisa,
    ] {
        charges = charges.checked_add(paisa(part)?).map_err(money_error)?;
    }
    paisa(lot.gross_paisa)?
        .checked_sub(paisa(lot.comm_paisa)?)
        .and_then(|c| c.checked_sub(charges))
        .map_err(money_error)
}

/// Works out the canonical advance adjustment of a seller and makes the active rows equal to it:
/// his live advance lines (oldest first) are adjusted, oldest first, by the credit lines that
/// can adjust (PRD FR-15.8): the seller credit of the first posting of each lot whose tick is on
/// (PRD FR-12.23), and the credit of a receipt from him (PRD FR-15.12). A row on any other
/// credit is kept and its amount comes out of the advance first, so a source this working does
/// not know yet is never switched off.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` if the credit lines of a posting and the lots stored
/// for it do not line up (a bug signal); `ERR_OVERFLOW`.
pub fn rematch_advances(
    tx: &Connection,
    user_id: i64,
    now: &str,
    account_id: i64,
) -> Result<Rematched, ServiceError> {
    let advances = repo::live_advance_lines(tx, account_id)?;
    let credit_lines = repo::seller_credit_lines(tx, account_id)?;
    // The place of each line in the order of the account: a credit adjusts only the advances
    // before it (PRD FR-15.8, "each later seller credit").
    let rank_of: HashMap<i64, usize> = repo::live_lines(tx, account_id)?
        .iter()
        .enumerate()
        .map(|(rank, line)| (line.line_id, rank))
        .collect();
    let rank = |line_id: i64| -> Result<usize, ServiceError> {
        rank_of.get(&line_id).copied().ok_or_else(|| {
            ServiceError::Data(format!("line {line_id} of an advance is not a live line"))
        })
    };
    let lots = repo::first_lots(tx, account_id)?;
    let active = repo::active_advance_rows(tx, account_id)?;

    // Which credit line belongs to which lot of its posting: a lot with a credit above 0 has one
    // credit line, and lots and lines are both in lot order (PRD C8).
    let mut ticked: HashMap<i64, bool> = HashMap::new();
    let mut by_entry: BTreeMap<i64, Vec<&repo::SellerCredit>> = BTreeMap::new();
    for line in &credit_lines {
        if line.posting_type == "receipt" {
            ticked.insert(line.line_id, true);
        } else {
            by_entry.entry(line.entry_id).or_default().push(line);
        }
    }
    for (entry_id, lines) in &by_entry {
        let mut with_credit: Vec<(&repo::FirstLot, Paisa)> = Vec::new();
        for lot in lots.iter().filter(|l| l.entry_id == *entry_id) {
            let credit = first_credit(lot)?;
            if credit > Paisa::ZERO {
                with_credit.push((lot, credit));
            }
        }
        if with_credit.len() != lines.len() {
            return Err(ServiceError::Data(format!(
                "entry {entry_id}: {} credit lines of the seller for {} lots",
                lines.len(),
                with_credit.len()
            )));
        }
        for ((lot, credit), line) in with_credit.into_iter().zip(lines) {
            if paisa(line.amount_paisa)? != credit {
                return Err(ServiceError::Data(format!(
                    "entry {entry_id}: a credit line is not the credit of its lot"
                )));
            }
            ticked.insert(line.line_id, lot.adjust_advance);
        }
    }

    // Rows on a credit this working does not know are kept; their amounts come out of the pools.
    let known: std::collections::HashSet<i64> = credit_lines.iter().map(|c| c.line_id).collect();
    let live_advance: std::collections::HashSet<i64> = advances.iter().map(|a| a.line_id).collect();
    let mut taken: HashMap<i64, Paisa> = HashMap::new();
    for row in active.iter().filter(|r| !known.contains(&r.credit_line_id)) {
        if live_advance.contains(&row.advance_line_id) {
            let slot = taken.entry(row.advance_line_id).or_insert(Paisa::ZERO);
            *slot = slot
                .checked_add(paisa(row.amount_paisa)?)
                .map_err(money_error)?;
        }
    }
    let mut open_advances: Vec<Ranked> = Vec::new();
    for line in &advances {
        let open = paisa(line.dr_paisa)?
            .checked_sub(taken.get(&line.line_id).copied().unwrap_or(Paisa::ZERO))
            .map_err(money_error)?;
        if open.is_negative() {
            return Err(ServiceError::Data(format!(
                "rows are bigger than advance line {}",
                line.line_id
            )));
        }
        open_advances.push(Ranked {
            line: Open {
                line_id: line.line_id,
                open,
            },
            rank: rank(line.line_id)?,
        });
    }
    let credits: Vec<Ranked> = credit_lines
        .iter()
        .filter(|c| ticked.get(&c.line_id).copied().unwrap_or(false))
        .map(|c| {
            Ok(Ranked {
                line: Open {
                    line_id: c.line_id,
                    open: paisa(c.amount_paisa)?,
                },
                rank: rank(c.line_id)?,
            })
        })
        .collect::<Result<_, ServiceError>>()?;
    let wanted = match_before(&open_advances, &credits).map_err(money_error)?;

    let mut wanted_by_pair: BTreeMap<(i64, i64), Allocation> = BTreeMap::new();
    for row in &wanted {
        wanted_by_pair.insert((row.bill_line_id, row.credit_line_id), *row);
    }
    let mut done = Rematched::default();
    let mut kept: std::collections::HashSet<(i64, i64)> = std::collections::HashSet::new();
    let mut to_switch_off: Vec<i64> = Vec::new();
    for row in active.iter().filter(|r| known.contains(&r.credit_line_id)) {
        let pair = (row.advance_line_id, row.credit_line_id);
        let same = wanted_by_pair
            .get(&pair)
            .is_some_and(|w| w.amount.as_paisa() == row.amount_paisa)
            && !kept.contains(&pair);
        if same {
            kept.insert(pair);
        } else {
            to_switch_off.push(row.id);
        }
    }
    for id in to_switch_off {
        if repo::deactivate_advance_allocation(tx, id, user_id, now)? {
            done.switched_off = done.switched_off.saturating_add(1);
        }
    }
    for row in wanted {
        if kept.contains(&(row.bill_line_id, row.credit_line_id)) {
            continue;
        }
        repo::insert_advance_allocation(
            tx,
            &repo::NewAdvanceAllocation {
                advance_line_id: row.bill_line_id,
                credit_line_id: row.credit_line_id,
                account_id,
                person_key: None,
                amount_paisa: row.amount.as_paisa(),
            },
            user_id,
            now,
        )?;
        done.added = done.added.saturating_add(1);
    }
    Ok(done)
}
