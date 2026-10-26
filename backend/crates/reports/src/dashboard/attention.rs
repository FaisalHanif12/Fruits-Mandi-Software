//! The attention list (PRD FR-7.44): what needs the Owner's action, one line for each kind, with
//! the count that shows on the bell. The items that count things are in `signals`; the ones that
//! need money are here:
//!
//! * **Buyers over their credit limit.** A named buyer with a credit limit set (0 is a limit) whose
//!   debit balance is above it: the rule of the party card. The amount is Σ (balance − limit).
//! * **Buyer dues older than 30 days.** The buyers (and one-time people) whose OLDEST open bill,
//!   as R20 works it out, is more than 30 days old. The amount is their open bills in the bands
//!   of 31 days and more.
//! * **Seller payables older than 7 days.** The open payable of a seller key is made of his
//!   NEWEST credits, because payments settle the oldest first (PRD FR-7.59). So part of it is
//!   older than the cutoff exactly when it is more than what his recent credits still hold. Let
//!   `P` be the open payable, `R` what his live credits dated on or after the cutoff (today less
//!   7 days) hold (each less the advances adjusted on it), and `Lold` what his older ordinary
//!   credits hold. The part older than 7 days is `min(max(0, P − R), Lold)`. An opening credit is
//!   the oldest posting whatever its date (FR-7.59), so what is left of it is aged by its own
//!   date. A credit exactly 7 days old is not older.
//! * **Loans and takes past their return-by day.** From the R32 working (`lent`).
//!
//! Example (PRD 4.9, E2E-30b and W1): on 18-10-26 کریم's payable 10,160, posted on 10-10-26, is
//! 8 days old, so it is listed; on 17-10-26 (7 days) it is not.

use std::collections::HashMap;

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::ledger::totals_for_accounts;
use mandi_db::rusqlite::{Connection, params};

use super::Facts;
use super::charts::days_before;
use super::lent::{LentFacts, Overdue};
use super::model::{
    AttentionFirst, AttentionItem, AttentionKind, AttentionList, AttentionTarget, BackupFacts,
    DUES_OLD_DAYS, PAYABLES_OLD_DAYS,
};
use super::signals::{self, counted};
use crate::error::ReportsError;
use crate::reports::common::{IN_CHUNK, LIVE, add, paisa};

/// An item with an amount and an oldest age.
fn item(
    kind: AttentionKind,
    count: i64,
    amount: Paisa,
    days: Option<i64>,
    first: Option<AttentionFirst>,
) -> AttentionItem {
    let mut out = counted(kind, count);
    out.amount_paisa = Some(amount);
    out.days = days;
    out.first = first;
    out
}

fn count_of(len: usize) -> Result<i64, ReportsError> {
    i64::try_from(len).map_err(|_| ReportsError::Data("a count does not fit".to_owned()))
}

/// Named buyers over their credit limit (the rule of the party card).
fn over_credit_limit(conn: &Connection) -> Result<Option<AttentionItem>, ReportsError> {
    let mut statement = conn.prepare(&format!(
        "SELECT a.id, a.name_ur, a.name_en, a.credit_limit_paisa, a.tag
         FROM accounts a JOIN account_controls k ON k.id = a.control_id
         WHERE k.code = '0304' AND a.is_pukhta_khata = 0 AND a.credit_limit_paisa IS NOT NULL
         ORDER BY {}",
        mandi_db::codes::order_by_code("a.code")
    ))?;
    let limits = statement
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let ids: Vec<i64> = limits.iter().map(|l| l.0).collect();
    let mut totals = HashMap::new();
    for chunk in ids.chunks(IN_CHUNK) {
        totals.extend(totals_for_accounts(conn, chunk)?);
    }
    // (excess, account id, names, tag)
    let mut over: Vec<(Paisa, i64, String, String, Option<String>)> = Vec::new();
    for (id, name_ur, name_en, limit, tag) in limits {
        let Some(t) = totals.get(&id) else { continue };
        let used = paisa(t.dr_paisa)?.checked_sub(paisa(t.cr_paisa)?)?;
        let limit = paisa(limit)?;
        if used > limit {
            over.push((used.checked_sub(limit)?, id, name_ur, name_en, tag));
        }
    }
    over.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut amount = Paisa::ZERO;
    for (excess, ..) in &over {
        add(&mut amount, *excess)?;
    }
    let count = count_of(over.len())?;
    Ok(over.first().cloned().map(|(_, id, name_ur, name_en, tag)| {
        item(
            AttentionKind::OverCreditLimit,
            count,
            amount,
            None,
            Some(AttentionFirst {
                target: AttentionTarget::Account,
                id,
                person_key: None,
                name_ur: Some(name_ur),
                name_en: Some(name_en),
                tag,
            }),
        )
    }))
}

