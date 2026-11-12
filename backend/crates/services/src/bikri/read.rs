//! Reads of saved بکری (PRD FR-12.7, FR-12.13, plan 13): one بکری with its stored values, the
//! list of a day or a range, the find, and first / previous / next / last. Read only (Code
//! Standards B29). Every number is a stored value: nothing is worked out again (B20).

use mandi_db::bikri::saved::{self as repo, ListFilter};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::bikri::{
    BikriDay, BikriDetail, BikriFindInput, BikriList, BikriListInput, BikriListRow,
    BikriListTotals, BikriListWhole, BikriListWholeInput, BikriNavigateInput, BikriStatus,
    SavedPosting,
};
use mandi_domain::core::dates::{self, BusinessDate};
use mandi_domain::core::edit::{EditEntity, EntryFlags};
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
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

/// The most days a window of `days` may span: a year (plan 72 P4). The screen asks for 7.
const MAX_DAY_WINDOW: i64 = 366;

/// How many rows one read of a whole list takes (plan 72 P8). A print holds every row, but the
/// queries that add the sellers and the edit marks of a page take one value for each row, so a long
/// list is read a page at a time and joined.
const WHOLE_PAGE: u32 = 500;

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

/// Refuses typed text that is too long to search with (Code Standards 4.2, V0): `ERR_TEXT_TOO_LONG`
/// on `field` with the most allowed, so a huge paste ends as a refusal with a code and never reaches
/// the database as a pattern it cannot take.
fn check_typed_length(field: &str, text: Option<&str>) -> Result<(), ServiceError> {
    let most = crate::search::MAX_QUERY_CHARS;
    if text.is_some_and(|t| t.chars().take(most.saturating_add(1)).count() > most) {
        return Err(ServiceError::reject_with(
            ErrorCode::ErrTextTooLong,
            Some(field),
            "max",
            i64::try_from(most).unwrap_or(i64::MAX),
        ));
    }
    Ok(())
}

/// A date typed in a filter of the list of past بکری: empty or missing is no limit, and a date
/// after today is `ERR_DATE_IN_FUTURE` (plan 72 P6, with the day of today as `today`).
fn past_date(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&String>,
    today: BusinessDate,
) -> Option<BusinessDate> {
    let text = text.map(String::as_str).filter(|t| !t.trim().is_empty())?;
    let date = errors.take(validate::date(field, text, today))?;
    if date > today {
        errors.push(
            FieldError::new(ErrorCode::ErrDateInFuture, field)
                .with("today", ErrorValue::Text(dates::format_ddmmyyyy(today))),
        );
        return None;
    }
    Some(date)
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

/// The list of بکری (PRD FR-12.7, پچھلی بکری, plan 72), newest first. `date` is one day and wins over
/// `from` and `to`. A day, a window of days or the whole book is read with the same call: the page,
/// the count and the totals over the whole filter, and with `group_by_day` the days of the window,
/// each with its count and خام بکری (empty days too). `all` sets the filters aside (P8).
///
/// # Errors
/// The date codes on `date`, `from` and `to`; `ERR_DATE_IN_FUTURE` for a date after today;
/// with `group_by_day`: `ERR_REQUIRED` on `from` or `to` when one is missing, `ERR_DATE_RANGE` on
/// `to` when it is before `from`, `ERR_OUT_OF_RANGE` on `to` when the window is more than 366 days.
pub fn list(ctx: &Context, input: &BikriListInput) -> Result<BikriList, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let prepared = prepare_list(today, input)?;
    // One page, the totals and the days over the whole filter, each from indexed queries (PRD
    // FR-19.11, B31), all in one read snapshot: a save that commits meanwhile shows in all of them or
    // in none, so a day heading always agrees with its rows and with `total_count` (PRD FR-6.1).
    ctx.db.read_snapshot(|conn| read_list(conn, &prepared))
}

/// A list input that has been checked and turned into the filter of the database (Code Standards
/// 4.1 layer 2), with what the read needs besides it.
#[derive(Clone)]
struct Prepared {
    filter: ListFilter,
    /// The list is in date order (no sort, or a sort by `entry_date`).
    in_date_order: bool,
    /// Newest first (no sort, or a descending sort).
    newest_first: bool,
    group_by_month: bool,
    group_by_day: bool,
}

