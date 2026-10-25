//! The روکڑ بک (PRD 16.1, FR-19.11, plan 17): every cash in and out from all modules, one page at a
//! time, with the opening and closing cash and the totals over the whole range. There is no cash
//! count (PRD FR-16.1): the book only shows what was posted.
//!
//! One row is one journal entry that moves cash. The running cash is the opening cash plus the
//! sums of the rows up to it, over the whole range, so a page far down has the right figure. The
//! sums are `SUM()` on INTEGER columns; the differences are worked out here with checked maths.

use std::collections::HashMap;

use mandi_db::accounts as account_repo;
use mandi_db::cashbook as repo;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::cashbook::{Cashbook, CashbookInput, CashbookRow, CashbookTotals};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};

use crate::cash::CASH_ACCOUNT_CODE;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::page;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// A date bound of the range: empty is open.
fn bound(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&str>,
    today: BusinessDate,
) -> Option<BusinessDate> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors.take(validate::date(field, text, today))
}

/// One page of the cash book with the opening and closing cash and the totals over the whole
/// range (PRD FR-19.11). Without `from` the book starts at the first entry and the opening is 0.
///
/// # Errors
/// The date codes on `from` and `to`; `ServiceError::Db`.
pub fn get(ctx: &Context, input: &CashbookInput) -> Result<Cashbook, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let from = bound(&mut errors, "from", input.from.as_deref(), today);
    let to = bound(&mut errors, "to", input.to.as_deref(), today);
    errors.finish()?;
    let (from_iso, to_iso) = (from.map(BusinessDate::to_iso), to.map(BusinessDate::to_iso));
    ctx.db.read(|conn| {
        let cash = account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
            .ok_or_else(|| ServiceError::Data("the cash account is missing".to_owned()))?;
        // The opening is the cash at the end of the day before the first day of the range.
        let opening = match from.and_then(BusinessDate::previous_day) {
            Some(day) => {
                ledger::signed(ledger::balance_in(conn, cash, None, Some(&day.to_iso()))?)?
            }
            None => Paisa::ZERO,
        };
        let range = repo::range(conn, cash, from_iso.as_deref(), to_iso.as_deref())?;
        let (total_in, total_out) = (paisa(range.in_paisa)?, paisa(range.out_paisa)?);
        let closing = opening
            .checked_add(total_in)
            .and_then(|v| v.checked_sub(total_out))
            .map_err(money_error)?;
        let stored = repo::page(
            conn,
            cash,
            from_iso.as_deref(),
            to_iso.as_deref(),
            i64::from(input.offset),
            page::limit_of(input.limit),
        )?;
        let ids: Vec<i64> = stored.iter().map(|r| r.entry_id).collect();
        let others = repo::counterparts(conn, cash, &ids)?;
        let mut rows = Vec::with_capacity(stored.len());
        for row in &stored {
            let running = opening
                .checked_add(paisa(row.run_in_paisa)?)
                .map_err(money_error)?
                .checked_sub(paisa(row.run_out_paisa)?)
                .map_err(money_error)?;
            // The other side is shown only when it is one account: the entry of a بکری has many.
            let account = counterpart(&others, row);
            rows.push(CashbookRow {
                entry_id: row.entry_id,
                entry_date: row.entry_date.clone(),
                posting_type: row.posting_type.clone(),
                number: row.number,
                account,
                narration: if row.narration.is_empty() {
                    row.reason.clone()
                } else {
                    row.narration.clone()
                },
                in_paisa: paisa(row.in_paisa)?,
                out_paisa: paisa(row.out_paisa)?,
                running_paisa: running,
            });
        }
        Ok(Cashbook {
            opening_paisa: opening,
            closing_paisa: closing,
            rows,
            total_count: range.count,
            totals: CashbookTotals {
                in_paisa: total_in,
                out_paisa: total_out,
            },
        })
    })
}

/// The one account on the other side of an entry that only takes cash in, or only gives it out.
fn counterpart(
    others: &HashMap<i64, Vec<repo::Counterpart>>,
    row: &repo::CashRow,
) -> Option<RecordRef> {
    // An entry that does both (cash in on one line and out on another) has no single other side.
    if row.in_paisa > 0 && row.out_paisa > 0 {
        return None;
    }
    // Cash received (debit) is matched by credits; cash paid out (credit) by debits.
    let cash_received = row.in_paisa > 0;
    let mut found: Option<&repo::Counterpart> = None;
    for line in others.get(&row.entry_id)?.iter() {
        if line.is_debit == cash_received {
            continue;
        }
        match found {
            Some(first) if first.account_id != line.account_id => return None,
            Some(_) => {}
            None => found = Some(line),
        }
    }
    found.map(|line| RecordRef {
        id: line.account_id,
        code: line.code.clone(),
        name_ur: line.name_ur.clone(),
        name_en: line.name_en.clone(),
        tag: None,
    })
}
