//! What `calc_posting` is told about the world outside the draft (plan 12): the accounts, items
//! and masters the draft names, the saved rates, and the Owner's switches. The services read
//! them in one go (Code Standards B22) and pass them in, so the domain never touches the
//! database (Code Standards B1).

use std::collections::{HashMap, HashSet};

use mandi_core::bikri::RateHint;
use mandi_core::money::Paisa;
use mandi_core::rules::ItemGroup;
use mandi_core::ventures::{GardenRef, TripRef};

use crate::ventures::VenturePartner;

/// What kind of party an account is (PRD 10.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyKind {
    /// A seller (control 03011).
    Seller,
    /// A buyer (control 0304).
    Buyer,
    /// A separate khata (control 0305, PRD FR-16.7). It is accepted only as the seller of a lot,
    /// and every seller-side default is 0 for such a lot.
    Khata,
    /// A garden's own account (control 0306, PRD FR-12.36). It is accepted only as the seller of a
    /// lot, and only while the garden is open (`ERR_GARDEN_CLOSED`). The normal seller-side defaults
    /// apply, and the lot has no نقد.
    Garden,
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
    /// True for the اپنا مال account (PRD FR-12.26). It is accepted only as the seller of a lot
    /// with a purchase; anywhere else it is refused with `ERR_OWN_GOODS_ACCOUNT_RESTRICTED`.
    pub is_own_goods: bool,
    /// True for the account of a closed garden (PRD FR-12.38): it takes no lot and no later sale.
    pub garden_closed: bool,
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

/// One partner of the trip a purchase belongs to: what the split of the seller credit needs (PRD
/// FR-12.26, C25) and the names the screen shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripPartnerFacts {
    /// His id, 0604 account and share of the trip. The source is not used for money that comes in.
    pub partner: VenturePartner,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
}

/// What `calc_posting` needs to know about a purchase of own goods (PRD FR-12.26, C21): its item
/// and packing, and the stock and the cost that the posting being worked out can take. In an
/// edit these are the figures with the posting's own lines added back, so the posting is worked
/// out against what it could take if it were not saved yet. It also names the trip and its
/// partners, because the seller credit of an own lot goes to them (C25).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchaseFacts {
    /// The number of the purchase (خ-1).
    pub number: i64,
    /// The day it was bought, `YYYY-MM-DD`. An own lot cannot be sold on an earlier day (PRD
    /// FR-12.24, FR-12.26).
    pub entry_date: String,
    /// Its trip.
    pub trip: TripRef,
    /// The partners of the trip with their shares, in the order the partners were created.
    pub partners: Vec<TripPartnerFacts>,
    /// The item of the purchase.
    pub item_id: i64,
    /// The packing of the purchase.
    pub packing_id: i64,
    /// نگ left in all: bought less sold less written off. The cost (C21) is worked out over these:
    /// the cost left belongs to ALL the نگ left, in both parts.
    pub qty_left: i64,
    /// نگ an own lot can take: the part for our shop that is left (PRD FR-12.41). At most
    /// `qty_left`; equal to it when nothing is for another shop. In an edit the posting's own lines
    /// are added back.
    pub our_left: i64,
    /// The cost of those نگ: total cost less the cost already released.
    pub cost_left: Paisa,
    /// True once the purchase is cancelled: nothing can be sold from it.
    pub cancelled: bool,
}

/// What `calc_posting` needs to know about a garden arrival that a garden lot is linked to (PRD
/// FR-12.40, plan 33 v1.1): its garden, item and packing, and the نگ the posting can take. In an
/// edit the نگ the posting being replaced took are added back. An arrival has no cost and posts
/// nothing, so no money is read from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrivalFacts {
    /// The number of the arrival, in its own series.
    pub number: i64,
    /// The day it arrived, `YYYY-MM-DD`. A lot cannot be sold on an earlier day.
    pub entry_date: String,
    /// Its garden, with the garden's account (the only seller a lot linked to it may have).
    pub garden: GardenRef,
    /// The item of the arrival.
    pub item_id: i64,
    /// The packing of the arrival.
    pub packing_id: i64,
    /// نگ the posting can take: arrived less sold less written off.
    pub qty_left: i64,
    /// True once the arrival is cancelled: nothing can be sold from it.
    pub cancelled: bool,
    /// The per-lot rule charges that another active lot of this arrival, outside the posting being
    /// worked on, already carries (plan 37). The lots of the draft itself are tracked by
    /// `calc_posting`.
    pub carried_elsewhere: ArrivalCarried,
}

/// A lot that took a per-lot charge of a goods-arrived entry: its بکری number (0 for the draft
/// being worked on) and its lot number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryLot {
    /// The number of its بکری, or 0 for a lot of the draft being worked on.
    pub bikri_number: i64,
    /// The lot number inside that بکری.
    pub lot_no: i64,
}

/// Which per-lot charges a goods-arrived entry already has on one of its lots, and on which lot
/// (PRD FR-12.40, plan 37). A charge is taken once per entry, on its first active lot; a later lot
/// carries 0 for a kind that is `Some` here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArrivalCarried {
    /// The per-lot seller commission (carried by a sale line of a lot).
    pub seller_commission: Option<EntryLot>,
    /// مزدوری per lot.
    pub labour: Option<EntryLot>,
    /// منشیانہ per lot.
    pub munshiana: Option<EntryLot>,
    /// فون per lot.
    pub phone: Option<EntryLot>,
}

impl ArrivalCarried {
    /// True if any kind is carried.
    pub fn any(&self) -> bool {
        self.seller_commission.is_some()
            || self.labour.is_some()
            || self.munshiana.is_some()
            || self.phone.is_some()
    }

    /// The kinds carried by this or by `other`; where both carry one, this one's lot is kept (the
    /// earlier lot).
    #[must_use]
    pub fn or(self, other: ArrivalCarried) -> ArrivalCarried {
        ArrivalCarried {
            seller_commission: self.seller_commission.or(other.seller_commission),
            labour: self.labour.or(other.labour),
            munshiana: self.munshiana.or(other.munshiana),
            phone: self.phone.or(other.phone),
        }
    }
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
    /// The purchases the lots of the draft name (PRD FR-12.26).
    pub purchases: HashMap<i64, PurchaseFacts>,
    /// The garden arrivals the lots of the draft name (PRD FR-12.40).
    pub arrivals: HashMap<i64, ArrivalFacts>,
}
