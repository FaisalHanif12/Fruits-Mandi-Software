//! The whole list of بکری for a print or an Excel file (PRD 12.7 پچھلی بکری, FR-12.46, FR-19.11, plan 72
//! P8). The screen reads its list one page at a time; a print or a sheet holds every row (Code Standards
//! B31), so the services read the window and today's section whole with these types and the reports
//! lay them out. Nothing is worked out here: every figure is a stored value that `bikri_list` sends.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::saved::{BikriList, BikriStatus};

/// What the print and the Excel file of پچھلی بکری take: the window and the filters of the screen
/// (the same meaning as `BikriListInput`), but no page and no sort. The rows are newest day first,
/// by day, as the screen shows them.
///
/// Today's section is the first part of the screen, so it is part of the print too (`with_today`).
/// The window then ends before today (PRD FR-12.46: it ends yesterday), else `ERR_DATE_RANGE` on
/// `to`: a day would be printed twice.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct BikriListWholeInput {
    /// First day of the window, `DD-MM-YYYY` or a shortcut. Required.
    pub from: String,
    /// Last day of the window. Required. At most 366 days from `from`.
    pub to: String,
    /// Also print today's section (the day of the clock), above the window. True when left out.
    #[serde(default = "default_with_today")]
    pub with_today: bool,
    /// Only بکری that have a lot of this seller.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub seller_id: Option<i64>,
    /// Only بکری that have a lot of this item.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// Only this status. When it is sent it decides, whatever `show_finished` says.
    #[serde(default)]
    pub status: Option<BikriStatus>,
    /// Show cancelled بکری too.
    #[serde(default)]
    pub show_finished: bool,
    /// A number or the start of a truck number.
    #[serde(default)]
    pub search: Option<String>,
    /// "پوری بکری دکھائیں": the filters above are set aside and every بکری of the days shows,
    /// cancelled ones too (plan 72 P8).
    #[serde(default)]
    pub all: bool,
}

fn default_with_today() -> bool {
    true
}

/// Every row of today's section and of the window, as `bikri_list` reads them, for a print or a
/// sheet. Read in the same way as the screen (the same filter, the same totals, the same days), so a
/// print can never differ from the screen. Not sent to the screens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BikriListWhole {
    /// The day of the clock, `YYYY-MM-DD`.
    pub today_iso: String,
    /// Today's section: every row of today, with one day heading. `None` when `with_today` is off.
    pub today: Option<BikriList>,
    /// The window: every row of its days, the totals over the whole filter and one heading for each
    /// day, empty days included, newest first.
    pub window: BikriList,
}
