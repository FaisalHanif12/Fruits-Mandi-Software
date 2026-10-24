//! جنرل: a journal entry of free lines (PRD 16.4, FR-6.2, FR-6.5, FR-15.4, plan 17).
//!
//! It is saved only when Σ بنام = Σ جمع and it has at least two lines, each with one side. The
//! preview and the save read the lines with the same function, [`plan_general`], and build the
//! entry with the same domain function (Code Standards B5).
//!
//! - Lines on income or shop expense accounts are split between the partners by the hook of plan 11
//!   inside the posting (PRD FR-6.5); the preview shows the same parts from the same hook.
//! - Lines on buyer accounts take part in bill matching (PRD FR-15.4): after the posting each
//!   buyer key is matched again, oldest first.
//! - A line on a seller account never adjusts his advances (PRD 16.4): a debit settles his
//!   payables and a credit is a payable posting. Nothing here writes advance matching.
//! - A line on اپنا مال is refused (PRD FR-12.26), and so is a line on نقدی منڈی (it never holds a
//!   balance) and on the bank (it is not used from v1.6).
//! - A cash credit asks the cash question of PRD FR-6.7.

use std::collections::HashMap;

use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::{self as repo, NewVoucher, VoucherExtra};
use mandi_domain::cash;
use mandi_domain::core::accounts::{Balance, NAQDI_MANDI_CODE, OWN_GOODS_CODE, RecordRef};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::expenses::ExpenseShare;
use mandi_domain::core::general::{
    GeneralInput, GeneralLineInput, GeneralLinePreview, GeneralPreview, GeneralPreviewInput,
};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::vouchers::{VoucherMode, VoucherSaved, VoucherType};
use mandi_domain::general::{GeneralLine, MIN_LINES, general_entry, totals};
use mandi_domain::ledger::{BalancedEntry, LedgerError, MAX_LINE_PAISA, Side};
use serde_json::{Value, json};

use super::cash_kinds::{Window, incomplete};
use super::common::{
    PostingSpec, Role, cash_account, cash_change, cash_check, insert_and_post, optional_text,
    rematch_party, role_of_account, voucher_json,
};
use super::expense_reads::split_errors;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::{cash_in_hand, require_confirmed};
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::partners::split_entry;
use crate::ventures::record_of;

/// The most lines in one جنرل entry.
const MAX_GENERAL_LINES: usize = super::MAX_LIST_LINES;
/// The bank account of the first migration: not used from v1.6 (PRD FR-10.4).
const BANK_CODE: &str = "01020001";

/// One line of the entry as it was read: what could be read of it and what is wrong with it.
#[derive(Debug, Clone)]
pub(super) struct ReadLine {
    /// The account, if the id was right.
    pub account: Option<RecordRef>,
    /// بنام, 0 if empty or unreadable.
    pub dr: Paisa,
    /// جمع, 0 if empty or unreadable.
    pub cr: Paisa,
    /// What is wrong with this line, at `lines[i].<field>`.
    pub errors: Vec<FieldError>,
    /// The person key, as sent.
    pub person_key: Option<String>,
    /// The line as the entry takes it, only when the line has no mistake.
    line: Option<GeneralLine>,
}

/// A جنرل entry that went through the checks of its form.
#[derive(Debug, Clone)]
pub(super) struct PlannedGeneral {
    /// Every mistake, in the order: date, lines, the whole entry.
    pub(super) errors: Vec<FieldError>,
    /// What belongs to the whole entry, not to one line (`lines`, `date`).
    pub(super) entry_errors: Vec<FieldError>,
    /// The posting date, if it was read.
    pub(super) date: Option<BusinessDate>,
    /// One row per line sent, in order.
    pub(super) lines: Vec<ReadLine>,
    /// Σ بنام of the lines that could be read.
    pub(super) total_dr: Paisa,
    /// Σ جمع of the lines that could be read.
    pub(super) total_cr: Paisa,
    entry: Option<BalancedEntry>,
    template: Option<NewVoucher>,
}

impl PlannedGeneral {
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

    /// The accounts and people the entry touches, for matching.
    pub(super) fn touched(&self) -> Vec<(i64, Option<String>)> {
        self.lines
            .iter()
            .filter_map(|l| l.line.as_ref())
            .map(|l| (l.account_id, l.person_key.clone()))
            .collect()
    }

