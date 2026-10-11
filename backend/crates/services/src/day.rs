//! The open-day model (PRD FR-6.6, plan 04 Section 4). Closing and reopening days come in D2.

use chrono::NaiveDate;
use mandi_db::rusqlite::OptionalExtension;

use crate::context::Context;
use crate::error::ServiceError;

/// The storage and display form of a business date.
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Today and the closed-up-to date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayStatusInfo {
    /// Today on this computer, `YYYY-MM-DD`.
    pub today: String,
    /// The last closed day. `None` until a day is closed.
    pub closed_up_to: Option<String>,
    /// True when today is after the closed-up-to date.
    pub today_open: bool,
}

/// Today and the day state. A posting can only be dated after the closed-up-to date and not
/// later than today (PRD FR-6.6).
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a date that is not `YYYY-MM-DD`.
pub fn status(ctx: &Context) -> Result<DayStatusInfo, ServiceError> {
    let today: NaiveDate = ctx.clock().local_today();
    let closed: Option<String> = ctx.db.read(|conn| {
        let value: Option<Option<String>> = conn
            .query_row(
                "SELECT closed_up_to_date FROM day_state WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok::<_, ServiceError>(value.flatten())
    })?;
    let closed_date = closed
        .as_deref()
        .map(|text| {
            NaiveDate::parse_from_str(text, DATE_FORMAT)
                .map_err(|e| ServiceError::Data(format!("closed_up_to_date {text}: {e}")))
        })
        .transpose()?;
    Ok(DayStatusInfo {
        today: today.format(DATE_FORMAT).to_string(),
        closed_up_to: closed,
        today_open: closed_date.is_none_or(|closed| today > closed),
    })
}
