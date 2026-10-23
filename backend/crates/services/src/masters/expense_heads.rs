//! The expense heads, most used first (PRD FR-7.62, plan 17).

use mandi_db::expense_heads as repo;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::expenses::{ExpenseHeadUse, ExpenseHeads};

use crate::context::Context;
use crate::error::ServiceError;

/// The active expense heads, most used first, then by code. A cancelled expense does not count
/// as a use. `id` is what a saved shop expense sends as `expense_head_id`; `head.id` is the same
/// number.
///
/// # Errors
/// `ServiceError::Db`.
pub fn expense_heads(ctx: &Context) -> Result<ExpenseHeads, ServiceError> {
    ctx.db.read(|conn| {
        let rows = repo::heads_by_use(conn)?
            .into_iter()
            .map(|used| ExpenseHeadUse {
                id: used.head.id,
                head: RecordRef {
                    id: used.head.id,
                    code: used.head.code.clone(),
                    name_ur: used.head.name_ur.clone(),
                    name_en: used.head.name_en.clone(),
                },
                account_id: used.head.account_id,
                use_count: used.use_count,
                last_used_date: used.last_used,
            })
            .collect();
        Ok(ExpenseHeads { rows })
    })
}