    /// The lines as the audit log keeps them (Code Standards S7).
    pub(super) fn lines_json(&self) -> Value {
        Value::Array(
            self.lines
                .iter()
                .filter_map(|l| l.line.as_ref())
                .map(|l| {
                    json!({
                        "account_id": l.account_id,
                        "person_key": l.person_key,
                        "side": if l.side == Side::Dr { "dr" } else { "cr" },
                        "amount_paisa": l.amount.as_paisa(),
                        "narration": l.narration,
                    })
                })
                .collect(),
        )
    }
}

/// Reads one line (PRD 16.4): the account (exists, active, one a جنرل may use), the person of a
/// پختہ کھاتہ account, one side with an amount above 0, and the note.
fn read_line(
    conn: &Connection,
    index: usize,
    input: &GeneralLineInput,
) -> Result<ReadLine, ServiceError> {
    let path = |field: &str| format!("lines[{index}].{field}");
    let mut errors = FieldErrors::default();
    let mut account = None;
    let mut is_pukhta = false;
    match input.account_id {
        None => errors.push(FieldError::new(ErrorCode::ErrRequired, &path("account_id"))),
        Some(id) => match account_repo::facts(conn, id)? {
            None => errors.push(FieldError::new(ErrorCode::ErrNotFound, &path("account_id"))),
            Some(facts) => {
                let field = path("account_id");
                if facts.code == OWN_GOODS_CODE {
                    errors.push(FieldError::new(
                        ErrorCode::ErrOwnGoodsAccountRestricted,
                        &field,
                    ));
                } else if facts.code == NAQDI_MANDI_CODE
                    || facts.code == BANK_CODE
                    // A garden's money moves only by its own entries and its closing (PRD FR-12.34 to
                    // FR-12.38), never by a جنرل line.
                    || facts.control_code == mandi_domain::core::accounts::GARDEN_CONTROL_CODE
                {
                    errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, &field));
                } else if !facts.is_active {
                    errors.push(FieldError::new(ErrorCode::ErrAccountInactive, &field));
                } else {
                    is_pukhta = facts.is_pukhta_khata;
                    account = super::cash_kinds::record_of_account(conn, id)?;
                }
            }
        },
    }
    // The person: needed for a پختہ کھاتہ account and known there, refused for any other.
    let key = input
        .person_key
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty());
    let mut person_key = None;
    if let Some(account) = &account {
        match (is_pukhta, key) {
            (true, None) => errors.push(FieldError::new(
                ErrorCode::ErrPersonRequired,
                &path("person_key"),
            )),
            (true, Some(key)) => {
                if repo::person_has_lines(conn, account.id, key)? {
                    person_key = Some(key.to_owned());
                } else {
                    errors.push(FieldError::new(ErrorCode::ErrNotFound, &path("person_key")));
                }
            }
            (false, Some(_)) => errors.push(FieldError::new(
                ErrorCode::ErrFormatInvalid,
                &path("person_key"),
            )),
            (false, None) => {}
        }
    }
    let dr = errors.take(validate::money_optional(&path("dr"), &input.dr));
    let cr = errors.take(validate::money_optional(&path("cr"), &input.cr));
    let (dr_read, cr_read) = (dr.is_some(), cr.is_some());
    let dr_amount = dr.flatten().unwrap_or(Paisa::ZERO);
    let cr_amount = cr.flatten().unwrap_or(Paisa::ZERO);
    // One side only. A line with nothing on either side is asking for an amount.
    if dr_amount > Paisa::ZERO && cr_amount > Paisa::ZERO {
        errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, &path("cr")));
    } else if dr_amount == Paisa::ZERO && cr_amount == Paisa::ZERO && dr_read && cr_read {
        errors.push(FieldError::new(ErrorCode::ErrRequired, &path("dr")));
    }
    let narration = optional_text(
        &mut errors,
        &path("narration"),
        input.narration.as_deref().unwrap_or(""),
    );
    let errors = errors.finish().err().unwrap_or_default();
    let line = match (&account, errors.is_empty()) {
        (Some(account), true) => {
            let (side, amount) = if dr_amount > Paisa::ZERO {
                (Side::Dr, dr_amount)
            } else {
                (Side::Cr, cr_amount)
            };
            Some(GeneralLine {
                account_id: account.id,
                person_key: person_key.clone(),
                side,
                amount,
                narration,
            })
        }
        _ => None,
    };
    Ok(ReadLine {
        account,
        dr: dr_amount,
        cr: cr_amount,
        errors,
        person_key,
        line,
    })
}