/// Buyers whose oldest open bill is older than [`DUES_OLD_DAYS`] days.
fn dues_old(facts: &Facts) -> Result<Option<AttentionItem>, ReportsError> {
    let mut count = 0_i64;
    let mut amount = Paisa::ZERO;
    // (days, balance, buyer) of the oldest.
    let mut oldest: Option<(i64, Paisa, &crate::reports::PartyRef)> = None;
    for buyer in &facts.buyers {
        let Some(ages) = &buyer.ages else { continue };
        let Some((_, days)) = &ages.oldest else {
            continue;
        };
        if *days <= DUES_OLD_DAYS {
            continue;
        }
        count = count
            .checked_add(1)
            .ok_or_else(|| ReportsError::Data("the buyers do not fit".to_owned()))?;
        // Bands 2 and 3 hold the open bills of 31 days and more.
        for band in ages.bands.iter().skip(2) {
            add(&mut amount, *band)?;
        }
        let better = oldest
            .as_ref()
            .is_none_or(|(d, b, _)| (*days, buyer.balance) > (*d, *b));
        if better {
            oldest = Some((*days, buyer.balance, &buyer.party));
        }
    }
    Ok(oldest.map(|(days, _, party)| {
        item(
            AttentionKind::DuesOld,
            count,
            amount,
            Some(days),
            Some(AttentionFirst {
                target: AttentionTarget::Account,
                id: party.account_id,
                person_key: party.person_key.clone(),
                name_ur: Some(party.name_ur.clone()),
                name_en: Some(party.name_en.clone()),
                tag: party.tag.clone(),
            }),
        )
    }))
}

/// What the recent and the opening credits of each seller key hold.
#[derive(Debug, Default, Clone, Copy)]
struct Held {
    /// Credits dated on or after the cutoff, not openings, each less the advance adjusted on it.
    recent: Paisa,
    /// What is left of his opening credit, and its day.
    opening: Paisa,
}

/// What the credits of each seller key hold, and the first day of his opening credit.
type HeldByKey = HashMap<(i64, Option<String>), (Held, Option<String>)>;

