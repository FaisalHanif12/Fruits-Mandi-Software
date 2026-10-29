//! Charges as data (PRD 11, plan 09): the shapes of a rule, of the rule that wins, of the
//! working behind an amount, and of what the rule and settings screens send and read back.
//!
//! The choosing of the winning rule and the amounts are in `mandi_domain::rules`; the database
//! is in `mandi_db::rules`. This module only holds the shapes, so every layer agrees on them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::money::{Bps, Paisa};
use crate::paging::SortInput;

/// Whose side a charge is on (PRD 11.1 "جانب").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ChargeSide {
    /// بیوپاری: deducted from the seller.
    Seller,
    /// گاہک: added to the buyer's bill.
    Buyer,
}

impl ChargeSide {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            ChargeSide::Seller => "seller",
            ChargeSide::Buyer => "buyer",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<ChargeSide> {
        match text {
            "seller" => Some(ChargeSide::Seller),
            "buyer" => Some(ChargeSide::Buyer),
            _ => None,
        }
    }
}

/// Which charge (PRD 11.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ChargeType {
    /// کمیشن.
    Commission,
    /// لاگا.
    Laga,
    /// مزدوری.
    Labour,
    /// منشیانہ.
    Munshiana,
    /// فون.
    Phone,
}

impl ChargeType {
    /// Every charge, in the order the test screen lists them.
    pub const ALL: [ChargeType; 5] = [
        ChargeType::Commission,
        ChargeType::Laga,
        ChargeType::Labour,
        ChargeType::Munshiana,
        ChargeType::Phone,
    ];

    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            ChargeType::Commission => "commission",
            ChargeType::Laga => "laga",
            ChargeType::Labour => "labour",
            ChargeType::Munshiana => "munshiana",
            ChargeType::Phone => "phone",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<ChargeType> {
        ChargeType::ALL.into_iter().find(|c| c.as_str() == text)
    }
}

/// How a charge is worked out (PRD 11.2 "طریقہ").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ChargeMethod {
    /// فیصد: a percent of the line value.
    Percent,
    /// فی نگ: an amount for each نگ.
    PerUnit,
    /// فی لاٹ: an amount once for the lot.
    PerLot,
}

impl ChargeMethod {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            ChargeMethod::Percent => "percent",
            ChargeMethod::PerUnit => "per_unit",
            ChargeMethod::PerLot => "per_lot",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<ChargeMethod> {
        match text {
            "percent" => Some(ChargeMethod::Percent),
            "per_unit" => Some(ChargeMethod::PerUnit),
            "per_lot" => Some(ChargeMethod::PerLot),
            _ => None,
        }
    }
}

/// The group of an item (PRD 10.3 "گروپ").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ItemGroup {
    /// پھل.
    Fruit,
    /// سبزی.
    Vegetable,
}

impl ItemGroup {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            ItemGroup::Fruit => "fruit",
            ItemGroup::Vegetable => "vegetable",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<ItemGroup> {
        match text {
            "fruit" => Some(ItemGroup::Fruit),
            "vegetable" => Some(ItemGroup::Vegetable),
            _ => None,
        }
    }
}

/// Whether a charge may use a method, on a side (PRD 11.2). Only these combinations exist; the
/// rule screen never offers another.
///
/// ```
/// use mandi_core::rules::{ChargeMethod, ChargeSide, ChargeType, method_allowed};
/// assert!(method_allowed(ChargeSide::Seller, ChargeType::Commission, ChargeMethod::PerLot));
/// assert!(!method_allowed(ChargeSide::Buyer, ChargeType::Commission, ChargeMethod::PerLot));
/// assert!(!method_allowed(ChargeSide::Seller, ChargeType::Labour, ChargeMethod::Percent));
/// assert!(!method_allowed(ChargeSide::Buyer, ChargeType::Phone, ChargeMethod::PerUnit));
/// ```
pub const fn method_allowed(side: ChargeSide, charge: ChargeType, method: ChargeMethod) -> bool {
    match (side, charge) {
        (ChargeSide::Seller, ChargeType::Commission) => true,
        (ChargeSide::Seller, ChargeType::Labour | ChargeType::Munshiana | ChargeType::Phone) => {
            matches!(method, ChargeMethod::PerUnit | ChargeMethod::PerLot)
        }
        (ChargeSide::Buyer, ChargeType::Commission | ChargeType::Laga) => {
            matches!(method, ChargeMethod::Percent | ChargeMethod::PerUnit)
        }
        _ => false,
    }
}