/// Checks the input of `list` and builds its filter. Nothing is read.
fn prepare_list(today: BusinessDate, input: &BikriListInput) -> Result<Prepared, ServiceError> {
    let mut errors = FieldErrors::default();
    let date = past_date(&mut errors, "date", input.date.as_ref(), today);
    let from = past_date(&mut errors, "from", input.from.as_ref(), today);
    let to = past_date(&mut errors, "to", input.to.as_ref(), today);
    errors.finish()?;
    let (from, to) = match date {
        Some(day) => (Some(day), Some(day)),
        None => (from, to),
    };
    // The window of the days is checked before anything is read (plan 72 P4).
    if input.group_by_day {
        let (Some(first), Some(last)) = (from, to) else {
            let missing = if from.is_none() { "from" } else { "to" };
            return Err(ServiceError::reject_field(ErrorCode::ErrRequired, missing));
        };
        if last < first {
            return Err(ServiceError::reject_field(ErrorCode::ErrDateRange, "to"));
        }
        if last.days_after(first) >= MAX_DAY_WINDOW {
            return Err(ServiceError::reject_with(
                ErrorCode::ErrOutOfRange,
                Some("to"),
                "max",
                MAX_DAY_WINDOW,
            ));
        }
    }
    let (from, to) = (from.map(BusinessDate::to_iso), to.map(BusinessDate::to_iso));
    // The typed search is checked whether it is used or set aside, like every input (V0).
    check_typed_length("search", input.search.as_deref())?;
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
    // "پوری بکری دکھائیں" (plan 72 P8) sets the filters, the search and the hiding of cancelled
    // بکری aside; the dates stay.
    let (seller_id, item_id, search, chosen_status, show_finished) = if input.all {
        (None, None, None, None, true)
    } else {
        (
            input.seller_id,
            input.item_id,
            search,
            input.status,
            input.show_finished,
        )
    };
    // Cancelled بکری step aside unless asked for, or unless a status was chosen.
    let status = match (chosen_status, show_finished) {
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
        seller_id,
        item_id,
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
    Ok(Prepared {
        filter,
        in_date_order,
        newest_first,
        group_by_month: input.group_by_month,
        group_by_day: input.group_by_day,
    })
}

/// Reads one page of the list, its totals, its months and its days from an open connection (inside
/// one read snapshot), so every figure of the answer is of the same moment.
fn read_list(conn: &Connection, prepared: &Prepared) -> Result<BikriList, ServiceError> {
    let filter = &prepared.filter;
    let totals = repo::list_totals(conn, filter)?;
    let mut rows = repo::list(conn, filter)?;
    mark_rows(conn, &mut rows)?;
    let paisa = |value: i64| Paisa::try_from_paisa(value).map_err(crate::error::money_error);
    // Month headings with their subtotals, over the whole filter (plan 38 R4).
    let mut months = Vec::new();
    if prepared.group_by_month && prepared.in_date_order {
        for rec in repo::list_months(conn, filter)? {
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
        if prepared.newest_first {
            months.reverse();
        }
    }
    // The days of the window, empty ones too, from one grouped indexed query (plan 72 P4, P7).
    let days = if prepared.group_by_day {
        // Oldest day first only for a list sorted by day, oldest first.
        let ascending = prepared.in_date_order && !prepared.newest_first;
        window_days(conn, filter, !ascending)?
    } else {
        Vec::new()
    };
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
        days,
    })
}

/// One entry for each date of the window of `filter` (both ends are set, checked by the caller):
/// the days that have بکری come from the database, the others are added with 0. Newest day first
/// when `descending`, else oldest first.
fn window_days(
    conn: &Connection,
    filter: &ListFilter,
    descending: bool,
) -> Result<Vec<BikriDay>, ServiceError> {
    let recs = repo::list_days(conn, filter)?;
    let (Some(first), Some(last)) = (
        filter.from.as_deref().and_then(BusinessDate::parse_iso),
        filter.to.as_deref().and_then(BusinessDate::parse_iso),
    ) else {
        return Ok(Vec::new());
    };
    let mut days = Vec::new();
    let mut at = first;
    loop {
        let iso = at.to_iso();
        let rec = recs.iter().find(|r| r.date == iso);
        days.push(BikriDay {
            date: iso,
            count: rec.map_or(0, |r| r.count),
            gross_paisa: Paisa::try_from_paisa(rec.map_or(0, |r| r.gross_paisa))
                .map_err(crate::error::money_error)?,
        });
        match at.next_day() {
            Some(next) if next <= last => at = next,
            _ => break,
        }
    }
    if descending {
        days.reverse();
    }
    Ok(days)
}

