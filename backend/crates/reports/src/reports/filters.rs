//! Reading and checking the filters of a report (PRD FR-19.1, FR-19.2, FR-19.10).
//!
//! The backend treats everything from the screen as untrusted (Code Standards V0): dates are
//! text, ids are looked up, and a filter the report does not take is refused. All problems are
//! returned together, each with its field (Code Standards 4.3).

use mandi_core::dates::{BusinessDate, parse_ddmmyyyy};
use mandi_core::error::{ErrorCode, FieldError};
use mandi_db::rusqlite::Connection;

use super::common::{MasterTable, load_party, names_of};
use super::model::{DateShortcut, FilterKind, GroupBy, PaidFilter, ReportFilters, ResolvedFilters};
use crate::error::ReportsError;

/// The filters of a run, read and checked. `resolved` is echoed back to the screen.
#[derive(Debug, Clone, Default)]
pub(crate) struct Typed {
    /// Today in Pakistan, from the database clock. Ages and "up to now" use it.
    pub(crate) today: Option<BusinessDate>,
    /// First day, included, `YYYY-MM-DD`.
    pub(crate) from: Option<String>,
    /// Last day, included, `YYYY-MM-DD`.
    pub(crate) to: Option<String>,
    /// The day of a balance list.
    pub(crate) as_of: Option<String>,
    /// The picked account.
    pub(crate) party_id: Option<i64>,
    /// The person inside a پختہ کھاتہ account.
    pub(crate) person_key: Option<String>,
    /// The picked city.
    pub(crate) city_id: Option<i64>,
    /// The picked seller.
    pub(crate) seller_id: Option<i64>,
    /// The picked زمیندار.
    pub(crate) zamindar_id: Option<i64>,
    /// The picked item.
    pub(crate) item_id: Option<i64>,
    /// The picked اڈا.
    pub(crate) adda_id: Option<i64>,
    /// The picked partner.
    pub(crate) partner_id: Option<i64>,
    /// How R28 groups its lines. Month when the screen sent nothing.
    pub(crate) group_by: GroupBy,
    /// The paid or due filter of R9.
    pub(crate) status: Option<PaidFilter>,
    /// The statement with its detail lines.
    pub(crate) detail: bool,
    /// What goes back to the screen.
    pub(crate) resolved: ResolvedFilters,
}

/// What a report takes.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Takes<'a> {
    /// The filters it shows.
    pub(crate) kinds: &'a [FilterKind],
    /// True when the party must be filled. For a report that takes a partner and no party
    /// (R31), it is the partner that must be filled.
    pub(crate) party_required: bool,
}

impl Takes<'_> {
    fn has(&self, kind: FilterKind) -> bool {
        self.kinds.contains(&kind)
    }
}

/// Reads a date typed by the user: the stored form `YYYY-MM-DD` or `DD-MM-YYYY` with the
/// shortcuts of PRD FR-7.19 (`10`, `10-9`).
fn parse_day(text: &str, today: BusinessDate) -> Result<BusinessDate, ErrorCode> {
    let text = text.trim();
    if let Some(day) = BusinessDate::parse_iso(text) {
        return Ok(day);
    }
    parse_ddmmyyyy(text, today)
}

/// The first day of the month of `day`.
fn first_of_month(day: BusinessDate) -> Option<BusinessDate> {
    BusinessDate::from_ymd(day.year(), day.month(), 1)
}

/// The days a shortcut stands for, from `today` (PRD FR-19.2). This week runs from Monday to
/// today [A]; this month from the 1st to today; last month is the whole of it.
fn shortcut_range(
    shortcut: DateShortcut,
    today: BusinessDate,
) -> Option<(BusinessDate, BusinessDate)> {
    match shortcut {
        DateShortcut::Today => Some((today, today)),
        DateShortcut::Yesterday => today.previous_day().map(|d| (d, d)),
        DateShortcut::ThisWeek => {
            let mut monday = today;
            for _ in 0..today.weekday_from_monday() {
                monday = monday.previous_day()?;
            }
            Some((monday, today))
        }
        DateShortcut::ThisMonth => first_of_month(today).map(|first| (first, today)),
        DateShortcut::LastMonth => {
            let first_this = first_of_month(today)?;
            let last = first_this.previous_day()?;
            first_of_month(last).map(|first| (first, last))
        }
    }
}