/// Whether the side has this charge at all (PRD 11.2).
pub const fn charge_allowed(side: ChargeSide, charge: ChargeType) -> bool {
    matches!(
        (side, charge),
        (ChargeSide::Seller, ChargeType::Commission)
            | (ChargeSide::Seller, ChargeType::Labour)
            | (ChargeSide::Seller, ChargeType::Munshiana)
            | (ChargeSide::Seller, ChargeType::Phone)
            | (ChargeSide::Buyer, ChargeType::Commission)
            | (ChargeSide::Buyer, ChargeType::Laga)
    )
}

/// The value of a rule: a percent, or an amount per نگ or per lot. One value, never two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeValue {
    /// A percent of the line value, in basis points.
    Percent(Bps),
    /// An amount for each نگ.
    PerUnit(Paisa),
    /// An amount once for the lot.
    PerLot(Paisa),
}

impl ChargeValue {
    /// The method this value belongs to.
    pub const fn method(self) -> ChargeMethod {
        match self {
            ChargeValue::Percent(_) => ChargeMethod::Percent,
            ChargeValue::PerUnit(_) => ChargeMethod::PerUnit,
            ChargeValue::PerLot(_) => ChargeMethod::PerLot,
        }
    }

    /// The value from the method and the two stored columns (`value_bps`, `value_paisa`).
    /// `None` if the column the method needs is empty or the other one is filled.
    pub fn from_columns(
        method: ChargeMethod,
        value_bps: Option<i32>,
        value_paisa: Option<i64>,
    ) -> Option<ChargeValue> {
        match (method, value_bps, value_paisa) {
            (ChargeMethod::Percent, Some(bps), None) => {
                Bps::new(bps).ok().map(ChargeValue::Percent)
            }
            (ChargeMethod::PerUnit, None, Some(p)) => {
                Paisa::try_from_paisa(p).ok().map(ChargeValue::PerUnit)
            }
            (ChargeMethod::PerLot, None, Some(p)) => {
                Paisa::try_from_paisa(p).ok().map(ChargeValue::PerLot)
            }
            _ => None,
        }
    }

    /// The two stored columns: `(value_bps, value_paisa)`.
    pub fn columns(self) -> (Option<i32>, Option<i64>) {
        match self {
            ChargeValue::Percent(bps) => (Some(bps.get()), None),
            ChargeValue::PerUnit(p) | ChargeValue::PerLot(p) => (None, Some(p.as_paisa())),
        }
    }
}

/// The six levels of PRD 11.3, most specific first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RuleLevel {
    /// 1. Typed on the line (commission and لاگا only, and only if the Owner allows it).
    LineOverride,
    /// 2. A rule for the party and the item.
    PartyItem,
    /// 3. A rule for the party, any item.
    Party,
    /// 4. A rule for the item, any party.
    Item,
    /// 5. A rule for the item's group (پھل or سبزی).
    ItemGroup,
    /// 6. The global default of the side and charge.
    Default,
}

impl RuleLevel {
    /// Every level, most specific first.
    pub const ALL: [RuleLevel; 6] = [
        RuleLevel::LineOverride,
        RuleLevel::PartyItem,
        RuleLevel::Party,
        RuleLevel::Item,
        RuleLevel::ItemGroup,
        RuleLevel::Default,
    ];

    /// The number of PRD 11.3 (1 to 6).
    pub const fn number(self) -> u8 {
        match self {
            RuleLevel::LineOverride => 1,
            RuleLevel::PartyItem => 2,
            RuleLevel::Party => 3,
            RuleLevel::Item => 4,
            RuleLevel::ItemGroup => 5,
            RuleLevel::Default => 6,
        }
    }
}

