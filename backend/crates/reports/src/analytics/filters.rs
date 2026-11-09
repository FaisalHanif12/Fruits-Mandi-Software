//! Reading and checking the filters of an analytics run (PRD 18.1, FR-18.1).
//!
//! As everywhere, the backend treats the screen as untrusted (Code Standards V0): dates are
//! text, ids are looked up, a filter the view does not take is refused, and all problems are
//! returned together with their fields (Code Standards 4.3).

use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_db::rusqlite::{Connection, OptionalExtension};

use super::model::{
    AnalyticsCompare, AnalyticsFilters, AnalyticsGoods, AnalyticsGroupBy, AnalyticsPeriod,
    AnalyticsResolvedFilters, AnalyticsSide, AnalyticsTripView, AnalyticsView,
};
use crate::error::ReportsError;
use crate::reports::common::{MasterTable, load_party, names_of, parse_day};

/// The filters of a run, read and checked.
#[derive(Debug, Clone)]
pub(crate) struct Typed {
    /// First day, included, `YYYY-MM-DD`.
    pub(crate) from: Option<String>,
    /// Last day, included, `YYYY-MM-DD`.
    pub(crate) to: Option<String>,
    /// The columns of view D.
    pub(crate) group_by: AnalyticsGroupBy,
    /// The picked item.
    pub(crate) item_id: Option<i64>,
    /// The picked quality of the sale lines (view A).
    pub(crate) grade: Option<mandi_core::bikri::SaleGrade>,
    /// The picked item group: all its items (views A, B, C, D, G and H).
    pub(crate) item_group: Option<mandi_core::rules::ItemGroup>,
    /// The picked seller.
    pub(crate) seller_id: Option<i64>,
    /// The picked buyer account.
    pub(crate) buyer_id: Option<i64>,
    /// The person inside a چکتا کھاتہ account.
    pub(crate) person_key: Option<String>,
    /// The picked city (the buyer's).
    pub(crate) city_id: Option<i64>,
    /// The picked اڈا.
    pub(crate) adda_id: Option<i64>,
    /// Which commission is counted.
    pub(crate) side: AnalyticsSide,
    /// The picked partner (views F, G and J).
    pub(crate) partner_id: Option<i64>,
    /// The picked market trip.
    pub(crate) trip_id: Option<i64>,
    /// The picked garden.
    pub(crate) garden_id: Option<i64>,
    /// The account of the picked garden: the seller of its lots.
    pub(crate) garden_account_id: Option<i64>,
    /// Which goods are counted.
    pub(crate) goods: AnalyticsGoods,
    /// How view J lays out its rows.
    pub(crate) trip_view: AnalyticsTripView,
    /// The partner section is locked: leave out the partner columns (PRD FR-17.11).
    pub(crate) hide_partners: bool,
    /// The compared period, if a comparison was asked for.
    pub(crate) compare: Option<(String, String)>,
    /// The sort asked for, for the view that cuts its page early (view C).
    pub(crate) sort: Option<mandi_core::paging::SortInput>,
    /// What goes back to the screen.
    pub(crate) resolved: AnalyticsResolvedFilters,
}

fn first_of_month(day: BusinessDate) -> Option<BusinessDate> {
    BusinessDate::from_ymd(day.year(), day.month(), 1)
}

/// The days a period shortcut stands for (PRD 18.1). This week runs from Monday to today; this
/// month and this year from their first day to today; last month and last year are whole.
fn period_range(
    period: AnalyticsPeriod,
    today: BusinessDate,
) -> Option<(BusinessDate, BusinessDate)> {
    match period {
        AnalyticsPeriod::Today => Some((today, today)),
        AnalyticsPeriod::ThisWeek => {
            let mut monday = today;
            for _ in 0..today.weekday_from_monday() {
                monday = monday.previous_day()?;
            }
            Some((monday, today))
        }
        AnalyticsPeriod::ThisMonth => first_of_month(today).map(|first| (first, today)),
        AnalyticsPeriod::LastMonth => {
            let last = first_of_month(today)?.previous_day()?;
            first_of_month(last).map(|first| (first, last))
        }
        AnalyticsPeriod::ThisYear => BusinessDate::from_ymd(today.year(), 1, 1).map(|f| (f, today)),
        AnalyticsPeriod::LastYear => {
            let year = today.year().checked_sub(1)?;
            let first = BusinessDate::from_ymd(year, 1, 1)?;
            let last = BusinessDate::from_ymd(year, 12, 31)?;
            Some((first, last))
        }
    }
}

