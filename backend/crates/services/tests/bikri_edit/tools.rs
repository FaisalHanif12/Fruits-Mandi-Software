//! Small helpers of the edit tests: open a posting, change its draft, preview and save, and read
//! rows back with plain SQL.

use mandi_services::ServiceError;
use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriDraft, BikriEdited, BikriPreview, CancelInput, EditOpenInput, EditPreviewInput,
    EditSaveInput, EditableBikri,
};
use mandi_services::core::error::{ErrorCode, ErrorValue, FieldError};

use crate::common::raw;
use crate::support::World;

/// The posting `entry_id` of a بکری, loaded for editing.
pub fn open(w: &World, bikri_id: i64, entry_id: i64) -> EditableBikri {
    bikri::open_for_edit(&w.env.ctx, &EditOpenInput { bikri_id, entry_id }).unwrap()
}

/// The preview of an edited draft.
pub fn preview(w: &World, opened: &EditableBikri, draft: &BikriDraft) -> BikriPreview {
    bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap()
}

/// Saves an edit, asking for the cash confirmation too.
pub fn try_edit(
    w: &World,
    opened: &EditableBikri,
    draft: &BikriDraft,
    confirm: bool,
) -> Result<BikriEdited, ServiceError> {
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft: draft.clone(),
            reason: None,
            confirm_cash_below_zero: confirm,
        },
    )
}

/// Saves an edit that must save, and checks that the preview just before was the save (Code
/// Standards B5).
pub fn edit(w: &World, opened: &EditableBikri, draft: &BikriDraft) -> BikriEdited {
    let before = preview(w, opened, draft);
    let saved =
        try_edit(w, opened, draft, true).unwrap_or_else(|e| panic!("the edit failed: {e:?}"));
    assert_eq!(saved.preview, before);
    saved
}

/// Cancels a بکری with a reason.
pub fn cancel(
    w: &World,
    bikri_id: i64,
    confirm: bool,
) -> Result<mandi_services::core::bikri::BikriCancelled, ServiceError> {
    let version = bikri::get(&w.env.ctx, bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: bikri_id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: confirm,
        },
    )
}

/// Every row of the tables a posting writes, as text, in id order: to prove that nothing posted is
/// changed by an edit (Code Standards B17).
pub fn history(w: &World) -> Vec<(String, Vec<String>)> {
    let conn = raw(&w.dir);
    [
        ("journal_entries", "id, entry_date, posting_type, source_id, reverses_entry_id, status, total_paisa, chain_hash, order_key"),
        ("journal_lines", "id, entry_id, line_no, account_id, pukhta_person_key, dr_paisa, cr_paisa, is_advance"),
        ("partner_share_rows", "id, journal_entry_id, partner_id, kind, base_paisa, share_bps, amount_paisa"),
        ("stock_moves", "id, lot_id, journal_entry_id, kind, qty, reverses_move_id"),
        ("analytics_facts", "id, journal_entry_id, sale_id, qty, seller_amount_paisa, sign"),
    ]
    .iter()
    .map(|(table, columns)| {
        let parts: Vec<String> = columns
            .split(", ")
            .map(|c| format!("quote({c})"))
            .collect();
        let sql = format!(
            "SELECT {} FROM {table} ORDER BY id",
            parts.join(" || '|' || ")
        );
        let mut statement = conn.prepare(&sql).unwrap();
        let rows: Vec<String> = statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        ((*table).to_owned(), rows)
    })
    .collect()
}

/// Rupees of the partner share rows of one entry, in partner order.
pub fn shares_of(w: &World, entry_id: i64) -> Vec<i64> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare("SELECT amount_paisa FROM partner_share_rows WHERE journal_entry_id = ?1 ORDER BY partner_id")
        .unwrap();
    statement
        .query_map([entry_id], |r| Ok(r.get::<_, i64>(0)? / 100))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// The codes and fields of a refusal.
pub fn refusals(error: &ServiceError) -> Vec<(ErrorCode, String)> {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .map(|i| (i.code, i.field.clone().unwrap_or_default()))
            .collect(),
        other => panic!("not a refusal: {other:?}"),
    }
}

/// True if a refusal has this code at this field.
pub fn refused(error: &ServiceError, code: ErrorCode, field: &str) -> bool {
    refusals(error)
        .iter()
        .any(|(c, f)| *c == code && f == field)
}