/// The level a rule with these scopes decides at (levels 2 to 6), or `None` for a combination
/// PRD 11.3 has no level for: an item group together with a party or an item.
///
/// ```
/// use mandi_core::rules::{ItemGroup, RuleLevel, level_of};
/// assert_eq!(level_of(Some(1), Some(2), None), Some(RuleLevel::PartyItem));
/// assert_eq!(level_of(Some(1), None, None), Some(RuleLevel::Party));
/// assert_eq!(level_of(None, Some(2), None), Some(RuleLevel::Item));
/// assert_eq!(level_of(None, None, Some(ItemGroup::Fruit)), Some(RuleLevel::ItemGroup));
/// assert_eq!(level_of(None, None, None), Some(RuleLevel::Default));
/// assert_eq!(level_of(Some(1), None, Some(ItemGroup::Fruit)), None);
/// ```
pub const fn level_of(
    account_id: Option<i64>,
    item_id: Option<i64>,
    item_group: Option<ItemGroup>,
) -> Option<RuleLevel> {
    match (account_id, item_id, item_group) {
        (Some(_), Some(_), None) => Some(RuleLevel::PartyItem),
        (Some(_), None, None) => Some(RuleLevel::Party),
        (None, Some(_), None) => Some(RuleLevel::Item),
        (None, None, Some(_)) => Some(RuleLevel::ItemGroup),
        (None, None, None) => Some(RuleLevel::Default),
        _ => None,
    }
}

/// True if a rule with this scope is one of the three global defaults of PRD 4.2: the seller or
/// buyer commission with no party, item or group, or the buyer's لاگا for the fruit group. Over
/// time a default is a chain of dated rows (the Owner changes it in Settings, PRD 11.3); the
/// rule form never creates, edits or switches off a row of that chain.
///
/// ```
/// use mandi_core::rules::{ChargeSide, ChargeType, ItemGroup, is_default_scope};
/// let s = ChargeSide::Seller;
/// let b = ChargeSide::Buyer;
/// assert!(is_default_scope(s, ChargeType::Commission, None, None, None));
/// assert!(is_default_scope(b, ChargeType::Laga, None, None, Some(ItemGroup::Fruit)));
/// assert!(!is_default_scope(b, ChargeType::Laga, None, None, Some(ItemGroup::Vegetable)));
/// assert!(!is_default_scope(s, ChargeType::Labour, None, None, None));
/// assert!(!is_default_scope(s, ChargeType::Commission, Some(1), None, None));
/// ```
pub const fn is_default_scope(
    side: ChargeSide,
    charge: ChargeType,
    account_id: Option<i64>,
    item_id: Option<i64>,
    item_group: Option<ItemGroup>,
) -> bool {
    matches!(
        (side, charge, account_id, item_id, item_group),
        (_, ChargeType::Commission, None, None, None)
            | (
                ChargeSide::Buyer,
                ChargeType::Laga,
                None,
                None,
                Some(ItemGroup::Fruit)
            )
    )
}

/// Why a level did not decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// No active rule of this level applies to the party, item and date.
    NoRule,
    /// Nothing was typed on the line, or the charge or the Owner's setting does not allow it.
    NoOverride,
}

/// A level that was looked at before the winner and did not decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct SkippedLevel {
    /// The level.
    pub level: RuleLevel,
    /// Why it did not decide.
    pub reason: SkipReason,
}

/// The rule that won, as the screen shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct WinnerView {
    /// Its level.
    pub level: RuleLevel,
    /// The rule, or null for a value typed on the line.
    #[ts(type = "number | null")]
    pub rule_id: Option<i64>,
    /// How it is worked out.
    pub method: ChargeMethod,
    /// A percent in basis points, for method `percent`.
    #[ts(type = "number | null")]
    pub value_bps: Option<i32>,
    /// The amount, for method `per_unit` or `per_lot`.
    #[ts(type = "Paisa | null")]
    pub value_paisa: Option<Paisa>,
    /// True for a value typed on the line (saved with the method "دستی").
    pub manual: bool,
}

/// One input or the result of a working, ready for the screen to format. The screen formats
/// these; it never calculates (Code Standards F42, B30).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkingValue {
    /// An amount of money.
    Money {
        /// The amount.
        #[ts(type = "Paisa")]
        paisa: Paisa,
    },
    /// A percent in basis points (600 is 6%).
    Percent {
        /// Basis points.
        bps: i32,
    },
    /// A count of نگ.
    Count {
        /// The count.
        #[ts(type = "number")]
        value: i64,
    },
    /// The level that decided.
    Level {
        /// The level.
        level: RuleLevel,
    },
}

/// A named input of a working.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct WorkingArg {
    /// The name the Fluent message uses, for example `base`, `bps`, `qty`, `value`, `result`.
    pub name: String,
    /// Its value.
    pub value: WorkingValue,
}

