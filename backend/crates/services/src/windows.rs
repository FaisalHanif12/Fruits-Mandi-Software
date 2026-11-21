//! `windows_bounds` (plan 76 point 8, PRD FR-19.11): the first day that has data in a list, so the
//! back button of its window (7 days or a month) stops there. Read only, one indexed question each.
//!
//! Example: the shop's first expense is dated 10-10-26 and today is 14-11-26: the expenses window
//! goes back to October 2026 and no further.

use mandi_db::partners as partner_repo;
use mandi_db::rusqlite::{Connection, OptionalExtension, params_from_iter, types::Value};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::windows::{WindowSource, WindowsBounds, WindowsBoundsInput};

use crate::context::Context;
use crate::day;
use crate::error::ServiceError;

fn first(conn: &Connection, sql: &str, args: Vec<Value>) -> Result<Option<String>, ServiceError> {
    Ok(conn
        .query_row(sql, params_from_iter(args), |r| {
            r.get::<_, Option<String>>(0)
        })
        .optional()?
        .flatten())
}

/// The code of the shop's one cash account (PRD FR-10.4).
const CASH_CODE: &str = "01010001";

/// The first day with a posted line on the cash account.
fn first_cash_day(conn: &Connection) -> Result<Option<String>, ServiceError> {
    let cash: Option<i64> = conn
        .query_row(
            "SELECT id FROM accounts WHERE code = ?1",
            [CASH_CODE],
            |r| r.get(0),
        )
        .optional()?;
    match cash {
        Some(id) => first_day_of(conn, id),
        None => Ok(None),
    }
}

/// The first day with a posted line on an account: one cheap read for any account, however many
/// lines it has (NFR-SPD-24, see `mandi_db::ledger::edge_day_of_account`).
fn first_day_of(conn: &Connection, account_id: i64) -> Result<Option<String>, ServiceError> {
    Ok(mandi_db::ledger::edge_day_of_account(
        conn,
        account_id,
        mandi_db::ledger::DayEdge::First,
    )?)
}

const VOUCHER_FIRST: &str =
    "SELECT MIN(entry_date) FROM vouchers WHERE status = 'saved' AND type IN";

fn voucher_types(types: &[&str]) -> String {
    format!(
        "{VOUCHER_FIRST} ({})",
        types
            .iter()
            .map(|t| format!("'{t}'"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// The first day with data in a list, and today as the last day.
///
/// # Errors
/// `ERR_NOT_FOUND` on `source.partner_id` for an unknown partner; `ServiceError::Db`.
pub fn bounds(ctx: &Context, input: &WindowsBoundsInput) -> Result<WindowsBounds, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let first_date = ctx.db.read(|conn| match &input.source {
        WindowSource::Sales => first(conn, "SELECT MIN(entry_date) FROM bikri_entries", vec![]),
        WindowSource::Journal => first(
            conn,
            "SELECT MIN(entry_date) FROM journal_entries WHERE status = 'posted'",
            vec![],
        ),
        WindowSource::Cashbook => first_cash_day(conn),
        WindowSource::Vouchers { voucher_type } => match voucher_type {
            Some(t) => first(
                conn,
                "SELECT MIN(entry_date) FROM vouchers WHERE status = 'saved' AND type = ?1",
                vec![Value::Text(t.as_str().to_owned())],
            ),
            None => first(
                conn,
                "SELECT MIN(entry_date) FROM vouchers WHERE status = 'saved'",
                vec![],
            ),
        },
        // The advances and the payments on a buyer account (PRD FR-15.11, v1.51).
        WindowSource::Advances => first(
            conn,
            "SELECT MIN(v.entry_date) FROM vouchers v
             WHERE v.status = 'saved'
               AND (v.type = 'advance'
                    OR (v.type = 'payment' AND EXISTS (SELECT 1 FROM accounts b
                        WHERE b.id = v.account_id AND b.party_type = 'buyer')))",
            vec![],
        ),
        // The index on the time gives the oldest row at once; the day is its first 10 characters.
        WindowSource::Audit => first(
            conn,
            "SELECT substr(at, 1, 10) FROM audit_log ORDER BY at LIMIT 1",
            vec![],
        ),
        WindowSource::Expenses => first(
            conn,
            &voucher_types(&["shop_expense", "personal_expense", "personal_return"]),
            vec![],
        ),
        WindowSource::Fruit => first(
            conn,
            "SELECT MIN(entry_date) FROM fruit_takes WHERE status = 'saved'",
            vec![],
        ),
        WindowSource::Lent | WindowSource::LentDues => first(
            conn,
            &voucher_types(&["money_lent", "money_lent_return"]),
            vec![],
        ),
        WindowSource::Khata { account_id } => {
            let sql = voucher_types(&["khata_out", "khata_in"]);
            match account_id {
                Some(id) => first(
                    conn,
                    &format!("{sql} AND account_id = ?1"),
                    vec![Value::Integer(*id)],
                ),
                None => first(conn, &sql, vec![]),
            }
        }
        WindowSource::Capital => {
            first(conn, &voucher_types(&["capital_in", "capital_out"]), vec![])
        }
        WindowSource::PartnerTakes { partner_id } => {
            let partner = partner_repo::get(conn, *partner_id)?.ok_or_else(|| {
                ServiceError::reject_field(ErrorCode::ErrNotFound, "source.partner_id")
            })?;
            first(
                conn,
                "SELECT MIN(e.entry_date) FROM journal_lines l
                 JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
                 WHERE l.account_id = ?1",
                vec![Value::Integer(partner.drawings_account_id)],
            )
        }
        WindowSource::PartnerStatement { partner_id } => {
            let partner = partner_repo::get(conn, *partner_id)?.ok_or_else(|| {
                ServiceError::reject_field(ErrorCode::ErrNotFound, "source.partner_id")
            })?;
            Ok(crate::partners::first_event_date(conn, &partner)?)
        }
        WindowSource::PartyLedger { account_id } => first_day_of(conn, *account_id),
        WindowSource::Commission => {
            first(conn, "SELECT MIN(entry_date) FROM analytics_facts", vec![])
        }
    })?;
    Ok(WindowsBounds {
        first_date,
        last_date: today.to_iso(),
    })
}
