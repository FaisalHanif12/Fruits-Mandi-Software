//! The voucher kinds of the خرچہ screen and ادھار رقم (PRD 16.2, 16.3, 16.5, FR-16.2, FR-16.4,
//! FR-16.9, plan 17): دکان خرچہ, ذاتی خرچہ, ذاتی خرچہ واپسی, ادھار رقم and its return.
//!
//! Each kind keeps its own checks, its own number series and its own posting (PRD FR-7.67). The
//! checks are one `plan_*` function per kind that the preview, the save and the edit all call
//! (Code Standards B5). The journal entry comes from `mandi_domain::expenses`, the same function
//! for the preview and the save. One save is one transaction: the voucher, its journal entry
//! (with the partner shares of a shop expense, written by the hook of plan 11) and its audit row
//! (Code Standards B3).
//!
//! Nothing here posts to a bank: there is no bank, no cheque and no mode (PRD v1.6). The one
//! exception is the salary deduction of ادھار رقم واپسی, which moves no cash at all.

use mandi_db::accounts as account_repo;
use mandi_db::expense_heads::{self as heads, HeadFacts};
use mandi_db::ledger::StoredEntry;
use mandi_db::partners::{self as partner_repo, PartnerRecord};
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::{self as repo, NewVoucher, VoucherExtra};
use mandi_domain::core::accounts::{MONEY_LENT_CONTROL, RecordRef};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::expenses::{PersonalExpenseInput, PersonalReturnInput, ShopExpenseInput};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::lent::{LentReturnMode, MoneyLentInput, MoneyLentReturnInput};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::vouchers::{RemovedPaymentFields, VoucherMode, VoucherSaved, VoucherType};
use mandi_domain::expenses as rules;
use mandi_domain::ledger::BalancedEntry;

use super::common::{
    MoneyText, PostingSpec, cash_change, cash_check, insert_and_post, optional_text, plan_money,
    voucher_json,
};
use super::posting_type::posting_type_of;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::require_confirmed;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::floor::DayNets;
use crate::ledger;

/// The open window for postings and today (PRD FR-6.6).
pub(super) type Window = (Option<BusinessDate>, BusinessDate);

/// A voucher of the خرچہ screen or ادھار رقم that went through the checks of its form.
#[derive(Debug, Clone)]
pub(super) struct PlannedCash {
    /// Which kind it is.
    pub(super) kind: VoucherType,
    /// What is wrong, at the fields as sent.
    pub(super) errors: Vec<FieldError>,
    /// The posting date, if it was read.
    pub(super) date: Option<BusinessDate>,
    /// The amount, if it was read.
    pub(super) amount: Option<Paisa>,
    /// The account that carries the voucher: the expense head's, the partner's drawings, or the
    /// person's.
    pub(super) account: Option<RecordRef>,
    /// The expense head of a shop expense, or the salary head of a salary deduction.
    pub(super) head: Option<HeadFacts>,
    /// The partner of a personal entry.
    pub(super) partner: Option<PartnerRecord>,
    /// "Paid to" of a shop expense.
    pub(super) paid_to: Option<String>,
    /// The journal entry, only when there is no mistake.
    pub(super) entry: Option<BalancedEntry>,
    /// The voucher row with number 0, only when there is no mistake.
    pub(super) template: Option<NewVoucher>,
}

impl PlannedCash {
    /// The journal entry.
    ///
    /// # Errors
    /// `ServiceError::Data` if the entry has a mistake.
    pub(super) fn entry(&self) -> Result<BalancedEntry, ServiceError> {
        self.entry.clone().ok_or_else(incomplete)
    }

    /// The voucher row, with the number it keeps.
    ///
    /// # Errors
    /// `ServiceError::Data` if the entry has a mistake.
    pub(super) fn row(&self, number: i64) -> Result<NewVoucher, ServiceError> {
        let mut row = self.template.clone().ok_or_else(incomplete)?;
        row.number = number;
        Ok(row)
    }
}

