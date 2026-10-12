//! Read-only help for the entry screens (Tech Stack 4.7, PRD 7.6): the party card now; the
//! memories and hints of plans 12 and 13 later. Nothing here writes (Code Standards B29).

use mandi_db::accounts as account_repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::vouchers as voucher_repo;
use mandi_domain::core::accounts::{
    AccountRow, AccountType, Balance, BalanceSide, CardWarning, CardWarningKind, PartyCard,
    SELLER_CONTROL,
};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::vouchers::{AdvancesInfo, OpenAdvance};

use crate::context::Context;
use crate::error::{ServiceError, money_error};
use crate::ledger;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// What a debit balance owes, or 0 for a credit or zero balance.
fn owed(balance: Balance) -> Paisa {
    if balance.side == BalanceSide::Dr {
        balance.amount_paisa
    } else {
        Paisa::ZERO
    }
}

/// A seller's open advance: what he was given as advances less what was adjusted (PRD FR-15.8).
/// A negative result means more was adjusted than given, which is a bug in the matching, so it
/// is reported as an internal error rather than hidden.
fn open_advance(
    conn: &mandi_db::rusqlite::Connection,
    account_id: i64,
) -> Result<Paisa, ServiceError> {
    let given = paisa(ledger_repo::advance_given(conn, account_id)?)?;
    let adjusted = paisa(ledger_repo::advance_adjusted(conn, account_id)?)?;
    let open = given.checked_sub(adjusted).map_err(money_error)?;
    if open.is_negative() {
        tracing::error!(
            account_id,
            given = given.as_paisa(),
            adjusted = adjusted.as_paisa(),
            "more was adjusted against advances than was given"
        );
        return Err(ServiceError::Data(format!(
            "account {account_id}: more adjusted than given as advance"
        )));
    }
    Ok(open)
}

/// The warnings of a card, each with a kind the screen turns into a word and a colour.
fn warnings(
    row: &AccountRow,
    credit_used: Option<Paisa>,
    advance_open: Option<Paisa>,
    balance: Balance,
) -> Result<Vec<CardWarning>, ServiceError> {
    let mut out = Vec::new();
    if !row.is_active {
        out.push(CardWarning {
            kind: CardWarningKind::Inactive,
            amount_paisa: None,
        });
    }
    if let (Some(limit), Some(used)) = (row.credit_limit_paisa, credit_used)
        && used > limit
    {
        out.push(CardWarning {
            kind: CardWarningKind::OverCreditLimit,
            amount_paisa: Some(used.checked_sub(limit).map_err(money_error)?),
        });
    }
    // A seller who owes the firm more than his advances cover (PRD FR-15.9): his debit balance
    // less his open advance.
    if advance_open.is_some() {
        let beyond = owed(balance)
            .checked_sub(advance_open.unwrap_or(Paisa::ZERO))
            .map_err(money_error)?;
        if beyond > Paisa::ZERO {
            out.push(CardWarning {
                kind: CardWarningKind::OwesFirm,
                amount_paisa: Some(beyond),
            });
        }
    }
    Ok(out)
}

/// The card shown next to an account field (PRD FR-7.38): names, father or firm, city, phone,
/// the balance and its side, the date of the last entry, a buyer's credit limit and how much of
/// it is used, a seller's open advance, and warnings. Balances come from the posted journal,
/// never from the search index (Code Standards B27). Only reads.
///
/// # Errors
/// `ERR_NOT_FOUND` on `account_id`; `ServiceError::Db`.
pub fn party_card(ctx: &Context, account_id: i64) -> Result<PartyCard, ServiceError> {
    ctx.db.read(|conn| {
        let row = account_repo::get(conn, account_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "account_id"))?;
        let balance = ledger::balance_in(conn, account_id, None, None)?;
        let last_entry_date = ledger_repo::last_entry_date(conn, account_id)?;
        // The پختہ کھاتہ accounts hold many people, each with his own balance (PRD FR-13.5): the
        // card shows the account's balance, but no credit or advance figure that would net one
        // person against another. A per-person card comes with the person picker.
        let is_buyer = row.account_type == AccountType::Buyer && !row.is_pukhta_khata;
        let is_seller = row.account_type == AccountType::Seller && !row.is_pukhta_khata;
        let credit_used = is_buyer.then(|| owed(balance));
        let advance_open = if is_seller {
            Some(open_advance(conn, account_id)?)
        } else {
            None
        };
        let warnings = warnings(&row, credit_used, advance_open, balance)?;
        let (city_ur, city_en) = row
            .city
            .as_ref()
            .map(|c| (c.name_ur.clone(), c.name_en.clone()))
            .unwrap_or_default();
        Ok(PartyCard {
            id: row.id,
            code: row.code.clone(),
            account_type: row.account_type,
            name_ur: row.name_ur.clone(),
            name_en: row.name_en.clone(),
            father_or_firm_ur: row.father_or_firm_ur.clone(),
            father_or_firm_en: row.father_or_firm_en.clone(),
            city_ur,
            city_en,
            phone: row.phone1.clone(),
            is_active: row.is_active,
            balance,
            last_entry_date,
            credit_limit: if is_buyer {
                row.credit_limit_paisa
            } else {
                None
            },
            credit_used,
            advance_open,
            warnings,
        })
    })
}

/// A seller's open advances and his last پیشگی voucher, for the card in the پیشگی screen (PRD
/// FR-7.60). Each open advance is a posted advance line (a پیشگی voucher or an opening debit
/// marked as an advance) with what has been adjusted against it; only lines with something left
/// are listed, oldest first. The total equals the open advance on the party card. `last` is the
/// last saved پیشگی voucher: an opening advance has no voucher number, so it is never "last".
/// Only reads.
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_NOT_A_SELLER` on `seller_id`; `ServiceError::Data` if more was
/// adjusted against an advance than was given.
pub fn advances(ctx: &Context, seller_id: i64) -> Result<AdvancesInfo, ServiceError> {
    ctx.db.read(|conn| {
        let facts = account_repo::facts(conn, seller_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "seller_id"))?;
        if facts.control_code != SELLER_CONTROL || facts.is_pukhta_khata {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNotASeller,
                "seller_id",
            ));
        }
        let mut open = Vec::new();
        let mut total = Paisa::ZERO;
        for line in voucher_repo::advance_lines(conn, seller_id)? {
            let given = paisa(line.given_paisa)?;
            let adjusted = paisa(line.adjusted_paisa)?;
            let left = given.checked_sub(adjusted).map_err(money_error)?;
            if left.is_negative() {
                tracing::error!(
                    seller_id,
                    "more was adjusted against an advance than was given"
                );
                return Err(ServiceError::Data(format!(
                    "account {seller_id}: more adjusted than given as advance"
                )));
            }
            if left == Paisa::ZERO {
                continue;
            }
            total = total.checked_add(left).map_err(money_error)?;
            open.push(OpenAdvance {
                // Only a پیشگی voucher or an opening debit makes an advance line.
                source: if line.voucher_number.is_some() {
                    "voucher".to_owned()
                } else {
                    "opening".to_owned()
                },
                voucher_number: line.voucher_number,
                entry_date: line.entry_date,
                given_paisa: given,
                adjusted_paisa: adjusted,
                open_paisa: left,
            });
        }
        Ok(AdvancesInfo {
            open,
            open_paisa: total,
            last: voucher_repo::last_advance(conn, seller_id)?,
        })
    })
}
