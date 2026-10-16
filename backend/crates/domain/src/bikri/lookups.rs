//! What `calc_posting` is told about the world outside the draft (plan 12): the accounts, items
//! and masters the draft names, the saved rates, and the Owner's switches. The services read
//! them in one go (Code Standards B22) and pass them in, so the domain never touches the
//! database (Code Standards B1).

use std::collections::{HashMap, HashSet};

use mandi_core::bikri::RateHint;
use mandi_core::money::Paisa;
use mandi_core::rules::ItemGroup;

/// What kind of party an account is (PRD 10.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyKind {
    /// A seller (control 03011).
    Seller,
    /// A buyer (control 0304).
    Buyer,
    /// Any other account.
    Other,
}

/// The facts about an account that the checks and the maths need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountFacts {
    /// What kind of party it is.
    pub kind: PartyKind,
    /// True for the two پختہ کھاتہ accounts (one-time people, PRD Section 13).
    pub is_pukhta: bool,
    /// True for the nameless cash buyer نقدی منڈی (PRD 10.4): always cash.
    pub is_naqdi_mandi: bool,
    /// True for the اپنا مال account (PRD FR-12.26). A lot of own goods comes with plan 16b, so a
    /// بکری of plan 12 refuses it as a seller.
    pub is_own_goods: bool,
    /// False when hidden from pickers: a new posting cannot use it (PRD FR-10.2).
    pub is_active: bool,
    /// False means cash only (PRD 10.2 "ادھار کی اجازت").
    pub credit_allowed: bool,
    /// The credit limit, if one is set.
    pub credit_limit: Option<Paisa>,
    /// The account balance now, debit positive (a credit balance is negative), for the credit
    /// limit check (PRD FR-12.4).
    pub balance: Paisa,
}

/// The facts about an item that the maths needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemFacts {
    /// پھل or سبزی, for the group rules (PRD 11.3 level 5).
    pub group: ItemGroup,
    /// True for an item sold by weight (PRD E23).
    pub by_weight: bool,
    /// False when hidden from pickers.
    pub is_active: bool,
}

/// The saved sales of an item over the last 7 days, for the rate check (PRD FR-7.55): the sum of
/// the quantities and the sum of the seller amounts, with reversals taken out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemSales {
    /// Σ `sign × qty`.
    pub qty: i64,
    /// Σ `sign × seller amount`.
    pub amount: Paisa,
}

/// The Owner's switches that `calc_posting` obeys (PRD Section 21 "Overrides" and "Checks").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Switches {
    /// A commission or لاگا may be typed on a line (PRD FR-11.10).
    pub allow_line_override: bool,
    /// A credit limit that is exceeded blocks the save (an error); otherwise it only warns.
    pub credit_limit_blocks: bool,
    /// The rate check percent, or `None` when the check is off (PRD FR-7.55).
    pub rate_check_percent: Option<u32>,
}

/// Everything read from the database for one call, keyed by id.
#[derive(Debug, Clone, Default)]
pub struct Lookups {
    /// The accounts the draft names (header, sellers, buyers).
    pub accounts: HashMap<i64, AccountFacts>,
    /// The items the draft names.
    pub items: HashMap<i64, ItemFacts>,
    /// The packings that exist.
    pub packings: HashSet<i64>,
    /// The مارکہ rows that exist.
    pub markas: HashSet<i64>,
    /// The stores that exist.
    pub stores: HashSet<i64>,
    /// The truck stands that exist.
    pub addas: HashSet<i64>,
    /// The saved rate hint of each (item, packing) the draft uses: today's, else the last day.
    pub saved_hints: HashMap<(i64, i64), RateHint>,
    /// The saved sales of each item the draft uses, for the rate check.
    pub item_sales: HashMap<i64, ItemSales>,
}