/// `day` moved back by `days` days.
fn back_days(day: BusinessDate, days: i64) -> Option<BusinessDate> {
    let mut found = day;
    for _ in 0..days {
        found = found.previous_day()?;
    }
    Some(found)
}

/// The same date one year earlier; 29 February becomes 28 February.
fn year_back(day: BusinessDate) -> Option<BusinessDate> {
    let year = day.year().checked_sub(1)?;
    BusinessDate::from_ymd(year, day.month(), day.day())
        .or_else(|| BusinessDate::from_ymd(year, day.month(), day.day().checked_sub(1)?))
}

/// The month a date is in, counted from year 0: the year times 12 plus the month minus 1.
fn month_index(day: BusinessDate) -> Option<i64> {
    i64::from(day.year())
        .checked_mul(12)?
        .checked_add(i64::from(day.month()))?
        .checked_sub(1)
}

/// How many whole calendar months a period covers, when it starts on the 1st of a month and ends
/// on the last day of a month. `None` for any other period.
///
/// Example: `01-10-26` to `31-10-26` is 1 month; `01-01-26` to `31-03-26` is 3; `02-10-26` to
/// `31-10-26` is not whole months.
fn whole_months(from: BusinessDate, to: BusinessDate) -> Option<i64> {
    if from.day() != 1 || to.next_day()?.day() != 1 {
        return None;
    }
    let months = month_index(to)?
        .checked_sub(month_index(from)?)?
        .checked_add(1)?;
    (months >= 1).then_some(months)
}

/// The compared period (PRD 18.1). Previous: for whole calendar months, the same number of whole
/// calendar months before (October is compared with September, not with the 31 days before it);
/// for any other period, the same number of days ending the day before `from`. Last year: the same
/// dates one year earlier.
fn compared(
    compare: AnalyticsCompare,
    from: BusinessDate,
    to: BusinessDate,
) -> Option<(BusinessDate, BusinessDate)> {
    match compare {
        AnalyticsCompare::Off => None,
        AnalyticsCompare::Previous => {
            let end = from.previous_day()?;
            if let Some(months) = whole_months(from, to) {
                let back = month_index(from)?.checked_sub(months)?;
                let year = i32::try_from(back.div_euclid(12)).ok()?;
                let month = u32::try_from(back.rem_euclid(12)).ok()?.checked_add(1)?;
                return Some((BusinessDate::from_ymd(year, month, 1)?, end));
            }
            let days = to.days_after(from).checked_add(1)?;
            let start = back_days(end, days.checked_sub(1)?)?;
            Some((start, end))
        }
        AnalyticsCompare::LastYear => Some((year_back(from)?, year_back(to)?)),
    }
}

/// The compared period of a period given as ISO dates (PRD 18.1), the way the engine works it out:
/// the whole calendar months before a whole-month period, else the same number of days ending the
/// day before `from`; or the same dates one year earlier. For the assistant, which must say what a
/// comparison covers before it reads it (plan 52 item 2). `None` for no comparison or a date that
/// does not exist.
///
/// Example: `10-10-26` to `10-10-26` compared with the previous period is `09-10-26` to `09-10-26`;
/// `01-09-26` to `30-09-26` is compared with `01-08-26` to `31-08-26`.
pub fn compare_window(compare: AnalyticsCompare, from: &str, to: &str) -> Option<(String, String)> {
    let (from, to) = (BusinessDate::parse_iso(from)?, BusinessDate::parse_iso(to)?);
    compared(compare, from, to).map(|(a, b)| (a.to_iso(), b.to_iso()))
}

