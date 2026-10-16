//! Which charge rule wins, and what it comes to (PRD 11, Architecture 8, plan 09).
//!
//! Pure functions. The six levels of PRD 11.3, most specific first:
//!
//! | Level | Rule matches on | Example |
//! |---|---|---|
//! | 1 | typed on the line (commission and لاگا only, if the Owner allows) | 4% for this line |
//! | 2 | party and item | رحیم selling کیلا: 5% |
//! | 3 | party only | رحیم, any item: 4% |
//! | 4 | item only | جاپانی پھل: 30 per نگ |
//! | 5 | item group | all fruit: لاگا 8 per نگ |
//! | 6 | the default of the side and charge | seller 6%, buyer 6% |
//!
//! A rule applies on a day when it is active and the day is from its first day to its last day
//! (both included). Two rules that both apply at one level: the newer one wins, the one made
//! last (PRD FR-11.5). The party is the seller for a seller-side charge and the buyer for a
//! buyer-side charge (PRD 11.3).
//!
//! ```
//! use mandi_domain::core::dates::BusinessDate;
//! use mandi_domain::core::money::{Bps, Paisa, Qty};
//! use mandi_domain::core::rules::{ChargeSide, ChargeType, ChargeValue, ItemGroup};
//! use mandi_domain::rules::{Query, Rule, RuleSet, charge_amount, resolve};
//! // PRD 4.4: the default seller commission is 6%, so 12,000 gives 720.
//! let day = BusinessDate::parse_iso("2026-10-10").unwrap();
//! let default = Rule {
//!     id: 1,
//!     side: ChargeSide::Seller,
//!     charge: ChargeType::Commission,
//!     account_id: None,
//!     item_id: None,
//!     item_group: None,
//!     value: ChargeValue::Percent(Bps::new(600).unwrap()),
//!     from_date: day,
//!     to_date: None,
//!     is_active: true,
//! };
//! let set = RuleSet::build(vec![default]);
//! let found = resolve(&set, &Query {
//!     side: ChargeSide::Seller,
//!     charge: ChargeType::Commission,
//!     party_id: Some(7),
//!     item_id: 3,
//!     item_group: Some(ItemGroup::Fruit),
//!     date: day,
//!     line_override: None,
//! });
//! let value = Paisa::from_rupees(12_000).unwrap();
//! let charged = charge_amount(
//!     found.winner.as_ref(), ChargeType::Commission, value, Qty::new(20).unwrap(), true,
//! ).unwrap();
//! assert_eq!(charged.amount, Paisa::from_rupees(720).unwrap());
//! ```

use std::collections::HashMap;

use mandi_core::dates::BusinessDate;
use mandi_core::money::{MoneyError, Paisa, Qty, apply_bps, per_unit};
use mandi_core::rules::{
    ChargeSide, ChargeType, ChargeValue, ItemGroup, RuleLevel, SkipReason, SkippedLevel,
    WinnerView, Working, WorkingArg, WorkingValue, method_allowed,
};

/// One rule, as resolution sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Row id. A bigger id is a newer rule.
    pub id: i64,
    /// Whose side.
    pub side: ChargeSide,
    /// Which charge.
    pub charge: ChargeType,
    /// The party, or `None` for any party.
    pub account_id: Option<i64>,
    /// The item, or `None` for any item.
    pub item_id: Option<i64>,
    /// The item group, or `None` for any.
    pub item_group: Option<ItemGroup>,
    /// A percent or an amount.
    pub value: ChargeValue,
    /// The first day it applies.
    pub from_date: BusinessDate,
    /// The last day it applies, or `None` for no end.
    pub to_date: Option<BusinessDate>,
    /// False hides it from resolution.
    pub is_active: bool,
}

pub use mandi_core::rules::level_of;

impl Rule {
    /// The level this rule decides at.
    pub const fn level(&self) -> Option<RuleLevel> {
        level_of(self.account_id, self.item_id, self.item_group)
    }