pub(super) fn incomplete() -> ServiceError {
    ServiceError::Data("a planned voucher is not complete".to_owned())
}

/// What is open to be returned on an account: never below 0 (money agent 8.2b: Σ open is the debit
/// balance).
///
/// - With a `date` (a return): the smaller of the balance at the end of that day and at the end of
///   every later day, so a return dated before the take it settles cannot be saved, and one that
///   would leave a later day overdrawn cannot either; with no returns dated later it is the
///   balance now (PRD FR-16.2, FR-16.5).
/// - With no date (the open amount of a take, shown by the preview): the balance now.
///
/// `without` is the posting an edit would replace, taken out first. The edit itself posts the
/// reversal before it plans, so it passes none.
pub(super) fn open_of(
    conn: &Connection,
    account_id: i64,
    date: Option<BusinessDate>,
    without: Option<&StoredEntry>,
) -> Result<Paisa, ServiceError> {
    let mut nets = DayNets::load(conn, account_id)?;
    if let Some(entry) = without {
        nets = nets.without(entry, account_id)?;
    }
    let open = match date {
        Some(date) => nets.open_on(&date.to_iso())?,
        None => nets.balance()?,
    };
    Ok(open.max(Paisa::ZERO))
}

/// A return above what is open is refused at `amount` (PRD FR-16.2, FR-16.5).
fn check_return(
    errors: &mut FieldErrors,
    open: Paisa,
    amount: Option<Paisa>,
) -> Result<(), ServiceError> {
    if let Some(amount) = amount
        && rules::return_excess(open, amount)
            .map_err(money_error)?
            .is_some()
    {
        errors.push(
            FieldError::new(ErrorCode::ErrReturnExceedsOpen, "amount")
                .with("open_paisa", ErrorValue::Number(open.as_paisa())),
        );
    }
    Ok(())
}

pub(super) fn record_of_account(
    conn: &Connection,
    id: i64,
) -> Result<Option<RecordRef>, ServiceError> {
    Ok(account_repo::get(conn, id)?.map(|row| RecordRef {
        id: row.id,
        code: row.code,
        name_ur: row.name_ur,
        name_en: row.name_en,
    }))
}

/// The common fields once checked: an open date, an amount above 0, the note (PRD 15.1).
pub(super) fn common_fields(
    conn: &Connection,
    window: Window,
    errors: &mut FieldErrors,
    date: &str,
    amount: &str,
    narration: Option<&str>,
) -> Result<super::common::Money, ServiceError> {
    plan_money(
        conn,
        window,
        errors,
        &MoneyText {
            date,
            amount,
            removed: &RemovedPaymentFields::default(),
            narration: narration.unwrap_or(""),
        },
    )
}

/// A required reason: `ERR_REASON_REQUIRED` when empty.
pub(super) fn required_reason(errors: &mut FieldErrors, text: &str) -> Option<String> {
    match validate::free_text("reason", text) {
        Ok(reason) if reason.is_empty() => {
            errors.push(FieldError::new(ErrorCode::ErrReasonRequired, "reason"));
            None
        }
        Ok(reason) => Some(reason),
        Err(error) => {
            errors.push(error);
            None
        }
    }
}

/// The return-by day (PRD FR-16.9): empty is none; a day before the entry's day is refused, the
/// same day is allowed.
fn return_by_day(
    errors: &mut FieldErrors,
    text: Option<&str>,
    entry_date: Option<BusinessDate>,
    today: BusinessDate,
) -> Option<BusinessDate> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    let day = errors.take(validate::date("return_by", text, today))?;
    if let Some(entry_date) = entry_date
        && day < entry_date
    {
        errors.push(FieldError::new(
            ErrorCode::ErrReturnByBeforeEntry,
            "return_by",
        ));
        return None;
    }
    Some(day)
}

