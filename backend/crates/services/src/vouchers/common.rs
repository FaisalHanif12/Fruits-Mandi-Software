//! What the three voucher types share (plan 15, PRD 15.1): the date, the amount, the way the
//! money moves (cash, bank, cheque), the note, the party and its person key, and the cash check.
//!
//! Every function here is a check or a read. The writes are in `receipt`, `payment` and
//! `advance`, each one transaction (Code Standards B3). The preview and the save call the same
//! checks (Code Standards B5).

use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::{self as repo, NewVoucher};
use mandi_domain::cash;
use mandi_domain::core::accounts::{BUYER_CONTROL, Balance, RecordRef, SELLER_CONTROL};
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::vouchers::VoucherMode;
use mandi_domain::ledger::{BalancedEntry, PostingType, SourceType};
use serde_json::{Value, json};

use crate::cash::{CASH_ACCOUNT_CODE, cash_after_entry};
use crate::context::Context;
use crate::error::{ServiceError, money_error};
use crate::ledger::{self, Posted, PostingEvent};
use crate::matching;

/// The control code of the payable accounts: labour, freight, store rent, munshiana (PRD
/// FR-10.4, FR-14.2).
pub(super) const PAYABLE_CONTROL: &str = "0201";
/// The cash buyer with no name. It holds no balance, so no voucher goes to it (PRD FR-10.4).
const NAQDI_MANDI_CODE: &str = "03040000";
/// The firm as the seller of its own goods. It never holds a balance (PRD FR-12.26, plan 16b).
const OWN_GOODS_CODE: &str = "030119998";

/// What the buyer's matching rows add up to just before a voucher is posted, so the fast paths can
/// find the open bills (or the waiting credits) by a walk back from the newest entry instead of
/// reading the whole history of the buyer. The rows are canonical before the posting; after it
/// they are not yet, so the totals must be read first.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Hint {
    /// Σ open on his bills.
    pub open_bills: Option<i64>,
    /// Σ of his credit that no bill has used.
    pub unmatched_credit: Option<i64>,
}

/// The hint for a buyer key (none for any other party).
///
/// # Errors
/// `ServiceError::Db`.
pub(super) fn hint_for(
    conn: &Connection,
    role: Role,
    account_id: i64,
    person_key: Option<&str>,
) -> Result<Hint, ServiceError> {
    if role != Role::Buyer {
        return Ok(Hint::default());
    }
    let (open_bills, unmatched_credit) =
        mandi_db::matching::expected_pair(conn, account_id, person_key)?;
    Ok(Hint {
        open_bills: Some(open_bills),
        unmatched_credit: Some(unmatched_credit),
    })
}

/// The ledger account of cash in hand.
pub(super) fn cash_account(conn: &Connection) -> Result<i64, ServiceError> {
    account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ServiceError::Data("the account روکڑ is missing".to_owned()))
}

/// A free-text field that may be empty: trimmed, `None` when empty.
pub(super) fn optional_text(errors: &mut FieldErrors, field: &str, text: &str) -> Option<String> {
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// An amount as a balance with its side, from a signed net (debit positive).
pub(super) fn balance_of_net(net: Paisa) -> Result<Balance, ServiceError> {
    Balance::from_net(net).map_err(money_error)
}

/// The fields of a voucher that every type has, as typed.
#[derive(Debug, Clone, Copy)]
pub(super) struct MoneyText<'a> {
    pub date: &'a str,
    pub amount: &'a str,
    pub mode: VoucherMode,
    pub bank_account_id: Option<i64>,
    pub cheque_no: &'a str,
    pub cheque_date: &'a str,
    pub narration: &'a str,
}

/// The same fields once checked. A field with a mistake is `None` and its error is in the list.
#[derive(Debug, Clone)]
pub(super) struct Money {
    pub date: Option<BusinessDate>,
    /// Whole rupees, more than 0.
    pub amount: Option<Paisa>,
    pub mode: VoucherMode,
    /// The ledger account the money moves through: cash, or the bank's account.
    pub pay_account: Option<i64>,
    pub bank_account_id: Option<i64>,
    pub cheque_no: Option<String>,
    pub cheque_date: Option<BusinessDate>,
    pub narration: Option<String>,
}

