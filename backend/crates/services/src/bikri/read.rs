//! Reads of saved بکری (PRD FR-12.7, FR-12.13, plan 13): one بکری with its stored values, the
//! list of a day or a range, the find, and first / previous / next / last. Read only (Code
//! Standards B29). Every number is a stored value: nothing is worked out again (B20).

use mandi_db::bikri::saved::{self as repo, ListFilter};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::bikri::{
    BikriDetail, BikriFindInput, BikriList, BikriListInput, BikriListRow, BikriListTotals,
    BikriNavigateInput, BikriStatus, SavedPosting,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::{EditEntity, EntryFlags};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::paging::MonthSubtotal;
use mandi_domain::core::text;
use mandi_domain::core::validate::{self, FieldErrors};

use crate::context::Context;
use crate::day;
use crate::error::ServiceError;
use crate::flags::Marker;

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
    ctx.db.read(|conn| {
        let mut detail = repo::get(conn, id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
        mark_flags(conn, &mut detail, closed_up_to)?;
        Ok(detail)
    })
}

/// Sets `editable` on the postings and the flags of the whole بکری (PRD FR-7.70, FR-7.71,
/// FR-21.4). The day of its first posting decides the reopen and the month lock; the stamp is of
/// the بکری.
pub(crate) fn mark_flags(
    conn: &Connection,
    detail: &mut BikriDetail,
    closed_up_to: Option<BusinessDate>,
) -> Result<(), ServiceError> {
    mark_editable(detail, closed_up_to);
    let saved = detail.status == BikriStatus::Saved;
    let has_live = detail.postings.iter().any(is_live);
    let mut flags = if saved && has_live {
        EntryFlags::open()
    } else {
        EntryFlags::plain(false, false)
    };
    Marker::load(conn, EditEntity::Bikri, &[detail.id])?.mark(
        detail.id,
        &detail.entry_date,
        saved,
        &mut flags,
    );
    detail.flags = flags;
    Ok(())
}

/// A first or later posting that no edit or cancel has reversed.
fn is_live(posting: &SavedPosting) -> bool {
    !posting.reversed && matches!(posting.posting_type.as_str(), "bikri_first" | "bikri_later")
}

/// Sets `editable` on the postings of a بکری: a first or later posting that is not reversed, of a
/// بکری that is saved, on a day after the "closed up to" day (PRD FR-12.18). It means "can be
/// edited right now"; the flags of [`mark_flags`] also count a closed day that a reopen would open.
pub(crate) fn mark_editable(detail: &mut BikriDetail, closed_up_to: Option<BusinessDate>) {
    let saved = detail.status == BikriStatus::Saved;
    let mut any_editable = false;
    let mut all_open = true;
    for posting in &mut detail.postings {
        let open = BusinessDate::parse_iso(&posting.entry_date)
            .is_some_and(|date| closed_up_to.is_none_or(|closed| date > closed));
        let live = is_live(posting);
        posting.editable = saved && live && open;
        any_editable |= posting.editable;
        all_open &= !live || open;
    }
    // A cancel reverses every live posting, so every one of their days must be open (PRD
    // FR-12.22). The flags a screen shows come from `mark_flags`.
    detail.flags = EntryFlags::plain(any_editable, saved && all_open);
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
        // The list has no buyer filter; the find has (plan 67).
        ..ListFilter::default()
    };
    // One page and the totals over the whole filter, each from indexed queries (PRD FR-19.11, B31).
    ctx.db.read(|conn| {
        let totals = repo::list_totals(conn, &filter)?;
        let mut rows = repo::list(conn, &filter)?;
        mark_rows(conn, &mut rows)?;
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

/// Finds بکری by number, the start of the truck number, seller, buyer, a چکتا buyer's name, day or
/// item (PRD FR-12.13, FR-7.50, plan 67). Every filter given must match. Newest first, at most 50
/// rows.
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
    let buyer_text = input
        .buyer_text
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned);
    let filter = ListFilter {
        from: date.clone(),
        to: date,
        seller_id: input.seller_id,
        item_id: input.item_id,
        buyer_id: input.buyer_account_id,
        // The typed name of a چکتا buyer, as the search keys of his record are built (PRD FR-12.13).
        chukta_name_ur: buyer_text.as_deref().map(text::search_ur),
        chukta_name_en: buyer_text.as_deref().map(text::search_en),
        status: None,
        number: input.number,
        truck_prefix,
        search: None,
        order_by: None,
        limit: FIND_LIMIT,
        offset: 0,
    };
    ctx.db.read(|conn| {
        let mut rows = repo::list(conn, &filter)?;
        mark_rows(conn, &mut rows)?;
        Ok(rows)
    })
}

/// Sets the flags of the rows of a list (PRD FR-7.70, FR-7.71, FR-21.4): both buttons work on a
/// saved بکری; on a closed day the row asks for a reopen; a month-closed month and a cancelled
/// بکری are locked; the stamp of the last edit is added.
fn mark_rows(conn: &Connection, rows: &mut [BikriListRow]) -> Result<(), ServiceError> {
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let marker = Marker::load(conn, EditEntity::Bikri, &ids)?;
    for row in rows {
        let mut flags = EntryFlags::open();
        marker.mark(
            row.id,
            &row.entry_date,
            row.status == BikriStatus::Saved,
            &mut flags,
        );
        row.flags = flags;
    }
    Ok(())
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
