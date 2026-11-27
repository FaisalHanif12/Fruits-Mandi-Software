//! The bounds of the windows of the list screens (plan 76 point 8, PRD FR-19.11): a screen with
//! many rows a day shows 7 days at a time, one with few rows a day a month at a time, and its
//! back button stops at the first day that has data. `windows_bounds` gives that first day.
//!
//! Example: expenses start on 10-10-2026, so the 1-month window goes back to October 2026.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::vouchers::VoucherType;

/// The list a window belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WindowSource {
    /// The بکری (پچھلی بکری): the day of the بکری.
    Sales,
    /// The روزنامچہ and every register that reads the journal.
    Journal,
    /// The روکڑ بک.
    Cashbook,
    /// The vouchers list, of one type when asked.
    Vouchers {
        /// The type, or null for all.
        #[serde(default)]
        voucher_type: Option<VoucherType>,
    },
    /// The پیشگی list (PRD FR-15.11, v1.51): the advances and the payments to a buyer (his advance
    /// is saved as cash lent, a payment voucher). Its first day is the first of either, so the back
    /// button does not stop after a day that has only a buyer's cash lent.
    Advances,
    /// The audit log.
    Audit,
    /// خرچہ: shop expenses, personal spending and its returns.
    Expenses,
    /// پھل: fruit taken.
    Fruit,
    /// ادھار رقم: money lent and its returns.
    Lent,
    /// The garden owners' and others' dues page (the same entries as `lent`).
    LentDues,
    /// الگ کھاتہ: money paid out and received, of one khata when asked.
    Khata {
        /// The khata account, or null for every khata.
        #[serde(default)]
        #[ts(type = "number | null", optional)]
        account_id: Option<i64>,
    },
    /// سرمایہ: capital in and out.
    Capital,
    /// A partner's money taken and returned.
    PartnerTakes {
        /// The partner, or null for the list of every partner (plan 98).
        #[serde(default)]
        #[ts(type = "number | null", optional)]
        partner_id: Option<i64>,
    },
    /// A partner's statement.
    PartnerStatement {
        /// The partner.
        #[ts(type = "number")]
        partner_id: i64,
    },
    /// A party's ledger.
    PartyLedger {
        /// The account.
        #[ts(type = "number")]
        account_id: i64,
    },
    /// The commission page.
    Commission,
    /// A garden's sale money list (PRD FR-12.53, plan 99).
    GardenSales {
        /// The garden (`ventures.id`).
        #[ts(type = "number")]
        garden_id: i64,
    },
    /// A partner's open gardens and trips (PRD FR-17.3, plan 99).
    PartnerVentures {
        /// The partner.
        #[ts(type = "number")]
        partner_id: i64,
    },
}

/// Input of `windows_bounds`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct WindowsBoundsInput {
    /// The list.
    pub source: WindowSource,
}

/// What `windows_bounds` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct WindowsBounds {
    /// The first day that has data in this list, `YYYY-MM-DD`; null when it has none yet. The back
    /// button of the window stops at this day.
    pub first_date: Option<String>,
    /// The last day a window may show: today, `YYYY-MM-DD`.
    pub last_date: String,
}
