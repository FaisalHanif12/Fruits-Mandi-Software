//! The reads of the خرچہ screen (PRD 16.2, FR-16.8, FR-7.62, FR-7.68, plan 17): what an entry
//! would do before it is saved, and last month's regular expenses.
//!
//! The preview calls the same `plan_*` checks and builds the same journal entry as the save
//! (Code Standards B5). A shop expense shows each partner's share, taken from the same hook that
//! writes the share rows when it is saved (`partners::split_entry`, plan 11). A personal entry shows
//! only the chosen partner's کل رقم and open amount, now and after, from
//! `mandi_domain::expenses::personal_effect`. Nothing is written.

use std::collections::{HashMap, HashSet};

use mandi_db::expense_heads as heads;
use mandi_db::ledger::{self as ledger_repo, StoredEntry};
use mandi_db::partners as partner_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers as repo;
use mandi_domain::cash;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::expenses::{
    ExpenseEntry, ExpensePreview, ExpensePreviewInput, ExpenseShare, PartnerTotalEffect,
    RepeatLastMonth, RepeatLastMonthInput, RepeatedExpense,
};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::text;
use mandi_domain::core::vouchers::VoucherType;
use mandi_domain::expenses::{PersonalKind, personal_effect};
use mandi_domain::partners::ShareKind;

use super::cash_kinds::{
    PlannedCash, open_of, plan_personal_expense, plan_personal_return, plan_shop_expense,
};
use super::common::{cash_change, stored_cash_change};
use crate::cash::cash_in_hand;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::floor::preview_refusal;
use crate::partners::{self, money::total_of};
use crate::ventures::record_of;

/// The voucher an edit would replace, with its posting.
struct Replacing {
    /// The account the old voucher is on.
    account_id: Option<i64>,
    entry: StoredEntry,
}

fn find_replacing(
    conn: &Connection,
    id: Option<i64>,
    kind: VoucherType,
    errors: &mut Vec<FieldError>,
) -> Result<Option<Replacing>, ServiceError> {
    let Some(id) = id else {
        return Ok(None);
    };
    let field = "replacing_voucher_id";
    let Some(row) = repo::get(conn, id)? else {
        errors.push(FieldError::new(ErrorCode::ErrNotFound, field));
        return Ok(None);
    };
    if row.status != "saved" {
        errors.push(FieldError::new(ErrorCode::ErrVoucherCancelled, field));
        return Ok(None);
    }
    if row.voucher_type != kind {
        errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, "entry.type"));
        return Ok(None);
    }
    let entry = match row.entry_id {
        Some(entry_id) => ledger_repo::get_entry(conn, entry_id)?,
        None => None,
    };
    let account_id = row.account.as_ref().map(|a| a.id);
    Ok(entry.map(|entry| Replacing { account_id, entry }))
}

/// A working with the named values, built next to the amount it explains (Code Standards B30).
fn working(message_id: &str, args: Vec<(&str, WorkingValue)>) -> Working {
    Working {
        message_id: message_id.to_owned(),
        args: args
            .into_iter()
            .map(|(name, value)| WorkingArg {
                name: name.to_owned(),
                value,
            })
            .collect(),
        rule_id: None,
    }
}

fn money_value(paisa: Paisa) -> WorkingValue {
    WorkingValue::Money { paisa }
}

/// The refusals of a split as field errors of the preview.
pub(super) fn split_errors(error: ServiceError) -> Result<Vec<FieldError>, ServiceError> {
    match error {
        ServiceError::Rejected(items) => Ok(items
            .into_iter()
            .map(|item| FieldError {
                code: item.code,
                field: item.field.unwrap_or_default(),
                values: item.values,
            })
            .collect()),
        other => Err(other),
    }
}