/// Every row of one prepared filter, newest first, read from an open snapshot in pages of `page`
/// rows: the first page also gives the count, the totals and the days; the next pages only add rows
/// (plan 72 P8). All pages are of the same moment, so a row, its day heading and the totals can never
/// disagree, whatever is saved meanwhile.
fn read_whole_in(
    conn: &Connection,
    prepared: &Prepared,
    page: u32,
) -> Result<BikriList, ServiceError> {
    let size = i64::from(page.max(1));
    let first = Prepared {
        filter: ListFilter {
            offset: 0,
            limit: size,
            ..prepared.filter.clone()
        },
        ..prepared.clone()
    };
    let mut whole = read_list(conn, &first)?;
    while i64::try_from(whole.rows.len()).unwrap_or(i64::MAX) < whole.total_count {
        let next = Prepared {
            filter: ListFilter {
                offset: i64::try_from(whole.rows.len()).unwrap_or(i64::MAX),
                limit: size,
                ..prepared.filter.clone()
            },
            group_by_month: false,
            group_by_day: false,
            ..prepared.clone()
        };
        let more = read_list(conn, &next)?;
        if more.rows.is_empty() {
            return Err(ServiceError::Data(
                "the list of بکری changed while it was read".to_owned(),
            ));
        }
        whole.rows.extend(more.rows);
    }
    Ok(whole)
}

/// Every row of today's section and of the window of پچھلی بکری, for a print or an Excel file (PRD
/// FR-12.46, FR-19.11, B31): the same query, filters, totals and days as `list`, so a print can never
/// differ from the screen. Both sections are read in one snapshot. The window ends before today when
/// today's section is read too.
///
/// # Errors
/// As `list` for the window (`ERR_REQUIRED` on `from` or `to`, the date codes, `ERR_DATE_IN_FUTURE`,
/// `ERR_DATE_RANGE`, `ERR_OUT_OF_RANGE`, `ERR_TEXT_TOO_LONG` on `search`); `ERR_DATE_RANGE` on `to`
/// when `with_today` is on and the window ends on or after today.
pub fn list_whole(
    ctx: &Context,
    input: &BikriListWholeInput,
) -> Result<BikriListWhole, ServiceError> {
    list_whole_paged(ctx, input, WHOLE_PAGE)
}

/// [`list_whole`] with the number of rows of each read set, so a test can cross a page boundary with
/// a few rows. The screen's commands use 500.
///
/// # Errors
/// As [`list_whole`].
pub fn list_whole_paged(
    ctx: &Context,
    input: &BikriListWholeInput,
    page: u32,
) -> Result<BikriListWhole, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let today_text = dates::format_ddmmyyyy(today);
    let base = |from: &str, to: &str| BikriListInput {
        from: Some(from.to_owned()),
        to: Some(to.to_owned()),
        seller_id: input.seller_id,
        item_id: input.item_id,
        status: input.status,
        show_finished: input.show_finished,
        search: input.search.clone(),
        all: input.all,
        group_by_day: true,
        ..BikriListInput::default()
    };
    let window_filter = prepare_list(today, &base(&input.from, &input.to))?;
    let today_section = if input.with_today {
        // Today's section and a window that reaches today would print today twice.
        if window_filter
            .filter
            .to
            .as_deref()
            .is_some_and(|to| to >= today.to_iso().as_str())
        {
            return Err(ServiceError::reject_field(ErrorCode::ErrDateRange, "to"));
        }
        Some(prepare_list(today, &base(&today_text, &today_text))?)
    } else {
        None
    };
    ctx.db.read_snapshot(|conn| {
        let window = read_whole_in(conn, &window_filter, page)?;
        let today_list = match &today_section {
            Some(prepared) => Some(read_whole_in(conn, prepared, page)?),
            None => None,
        };
        Ok(BikriListWhole {
            today_iso: today.to_iso(),
            today: today_list,
            window,
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
    // Typed text is checked before it becomes a pattern (V0, Code Standards 4.2).
    check_typed_length("truck_no", input.truck_no.as_deref())?;
    check_typed_length("buyer_text", input.buyer_text.as_deref())?;
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

/// The lots of saved بکری of these sellers and items, with this counter if given, newest day first, and
/// how many lots fit in all (PRD FR-12.45, FR-7.77, plan 71 L9). The assistant finds "ehsan aam-2" and
/// "ehsan ke aam" with it. Read only; at most `limit` lots, 100 at most.
///
/// # Errors
/// `ServiceError::Db`.
pub fn find_lots(
    ctx: &Context,
    input: &mandi_domain::core::bikri::LotFindInput,
) -> Result<(Vec<mandi_domain::core::bikri::LotFound>, i64), ServiceError> {
    let limit = input.limit.clamp(1, 100);
    ctx.db.read(|conn| {
        Ok(mandi_db::bikri::lots_named(
            conn,
            &input.seller_ids,
            &input.item_ids,
            input.counter,
            input.lot_id,
            limit,
        )?)
    })
}
