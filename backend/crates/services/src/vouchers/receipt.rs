//! The وصولی voucher (PRD 15.2, FR-15.4, FR-15.5, FR-15.12, FR-13.7, plan 15): money received
//! from a buyer, or an advance returned by a seller.
//!
//! - **Buyer** (also a one-time buyer, by his person key): cash or bank بنام, the buyer جمع. The
//!   credit settles his oldest open bills first; what is more than all his bills stays as credit
//!   and settles his next bill (PRD FR-15.5).
//! - **Seller** (PRD FR-15.12): the same posting. The credit adjusts his open advances, oldest
//!   first, and the rest settles what he owes the firm beyond them. A receipt may not make more
//!   payable to him than before it (money agent review of plan 15, problem 1): for a receipt
//!   dated today this is "no more than his open advances plus what he owes".
//!
//! One receipt is one posting event in one transaction (Code Standards B3).

use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::{self as repo, NewVoucher};
use mandi_domain::core::accounts::LineError;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::{MoneyError, Paisa};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::vouchers::{
    ReceiptInput, ReceiptRowResult, ReceiptsInput, ReceiptsSaved, VoucherSaved, VoucherType,
};
use mandi_domain::ledger::{BalancedEntry, LedgerLine, PostingType, Side, build_entry};
use mandi_domain::payables;

use super::MAX_LIST_LINES;
use super::common::{
    Accepts, Hint, Money, MoneyText, Party, PostingSpec, Role, check_collector, hint_for,
    insert_and_post, plan_money, resolve_party, stored_line, voucher_json,
};
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger;
use crate::matching::{self, AdvChange, NewLine, SellerView};

/// A receipt that went through the checks of the form.
#[derive(Debug, Clone)]
pub(super) struct PlannedReceipt {
    pub party: Option<Party>,
    pub money: Money,
    pub collector: Option<i64>,
    /// What is wrong, at the fields as sent.
    pub errors: Vec<FieldError>,
}

impl PlannedReceipt {
    /// The pieces a posting needs, when there is no mistake.
    pub(super) fn parts(&self) -> Result<(&Party, BusinessDate, Paisa, i64), ServiceError> {
        match (
            self.party.as_ref(),
            self.money.date,
            self.money.amount,
            self.money.pay_account,
        ) {
            (Some(party), Some(date), Some(amount), Some(pay)) if self.errors.is_empty() => {
                Ok((party, date, amount, pay))
            }
            _ => Err(ServiceError::Data(
                "a planned receipt is not complete".to_owned(),
            )),
        }
    }
}

/// Checks a receipt as typed: the form fields, the account and its person, the collector.
///
/// # Errors
/// `ServiceError::Db` only.
pub(super) fn plan_receipt(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &ReceiptInput,
) -> Result<PlannedReceipt, ServiceError> {
    let mut errors = FieldErrors::default();
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
    let party = resolve_party(
        conn,
        &mut errors,
        input.account_id,
        input.person_key.as_deref(),
        Accepts {
            buyer: true,
            seller: true,
            payable: false,
            allow_inactive: false,
            refusal: ErrorCode::ErrNotABuyer,
        },
    )?;
    check_collector(conn, &mut errors, input.collector_staff_id)?;
    Ok(PlannedReceipt {
        party,
        money,
        collector: input.collector_staff_id,
        errors: errors.finish().err().unwrap_or_default(),
    })
}

/// The journal entry of a receipt (PRD 15.2): cash or the bank بنام, the party جمع.
///
/// # Errors
/// `ServiceError::Data` if the plan has a mistake; the ledger errors of the entry.
pub(super) fn receipt_entry(planned: &PlannedReceipt) -> Result<BalancedEntry, ServiceError> {
    let (party, _, amount, pay_account) = planned.parts()?;
    let mut credit = LedgerLine::new(AccountId::new(party.record.id), Side::Cr, amount);
    credit.person_key.clone_from(&party.person_key);
    build_entry(vec![
        LedgerLine::new(AccountId::new(pay_account), Side::Dr, amount),
        credit,
    ])
    .map_err(ledger::ledger_error)
}

/// The voucher row of a receipt.
pub(super) fn receipt_row(
    planned: &PlannedReceipt,
    number: i64,
) -> Result<NewVoucher, ServiceError> {
    let (party, date, amount, _) = planned.parts()?;
    Ok(NewVoucher {
        voucher_type: VoucherType::Receipt,
        number,
        entry_date: date.to_iso(),
        account_id: Some(party.record.id),
        amount_paisa: amount.as_paisa(),
        mode: planned.money.mode,
        bank_account_id: planned.money.bank_account_id,
        cheque_no: planned.money.cheque_no.clone(),
        cheque_date: planned.money.cheque_date.map(BusinessDate::to_iso),
        expected_goods: None,
        narration: planned.money.narration.clone(),
        person_key: party.person_key.clone(),
        collector_staff_id: planned.collector,
        reason: None,
    })
}

