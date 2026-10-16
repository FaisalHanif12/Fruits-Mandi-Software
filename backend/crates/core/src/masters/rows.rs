//! The rows the screens read back (plan 06). A row carries what the list and the form show,
//! including the names of the records it points at, so the screen needs no second call.

use serde::Serialize;
use ts_rs::TS;

use super::MasterKind;

/// One master row, tagged by kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MasterRow {
    /// شہر.
    City(CityRow),
    /// قسم مال.
    Item(ItemRow),
    /// پیکنگ.
    Packing(PackingRow),
    /// اڈا.
    Adda(AddaRow),
    /// مارکہ.
    Marka(MarkaRow),
    /// سٹور.
    Store(StoreRow),
    /// تفصیل.
    Narration(NarrationRow),
    /// خرچہ مد.
    ExpenseHead(ExpenseHeadRow),
    /// عملہ.
    Staff(StaffRow),
    /// بینک.
    BankAccount(BankAccountRow),
}

impl MasterRow {
    /// The kind of this row.
    pub fn kind(&self) -> MasterKind {
        match self {
            MasterRow::City(_) => MasterKind::City,
            MasterRow::Item(_) => MasterKind::Item,
            MasterRow::Packing(_) => MasterKind::Packing,
            MasterRow::Adda(_) => MasterKind::Adda,
            MasterRow::Marka(_) => MasterKind::Marka,
            MasterRow::Store(_) => MasterKind::Store,
            MasterRow::Narration(_) => MasterKind::Narration,
            MasterRow::ExpenseHead(_) => MasterKind::ExpenseHead,
            MasterRow::Staff(_) => MasterKind::Staff,
            MasterRow::BankAccount(_) => MasterKind::BankAccount,
        }
    }

    /// The row id.
    pub fn id(&self) -> i64 {
        match self {
            MasterRow::City(r) => r.id,
            MasterRow::Item(r) => r.id,
            MasterRow::Packing(r) => r.id,
            MasterRow::Adda(r) => r.id,
            MasterRow::Marka(r) => r.id,
            MasterRow::Store(r) => r.id,
            MasterRow::Narration(r) => r.id,
            MasterRow::ExpenseHead(r) => r.id,
            MasterRow::Staff(r) => r.id,
            MasterRow::BankAccount(r) => r.id,
        }
    }

    /// The row version (for the next edit).
    pub fn version(&self) -> i64 {
        match self {
            MasterRow::City(r) => r.version,
            MasterRow::Item(r) => r.version,
            MasterRow::Packing(r) => r.version,
            MasterRow::Adda(r) => r.version,
            MasterRow::Marka(r) => r.version,
            MasterRow::Store(r) => r.version,
            MasterRow::Narration(r) => r.version,
            MasterRow::ExpenseHead(r) => r.version,
            MasterRow::Staff(r) => r.version,
            MasterRow::BankAccount(r) => r.version,
        }
    }

    /// True if the master is in use (offered in pickers).
    pub fn is_active(&self) -> bool {
        match self {
            MasterRow::City(r) => r.is_active,
            MasterRow::Item(r) => r.is_active,
            MasterRow::Packing(r) => r.is_active,
            MasterRow::Adda(r) => r.is_active,
            MasterRow::Marka(r) => r.is_active,
            MasterRow::Store(r) => r.is_active,
            MasterRow::Narration(r) => r.is_active,
            MasterRow::ExpenseHead(r) => r.is_active,
            MasterRow::Staff(r) => r.is_active,
            MasterRow::BankAccount(r) => r.is_active,
        }
    }
}

/// A city.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CityRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// False when made inactive.
    pub is_active: bool,
}

/// An item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ItemRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `fruit` or `vegetable`.
    pub item_group: String,
    /// The usual packing, if any.
    #[ts(type = "number | null")]
    pub default_packing_id: Option<i64>,
    /// Urdu name of the usual packing.
    pub default_packing_name_ur: Option<String>,
    /// English name of the usual packing.
    pub default_packing_name_en: Option<String>,
    /// `unit` or `weight`.
    pub sold_by: String,
    /// False when made inactive.
    pub is_active: bool,
}

/// A packing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PackingRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Approximate weight in kg as text with up to 3 decimals (`12.5`), or null.
    pub approx_kg: Option<String>,
    /// False when made inactive.
    pub is_active: bool,
}

