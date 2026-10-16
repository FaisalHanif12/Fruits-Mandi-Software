//! Small helpers of the save tests: saving, reading rows back with plain SQL, and balances.

use mandi_services::bikri;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{BikriDraft, BikriSaveInput, BikriSaved, PukhtaInput};
use mandi_services::ledger;
use mandi_services::{ServiceError, db::rusqlite::Connection};

use crate::common::raw;
use crate::support::World;

/// Saves a draft, asking for the cash confirmation too, so only the other errors can stop it.
pub fn save(w: &World, draft: &BikriDraft) -> Result<BikriSaved, ServiceError> {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: true,
        },
    )
}

/// Saves a draft that must save.
pub fn save_ok(w: &World, draft: &BikriDraft) -> BikriSaved {
    save(w, draft).unwrap_or_else(|e| panic!("the save failed: {e:?}"))
}

/// A one-time person.
pub fn person(name_ur: &str, name_en: &str, phone: &str) -> Option<PukhtaInput> {
    Some(PukhtaInput {
        name_ur: name_ur.into(),
        name_en: name_en.into(),
        phone: phone.into(),
    })
}

/// A number from the data file.
pub fn scalar(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// The counts of every table a save writes, to compare before and after.
pub fn snapshot(conn: &Connection) -> Vec<(String, i64)> {
    [
        "bikri_entries",
        "bikri_lots",
        "bikri_sales",
        "journal_entries",
        "journal_lines",
        "partner_share_rows",
        "stock_moves",
        "analytics_facts",
        "bill_allocations",
        "advance_allocations",
        "audit_log",
        "drafts",
        "vouchers",
    ]
    .iter()
    .map(|table| {
        let n: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        ((*table).to_owned(), n)
    })
    .collect()
}

/// The lines of a posted entry as (code, debit, credit, person key), in rupees.
pub fn entry_lines(w: &World, entry_id: i64) -> Vec<(String, i64, i64, Option<String>)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT a.code, l.dr_paisa, l.cr_paisa, l.pukhta_person_key
             FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             WHERE l.entry_id = ?1 ORDER BY l.line_no",
        )
        .unwrap();
    statement
        .query_map([entry_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)? / 100,
                r.get::<_, i64>(2)? / 100,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// A balance as (side, rupees).
pub fn balance(w: &World, account_id: i64) -> (BalanceSide, i64) {
    let b = ledger::account_balance(&w.env.ctx, account_id).unwrap();
    (b.side, b.amount_paisa.rupees())
}

/// The active matching rows of a table as (bill or advance line, credit line, rupees).
pub fn rows(w: &World, table: &str) -> Vec<(i64, i64, i64)> {
    let (a, b) = if table == "bill_allocations" {
        ("bill_line_id", "payment_line_id")
    } else {
        ("advance_line_id", "credit_line_id")
    };
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(&format!(
            "SELECT {a}, {b}, amount_paisa FROM {table} WHERE is_active = 1 ORDER BY id"
        ))
        .unwrap();
    statement
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)? / 100,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// Gives a seller an advance in cash on 10-10-2026 (PRD 15.4).
pub fn give_advance(w: &World, seller: i64, rupees: &str) {
    give_advance_on(w, seller, rupees, "10-10-2026");
}

/// Gives a seller an advance in cash on a day (PRD 15.4).
pub fn give_advance_on(w: &World, seller: i64, rupees: &str, date: &str) {
    mandi_services::vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::AdvanceInput {
            date: date.into(),
            seller_account_id: seller,
            amount: rupees.into(),
            mode: mandi_services::core::vouchers::VoucherMode::Cash,
            bank_account_id: None,
            cheque_no: String::new(),
            cheque_date: String::new(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
}

/// An opening balance of an account dated 10-10-2026 (PRD FR-10.9): a debit or a credit.
pub fn open_account(w: &World, account_id: i64, dr: &str, cr: &str) {
    open_account_as(w, account_id, dr, cr, false);
}

/// An opening debit of a seller that counts as an advance (PRD FR-15.8).
pub fn open_advance(w: &World, account_id: i64, dr: &str) {
    open_account_as(w, account_id, dr, "", true);
}

fn open_account_as(w: &World, account_id: i64, dr: &str, cr: &str, is_advance: bool) {
    mandi_services::accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::accounts::OpeningsInput {
            lines: vec![mandi_services::core::accounts::OpeningLineInput {
                account_id,
                version: mandi_services::accounts::get(&w.env.ctx, account_id)
                    .unwrap()
                    .version,
                opening_date: "10-10-2026".into(),
                opening_dr: dr.into(),
                opening_cr: cr.into(),
                opening_is_advance: is_advance,
            }],
        },
    )
    .unwrap();
}

/// True if an error is a rejection with this code at this field.
pub fn rejects(
    error: &ServiceError,
    code: mandi_services::core::error::ErrorCode,
    field: &str,
) -> bool {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .any(|i| i.code == code && i.field.as_deref() == Some(field)),
        _ => false,
    }
}
