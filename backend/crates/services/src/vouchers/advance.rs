//! The پیشگی voucher (PRD 15.4, plans 10 and 15): giving an advance to a seller, and the running
//! list of FR-15.1.
//!
//! One voucher is one posting event in one transaction: the voucher row, its journal entry and
//! its audit row are written together or not at all (Code Standards B3). The running list
//! checks and saves through one plan function, so what the preview showed is what is saved
//! (Code Standards B5). An advance takes cash out, so the save asks the cash question of PRD
//! FR-6.7.

use std::collections::HashMap;

use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::{self as repo, NewVoucher};
use mandi_domain::core::accounts::{LineError, RecordRef, SELLER_CONTROL};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::core::vouchers::{
    AdvanceInput, AdvancePreviewRow, AdvanceRowResult, AdvancesInput, AdvancesPreview,
    AdvancesSaved, VoucherMode, VoucherSaved, VoucherType,
};
use mandi_domain::ledger::{BalancedEntry, LedgerLine, PostingType, Side, build_entry};

use super::MAX_LIST_LINES;
use super::common::{
    MoneyText, PostingSpec, balance_of_net, cash_change, cash_check, insert_and_post,
    optional_text, plan_money, voucher_json,
};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::require_confirmed;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;

/// A پیشگی voucher that passed every check, with the balances around it.
#[derive(Debug, Clone)]
pub(super) struct PlannedAdvance {
    /// The seller, if the id was right.
    pub(super) account: Option<RecordRef>,
    /// What is wrong, with the field names as sent (no list prefix yet).
    pub(super) errors: Vec<FieldError>,
    pub(super) amount: Paisa,
    pub(super) date: Option<BusinessDate>,
    pub(super) mode: VoucherMode,
    /// The ledger account the money comes from: cash, or the bank's account.
    pub(super) pay_account: Option<i64>,
    pub(super) bank_account_id: Option<i64>,
    pub(super) cheque_no: Option<String>,
    pub(super) cheque_date: Option<BusinessDate>,
    pub(super) expected_goods: Option<String>,
    pub(super) narration: Option<String>,
    /// The seller's balance before and after, as signed amounts (debit positive).
    pub(super) before: Paisa,
    pub(super) after: Paisa,
}

/// Checks one پیشگی voucher against the database and works out the seller's balance before and
/// after. `running` holds the signed balance of sellers already seen in this list, so a second
/// line for the same seller starts where the first one ended. Used by the preview and the save.
pub(super) fn plan_advance(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &AdvanceInput,
    running: &mut HashMap<i64, Paisa>,
) -> Result<PlannedAdvance, ServiceError> {
    let mut errors = validate::FieldErrors::default();
    let money = plan_money(
        conn,
        window,
        &mut errors,
        &MoneyText {
            date: &input.date,
            amount: &input.amount,
            mode: input.mode,
            bank_account_id: input.bank_account_id,
            cheque_no: &input.cheque_no,
            cheque_date: &input.cheque_date,
            narration: &input.narration,
        },
    )?;
    let expected_goods = optional_text(&mut errors, "expected_goods", &input.expected_goods);

    // The seller: a seller account that is not the پختہ کھاتہ one, and active.
    let mut account = None;
    let mut seller = None;
    match account_repo::facts(conn, input.seller_account_id)? {
        None => errors.push(FieldError::new(ErrorCode::ErrNotFound, "seller_account_id")),
        Some(facts) => {
            account = account_repo::get(conn, facts.id)?.map(|row| RecordRef {
                id: row.id,
                code: row.code,
                name_ur: row.name_ur,
                name_en: row.name_en,
            });
            if facts.code == mandi_domain::core::accounts::OWN_GOODS_CODE {
                errors.push(FieldError::new(
                    ErrorCode::ErrOwnGoodsAccountRestricted,
                    "seller_account_id",
                ));
            } else if facts.control_code != SELLER_CONTROL || facts.is_pukhta_khata {
                errors.push(FieldError::new(
                    ErrorCode::ErrNotASeller,
                    "seller_account_id",
                ));
            } else if !facts.is_active {
                errors.push(FieldError::new(
                    ErrorCode::ErrAccountInactive,
                    "seller_account_id",
                ));
            } else {
                seller = Some(facts.id);
            }
        }
    }

    // The balance before and after, counting the lines above for the same seller. A line with a
    // mistake will not be saved, so it moves nothing: its "after" is its "before".
    let errors = errors.finish().err().unwrap_or_default();
    let (before, after) = match account.as_ref().map(|a| a.id) {
        Some(id) => {
            let before = match running.get(&id) {
                Some(net) => *net,
                None => ledger::signed(ledger::balance_in(conn, id, None, None)?)?,
            };
            let after = match money.amount {
                Some(a) if seller.is_some() && errors.is_empty() => {
                    before.checked_add(a).map_err(money_error)?
                }
                _ => before,
            };
            (before, after)
        }
        None => (Paisa::ZERO, Paisa::ZERO),
    };
    let planned = PlannedAdvance {
        account,
        errors,
        amount: money.amount.unwrap_or(Paisa::ZERO),
        date: money.date,
        mode: input.mode,
        pay_account: money.pay_account,
        bank_account_id: money.bank_account_id,
        cheque_no: money.cheque_no,
        cheque_date: money.cheque_date,
        expected_goods,
        narration: money.narration,
        before,
        after,
    };
    // Only a line without a mistake moves the running balance.
    if planned.errors.is_empty()
        && let Some(account) = &planned.account
    {
        running.insert(account.id, planned.after);
    }
    Ok(planned)
}