/// Each partner's share of a shop expense (PRD C11, E2E-39: 3,000 shows 1,200 / 900 / 900). The
/// parts come from the hook that writes the share rows, so they are the rows the save writes.
fn shares_of(
    conn: &Connection,
    planned: &PlannedCash,
    errors: &mut Vec<FieldError>,
) -> Result<Vec<ExpenseShare>, ServiceError> {
    let (Some(date), Some(amount)) = (planned.date, planned.amount) else {
        return Ok(Vec::new());
    };
    let entry = planned.entry()?;
    let drafts = match partners::split_entry(conn, date, &entry) {
        Ok(drafts) => drafts,
        Err(error) => {
            errors.extend(split_errors(error)?);
            return Ok(Vec::new());
        }
    };
    let names: HashMap<i64, RecordRef> = partner_repo::list(conn)?
        .iter()
        .map(|p| (p.id, record_of(p.id, &p.name_ur, &p.name_en)))
        .collect();
    let mut shares = Vec::new();
    for draft in drafts.iter().filter(|d| d.kind == ShareKind::Expense) {
        // The stored row is negative; the screen shows the part as a positive number.
        let part = if draft.amount.is_negative() {
            draft.amount.checked_neg().map_err(money_error)?
        } else {
            draft.amount
        };
        let partner = names
            .get(&draft.partner_id.get())
            .cloned()
            .ok_or_else(|| ServiceError::Data("a share row names no partner".to_owned()))?;
        shares.push(ExpenseShare {
            partner,
            share_bps: draft.share_bps.get(),
            amount_paisa: part,
            working: working(
                "working-expense-share",
                vec![
                    ("amount", money_value(amount)),
                    (
                        "bps",
                        WorkingValue::Percent {
                            bps: draft.share_bps.get(),
                        },
                    ),
                    ("result", money_value(part)),
                ],
            ),
        });
    }
    Ok(shares)
}

/// The chosen partner's کل رقم and open amount before and after a personal entry (PRD FR-16.8).
fn partner_effect(
    conn: &Connection,
    planned: &PlannedCash,
    replacing: Option<&Replacing>,
) -> Result<Option<PartnerTotalEffect>, ServiceError> {
    let (Some(partner), Some(amount), Some(account)) = (
        planned.partner.as_ref(),
        planned.amount,
        planned.account.as_ref(),
    ) else {
        return Ok(None);
    };
    let kind = match planned.kind {
        VoucherType::PersonalExpense => PersonalKind::Take,
        _ => PersonalKind::Return,
    };
    let without = replacing.map(|r| &r.entry);
    // With an edit, the old entry is taken out of both numbers first. What a return can settle is
    // judged on its own day (the same function as the save); the open amount of a take is the
    // balance now.
    let day = match kind {
        PersonalKind::Return => planned.date,
        PersonalKind::Take => None,
    };
    let open_before = open_of(conn, account.id, day, without)?;
    let mut total_before = total_of(conn, partner, None)?;
    if let Some(old) = without {
        // The old entry moved his total by minus (debit less credit) of his drawings account.
        let mut old_net = Paisa::ZERO;
        for line in old.lines.iter().filter(|l| l.account_id == account.id) {
            let dr = Paisa::try_from_paisa(line.dr_paisa).map_err(money_error)?;
            let cr = Paisa::try_from_paisa(line.cr_paisa).map_err(money_error)?;
            old_net = old_net
                .checked_add(dr)
                .and_then(|v| v.checked_sub(cr))
                .map_err(money_error)?;
        }
        total_before = total_before.checked_add(old_net).map_err(money_error)?;
    }
    let effect = personal_effect(kind, total_before, open_before, amount).map_err(money_error)?;
    let open_after = effect.open_after;
    Ok(Some(PartnerTotalEffect {
        partner: record_of(partner.id, &partner.name_ur, &partner.name_en),
        total_before_paisa: total_before,
        total_after_paisa: effect.total_after,
        open_before_paisa: open_before,
        open_after_paisa: open_after,
        working: working(
            "working-partner-total",
            vec![
                ("before", money_value(total_before)),
                ("amount", money_value(amount)),
                ("after", money_value(effect.total_after)),
            ],
        ),
    }))
}

