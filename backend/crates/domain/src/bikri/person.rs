//! The one-time person of a lot or a line as it is saved (PRD FR-13.2, FR-13.5): both names and
//! the phone cleaned, and the key that ties his lines together.

use mandi_core::bikri::PreparedPukhta;
use mandi_core::text;
use mandi_core::validate;

/// A one-time person, cleaned for saving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanPerson {
    /// Urdu name, cleaned (NFC, no invisible marks, single spaces).
    pub name_ur: String,
    /// English name, cleaned.
    pub name_en: String,
    /// Phone as 11 digits, or empty if there is none.
    pub phone: String,
    /// The person key: normalised Urdu name and phone digits (PRD FR-13.5).
    pub key: String,
}

/// Cleans a one-time person and makes his key. `None` if a name or the phone does not pass the
/// checks of PRD FR-7.28 and FR-13.2; `calc_posting` has already reported those, so a row that
/// is worked out always has a person that passes.
///
/// Example: `شاہد` with phone `0300-0000000` is saved with phone `03000000000` and key
/// `شاہد|03000000000`.
pub fn clean_person(person: &PreparedPukhta) -> Option<CleanPerson> {
    let name_ur = validate::name_ur("name_ur", &person.name_ur).ok()?;
    let name_en = validate::name_en("name_en", &person.name_en).ok()?;
    let phone = if person.phone.is_empty() {
        String::new()
    } else {
        validate::phone("phone", &person.phone).ok()?
    };
    let key = text::person_key(&name_ur, &phone);
    Some(CleanPerson {
        name_ur,
        name_en,
        phone,
        key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(ur: &str, en: &str, phone: &str) -> PreparedPukhta {
        PreparedPukhta {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.into(),
        }
    }

    #[test]
    fn shahid_of_prd_4_5_gets_his_key() {
        let clean = clean_person(&person("شاہد", "Shahid", "0300-0000000")).unwrap();
        assert_eq!(clean.phone, "03000000000");
        assert_eq!(clean.key, "شاہد|03000000000");
        assert_eq!(
            (clean.name_ur.as_str(), clean.name_en.as_str()),
            ("شاہد", "Shahid")
        );
    }

    #[test]
    fn the_same_person_typed_twice_has_one_key_and_two_people_have_two() {
        let a = clean_person(&person("ناصر", "Nasir", "03001111111")).unwrap();
        let b = clean_person(&person(" ناصر ", "Nasir", "0300-1111111")).unwrap();
        let c = clean_person(&person("ناصر", "Nasir", "")).unwrap();
        assert_eq!(a.key, b.key);
        assert_ne!(a.key, c.key);
    }

    #[test]
    fn a_bad_name_or_phone_is_none() {
        assert!(clean_person(&person("Shahid", "Shahid", "")).is_none());
        assert!(clean_person(&person("شاہد", "شاہد", "")).is_none());
        assert!(clean_person(&person("شاہد", "Shahid", "123")).is_none());
        assert!(clean_person(&person("", "Shahid", "")).is_none());
    }
}
