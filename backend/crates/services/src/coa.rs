//! The chart of accounts (PRD FR-10.1, plan 07): heads, controls and accounts with live balances.
//!
//! Only reads. Balances come from the posted journal, never from a stored total (PRD FR-6.1).
//! A head or control balance is the debit total minus the credit total of everything under it,
//! worked out with checked maths.

use std::collections::HashMap;

use mandi_db::accounts as repo;
use mandi_db::ledger::Totals;
use mandi_domain::core::accounts::{
    AccountNode, Balance, CoaControl, CoaHead, CoaTree, ControlNode, HeadNode,
};
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

/// Heads with controls with accounts and their balances, all from one read.
///
/// # Errors
/// `ServiceError::Db`; `ERR_OVERFLOW` if a total would not fit.
pub fn tree(ctx: &Context) -> Result<CoaTree, ServiceError> {
    ctx.db.read(|conn| {
        let heads = repo::heads(conn)?;
        let controls = repo::controls(conn)?;
        let accounts = repo::tree_accounts(conn)?;
        let totals: HashMap<i64, Totals> = ledger::balances_by_account(conn, None)?;
        build(&heads, &controls, &accounts, &totals).map_err(money_error)
    })
}

/// Builds the three levels. The accounts and controls arrive in code order and keep it.
fn build(
    heads: &[CoaHead],
    controls: &[CoaControl],
    accounts: &[repo::TreeAccount],
    totals: &HashMap<i64, Totals>,
) -> Result<CoaTree, MoneyError> {
    let mut head_nodes = Vec::with_capacity(heads.len());
    for head in heads {
        let mut head_sum = Sum::ZERO;
        let mut control_nodes = Vec::new();
        for control in controls.iter().filter(|c| c.head_id == head.id) {
            let mut control_sum = Sum::ZERO;
            let mut account_nodes = Vec::new();
            for account in accounts.iter().filter(|a| a.control_id == control.id) {
                let sum = match totals.get(&account.id) {
                    Some(found) => Sum::of(*found)?,
                    None => Sum::ZERO,
                };
                control_sum = control_sum.add(sum)?;
                account_nodes.push(AccountNode {
                    id: account.id,
                    code: account.code.clone(),
                    name_ur: account.name_ur.clone(),
                    name_en: account.name_en.clone(),
                    balance: sum.balance()?,
                    is_active: account.is_active,
                    is_system: account.is_system,
                    tag: None,
                });
            }
            head_sum = head_sum.add(control_sum)?;
            control_nodes.push(ControlNode {
                id: control.id,
                code: control.code.clone(),
                name_ur: control.name_ur.clone(),
                name_en: control.name_en.clone(),
                party_type: control.party_type.clone(),
                balance: control_sum.balance()?,
                accounts: account_nodes,
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
