//! The automatic day close (plan 19, PRD Section 21 "Day", FR-21.1): when the Owner has switched
//! "auto day close" on, the app closes today by itself at the closing time, by calling the very
//! same close function the screen uses (`day::close`). It adds no rule of its own to the close.
//!
//! A background thread looks every [`TICK`] and calls [`run_once`]. A run closes the days up to
//! today only when all of this holds, checked in this order:
//!
//! 1. the switch is on (`settings_day_get`, `auto_day_close`);
//! 2. today is still open (the "closed up to" day is before today);
//! 3. the shop time (Pakistan, UTC+5, like the backup scheduler) is at or after the closing time;
//! 4. the last try for this day was more than [`RETRY_AFTER_MINUTES`] ago (a close that failed is
//!    not tried again every five minutes);
//! 5. there are no unsaved drafts: the Owner is still typing something, so nothing is closed
//!    under his hands. This is checked again at the next tick.
//!
//! The close itself refuses a trial balance that does not balance (`ERR_TRIAL_NOT_BALANCED`) or a
//! day that cannot be closed; then nothing changes, the reason goes to the log, and the earlier
//! open days show on the attention list the next morning (FR-7.44). The close is written to the
//! audit log under the Owner, like one made on the screen.
//!
//! Example: closing time 21:00, auto close on, today 10-10-26 open, no drafts. At 21:05 shop time
//! the run closes up to 10-10-26. At 21:10 it finds today closed and does nothing.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Timelike, Utc};
use mandi_backup::day_settings;
use mandi_backup::settings::closing_minutes;
use mandi_services::auth::Actor;
use mandi_services::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::{Context, ServiceError, day};

use crate::state::AppState;

/// How often the thread looks at whether a close is due.
pub const TICK: Duration = Duration::from_secs(5 * 60);
/// How long after start-up the first look waits, so the data check and the backup scheduler can
/// start first.
const FIRST_DELAY: Duration = Duration::from_secs(120);
/// A close that failed is tried again for the same day after this many minutes.
pub const RETRY_AFTER_MINUTES: i64 = 60;
/// The closing time when the setting cannot be read (21:00, like the backup scheduler).
const DEFAULT_CLOSING_MINUTES: u32 = 21 * 60;
/// Pakistan time is UTC+5 (the offset of the backup scheduler and the database clock).
const SHOP_OFFSET_HOURS: i64 = 5;

/// What one run did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing was due: the switch is off, today is closed, or it is before the closing time.
    NotDue,
    /// A close failed a short time ago for this day; waiting before the next try.
    Waiting,
    /// There are unsaved drafts; nothing was closed. Looked at again at the next tick.
    SkippedDrafts,
    /// The days up to this day (`YYYY-MM-DD`) were closed.
    Closed(String),
    /// The close was refused or failed. The text is for the log only.
    Failed(String),
}

/// What the thread remembers between runs: the day of the last failed try and when it was.
#[derive(Debug, Default)]
pub struct Memory {
    failed: Option<(String, DateTime<Utc>)>,
}

impl Memory {
    /// True while a close that failed for this day (`YYYY-MM-DD`) is less than
    /// [`RETRY_AFTER_MINUTES`] old.
    pub fn waiting(&self, day: &str, now: DateTime<Utc>) -> bool {
        self.failed.as_ref().is_some_and(|(failed_day, at)| {
            failed_day == day && (now - *at) < chrono::Duration::minutes(RETRY_AFTER_MINUTES)
        })
    }

    /// Notes a close that failed for this day.
    pub fn note_failure(&mut self, day: &str, now: DateTime<Utc>) {
        self.failed = Some((day.to_owned(), now));
    }
}

/// True when the shop time has reached the closing time (both as minutes since midnight).
pub fn closing_time_reached(minutes_now: u32, closing: u32) -> bool {
    minutes_now >= closing
}

fn shop_minutes(now: DateTime<Utc>) -> u32 {
    let shop = now + chrono::Duration::hours(SHOP_OFFSET_HOURS);
    shop.hour() * 60 + shop.minute()
}