/// A truck stand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AddaRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The city, if any.
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
    /// Urdu name of the city.
    pub city_name_ur: Option<String>,
    /// English name of the city.
    pub city_name_en: Option<String>,
    /// False when made inactive.
    pub is_active: bool,
}

/// A mark.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MarkaRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Code.
    pub code: String,
    /// The mark as typed.
    pub mark_text: String,
    /// The seller it belongs to, if any.
    #[ts(type = "number | null")]
    pub owner_account_id: Option<i64>,
    /// Code of the seller's account.
    pub owner_account_code: Option<String>,
    /// Urdu name of the seller.
    pub owner_name_ur: Option<String>,
    /// English name of the seller.
    pub owner_name_en: Option<String>,
    /// False when made inactive.
    pub is_active: bool,
}

/// A store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StoreRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// False when made inactive.
    pub is_active: bool,
}

/// A standard narration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct NarrationRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Urdu text.
    pub text_ur: String,
    /// English text.
    pub text_en: String,
    /// The voucher types it is offered in. Empty means every type.
    pub used_in: Vec<String>,
    /// False when made inactive.
    pub is_active: bool,
}

/// An expense head.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ExpenseHeadRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The account it made (control 0501).
    #[ts(type = "number")]
    pub account_id: i64,
    /// The code of that account, for example `05010001`.
    pub account_code: String,
    /// False when made inactive.
    pub is_active: bool,
}

/// A staff member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StaffRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `munshi`, `collector` or `labour`.
    pub job: String,
    /// Phone as 11 digits, or null.
    pub phone: Option<String>,
    /// His ادھار رقم account, made when he is first lent money (read only). Null until then.
    #[ts(type = "number | null")]
    pub account_id: Option<i64>,
    /// The code of that account.
    pub account_code: Option<String>,
    /// False when made inactive.
    pub is_active: bool,
}

/// A bank account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BankAccountRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// The bank, as typed.
    pub bank_name: String,
    /// The account title, as typed.
    pub title: String,
    /// The account number, as typed.
    pub number: String,
    /// The account it made (control 0102).
    #[ts(type = "number")]
    pub account_id: i64,
    /// The code of that account, for example `01020002`.
    pub account_code: String,
    /// Urdu name of that account.
    pub name_ur: String,
    /// English name of that account.
    pub name_en: String,
    /// False when made inactive.
    pub is_active: bool,
}

/// Grams as kg text with up to 3 decimals and no trailing zeros: 12,500 gives `12.5`,
/// 12,000 gives `12`, 250 gives `0.25`. Zero or negative grams give `None` (a weight is more
/// than 0, so 0 is never stored).
///
/// ```
/// use mandi_core::masters::format_kg;
/// assert_eq!(format_kg(12_500).as_deref(), Some("12.5"));
/// assert_eq!(format_kg(12_000).as_deref(), Some("12"));
/// assert_eq!(format_kg(250).as_deref(), Some("0.25"));
/// assert_eq!(format_kg(1).as_deref(), Some("0.001"));
/// ```
pub fn format_kg(grams: i64) -> Option<String> {
    if grams <= 0 {
        return None;
    }
    let whole = grams.checked_div(1000)?;
    let rest = grams.checked_rem(1000)?;
    if rest == 0 {
        return Some(whole.to_string());
    }
    let decimals = format!("{rest:03}");
    Some(format!("{whole}.{}", decimals.trim_end_matches('0')))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::parse_weight;
    use proptest::prelude::*;

    #[test]
    fn kg_text_at_the_boundaries() {
        for (grams, text) in [
            (1, "0.001"),
            (10, "0.01"),
            (100, "0.1"),
            (999, "0.999"),
            (1_000, "1"),
            (1_001, "1.001"),
            (12_500, "12.5"),
            (100_000_000, "100000"),
        ] {
            assert_eq!(format_kg(grams).as_deref(), Some(text), "{grams}");
        }
        for grams in [0, -1, i64::MIN] {
            assert_eq!(format_kg(grams), None, "{grams}");
        }
        assert!(
            format_kg(i64::MAX).is_some(),
            "no overflow on the biggest number"
        );
    }

    proptest! {
        /// Grams -> kg text -> grams comes back exactly for every weight one input can hold
        /// (1 g to 100,000 kg).
        #[test]
        fn kg_text_round_trips(grams in 1_i64..=100_000_000) {
            let text = format_kg(grams).unwrap();
            let back = parse_weight(&text).unwrap();
            prop_assert_eq!(back.get(), grams);
        }
    }
}
