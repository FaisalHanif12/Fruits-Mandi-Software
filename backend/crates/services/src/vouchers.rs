//! The voucher framework (PRD 15, plan 10): numbering, the common fields, posting and the running
//! list. Plan 10 has the پیشگی voucher (PRD 15.4); the other kinds use the same numbering, the
//! same list and the same posting function in later plans.
//!
//! One voucher is one posting event in one transaction: the voucher row, its journal entry and
//! its audit row are written together or not at all (Code Standards B3). The running list
//! checks and saves through one plan function, so what the preview showed is what is saved
//! (Code Standards B5).

use std::collections::HashMap;

use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::{self as repo, NewVoucher};
use mandi_domain::core::accounts::{Balance, LineError, RecordRef, SELLER_CONTROL};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::core::vouchers::{
    AdvanceInput, AdvancePreviewRow, AdvanceRowResult, AdvancesInput, AdvancesPreview,
    AdvancesSaved, VoucherMode, VoucherRow, VoucherSaved, VoucherType, VouchersListInput,
};
use mandi_domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger::{self, PostingEvent};

/// The cash in hand (روکڑ, PRD FR-10.4).
const CASH_ACCOUNT_CODE: &str = "01010001";
/// The most lines in one running list.
pub const MAX_LIST_LINES: usize = 500;

/// A پیشگی voucher that passed every check, with the balances around it.
struct PlannedAdvance {
    /// The seller, if the id was right.
    account: Option<RecordRef>,
    /// What is wrong, with the field names as sent (no list prefix yet).
    errors: Vec<FieldError>,
    amount: Paisa,
    date: Option<BusinessDate>,
    mode: VoucherMode,
    /// The ledger account the money comes from: cash, or the bank's account.
    pay_account: Option<i64>,
    bank_account_id: Option<i64>,
    cheque_no: Option<String>,
    cheque_date: Option<BusinessDate>,
    expected_goods: Option<String>,
    narration: Option<String>,
    /// The seller's balance before and after, as signed amounts (debit positive).
    before: Paisa,
    after: Paisa,
}

fn cash_account(conn: &Connection) -> Result<i64, ServiceError> {
    account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ServiceError::Data("the account روکڑ is missing".to_owned()))
}