/// The expense head of an entry (or the salary head of a deduction): it exists and is active.
fn resolve_head(
    conn: &Connection,
    errors: &mut FieldErrors,
    id: Option<i64>,
    field: &str,
) -> Result<Option<HeadFacts>, ServiceError> {
    let Some(id) = id else {
        errors.push(FieldError::new(ErrorCode::ErrExpenseHeadRequired, field));
        return Ok(None);
    };
    match heads::head(conn, id)? {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrNotFound, field));
            Ok(None)
        }
        Some(head) if !head.is_active => {
            errors.push(FieldError::new(ErrorCode::ErrAccountInactive, field));
            Ok(None)
        }
        Some(head) => Ok(Some(head)),
    }
}

/// The partner of a personal entry. A take needs an active partner; a return may come from one
/// who has left.
pub(super) fn resolve_partner(
    conn: &Connection,
    errors: &mut FieldErrors,
    id: Option<i64>,
    must_be_active: bool,
) -> Result<Option<PartnerRecord>, ServiceError> {
    let Some(id) = id else {
        errors.push(FieldError::new(ErrorCode::ErrPartnerRequired, "partner_id"));
        return Ok(None);
    };
    match partner_repo::get(conn, id)? {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrNotFound, "partner_id"));
            Ok(None)
        }
        Some(partner) if must_be_active && !partner.is_active => {
            errors.push(FieldError::new(ErrorCode::ErrAccountInactive, "partner_id"));
            Ok(None)
        }
        Some(partner) => Ok(Some(partner)),
    }
}

/// The person of money lent: an account under control 0103 that is active.
fn resolve_person(
    conn: &Connection,
    errors: &mut FieldErrors,
    id: Option<i64>,
) -> Result<Option<RecordRef>, ServiceError> {
    let field = "person_account_id";
    let Some(id) = id else {
        errors.push(FieldError::new(ErrorCode::ErrLentPersonRequired, field));
        return Ok(None);
    };
    let Some(facts) = account_repo::facts(conn, id)? else {
        errors.push(FieldError::new(ErrorCode::ErrNotFound, field));
        return Ok(None);
    };
    if facts.control_code != MONEY_LENT_CONTROL {
        errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, field));
        return Ok(None);
    }
    if !facts.is_active {
        errors.push(FieldError::new(ErrorCode::ErrAccountInactive, field));
        return Ok(None);
    }
    record_of_account(conn, id)
}

