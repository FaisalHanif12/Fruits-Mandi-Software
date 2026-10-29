//! The chart of accounts (PRD FR-10.1, plan 07): heads, controls and accounts with live balances.
//!
//! Only reads. Balances come from the posted journal, never from a stored total (PRD FR-6.1).
//! A head or control balance is the debit total minus the credit total of everything under it,
//! worked out with checked maths.

use std::collections::HashMap;

use mandi_db::accounts as repo;
use mandi_db::ledger::Totals;
use mandi_domain::core::accounts::{
    AccountNode, Balance, CoaControl, CoaControlAccounts, CoaControlAccountsInput, CoaHead,
    CoaTree, ControlNode, HeadNode,
};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::{MoneyError, Paisa};

use crate::context::Context;
use crate::error::{ServiceError, money_error};
use crate::ledger;

/// Running debit and credit totals in paisa, added with checked maths.
#[derive(Clone, Copy)]
struct Sum {
    dr: Paisa,
    cr: Paisa,
}

impl Sum {
    const ZERO: Sum = Sum {
        dr: Paisa::ZERO,
        cr: Paisa::ZERO,
    };

    fn of(totals: Totals) -> Result<Sum, MoneyError> {
        Ok(Sum {
            dr: Paisa::try_from_paisa(totals.dr_paisa)?,
            cr: Paisa::try_from_paisa(totals.cr_paisa)?,
        })
    }

    fn add(self, other: Sum) -> Result<Sum, MoneyError> {
        Ok(Sum {
            dr: self.dr.checked_add(other.dr)?,
            cr: self.cr.checked_add(other.cr)?,
        })
    }

    fn balance(self) -> Result<Balance, MoneyError> {
        Balance::from_totals(self.dr, self.cr)
    }
}

/// Heads with their controls, each with its balance and its number of accounts (plan 38 B1). The
/// accounts themselves load a page at a time with [`control_accounts`] when a control is opened,
/// so a firm with 20,000 accounts still opens the chart at once. Balances come from one pass over
/// the journal.
///
/// # Errors
/// `ServiceError::Db`; `ERR_OVERFLOW` if a total would not fit.
pub fn tree(ctx: &Context) -> Result<CoaTree, ServiceError> {
    ctx.db.read(|conn| {
        let heads = repo::heads(conn)?;
        let controls = repo::controls(conn)?;
        let control_of = repo::account_control_ids(conn)?;
        let totals: HashMap<i64, Totals> = ledger::balances_by_account(conn, None)?;
        build(&heads, &controls, &control_of, &totals).map_err(money_error)
    })
}

/// Builds the head and control levels. The controls arrive in code order and keep it.
fn build(
    heads: &[CoaHead],
    controls: &[CoaControl],
    control_of: &[(i64, i64)],
    totals: &HashMap<i64, Totals>,
) -> Result<CoaTree, MoneyError> {
    // One pass over the accounts: how many each control has and what they add up to.
    let mut by_control: HashMap<i64, (i64, Sum)> = HashMap::new();
    for (account_id, control_id) in control_of {
        let sum = match totals.get(account_id) {
            Some(found) => Sum::of(*found)?,
            None => Sum::ZERO,
        };
        let slot = by_control.entry(*control_id).or_insert((0, Sum::ZERO));
        slot.0 = slot.0.saturating_add(1);
        slot.1 = slot.1.add(sum)?;
    }
    let mut head_nodes = Vec::with_capacity(heads.len());
    for head in heads {
        let mut head_sum = Sum::ZERO;
        let mut control_nodes = Vec::new();
        for control in controls.iter().filter(|c| c.head_id == head.id) {
            let (account_count, control_sum) = by_control
                .get(&control.id)
                .copied()
                .unwrap_or((0, Sum::ZERO));
            head_sum = head_sum.add(control_sum)?;
            control_nodes.push(ControlNode {
                id: control.id,
                code: control.code.clone(),
                name_ur: control.name_ur.clone(),
                name_en: control.name_en.clone(),
                party_type: control.party_type.clone(),
                balance: control_sum.balance()?,
                account_count,
            });
        }
        head_nodes.push(HeadNode {
            id: head.id,
            code: head.code.clone(),
            name_ur: head.name_ur.clone(),
            name_en: head.name_en.clone(),
            nature: head.nature.clone(),
            balance: head_sum.balance()?,
            controls: control_nodes,
        });
    }
    Ok(CoaTree { heads: head_nodes })
}

/// One page of the accounts of one control, by code, each with its balance now, and the control's
/// own balance over all its accounts (plan 38 B1, Code Standards B31). A page of 25, 50 or 100
/// (0 gives 100); an offset past the end gives no rows and the same count and balance.
///
/// # Errors
/// `ERR_NOT_FOUND` on `control_id`; `ERR_OUT_OF_RANGE` on `limit` above 500; `ServiceError::Db`.
pub fn control_accounts(
    ctx: &Context,
    input: &CoaControlAccountsInput,
) -> Result<CoaControlAccounts, ServiceError> {
    if input.limit > MAX_PAGE {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrOutOfRange,
            "limit",
        ));
    }
    let limit = if input.limit == 0 {
        crate::page::DEFAULT_PAGE
    } else {
        input.limit
    };
    ctx.db.read(|conn| {
        if !repo::controls(conn)?
            .iter()
            .any(|c| c.id == input.control_id)
        {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNotFound,
                "control_id",
            ));
        }
        let count = repo::control_account_count(conn, input.control_id)?;
        let page = repo::control_accounts_page(
            conn,
            input.control_id,
            i64::from(input.offset),
            i64::from(limit),
        )?;
        let ids: Vec<i64> = page.iter().map(|a| a.id).collect();
        let totals = ledger::balances_for_accounts(conn, &ids)?;
        let mut rows = Vec::with_capacity(page.len());
        for account in &page {
            let sum = match totals.get(&account.id) {
                Some(found) => Sum::of(*found).map_err(money_error)?,
                None => Sum::ZERO,
            };
            rows.push(AccountNode {
                id: account.id,
                code: account.code.clone(),
                name_ur: account.name_ur.clone(),
                name_en: account.name_en.clone(),
                balance: sum.balance().map_err(money_error)?,
                is_active: account.is_active,
                is_system: account.is_system,
                tag: account.tag.clone(),
            });
        }
        let whole = Sum::of(mandi_db::ledger::totals_for_control(
            conn,
            input.control_id,
        )?)
        .map_err(money_error)?;
        Ok(CoaControlAccounts {
            control_id: input.control_id,
            rows,
            total_count: u32::try_from(count).map_err(|_| {
                ServiceError::Data("the number of accounts does not fit".to_owned())
            })?,
            offset: input.offset,
            limit,
            balance: whole.balance().map_err(money_error)?,
        })
    })
}

/// The biggest page a screen may ask for.
const MAX_PAGE: u32 = 500;

/// The heads.
///
/// # Errors
/// `ServiceError::Db`.
pub fn heads(ctx: &Context) -> Result<Vec<CoaHead>, ServiceError> {
    ctx.db
        .read(|conn| repo::heads(conn).map_err(ServiceError::from))
}

/// The controls, each with the flag that says whether an account of type `other` may go there.
///
/// # Errors
/// `ServiceError::Db`.
pub fn controls(ctx: &Context) -> Result<Vec<CoaControl>, ServiceError> {
    ctx.db
        .read(|conn| repo::controls(conn).map_err(ServiceError::from))
}