/// Checks a جنرل entry as typed: the date (when one is sent, or when `need_date`), every line,
/// the number of lines and that the totals are equal. Nothing is written.
///
/// # Errors
/// `ServiceError::Db`, `ERR_OVERFLOW`; the user's mistakes are in the answer.
pub(super) fn plan_general(
    conn: &Connection,
    window: Window,
    date: Option<&str>,
    narration: Option<&str>,
    lines: &[GeneralLineInput],
    need_date: bool,
) -> Result<PlannedGeneral, ServiceError> {
    let (closed, today) = window;
    let mut head_errors = FieldErrors::default();
    let date_text = date.map(str::trim).filter(|d| !d.is_empty());
    let date = match (date_text, need_date) {
        (Some(text), _) => head_errors.take(validate::posting_date("date", text, closed, today)),
        (None, true) => {
            head_errors.push(FieldError::new(ErrorCode::ErrRequired, "date"));
            None
        }
        (None, false) => None,
    };
    let narration = optional_text(&mut head_errors, "narration", narration.unwrap_or(""));
    let mut entry_errors: Vec<FieldError> = head_errors.finish().err().unwrap_or_default();
    let mut read = Vec::with_capacity(lines.len().min(MAX_GENERAL_LINES));
    for (index, line) in lines.iter().take(MAX_GENERAL_LINES).enumerate() {
        read.push(read_line(conn, index, line)?);
    }
    if lines.len() > MAX_GENERAL_LINES {
        entry_errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, "lines"));
    }
    // The totals of every line that could be read, also when another line has a mistake, so the
    // screen can show the difference while the user types.
    let (mut total_dr, mut total_cr) = (Paisa::ZERO, Paisa::ZERO);
    for line in &read {
        total_dr = total_dr.checked_add(line.dr).map_err(money_error)?;
        total_cr = total_cr.checked_add(line.cr).map_err(money_error)?;
    }
    let lines_clean = read.iter().all(|l| l.errors.is_empty());
    let mut entry = None;
    let mut template = None;
    if lines.len() < MIN_LINES {
        entry_errors.push(FieldError::new(ErrorCode::ErrGeneralTooFewLines, "lines"));
    } else if lines_clean && entry_errors.is_empty() {
        let built: Vec<GeneralLine> = read.iter().filter_map(|l| l.line.clone()).collect();
        let (dr, cr) = totals(&built).map_err(money_error)?;
        if dr != cr {
            entry_errors.push(
                FieldError::new(ErrorCode::ErrGeneralNotBalanced, "lines")
                    .with("dr_paisa", ErrorValue::Number(dr.as_paisa()))
                    .with("cr_paisa", ErrorValue::Number(cr.as_paisa())),
            );
        } else {
            match general_entry(&built) {
                Ok(made) => {
                    template = date.map(|day| NewVoucher {
                        voucher_type: VoucherType::General,
                        number: 0,
                        entry_date: day.to_iso(),
                        account_id: None,
                        amount_paisa: made.total().as_paisa(),
                        mode: VoucherMode::Cash,
                        bank_account_id: None,
                        cheque_no: None,
                        cheque_date: None,
                        expected_goods: None,
                        narration: narration.clone(),
                        person_key: None,
                        collector_staff_id: None,
                        reason: None,
                        extra: VoucherExtra::default(),
                    });
                    entry = Some(made);
                }
                Err(LedgerError::TooLarge) => {
                    entry_errors.push(FieldError::new(ErrorCode::ErrEntryTooLarge, "lines").with(
                        "max",
                        ErrorValue::Number(MAX_LINE_PAISA.checked_div(100).unwrap_or(i64::MAX)),
                    ))
                }
                Err(other) => return Err(ledger::ledger_error(other)),
            }
        }
    }
    // The date comes first, then the lines in order, then the whole entry.
    let mut errors: Vec<FieldError> = entry_errors
        .iter()
        .filter(|e| e.field == "date" || e.field == "narration")
        .cloned()
        .collect();
    for line in &read {
        errors.extend(line.errors.iter().cloned());
    }
    errors.extend(
        entry_errors
            .iter()
            .filter(|e| e.field != "date" && e.field != "narration")
            .cloned(),
    );
    // A mistake anywhere means there is nothing to post.
    if !errors.is_empty() {
        entry = None;
        template = None;
    }
    Ok(PlannedGeneral {
        errors,
        entry_errors,
        date,
        lines: read,
        total_dr,
        total_cr,
        entry,
        template,
    })
}

