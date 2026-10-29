//! What an edit or a later sale knows about the posting it works on (PRD FR-12.17, FR-12.20,
//! FR-11.6, FR-11.9, plan 16).
//!
//! `calc_posting` is pure: the services read the stored rows and pass them in here. A line that
//! the user did not change keeps its stored method, value, rule and amount exactly; only changed
//! and new lines look the rules up again (by the posting date). A lot charge the user did not
//! change keeps its stored method, value and rule; when only the quantity received changes, a
//! per-نگ charge is worked out again from its stored value (C3).
//!
//! Example (PRD E2E-13b): Event 1 is saved with 6% buyer commission, the Owner changes the
//! default to 7%, then edits اکرم's rate. His line is changed, so it looks the rules up again and
//! gets 7%. ارشد's line is not changed, so it keeps its stored 6% and 1,044.

use std::collections::HashMap;

use mandi_core::bikri::{LotChargeMethod, PaymentType, StoredMethod, TypedLotCharge};
use mandi_core::money::Paisa;
use mandi_core::rules::{ChargeMethod, ChargeValue, RuleLevel};

use super::PostingKind;
use crate::rules::Winner;

/// What a `calc_posting` call is for (plan 16 contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostingTarget {
    /// The first posting of a new بکری.
    New,
    /// A later sale of goods left in this lot (PRD FR-12.17): no lot charges, no per-lot
    /// commission, and the quantity available is what the lot has left.
    LaterSale {
        /// The lot.
        lot_id: i64,
    },
    /// An edit of this posting (PRD FR-12.18): the numbers are worked out again, and the lines
    /// and lot charges that did not change keep their stored values.
    Edit {
        /// The posting (journal entry) being replaced.
        entry_id: i64,
    },
}

/// A commission or لاگا of a stored sale line (PRD FR-11.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCharge {
    /// The method kept with the line.
    pub method: StoredMethod,
    /// The percent, in basis points.
    pub value_bps: Option<i32>,
    /// The amount per نگ or per lot.
    pub value_paisa: Option<Paisa>,
    /// The amount, as stored.
    pub amount: Paisa,
    /// The rule that decided.
    pub rule_id: Option<i64>,
    /// The level of that rule, read from the rule's row (for the tag of the line). `None` for a
    /// value typed on the line or when no rule applied.
    pub level: Option<RuleLevel>,
}

impl StoredCharge {
    /// The value as the maths wants it, or `None` for a stored charge without a value (nothing
    /// applied, so the amount is 0).
    pub fn value(&self) -> Option<ChargeValue> {
        let method = match self.method {
            StoredMethod::Percent => ChargeMethod::Percent,
            StoredMethod::PerUnit => ChargeMethod::PerUnit,
            StoredMethod::PerLot => ChargeMethod::PerLot,
            // A typed value keeps the percent or the amount per نگ that was typed.
            StoredMethod::Manual => {
                if self.value_bps.is_some() {
                    ChargeMethod::Percent
                } else {
                    ChargeMethod::PerUnit
                }
            }
        };
        ChargeValue::from_columns(
            method,
            self.value_bps,
            self.value_paisa.map(Paisa::as_paisa),
        )
    }

    /// The value the user typed on the line to get this charge: only a manual charge was typed.
    /// An open_for_edit draft carries it, so an unchanged line sends the same value back.
    pub fn typed(&self) -> Option<ChargeValue> {
        if self.method == StoredMethod::Manual {
            self.value()
        } else {
            None
        }
    }

    /// The winner this charge came from, to work its amount and its working out again from its
    /// stored method and value. `None` if there is no value.
    pub fn winner(&self) -> Option<Winner> {
        let value = self.value()?;
        let manual = self.method == StoredMethod::Manual;
        Some(Winner {
            level: if manual {
                RuleLevel::LineOverride
            } else {
                self.level.unwrap_or(RuleLevel::Default)
            },
            rule_id: self.rule_id,
            value,
            manual,
        })
    }

    /// True if this is the charge of a per-lot commission that was actually charged (it carries
    /// the amount; the other lines of the lot have the same method and 0).
    pub fn carries_per_lot(&self) -> bool {
        self.method == StoredMethod::PerLot && self.amount > Paisa::ZERO
    }
}

