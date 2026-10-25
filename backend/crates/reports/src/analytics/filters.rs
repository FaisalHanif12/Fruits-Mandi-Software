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
    /// The picked seller.
    pub(crate) seller_id: Option<i64>,
    /// The picked buyer account.
    pub(crate) buyer_id: Option<i64>,
    /// The person inside a پختہ کھاتہ account.
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
    /// The compared period, if a comparison was asked for.
    pub(crate) compare: Option<(String, String)>,
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

/// The compared period (PRD 18.1): the same number of days ending the day before `from`, or
/// the same dates one year earlier.
fn compared(
    compare: AnalyticsCompare,
    from: BusinessDate,
    to: BusinessDate,
) -> Option<(BusinessDate, BusinessDate)> {
    match compare {
        AnalyticsCompare::Off => None,
        AnalyticsCompare::Previous => {
            let days = to.days_after(from).checked_add(1)?;
            let end = from.previous_day()?;
            let start = back_days(end, days.checked_sub(1)?)?;
            Some((start, end))
        }
        AnalyticsCompare::LastYear => Some((year_back(from)?, year_back(to)?)),
    }
}

/// True if the account is one of the two پختہ کھاتہ accounts (PRD FR-13.1).
fn is_pukhta(conn: &Connection, account_id: i64) -> Result<bool, ReportsError> {
    Ok(conn.query_row(
        "SELECT is_pukhta_khata FROM accounts WHERE id = ?1",
        [account_id],
        |r| r.get::<_, i64>(0),
    )? == 1)
}

/// Reads and checks the filters of one view.
///
/// # Errors
/// `ReportsError::Rejected` with every problem and its field; a database error.
pub(crate) fn resolve(
    conn: &Connection,
    today: BusinessDate,
    view: AnalyticsView,
    input: &AnalyticsFilters,
) -> Result<Typed, ReportsError> {
    let mut problems: Vec<FieldError> = Vec::new();
    let mut typed = Typed {
        from: None,
        to: None,
        group_by: AnalyticsGroupBy::Month,
        item_id: None,
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
        AnalyticsView::F | AnalyticsView::G | AnalyticsView::I | AnalyticsView::J
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
            AnalyticsView::F | AnalyticsView::G | AnalyticsView::I | AnalyticsView::J
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
                if key.is_some() && !is_pukhta(conn, party.account_id)? {
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
                // The partner is read above.
                MasterTable::Partners => {}
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
        // A whole month goes back over the month end.
        let (a, b) = compared(
            AnalyticsCompare::Previous,
            day(2026, 10, 1),
            day(2026, 10, 31),
        )
        .unwrap();
        assert_eq!(
            (a.to_iso(), b.to_iso()),
            ("2026-08-31".into(), "2026-09-30".into())
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