/// Reads and checks the filters of one report.
///
/// # Errors
/// `ReportsError::Rejected` with every problem and its field; a database error.
pub(crate) fn resolve(
    conn: &Connection,
    today: BusinessDate,
    takes: Takes<'_>,
    input: &ReportFilters,
) -> Result<Typed, ReportsError> {
    let mut problems: Vec<FieldError> = Vec::new();
    let mut typed = Typed {
        today: Some(today),
        ..Typed::default()
    };

    // A filter that was filled but is not one of this report's filters.
    let mut refuse_unless = |kind: FilterKind, filled: bool, field: &str| {
        if filled && !takes.has(kind) {
            problems.push(FieldError::new(ErrorCode::ErrOutOfRange, field));
        }
    };
    refuse_unless(
        FilterKind::DateRange,
        input.shortcut.is_some() || !input.from.trim().is_empty() || !input.to.trim().is_empty(),
        "filters.from",
    );
    refuse_unless(
        FilterKind::AsOf,
        !input.as_of.trim().is_empty(),
        "filters.as_of",
    );
    refuse_unless(FilterKind::Party, input.party.is_some(), "filters.party");
    refuse_unless(
        FilterKind::Seller,
        input.seller_id.is_some(),
        "filters.seller_id",
    );
    refuse_unless(
        FilterKind::Zamindar,
        input.zamindar_id.is_some(),
        "filters.zamindar_id",
    );
    refuse_unless(FilterKind::Item, input.item_id.is_some(), "filters.item_id");
    refuse_unless(FilterKind::City, input.city_id.is_some(), "filters.city_id");
    refuse_unless(FilterKind::Adda, input.adda_id.is_some(), "filters.adda_id");
    refuse_unless(
        FilterKind::Partner,
        input.partner_id.is_some(),
        "filters.partner_id",
    );
    refuse_unless(FilterKind::Detail, input.detail, "filters.detail");
    refuse_unless(
        FilterKind::GroupBy,
        input.group_by.is_some(),
        "filters.group_by",
    );
    refuse_unless(FilterKind::Status, input.status.is_some(), "filters.status");

    // Dates. A shortcut wins over typed dates.
    if takes.has(FilterKind::DateRange) {
        if let Some(shortcut) = input.shortcut {
            match shortcut_range(shortcut, today) {
                Some((from, to)) => {
                    typed.from = Some(from.to_iso());
                    typed.to = Some(to.to_iso());
                    typed.resolved.shortcut = Some(shortcut);
                }
                None => problems.push(FieldError::new(
                    ErrorCode::ErrDateInvalid,
                    "filters.shortcut",
                )),
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
            if let (Some(from), Some(to)) = (&typed.from, &typed.to) {
                // `YYYY-MM-DD` sorts as a date.
                if from > to {
                    problems.push(FieldError::new(ErrorCode::ErrDateRange, "filters.to"));
                }
            }
        }
    }
    if takes.has(FilterKind::AsOf) && !input.as_of.trim().is_empty() {
        match parse_day(&input.as_of, today) {
            Ok(day) => typed.as_of = Some(day.to_iso()),
            Err(code) => problems.push(FieldError::new(code, "filters.as_of")),
        }
    }

    // The picked records.
    if takes.has(FilterKind::Party) {
        match &input.party {
            Some(filter) => match load_party(conn, filter.account_id)? {
                Some(mut party) => {
                    let key = filter
                        .person_key
                        .as_deref()
                        .map(str::trim)
                        .filter(|k| !k.is_empty());
                    if key.is_some() && !is_pukhta(conn, party.account_id)? {
                        problems.push(FieldError::new(
                            ErrorCode::ErrOutOfRange,
                            "filters.party.person_key",
                        ));
                    }
                    typed.party_id = Some(party.account_id);
                    typed.person_key = key.map(str::to_owned);
                    party.person_key.clone_from(&typed.person_key);
                    typed.resolved.party = Some(party);
                }
                None => problems.push(FieldError::new(
                    ErrorCode::ErrNotFound,
                    "filters.party.account_id",
                )),
            },
            None if takes.party_required => {
                problems.push(FieldError::new(ErrorCode::ErrRequired, "filters.party"));
            }
            None => {}
        }
    }
    if takes.has(FilterKind::Seller) {
        typed.resolved.seller =
            picked_party(conn, input.seller_id, "filters.seller_id", &mut problems)?;
        if typed.resolved.seller.is_some() {
            typed.seller_id = input.seller_id;
        }
    }
    if takes.has(FilterKind::Zamindar) {
        typed.resolved.zamindar = picked_party(
            conn,
            input.zamindar_id,
            "filters.zamindar_id",
            &mut problems,
        )?;
        if typed.resolved.zamindar.is_some() {
            typed.zamindar_id = input.zamindar_id;
        }
    }
    if takes.has(FilterKind::Item) {
        typed.resolved.item = picked_names(
            conn,
            MasterTable::Items,
            input.item_id,
            "filters.item_id",
            &mut problems,
        )?;
        if typed.resolved.item.is_some() {
            typed.item_id = input.item_id;
        }
    }
    if takes.has(FilterKind::City) {
        typed.resolved.city = picked_names(
            conn,
            MasterTable::Cities,
            input.city_id,
            "filters.city_id",
            &mut problems,
        )?;
        if typed.resolved.city.is_some() {
            typed.city_id = input.city_id;
        }
    }
    if takes.has(FilterKind::Adda) {
        typed.resolved.adda = picked_names(
            conn,
            MasterTable::Addas,
            input.adda_id,
            "filters.adda_id",
            &mut problems,
        )?;
        if typed.resolved.adda.is_some() {
            typed.adda_id = input.adda_id;
        }
    }
    if takes.has(FilterKind::Partner) {
        typed.resolved.partner = picked_names(
            conn,
            MasterTable::Partners,
            input.partner_id,
            "filters.partner_id",
            &mut problems,
        )?;
        if typed.resolved.partner.is_some() {
            typed.partner_id = input.partner_id;
        } else if input.partner_id.is_none()
            && takes.party_required
            && !takes.has(FilterKind::Party)
        {
            problems.push(FieldError::new(
                ErrorCode::ErrRequired,
                "filters.partner_id",
            ));
        }
    }
    typed.detail = takes.has(FilterKind::Detail) && input.detail;
    if takes.has(FilterKind::GroupBy) {
        typed.group_by = input.group_by.unwrap_or(GroupBy::Month);
        typed.resolved.group_by = Some(typed.group_by);
    }
    if takes.has(FilterKind::Status) {
        typed.status = input.status;
        typed.resolved.status = input.status;
    }

    if !problems.is_empty() {
        return Err(ReportsError::Rejected(problems));
    }
    typed.resolved.from.clone_from(&typed.from);
    typed.resolved.to.clone_from(&typed.to);
    typed.resolved.as_of.clone_from(&typed.as_of);
    typed.resolved.detail = typed.detail;
    Ok(typed)
}

fn picked_party(
    conn: &Connection,
    id: Option<i64>,
    field: &str,
    problems: &mut Vec<FieldError>,
) -> Result<Option<super::model::PartyRef>, ReportsError> {
    let Some(id) = id else { return Ok(None) };
    let found = load_party(conn, id)?;
    if found.is_none() {
        problems.push(FieldError::new(ErrorCode::ErrNotFound, field));
    }
    Ok(found)
}

fn picked_names(
    conn: &Connection,
    table: MasterTable,
    id: Option<i64>,
    field: &str,
    problems: &mut Vec<FieldError>,
) -> Result<Option<super::model::Names>, ReportsError> {
    let Some(id) = id else { return Ok(None) };
    let found = names_of(conn, table, id)?;
    if found.is_none() {
        problems.push(FieldError::new(ErrorCode::ErrNotFound, field));
    }
    Ok(found)
}

/// True if the account is one of the two پختہ کھاتہ accounts (PRD FR-13.1).
fn is_pukhta(conn: &Connection, account_id: i64) -> Result<bool, ReportsError> {
    Ok(conn.query_row(
        "SELECT is_pukhta_khata FROM accounts WHERE id = ?1",
        [account_id],
        |r| r.get::<_, i64>(0),
    )? == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> BusinessDate {
        BusinessDate::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn shortcuts_of_fr_19_2() {
        // 14-10-2026 is a Wednesday.
        let today = day(2026, 10, 14);
        let range = |s| {
            let (a, b) = shortcut_range(s, today).unwrap();
            (a.to_iso(), b.to_iso())
        };
        assert_eq!(
            range(DateShortcut::Today),
            ("2026-10-14".into(), "2026-10-14".into())
        );
        assert_eq!(
            range(DateShortcut::Yesterday),
            ("2026-10-13".into(), "2026-10-13".into())
        );
        assert_eq!(
            range(DateShortcut::ThisWeek),
            ("2026-10-12".into(), "2026-10-14".into())
        );
        assert_eq!(
            range(DateShortcut::ThisMonth),
            ("2026-10-01".into(), "2026-10-14".into())
        );
        assert_eq!(
            range(DateShortcut::LastMonth),
            ("2026-09-01".into(), "2026-09-30".into())
        );
    }

    #[test]
    fn last_month_crosses_the_year() {
        let (a, b) = shortcut_range(DateShortcut::LastMonth, day(2027, 1, 5)).unwrap();
        assert_eq!(
            (a.to_iso(), b.to_iso()),
            ("2026-12-01".into(), "2026-12-31".into())
        );
    }

    #[test]
    fn this_week_on_a_monday_is_one_day() {
        let (a, b) = shortcut_range(DateShortcut::ThisWeek, day(2026, 10, 12)).unwrap();
        assert_eq!(
            (a.to_iso(), b.to_iso()),
            ("2026-10-12".into(), "2026-10-12".into())
        );
    }

    #[test]
    fn typed_dates_in_both_forms() {
        let today = day(2026, 10, 14);
        assert_eq!(
            parse_day("10-10-2026", today).unwrap().to_iso(),
            "2026-10-10"
        );
        assert_eq!(
            parse_day("2026-10-10", today).unwrap().to_iso(),
            "2026-10-10"
        );
        assert_eq!(parse_day("10", today).unwrap().to_iso(), "2026-10-10");
        assert_eq!(parse_day("9-9", today).unwrap().to_iso(), "2026-09-09");
        assert_eq!(
            parse_day("31-02-2026", today),
            Err(ErrorCode::ErrDateInvalid)
        );
        assert_eq!(parse_day("abc", today), Err(ErrorCode::ErrDateInvalid));
    }
}
