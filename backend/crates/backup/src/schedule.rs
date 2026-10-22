//! What is due now (PRD FR-20.2, FR-20.3, FR-20.4, FR-20.6): a copy every 15 minutes while the
//! data changes, the daily USB and cloud copy at closing time, the retry of waiting cloud copies
//! every 10 minutes, and a USB copy when a drive is plugged in. The app calls [`BackupService::tick`]
//! about every 30 seconds from one background thread.

use chrono::{DateTime, Duration, Timelike, Utc};

use crate::api::RunReport;
use crate::error::BackupError;
use crate::run::{Scope, shop_day};
use crate::service::{BackupService, lock, usb_present};
use crate::settings::closing_minutes;

/// A local copy is made at most this often (FR-20.2).
pub const LOCAL_EVERY_MINUTES: i64 = 15;
/// The cloud queue is tried this often (FR-20.6, NFR-BAK-9).
pub const RETRY_EVERY_MINUTES: i64 = 10;
/// A daily run that did not get as far as a snapshot is tried again after this many minutes.
pub const DAILY_RETRY_MINUTES: i64 = 10;
/// A USB drive that is plugged in gets a copy if its last one is older than this (FR-20.4).
pub const USB_PLUG_HOURS: i64 = 12;

/// One thing the scheduler can start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Job {
    /// Try the cloud copies that wait.
    RetryQueue,
    /// The daily copy to every place (at closing time).
    Daily,
    /// The 15-minute local copy.
    Local,
    /// A copy to a USB drive that was just plugged in.
    UsbPlugged,
}

fn age(now: DateTime<Utc>, text: Option<&String>) -> Option<Duration> {
    let at = DateTime::parse_from_rfc3339(text?).ok()?;
    Some(now - at.with_timezone(&Utc))
}

impl BackupService {
    /// True when the data changed since the last copy. Unknown (the app just started) counts as
    /// changed.
    fn data_changed(&self) -> bool {
        match (self.db.data_version(), *lock(&self.data_seen)) {
            (Ok(now), Some(seen)) => now != seen,
            _ => true,
        }
    }

    /// The jobs that are due now, in the order they should run. Reads only.
    ///
    /// # Errors
    /// `ERR_INTERNAL` if the settings cannot be read.
    pub fn due_jobs(&self) -> Result<Vec<Job>, BackupError> {
        let settings = self.settings()?;
        let state = self.state_copy();
        let now = self.now();
        let shop_time = now + Duration::hours(5);
        let mut jobs = Vec::new();

        let queue_wait = age(now, state.last_queue_try.as_ref())
            .is_none_or(|a| a >= Duration::minutes(RETRY_EVERY_MINUTES));
        if settings.cloud_enabled && !state.queue.is_empty() && queue_wait {
            jobs.push(Job::RetryQueue);
        }

        let closing = closing_minutes(&settings.closing_time).unwrap_or(21 * 60);
        let minutes_now = shop_time.hour() * 60 + shop_time.minute();
        let today = shop_day(now);
        let daily_wanted = settings.usb_enabled || settings.cloud_enabled;
        let daily_wait = age(now, state.last_daily_try.as_ref())
            .is_none_or(|a| a >= Duration::minutes(DAILY_RETRY_MINUTES));
        if daily_wanted
            && minutes_now >= closing
            && state.last_daily_day.as_deref() != Some(today.as_str())
            && daily_wait
        {
            jobs.push(Job::Daily);
        } else {
            let local_wait = age(now, state.last_local_try.as_ref())
                .is_none_or(|a| a >= Duration::minutes(LOCAL_EVERY_MINUTES));
            if local_wait && self.data_changed() {
                jobs.push(Job::Local);
            }
        }

        let present = settings.usb_enabled && usb_present(&settings.usb_folder);
        let mut was = lock(&self.usb_was_present);
        let just_plugged = present && !*was;
        *was = present;
        let usb_old = age(now, state.last_usb.as_ref().map(|s| &s.at))
            .is_none_or(|a| a >= Duration::hours(USB_PLUG_HOURS));
        if just_plugged && usb_old && !jobs.contains(&Job::Daily) {
            jobs.push(Job::UsbPlugged);
        }
        Ok(jobs)
    }

    /// Runs what is due. Returns the reports of the runs. A busy service does nothing.
    ///
    /// # Errors
    /// `ERR_INTERNAL` if the settings cannot be read.
    pub fn tick(&self) -> Result<Vec<RunReport>, BackupError> {
        let mut reports = Vec::new();
        for job in self.due_jobs()? {
            match job {
                Job::RetryQueue => {
                    if let Err(error) = self.retry_queue() {
                        tracing::debug!("queue retry skipped: {error}");
                    }
                }
                Job::Daily => {
                    let at = self.now_text();
                    let today = shop_day(self.now());
                    self.update_state(|s| s.last_daily_try = Some(at));
                    if let Ok(report) = self.run(Scope::ALL) {
                        // A run that got as far as a snapshot is the day's run, whatever its
                        // outcome; a failed upload waits in the queue.
                        if report.snapshot_ms > 0
                            || report.check_ms > 0
                            || report.results.iter().any(|r| r.ok || r.queued)
                        {
                            self.update_state(|s| s.last_daily_day = Some(today));
                        }
                        reports.push(report);
                    }
                }
                Job::Local => {
                    if let Ok(report) = self.run(Scope::LOCAL_ONLY) {
                        reports.push(report);
                    }
                }
                Job::UsbPlugged => {
                    if let Ok(report) = self.run(Scope::USB_ONLY) {
                        reports.push(report);
                    }
                }
            }
        }
        Ok(reports)
    }
}
