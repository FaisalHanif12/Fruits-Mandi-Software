//! The wire types of the party ledgers screen (پارٹی کھاتے, PRD FR-12.43, plan 66, D8 Feature 6).
//!
//! Three levels, all read only: a list of buyers or sellers with their balances; one party's
//! statement, one row for each date on which he has an entry; and the day detail, which is the
//! R35 day slip of that party and date (`reports_print_data` for R35 and its print route), so the
//! figures of the third level cannot differ from the slip.
//!
//! No money is worked out here. The services read the same posted journal lines as the party card
//! and R13 and do every sum and difference with checked maths.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::BalanceSide;
use crate::money::Paisa;

/// Which parties a ledger is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LedgerSide {
    /// Buyers (control 0304). The shared چکتا account and نقدی منڈی are not listed: they have no
    /// ledger (PRD FR-13.6).
    Buyer,
    /// Sellers (control 03011).
    Seller,
}

/// Input of `party_ledgers_list` (level 1).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartyLedgersListInput {
    /// Buyers or sellers.
    pub side: LedgerSide,
    /// Part of the code, the running number, or part of a name in either script (the search of the
    /// account list). Null or empty lists every party of the side.
    #[serde(default)]
    pub text: Option<String>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives the default page.
    #[serde(default)]
    pub limit: u32,
}

/// One party of the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartyLedgerRow {
    /// The account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The full code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag (نشان), or absent if none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
    /// His balance now, 0 or more. The same figure as his party card and the closing balance of
    /// R13.
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
    /// The side of the balance: `dr` is بنام (a buyer owes the shop), `cr` is جمع (the shop owes
    /// him).
    pub side: BalanceSide,
    /// False for an account made inactive.
    pub is_active: bool,
    /// True while he is in the dead amount (ڈوبی رقم, PRD FR-10.10). His balance is shown, marked.
    pub is_dead: bool,
}

/// A page of level 1, with the footer over the whole filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartyLedgersList {
    /// This page, by code.
    pub rows: Vec<PartyLedgerRow>,
    /// How many parties match.
    pub count: u32,
    /// The sum of the debit balances of every matching party (never summed from the page).
    #[ts(type = "Paisa")]
    pub total_dr_paisa: Paisa,
    /// The sum of the credit balances of every matching party.
    #[ts(type = "Paisa")]
    pub total_cr_paisa: Paisa,
}

/// Input of `party_ledgers_statement` (level 2).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartyLedgerStatementInput {
    /// The party.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The first date, `DD-MM-YYYY` (or a date shortcut), or null for the beginning.
    #[serde(default)]
    pub from: Option<String>,
    /// The last date, or null for today.
    #[serde(default)]
    pub to: Option<String>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives the default page.
    #[serde(default)]
    pub limit: u32,
}

/// Who a statement is about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartyLedgerHead {
    /// The account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The full code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag (نشان), or absent if none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
    /// Buyer or seller: it says what `charged` and `paid` mean (see [`PartyLedgerDay`]).
    pub kind: LedgerSide,
    /// False for an account made inactive.
    pub is_active: bool,
    /// True while he is in the dead amount.
    pub is_dead: bool,
}

/// One date of a statement.
///
/// For a buyer, `charged` is everything debited to him that day (his bill, cash lent) and `paid`
/// everything credited (his receipts, cash paid on the spot). For a seller, `charged` is everything
/// credited to him (his net sale, fruit taken from his lot, his returns of an advance) and `paid`
/// everything debited to him (نقد, ادائیگی, an advance); a lot with a negative net lands in `paid`.
/// These are the debit and credit columns of the R35 all-parties table, not the "paid today" line
/// of the R35 day slip, which keeps cash lent and journal entries in "other entries".
///
/// The two halves of a same-day edit or cancel cancel out and are left out, so a row shows what
/// stands. A row always adds up: the balance before the day, plus the debits of the day and minus
/// its credits, is the balance after it (a buyer's balance is debit minus credit; so is a seller's,
/// whose `cr` side means the shop owes him). The screen never works this out, it only shows the
/// sides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartyLedgerDay {
    /// The date, `YYYY-MM-DD`.
    pub date: String,
    /// His balance at the start of the day, 0 or more.
    #[ts(type = "Paisa")]
    pub previous_paisa: Paisa,
    /// The side of the previous balance.
    pub previous_side: BalanceSide,
    /// Charged that day.
    #[ts(type = "Paisa")]
    pub charged_paisa: Paisa,
    /// Paid that day.
    #[ts(type = "Paisa")]
    pub paid_paisa: Paisa,
    /// His balance after the day, 0 or more.
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
    /// The side of the balance after the day.
    pub balance_side: BalanceSide,
}

/// A page of level 2, newest date first, with the totals over the whole filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartyLedgerStatement {
    /// The party.
    pub head: PartyLedgerHead,
    /// This page: one row for each date with an entry, newest first.
    pub rows: Vec<PartyLedgerDay>,
    /// How many dates match the filter.
    pub count: u32,
    /// The sum of `charged` over every date that matches (never summed from the page).
    #[ts(type = "Paisa")]
    pub charged_total_paisa: Paisa,
    /// The sum of `paid` over every date that matches.
    #[ts(type = "Paisa")]
    pub paid_total_paisa: Paisa,
    /// His balance before the first date of the filter.
    #[ts(type = "Paisa")]
    pub opening_paisa: Paisa,
    /// The side of the opening balance.
    pub opening_side: BalanceSide,
    /// His balance at the end of the filter. With no `to` it is his balance now: the figure of his
    /// party card and the closing balance of R13.
    #[ts(type = "Paisa")]
    pub closing_paisa: Paisa,
    /// The side of the closing balance.
    pub closing_side: BalanceSide,
}
