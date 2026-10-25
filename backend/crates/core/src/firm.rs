//! The firm's contact persons and trade mark, printed in the header of every print (PRD Section 21
//! "Firm", FR-19.13, plan 17e).
//!
//! Pure types. The Settings screen reads and writes them through `settings_get` and
//! `settings_set_firm`; `mandi-reports` reads them to build the print header.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The `settings` key of the contact persons: a JSON list of [`FirmContact`], at most
/// [`MAX_CONTACTS`] (Architecture 15.3).
pub const SETTING_KEY_CONTACTS: &str = "firm_contacts";
/// The `settings` key of the trade mark: a JSON string, as typed (Architecture 15.3).
pub const SETTING_KEY_TRADE_MARK: &str = "trade_mark";
/// The most contact persons the header prints (PRD Section 21).
pub const MAX_CONTACTS: usize = 3;

/// One contact person of the firm for the print header, for example "منشی آصف بھٹی" /
/// "Munshi Asif Bhatti" with phone 0300-4058860. Both names are required, each in its own script
/// (PRD FR-7.24, FR-7.28); the phone is checked like any phone (Code Standards 4.2). The print
/// shows the name in the print language and the phone as typed (FR-7.25).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FirmContact {
    /// Name, Urdu letters.
    pub name_ur: String,
    /// Name, English letters.
    pub name_en: String,
    /// Phone, `03XXXXXXXXX` or `03XX-XXXXXXX`.
    pub phone: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keys_are_the_ones_of_architecture_15_3() {
        assert_eq!(SETTING_KEY_CONTACTS, "firm_contacts");
        assert_eq!(SETTING_KEY_TRADE_MARK, "trade_mark");
        assert_eq!(MAX_CONTACTS, 3);
    }

    #[test]
    fn a_contact_round_trips_as_json() {
        let contact = FirmContact {
            name_ur: "منشی آصف بھٹی".to_owned(),
            name_en: "Munshi Asif Bhatti".to_owned(),
            phone: "0300-4058860".to_owned(),
        };
        let text = serde_json::to_string(&contact).unwrap();
        assert_eq!(serde_json::from_str::<FirmContact>(&text).unwrap(), contact);
    }
}