/// How an amount was worked out (PRD FR-7.43, Code Standards B30). Built by the same function
/// that works the amount out, so the result in it is the stored amount.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Working {
    /// The Fluent message: `working-percent`, `working-per-unit`, `working-per-lot`,
    /// `working-manual` or `working-none`.
    pub message_id: String,
    /// The inputs and the result, named for the message.
    pub args: Vec<WorkingArg>,
    /// The rule that decided, or null (typed on the line, or no rule).
    #[ts(type = "number | null")]
    pub rule_id: Option<i64>,
}

// ---- What the rule screens send and read back ----

/// Everything the rule form sends: a new rule (`id` null) or an edit (`id` and the `version` read).
/// Money and dates are text; the backend checks every value.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct RuleInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Whose side.
    pub side: ChargeSide,
    /// Which charge.
    pub charge_type: ChargeType,
    /// The party, or null for any party. A seller on the seller side, a buyer on the buyer side.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub account_id: Option<i64>,
    /// The item, or null for any item.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// The item group, or null for any. Only when the item is empty and the party is empty.
    #[serde(default)]
    pub item_group: Option<ItemGroup>,
    /// How it is worked out. Must be allowed for the charge (PRD 11.2).
    pub method: ChargeMethod,
    /// A percent (`6`, `2.5`, up to 2 decimals, 0 to 100) for method `percent`; whole rupees for
    /// `per_unit` and `per_lot`.
    pub value: String,
    /// The first day it applies, `DD-MM-YYYY`.
    pub from_date: String,
    /// The last day it applies, `DD-MM-YYYY`, or empty for no end.
    pub to_date: String,
    /// False hides the rule from resolution.
    pub is_active: bool,
    /// A note, up to 500 characters.
    pub note: String,
}

/// A rule as the screens read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RuleRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Whose side.
    pub side: ChargeSide,
    /// Which charge.
    pub charge_type: ChargeType,
    /// The party, if any.
    pub account: Option<RecordRef>,
    /// The item, if any.
    pub item: Option<RecordRef>,
    /// The item group, if any.
    pub item_group: Option<ItemGroup>,
    /// The level this rule decides at (2 to 6).
    pub level: RuleLevel,
    /// True for the three global defaults (seller commission, buyer commission, لاگا on fruit).
    /// They are changed only in Settings, "Charges": the rule form must not edit or switch them
    /// off.
    pub is_default: bool,
    /// How it is worked out.
    pub method: ChargeMethod,
    /// A percent in basis points, for method `percent`.
    #[ts(type = "number | null")]
    pub value_bps: Option<i32>,
    /// The amount, for method `per_unit` and `per_lot`.
    #[ts(type = "Paisa | null")]
    pub value_paisa: Option<Paisa>,
    /// The value as the form takes it back: `6`, `2.5`, or whole rupees.
    pub value_text: String,
    /// The first day, `YYYY-MM-DD`.
    pub from_date: String,
    /// The last day, `YYYY-MM-DD`, or null for no end.
    pub to_date: Option<String>,
    /// False when hidden from resolution.
    pub is_active: bool,
    /// The note. Empty if none.
    pub note: String,
}

/// What the rule list is filtered by.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct RulesFilter {
    /// Only this side, or null for both.
    #[serde(default)]
    pub side: Option<ChargeSide>,
    /// Only this charge, or null for all.
    #[serde(default)]
    pub charge_type: Option<ChargeType>,
    /// Only rules for this party, or null for all.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub account_id: Option<i64>,
    /// Only rules for this item, or null for all.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// True to list the inactive rules too.
    pub include_inactive: bool,
}

/// The rule list.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct RulesListInput {
    /// The filter.
    pub filter: RulesFilter,
    /// How many rows to skip. 0 for the first page (PRD FR-19.11).
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives a page of 100 (Code Standards B31).
    #[serde(default)]
    pub limit: u32,
    /// Only rules whose party or item (code or name) or note holds this text. The counts are over
    /// these rules.
    #[serde(default)]
    pub search: Option<String>,
    /// The column to sort the whole filter by: `side`, `charge_type`, `account`, `item`,
    /// `item_group`, `level`, `method`, `from_date`, `to_date`, `is_active` or `note`. Null is the
    /// order of the rule list: by side and charge, then most specific level, then newest (plan 38
    /// R4). Inactive rules are left out unless `filter.include_inactive` is true.
    #[serde(default)]
    pub sort: Option<SortInput>,
}

