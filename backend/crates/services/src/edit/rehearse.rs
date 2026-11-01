//! The rehearsal of an edit: the real edit function runs inside a transaction that is always rolled
//! back, and what it did is read before the rollback (PRD FR-7.70 "Shown before save", Code
//! Standards B5: the preview and the save are the same function).
//!
//! What the rehearsal reads:
//! - the stored row of the entry before and after (the screen's change list is their difference);
//! - the party accounts the edit's postings touched, with their balance before and after;
//! - cash in hand before and after, and whether the edit needs the cash question of PRD FR-6.7;
//! - each partner's share of the posting (his share rows plus his own accounts' lines of the posting)
//!   before and after: the posting being edited, and the posting that replaced it.
//!
//! "Before" is worked out from the same transaction: the lines the edit wrote all belong to entries
//! with a higher id than any entry that existed when it began, so the books before the edit are the
//! books without them. Money is only read and subtracted with checked arithmetic.
//!
//! Nothing stays: the rolled-back transaction leaves no row, no audit entry and no number used. The
//! remembered sums of the ledger (`mandi_db::memo`) are never touched: only the read connection uses
//! them, and this runs on the write connection.

use std::collections::BTreeMap;

use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::{Connection, params};
use mandi_domain::cash;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::edit::{EffectCash, EffectPartner, EffectParty};
use mandi_domain::core::money::Paisa;

use crate::cash::{CASH_ACCOUNT_CODE, cash_in_hand};
use crate::context::Context;
use crate::error::{ServiceError, money_error};
use crate::ledger;

/// The marker of the error that ends a rehearsal. It never leaves this module.
const ROLLBACK: &str = "rehearsal: always rolled back";

/// The posting an edit replaces and where it comes from (PRD Architecture 15.7).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Scope<'a> {
    /// `voucher`, `fruit`, `purchase`, `venture`, `bikri` ...
    pub source_type: &'a str,
    /// The id of the entry in its own table.
    pub source_id: i64,
    /// The journal entry being edited: the posting in force before the edit. The partners' figures
    /// are of this posting and of the one that takes its place, not of the whole entry (a بکری has
    /// several postings).
    pub posting_id: i64,
}

/// What a rehearsal found, apart from the rows.
#[derive(Debug, Clone, Default)]
pub(crate) struct Facts {
    /// Each party whose balance the edit moved.
    pub parties: Vec<EffectParty>,
    /// Cash in hand, when the edit moved it.
    pub cash: Option<EffectCash>,
    /// Each partner's share of the entry, when it changed.
    pub partners: Vec<EffectPartner>,
}

/// The result of a rehearsal.
pub(crate) struct Rehearsed<R> {
    /// The stored row before the edit.
    pub before: R,
    /// The stored row after the edit.
    pub after: R,
    /// What the edit did to the books.
    pub facts: Facts,
}

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// Runs `body` (an edit) and rolls everything back. `row` reads the stored row of the entry. If
/// `body` returns an error it is returned as it is, and nothing was written either.
///
/// # Errors
/// What `body` returns; `ServiceError::Db` and `ServiceError::Data`.
pub(crate) fn rehearse<R, T>(
    ctx: &Context,
    scope: Scope<'_>,
    row: impl Fn(&Connection) -> Result<R, ServiceError>,
    body: impl FnOnce(&Connection) -> Result<T, ServiceError>,
) -> Result<Rehearsed<R>, ServiceError> {
    let mut found: Option<Rehearsed<R>> = None;
    let outcome = ctx.db.write(|tx| {
        let max_before: i64 = tx.query_row(
            "SELECT COALESCE(MAX(id), 0) FROM journal_entries",
            [],
            |r| r.get(0),
        )?;
        let before = row(tx)?;
        let partners_before = partner_sums(tx, scope, Postings::Old)?;
        body(tx)?;
        let after = row(tx)?;
        let facts = collect(tx, scope, max_before, &partners_before)?;
        found = Some(Rehearsed {
            before,
            after,
            facts,
        });
        Err::<(), ServiceError>(data(ROLLBACK))
    });
    match outcome {
        Err(ServiceError::Data(message)) if message == ROLLBACK => {
            found.ok_or_else(|| data("a rehearsal found nothing"))
        }
        Err(error) => Err(error),
        Ok(()) => Err(data("a rehearsal was not rolled back")),
    }
}

/// Which postings a partner figure is of.
#[derive(Clone, Copy)]
enum Postings {
    /// The posting being edited.
    Old,
    /// The posting that took its place: the new entries of the entry that reverse nothing.
    New { max_before: i64 },
}

impl Postings {
    /// The condition on the entry `e`, with its arguments after `?1` (source type) and `?2`
    /// (source id).
    fn condition(self) -> &'static str {
        match self {
            Postings::Old => "e.id = ?3",
            Postings::New { .. } => "e.id > ?3 AND e.reverses_entry_id IS NULL",
        }
    }

    fn bound(self, scope: Scope<'_>) -> i64 {
        match self {
            Postings::Old => scope.posting_id,
            Postings::New { max_before } => max_before,
        }
    }
}

