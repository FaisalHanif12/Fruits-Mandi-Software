//! The firm's people and trade mark, printed in the header of every print (PRD Section 21
//! "Firm", FR-19.13, plans 17e and 42).
//!
//! Pure types. The Settings screen reads and writes them through `settings_get` and
//! `settings_set_firm`; `mandi-reports` reads them to build the print header.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The `settings` key of the firm's people: a JSON list of [`FirmContact`], at most
/// [`MAX_CONTACTS`] (Architecture 15.3). The key keeps its old name `firm_contacts`.
pub const SETTING_KEY_CONTACTS: &str = "firm_contacts";
/// The `settings` key of the trade mark: a JSON string, as typed (Architecture 15.3).
pub const SETTING_KEY_TRADE_MARK: &str = "trade_mark";
/// The most people the header prints (PRD Section 21, v1.12: up to 6).
pub const MAX_CONTACTS: usize = 6;

/// The role of a person in the print header (PRD Section 21, v1.12, Umar 31-10-2026). The header
/// prints the owners first, then the munshi.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FirmRole {
    /// مالک: an owner of the firm.
    Owner,
    /// منشی: the munshi. A contact saved before v1.12 has no role and is read as this.
    #[default]
    Munshi,
}

impl FirmRole {
    /// The place in the header: 0 for an owner, 1 for the munshi (owners come first).
    #[must_use]
    pub const fn header_rank(self) -> u8 {
        match self {
            Self::Owner => 0,
            Self::Munshi => 1,
        }
    }
}

/// One person of the firm for the print header, for example the owner "حاجی محمد عدنان" /
/// "Haji Muhammad Adnan" or the munshi "آصف بھٹی" / "Asif Bhatti" with phone 0300-4058860.
/// Both names are required, each in its own script (PRD FR-7.24, FR-7.28). The role says where in
/// the header he prints. The phone is optional: empty means none, and a phone that is typed is
/// checked like any phone (Code Standards 4.2). The print shows the name in the print language and
/// the phone as `03XX-XXXXXXX` (FR-7.25).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FirmContact {
    /// Name, Urdu letters.
    pub name_ur: String,
    /// Name, English letters.
    pub name_en: String,
    /// The role. A value saved before v1.12 has none and is read as the munshi.
    #[serde(default)]
    pub role: FirmRole,
    /// Phone, `03XXXXXXXXX` or `03XX-XXXXXXX`; empty for none. Stored as 11 digits.
    #[serde(default)]
    pub phone: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keys_are_the_ones_of_architecture_15_3() {
        assert_eq!(SETTING_KEY_CONTACTS, "firm_contacts");
        assert_eq!(SETTING_KEY_TRADE_MARK, "trade_mark");
        assert_eq!(MAX_CONTACTS, 6);
    }

    #[test]
    fn a_contact_round_trips_as_json() {
        let contact = FirmContact {
            name_ur: "آصف بھٹی".to_owned(),
            name_en: "Asif Bhatti".to_owned(),
            role: FirmRole::Munshi,
            phone: "0300-4058860".to_owned(),
        };
        let text = serde_json::to_string(&contact).unwrap();
        assert!(text.contains("\"role\":\"munshi\""));
        assert_eq!(serde_json::from_str::<FirmContact>(&text).unwrap(), contact);
    }

    #[test]
    fn a_contact_saved_before_v1_12_reads_as_the_munshi_and_a_missing_phone_as_none() {
        let old = r#"{"name_ur":"منشی","name_en":"Munshi","phone":"0300-4058860"}"#;
        let contact: FirmContact = serde_json::from_str(old).unwrap();
        assert_eq!(contact.role, FirmRole::Munshi);
        let no_phone: FirmContact =
            serde_json::from_str(r#"{"name_ur":"علی","name_en":"Ali","role":"owner"}"#).unwrap();
        assert_eq!(no_phone.role, FirmRole::Owner);
        assert_eq!(no_phone.phone, "");
    }

    #[test]
    fn owners_come_before_the_munshi() {
        assert!(FirmRole::Owner.header_rank() < FirmRole::Munshi.header_rank());
    }
}