/// The row of a voucher of this plan: cash mode, no bank, no cheque.
pub(super) fn template(
    kind: VoucherType,
    date: BusinessDate,
    amount: Paisa,
    account_id: i64,
    narration: Option<String>,
    reason: Option<String>,
    extra: VoucherExtra,
) -> NewVoucher {
    NewVoucher {
        voucher_type: kind,
        number: 0,
        entry_date: date.to_iso(),
        account_id: Some(account_id),
        amount_paisa: amount.as_paisa(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: None,
        cheque_date: None,
        expected_goods: None,
        narration,
        person_key: None,
        collector_staff_id: None,
        reason,
        extra,
    }
}

/// Checks a دکان خرچہ (PRD 16.2): an open date, an amount above 0, an active expense head.
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are in the answer.
pub(super) fn plan_shop_expense(
    conn: &Connection,
    window: Window,
    input: &ShopExpenseInput,
) -> Result<PlannedCash, ServiceError> {
    let mut errors = FieldErrors::default();
    let money = common_fields(
        conn,
        window,
        &mut errors,
        &input.date,
        &input.amount,
        input.narration.as_deref(),
    )?;
    let paid_to = optional_text(
        &mut errors,
        "paid_to",
        input.paid_to.as_deref().unwrap_or(""),
    );
    let head = resolve_head(conn, &mut errors, input.expense_head_id, "expense_head_id")?;
    let account = match &head {
        Some(head) => record_of_account(conn, head.account_id)?,
        None => None,
    };
    let errors = errors.finish().err().unwrap_or_default();
    let (mut entry, mut row) = (None, None);
    if let (true, Some(date), Some(amount), Some(cash), Some(head)) = (
        errors.is_empty(),
        money.date,
        money.amount,
        money.pay_account,
        head.as_ref(),
    ) {
        entry = Some(
            rules::shop_expense_entry(
                AccountId::new(head.account_id),
                AccountId::new(cash),
                amount,
            )
            .map_err(ledger::ledger_error)?,
        );
        // A shop expense has no reason of its own: the column keeps "paid to" (PRD 16.2).
        row = Some(template(
            VoucherType::ShopExpense,
            date,
            amount,
            head.account_id,
            money.narration.clone(),
            paid_to.clone(),
            VoucherExtra {
                expense_head_id: Some(head.id),
                ..VoucherExtra::default()
            },
        ));
    }
    Ok(PlannedCash {
        kind: VoucherType::ShopExpense,
        errors,
        date: money.date,
        amount: money.amount,
        account,
        head,
        partner: None,
        paid_to,
        entry,
        template: row,
    })
}

/// Checks a ذاتی خرچہ (PRD FR-16.2): an active partner, a reason, a return-by day not before the
/// entry (PRD FR-16.9).
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are in the answer.
pub(super) fn plan_personal_expense(
    conn: &Connection,
    window: Window,
    input: &PersonalExpenseInput,
) -> Result<PlannedCash, ServiceError> {
    let mut errors = FieldErrors::default();
    let money = common_fields(
        conn,
        window,
        &mut errors,
        &input.date,
        &input.amount,
        input.narration.as_deref(),
    )?;
    let partner = resolve_partner(conn, &mut errors, input.partner_id, true)?;
    let reason = required_reason(&mut errors, &input.reason);
    let return_by = return_by_day(
        &mut errors,
        input.return_by.as_deref(),
        money.date,
        window.1,
    );
    let taken_by = optional_text(
        &mut errors,
        "taken_by",
        input.taken_by.as_deref().unwrap_or(""),
    );
    let account = match &partner {
        Some(p) => record_of_account(conn, p.drawings_account_id)?,
        None => None,
    };
    let errors = errors.finish().err().unwrap_or_default();
    let (mut entry, mut row) = (None, None);
    if let (true, Some(date), Some(amount), Some(cash), Some(p)) = (
        errors.is_empty(),
        money.date,
        money.amount,
        money.pay_account,
        partner.as_ref(),
    ) {
        entry = Some(
            rules::personal_expense_entry(
                AccountId::new(p.drawings_account_id),
                AccountId::new(cash),
                amount,
            )
            .map_err(ledger::ledger_error)?,
        );
        row = Some(template(
            VoucherType::PersonalExpense,
            date,
            amount,
            p.drawings_account_id,
            money.narration.clone(),
            reason,
            VoucherExtra {
                partner_id: Some(p.id),
                return_by_date: return_by.map(BusinessDate::to_iso),
                taken_by,
                ..VoucherExtra::default()
            },
        ));
    }
    Ok(PlannedCash {
        kind: VoucherType::PersonalExpense,
        errors,
        date: money.date,
        amount: money.amount,
        account,
        head: None,
        partner,
        paid_to: None,
        entry,
        template: row,
    })
}

/// Checks a ذاتی خرچہ واپسی (PRD FR-16.2): a partner, and an amount not above what he has open.
/// `without` is the posting an edit replaces (see [`open_of`]).
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are in the answer.
pub(super) fn plan_personal_return(
    conn: &Connection,
    window: Window,
    input: &PersonalReturnInput,
    without: Option<&StoredEntry>,
) -> Result<PlannedCash, ServiceError> {
    let mut errors = FieldErrors::default();
    let money = common_fields(
        conn,
        window,
        &mut errors,
        &input.date,
        &input.amount,
        input.narration.as_deref(),
    )?;
    let partner = resolve_partner(conn, &mut errors, input.partner_id, false)?;
    let account = match &partner {
        Some(p) => record_of_account(conn, p.drawings_account_id)?,
        None => None,
    };
    if let (Some(p), Some(_)) = (&partner, money.date) {
        let open = open_of(conn, p.drawings_account_id, money.date, without)?;
        check_return(&mut errors, open, money.amount)?;
    }
    let errors = errors.finish().err().unwrap_or_default();
    let (mut entry, mut row) = (None, None);
    if let (true, Some(date), Some(amount), Some(cash), Some(p)) = (
        errors.is_empty(),
        money.date,
        money.amount,
        money.pay_account,
        partner.as_ref(),
    ) {
        entry = Some(
            rules::personal_return_entry(
                AccountId::new(p.drawings_account_id),
                AccountId::new(cash),
                amount,
            )
            .map_err(ledger::ledger_error)?,
        );
        row = Some(template(
            VoucherType::PersonalReturn,
            date,
            amount,
            p.drawings_account_id,
            money.narration.clone(),
            None,
            VoucherExtra {
                partner_id: Some(p.id),
                ..VoucherExtra::default()
            },
        ));
    }
    Ok(PlannedCash {
        kind: VoucherType::PersonalReturn,
        errors,
        date: money.date,
        amount: money.amount,
        account,
        head: None,
        partner,
        paid_to: None,
        entry,
        template: row,
    })
}

/// Checks money lent (PRD 16.5): a person under control 0103, a reason, a return-by day not
/// before the entry (PRD FR-16.9).
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are in the answer.
pub(super) fn plan_money_lent(
    conn: &Connection,
    window: Window,
    input: &MoneyLentInput,
) -> Result<PlannedCash, ServiceError> {
    let mut errors = FieldErrors::default();
    let money = common_fields(
        conn,
        window,
        &mut errors,
        &input.date,
        &input.amount,
        input.narration.as_deref(),
    )?;
    let account = resolve_person(conn, &mut errors, input.person_account_id)?;
    let reason = required_reason(&mut errors, &input.reason);
    let return_by = return_by_day(
        &mut errors,
        input.return_by.as_deref(),
        money.date,
        window.1,
    );
    let errors = errors.finish().err().unwrap_or_default();
    let (mut entry, mut row) = (None, None);
    if let (true, Some(date), Some(amount), Some(cash), Some(person)) = (
        errors.is_empty(),
        money.date,
        money.amount,
        money.pay_account,
        account.as_ref(),
    ) {
        entry = Some(
            rules::money_lent_entry(AccountId::new(person.id), AccountId::new(cash), amount)
                .map_err(ledger::ledger_error)?,
        );
        row = Some(template(
            VoucherType::MoneyLent,
            date,
            amount,
            person.id,
            money.narration.clone(),
            reason,
            VoucherExtra {
                return_by_date: return_by.map(BusinessDate::to_iso),
                ..VoucherExtra::default()
            },
        ));
    }
    Ok(PlannedCash {
        kind: VoucherType::MoneyLent,
        errors,
        date: money.date,
        amount: money.amount,
        account,
        head: None,
        partner: None,
        paid_to: None,
        entry,
        template: row,
    })
}

/// Checks a return of money lent, in cash or from salary (PRD FR-16.4): an amount not above what
/// the person has open. A salary deduction names its head, the head called تنخواہ when none is
/// sent. `without` is the posting an edit replaces (see [`open_of`]).
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are in the answer.
pub(super) fn plan_money_lent_return(
    conn: &Connection,
    window: Window,
    input: &MoneyLentReturnInput,
    without: Option<&StoredEntry>,
) -> Result<PlannedCash, ServiceError> {
    let mut errors = FieldErrors::default();
    let money = common_fields(
        conn,
        window,
        &mut errors,
        &input.date,
        &input.amount,
        input.narration.as_deref(),
    )?;
    let account = resolve_person(conn, &mut errors, input.person_account_id)?;
    let salary = input.mode == LentReturnMode::Salary;
    let head = if salary {
        // The default is the head called تنخواہ (PRD FR-16.4).
        let id = match input.salary_head_id {
            Some(id) => Some(id),
            None => heads::salary_head(conn)?.map(|h| h.id),
        };
        resolve_head(conn, &mut errors, id, "salary_head_id")?
    } else {
        if input.salary_head_id.is_some() {
            errors.push(FieldError::new(
                ErrorCode::ErrFormatInvalid,
                "salary_head_id",
            ));
        }
        None
    };
    if let (Some(person), Some(_)) = (&account, money.date) {
        let open = open_of(conn, person.id, money.date, without)?;
        check_return(&mut errors, open, money.amount)?;
    }
    let errors = errors.finish().err().unwrap_or_default();
    let (mut entry, mut row) = (None, None);
    if let (true, Some(date), Some(amount), Some(cash), Some(person)) = (
        errors.is_empty(),
        money.date,
        money.amount,
        money.pay_account,
        account.as_ref(),
    ) {
        let person_id = AccountId::new(person.id);
        let built = match &head {
            Some(head) => rules::money_lent_return_salary_entry(
                person_id,
                AccountId::new(head.account_id),
                amount,
            ),
            None => rules::money_lent_return_cash_entry(person_id, AccountId::new(cash), amount),
        };
        entry = Some(built.map_err(ledger::ledger_error)?);
        row = Some(template(
            VoucherType::MoneyLentReturn,
            date,
            amount,
            person.id,
            money.narration.clone(),
            None,
            VoucherExtra {
                expense_head_id: head.as_ref().map(|h| h.id),
                salary,
                ..VoucherExtra::default()
            },
        ));
    }
    Ok(PlannedCash {
        kind: VoucherType::MoneyLentReturn,
        errors,
        date: money.date,
        amount: money.amount,
        account,
        head,
        partner: None,
        paid_to: None,
        entry,
        template: row,
    })
}

/// Saves a planned voucher without a mistake, inside the caller's transaction: the voucher row,
/// its journal entry (the hook of plan 11 writes the partner shares of a shop expense and of a
/// salary deduction) and its audit row. Asks the cash question of PRD FR-6.7 first.
pub(super) fn post_planned(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    planned: &PlannedCash,
    confirm_cash_below_zero: bool,
) -> Result<VoucherSaved, ServiceError> {
    let (Some(date), Some(account)) = (planned.date, planned.account.as_ref()) else {
        return Err(incomplete());
    };
    let entry = planned.entry()?;
    require_confirmed(
        &cash_check(tx, cash_change(tx, &entry)?)?,
        confirm_cash_below_zero,
    )?;
    let number = repo::next_number(tx, planned.kind)?;
    let voucher = planned.row(number)?;
    let (voucher_id, posted) = insert_and_post(
        ctx,
        tx,
        actor.user_id,
        PostingSpec {
            posting_type: posting_type_of(planned.kind),
            date,
            narration: voucher.narration.clone(),
        },
        &voucher,
        entry,
    )?;
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), "create", "voucher")
            .entity_id(voucher_id)
            .new_value(voucher_json(number, planned.kind.as_str(), &voucher))
            .into_entry(),
    )?;
    Ok(VoucherSaved {
        voucher_id,
        voucher_type: planned.kind,
        number,
        entry_id: posted.entry_id,
        new_balance: ledger::balance_in(tx, account.id, None, None)?,
    })
}

