//! How long copies are kept (NFR-BAK-8): local the last 48, USB the last 30, cloud 30 daily,
//! 12 monthly and 1 per year for ever. The rules are pure functions of file names (their time
//! is in the name), so they are easy to test and cannot touch a file they were not given.

use std::collections::BTreeSet;

use chrono::{DateTime, Datelike, Duration, Utc};

use crate::names::parse_stamp;

/// Local copies kept (NFR-BAK-8).
pub const KEEP_LOCAL: usize = 48;
/// USB copies kept (NFR-BAK-8).
pub const KEEP_USB: usize = 30;
/// Daily cloud copies kept.
pub const KEEP_CLOUD_DAILY: usize = 30;
/// Monthly cloud copies kept.
pub const KEEP_CLOUD_MONTHLY: usize = 12;

/// Pakistan time is UTC+5 (Code Standards G9): a copy made at 02:00 on the 11th belongs to the
/// 11th for the shop, even though it is the 10th in UTC.
fn shop_day(time: DateTime<Utc>) -> DateTime<Utc> {
    time + Duration::hours(5)
}

/// The names to delete so that only the newest `keep` remain. Names that carry no time in them
/// are never returned. The newest copy is never returned, whatever `keep` says.
pub fn newest_n_delete(names: &[String], keep: usize) -> Vec<String> {
    let mut dated: Vec<(DateTime<Utc>, &String)> = names
        .iter()
        .filter_map(|name| parse_stamp(name).map(|time| (time, name)))
        .collect();
    dated.sort();
    let keep = keep.max(1);
    let cut = dated.len().saturating_sub(keep);
    dated
        .into_iter()
        .take(cut)
        .map(|(_, name)| name.clone())
        .collect()
}

/// The cloud names to delete: keep the newest copy of each of the 30 newest days, the newest
/// copy of each of the 12 newest months, and the newest copy of every year. Everything else
/// goes. The newest copy is always kept.
pub fn cloud_delete(names: &[String]) -> Vec<String> {
    let mut dated: Vec<(DateTime<Utc>, &String)> = names
        .iter()
        .filter_map(|name| parse_stamp(name).map(|time| (time, name)))
        .collect();
    // Newest first.
    dated.sort_by(|a, b| b.cmp(a));

    let mut keep: BTreeSet<&String> = BTreeSet::new();
    let mut days: BTreeSet<(i32, u32, u32)> = BTreeSet::new();
    let mut months: BTreeSet<(i32, u32)> = BTreeSet::new();
    let mut years: BTreeSet<i32> = BTreeSet::new();
    for (time, name) in &dated {
        let local = shop_day(*time);
        let day = (local.year(), local.month(), local.day());
        let month = (local.year(), local.month());
        // The first copy seen of a period is the newest of it.
        if days.len() < KEEP_CLOUD_DAILY && days.insert(day) {
            keep.insert(name);
        }
        if months.len() < KEEP_CLOUD_MONTHLY && months.insert(month) {
            keep.insert(name);
        }
        if years.insert(local.year()) {
            keep.insert(name);
        }
    }
    dated
        .into_iter()
        .filter(|(_, name)| !keep.contains(name))
        .map(|(_, name)| name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::{bundle_name, local_name, stamp};
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }

    #[test]
    fn local_keeps_the_newest_48() {
        let start = at(2026, 10, 1, 0);
        let names: Vec<String> = (0..60)
            .map(|i| local_name(&stamp(start + Duration::minutes(15 * i))))
            .collect();
        let gone = newest_n_delete(&names, KEEP_LOCAL);
        assert_eq!(gone.len(), 12);
        assert_eq!(gone.first(), names.first());
        assert!(!gone.contains(names.last().unwrap()));
    }

    #[test]
    fn the_newest_copy_is_never_deleted() {
        let names = vec![local_name(&stamp(at(2026, 10, 1, 0)))];
        assert!(newest_n_delete(&names, 0).is_empty());
        assert!(cloud_delete(&[bundle_name(&stamp(at(2020, 1, 1, 0)))]).is_empty());
    }

    #[test]
    fn cloud_keeps_30_days_12_months_and_a_copy_for_every_year() {
        // One copy a day for 800 days up to 2026-10-10.
        let end = at(2026, 10, 10, 16);
        let names: Vec<String> = (0..800)
            .map(|i| bundle_name(&stamp(end - Duration::days(i))))
            .collect();
        let gone = cloud_delete(&names);
        let kept: Vec<&String> = names.iter().filter(|n| !gone.contains(n)).collect();
        // 30 days; the newest of each of the 12 newest months, which adds at most 12 more but
        // overlaps the days for this month; and one per year.
        assert!(
            kept.len() >= 30 && kept.len() <= 30 + 12 + 3,
            "{}",
            kept.len()
        );
        assert!(kept.contains(&names.first().unwrap()), "the newest is kept");
        // Every one of the last 30 days is there.
        for name in names.iter().take(30) {
            assert!(!gone.contains(name));
        }
        // The newest copy of each year in the data is kept: 2024, 2025, 2026.
        for year in [2024, 2025, 2026] {
            assert!(
                kept.iter().any(|n| parse_stamp(n).unwrap().year() == year),
                "{year}"
            );
        }
    }

    #[test]
    fn several_copies_in_one_day_keep_only_the_newest() {
        let day = at(2026, 10, 10, 0);
        let names: Vec<String> = (0..5)
            .map(|i| bundle_name(&stamp(day + Duration::hours(i))))
            .collect();
        let gone = cloud_delete(&names);
        assert_eq!(gone.len(), 4);
        assert!(!gone.contains(names.last().unwrap()));
    }

    #[test]
    fn names_without_a_time_are_left_alone() {
        let names = vec![
            "readme.txt".to_owned(),
            bundle_name(&stamp(at(2026, 10, 1, 0))),
        ];
        assert!(cloud_delete(&names).is_empty());
        assert!(newest_n_delete(&names, 1).is_empty());
    }
}