/// Checks the common fields (PRD 15.1): an open date, an amount above 0, the bank and the cheque.
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are pushed to `errors`.
pub(super) fn plan_money(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    errors: &mut FieldErrors,
    text: &MoneyText<'_>,
) -> Result<Money, ServiceError> {
    let (closed, today) = window;
    let date = errors.take(validate::posting_date("date", text.date, closed, today));
    let amount = errors
        .take(validate::money("amount", text.amount))
        .and_then(|a| {
            if a == Paisa::ZERO {
                errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, "amount"));
                None
            } else {
                Some(a)
            }
        });
    let narration = optional_text(errors, "narration", text.narration);

    // Where the money moves (PRD 15.1): cash, or a bank account (a cheque posts to the bank at
    // once, A30).
    let mut pay_account = None;
    let mut bank_account_id = None;
    let mut cheque_no = None;
    let mut cheque_date = None;
    match text.mode {
        VoucherMode::Cash => {
            if text.bank_account_id.is_some() {
                errors.push(FieldError::new(
                    ErrorCode::ErrFormatInvalid,
                    "bank_account_id",
                ));
            }
            pay_account = Some(cash_account(conn)?);
        }
        VoucherMode::Bank | VoucherMode::Cheque => {
            match text.bank_account_id {
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
            if text.mode == VoucherMode::Cheque {
                // PRD 15.1: a cheque needs its number and its date.
                cheque_no = errors
                    .take(validate::free_text("cheque_no", text.cheque_no))
                    .and_then(|n| {
                        if n.is_empty() {
                            errors.push(FieldError::new(ErrorCode::ErrChequeRequired, "cheque_no"));
                            None
                        } else {
                            Some(n)
                        }
                    });
                if validate::free_text("cheque_date", text.cheque_date)
                    .map(|t| t.is_empty())
                    .unwrap_or(false)
                {
                    errors.push(FieldError::new(ErrorCode::ErrChequeRequired, "cheque_date"));
                } else {
                    cheque_date =
                        errors.take(validate::date("cheque_date", text.cheque_date, today));
                }
            }
        }
    }
    Ok(Money {
        date,
        amount,
        mode: text.mode,
        pay_account,
        bank_account_id,
        cheque_no,
        cheque_date,
        narration,
    })
}

/// What a voucher does with its party.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    /// A buyer (control 0304), not the cash buyer.
    Buyer,
    /// A seller (control 03011), not the firm's own goods.
    Seller,
    /// A payable account such as the transporters (control 0201).
    Payable,
}

/// The account of a voucher after the checks.
#[derive(Debug, Clone)]
pub(super) struct Party {
    pub record: RecordRef,
    pub role: Role,
    /// True for the two پختہ کھاتہ accounts.
    pub is_pukhta: bool,
    /// The one-time person, for a پختہ کھاتہ account.
    pub person_key: Option<String>,
}

/// Which roles a voucher type accepts, and the code it uses for an account of any other kind.
#[derive(Debug, Clone, Copy)]
pub(super) struct Accepts {
    pub buyer: bool,
    pub seller: bool,
    pub payable: bool,
    /// True for a read: the bills of a buyer who was made inactive are still worth showing.
    /// A new posting never takes an inactive account (PRD FR-10.2).
    pub allow_inactive: bool,
    /// `ERR_NOT_A_BUYER` for a receipt, `ERR_FORMAT_INVALID` for a payment.
    pub refusal: ErrorCode,
}