/// Gives an advance to a seller (PRD 15.4): the seller is debited (an advance line, PRD FR-15.8)
/// and cash or the bank is credited. One transaction: the voucher, its journal entry and its
/// audit row. The number is given here and never used again.
///
/// # Errors
/// The field errors of the date, amount, bank and cheque fields; `ERR_NOT_FOUND`,
/// `ERR_NOT_A_SELLER` and `ERR_ACCOUNT_INACTIVE` on `seller_account_id`; `ERR_DATE_CLOSED` and
/// `ERR_DATE_IN_FUTURE` on `date`. Nothing is written when any of them is returned.
pub fn save_advance(
    ctx: &Context,
    actor: &Actor,
    input: &AdvanceInput,
) -> Result<VoucherSaved, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.write(|tx| {
        let planned = plan_advance(tx, window, input, &mut HashMap::new())?;
        if !planned.errors.is_empty() {
            return Err(planned.errors.into());
        }
        post_advance(ctx, tx, actor, planned, input.confirm_cash_below_zero)
    })
}

/// The journal entry of an advance (PRD 15.4): the seller بنام as an advance (PRD FR-15.8), cash
/// or the bank جمع.
///
/// # Errors
/// `ServiceError::Data` if the plan is not complete; the ledger errors of the entry.
pub(super) fn advance_entry(planned: &PlannedAdvance) -> Result<BalancedEntry, ServiceError> {
    let (Some(account), Some(pay_account)) = (planned.account.as_ref(), planned.pay_account) else {
        return Err(ServiceError::Data(
            "a planned advance is not complete".to_owned(),
        ));
    };
    let mut seller_line = LedgerLine::new(AccountId::new(account.id), Side::Dr, planned.amount);
    seller_line.is_advance = true;
    build_entry(vec![
        seller_line,
        LedgerLine::new(AccountId::new(pay_account), Side::Cr, planned.amount),
    ])
    .map_err(ledger::ledger_error)
}

/// Writes a planned, error-free advance inside the caller's transaction.
fn post_advance(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    planned: PlannedAdvance,
    confirm_cash_below_zero: bool,
) -> Result<VoucherSaved, ServiceError> {
    let (Some(account), Some(date)) = (planned.account.as_ref(), planned.date) else {
        return Err(ServiceError::Data(
            "a planned advance is not complete".to_owned(),
        ));
    };
    // PRD FR-6.7: an advance takes cash out. If cash would end below 0 the user must confirm.
    let entry = advance_entry(&planned)?;
    require_confirmed(
        &cash_check(tx, cash_change(tx, &entry)?)?,
        confirm_cash_below_zero,
    )?;
    let now = ctx.now_text();
    let number = repo::next_number(tx, VoucherType::Advance)?;
    let voucher = advance_row(&planned, number)?;
    let (voucher_id, posted) = insert_and_post(
        ctx,
        tx,
        actor.user_id,
        PostingSpec {
            posting_type: PostingType::Advance,
            date,
            narration: voucher.narration.clone(),
        },
        &voucher,
        entry,
    )?;
    advance_match(tx, actor.user_id, &now, &planned)?;
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), "create", "voucher")
            .entity_id(voucher_id)
            .new_value(voucher_json(number, "advance", &voucher))
            .into_entry(),
    )?;
    Ok(VoucherSaved {
        voucher_id,
        voucher_type: VoucherType::Advance,
        number,
        entry_id: posted.entry_id,
        new_balance: ledger::balance_in(tx, account.id, None, None)?,
    })
}

/// The voucher row of an advance.
///
/// # Errors
/// `ServiceError::Data` if the plan is not complete.
pub(super) fn advance_row(
    planned: &PlannedAdvance,
    number: i64,
) -> Result<NewVoucher, ServiceError> {
    let (Some(account), Some(date)) = (planned.account.as_ref(), planned.date) else {
        return Err(ServiceError::Data(
            "a planned advance is not complete".to_owned(),
        ));
    };
    Ok(NewVoucher {
        voucher_type: VoucherType::Advance,
        number,
        entry_date: date.to_iso(),
        account_id: Some(account.id),
        amount_paisa: planned.amount.as_paisa(),
        mode: planned.mode,
        bank_account_id: planned.bank_account_id,
        cheque_no: planned.cheque_no.clone(),
        cheque_date: planned.cheque_date.map(BusinessDate::to_iso),
        expected_goods: planned.expected_goods.clone(),
        narration: planned.narration.clone(),
        person_key: None,
        collector_staff_id: None,
        reason: None,
    })
}