    /// True if the charge may use this rule's method on this side (PRD 11.2). A row that breaks
    /// this (made outside the app) is never used.
    pub fn method_is_allowed(&self) -> bool {
        method_allowed(self.side, self.charge, self.value.method()) && value_is_sound(self.value)
    }

    /// True if the rule is active and the day is inside its dates.
    pub fn applies_on(&self, date: BusinessDate) -> bool {
        self.is_active && self.from_date <= date && self.to_date.is_none_or(|end| date <= end)
    }
}

/// The rules of one side and charge, filed by what they match on, so a lookup touches only the
/// few rules that could match (NFR-SCL-6: under 1 ms with 10,000 rules).
#[derive(Debug, Default, Clone)]
struct Shelf {
    party_item: HashMap<(i64, i64), Vec<usize>>,
    party: HashMap<i64, Vec<usize>>,
    item: HashMap<i64, Vec<usize>>,
    group: HashMap<ItemGroup, Vec<usize>>,
    default: Vec<usize>,
}

impl Shelf {
    /// Files the rule at `index` under what it matches on. Returns false, and files nothing, for
    /// a rule with no level (an item group together with a party or an item).
    fn file(&mut self, rule: &Rule, index: usize) -> bool {
        match (rule.account_id, rule.item_id, rule.item_group) {
            (Some(party), Some(item), None) => self
                .party_item
                .entry((party, item))
                .or_default()
                .push(index),
            (Some(party), None, None) => self.party.entry(party).or_default().push(index),
            (None, Some(item), None) => self.item.entry(item).or_default().push(index),
            (None, None, Some(group)) => self.group.entry(group).or_default().push(index),
            (None, None, None) => self.default.push(index),
            _ => return false,
        }
        true
    }

    /// The rules filed at one level that could match the query. A value typed on the line is
    /// never filed, so that level has none.
    fn candidates(&self, level: RuleLevel, query: &Query) -> Option<&Vec<usize>> {
        match level {
            RuleLevel::PartyItem => query
                .party_id
                .and_then(|party| self.party_item.get(&(party, query.item_id))),
            RuleLevel::Party => query.party_id.and_then(|party| self.party.get(&party)),
            RuleLevel::Item => self.item.get(&query.item_id),
            RuleLevel::ItemGroup => query.item_group.and_then(|g| self.group.get(&g)),
            RuleLevel::Default => Some(&self.default),
            RuleLevel::LineOverride => None,
        }
    }
}

/// All active rules, ready for [`resolve`]. Built once and used for many lookups.
#[derive(Debug, Default, Clone)]
pub struct RuleSet {
    rules: Vec<Rule>,
    shelves: HashMap<(ChargeSide, ChargeType), Shelf>,
}

impl RuleSet {
    /// Files the rules. Inactive rules and rules with no level (an item group together with a
    /// party or an item, which the rule screen refuses) are left out.
    pub fn build(rules: Vec<Rule>) -> RuleSet {
        let rules: Vec<Rule> = rules
            .into_iter()
            .filter(|r| r.is_active && r.level().is_some() && r.method_is_allowed())
            .collect();
        let mut shelves: HashMap<(ChargeSide, ChargeType), Shelf> = HashMap::new();
        for (index, rule) in rules.iter().enumerate() {
            // A rule with no level was filtered out above, so `file` always takes it here.
            shelves
                .entry((rule.side, rule.charge))
                .or_default()
                .file(rule, index);
        }
        RuleSet { rules, shelves }
    }

    /// How many active rules are filed.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// True if no rule is filed.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// The newest rule among `candidates` that applies on the day (see [`newer_key`]).
    fn newest(&self, candidates: Option<&Vec<usize>>, date: BusinessDate) -> Option<&Rule> {
        candidates?
            .iter()
            .filter_map(|i| self.rules.get(*i))
            .filter(|r| r.applies_on(date))
            .max_by_key(|r| newer_key(r))
    }
}