/// What a seller's receipt does, worked out in memory with the same canonical working as the
/// save (Code Standards B5).
#[derive(Debug, Clone)]
pub(super) struct SellerReceipt {
    /// The seller without this receipt (and without the voucher being edited).
    pub before: SellerView,
    /// The seller with it.
    pub after: SellerView,
    /// The most he can return: his open advances plus what he owes beyond them. For an edited
    /// receipt it also counts what the old version already returned.
    pub limit: Paisa,
    /// What was payable to him with the old version of an edited receipt still in force: the
    /// edit may not make more payable than that (plan 16). `None` for a new receipt.
    pub old_open: Option<Paisa>,
}

impl SellerReceipt {
    /// True if the receipt would make more payable to the seller than before it, which is the
    /// refusal of PRD FR-15.12. An edited receipt is compared with its old version, so the same
    /// return, or a smaller one, stays allowed even when a later entry of its day used the
    /// advance (plan 16).
    pub fn exceeds(&self) -> bool {
        self.after.work.open > self.old_open.unwrap_or(self.before.work.open)
    }
}

/// Works out a seller's receipt. `replacing` is the entry of the voucher being edited, if it is
/// still in force; `place_like` is the entry whose place among the entries of its day the new
/// receipt takes (migration 0005): the same entry in a preview, and the entry already reversed in
/// the save of an edit.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data`, `ERR_OVERFLOW`.
pub(super) fn seller_receipt(
    conn: &Connection,
    party: &Party,
    date: BusinessDate,
    amount: Paisa,
    replacing: Option<i64>,
    place_like: Option<i64>,
) -> Result<SellerReceipt, ServiceError> {
    let inputs = matching::load_adv(conn, party.record.id)?;
    let day = date.to_iso();
    let key = party.person_key.as_deref();
    // An edited receipt keeps the place of the one it replaces among the entries of its day
    // (migration 0005), so the preview puts it where the save will.
    let add_order_key = place_like
        .map(|entry_id| crate::ledger::order_key_of(conn, entry_id))
        .transpose()?;
    let before = matching::seller_view(
        &inputs,
        key,
        &AdvChange {
            remove_entry: replacing,
            ..AdvChange::default()
        },
    )?;
    // An edited receipt: the books with its old version still in force, and the amount that
    // version holds. The new version may return at most what is free plus what the old one held.
    let (old_open, old_amount) = match replacing {
        Some(entry_id) => {
            let now = matching::seller_view(&inputs, key, &AdvChange::default())?;
            (Some(now.work.open), inputs.credit_of_entry(entry_id)?)
        }
        None => (None, Paisa::ZERO),
    };
    let base_open = old_open.unwrap_or(before.work.open);
    let after = matching::seller_view(
        &inputs,
        key,
        &AdvChange {
            remove_entry: replacing,
            add_receipt: Some((day.as_str(), amount)),
            add_payment: None,
            add_order_key,
        },
    )?;
    let mut limit = payables::return_limit(before.work.advances_open, before.work.excess_paid)
        .map_err(money_error)?
        .checked_add(old_amount)
        .map_err(money_error)?;
    // A receipt dated before later credits takes advances those credits had used, so the open
    // advances of today are not always the most he can return at that date. One look at the limit
    // itself says (the usual case: nothing is later, and it passes); if it exceeds, search.
    let exceeds_at = |amount: Paisa| -> Result<bool, ServiceError> {
        let view = matching::seller_view(
            &inputs,
            key,
            &AdvChange {
                remove_entry: replacing,
                add_receipt: Some((day.as_str(), amount)),
                add_payment: None,
                add_order_key,
            },
        )?;
        Ok(view.work.open > base_open)
    };
    if limit > Paisa::ZERO && exceeds_at(limit)? {
        limit = largest_allowed(exceeds_at, limit)?;
    }
    Ok(SellerReceipt {
        before,
        after,
        limit,
        old_open,
    })
}

