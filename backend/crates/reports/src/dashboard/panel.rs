//! The bell panel (PRD FR-7.44, v1.61, plan 100 J). Read only: the items come from the books, so
//! they stay until they are settled. "بعد میں" only hides an item until the next بکری save; the
//! caller gives the hidden keys.
//!
//! * **بیوپاریوں کو ادائیگی.** The sellers with the largest net payable (plan 100 A), top
//!   [`PANEL_TOP`]. The days are those of the oldest credit still needed to hold the net amount:
//!   payments and the netted advance settle the oldest first (PRD FR-7.59, FR-15.8).
//! * **گاہکوں سے وصولی.** The buyers of R20 who owe the most, with the days of their oldest open
//!   bill.
//! * **عام یاد دہانیاں.** The head 02 payables with something to pay, then the other lines of the
//!   attention list. The old "buyer dues older than 30 days" and "payables older than 7 days" are
//!   the two groups above.
//!
//! A hidden row is not replaced by the next one, so hiding lowers the number on the bell.
//!
//! Example (after Events 1 to 5): sellers کریم 10,160; buyers ارشد 18,684, اکرم 2,880; payables
//! مزدوری 300, کرایہ 4,000, منشیانہ 100.

use std::collections::HashSet;

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_core::seller_net::oldest_holding;
use mandi_db::Db;
use mandi_db::ledger::account_totals_in_controls;
use mandi_db::rusqlite::Connection;

use super::model::{AttentionKind, BackupFacts};
use super::panel_model::{
    AttentionPanel, AttentionPanelItem, AttentionPartyGroup, AttentionPartyRow,
    AttentionPayableRow, PANEL_TOP,
};
use super::{attention, lent, load_facts};
use crate::error::ReportsError;
use crate::reports::PartyRef;
use crate::reports::common::{add, in_snapshot, load_parties, paisa, today as today_of};

/// The control of the payables of head 02 (مزدوری, کرایہ, کرایہ سٹور, منشیانہ).
const PAYABLE_CONTROL: &str = "0201";

/// The word of an attention kind in its `item_key`, the same as on the wire.
fn kind_key(kind: AttentionKind) -> Result<String, ReportsError> {
    match serde_json::to_value(kind) {
        Ok(serde_json::Value::String(word)) => Ok(word),
        _ => Err(ReportsError::Data(
            "an attention kind has no word".to_owned(),
        )),
    }
}

/// The days of the oldest credit still needed to hold a seller's net payable.
fn net_days(
    conn: &Connection,
    today: BusinessDate,
    account_id: i64,
    net: Paisa,
) -> Result<Option<i64>, ReportsError> {
    let credits = mandi_db::vouchers::payable_credits_oldest_first(conn, account_id, None)?;
    let adjusted = mandi_db::vouchers::advance_adjusted_by_credit(conn, account_id)?;
    let mut newest_first = Vec::with_capacity(credits.len());
    for credit in credits.iter().rev() {
        let left = paisa(credit.credit_paisa)?
            .checked_sub(paisa(adjusted.get(&credit.line_id).copied().unwrap_or(0))?)?;
        newest_first.push(left);
    }
    let Some(index) = oldest_holding(newest_first, net)? else {
        return Ok(None);
    };
    let Some(credit) = credits.iter().rev().nth(index) else {
        return Ok(None);
    };
    let row = mandi_db::vouchers::credit_row(conn, credit.line_id)?;
    let day = BusinessDate::parse_iso(&row.entry_date)
        .ok_or_else(|| ReportsError::Data(format!("entry date {}", row.entry_date)))?;
    Ok(Some(today.days_after(day).max(0)))
}

/// A row before its days are read.
struct Ranked {
    party: PartyRef,
    amount: Paisa,
    days: Option<i64>,
}

/// The largest first, then by account.
fn rank(rows: &mut [Ranked]) {
    rows.sort_by(|a, b| {
        b.amount
            .cmp(&a.amount)
            .then(a.party.account_id.cmp(&b.party.account_id))
    });
}

/// The count and total over every row, and the top rows that are not hidden.
fn group(
    rows: Vec<Ranked>,
    prefix: &str,
    hidden: &HashSet<&str>,
    hidden_count: &mut i64,
) -> Result<AttentionPartyGroup, ReportsError> {
    let count = i64::try_from(rows.len())
        .map_err(|_| ReportsError::Data("a count does not fit".to_owned()))?;
    let mut total = Paisa::ZERO;
    for row in &rows {
        add(&mut total, row.amount)?;
    }
    let mut shown = Vec::new();
    for row in rows.into_iter().take(PANEL_TOP) {
        let item_key = format!("{prefix}:{}", row.party.account_id);
        if hidden.contains(item_key.as_str()) {
            *hidden_count = hidden_count.saturating_add(1);
            continue;
        }
        shown.push(AttentionPartyRow {
            item_key,
            party: row.party,
            amount_paisa: row.amount,
            days: row.days,
        });
    }
    Ok(AttentionPartyGroup {
        rows: shown,
        count,
        total_paisa: total,
    })
}