/// The live credit lines of the sellers that matter for the age: the recent ones and the
/// openings, each with the advance adjusted on it (active rows only). The line set is the one of
/// the payable sums (live entries), grouped by account and person key.
fn held_by_key(conn: &Connection, cutoff: &str) -> Result<HeldByKey, ReportsError> {
    let sql = format!(
        "SELECT l.account_id, l.pukhta_person_key, e.posting_type = 'opening', e.entry_date,
                l.cr_paisa,
                COALESCE((SELECT SUM(a.amount_paisa) FROM advance_allocations a
                          WHERE a.credit_line_id = l.id AND a.is_active = 1), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts ac ON ac.id = l.account_id
         JOIN account_controls k ON k.id = ac.control_id
         WHERE k.code = '03011' AND l.cr_paisa > 0 AND {LIVE}
           AND (e.posting_type = 'opening' OR e.entry_date >= ?1)"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params![cutoff], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, bool>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out: HeldByKey = HashMap::new();
    for (account, key, is_opening, day, credit, adjusted) in rows {
        let left = paisa(credit)?.checked_sub(paisa(adjusted)?)?;
        if left < Paisa::ZERO {
            return Err(ReportsError::Data(
                "more is adjusted on a credit than the credit holds".to_owned(),
            ));
        }
        let slot = out.entry((account, key)).or_default();
        if is_opening {
            add(&mut slot.0.opening, left)?;
            // The earliest day of his opening credit.
            if slot.1.as_ref().is_none_or(|d| day < *d) {
                slot.1 = Some(day);
            }
        } else {
            add(&mut slot.0.recent, left)?;
        }
    }
    Ok(out)
}

/// Seller payables with a part older than [`PAYABLES_OLD_DAYS`] days.
fn payables_old(conn: &Connection, facts: &Facts) -> Result<Option<AttentionItem>, ReportsError> {
    let cutoff = days_before(facts.today, PAYABLES_OLD_DAYS)?.to_iso();
    let held = held_by_key(conn, &cutoff)?;
    let mut count = 0_i64;
    let mut amount = Paisa::ZERO;
    // (old amount, seller)
    let mut biggest: Option<(Paisa, &crate::reports::PartyRef)> = None;
    for seller in &facts.sellers {
        let Some(working) = seller.working else {
            continue;
        };
        let open = working.open;
        if open <= Paisa::ZERO {
            continue;
        }
        let key = seller.party.person_key.clone();
        let account = seller.party.account_id;
        let (held, opening_day) = held
            .get(&(account, key.clone()))
            .cloned()
            .unwrap_or_default();
        let sums = facts.sums.of(account, key.as_deref());
        // What his live credits hold, less the advances adjusted on them.
        let credit_net = paisa(sums.credit)?.checked_sub(paisa(sums.adjusted_on_credits)?)?;
        // What his older ordinary credits hold. The credits of `held` are part of the credits of
        // `sums`, so this is never below 0 unless the matching rows are broken: then the numbers
        // cannot be trusted and the bell says so by failing, not by hiding it.
        let old_ordinary = credit_net
            .checked_sub(held.recent)?
            .checked_sub(held.opening)?;
        if old_ordinary < Paisa::ZERO {
            return Err(ReportsError::Data(
                "a seller's credits hold less than his recent and opening credits".to_owned(),
            ));
        }
        let rest = open.checked_sub(held.recent)?.max(Paisa::ZERO);
        let on_old = rest.min(old_ordinary);
        let on_opening = rest.checked_sub(on_old)?.min(held.opening);
        let mut old = on_old;
        if opening_day
            .as_deref()
            .is_some_and(|day| day < cutoff.as_str())
        {
            add(&mut old, on_opening)?;
        }
        if old > Paisa::ZERO {
            count = count
                .checked_add(1)
                .ok_or_else(|| ReportsError::Data("the sellers do not fit".to_owned()))?;
            add(&mut amount, old)?;
            if biggest.is_none_or(|(b, _)| old > b) {
                biggest = Some((old, &seller.party));
            }
        }
    }
    Ok(biggest.map(|(_, party)| {
        item(
            AttentionKind::PayablesOld,
            count,
            amount,
            None,
            Some(AttentionFirst {
                target: AttentionTarget::Account,
                id: party.account_id,
                person_key: party.person_key.clone(),
                name_ur: Some(party.name_ur.clone()),
                name_en: Some(party.name_en.clone()),
                tag: party.tag.clone(),
            }),
        )
    }))
}

/// Loans or takes past their return-by day.
fn overdue_item(
    kind: AttentionKind,
    target: AttentionTarget,
    rows: &[Overdue],
) -> Result<Option<AttentionItem>, ReportsError> {
    let Some(first) = rows.first() else {
        return Ok(None);
    };
    let mut amount = Paisa::ZERO;
    for row in rows {
        add(&mut amount, row.open)?;
    }
    Ok(Some(item(
        kind,
        count_of(rows.len())?,
        amount,
        Some(first.days_past),
        Some(AttentionFirst {
            target,
            id: first.who_id,
            person_key: None,
            name_ur: Some(first.name_ur.clone()),
            name_en: Some(first.name_en.clone()),
            tag: None,
        }),
    )))
}

/// The attention list and its count, from one snapshot of the books.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit.
pub(crate) fn list(
    conn: &Connection,
    facts: &Facts,
    lent: &LentFacts,
    backup: BackupFacts,
) -> Result<AttentionList, ReportsError> {
    let today: BusinessDate = facts.today;
    // In the order of `AttentionKind::ALL`.
    let found: [Option<AttentionItem>; 10] = [
        signals::drafts(conn)?,
        signals::open_days(conn, today)?,
        signals::stock_waiting(conn, today)?,
        over_credit_limit(conn)?,
        dues_old(facts)?,
        payables_old(conn, facts)?,
        signals::backup_old(backup),
        overdue_item(
            AttentionKind::LoansOverdue,
            AttentionTarget::Account,
            &lent.loans,
        )?,
        overdue_item(
            AttentionKind::TakesOverdue,
            AttentionTarget::Partner,
            &lent.takes,
        )?,
        signals::names_missing(conn)?,
    ];
    let items: Vec<AttentionItem> = found.into_iter().flatten().collect();
    let mut total_count = 0_i64;
    for it in &items {
        total_count = total_count
            .checked_add(it.count)
            .ok_or_else(|| ReportsError::Data("the attention count does not fit".to_owned()))?;
    }
    Ok(AttentionList { items, total_count })
}