/// The largest whole-rupee amount, below `limit`, that `exceeds` says is fine. `exceeds(limit)` is
/// true. Each rupee returned takes the oldest free advance rupee, which a later credit either had
/// used (the payable rises by 1) or not (no change), so once an amount exceeds, every larger
/// amount does too, and halving finds the edge.
fn largest_allowed(
    exceeds: impl Fn(Paisa) -> Result<bool, ServiceError>,
    limit: Paisa,
) -> Result<Paisa, ServiceError> {
    let overflow = || money_error(MoneyError::Overflow);
    let mut fine: i64 = 0;
    let mut too_much: i64 = limit.rupees();
    loop {
        let gap = too_much.checked_sub(fine).ok_or_else(overflow)?;
        if gap <= 1 {
            break;
        }
        let middle = fine.checked_add(gap / 2).ok_or_else(overflow)?;
        if exceeds(Paisa::from_rupees(middle).map_err(money_error)?)? {
            too_much = middle;
        } else {
            fine = middle;
        }
    }
    Paisa::from_rupees(fine).map_err(money_error)
}

/// The refusal of a seller's receipt that is more than he can return.
pub(super) fn exceeds_error(limit: Paisa) -> ServiceError {
    ServiceError::Rejected(vec![Rejection {
        code: ErrorCode::ErrReturnExceedsOpen,
        field: Some("amount".to_owned()),
        values: std::collections::BTreeMap::from([(
            "open_paisa".to_owned(),
            ErrorValue::Number(limit.as_paisa()),
        )]),
    }])
}

/// Refuses a seller's receipt that would make more payable to him than before it (PRD
/// FR-15.12). A receipt from anyone else passes. `replacing` is the entry of the voucher being
/// edited, when the books are not already without it.
///
/// # Errors
/// `ERR_RETURN_EXCEEDS_OPEN` (value `open_paisa`) on `amount`.
pub(super) fn check_seller_limit(
    conn: &Connection,
    planned: &PlannedReceipt,
    replacing: Option<i64>,
    place_like: Option<i64>,
) -> Result<(), ServiceError> {
    let (party, date, amount, _) = planned.parts()?;
    if party.role == Role::Seller {
        let check = seller_receipt(conn, party, date, amount, replacing, place_like)?;
        if check.exceeds() {
            return Err(exceeds_error(check.limit));
        }
    }
    Ok(())
}

/// Receives money (PRD 15.2): see the module text. One transaction: the voucher, its journal
/// entry, the matching rows and the audit row.
///
/// # Errors
/// The field errors of the date, amount, bank, cheque and collector fields; `ERR_NOT_FOUND`,
/// `ERR_NOT_A_BUYER`, `ERR_ACCOUNT_INACTIVE` on `account_id`; `ERR_PERSON_REQUIRED` and
/// `ERR_NOT_FOUND` on `person_key`; `ERR_RETURN_EXCEEDS_OPEN` (value `open_paisa`) on `amount`
/// for a seller; `ERR_DATE_CLOSED` and `ERR_DATE_IN_FUTURE` on `date`. Nothing is written when
/// any of them is returned.
pub fn save_receipt(
    ctx: &Context,
    actor: &Actor,
    input: &ReceiptInput,
) -> Result<VoucherSaved, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.write(|tx| {
        let planned = plan_receipt(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(planned.errors.into());
        }
        post_receipt(ctx, tx, actor, &planned)
    })
}

/// Checks the limit of a seller's receipt, then writes a planned, error-free receipt inside the
/// caller's transaction.
fn post_receipt(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    planned: &PlannedReceipt,
) -> Result<VoucherSaved, ServiceError> {
    let (party, date, _, _) = planned.parts()?;
    check_seller_limit(tx, planned, None, None)?;
    let entry = receipt_entry(planned)?;
    let hint = hint_for(tx, party.role, party.record.id, party.person_key.as_deref())?;
    let now = ctx.now_text();
    let number = repo::next_number(tx, VoucherType::Receipt)?;
    let row = receipt_row(planned, number)?;
    let (voucher_id, posted) = insert_and_post(
        ctx,
        tx,
        actor.user_id,
        PostingSpec {
            posting_type: PostingType::Receipt,
            date,
            narration: row.narration.clone(),
        },
        &row,
        entry,
    )?;
    receipt_match(tx, actor.user_id, &now, planned, posted.entry_id, &hint)?;
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), "create", "voucher")
            .entity_id(voucher_id)
            .new_value(voucher_json(number, "receipt", &row))
            .into_entry(),
    )?;
    Ok(VoucherSaved {
        voucher_id,
        voucher_type: VoucherType::Receipt,
        number,
        entry_id: posted.entry_id,
        new_balance: ledger::balance_in(tx, party.record.id, party.person_key.as_deref(), None)?,
    })
}