/// One page of the rule list with the counts over the whole filter (PRD FR-19.11, B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RulesList {
    /// This page, in the order the rules are listed.
    pub rows: Vec<RuleRow>,
    /// How many rules match the filter.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The counts over every matching rule.
    pub totals: RulesTotals,
}

/// The counts of the rule list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RulesTotals {
    /// How many of the matching rules are active.
    #[ts(type = "number")]
    pub active_count: i64,
}

/// Ask for one rule.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct RulesGetInput {
    /// The rule.
    #[ts(type = "number")]
    pub id: i64,
}

/// Make a rule active or inactive.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct RulesSetActiveInput {
    /// The rule.
    #[ts(type = "number")]
    pub id: i64,
    /// The new state.
    pub is_active: bool,
}

/// Another active rule at the same level with dates that overlap (PRD FR-11.5). The newer rule
/// wins; the screen warns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RuleOverlap {
    /// The other rule.
    #[ts(type = "number")]
    pub id: i64,
    /// Its first day, `YYYY-MM-DD`.
    pub from_date: String,
    /// Its last day, or null for no end.
    pub to_date: Option<String>,
}

/// A rule saved, with the warnings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RuleSaved {
    /// The rule as saved.
    pub row: RuleRow,
    /// Other active rules of the same scope whose dates overlap this one. Empty if none, or if
    /// the rule is inactive.
    pub overlaps: Vec<RuleOverlap>,
}

/// "Test a rule" (PRD FR-11.8): which rule wins for a seller, a buyer and an item on a date, and
/// the amounts on a quantity and a rate. Nothing is saved.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct RuleTestInput {
    /// The seller.
    #[ts(type = "number")]
    pub seller_id: i64,
    /// The buyer.
    #[ts(type = "number")]
    pub buyer_id: i64,
    /// The item.
    #[ts(type = "number")]
    pub item_id: i64,
    /// The day the rules are looked up for, `DD-MM-YYYY`.
    pub date: String,
    /// The quantity in نگ, as text.
    pub qty: String,
    /// The rate for each نگ in whole rupees, as text.
    pub rate: String,
}

/// The result for one charge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ChargeTest {
    /// Whose side.
    pub side: ChargeSide,
    /// Which charge.
    pub charge_type: ChargeType,
    /// The winner, or null if no rule applies (the amount is then typed on the lot, or 0).
    pub winner: Option<WinnerView>,
    /// The levels looked at before the winner that did not decide.
    pub skipped: Vec<SkippedLevel>,
    /// The amount on the quantity and rate of the test. For a per-lot charge, the value once.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// How the amount was worked out.
    pub working: Working,
}

/// The test result: seller commission, seller lot charges, buyer commission and لاگا.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RuleTest {
    /// The value of the line, quantity times rate.
    #[ts(type = "Paisa")]
    pub line_value_paisa: Paisa,
    /// One entry for each allowed charge of PRD 11.2.
    pub charges: Vec<ChargeTest>,
}

// ---- Settings: global defaults and overrides ----

/// A default as the Charges tab shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DefaultCharge {
    /// The rule that holds it.
    #[ts(type = "number")]
    pub rule_id: i64,
    /// How it is worked out.
    pub method: ChargeMethod,
    /// The value as the form takes it: `6`, `2.5`, or whole rupees.
    pub value_text: String,
    /// The version of the rule, to send back.
    #[ts(type = "number")]
    pub version: i64,
    /// The first day this value applies, `YYYY-MM-DD`. A change made in Settings starts on the
    /// day it is made; the value before it ends the day before (PRD 11.3: the rule date is the
    /// posting date).
    pub from_date: String,
}

/// The three defaults of PRD 4.2 and Section 21 "Charges".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ChargesSettings {
    /// Seller commission, default 6%.
    pub seller_commission: DefaultCharge,
    /// Buyer commission, default 6%.
    pub buyer_commission: DefaultCharge,
    /// لاگا on fruit, default 8 per نگ.
    pub laga_fruit: DefaultCharge,
}

/// One default sent back.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DefaultChargeInput {
    /// The `version` read with the default.
    #[ts(type = "number")]
    pub version: i64,
    /// How it is worked out.
    pub method: ChargeMethod,
    /// A percent or whole rupees, as text.
    pub value: String,
}

