//! Reads of saved بکری (PRD FR-12.7, FR-12.13, plan 13): one بکری with its stored values, the
//! list of a day or a range, the find, and first / previous / next / last. Read only (Code
//! Standards B29). Every number is a stored value: nothing is worked out again (B20).

use mandi_db::bikri::saved::{self as repo, ListFilter};
use mandi_domain::core::bikri::{
    BikriDetail, BikriFindInput, BikriList, BikriListInput, BikriListRow, BikriListTotals,
    BikriNavigateInput, BikriStatus,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::paging::MonthSubtotal;
use mandi_domain::core::validate::{self, FieldErrors};

use crate::context::Context;
use crate::day;
use crate::error::ServiceError;

/// The most rows of a find.
const FIND_LIMIT: i64 = 50;

/// A date typed in a filter: empty or missing is no limit.
pub(super) fn optional_date(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&String>,
    today: BusinessDate,
) -> Option<String> {
    let text = text.map(String::as_str).filter(|t| !t.trim().is_empty())?;
    errors
        .take(validate::date(field, text, today))
        .map(BusinessDate::to_iso)
}

/// A saved بکری with its lots, lines, postings and partner split, as stored.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, id: i64) -> Result<BikriDetail, ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    let mut detail = ctx
        .db
        .read(|conn| repo::get(conn, id).map_err(ServiceError::from))?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
    mark_editable(&mut detail, closed_up_to);
    Ok(detail)
}

/// Sets `editable` on the postings of a بکری: a first or later posting that is not reversed, of a
/// بکری that is saved, on a day after the "closed up to" day (PRD FR-12.18).
pub(crate) fn mark_editable(detail: &mut BikriDetail, closed_up_to: Option<BusinessDate>) {
    let saved = detail.status == BikriStatus::Saved;
    for posting in &mut detail.postings {
        let open = BusinessDate::parse_iso(&posting.entry_date)
            .is_some_and(|date| closed_up_to.is_none_or(|closed| date > closed));
        posting.editable = saved
            && !posting.reversed
            && matches!(posting.posting_type.as_str(), "bikri_first" | "bikri_later")
            && open;
    }
}

/// The list of بکری (PRD FR-12.7), newest first. `date` is one day and wins over `from` and
/// `to`.
///
/// # Errors
/// The date codes on `date`, `from` and `to`.
pub fn list(ctx: &Context, input: &BikriListInput) -> Result<BikriList, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let date = optional_date(&mut errors, "date", input.date.as_ref(), today);
    let from = optional_date(&mut errors, "from", input.from.as_ref(), today);
    let to = optional_date(&mut errors, "to", input.to.as_ref(), today);
    errors.finish()?;
    let (from, to) = match date {
        Some(day) => (Some(day.clone()), Some(day)),
        None => (from, to),
    };
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    let order_by = input
        .sort
        .as_ref()
        .map(|s| {
            s.order_by(&repo::SORT_COLUMNS).map(|order| {
                // Rows of the same day follow the same direction by number.
                if s.column == "entry_date" {
                    format!("{order}, e.number {}", s.direction.sql())
                } else {
                    order
                }
            })
        })
        .transpose()
        .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    let search = input
        .search
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned);
    // Cancelled بکری step aside unless asked for, or unless a status was chosen.
    let status = match (input.status, input.show_finished) {
        (None, false) => Some(BikriStatus::Saved),
        (chosen, _) => chosen,
    };
    let in_date_order = input.sort.as_ref().is_none_or(|s| s.column == "entry_date");
    let newest_first = input
        .sort
        .as_ref()
        .is_none_or(|s| s.direction == mandi_domain::core::paging::SortDirection::Desc);
    let filter = ListFilter {
        from,
        to,
        seller_id: input.seller_id,
        item_id: input.item_id,
        status,
        number: None,
        truck_prefix: None,
        search,
        order_by,
        limit: crate::page::limit_of(input.limit),
        offset: i64::from(input.offset),
    };
    // One page and the totals over the whole filter, each from indexed queries (PRD FR-19.11, B31).
    ctx.db.read(|conn| {
        let totals = repo::list_totals(conn, &filter)?;
        let rows = repo::list(conn, &filter)?;
        let paisa = |value: i64| Paisa::try_from_paisa(value).map_err(crate::error::money_error);
        // Month headings with their subtotals, over the whole filter (plan 38 R4).
        let mut months = Vec::new();
        if input.group_by_month && in_date_order {
            for rec in repo::list_months(conn, &filter)? {
                months.push(MonthSubtotal {
                    month: rec.month,
                    count: rec.count,
                    figures: vec![
                        MonthSubtotal::count_of("qty_received", rec.qty_received),
                        MonthSubtotal::count_of("qty_sold", rec.qty_sold),
                        MonthSubtotal::money("gross", rec.gross_paisa),
                        MonthSubtotal::money("net", rec.net_paisa),
                        MonthSubtotal::money("buyer_total", rec.buyer_total_paisa),
                    ],
                });
            }
            if newest_first {
                months.reverse();
            }
        }
        Ok(BikriList {
            rows,
            total_count: totals.count,
            totals: BikriListTotals {
                qty_received: totals.qty_received,
                qty_sold: totals.qty_sold,
                gross: paisa(totals.gross_paisa)?,
                net: paisa(totals.net_paisa)?,
                buyer_total: paisa(totals.buyer_total_paisa)?,
            },
            months,
        })
    })
}

/// Finds بکری by number, the start of the truck number, seller, day or item (PRD FR-12.13,
/// FR-7.50). Every filter given must match. Newest first, at most 50 rows.
///
/// # Errors
/// The date codes on `date`.
pub fn find(ctx: &Context, input: &BikriFindInput) -> Result<Vec<BikriListRow>, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let date = optional_date(&mut errors, "date", input.date.as_ref(), today);
    errors.finish()?;
    let truck_prefix = input
        .truck_no
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned);
    let filter = ListFilter {
        from: date.clone(),
        to: date,
        seller_id: input.seller_id,
        item_id: input.item_id,
        status: None,
        number: input.number,
        truck_prefix,
        search: None,
        order_by: None,
        limit: FIND_LIMIT,
        offset: 0,
    };
    ctx.db
        .read(|conn| repo::list(conn, &filter).map_err(ServiceError::from))
}

/// The id of the first, previous, next or last بکری by number (PRD FR-12.13). `None` when there
/// is none in that direction (the first has no previous one, the last no next one, and an empty
/// book has none at all).
///
/// # Errors
/// `ERR_NOT_FOUND` on `current_id`.
pub fn navigate(ctx: &Context, input: &BikriNavigateInput) -> Result<Option<i64>, ServiceError> {
    ctx.db.read(|conn| {
        let current_number =
            match input.current_id {
                Some(id) => Some(repo::number_of(conn, id)?.ok_or_else(|| {
                    ServiceError::reject_field(ErrorCode::ErrNotFound, "current_id")
                })?),
                None => None,
            };
        Ok(repo::neighbour(conn, current_number, input.to)?)
    })
}