/// Runs one save: plan, refuse on a mistake, post. One transaction.
pub(super) fn save_with(
    ctx: &Context,
    actor: &Actor,
    confirm_cash_below_zero: bool,
    plan: impl FnOnce(&Connection, Window) -> Result<PlannedCash, ServiceError>,
) -> Result<VoucherSaved, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.write(|tx| {
        let planned = plan(tx, window)?;
        if !planned.errors.is_empty() {
            return Err(planned.errors.into());
        }
        post_planned(ctx, tx, actor, &planned, confirm_cash_below_zero)
    })
}

/// Saves a دکان خرچہ (PRD 16.2): head بنام, روکڑ جمع, split by the shares of the date through the
/// hook of plan 11 (E2E-20).
///
/// # Errors
/// The date codes on `date`; `ERR_EXPENSE_HEAD_REQUIRED`, `ERR_NOT_FOUND` and `ERR_ACCOUNT_INACTIVE`
/// on `expense_head_id`; the money codes on `amount`; the text codes on `paid_to` and `narration`;
/// `ERR_SHARES_NOT_100`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written when any of them
/// is returned.
pub fn save_shop_expense(
    ctx: &Context,
    actor: &Actor,
    input: &ShopExpenseInput,
) -> Result<VoucherSaved, ServiceError> {
    save_with(ctx, actor, input.confirm_cash_below_zero, |conn, window| {
        plan_shop_expense(conn, window, input)
    })
}