/// Each partner's net from one posting: his share rows plus the credits less the debits on his
/// capital, drawings and gardens-trips-fruit accounts.
fn partner_sums(
    conn: &Connection,
    scope: Scope<'_>,
    postings: Postings,
) -> Result<BTreeMap<i64, i64>, ServiceError> {
    let condition = postings.condition();
    let bound = postings.bound(scope);
    let mut sums: BTreeMap<i64, i64> = BTreeMap::new();
    let mut shares = conn.prepare(&format!(
        "SELECT r.partner_id, COALESCE(SUM(r.amount_paisa), 0)
         FROM partner_share_rows r JOIN journal_entries e ON e.id = r.journal_entry_id
         WHERE e.source_type = ?1 AND e.source_id = ?2 AND e.status = 'posted' AND {condition}
         GROUP BY r.partner_id"
    ))?;
    for row in shares.query_map(params![scope.source_type, scope.source_id, bound], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (partner, amount) = row?;
        let slot = sums.entry(partner).or_insert(0);
        *slot = slot
            .checked_add(amount)
            .ok_or_else(|| data("a partner share does not fit"))?;
    }
    let mut own = conn.prepare(&format!(
        "SELECT p.id, COALESCE(SUM(l.cr_paisa), 0), COALESCE(SUM(l.dr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN partners p ON l.account_id IN (p.capital_account_id, p.drawings_account_id, p.venture_account_id)
         WHERE e.source_type = ?1 AND e.source_id = ?2 AND e.status = 'posted' AND {condition}
         GROUP BY p.id"
    ))?;
    for row in own.query_map(params![scope.source_type, scope.source_id, bound], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })? {
        let (partner, credits, debits) = row?;
        let net = credits
            .checked_sub(debits)
            .ok_or_else(|| data("a partner net does not fit"))?;
        let slot = sums.entry(partner).or_insert(0);
        *slot = slot
            .checked_add(net)
            .ok_or_else(|| data("a partner share does not fit"))?;
    }
    Ok(sums)
}

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// The parties, the cash and the partners the edit moved.
fn collect(
    tx: &Connection,
    scope: Scope<'_>,
    max_before: i64,
    partners_before: &BTreeMap<i64, i64>,
) -> Result<Facts, ServiceError> {
    // The party accounts (sellers, buyers, separate khatas, gardens and money lent) and the cash
    // account that the lines of the edit's own entries touched, with what those lines add up to.
    let mut statement = tx.prepare(
        "SELECT a.id, a.code, a.name_ur, a.name_en, a.tag, k.code, l.pukhta_person_key,
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0), a.is_pukhta_khata
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE e.id > ?1 AND e.status = 'posted'
         GROUP BY a.id, l.pukhta_person_key ORDER BY a.id, l.pukhta_person_key",
    )?;
    let rows = statement
        .query_map([max_before], |r| {
            Ok((
                RecordRef {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name_ur: r.get(2)?,
                    name_en: r.get(3)?,
                    tag: r.get(4)?,
                },
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
                r.get::<_, bool>(9)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut facts = Facts::default();
    let mut cash_change = 0_i64;
    for (account, control, person, dr, cr, is_pukhta) in rows {
        if account.code == CASH_ACCOUNT_CODE {
            cash_change = cash_change
                .checked_add(dr.checked_sub(cr).ok_or_else(|| data("a cash change"))?)
                .ok_or_else(|| data("a cash change does not fit"))?;
            continue;
        }
        // Sellers 03011, buyers 0304, separate khatas 0305, gardens 0306, money lent 0103.
        if !matches!(
            control.as_str(),
            "03011" | "0304" | "0305" | "0306" | "0103"
        ) {
            continue;
        }
        // A line of a پختہ کھاتہ account with no person would stand for every person at once: leave
        // it out rather than show a balance that is not one person's.
        if is_pukhta && person.is_none() {
            continue;
        }
        let totals = ledger_repo::totals_of(tx, account.id, person.as_deref(), None)?;
        let after = ledger::balance_of(totals)?;
        let before = ledger::balance_of(ledger_repo::Totals {
            dr_paisa: totals
                .dr_paisa
                .checked_sub(dr)
                .ok_or_else(|| data("a balance before the edit"))?,
            cr_paisa: totals
                .cr_paisa
                .checked_sub(cr)
                .ok_or_else(|| data("a balance before the edit"))?,
        })?;
        if before != after {
            facts.parties.push(EffectParty {
                account,
                person_key: person,
                before,
                after,
            });
        }
    }

    if cash_change != 0 {
        let after = cash_in_hand(tx)?;
        let change = paisa(cash_change)?;
        let before = after.checked_sub(change).map_err(money_error)?;
        let check = cash::check(before, change).map_err(money_error)?;
        facts.cash = Some(EffectCash {
            before_paisa: before,
            after_paisa: after,
            needs_confirm: check.needs_confirm,
        });
    }

    let after_sums = partner_sums(tx, scope, Postings::New { max_before })?;
    let mut ids: Vec<i64> = partners_before
        .keys()
        .chain(after_sums.keys())
        .copied()
        .collect();
    ids.sort_unstable();
    ids.dedup();
    for id in ids {
        let before = partners_before.get(&id).copied().unwrap_or(0);
        let after = after_sums.get(&id).copied().unwrap_or(0);
        if before == after {
            continue;
        }
        let (name_ur, name_en): (String, String) = tx.query_row(
            "SELECT name_ur, name_en FROM partners WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        facts.partners.push(EffectPartner {
            partner: RecordRef {
                id,
                code: String::new(),
                name_ur,
                name_en,
                tag: None,
            },
            before_paisa: paisa(before)?,
            after_paisa: paisa(after)?,
        });
    }
    Ok(facts)
}