/// A stored sale line, with what it needs to tell whether the user changed it (PRD FR-12.20).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSale {
    /// The lot the line belongs to. A line sent under another lot is a new line.
    pub lot_id: i64,
    /// The buyer account.
    pub buyer_account_id: i64,
    /// The one-time person's key, for a پختہ کھاتہ buyer.
    pub person_key: Option<String>,
    /// Quantity in نگ.
    pub qty: i64,
    /// Weight in grams, for a weight item.
    pub weight_grams: Option<i64>,
    /// The seller's rate.
    pub seller_rate: Paisa,
    /// The buyer's rate (the seller's when the rates did not differ).
    pub buyer_rate: Paisa,
    /// How the buyer paid.
    pub payment_type: PaymentType,
    /// The seller's commission.
    pub seller_comm: StoredCharge,
    /// The buyer's commission.
    pub buyer_comm: StoredCharge,
    /// لاگا.
    pub laga: StoredCharge,
    /// C21: the cost of the own goods the line sold, kept exactly (PRD FR-12.20). 0 on a line of
    /// any other lot.
    pub cost: Paisa,
}

/// A lot charge (مزدوری, منشیانہ, فون) as the lot stored it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredLotCharge {
    /// The method.
    pub method: LotChargeMethod,
    /// What the field shows: per نگ, per lot, or 0 for a manual one.
    pub value: Paisa,
    /// The amount charged for the lot.
    pub amount: Paisa,
    /// The rule that suggested it.
    pub rule_id: Option<i64>,
    /// The level of that rule.
    pub level: Option<RuleLevel>,
}

impl StoredLotCharge {
    /// What the user typed to get this charge. A charge a rule decided, and a charge with
    /// nothing in it, were not typed (`None`). A typed manual charge keeps its amount as value.
    pub fn typed(&self) -> Option<TypedLotCharge> {
        if self.rule_id.is_some() {
            return None;
        }
        match self.method {
            LotChargeMethod::Manual => (self.amount > Paisa::ZERO).then_some(TypedLotCharge {
                method: LotChargeMethod::Manual,
                value: self.amount,
            }),
            method => Some(TypedLotCharge {
                method,
                value: self.value,
            }),
        }
    }
}

/// The stored line of a lot that carried its per-lot seller commission in the first posting
/// (PRD FR-11.9). If an edit removes that line, the charge moves to the first remaining line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCarrier {
    /// The line that carried it.
    pub sale_id: i64,
    /// Its seller commission as stored (method per lot, its value and rule, the amount).
    pub charge: StoredCharge,
}

/// Who the lot belongs to and what it holds, as stored: the seller, the item and, for a one-time
/// seller, his person key. When an edit changes any of them, the lot is worked out as new:
/// nothing stored for the old seller or item is kept (money agent review of plan 16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredIdentity {
    /// The seller account.
    pub seller_account_id: i64,
    /// The item.
    pub item_id: i64,
    /// The person key of a one-time seller, `None` for any other seller.
    pub person_key: Option<String>,
    /// The purchase an own lot takes its goods from, `None` for any other lot.
    pub purchase_id: Option<i64>,
    /// The garden arrival a garden lot takes its goods from, `None` for any other lot.
    pub arrival_id: Option<i64>,
}

/// A stored lot, as an edit or a later sale needs to know it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StoredLot {
    /// Who the lot belongs to. `None` when the caller does not say (a later sale), which counts as
    /// unchanged.
    pub identity: Option<StoredIdentity>,
    /// The line that carries the per-lot seller commission of the posting being edited, if any.
    pub carrier: Option<StoredCarrier>,
    /// نگ of this lot that are NOT in the posting being worked on: sold in other postings in
    /// force, plus written off. What the lot can still give this posting is its quantity
    /// received less this (PRD FR-12.6).
    pub elsewhere_qty: i64,
    /// مزدوری of the first posting. `None` for a lot with no stored charge.
    pub labour: Option<StoredLotCharge>,
    /// منشیانہ of the first posting.
    pub munshiana: Option<StoredLotCharge>,
    /// فون of the first posting.
    pub phone: Option<StoredLotCharge>,
}

/// Everything stored about the posting a call works on. Empty for a new بکری.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPosting {
    /// Whether the posting is a first posting or a later sale. An edit works as its kind.
    pub kind: PostingKind,
    /// The stored lots, by lot id.
    pub lots: HashMap<i64, StoredLot>,
    /// The stored sale lines of the posting, by line id.
    pub sales: HashMap<i64, StoredSale>,
}

impl StoredPosting {
    /// Nothing stored: a new بکری.
    pub fn none() -> Self {
        StoredPosting {
            kind: PostingKind::First,
            lots: HashMap::new(),
            sales: HashMap::new(),
        }
    }
}

impl PostingTarget {
    /// The kind of posting this call works out: a later sale, or an edit of a later sale, charges
    /// no lot charge and no per-lot commission (PRD FR-12.17).
    pub fn kind(self, stored: &StoredPosting) -> PostingKind {
        match self {
            PostingTarget::New => PostingKind::First,
            PostingTarget::LaterSale { .. } => PostingKind::Later,
            PostingTarget::Edit { .. } => stored.kind,
        }
    }
}