/// True if the account is چکتا کھاتہ (0030) or the old closed account 0031 (PRD FR-13.1, FR-13.9).
fn is_chukta_khata(conn: &Connection, account_id: i64) -> Result<bool, ReportsError> {
    Ok(conn.query_row(
        "SELECT is_chukta_khata FROM accounts WHERE id = ?1",
        [account_id],
        |r| r.get::<_, i64>(0),
    )? == 1)
}

/// Reads and checks the filters of one view.
///
/// # Errors
/// `ReportsError::Rejected` with every problem and its field; a database error.
/// The views that take the item group (PRD v1.29): the item-wise A, the seller and buyer views B
/// and C, the item and month view D, the trend G and the top ten H.
pub(crate) fn takes_item_group(view: AnalyticsView) -> bool {
    matches!(
        view,
        AnalyticsView::A
            | AnalyticsView::B
            | AnalyticsView::C
            | AnalyticsView::D
            | AnalyticsView::G
            | AnalyticsView::H
    )
}

pub(crate) fn resolve(
    conn: &Connection,
    today: BusinessDate,
    view: AnalyticsView,
    input: &AnalyticsFilters,
) -> Result<Typed, ReportsError> {
    let mut problems: Vec<FieldError> = Vec::new();
    let mut typed = Typed {
        sort: None,
        from: None,
        to: None,
        group_by: AnalyticsGroupBy::Month,
        item_id: None,
        item_group: None,
        grade: None,
        seller_id: None,
        buyer_id: None,
        person_key: None,
        city_id: None,
        adda_id: None,
        side: input.side,
        partner_id: None,
        trip_id: None,
        garden_id: None,
        garden_account_id: None,
        goods: input.goods,
        trip_view: AnalyticsTripView::Trip,
        hide_partners: input.hide_partners,
        compare: None,
        resolved: AnalyticsResolvedFilters {
            side: input.side,
            compare: input.compare,
            goods: input.goods,
            ..AnalyticsResolvedFilters::default()
        },
    };

    // The grouping belongs to views D and G.
    if matches!(view, AnalyticsView::D | AnalyticsView::G) {
        typed.group_by = input.group_by.unwrap_or(AnalyticsGroupBy::Month);
        typed.resolved.group_by = Some(typed.group_by);
    } else if input.group_by.is_some() {
        problems.push(FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.group_by",
        ));
    }
    // The layout of the rows belongs to view J only.
    if view == AnalyticsView::J {
        typed.trip_view = input.trip_view.unwrap_or_default();
        typed.resolved.trip_view = Some(typed.trip_view);
    } else if input.trip_view.is_some() {
        problems.push(FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.trip_view",
        ));
    }
    // Views F, G and J work on periods and take no comparison; view F counts no sales, so it
    // takes none of the sale filters either (PRD 18.1).
    if matches!(
        view,
        AnalyticsView::F
            | AnalyticsView::G
            | AnalyticsView::I
            | AnalyticsView::J
            | AnalyticsView::K
    ) && input.compare != AnalyticsCompare::Off
    {
        problems.push(FieldError::new(ErrorCode::ErrOutOfRange, "filters.compare"));
    }
    if view == AnalyticsView::F {
        let sale_filters = [
            (input.item_id.is_some(), "filters.item_id"),
            (input.seller_id.is_some(), "filters.seller_id"),
            (input.buyer.is_some(), "filters.buyer"),
            (input.city_id.is_some(), "filters.city_id"),
            (input.adda_id.is_some(), "filters.adda_id"),
            (input.trip_id.is_some(), "filters.trip_id"),
            (input.goods != AnalyticsGoods::All, "filters.goods"),
            (input.side != AnalyticsSide::Both, "filters.side"),
        ];
        for (set, field) in sale_filters {
            if set {
                problems.push(FieldError::new(ErrorCode::ErrOutOfRange, field));
            }
        }
    }
    // A garden filter belongs to every view that counts sales; F counts none and J counts the
    // purchases of trips. View I is the gardens themselves: it takes no sale filter but the side
    // and no trip, and only garden goods.
    if matches!(view, AnalyticsView::F | AnalyticsView::J) && input.garden_id.is_some() {
        problems.push(FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.garden_id",
        ));
    }
    if view == AnalyticsView::I {
        let not_taken = [
            (input.seller_id.is_some(), "filters.seller_id"),
            (input.buyer.is_some(), "filters.buyer"),
            (input.city_id.is_some(), "filters.city_id"),
            (input.adda_id.is_some(), "filters.adda_id"),
            (input.trip_id.is_some(), "filters.trip_id"),
            (
                !matches!(input.goods, AnalyticsGoods::All | AnalyticsGoods::Garden),
                "filters.goods",
            ),
        ];
        for (set, field) in not_taken {
            if set {
                problems.push(FieldError::new(ErrorCode::ErrOutOfRange, field));
            }
        }
    }
    // View K counts fruit taken: no seller, buyer, city, اڈا or trip, no commission side, and no
    // trip goods (they are not offered as fruit, PRD FR-16.11).
    if view == AnalyticsView::K {
        let not_taken = [
            (input.seller_id.is_some(), "filters.seller_id"),
            (input.buyer.is_some(), "filters.buyer"),
            (input.city_id.is_some(), "filters.city_id"),
            (input.adda_id.is_some(), "filters.adda_id"),
            (input.trip_id.is_some(), "filters.trip_id"),
            (input.side != AnalyticsSide::Both, "filters.side"),
            (input.goods == AnalyticsGoods::Trip, "filters.goods"),
        ];
        for (set, field) in not_taken {
            if set {
                problems.push(FieldError::new(ErrorCode::ErrOutOfRange, field));
            }
        }
    }
    // View J counts purchases, which have no seller, buyer, city or اڈا.
    if view == AnalyticsView::J {
        let sale_filters = [
            (input.seller_id.is_some(), "filters.seller_id"),
            (input.buyer.is_some(), "filters.buyer"),
            (input.city_id.is_some(), "filters.city_id"),
            (input.adda_id.is_some(), "filters.adda_id"),
        ];
        for (set, field) in sale_filters {
            if set {
                problems.push(FieldError::new(ErrorCode::ErrOutOfRange, field));
            }
        }
    }
    // The partner belongs to views F, G, I and J; the trip and goods to every view but F.
    if input.partner_id.is_some()
        && !matches!(
            view,
            AnalyticsView::F
                | AnalyticsView::G
                | AnalyticsView::I
                | AnalyticsView::J
                | AnalyticsView::K
        )
    {
        problems.push(FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.partner_id",
        ));
    }

    // Dates. A period shortcut wins over typed dates.
    if let Some(period) = input.period {
        match period_range(period, today) {
            Some((from, to)) => {
                typed.from = Some(from.to_iso());
                typed.to = Some(to.to_iso());
                typed.resolved.period = Some(period);
            }
            None => problems.push(FieldError::new(ErrorCode::ErrDateInvalid, "filters.period")),
        }
    } else {
        let mut read = |text: &str, field: &str| -> Option<String> {
            if text.trim().is_empty() {
                return None;
            }
            match parse_day(text, today) {
                Ok(day) => Some(day.to_iso()),
                Err(code) => {
                    problems.push(FieldError::new(code, field));
                    None
                }
            }
        };
        typed.from = read(&input.from, "filters.from");
        typed.to = read(&input.to, "filters.to");
        if let (Some(from), Some(to)) = (&typed.from, &typed.to)
            && from > to
        {
            problems.push(FieldError::new(ErrorCode::ErrDateRange, "filters.to"));
        }
    }

    // The item group belongs to views A, B, C, D, G and H, and replaces one item (PRD v1.29).
    if let Some(group) = input.item_group {
        if !takes_item_group(view) || input.item_id.is_some() {
            problems.push(FieldError::new(
                ErrorCode::ErrOutOfRange,
                "filters.item_group",
            ));
        }
        typed.item_group = Some(group);
        typed.resolved.item_group = Some(group);
    }

    // The quality of the sale lines belongs to view A (PRD FR-12.42).
    if let Some(grade) = input.grade {
        if view != AnalyticsView::A {
            problems.push(FieldError::new(ErrorCode::ErrOutOfRange, "filters.grade"));
        }
        typed.grade = Some(grade);
        typed.resolved.grade = Some(grade);
    }

    // The comparison needs a closed period.
    if input.compare != AnalyticsCompare::Off {
        match (&typed.from, &typed.to) {
            (Some(from), Some(to)) => {
                if let (Some(from), Some(to)) =
                    (BusinessDate::parse_iso(from), BusinessDate::parse_iso(to))
                {
                    match compared(input.compare, from, to) {
                        Some((cf, ct)) => {
                            typed.compare = Some((cf.to_iso(), ct.to_iso()));
                            typed.resolved.compare_from = Some(cf.to_iso());
                            typed.resolved.compare_to = Some(ct.to_iso());
                        }
                        None => problems.push(FieldError::new(
                            ErrorCode::ErrDateInvalid,
                            "filters.compare",
                        )),
                    }
                }
            }
            (from, to) => {
                if from.is_none() {
                    problems.push(FieldError::new(ErrorCode::ErrRequired, "filters.from"));
                }
                if to.is_none() {
                    problems.push(FieldError::new(ErrorCode::ErrRequired, "filters.to"));
                }
            }
        }
    }

    // The picked records.
    if let Some(filter) = &input.buyer {
        match load_party(conn, filter.account_id)? {
            Some(mut party) => {
                let key = filter
                    .person_key
                    .as_deref()
                    .map(str::trim)
                    .filter(|k| !k.is_empty());
                if key.is_some() && !is_chukta_khata(conn, party.account_id)? {
                    problems.push(FieldError::new(
                        ErrorCode::ErrOutOfRange,
                        "filters.buyer.person_key",
                    ));
                }
                typed.buyer_id = Some(party.account_id);
                typed.person_key = key.map(str::to_owned);
                party.person_key.clone_from(&typed.person_key);
                typed.resolved.buyer = Some(party);
            }
            None => problems.push(FieldError::new(
                ErrorCode::ErrNotFound,
                "filters.buyer.account_id",
            )),
        }
    }
    if let Some(id) = input.seller_id {
        match load_party(conn, id)? {
            Some(party) => {
                typed.seller_id = Some(id);
                typed.resolved.seller = Some(party);
            }
            None => problems.push(FieldError::new(ErrorCode::ErrNotFound, "filters.seller_id")),
        }
    }
    if let Some(id) = input.partner_id {
        match names_of(conn, MasterTable::Partners, id)? {
            Some(names) => {
                typed.partner_id = Some(id);
                typed.resolved.partner = Some(names);
            }
            None => problems.push(FieldError::new(
                ErrorCode::ErrNotFound,
                "filters.partner_id",
            )),
        }
    }
    if let Some(id) = input.trip_id {
        let names = conn
            .query_row(
                "SELECT name_ur, name_en FROM ventures WHERE id = ?1 AND kind = 'trip'",
                [id],
                |r| {
                    Ok(crate::reports::Names {
                        ur: r.get(0)?,
                        en: r.get(1)?,
                    })
                },
            )
            .optional()?;
        match names {
            Some(names) => {
                typed.trip_id = Some(id);
                typed.resolved.trip = Some(names);
            }
            None => problems.push(FieldError::new(ErrorCode::ErrNotFound, "filters.trip_id")),
        }
    }
    if let Some(id) = input.garden_id {
        let found = conn
            .query_row(
                "SELECT name_ur, name_en, account_id FROM ventures WHERE id = ?1 AND kind = 'garden'",
                [id],
                |r| {
                    Ok((
                        crate::reports::Names {
                            ur: r.get(0)?,
                            en: r.get(1)?,
                        },
                        r.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()?;
        match found {
            Some((names, account)) => {
                typed.garden_id = Some(id);
                typed.garden_account_id = Some(account);
                typed.resolved.garden = Some(names);
            }
            None => problems.push(FieldError::new(ErrorCode::ErrNotFound, "filters.garden_id")),
        }
    }
    for (id, table, field) in [
        (input.item_id, MasterTable::Items, "filters.item_id"),
        (input.city_id, MasterTable::Cities, "filters.city_id"),
        (input.adda_id, MasterTable::Addas, "filters.adda_id"),
    ] {
        let Some(id) = id else { continue };
        match names_of(conn, table, id)? {
            Some(names) => match table {
                MasterTable::Items => {
                    typed.item_id = Some(id);
                    typed.resolved.item = Some(names);
                }
                MasterTable::Cities => {
                    typed.city_id = Some(id);
                    typed.resolved.city = Some(names);
                }
                MasterTable::Addas => {
                    typed.adda_id = Some(id);
                    typed.resolved.adda = Some(names);
                }
                // The partner is read above; no view has a collector.
                MasterTable::Partners | MasterTable::Staff => {}
            },
            None => problems.push(FieldError::new(ErrorCode::ErrNotFound, field)),
        }
    }

    if !problems.is_empty() {
        return Err(ReportsError::Rejected(problems));
    }
    typed.resolved.from.clone_from(&typed.from);
    typed.resolved.to.clone_from(&typed.to);
    Ok(typed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> BusinessDate {
        BusinessDate::from_ymd(y, m, d).unwrap()
    }

    fn range(period: AnalyticsPeriod, today: BusinessDate) -> (String, String) {
        let (a, b) = period_range(period, today).unwrap();
        (a.to_iso(), b.to_iso())
    }

    #[test]
    fn the_periods_of_prd_18_1() {
        // 14-10-2026 is a Wednesday.
        let today = day(2026, 10, 14);
        assert_eq!(
            range(AnalyticsPeriod::Today, today),
            ("2026-10-14".into(), "2026-10-14".into())
        );
        assert_eq!(
            range(AnalyticsPeriod::ThisWeek, today),
            ("2026-10-12".into(), "2026-10-14".into())
        );
        assert_eq!(
            range(AnalyticsPeriod::ThisMonth, today),
            ("2026-10-01".into(), "2026-10-14".into())
        );
        assert_eq!(
            range(AnalyticsPeriod::LastMonth, today),
            ("2026-09-01".into(), "2026-09-30".into())
        );
        assert_eq!(
            range(AnalyticsPeriod::ThisYear, today),
            ("2026-01-01".into(), "2026-10-14".into())
        );
        assert_eq!(
            range(AnalyticsPeriod::LastYear, today),
            ("2025-01-01".into(), "2025-12-31".into())
        );
        // Last month crosses the year.
        assert_eq!(
            range(AnalyticsPeriod::LastMonth, day(2027, 1, 5)),
            ("2026-12-01".into(), "2026-12-31".into())
        );
    }

    #[test]
    fn the_compared_window_as_iso_dates() {
        // The same number of days ending the day before, as the engine works it out.
        let window = |c, a: &str, b: &str| compare_window(c, a, b);
        assert_eq!(
            window(AnalyticsCompare::Previous, "2026-10-10", "2026-10-10"),
            Some(("2026-10-09".into(), "2026-10-09".into()))
        );
        assert_eq!(
            window(AnalyticsCompare::Previous, "2026-10-11", "2026-10-20"),
            Some(("2026-10-01".into(), "2026-10-10".into()))
        );
        assert_eq!(
            window(AnalyticsCompare::LastYear, "2026-10-01", "2026-10-14"),
            Some(("2025-10-01".into(), "2025-10-14".into()))
        );
        assert_eq!(
            window(AnalyticsCompare::Off, "2026-10-01", "2026-10-14"),
            None
        );
        // A date that does not exist gives no window.
        assert_eq!(
            window(AnalyticsCompare::Previous, "2026-02-30", "2026-03-01"),
            None
        );
    }

    #[test]
    fn whole_calendar_months_are_compared_with_the_months_before() {
        let previous = |from: (i32, u32, u32), to: (i32, u32, u32)| {
            compared(
                AnalyticsCompare::Previous,
                day(from.0, from.1, from.2),
                day(to.0, to.1, to.2),
            )
            .map(|(a, b)| (a.to_iso(), b.to_iso()))
        };
        let iso = |a: &str, b: &str| Some((a.to_owned(), b.to_owned()));
        // September 2026 against August (31 days) and not against 02-08 to 31-08.
        assert_eq!(
            previous((2026, 9, 1), (2026, 9, 30)),
            iso("2026-08-01", "2026-08-31")
        );
        // February of a common year (28 days) against January.
        assert_eq!(
            previous((2027, 2, 1), (2027, 2, 28)),
            iso("2027-01-01", "2027-01-31")
        );
        // January against the December before, over the year end.
        assert_eq!(
            previous((2027, 1, 1), (2027, 1, 31)),
            iso("2026-12-01", "2026-12-31")
        );
        // A quarter against the quarter before; a whole year against the year before.
        assert_eq!(
            previous((2026, 7, 1), (2026, 9, 30)),
            iso("2026-04-01", "2026-06-30")
        );
        assert_eq!(
            previous((2026, 1, 1), (2026, 12, 31)),
            iso("2025-01-01", "2025-12-31")
        );
        // Not whole months: the same number of days before, as always.
        assert_eq!(
            previous((2026, 10, 2), (2026, 10, 31)),
            iso("2026-09-02", "2026-10-01")
        );
        assert_eq!(
            previous((2026, 10, 1), (2026, 10, 20)),
            iso("2026-09-11", "2026-09-30")
        );
        // The month so far (this month) is not a whole month either.
        assert_eq!(
            previous((2026, 11, 1), (2026, 11, 3)),
            iso("2026-10-29", "2026-10-31")
        );
    }

    #[test]
    fn every_whole_month_of_ten_years_goes_back_to_the_month_before_it() {
        for year in 2020..=2030 {
            for month in 1..=12u32 {
                let first = day(year, month, 1);
                let last = first
                    .next_day()
                    .map(|_| {
                        let (y, m) = if month == 12 {
                            (year + 1, 1)
                        } else {
                            (year, month + 1)
                        };
                        day(y, m, 1).previous_day().unwrap()
                    })
                    .unwrap();
                let (a, b) = compared(AnalyticsCompare::Previous, first, last).unwrap();
                // It ends the day before the month starts and starts on the 1st of the month before.
                assert_eq!(b, first.previous_day().unwrap(), "{year}-{month}");
                assert_eq!(a.day(), 1, "{year}-{month}");
                let (py, pm) = if month == 1 {
                    (year - 1, 12)
                } else {
                    (year, month - 1)
                };
                assert_eq!((a.year(), a.month()), (py, pm), "{year}-{month}");
                assert_eq!(
                    b.year() * 12 + i32::try_from(b.month()).unwrap(),
                    py * 12 + i32::try_from(pm).unwrap()
                );
            }
        }
    }

    #[test]
    fn the_compared_periods() {
        // 10 days ending 20-10: the previous period is 10 days ending 09-10.
        let (a, b) = compared(
            AnalyticsCompare::Previous,
            day(2026, 10, 11),
            day(2026, 10, 20),
        )
        .unwrap();
        assert_eq!(
            (a.to_iso(), b.to_iso()),
            ("2026-10-01".into(), "2026-10-10".into())
        );
        // One day: the day before.
        let (a, b) = compared(
            AnalyticsCompare::Previous,
            day(2026, 10, 10),
            day(2026, 10, 10),
        )
        .unwrap();
        assert_eq!(
            (a.to_iso(), b.to_iso()),
            ("2026-10-09".into(), "2026-10-09".into())
        );
        // A whole calendar month is compared with the whole calendar month before it, not with
        // the 31 days before it (Q146 of the live run of 03-11-2026: September against August).
        let (a, b) = compared(
            AnalyticsCompare::Previous,
            day(2026, 10, 1),
            day(2026, 10, 31),
        )
        .unwrap();
        assert_eq!(
            (a.to_iso(), b.to_iso()),
            ("2026-09-01".into(), "2026-09-30".into())
        );
        // The same dates last year; 29 February becomes 28 February.
        let (a, b) = compared(
            AnalyticsCompare::LastYear,
            day(2028, 2, 29),
            day(2028, 3, 31),
        )
        .unwrap();
        assert_eq!(
            (a.to_iso(), b.to_iso()),
            ("2027-02-28".into(), "2027-03-31".into())
        );
        assert!(compared(AnalyticsCompare::Off, day(2026, 1, 1), day(2026, 1, 2)).is_none());
    }
}
