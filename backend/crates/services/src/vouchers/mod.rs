//! The vouchers of PRD 15 (plans 10 and 15): پیشگی, وصولی and ادائیگی on one screen with a type
//! list (PRD FR-7.67). Each type keeps its own service function, its own number series and its
//! own posting; the checks they share are in `common`.
//!
//! One voucher is one posting event in one transaction: the voucher row, its journal entry, the
//! matching rows and its audit row are written together or not at all (Code Standards B3). The
//! preview and the save call the same checks (Code Standards B5). A voucher is edited or
//! cancelled by a reversal, never by changing a posted row (PRD FR-6.3, FR-15.3).

mod advance;
mod capital_kinds;
mod cash_kinds;
mod common;
mod edit;
mod expense_reads;
mod general_kind;
mod khata_kinds;
mod kinds;
mod payment;
mod posting_type;
mod preview;
mod reads;
mod receipt;
mod regular;

use mandi_db::vouchers as repo;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::core::vouchers::{VoucherRow, VouchersList, VouchersListInput, VouchersTotals};

pub use advance::{preview_advances, save_advance, save_advances};
pub use capital_kinds::{save_capital_in, save_capital_out};
pub use cash_kinds::{
    save_money_lent, save_money_lent_return, save_personal_expense, save_personal_return,
    save_shop_expense,
};
pub(crate) use common::{rematch_party, role_of_account, stored_cash_change};
pub use edit::{cancel, edit, get};
pub use expense_reads::{preview_expense, repeat_last_month_preview};
pub use general_kind::{general_preview, general_save};
pub use khata_kinds::{save_khata_in, save_khata_out};
pub use payment::save_payment;
pub use preview::preview;
pub use reads::{advances_report, collection_list, open_bills, open_payables};
pub use receipt::{save_receipt, save_receipts_batch};
pub use regular::make_regular;

use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};

/// The most lines in one running list.
pub const MAX_LIST_LINES: usize = 500;

/// The vouchers of a kind in a date range, by day and number. `editable` is true for a saved
/// voucher whose day is still open (PRD FR-15.3, FR-6.6).
///
/// # Errors
/// The date codes on `from` and `to`; `ServiceError::Db`.
pub fn list(ctx: &Context, input: &VouchersListInput) -> Result<VouchersList, ServiceError> {
    let (closed, today) = day::window(ctx)?;
    let mut errors = validate::FieldErrors::default();
    let bound = |errors: &mut validate::FieldErrors, field: &str, text: &str| {
        if text.trim().is_empty() {
            None
        } else {
            errors
                .take(validate::date(field, text, today))
                .map(BusinessDate::to_iso)
        }
    };
    let from = bound(&mut errors, "from", &input.from);
    let to = bound(&mut errors, "to", &input.to);
    errors.finish()?;
    let filter = repo::ListFilter {
        voucher_type: input.voucher_type,
        from: from.as_deref(),
        to: to.as_deref(),
        account_id: input.account_id,
        person_key: input.person_key.as_deref(),
    };
    // One page and the totals over the whole filter, each from one query (PRD FR-19.11, B31).
    let (mut rows, totals) = ctx.db.read(|conn| {
        let totals = repo::list_totals(conn, &filter)?;
        let rows = repo::list_page(
            conn,
            &filter,
            i64::from(input.offset),
            crate::page::limit_of(input.limit),
        )?;
        Ok::<_, ServiceError>((rows, totals))
    })?;
    mark_editable(&mut rows, closed);
    // The total is of what is saved: a cancelled voucher no longer moves money. It is given only
    // for one type (PRD FR-15.1).
    let saved = Paisa::try_from_paisa(totals.saved_paisa).map_err(money_error)?;
    Ok(VouchersList {
        rows,
        total_count: totals.count,
        totals: VouchersTotals {
            cancelled_count: totals.cancelled,
            total_paisa: input.voucher_type.map(|_| saved),
        },
    })
}

/// Sets `editable` on rows read from the repository: saved, and dated after the "closed up to"
/// day (PRD FR-6.6). Dates are `YYYY-MM-DD`, so text order is date order.
pub(crate) fn mark_editable(rows: &mut [VoucherRow], closed: Option<BusinessDate>) {
    let closed = closed.map(BusinessDate::to_iso);
    for row in rows {
        row.editable = row.status == "saved"
            && closed
                .as_deref()
                .is_none_or(|c| row.entry_date.as_str() > c);
    }
}