/// Matches a posted advance. A credit adjusts only the advances before it (PRD FR-15.8). An
/// advance dated before a credit that is already saved (a day entered late) is one that credit
/// must now adjust, so the seller is worked out again; an advance of the newest day is after
/// every credit and needs nothing.
///
/// # Errors
/// As [`crate::matching::rematch_advances`].
pub(super) fn advance_match(
    tx: &Connection,
    user_id: i64,
    now: &str,
    planned: &PlannedAdvance,
) -> Result<(), ServiceError> {
    let (Some(account), Some(date)) = (planned.account.as_ref(), planned.date) else {
        return Err(ServiceError::Data(
            "a planned advance is not complete".to_owned(),
        ));
    };
    if mandi_db::matching::has_later_entry(tx, account.id, None, &date.to_iso())? {
        crate::matching::rematch_advances(tx, user_id, now, account.id)?;
    }
    Ok(())
}

/// Turns the errors of line `index` into `lines[index].<field>` paths.
fn line_errors(index: usize, errors: &[FieldError]) -> Vec<LineError> {
    errors
        .iter()
        .map(|e| LineError {
            code: e.code,
            field: format!("lines[{index}].{}", e.field),
            values: e.values.clone(),
        })
        .collect()
}

fn check_list_size(input: &AdvancesInput) -> Result<(), ServiceError> {
    if input.lines.len() > MAX_LIST_LINES {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrOutOfRange,
            "lines",
        ));
    }
    Ok(())
}

/// Checks the running list (PRD FR-15.1) and works out each line's balance before and after, and
/// the total. Nothing is saved. A second line for the same seller starts from the balance the
/// first one leaves.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `lines` for more than [`MAX_LIST_LINES`] lines; `ServiceError::Db`.
pub fn preview_advances(
    ctx: &Context,
    input: &AdvancesInput,
) -> Result<AdvancesPreview, ServiceError> {
    check_list_size(input)?;
    let window = day::window(ctx)?;
    ctx.db.read(|conn| {
        let mut running = HashMap::new();
        let mut rows = Vec::with_capacity(input.lines.len());
        let mut total = Paisa::ZERO;
        for (index, line) in input.lines.iter().enumerate() {
            let planned = plan_advance(conn, window, line, &mut running)?;
            if planned.errors.is_empty() {
                total = total.checked_add(planned.amount).map_err(money_error)?;
            }
            rows.push(AdvancePreviewRow {
                errors: line_errors(index, &planned.errors),
                account: planned.account,
                amount_paisa: planned.amount,
                current_balance: balance_of_net(planned.before)?,
                new_balance: balance_of_net(planned.after)?,
            });
        }
        Ok(AdvancesPreview {
            rows,
            total_paisa: total,
        })
    })
}

/// Saves the running list (PRD FR-15.1): each voucher is its own posting event in its own
/// transaction, so a line that fails does not stop the others. The answer has one result per
/// line, in the order sent.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `lines` for more than [`MAX_LIST_LINES`] lines. A mistake on a line is
/// in that line's result, not an error of the call.
pub fn save_advances(
    ctx: &Context,
    actor: &Actor,
    input: &AdvancesInput,
) -> Result<AdvancesSaved, ServiceError> {
    check_list_size(input)?;
    let mut rows = Vec::with_capacity(input.lines.len());
    let mut saved_total = Paisa::ZERO;
    for (index, line) in input.lines.iter().enumerate() {
        match save_advance(ctx, actor, line) {
            Ok(saved) => {
                saved_total = saved_total
                    .checked_add(
                        Paisa::try_from_paisa(line_amount(ctx, saved.voucher_id)?)
                            .map_err(money_error)?,
                    )
                    .map_err(money_error)?;
                rows.push(AdvanceRowResult {
                    saved: Some(saved),
                    errors: Vec::new(),
                });
            }
            Err(ServiceError::Rejected(items)) => {
                let errors = items
                    .into_iter()
                    .map(|item| LineError {
                        code: item.code,
                        field: format!("lines[{index}].{}", item.field.unwrap_or_default()),
                        values: item.values,
                    })
                    .collect();
                rows.push(AdvanceRowResult {
                    saved: None,
                    errors,
                });
            }
            Err(other) => {
                // A failure that is not the user's mistake (the disk, the database). The lines
                // before this one are already saved, so their results must not be lost: this
                // line gets the error and the list goes on, each line being its own voucher.
                tracing::error!(index, error = %other, "a voucher of the list was not saved");
                rows.push(AdvanceRowResult {
                    saved: None,
                    errors: vec![LineError {
                        code: other.code(),
                        field: format!("lines[{index}]"),
                        values: std::collections::BTreeMap::new(),
                    }],
                });
            }
        }
    }
    Ok(AdvancesSaved {
        rows,
        saved_paisa: saved_total,
    })
}

/// The amount of a saved voucher, read back (the total of the list is the sum of what was
/// written, never of what was typed).
fn line_amount(ctx: &Context, voucher_id: i64) -> Result<i64, ServiceError> {
    ctx.db.read(|conn| {
        conn.query_row(
            "SELECT amount_paisa FROM vouchers WHERE id = ?1",
            [voucher_id],
            |r| r.get(0),
        )
        .map_err(ServiceError::from)
    })
}