/// The partner whose name (either script) is the "paid to" of a shop expense (PRD FR-7.68). The
/// match is on the normalised names, so spacing, case and the look-alike Urdu letters do not
/// matter. Empty text matches nobody.
fn partner_named(conn: &Connection, paid_to: &str) -> Result<Option<RecordRef>, ServiceError> {
    let (ur, en) = (text::search_ur(paid_to), text::search_en(paid_to));
    if ur.is_empty() && en.is_empty() {
        return Ok(None);
    }
    Ok(partner_repo::list(conn)?
        .iter()
        .find(|p| {
            (!ur.is_empty() && text::search_ur(&p.name_ur) == ur)
                || (!en.is_empty() && text::search_en(&p.name_en) == en)
        })
        .map(|p| record_of(p.id, &p.name_ur, &p.name_en)))
}

/// Works out what an entry of the خرچہ screen would do, without saving it (PRD FR-16.8).
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data`, `ERR_OVERFLOW`. A mistake of the user is in the
/// answer's `errors`, not an error of the call.
pub fn preview_expense(
    ctx: &Context,
    input: &ExpensePreviewInput,
) -> Result<ExpensePreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| {
        let kind = input.entry.voucher_type();
        let mut errors: Vec<FieldError> = Vec::new();
        let replacing = find_replacing(conn, input.replacing_voucher_id, kind, &mut errors)?;
        let without = replacing.as_ref().map(|r| &r.entry);
        let planned = match &input.entry {
            ExpenseEntry::ShopExpense(i) => plan_shop_expense(conn, window, i)?,
            ExpenseEntry::PersonalExpense(i) => plan_personal_expense(conn, window, i)?,
            ExpenseEntry::PersonalReturn(i) => plan_personal_return(conn, window, i, without)?,
        };
        errors.extend(planned.errors.iter().cloned());
        // An edit of a take that would leave returns above what was taken says so now, like the
        // save does (Code Standards B5): the old account is checked too.
        if errors.is_empty()
            && let (Some(account), Some(date), Some(amount)) =
                (planned.account.as_ref(), planned.date, planned.amount)
        {
            let old = replacing
                .as_ref()
                .and_then(|r| r.account_id.map(|id| (id, &r.entry)));
            if let Some(refusal) =
                preview_refusal(conn, kind, old, Some((account.id, &date.to_iso(), amount)))?
            {
                errors.push(refusal);
            }
        }
        let clean = errors.is_empty();
        let warnings: Vec<FieldError> = Vec::new();

        let shares = if clean && kind == VoucherType::ShopExpense {
            shares_of(conn, &planned, &mut errors)?
        } else {
            Vec::new()
        };
        let clean = errors.is_empty();
        let partner_effect = if clean && kind != VoucherType::ShopExpense {
            partner_effect(conn, &planned, replacing.as_ref())?
        } else {
            None
        };
        let paid_to_partner = match (kind, planned.paid_to.as_deref()) {
            (VoucherType::ShopExpense, Some(paid_to)) => partner_named(conn, paid_to)?,
            _ => None,
        };
        // PRD FR-6.7: cash after this entry. An edit also gives back what the old one took.
        let new_cash = if clean {
            cash_change(conn, &planned.entry()?)?
        } else {
            Paisa::ZERO
        };
        let old_cash = match &replacing {
            Some(old) => stored_cash_change(conn, &old.entry)?,
            None => Paisa::ZERO,
        };
        let change = new_cash.checked_sub(old_cash).map_err(money_error)?;
        let cash = if change.is_negative() {
            cash::check(cash_in_hand(conn)?, change)
        } else {
            cash::check_unread(change)
        }
        .map_err(money_error)?;

        Ok(ExpensePreview {
            voucher_type: kind,
            amount_paisa: planned.amount.unwrap_or(Paisa::ZERO),
            head: planned.head.as_ref().map(|h| RecordRef {
                id: h.id,
                code: h.code.clone(),
                name_ur: h.name_ur.clone(),
                name_en: h.name_en.clone(),
            }),
            partner: planned
                .partner
                .as_ref()
                .map(|p| record_of(p.id, &p.name_ur, &p.name_en)),
            others_not_affected: partner_effect.is_some(),
            shares,
            partner_effect,
            paid_to_partner,
            cash,
            errors,
            warnings,
        })
    })
}

/// The first day of a `YYYY-MM` month.
fn month_start(month: &str) -> Option<BusinessDate> {
    let (year, mon) = month.split_once('-')?;
    if year.len() != 4 || mon.len() != 2 {
        return None;
    }
    BusinessDate::from_ymd(year.parse().ok()?, mon.parse().ok()?, 1)
}

/// The first day of the month after `start`.
fn next_month(start: BusinessDate) -> Option<BusinessDate> {
    if start.month() == 12 {
        BusinessDate::from_ymd(start.year().checked_add(1)?, 1, 1)
    } else {
        BusinessDate::from_ymd(start.year(), start.month().checked_add(1)?, 1)
    }
}

/// The first day of the month before `start`.
fn previous_month(start: BusinessDate) -> Option<BusinessDate> {
    if start.month() == 1 {
        BusinessDate::from_ymd(start.year().checked_sub(1)?, 12, 1)
    } else {
        BusinessDate::from_ymd(start.year(), start.month().checked_sub(1)?, 1)
    }
}

fn month_text(start: BusinessDate) -> String {
    format!("{:04}-{:02}", start.year(), start.month())
}

/// Proposes this month's regular expenses from last month (PRD FR-7.62): each active head that
/// had a saved shop expense last month and none yet in `month`, with the amount, "paid to" and
/// note of its latest one. The user checks and saves each; nothing is saved here. Read only.
///
/// # Errors
/// `ERR_FORMAT_INVALID` on `month` if it is not `YYYY-MM`; `ServiceError::Db`.
pub fn repeat_last_month_preview(
    ctx: &Context,
    input: &RepeatLastMonthInput,
) -> Result<RepeatLastMonth, ServiceError> {
    let bad = || ServiceError::reject_field(ErrorCode::ErrFormatInvalid, "month");
    let this = month_start(input.month.trim()).ok_or_else(bad)?;
    let last = previous_month(this).ok_or_else(bad)?;
    let after = next_month(this).ok_or_else(bad)?;
    ctx.db.read(|conn| {
        let last_rows = heads::shop_expenses_between(conn, &last.to_iso(), &this.to_iso())?;
        let this_rows = heads::shop_expenses_between(conn, &this.to_iso(), &after.to_iso())?;
        let entered: HashSet<i64> = this_rows.iter().map(|r| r.head_id).collect();
        let active: HashMap<i64, heads::HeadFacts> = heads::heads_by_use(conn)?
            .into_iter()
            .map(|h| (h.head.id, h.head))
            .collect();
        // The latest expense of each head last month (the rows come by day and number).
        let mut latest: HashMap<i64, &heads::MonthExpense> = HashMap::new();
        for row in &last_rows {
            latest.insert(row.head_id, row);
        }
        let mut rows = Vec::new();
        let mut total = Paisa::ZERO;
        for (head_id, expense) in latest {
            if entered.contains(&head_id) {
                continue;
            }
            let Some(head) = active.get(&head_id) else {
                continue;
            };
            let amount = Paisa::try_from_paisa(expense.amount_paisa).map_err(money_error)?;
            total = total.checked_add(amount).map_err(money_error)?;
            rows.push(RepeatedExpense {
                head: RecordRef {
                    id: head.id,
                    code: head.code.clone(),
                    name_ur: head.name_ur.clone(),
                    name_en: head.name_en.clone(),
                },
                expense_head_id: head.id,
                amount_paisa: amount,
                paid_to: expense.paid_to.clone(),
                narration: expense.narration.clone(),
                last_date: expense.entry_date.clone(),
            });
        }
        rows.sort_by(|a, b| {
            a.head
                .code
                .cmp(&b.head.code)
                .then(a.head.id.cmp(&b.head.id))
        });
        Ok(RepeatLastMonth {
            month: month_text(this),
            last_month: month_text(last),
            rows,
            total_paisa: total,
        })
    })
}