fn optional_text(errors: &mut validate::FieldErrors, field: &str, text: &str) -> Option<String> {
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// Checks one پیشگی voucher against the database and works out the seller's balance before and
/// after. `running` holds the signed balance of sellers already seen in this list, so a second
/// line for the same seller starts where the first one ended. Used by the preview and the save.
fn plan_advance(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &AdvanceInput,
    running: &mut HashMap<i64, Paisa>,
) -> Result<PlannedAdvance, ServiceError> {
    let (closed, today) = window;
    let mut errors = validate::FieldErrors::default();
    let date = errors.take(validate::posting_date("date", &input.date, closed, today));
    let amount = errors
        .take(validate::money("amount", &input.amount))
        .and_then(|a| {
            if a == Paisa::ZERO {
                errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, "amount"));
                None
            } else {
                Some(a)
            }
        });
    let expected_goods = optional_text(&mut errors, "expected_goods", &input.expected_goods);
    let narration = optional_text(&mut errors, "narration", &input.narration);

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
            if facts.control_code != SELLER_CONTROL || facts.is_pukhta_khata {
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

    // Where the money comes from (PRD 15.1): cash, or a bank account (a cheque posts to the bank
    // at once, A30).
    let mut pay_account = None;
    let mut bank_account_id = None;
    let mut cheque_no = None;
    let mut cheque_date = None;
    match input.mode {
        VoucherMode::Cash => {
            if input.bank_account_id.is_some() {
                errors.push(FieldError::new(
                    ErrorCode::ErrFormatInvalid,
                    "bank_account_id",
                ));
            }
            pay_account = Some(cash_account(conn)?);
        }
        VoucherMode::Bank | VoucherMode::Cheque => {
            match input.bank_account_id {
                None => errors.push(FieldError::new(ErrorCode::ErrRequired, "bank_account_id")),
                Some(id) => match repo::bank_account(conn, id)? {
                    None => errors.push(FieldError::new(ErrorCode::ErrNotFound, "bank_account_id")),
                    Some(bank) if !bank.is_active => errors.push(FieldError::new(
                        ErrorCode::ErrAccountInactive,
                        "bank_account_id",
                    )),
                    Some(bank) => {
                        pay_account = Some(bank.account_id);
                        bank_account_id = Some(id);
                    }
                },
            }
            if input.mode == VoucherMode::Cheque {
                cheque_no = errors
                    .take(validate::free_text("cheque_no", &input.cheque_no))
                    .and_then(|n| {
                        if n.is_empty() {
                            errors.push(FieldError::new(ErrorCode::ErrRequired, "cheque_no"));
                            None
                        } else {
                            Some(n)
                        }
                    });
                if validate::free_text("cheque_date", &input.cheque_date)
                    .map(|t| t.is_empty())
                    .unwrap_or(false)
                {
                    errors.push(FieldError::new(ErrorCode::ErrRequired, "cheque_date"));
                } else {
                    cheque_date =
                        errors.take(validate::date("cheque_date", &input.cheque_date, today));
                }
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
            let after = match amount {
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
        amount: amount.unwrap_or(Paisa::ZERO),
        date,
        mode: input.mode,
        pay_account,
        bank_account_id,
        cheque_no,
        cheque_date,
        expected_goods,
        narration,
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

fn balance_of_net(net: Paisa) -> Result<Balance, ServiceError> {
    Balance::from_net(net).map_err(money_error)
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
        post_advance(ctx, tx, actor, planned)
    })
}

/// Writes a planned, error-free advance inside the caller's transaction.
fn post_advance(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    planned: PlannedAdvance,
) -> Result<VoucherSaved, ServiceError> {
    let (Some(account), Some(date), Some(pay_account)) =
        (planned.account.as_ref(), planned.date, planned.pay_account)
    else {
        return Err(ServiceError::Data(
            "a planned advance is not complete".to_owned(),
        ));
    };
    let now = ctx.now_text();
    let number = repo::next_number(tx, VoucherType::Advance)?;
    let voucher = NewVoucher {
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
    };
    let voucher_id = repo::insert(tx, &voucher, actor.user_id, &now)?;
    // Seller بنام as an advance, cash or bank جمع (PRD 15.4).
    let mut seller_line = LedgerLine::new(AccountId::new(account.id), Side::Dr, planned.amount);
    seller_line.is_advance = true;
    let entry = build_entry(vec![
        seller_line,
        LedgerLine::new(AccountId::new(pay_account), Side::Cr, planned.amount),
    ])
    .map_err(ledger::ledger_error)?;
    let posted = ledger::post(
        ctx,
        tx,
        actor.user_id,
        &PostingEvent {
            date,
            posting_type: PostingType::Advance,
            source_type: SourceType::Voucher,
            source_id: voucher_id,
            narration: planned.narration.clone(),
            reverses: None,
            entry,
        },
    )?;
    repo::set_current_entry(tx, voucher_id, posted.entry_id, actor.user_id, &now)?;
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), "create", "voucher")
            .entity_id(voucher_id)
            .new_value(json!({
                "type": "advance",
                "number": number,
                "entry_date": voucher.entry_date,
                "account_id": account.id,
                "amount_paisa": voucher.amount_paisa,
                "mode": voucher.mode.as_str(),
                "bank_account_id": voucher.bank_account_id,
                "cheque_no": voucher.cheque_no,
                "cheque_date": voucher.cheque_date,
                "expected_goods": voucher.expected_goods,
                "narration": voucher.narration,
            }))
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

/// The vouchers of a kind in a date range, by day and number.
///
/// # Errors
/// The date codes on `from` and `to`; `ServiceError::Db`.
pub fn list(ctx: &Context, input: &VouchersListInput) -> Result<Vec<VoucherRow>, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = validate::FieldErrors::default();
    let bound = |errors: &mut validate::FieldErrors, field: &str, text: &str| {
        if text.trim().is_empty() {
            None
        } else {
            errors
                .take(validate::date(field, text, today))
                .map(BusinessDate::to_iso)
        }
    };
    let from = bound(&mut errors, "from", &input.from);
    let to = bound(&mut errors, "to", &input.to);
    errors.finish()?;
    ctx.db.read(|conn| {
        repo::list(conn, input.voucher_type, from.as_deref(), to.as_deref())
            .map_err(ServiceError::from)
    })
}