/// Matches a posted receipt (PRD FR-15.4, FR-15.5, FR-15.12). This is the fast path: it assumes
/// the rows of the party are canonical, which they are after a save and after the re-match that
/// follows a reversal.
///
/// # Errors
/// As [`matching::match_new_credits`] and [`matching::adjust_advances`].
pub(super) fn receipt_match(
    tx: &Connection,
    user_id: i64,
    now: &str,
    planned: &PlannedReceipt,
    entry_id: i64,
    hint: &Hint,
) -> Result<(), ServiceError> {
    let (party, date, amount, _) = planned.parts()?;
    let credit_line = stored_line(tx, entry_id, party.record.id, true)?;
    let day = date.to_iso();
    match party.role {
        // PRD FR-15.4, FR-15.5: the oldest open bills first; the rest waits for the next bill.
        Role::Buyer => {
            matching::match_new_credits_hinted(
                tx,
                user_id,
                now,
                party.record.id,
                party.person_key.as_deref(),
                &day,
                &[NewLine {
                    line_id: credit_line,
                    amount,
                }],
                hint.open_bills,
            )?;
        }
        // PRD FR-15.12: the advances before the credit, oldest first. A one-time seller never
        // has an advance (PRD FR-15.4).
        Role::Seller if !party.is_pukhta => {
            matching::adjust_advances(
                tx,
                user_id,
                now,
                party.record.id,
                credit_line,
                amount,
                &day,
            )?;
        }
        Role::Seller | Role::Payable => {}
    }
    Ok(())
}

/// True for a row of the collection round that is left out: no amount, or 0 (PRD FR-7.61).
fn is_blank(row: &ReceiptInput) -> bool {
    row.amount.trim().is_empty()
        || validate::money("amount", &row.amount).is_ok_and(|a| a == Paisa::ZERO)
}

/// Saves the receipts of the collection round (PRD FR-7.61, FR-15.1): each is its own posting
/// event in its own transaction, a row with amount 0 or empty is skipped, and a row that fails
/// does not stop the others. The answer has one result per row, in the order sent.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `rows` for more than [`MAX_LIST_LINES`] rows. A mistake on a row is in
/// that row's result, not an error of the call.
pub fn save_receipts_batch(
    ctx: &Context,
    actor: &Actor,
    input: &ReceiptsInput,
) -> Result<ReceiptsSaved, ServiceError> {
    if input.rows.len() > MAX_LIST_LINES {
        return Err(ServiceError::reject_field(ErrorCode::ErrOutOfRange, "rows"));
    }
    let mut rows = Vec::with_capacity(input.rows.len());
    let mut saved_total = Paisa::ZERO;
    for (index, row) in input.rows.iter().enumerate() {
        if is_blank(row) {
            rows.push(ReceiptRowResult {
                saved: None,
                skipped: true,
                errors: Vec::new(),
            });
            continue;
        }
        let at = |field: Option<String>| match field {
            Some(field) => format!("rows[{index}].{field}"),
            None => format!("rows[{index}]"),
        };
        match save_receipt(ctx, actor, row) {
            Ok(saved) => {
                // The total is the sum of what was written, never of what was typed.
                let amount = ctx.db.read(|conn| {
                    conn.query_row(
                        "SELECT amount_paisa FROM vouchers WHERE id = ?1",
                        [saved.voucher_id],
                        |r| r.get::<_, i64>(0),
                    )
                    .map_err(ServiceError::from)
                })?;
                saved_total = saved_total
                    .checked_add(Paisa::try_from_paisa(amount).map_err(money_error)?)
                    .map_err(money_error)?;
                rows.push(ReceiptRowResult {
                    saved: Some(saved),
                    skipped: false,
                    errors: Vec::new(),
                });
            }
            Err(ServiceError::Rejected(items)) => {
                rows.push(ReceiptRowResult {
                    saved: None,
                    skipped: false,
                    errors: items
                        .into_iter()
                        .map(|item| LineError {
                            code: item.code,
                            field: at(item.field),
                            values: item.values,
                        })
                        .collect(),
                });
            }
            Err(other) => {
                // A failure that is not the user's mistake (the disk, the database). The rows
                // before this one are already saved, so their results must not be lost.
                tracing::error!(index, error = %other, "a receipt of the round was not saved");
                rows.push(ReceiptRowResult {
                    saved: None,
                    skipped: false,
                    errors: vec![LineError {
                        code: other.code(),
                        field: format!("rows[{index}]"),
                        values: std::collections::BTreeMap::new(),
                    }],
                });
            }
        }
    }
    Ok(ReceiptsSaved {
        rows,
        saved_paisa: saved_total,
    })
}
