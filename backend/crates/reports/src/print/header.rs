//! The one header and footer of every print (PRD FR-19.13, Section 21 "Firm", plan 17e).
//!
//! The header carries the firm name and mandi in the print language, the contact persons (name in
//! the print language and the phone as typed), the trade mark, the title and the facts (the
//! document number, the date, the filters). The footer carries the print time and the labels of
//! "Page x of y"; the page numbers themselves are drawn by the template's page counters.
//!
//! A contact or a trade mark that is not set is left out: the header never has an empty box.
//! Titles and labels are Fluent message IDs; the template resolves them in the print language
//! (CLAUDE.md rule 7). Every print builds its header here, so the five built prints and the D5
//! reports look the same.

use mandi_core::auth::Language;
use serde::Serialize;
use ts_rs::TS;

use super::ids::{FOOTER_PAGE, FOOTER_PAGE_OF, FOOTER_PRINTED};
use super::load::FirmNames;
use super::{PrintFact, PrintFirm};

/// One contact person in the header, in the print language (PRD Section 21 "Firm").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintContact {
    /// His name in the print language.
    pub name: String,
    /// His phone as typed (PRD FR-7.25).
    pub phone: String,
}

/// The header of every print (PRD FR-19.13): the firm, its contact persons and trade mark, then
/// the title and the facts. Built once, in `header.rs`, for all the prints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintHeader {
    /// The firm name and mandi in the print language.
    pub firm: PrintFirm,
    /// The contact persons, at most 3, in the order of the Settings. Empty if none are set.
    pub contacts: Vec<PrintContact>,
    /// The trade mark, printed in a box. Null if not set: the template draws no empty box.
    pub trade_mark: Option<String>,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The facts under the title, in order: the document number and date, or the filters.
    pub facts: Vec<PrintFact>,
}

/// The footer of every print (PRD FR-19.13): "Printed <time> · Page x of y". The page numbers are
/// drawn by the template's page counters; the labels and the time come from here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintFooter {
    /// Fluent message ID of "Printed".
    pub printed_label_id: String,
    /// The time of this print in Pakistan time, `DD-MM-YYYY HH:MM`.
    pub printed_at: String,
    /// Fluent message ID of "Page".
    pub page_label_id: String,
    /// Fluent message ID of "of".
    pub of_label_id: String,
}

/// The name in the print language. If that script is empty the other one is used, so a header is
/// never nameless (a record has both names, PRD FR-7.24; this guards old data).
pub(super) fn pick(language: Language, name_ur: &str, name_en: &str) -> String {
    let (first, second) = match language {
        Language::Ur => (name_ur, name_en),
        Language::En => (name_en, name_ur),
    };
    if first.trim().is_empty() {
        second.to_owned()
    } else {
        first.to_owned()
    }
}

/// The header of one print (PRD FR-19.13).
///
/// Example: a firm with two contacts and the trade mark "RH" gives a header with two
/// [`PrintContact`] rows and `trade_mark == Some("RH")`; a firm with none set gives an empty
/// `contacts` and `trade_mark == None`.
pub(super) fn header(
    language: Language,
    firm: &FirmNames,
    title_id: &str,
    facts: Vec<PrintFact>,
) -> PrintHeader {
    let contacts = firm
        .contacts
        .iter()
        .map(|contact| PrintContact {
            name: pick(language, &contact.name_ur, &contact.name_en),
            // The phone is as typed (PRD FR-7.25).
            phone: contact.phone.trim().to_owned(),
        })
        .filter(|contact| !contact.name.trim().is_empty() || !contact.phone.is_empty())
        .collect();
    let trade_mark = Some(firm.trade_mark.trim())
        .filter(|text| !text.is_empty())
        .map(str::to_owned);
    PrintHeader {
        firm: PrintFirm {
            name: pick(language, &firm.name_ur, &firm.name_en),
            mandi: pick(language, &firm.mandi_ur, &firm.mandi_en),
            address: firm.address.clone(),
            phone: firm.phone.clone(),
        },
        contacts,
        trade_mark,
        title_id: title_id.to_owned(),
        facts,
    }
}

/// The footer of one print: the print time and the labels of "Page x of y" (PRD FR-19.13).
pub(super) fn footer(printed_at: String) -> PrintFooter {
    PrintFooter {
        printed_label_id: FOOTER_PRINTED.to_owned(),
        printed_at,
        page_label_id: FOOTER_PAGE.to_owned(),
        of_label_id: FOOTER_PAGE_OF.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandi_core::firm::FirmContact;

    fn firm() -> FirmNames {
        FirmNames {
            name_ur: "مہر عدنان اینڈ برادرز".to_owned(),
            name_en: "Maher Adnan and Brothers".to_owned(),
            mandi_ur: "نیو سبزی منڈی".to_owned(),
            mandi_en: "New Sabzi Mandi".to_owned(),
            ..FirmNames::default()
        }
    }

    #[test]
    fn nothing_set_gives_no_contacts_and_no_trade_mark() {
        let out = header(Language::En, &firm(), "print-seller-bill-title", Vec::new());
        assert!(out.contacts.is_empty());
        assert_eq!(out.trade_mark, None);
        assert_eq!(out.firm.name, "Maher Adnan and Brothers");
    }

    #[test]
    fn contacts_follow_the_print_language_and_a_blank_trade_mark_is_left_out() {
        let mut names = firm();
        names.contacts = vec![FirmContact {
            name_ur: "منشی آصف بھٹی".to_owned(),
            name_en: "Munshi Asif Bhatti".to_owned(),
            phone: " 0300-4058860 ".to_owned(),
        }];
        names.trade_mark = "   ".to_owned();
        let ur = header(Language::Ur, &names, "t", Vec::new());
        assert_eq!(ur.contacts[0].name, "منشی آصف بھٹی");
        assert_eq!(ur.contacts[0].phone, "0300-4058860");
        assert_eq!(ur.trade_mark, None);
        names.trade_mark = "RH".to_owned();
        let en = header(Language::En, &names, "t", Vec::new());
        assert_eq!(en.contacts[0].name, "Munshi Asif Bhatti");
        assert_eq!(en.trade_mark.as_deref(), Some("RH"));
    }
}