/// Finds the account of a receipt or a payment and checks it: it exists, it is active, its kind
/// is one the voucher type takes, and a پختہ کھاتہ account names its person who is on the books
/// (PRD FR-13.5, FR-13.7). A mistake goes to `errors` at `account_id` or `person_key`.
///
/// # Errors
/// `ServiceError::Db` only.
pub(super) fn resolve_party(
    conn: &Connection,
    errors: &mut FieldErrors,
    account_id: i64,
    person_key: Option<&str>,
    accepts: Accepts,
) -> Result<Option<Party>, ServiceError> {
    let Some(facts) = account_repo::facts(conn, account_id)? else {
        errors.push(FieldError::new(ErrorCode::ErrNotFound, "account_id"));
        return Ok(None);
    };
    let record = account_repo::get(conn, facts.id)?.map(|row| RecordRef {
        id: row.id,
        code: row.code,
        name_ur: row.name_ur,
        name_en: row.name_en,
    });
    let Some(record) = record else {
        errors.push(FieldError::new(ErrorCode::ErrNotFound, "account_id"));
        return Ok(None);
    };
    let role = match facts.control_code.as_str() {
        _ if facts.code == NAQDI_MANDI_CODE || facts.code == OWN_GOODS_CODE => None,
        BUYER_CONTROL => Some(Role::Buyer),
        SELLER_CONTROL => Some(Role::Seller),
        PAYABLE_CONTROL => Some(Role::Payable),
        _ => None,
    };
    let allowed = match role {
        Some(Role::Buyer) => accepts.buyer,
        Some(Role::Seller) => accepts.seller,
        Some(Role::Payable) => accepts.payable,
        None => false,
    };
    let Some(role) = role.filter(|_| allowed) else {
        errors.push(FieldError::new(accepts.refusal, "account_id"));
        return Ok(None);
    };
    if !facts.is_active && !accepts.allow_inactive {
        errors.push(FieldError::new(ErrorCode::ErrAccountInactive, "account_id"));
        return Ok(None);
    }
    // The person: needed for a پختہ کھاتہ account, refused for any other (PRD FR-13.5).
    let key = person_key.map(str::trim).filter(|k| !k.is_empty());
    let person_key = match (facts.is_pukhta_khata, key) {
        (true, None) => {
            errors.push(FieldError::new(ErrorCode::ErrPersonRequired, "person_key"));
            return Ok(None);
        }
        (true, Some(key)) => {
            if !repo::person_has_lines(conn, facts.id, key)? {
                errors.push(FieldError::new(ErrorCode::ErrNotFound, "person_key"));
                return Ok(None);
            }
            Some(key.to_owned())
        }
        (false, Some(_)) => {
            errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, "person_key"));
            return Ok(None);
        }
        (false, None) => None,
    };
    Ok(Some(Party {
        record,
        role,
        is_pukhta: facts.is_pukhta_khata,
        person_key,
    }))
}

/// What an entry does to cash in hand: Σ debit − Σ credit of its lines on the cash account.
pub(super) fn cash_change(conn: &Connection, entry: &BalancedEntry) -> Result<Paisa, ServiceError> {
    let cash = AccountId::new(cash_account(conn)?);
    cash::net_change(entry, cash).map_err(money_error)
}

/// The cash check of an entry (or of an edit: the sum of the changes of the reversal and the
/// new entry), PRD FR-6.7.
pub(super) fn cash_check(conn: &Connection, change: Paisa) -> Result<CashCheck, ServiceError> {
    cash_after_entry(conn, change)
}

/// The id of the stored line of a posted entry on an account, on one side, in line order. The
/// matching rows point at stored lines.
///
/// # Errors
/// `ServiceError::Data` if the posted entry cannot be read back or has no such line.
pub(super) fn stored_line(
    conn: &Connection,
    entry_id: i64,
    account_id: i64,
    credit: bool,
) -> Result<i64, ServiceError> {
    let stored = mandi_db::ledger::get_entry(conn, entry_id)?.ok_or_else(|| {
        ServiceError::Data("the posted journal entry cannot be read back".to_owned())
    })?;
    stored
        .lines
        .iter()
        .find(|l| {
            l.account_id == account_id
                && if credit {
                    l.cr_paisa > 0
                } else {
                    l.dr_paisa > 0
                }
        })
        .map(|l| l.id)
        .ok_or_else(|| {
            ServiceError::Data(format!(
                "entry {entry_id} has no line on account {account_id}"
            ))
        })
}

/// A voucher as the audit log keeps it (Code Standards S7).
pub(super) fn voucher_json(number: i64, voucher_type: &str, v: &NewVoucher) -> Value {
    json!({
        "type": voucher_type,
        "number": number,
        "entry_date": v.entry_date,
        "account_id": v.account_id,
        "person_key": v.person_key,
        "amount_paisa": v.amount_paisa,
        "mode": v.mode.as_str(),
        "bank_account_id": v.bank_account_id,
        "cheque_no": v.cheque_no,
        "cheque_date": v.cheque_date,
        "collector_staff_id": v.collector_staff_id,
        "reason": v.reason,
        "expected_goods": v.expected_goods,
        "narration": v.narration,
    })
}

/// Checks that a collector (a staff member) exists and is active.
///
/// # Errors
/// `ServiceError::Db` only.
pub(super) fn check_collector(
    conn: &Connection,
    errors: &mut FieldErrors,
    collector: Option<i64>,
) -> Result<(), ServiceError> {
    if let Some(id) = collector {
        match repo::staff_active(conn, id)? {
            None => errors.push(FieldError::new(
                ErrorCode::ErrNotFound,
                "collector_staff_id",
            )),
            Some(false) => errors.push(FieldError::new(
                ErrorCode::ErrAccountInactive,
                "collector_staff_id",
            )),
            Some(true) => {}
        }
    }
    Ok(())
}