/// Saves a ذاتی خرچہ (PRD 16.3): the partner's drawings account بنام, روکڑ جمع. Never split.
///
/// # Errors
/// As [`save_shop_expense`], with `ERR_PARTNER_REQUIRED`, `ERR_NOT_FOUND` and
/// `ERR_ACCOUNT_INACTIVE` on `partner_id`, `ERR_REASON_REQUIRED` on `reason` and
/// `ERR_RETURN_BY_BEFORE_ENTRY` on `return_by`.
pub fn save_personal_expense(
    ctx: &Context,
    actor: &Actor,
    input: &PersonalExpenseInput,
) -> Result<VoucherSaved, ServiceError> {
    save_with(ctx, actor, input.confirm_cash_below_zero, |conn, window| {
        plan_personal_expense(conn, window, input)
    })
}

/// Saves a ذاتی خرچہ واپسی (PRD 16.3): روکڑ بنام, drawings جمع. Cash comes in, so it never asks
/// the cash question.
///
/// # Errors
/// The date and money codes, `ERR_PARTNER_REQUIRED` and `ERR_NOT_FOUND` on `partner_id`,
/// `ERR_RETURN_EXCEEDS_OPEN` (value `open_paisa`) on `amount`.
pub fn save_personal_return(
    ctx: &Context,
    actor: &Actor,
    input: &PersonalReturnInput,
) -> Result<VoucherSaved, ServiceError> {
    save_with(ctx, actor, false, |conn, window| {
        plan_personal_return(conn, window, input, None)
    })
}