/// The three defaults sent back.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ChargesInput {
    /// Seller commission.
    pub seller_commission: DefaultChargeInput,
    /// Buyer commission.
    pub buyer_commission: DefaultChargeInput,
    /// لاگا on fruit.
    pub laga_fruit: DefaultChargeInput,
}

/// What happens when a buyer goes over his credit limit (PRD A18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CreditLimitMode {
    /// A warning on the line.
    Warn,
    /// The sale cannot be saved.
    Block,
}

/// The switches of Section 21 "Overrides" and "Checks". They start as: a typed commission or
/// لاگا is allowed (PRD FR-11.10 v1.5), a credit limit warns (A18), and the rate check is on at
/// 30% (PRD FR-7.55, E8; plan 12 team default).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct OverridesSettings {
    /// Commission and لاگا may be typed on a sale line (saved with the method "دستی"). Starts on;
    /// the Owner can turn it off.
    pub allow_line_override: bool,
    /// Warn or block when a buyer is over his credit limit.
    pub credit_limit_mode: CreditLimitMode,
    /// Warn when a rate is far from the item's average of the last 7 days (PRD FR-7.55). Starts
    /// on. The warning never blocks a save.
    pub rate_check_enabled: bool,
    /// How far, in percent, a rate may be above or below the average before the warning shows
    /// (1 to 1,000; starts at 30).
    pub rate_check_percent: u32,
}

/// The rate check percent a new shop starts with (PRD FR-7.55: 30%).
pub const DEFAULT_RATE_CHECK_PERCENT: u32 = 30;
/// The least and most a rate check percent can be set to.
pub const RATE_CHECK_PERCENT_RANGE: std::ops::RangeInclusive<u32> = 1..=1_000;

/// A percent in basis points as the form takes it back: `600` is `6`, `250` is `2.5`, `1234`
/// is `12.34`. Display text only.
///
/// ```
/// use mandi_core::money::Bps;
/// use mandi_core::rules::percent_text;
/// assert_eq!(percent_text(Bps::new(600).unwrap()), "6");
/// assert_eq!(percent_text(Bps::new(250).unwrap()), "2.5");
/// assert_eq!(percent_text(Bps::new(1234).unwrap()), "12.34");
/// assert_eq!(percent_text(Bps::new(5).unwrap()), "0.05");
/// assert_eq!(percent_text(Bps::new(10_000).unwrap()), "100");
/// ```
pub fn percent_text(bps: Bps) -> String {
    let whole = bps.get().div_euclid(100);
    let fraction = bps.get().rem_euclid(100);
    match fraction {
        0 => whole.to_string(),
        f if f % 10 == 0 => format!("{whole}.{}", f.div_euclid(10)),
        f => format!("{whole}.{f:02}"),
    }
}

// ---- Checking what the rule form sends ----

use crate::dates::BusinessDate;
use crate::error::{ErrorCode, FieldError};
use crate::validate::{self, FieldErrors};

/// The value of a rule from its text, for the method (Code Standards 4.2): a percent is 0 to 100
/// with up to 2 decimals; an amount per نگ or per lot is a whole number of rupees, 0 or more. A
/// value of 0 is how the Owner says "no لاگا for this buyer" (lead decision 12-10-2026).
///
/// `field` is the field the errors point at (`value`, or `seller_commission.value` on the
/// Settings tab).
///
/// # Errors
/// The percent and money codes of the parsers (a negative amount is refused; 0 is allowed).
///
/// ```
/// use mandi_core::rules::{ChargeMethod, ChargeValue, rule_value};
/// assert!(matches!(rule_value("value", ChargeMethod::Percent, "2.5"), Ok(ChargeValue::Percent(_))));
/// assert!(rule_value("value", ChargeMethod::Percent, "101").is_err());
/// assert!(rule_value("value", ChargeMethod::PerUnit, "0").is_ok());
/// assert!(rule_value("value", ChargeMethod::PerUnit, "-1").is_err());
/// assert!(rule_value("value", ChargeMethod::PerLot, "0").is_ok());
/// ```
pub fn rule_value(
    field: &str,
    method: ChargeMethod,
    text: &str,
) -> Result<ChargeValue, FieldError> {
    match method {
        // `validate::percent` adds the `max` value the message needs (finding 13).
        ChargeMethod::Percent => validate::percent(field, text).map(ChargeValue::Percent),
        ChargeMethod::PerUnit => validate::money(field, text).map(ChargeValue::PerUnit),
        ChargeMethod::PerLot => validate::money(field, text).map(ChargeValue::PerLot),
    }
}