/// True if a list of field errors has this code at this field.
pub fn flagged(list: &[FieldError], code: ErrorCode, field: &str) -> bool {
    list.iter().any(|e| e.code == code && e.field == field)
}

/// A number value of the first error with this code.
pub fn number(list: &[FieldError], code: ErrorCode, key: &str) -> Option<i64> {
    list.iter()
        .find(|e| e.code == code)
        .and_then(|e| match e.values.get(key) {
            Some(ErrorValue::Number(n)) => Some(*n),
            _ => None,
        })
}

/// A number value of the first refusal with this code.
pub fn refusal_number(error: &ServiceError, code: ErrorCode, key: &str) -> Option<i64> {
    match error {
        ServiceError::Rejected(items) => {
            items
                .iter()
                .find(|i| i.code == code)
                .and_then(|i| match i.values.get(key) {
                    Some(ErrorValue::Number(n)) => Some(*n),
                    _ => None,
                })
        }
        _ => None,
    }
}

/// Cash in hand as signed rupees (a debit balance is plus).
pub fn cash(w: &World) -> i64 {
    let b = mandi_services::ledger::account_balance(
        &w.env.ctx,
        crate::support::id_of(&w.env, "01010001"),
    )
    .unwrap();
    match b.side {
        mandi_services::core::accounts::BalanceSide::Cr => -b.amount_paisa.rupees(),
        _ => b.amount_paisa.rupees(),
    }
}

/// The open advance of a seller in rupees.
pub fn open_advance_of(w: &World, seller: i64) -> i64 {
    mandi_services::assist::advances(&w.env.ctx, seller)
        .unwrap()
        .open_paisa
        .rupees()
}

/// Every unequal debit and credit of the whole journal, in paisa: must be zero.
pub fn trial_difference(w: &World) -> i64 {
    let t = mandi_services::ledger::trial_balance(&w.env.ctx, None).unwrap();
    t.total_dr.as_paisa() - t.total_cr.as_paisa()
}

/// A cash receipt from a buyer on a day (PRD 15.2).
pub fn receipt_on(w: &World, account: i64, amount: &str, date: &str) {
    mandi_services::vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::ReceiptInput {
            date: date.into(),
            account_id: account,
            person_key: None,
            amount: amount.into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
}

/// A cash receipt from a buyer today.
pub fn receipt(w: &World, account: i64, amount: &str) {
    receipt_on(w, account, amount, "10-10-2026");
}

/// What a buyer still owes on his open bills and his unmatched credit, in rupees.
pub fn open_bills(w: &World, account: i64) -> (i64, i64) {
    let bills = mandi_services::vouchers::open_bills(
        &w.env.ctx,
        &mandi_services::core::vouchers::OpenBillsInput {
            account_id: account,
            person_key: None,
        },
    )
    .unwrap();
    (
        bills.open_paisa.rupees(),
        bills.unmatched_credit_paisa.rupees(),
    )
}

/// The sum of the active matching rows of a table, in rupees.
pub fn matched(w: &World, table: &str) -> i64 {
    scalar(
        w,
        &format!("SELECT COALESCE(SUM(amount_paisa), 0) FROM {table} WHERE is_active = 1"),
    ) / 100
}

/// A number from the data file.
pub fn scalar(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// Every row of the tables an edit, a cancel or a later sale may touch, as text: to prove that a
/// failed action leaves the file as it was (NFR-REL-1, E20).
pub fn fingerprint(conn: &mandi_services::db::rusqlite::Connection) -> Vec<(String, Vec<String>)> {
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
        "settings",
        "vouchers",
        "drafts",
    ]
    .iter()
    .map(|table| {
        let mut columns = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        let names: Vec<String> = columns
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let parts: Vec<String> = names.iter().map(|c| format!("quote({c})")).collect();
        let sql = format!(
            "SELECT {} FROM {table} ORDER BY rowid",
            parts.join(" || '|' || ")
        );
        let mut statement = conn.prepare(&sql).unwrap();
        let rows: Vec<String> = statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        ((*table).to_owned(), rows)
    })
    .collect()
}

/// The `rule` name of an ERR_EDIT_NOT_ALLOWED refusal at a field.
pub fn rule_at(error: &ServiceError, field: &str) -> Option<String> {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .find(|i| i.code == ErrorCode::ErrEditNotAllowed && i.field.as_deref() == Some(field))
            .and_then(|i| match i.values.get("rule") {
                Some(ErrorValue::Text(name)) => Some(name.clone()),
                _ => None,
            }),
        _ => None,
    }
}