/// Saves money lent (PRD 16.5): the person's account بنام, روکڑ جمع.
///
/// # Errors
/// The date and money codes, `ERR_LENT_PERSON_REQUIRED`, `ERR_NOT_FOUND`, `ERR_FORMAT_INVALID`
/// and `ERR_ACCOUNT_INACTIVE` on `person_account_id`, `ERR_REASON_REQUIRED` on `reason`,
/// `ERR_RETURN_BY_BEFORE_ENTRY` on `return_by`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
pub fn save_money_lent(
    ctx: &Context,
    actor: &Actor,
    input: &MoneyLentInput,
) -> Result<VoucherSaved, ServiceError> {
    save_with(ctx, actor, input.confirm_cash_below_zero, |conn, window| {
        plan_money_lent(conn, window, input)
    })
}

/// Saves a return of money lent, in cash or from salary (PRD FR-16.4). A salary deduction moves
/// no cash and is split between the partners as a shop expense.
///
/// # Errors
/// As [`save_money_lent`] without the cash question, with `ERR_RETURN_EXCEEDS_OPEN` (value
/// `open_paisa`) on `amount` and `ERR_EXPENSE_HEAD_REQUIRED`, `ERR_NOT_FOUND` on `salary_head_id`.
pub fn save_money_lent_return(
    ctx: &Context,
    actor: &Actor,
    input: &MoneyLentReturnInput,
) -> Result<VoucherSaved, ServiceError> {
    save_with(ctx, actor, false, |conn, window| {
        plan_money_lent_return(conn, window, input, None)
    })
}