/// Matches again each buyer key that the lines touch, oldest first (PRD FR-15.4). A seller or any
/// other account is left alone: a جنرل never adjusts an advance (PRD 16.4).
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn rematch_buyers(
    tx: &Connection,
    user_id: i64,
    now: &str,
    lines: &[(i64, Option<String>)],
) -> Result<(), ServiceError> {
    let mut seen: Vec<&(i64, Option<String>)> = Vec::new();
    for key in lines {
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        if let Some((Role::Buyer, is_pukhta)) = role_of_account(tx, key.0)? {
            rematch_party(
                tx,
                user_id,
                now,
                key.0,
                key.1.as_deref(),
                Role::Buyer,
                is_pukhta,
            )?;
        }
    }
    Ok(())
}

/// A working with the named values (Code Standards B30).
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

/// The partners' parts of the income and shop expense lines, net of the lines (PRD FR-6.5), from
/// the same hook that writes the share rows. A part is positive when the entry is an expense for
/// him and negative when it is income, so a pure shop expense reads like the خرچہ screen.
fn split_of(
    conn: &Connection,
    date: BusinessDate,
    entry: &BalancedEntry,
    errors: &mut Vec<FieldError>,
) -> Result<Vec<ExpenseShare>, ServiceError> {
    let drafts = match split_entry(conn, date, entry) {
        Ok(drafts) => drafts,
        Err(error) => {
            errors.extend(split_errors(error)?);
            return Ok(Vec::new());
        }
    };
    let names: HashMap<i64, RecordRef> = mandi_db::partners::list(conn)?
        .iter()
        .map(|p| (p.id, record_of(p.id, &p.name_ur, &p.name_en)))
        .collect();
    // One row per partner: income parts and expense parts added up (income plus, expense minus).
    let mut order: Vec<i64> = Vec::new();
    let mut net: HashMap<i64, (Paisa, i32)> = HashMap::new();
    for draft in &drafts {
        let id = draft.partner_id.get();
        if !order.contains(&id) {
            order.push(id);
        }
        let slot = net
            .entry(id)
            .or_insert((Paisa::ZERO, draft.share_bps.get()));
        slot.0 = slot.0.checked_add(draft.amount).map_err(money_error)?;
    }
    let mut shares = Vec::with_capacity(order.len());
    for id in order {
        let Some((gain, bps)) = net.get(&id).copied() else {
            continue;
        };
        let part = gain.checked_neg().map_err(money_error)?;
        let partner = names
            .get(&id)
            .cloned()
            .ok_or_else(|| ServiceError::Data("a share row names no partner".to_owned()))?;
        shares.push(ExpenseShare {
            partner,
            share_bps: bps,
            amount_paisa: part,
            working: working(
                "working-general-share",
                vec![
                    ("bps", WorkingValue::Percent { bps }),
                    ("result", money_value(part)),
                ],
            ),
        });
    }
    Ok(shares)
}

/// What a جنرل entry would do to cash in hand, from the lines that could be read.
fn cash_effect(
    conn: &Connection,
    planned: &PlannedGeneral,
) -> Result<mandi_domain::core::cash::CashCheck, ServiceError> {
    let cash_id = cash_account(conn)?;
    let mut change = Paisa::ZERO;
    for line in &planned.lines {
        if line.account.as_ref().is_some_and(|a| a.id == cash_id) {
            change = change
                .checked_add(line.dr)
                .and_then(|v| v.checked_sub(line.cr))
                .map_err(money_error)?;
        }
    }
    if change.is_negative() {
        cash::check(cash_in_hand(conn)?, change)
    } else {
        cash::check_unread(change)
    }
    .map_err(money_error)
}