/// Whether a side may have this charge with this method at all (PRD 11.2). The error says which
/// part is wrong: the charge (`charge_type`) or the method (`method`).
///
/// # Errors
/// `ERR_RULE_NOT_ALLOWED` on `charge_type` or `method` (with the field prefix, if any).
pub fn check_combination(
    prefix: &str,
    side: ChargeSide,
    charge: ChargeType,
    method: ChargeMethod,
) -> Result<(), FieldError> {
    if !charge_allowed(side, charge) {
        return Err(FieldError::new(
            ErrorCode::ErrRuleNotAllowed,
            &format!("{prefix}charge_type"),
        ));
    }
    if !method_allowed(side, charge, method) {
        return Err(FieldError::new(
            ErrorCode::ErrRuleNotAllowed,
            &format!("{prefix}method"),
        ));
    }
    Ok(())
}

/// A rule form that passed every check that needs no database.
#[derive(Debug, Clone)]
pub struct PreparedRule {
    /// The row, for an edit.
    pub id: Option<i64>,
    /// The version read, for an edit.
    pub version: Option<i64>,
    /// Whose side.
    pub side: ChargeSide,
    /// Which charge.
    pub charge: ChargeType,
    /// The party, if any. Its side is checked against the database.
    pub account_id: Option<i64>,
    /// The item, if any. It is checked against the database.
    pub item_id: Option<i64>,
    /// The item group, if any.
    pub item_group: Option<ItemGroup>,
    /// The percent or amount.
    pub value: ChargeValue,
    /// The first day.
    pub from_date: BusinessDate,
    /// The last day, if any.
    pub to_date: Option<BusinessDate>,
    /// Active.
    pub is_active: bool,
    /// The note, if any.
    pub note: Option<String>,
}

/// Checks everything about a rule form that needs no database (PRD 11.1, 11.2, 11.3,
/// Code Standards 4): the charge and method are allowed together, a group is for a rule with
/// no item and no party, the value fits the method, the dates are right. All mistakes are
/// returned together.
///
/// # Errors
/// Every wrong field, each with its code and field.
pub fn prepare_rule(
    input: &RuleInput,
    today: BusinessDate,
) -> Result<PreparedRule, Vec<FieldError>> {
    let mut errors = FieldErrors::default();
    let version = match (input.id, input.version) {
        (Some(_), None) => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, "version"));
            None
        }
        (Some(_), Some(v)) => Some(v),
        (None, _) => None,
    };
    let combination = check_combination("", input.side, input.charge_type, input.method);
    let combination_ok = combination.is_ok();
    if let Err(e) = combination {
        errors.push(e);
    }
    // A group rule is for any party and any item (PRD 11.1: only when the item is blank; PRD
    // 11.3 has no level for a party together with a group).
    if input.item_group.is_some() && (input.item_id.is_some() || input.account_id.is_some()) {
        errors.push(FieldError::new(ErrorCode::ErrRuleNotAllowed, "item_group"));
    }
    let value = if combination_ok {
        errors.take(rule_value("value", input.method, &input.value))
    } else {
        None
    };
    let from_date = errors.take(validate::date("from_date", &input.from_date, today));
    let to_date = if input.to_date.trim().is_empty() {
        None
    } else {
        errors.take(validate::date("to_date", &input.to_date, today))
    };
    if let (Some(from), Some(to)) = (from_date, to_date)
        && to < from
    {
        errors.push(FieldError::new(ErrorCode::ErrDateRange, "to_date"));
    }
    let note = errors
        .take(validate::free_text("note", &input.note))
        .filter(|n| !n.is_empty());
    errors.finish()?;
    // With no error above, both are present; the fallback is only for a bug.
    let (Some(value), Some(from_date)) = (value, from_date) else {
        return Err(vec![FieldError::new(ErrorCode::ErrInternal, "value")]);
    };
    Ok(PreparedRule {
        id: input.id,
        version,
        side: input.side,
        charge: input.charge_type,
        account_id: input.account_id,
        item_id: input.item_id,
        item_group: input.item_group,
        value,
        from_date,
        to_date,
        is_active: input.is_active,
        note,
    })
}