/// The Owner as the close needs him: the one user of the book. `None` before the first run.
fn owner(ctx: &Context) -> Result<Option<Actor>, ServiceError> {
    ctx.db().read(|conn| {
        let found = conn
            .query_row(
                "SELECT id, username, display_name FROM users WHERE is_active = 1
                 ORDER BY id LIMIT 1",
                [],
                |r| {
                    Ok(Actor {
                        user_id: r.get(0)?,
                        username: r.get(1)?,
                        display_name: r.get(2)?,
                    })
                },
            )
            .map(Some);
        match found {
            Ok(actor) => Ok::<_, ServiceError>(actor),
            Err(mandi_services::db::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    })
}

/// How many drafts are kept (PRD FR-12.10).
fn draft_count(ctx: &Context) -> Result<i64, ServiceError> {
    ctx.db().read(|conn| {
        Ok::<_, ServiceError>(conn.query_row("SELECT COUNT(*) FROM drafts", [], |r| r.get(0))?)
    })
}

/// One look at whether today is to be closed, and the close if so. Never panics and never leaves
/// a half close: the close is one transaction (`day::close`).
pub fn run_once(ctx: &Context, memory: &mut Memory) -> Outcome {
    match try_run(ctx, memory) {
        Ok(outcome) => outcome,
        Err(error) => Outcome::Failed(error.to_string()),
    }
}

fn try_run(ctx: &Context, memory: &mut Memory) -> Result<Outcome, ServiceError> {
    let settings = ctx
        .db()
        .read(day_settings::load)
        .map_err(|e| ServiceError::Data(format!("the Day settings: {e}")))?;
    if !settings.auto_day_close {
        return Ok(Outcome::NotDue);
    }
    let status = day::status(ctx)?;
    if !status.today_open {
        return Ok(Outcome::NotDue);
    }
    let now = ctx.now_utc();
    let closing = closing_minutes(&settings.closing_time).unwrap_or(DEFAULT_CLOSING_MINUTES);
    if !closing_time_reached(shop_minutes(now), closing) {
        return Ok(Outcome::NotDue);
    }
    if memory.waiting(&status.today, now) {
        return Ok(Outcome::Waiting);
    }
    if draft_count(ctx)? > 0 {
        return Ok(Outcome::SkippedDrafts);
    }
    let Some(actor) = owner(ctx)? else {
        return Ok(Outcome::NotDue);
    };
    let today = BusinessDate::parse_iso(&status.today)
        .ok_or_else(|| ServiceError::Data(format!("today {}", status.today)))?;
    let input = DayCloseInput {
        up_to: format_ddmmyyyy(today),
    };
    match day::close(ctx, &actor, &input) {
        Ok(_) => {
            memory.failed = None;
            Ok(Outcome::Closed(status.today))
        }
        Err(error) => {
            memory.note_failure(&status.today, now);
            Ok(Outcome::Failed(error.to_string()))
        }
    }
}

/// Starts the thread. It never stops the app, and a data file that did not open is skipped.
pub fn start<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("day-auto-close".to_owned())
        .spawn(move || {
            std::thread::sleep(FIRST_DELAY);
            let mut memory = Memory::default();
            loop {
                if let Ok(state) = AppState::of(&app)
                    && let Ok(ctx) = state.context()
                {
                    let ctx: Arc<Context> = ctx;
                    match run_once(&ctx, &mut memory) {
                        Outcome::Closed(day) => {
                            tracing::info!(day = %day, "the day was closed by itself");
                        }
                        Outcome::Failed(reason) => {
                            tracing::warn!("the automatic day close did not run: {reason}");
                        }
                        Outcome::SkippedDrafts => {
                            tracing::info!("the automatic day close waits: there are drafts");
                        }
                        Outcome::NotDue | Outcome::Waiting => {}
                    }
                }
                std::thread::sleep(TICK);
            }
        });
    if let Err(error) = spawned {
        tracing::warn!("the automatic day close thread did not start: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_closing_time_is_reached_at_the_minute() {
        assert!(!closing_time_reached(20 * 60 + 59, 21 * 60));
        assert!(closing_time_reached(21 * 60, 21 * 60));
        assert!(closing_time_reached(23 * 60 + 59, 21 * 60));
    }

    #[test]
    fn a_failed_close_is_not_tried_again_for_an_hour_but_a_new_day_is() {
        use chrono::TimeZone;
        let at = Utc.with_ymd_and_hms(2026, 10, 10, 16, 0, 0).unwrap();
        let mut memory = Memory::default();
        assert!(!memory.waiting("2026-10-10", at));
        memory.note_failure("2026-10-10", at);
        let later = |minutes: i64| at + chrono::Duration::minutes(minutes);
        assert!(memory.waiting("2026-10-10", later(1)));
        assert!(memory.waiting("2026-10-10", later(RETRY_AFTER_MINUTES - 1)));
        assert!(!memory.waiting("2026-10-10", later(RETRY_AFTER_MINUTES)));
        // Another day starts with a clean slate.
        assert!(!memory.waiting("2026-10-11", later(1)));
    }

    #[test]
    fn the_shop_time_is_five_hours_after_utc() {
        use chrono::TimeZone;
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 16, 5, 0).unwrap();
        assert_eq!(shop_minutes(now), 21 * 60 + 5);
        // Over midnight: 20:00 UTC is 01:00 the next shop day.
        let late = Utc.with_ymd_and_hms(2026, 10, 10, 20, 0, 0).unwrap();
        assert_eq!(shop_minutes(late), 60);
    }
}