/// Checks a جنرل entry for the screen (PRD 16.4): each line's account and balance after the
/// entry, the totals, the difference, whether it can be saved, the partner split and the cash
/// note. The same function as the save (Code Standards B5). Nothing is written.
///
/// # Errors
/// `ServiceError::Db`, `ERR_OVERFLOW`; a mistake of the user is in the answer.
pub fn general_preview(
    ctx: &Context,
    input: &GeneralPreviewInput,
) -> Result<GeneralPreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| {
        let planned = plan_general(
            conn,
            window,
            input.date.as_deref(),
            None,
            &input.lines,
            false,
        )?;
        // The net of the lines on one account and person, for the balance after the entry.
        let mut nets: HashMap<(i64, Option<String>), Paisa> = HashMap::new();
        for line in &planned.lines {
            if let Some(account) = &line.account {
                let slot = nets
                    .entry((account.id, line.person_key.clone()))
                    .or_insert(Paisa::ZERO);
                *slot = slot
                    .checked_add(line.dr)
                    .and_then(|v| v.checked_sub(line.cr))
                    .map_err(money_error)?;
            }
        }
        let clean = planned.errors.is_empty();
        let mut lines = Vec::with_capacity(planned.lines.len());
        for line in &planned.lines {
            let balance_after = match (&line.account, line.errors.is_empty()) {
                (Some(account), true) => {
                    let now = ledger::signed(ledger::balance_in(
                        conn,
                        account.id,
                        line.person_key.as_deref(),
                        None,
                    )?)?;
                    let net = nets
                        .get(&(account.id, line.person_key.clone()))
                        .copied()
                        .unwrap_or(Paisa::ZERO);
                    Some(
                        Balance::from_net(now.checked_add(net).map_err(money_error)?)
                            .map_err(money_error)?,
                    )
                }
                _ => None,
            };
            lines.push(GeneralLinePreview {
                account: line.account.clone(),
                dr_paisa: line.dr,
                cr_paisa: line.cr,
                balance_after,
                errors: line.errors.clone(),
            });
        }
        let difference = planned
            .total_dr
            .checked_sub(planned.total_cr)
            .map_err(money_error)?;
        let is_balanced = difference == Paisa::ZERO && planned.total_dr > Paisa::ZERO;
        let mut errors = planned.entry_errors.clone();
        let split = match (planned.date, planned.entry.as_ref()) {
            (Some(date), Some(entry)) if clean => split_of(conn, date, entry, &mut errors)?,
            _ => Vec::new(),
        };
        // A split refused (shares not 100%) stops the save as well.
        let can_save = clean && errors.is_empty() && is_balanced && planned.date.is_some();
        Ok(GeneralPreview {
            lines,
            total_dr_paisa: planned.total_dr,
            total_cr_paisa: planned.total_cr,
            difference_paisa: difference,
            is_balanced,
            can_save,
            split,
            cash: cash_effect(conn, &planned)?,
            errors,
            warnings: Vec::new(),
        })
    })
}

/// Saves a planned جنرل without a mistake, inside the caller's transaction: the voucher row, its
/// journal entry (the hook of plan 11 writes the partner shares), the matching of each buyer it
/// touches and its audit row. Asks the cash question of PRD FR-6.7 first.
fn post_general(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    planned: &PlannedGeneral,
    confirm_cash_below_zero: bool,
) -> Result<VoucherSaved, ServiceError> {
    let date = planned.date.ok_or_else(incomplete)?;
    let entry = planned.entry()?;
    require_confirmed(
        &cash_check(tx, cash_change(tx, &entry)?)?,
        confirm_cash_below_zero,
    )?;
    let number = repo::next_number(tx, VoucherType::General)?;
    let voucher = planned.row(number)?;
    let (voucher_id, posted) = insert_and_post(
        ctx,
        tx,
        actor.user_id,
        PostingSpec {
            posting_type: super::posting_type::posting_type_of(VoucherType::General),
            date,
            narration: voucher.narration.clone(),
        },
        &voucher,
        entry,
    )?;
    rematch_buyers(tx, actor.user_id, &ctx.now_text(), &planned.touched())?;
    let mut value = voucher_json(number, VoucherType::General.as_str(), &voucher);
    if let Some(map) = value.as_object_mut() {
        map.insert("lines".to_owned(), planned.lines_json());
    }
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), "create", "voucher")
            .entity_id(voucher_id)
            .new_value(value)
            .into_entry(),
    )?;
    Ok(VoucherSaved {
        voucher_id,
        voucher_type: VoucherType::General,
        number,
        entry_id: posted.entry_id,
        new_balance: Balance::ZERO,
    })
}

/// Saves a جنرل entry only when Σ بنام = Σ جمع (PRD 16.4, FR-6.2).
///
/// # Errors
/// The date codes; the line codes at `lines[i].<field>`; `ERR_GENERAL_TOO_FEW_LINES` and
/// `ERR_GENERAL_NOT_BALANCED` (values `dr_paisa`, `cr_paisa`) on `lines`;
/// `ERR_OWN_GOODS_ACCOUNT_RESTRICTED` on `lines[i].account_id`; `ERR_SHARES_NOT_100`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`). Nothing is written when any of
/// them is returned.
pub fn general_save(
    ctx: &Context,
    actor: &Actor,
    input: &GeneralInput,
) -> Result<VoucherSaved, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.write(|tx| {
        let planned = plan_general(
            tx,
            window,
            Some(&input.date),
            input.narration.as_deref(),
            &input.lines,
            true,
        )?;
        if !planned.errors.is_empty() {
            return Err(planned.errors.into());
        }
        post_general(ctx, tx, actor, &planned, input.confirm_cash_below_zero)
    })
}