/// An amount is never negative. The parsers and the table refuse it; this is the last guard, so
/// a row made outside the app can never give a negative charge.
fn value_is_sound(value: ChargeValue) -> bool {
    match value {
        ChargeValue::Percent(_) => true,
        ChargeValue::PerUnit(amount) | ChargeValue::PerLot(amount) => !amount.is_negative(),
    }
}

/// Which of two rules at one level is the newer (PRD FR-11.5): the one that starts later; if
/// they start on the same day, the one made last (the bigger id). So a planned change ("7% from
/// 01-11") is not cancelled by a later entry that starts earlier. This is the one place that
/// decides "newer"; the rule screen's overlap warning uses the same order.
pub fn newer_key(rule: &Rule) -> (BusinessDate, i64) {
    (rule.from_date, rule.id)
}

/// A value typed on the line, which beats every rule (level 1, PRD 11.3 and FR-11.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineOverride {
    /// The percent or the amount typed.
    pub value: ChargeValue,
}

/// What to look up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Query {
    /// Whose side.
    pub side: ChargeSide,
    /// Which charge.
    pub charge: ChargeType,
    /// The party of the side: the seller for a seller-side charge, the buyer for a buyer-side
    /// charge. `None` if there is none.
    pub party_id: Option<i64>,
    /// The item.
    pub item_id: i64,
    /// The item's group.
    pub item_group: Option<ItemGroup>,
    /// The posting date.
    pub date: BusinessDate,
    /// A value typed on the line. The caller passes it only when the Owner allows overrides; it
    /// is used only for commission and لاگا (PRD FR-11.10).
    pub line_override: Option<LineOverride>,
}

/// The rule that won.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Winner {
    /// The level that decided.
    pub level: RuleLevel,
    /// The rule, or `None` for a value typed on the line.
    pub rule_id: Option<i64>,
    /// The percent or amount.
    pub value: ChargeValue,
    /// True for a value typed on the line (saved with the method "دستی").
    pub manual: bool,
}

impl Winner {
    /// The winner as the screens show it.
    pub fn view(&self) -> WinnerView {
        let (bps, paisa) = self.value.columns();
        WinnerView {
            level: self.level,
            rule_id: self.rule_id,
            method: self.value.method(),
            value_bps: bps,
            value_paisa: paisa.and_then(|p| Paisa::try_from_paisa(p).ok()),
            manual: self.manual,
        }
    }
}

/// What the lookup found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    /// The winner, or `None` if no rule applies.
    pub winner: Option<Winner>,
    /// The levels looked at before the winner (all six if there is none) that did not decide.
    pub skipped: Vec<SkippedLevel>,
}

/// Finds the rule that wins (PRD 11.3). Pure and never fails: no rule is a valid answer.
pub fn resolve(set: &RuleSet, query: &Query) -> Resolution {
    let mut skipped = Vec::new();
    let shelf = set.shelves.get(&(query.side, query.charge));
    for level in RuleLevel::ALL {
        let winner = match level {
            RuleLevel::LineOverride => query
                .line_override
                .filter(|_| matches!(query.charge, ChargeType::Commission | ChargeType::Laga))
                // A typed value follows PRD 11.2 like a rule: no per-lot on the buyer side.
                .filter(|line| {
                    method_allowed(query.side, query.charge, line.value.method())
                        && value_is_sound(line.value)
                })
                .map(|line| Winner {
                    level,
                    rule_id: None,
                    value: line.value,
                    manual: true,
                }),
            _ => shelf.and_then(|shelf| {
                let candidates = shelf.candidates(level, query);
                set.newest(candidates, query.date).map(|rule| Winner {
                    level,
                    rule_id: Some(rule.id),
                    value: rule.value,
                    manual: false,
                })
            }),
        };
        match winner {
            Some(winner) => {
                return Resolution {
                    winner: Some(winner),
                    skipped,
                };
            }
            None => skipped.push(SkippedLevel {
                level,
                reason: if level == RuleLevel::LineOverride {
                    SkipReason::NoOverride
                } else {
                    SkipReason::NoRule
                },
            }),
        }
    }
    Resolution {
        winner: None,
        skipped,
    }
}