/// How a voucher's entry is posted: its type, its day and its note.
#[derive(Debug, Clone)]
pub(super) struct PostingSpec {
    pub posting_type: PostingType,
    pub date: BusinessDate,
    pub narration: Option<String>,
}

/// Posts the entry of a voucher, inside the caller's transaction. `order_key` is the key of the
/// entry an edit replaces, so the voucher keeps its place among the entries of its day for
/// matching (migration 0005); `None` for a new voucher.
///
/// # Errors
/// The errors of [`ledger::post`] (a closed date, an inactive account) and `ServiceError::Db`.
pub(super) fn post_entry(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    voucher_id: i64,
    spec: PostingSpec,
    entry: BalancedEntry,
    order_key: Option<i64>,
) -> Result<Posted, ServiceError> {
    ledger::post_ordered(
        ctx,
        tx,
        user_id,
        &PostingEvent {
            date: spec.date,
            posting_type: spec.posting_type,
            source_type: SourceType::Voucher,
            source_id: voucher_id,
            narration: spec.narration,
            reverses: None,
            entry,
        },
        order_key,
    )
}

/// Writes the voucher row and posts its entry, inside the caller's transaction. The number is
/// given here and never used again (PRD 15.1).
///
/// # Errors
/// The errors of [`ledger::post`] (a closed date, an inactive account) and `ServiceError::Db`.
pub(super) fn insert_and_post(
    ctx: &Context,
    tx: &Connection,
    actor_id: i64,
    spec: PostingSpec,
    voucher: &NewVoucher,
    entry: BalancedEntry,
) -> Result<(i64, Posted), ServiceError> {
    let now = ctx.now_text();
    let voucher_id = repo::insert(tx, voucher, actor_id, &now)?;
    let posted = post_entry(ctx, tx, actor_id, voucher_id, spec, entry, None)?;
    repo::set_current_entry(tx, voucher_id, posted.entry_id, actor_id, &now)?;
    Ok((voucher_id, posted))
}

/// What a stored entry did to cash in hand: Σ debit − Σ credit of its lines on the cash account.
///
/// # Errors
/// `ServiceError::Db`; `ERR_OVERFLOW`.
pub(crate) fn stored_cash_change(
    conn: &Connection,
    entry: &mandi_db::ledger::StoredEntry,
) -> Result<Paisa, ServiceError> {
    let cash = cash_account(conn)?;
    let mut change = Paisa::ZERO;
    for line in entry.lines.iter().filter(|l| l.account_id == cash) {
        let dr = Paisa::try_from_paisa(line.dr_paisa).map_err(money_error)?;
        let cr = Paisa::try_from_paisa(line.cr_paisa).map_err(money_error)?;
        change = change
            .checked_add(dr)
            .and_then(|v| v.checked_sub(cr))
            .map_err(money_error)?;
    }
    Ok(change)
}

/// Works the matching of one party out again after a reversal (Architecture 12, money agent
/// 8.3): the canonical working of a buyer's bills, or of a seller's advances. The fast paths of
/// `matching` assume canonical rows, which a reversal leaves broken, so only the full workings
/// are used here.
///
/// # Errors
/// As [`matching::rematch`] and [`matching::rematch_advances`].
pub(crate) fn rematch_party(
    tx: &Connection,
    user_id: i64,
    now: &str,
    account_id: i64,
    person_key: Option<&str>,
    role: Role,
    is_pukhta: bool,
) -> Result<(), ServiceError> {
    match role {
        Role::Buyer => {
            matching::rematch(tx, user_id, now, account_id, person_key)?;
        }
        // A one-time seller never has an advance (PRD FR-15.4), and a payable account has no
        // matching at all.
        Role::Seller if !is_pukhta => {
            matching::rematch_advances(tx, user_id, now, account_id)?;
        }
        Role::Seller | Role::Payable => {}
    }
    Ok(())
}

/// The role of an account for matching, from its facts.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn role_of_account(
    conn: &Connection,
    account_id: i64,
) -> Result<Option<(Role, bool)>, ServiceError> {
    let Some(facts) = account_repo::facts(conn, account_id)? else {
        return Ok(None);
    };
    let role = match facts.control_code.as_str() {
        BUYER_CONTROL => Some(Role::Buyer),
        SELLER_CONTROL => Some(Role::Seller),
        PAYABLE_CONTROL => Some(Role::Payable),
        _ => None,
    };
    Ok(role.map(|r| (r, facts.is_pukhta_khata)))
}