/// The head 02 payables with something to pay, in code order.
fn payables(conn: &Connection) -> Result<Vec<(i64, Paisa)>, ReportsError> {
    let mut found = Vec::new();
    for (account_id, totals) in account_totals_in_controls(conn, &[PAYABLE_CONTROL], None)? {
        let owed = paisa(totals.cr_paisa)?.checked_sub(paisa(totals.dr_paisa)?)?;
        if owed > Paisa::ZERO {
            found.push((account_id, owed));
        }
    }
    Ok(found)
}

/// The bell panel, from one snapshot of the books. `hidden` are the `item_key`s hidden with
/// "بعد میں" since the last بکری save.
///
/// # Errors
/// As the dashboard.
pub fn attention_panel(
    db: &Db,
    backup: BackupFacts,
    hidden: &[String],
) -> Result<AttentionPanel, ReportsError> {
    let hidden: HashSet<&str> = hidden.iter().map(String::as_str).collect();
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let facts = load_facts(conn, today)?;
            let lent = lent::facts(conn, today)?;
            let mut hidden_count = 0_i64;

            // The sellers by net payable. The days are read for the shown ones only.
            let mut sellers = Vec::new();
            for seller in &facts.sellers {
                let Some(working) = seller.working else {
                    continue;
                };
                let net = working.net()?.payable;
                if net > Paisa::ZERO {
                    sellers.push(Ranked {
                        party: seller.party.clone(),
                        amount: net,
                        days: None,
                    });
                }
            }
            rank(&mut sellers);
            for row in sellers.iter_mut().take(PANEL_TOP) {
                row.days = net_days(conn, today, row.party.account_id, row.amount)?;
            }
            let sellers = group(sellers, "seller", &hidden, &mut hidden_count)?;

            let mut buyers: Vec<Ranked> = facts
                .buyers
                .iter()
                .map(|b| Ranked {
                    party: b.party.clone(),
                    amount: b.balance,
                    days: b
                        .ages
                        .as_ref()
                        .and_then(|ages| ages.oldest.as_ref().map(|(_, days)| *days)),
                })
                .collect();
            rank(&mut buyers);
            let buyers = group(buyers, "buyer", &hidden, &mut hidden_count)?;

            let owed = payables(conn)?;
            let ids: Vec<i64> = owed.iter().map(|(id, _)| *id).collect();
            let parties = load_parties(conn, &ids)?;
            let mut payable_rows = Vec::new();
            for (account_id, amount) in owed {
                let item_key = format!("payable:{account_id}");
                if hidden.contains(item_key.as_str()) {
                    hidden_count = hidden_count.saturating_add(1);
                    continue;
                }
                let party = parties.get(&account_id).ok_or_else(|| {
                    ReportsError::Data(format!("payable account {account_id} is missing"))
                })?;
                payable_rows.push(AttentionPayableRow {
                    item_key,
                    account_id,
                    code: party.code.clone(),
                    name_ur: party.name_ur.clone(),
                    name_en: party.name_en.clone(),
                    amount_paisa: amount,
                });
            }

            let mut general = Vec::new();
            for item in attention::list(conn, &facts, &lent, backup)?.items {
                // These two are the two groups above.
                if matches!(
                    item.kind,
                    AttentionKind::DuesOld | AttentionKind::PayablesOld
                ) {
                    continue;
                }
                let item_key = format!("kind:{}", kind_key(item.kind)?);
                if hidden.contains(item_key.as_str()) {
                    hidden_count = hidden_count.saturating_add(1);
                    continue;
                }
                general.push(AttentionPanelItem {
                    item_key,
                    kind: item.kind,
                    count: item.count,
                    amount_paisa: item.amount_paisa,
                    qty: item.qty,
                    days: item.days,
                    first: item.first,
                });
            }

            let shown = [
                sellers.rows.len(),
                buyers.rows.len(),
                payable_rows.len(),
                general.len(),
            ];
            let bell_count = shown.iter().try_fold(0_i64, |sum, n| {
                i64::try_from(*n)
                    .ok()
                    .and_then(|n| sum.checked_add(n))
                    .ok_or_else(|| ReportsError::Data("the bell count does not fit".to_owned()))
            })?;
            Ok(AttentionPanel {
                sellers,
                buyers,
                payables: payable_rows,
                general,
                bell_count,
                hidden_count,
                partners_locked: false,
            })
        })
    })
}

impl AttentionPanel {
    /// Leaves out what is behind the partner code (PRD FR-17.11): the amount of the money partners
    /// took past its return-by day, and which partner. The count stays.
    pub fn hide_partners(&mut self) {
        for item in &mut self.general {
            if item.kind == AttentionKind::TakesOverdue {
                item.amount_paisa = None;
            }
        }
        self.partners_locked = true;
    }
}