/// An amount and how it was worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charged {
    /// The amount, whole rupees.
    pub amount: Paisa,
    /// The working. Its `result` is exactly `amount`.
    pub working: Working,
}

fn arg(name: &str, value: WorkingValue) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value,
    }
}

/// The amount of a charge for the winner on a line, and its working (PRD C2, C3, C4, FR-7.43).
///
/// - A percent charge is `round(base × percent)`, half up (`apply_bps`). `base` is the line
///   value: the seller amount for the seller's commission, the buyer value for the buyer's
///   commission and لاگا.
/// - A per-نگ charge is `qty × value`, exact. `qty` is the line's qty for a line charge and the
///   lot's qty received for a lot charge (labour, munshiana, phone).
/// - A per-lot charge is `value`, once: `per_lot_due` is true only for the first line of the
///   lot's first posting (PRD FR-11.9, FR-12.17). When it is false the amount is 0 and the working
///   is `working-per-lot-already-charged`.
/// - No winner: 0, and the working says so (the amount is typed on the lot, or there is none).
///
/// A value typed on the line (`manual`) is worked out the same way; its working message is
/// `working-manual-percent`, `working-manual-per-unit` or `working-manual-per-lot` and it
/// carries the same inputs.
///
/// # Errors
/// `MoneyError::Overflow` if the product does not fit.
pub fn charge_amount(
    winner: Option<&Winner>,
    charge: ChargeType,
    base: Paisa,
    qty: Qty,
    per_lot_due: bool,
) -> Result<Charged, MoneyError> {
    let Some(winner) = winner else {
        return Ok(Charged {
            amount: Paisa::ZERO,
            working: Working {
                message_id: "working-none".to_owned(),
                args: vec![arg("result", WorkingValue::Money { paisa: Paisa::ZERO })],
                rule_id: None,
            },
        });
    };
    let level = arg(
        "level",
        WorkingValue::Level {
            level: winner.level,
        },
    );
    let (amount, kind, mut args) = match winner.value {
        ChargeValue::Percent(bps) => (
            apply_bps(base, bps)?,
            "working-percent",
            vec![
                arg("base", WorkingValue::Money { paisa: base }),
                arg("bps", WorkingValue::Percent { bps: bps.get() }),
            ],
        ),
        ChargeValue::PerUnit(value) => (
            per_unit(qty, value)?,
            "working-per-unit",
            vec![
                // A lot charge counts the نگ received, not the line's.
                arg(
                    if matches!(
                        charge,
                        ChargeType::Labour | ChargeType::Munshiana | ChargeType::Phone
                    ) {
                        "qty_received"
                    } else {
                        "qty"
                    },
                    WorkingValue::Count { value: qty.get() },
                ),
                arg("value", WorkingValue::Money { paisa: value }),
            ],
        ),
        ChargeValue::PerLot(value) if per_lot_due => (
            value,
            "working-per-lot",
            vec![arg("value", WorkingValue::Money { paisa: value })],
        ),
        ChargeValue::PerLot(_) => (Paisa::ZERO, "working-per-lot-already-charged", Vec::new()),
    };
    args.push(arg("result", WorkingValue::Money { paisa: amount }));
    args.push(level);
    Ok(Charged {
        amount,
        working: Working {
            // A typed value says its method too: working-manual-percent, -per-unit, -per-lot.
            message_id: if winner.manual {
                kind.replacen("working-", "working-manual-", 1)
            } else {
                kind.to_owned()
            },
            args,
            rule_id: winner.rule_id,
        },
    })
}

#[cfg(test)]
mod tests;
