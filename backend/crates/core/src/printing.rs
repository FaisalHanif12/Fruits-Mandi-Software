//! Printing: the documents, the paper sizes and the Printing settings that the print data, the
//! print call and the Settings screen share (PRD Section 21 "Printing", FR-7.36, FR-7.57,
//! FR-19.3, A21, plan 14).
//!
//! Pure types and checks. No money maths is done here.
//!
//! Plan 17e adds a fifth document, the collection list (اگراہی فہرست, PRD FR-15.6), so that the
//! five built prints share one print pipeline and one header and footer (FR-19.13).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::auth::Language;
use crate::error::{ErrorCode, ErrorValue, FieldError};

/// The key of the Printing setup in the `settings` table (per computer). The list "print after
/// save" is per user and lives in `user_prefs` under [`PRINT_AFTER_SAVE_PREF`].
pub const SETTING_KEY: &str = "printing";
/// The `user_prefs` key of the "print after save" list (PRD FR-7.57, Architecture 15.3).
pub const PRINT_AFTER_SAVE_PREF: &str = "print_after_save";

/// The fewest copies of one print.
pub const MIN_COPIES: u32 = 1;
/// The most copies of one print.
pub const MAX_COPIES: u32 = 99;
/// The longest printer name kept (Windows printer names are at most 259 characters; this is the
/// limit of what the Settings screen accepts).
pub const MAX_PRINTER_CHARS: usize = 259;

/// The documents the app prints: three of a saved بکری (PRD FR-12.12), the slip of a voucher
/// (PRD FR-15.2) and the collection list (PRD FR-15.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PrintDoc {
    /// The seller bill (R1): one bill per seller.
    SellerBill,
    /// The buyer slips (R5): one slip per buyer, each on a new page.
    BuyerSlips,
    /// The slip of a one-time buyer of the پختہ کھاتہ (FR-13.1 to FR-13.3).
    PukhtaSlip,
    /// The slip of a voucher: a وصولی, an ادائیگی or a پیشگی, with the amount in words (PRD
    /// FR-15.2). One page.
    ReceiptSlip,
    /// The collection list: the buyers planned for a day with their dues, by collector and city
    /// (PRD FR-15.6, FR-7.61). Built from the day and the filters, not from a saved document.
    CollectionList,
}

impl PrintDoc {
    /// Every document, in the order the Settings screen lists them.
    pub const ALL: [PrintDoc; 5] = [
        PrintDoc::SellerBill,
        PrintDoc::BuyerSlips,
        PrintDoc::PukhtaSlip,
        PrintDoc::ReceiptSlip,
        PrintDoc::CollectionList,
    ];

    /// The documents of a saved بکری. "Print after save" (PRD FR-7.57) chooses among these.
    pub const BIKRI_DOCS: [PrintDoc; 3] = [
        PrintDoc::SellerBill,
        PrintDoc::BuyerSlips,
        PrintDoc::PukhtaSlip,
    ];

    /// The storage form: `seller_bill`, `buyer_slips` or `pukhta_slip`.
    pub const fn as_str(self) -> &'static str {
        match self {
            PrintDoc::SellerBill => "seller_bill",
            PrintDoc::BuyerSlips => "buyer_slips",
            PrintDoc::PukhtaSlip => "pukhta_slip",
            PrintDoc::ReceiptSlip => "receipt_slip",
            PrintDoc::CollectionList => "collection_list",
        }
    }

    /// The first part of a PDF file name: `seller-bill-1-ur.pdf` (plan 14).
    pub const fn file_stem(self) -> &'static str {
        match self {
            PrintDoc::SellerBill => "seller-bill",
            PrintDoc::BuyerSlips => "buyer-slips",
            PrintDoc::PukhtaSlip => "pukhta-slip",
            PrintDoc::ReceiptSlip => "receipt-slip",
            PrintDoc::CollectionList => "collection-list",
        }
    }
}

/// The paper classes (PRD Section 21, A21): A4, A5 and an 80 mm roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Paper {
    /// 210 × 297 mm.
    A4,
    /// 148 × 210 mm.
    A5,
    /// 80 mm wide roll; the height follows the content.
    Mm80,
}

impl Paper {
    /// The page width in hundredths of an inch, for the print settings of the web view
    /// (A4 8.27, A5 5.83, roll 3.15).
    pub const fn width_centi_inches(self) -> u32 {
        match self {
            Paper::A4 => 827,
            Paper::A5 => 583,
            Paper::Mm80 => 315,
        }
    }

    /// The page height in hundredths of an inch. `None` for the roll: its height is taken from
    /// the rendered content.
    pub const fn height_centi_inches(self) -> Option<u32> {
        match self {
            Paper::A4 => Some(1169),
            Paper::A5 => Some(827),
            Paper::Mm80 => None,
        }
    }
}

impl Paper {
    /// The page margins in whole millimetres: top, right, bottom, left. They are the same as the
    /// `@page` margins of the print CSS (reference.md of the `print-and-export` skill), and the
    /// print call sets the web view's margins to them, so the two never add up.
    pub const fn margins_mm(self) -> [u32; 4] {
        match self {
            Paper::A4 => [14, 10, 14, 10],
            Paper::A5 => [10, 8, 10, 8],
            Paper::Mm80 => [4, 3, 4, 3],
        }
    }
}

/// Where a print goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PrintTo {
    /// The printer of the Printing settings (empty name = the default printer).
    Printer,
    /// A PDF file.
    Pdf,
}

/// The per-computer part of the Printing settings, as stored in the `settings` table under
/// [`SETTING_KEY`]. A missing field takes its starting value (A21), so an old or damaged value
/// never stops a print.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PrintSetup {
    /// Paper of the seller bill.
    pub paper_seller_bill: Paper,
    /// Paper of the buyer slips.
    pub paper_buyer_slips: Paper,
    /// Paper of the پختہ slip.
    pub paper_pukhta_slip: Paper,
    /// Paper of the voucher slip (PRD FR-15.2).
    pub paper_receipt_slip: Paper,
    /// Paper of the collection list (PRD FR-15.6).
    pub paper_collection_list: Paper,
    /// Printer name. Empty means the default printer.
    pub printer: String,
    /// Copies of one print.
    pub copies: u32,
    /// Urdu digits (۰-۹) on Urdu prints (NFR-URD-14).
    pub urdu_digits: bool,
}

impl Default for PrintSetup {
    /// A21: bill A4, slips A5, one copy, English digits, the default printer.
    fn default() -> Self {
        Self {
            paper_seller_bill: Paper::A4,
            paper_buyer_slips: Paper::A5,
            paper_pukhta_slip: Paper::A5,
            paper_receipt_slip: Paper::A5,
            paper_collection_list: Paper::A4,
            printer: String::new(),
            copies: MIN_COPIES,
            urdu_digits: false,
        }
    }
}

impl PrintSetup {
    /// The paper of one document.
    pub const fn paper_of(&self, doc: PrintDoc) -> Paper {
        match doc {
            PrintDoc::SellerBill => self.paper_seller_bill,
            PrintDoc::BuyerSlips => self.paper_buyer_slips,
            PrintDoc::PukhtaSlip => self.paper_pukhta_slip,
            PrintDoc::ReceiptSlip => self.paper_receipt_slip,
            PrintDoc::CollectionList => self.paper_collection_list,
        }
    }
}

/// The documents ticked in "print after save" until the Owner changes the list: the three of a
/// بکری. A document with nothing to print (no one-time buyer) is skipped without an error.
pub fn default_print_after_save() -> Vec<PrintDoc> {
    PrintDoc::BIKRI_DOCS.to_vec()
}

/// The Printing settings the Settings screen shows (PRD Section 21): paper per document, the
/// printer, copies, Urdu digits, and the per-user list "print after save" (FR-7.57).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintingSettings {
    /// Paper of the seller bill (starts as A4).
    pub paper_seller_bill: Paper,
    /// Paper of the buyer slips (starts as A5).
    pub paper_buyer_slips: Paper,
    /// Paper of the پختہ slip (starts as A5).
    pub paper_pukhta_slip: Paper,
    /// Paper of the voucher slip (starts as A5).
    pub paper_receipt_slip: Paper,
    /// Paper of the collection list (starts as A4: it is a list of rows, PRD FR-15.6).
    pub paper_collection_list: Paper,
    /// Printer name. Empty means the default printer.
    pub printer: String,
    /// Copies of one print, 1 to 99 (starts at 1).
    #[ts(type = "number")]
    pub copies: u32,
    /// Urdu digits on Urdu prints (starts off).
    pub urdu_digits: bool,
    /// The documents ticked in "print after save", remembered per user.
    pub print_after_save: Vec<PrintDoc>,
}

impl PrintingSettings {
    /// The screen form from its two stored parts.
    pub fn from_parts(setup: PrintSetup, print_after_save: Vec<PrintDoc>) -> Self {
        Self {
            paper_seller_bill: setup.paper_seller_bill,
            paper_buyer_slips: setup.paper_buyer_slips,
            paper_pukhta_slip: setup.paper_pukhta_slip,
            paper_receipt_slip: setup.paper_receipt_slip,
            paper_collection_list: setup.paper_collection_list,
            printer: setup.printer,
            copies: setup.copies,
            urdu_digits: setup.urdu_digits,
            print_after_save,
        }
    }

    /// The per-computer part.
    pub fn setup(&self) -> PrintSetup {
        PrintSetup {
            paper_seller_bill: self.paper_seller_bill,
            paper_buyer_slips: self.paper_buyer_slips,
            paper_pukhta_slip: self.paper_pukhta_slip,
            paper_receipt_slip: self.paper_receipt_slip,
            paper_collection_list: self.paper_collection_list,
            printer: self.printer.clone(),
            copies: self.copies,
            urdu_digits: self.urdu_digits,
        }
    }
}

/// Input of `settings_set_printing`. The same fields as [`PrintingSettings`]; the number of
/// copies arrives as typed (text), like the idle lock minutes.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SetPrintingInput {
    /// Paper of the seller bill.
    pub paper_seller_bill: Paper,
    /// Paper of the buyer slips.
    pub paper_buyer_slips: Paper,
    /// Paper of the پختہ slip.
    pub paper_pukhta_slip: Paper,
    /// Paper of the voucher slip. Missing means A5.
    #[serde(default = "default_receipt_paper")]
    pub paper_receipt_slip: Paper,
    /// Paper of the collection list. Missing means A4.
    #[serde(default = "default_collection_paper")]
    pub paper_collection_list: Paper,
    /// Printer name. Empty means the default printer. At most 259 characters.
    pub printer: String,
    /// Copies as typed, whole number 1 to 99.
    pub copies: String,
    /// Urdu digits on Urdu prints.
    pub urdu_digits: bool,
    /// The documents ticked in "print after save". A repeated document counts once.
    pub print_after_save: Vec<PrintDoc>,
}

fn default_receipt_paper() -> Paper {
    PrintSetup::default().paper_receipt_slip
}

fn default_collection_paper() -> Paper {
    PrintSetup::default().paper_collection_list
}

/// Input of `reports_print` (plan 14): print or save as PDF the documents of one saved بکری.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrintInput {
    /// The saved بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// The documents to print, in this order. At least one.
    pub docs: Vec<PrintDoc>,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// Copies for this print, as typed, 1 to 99. Missing or empty means the Settings value.
    #[serde(default)]
    pub copies: Option<String>,
    /// Printer or PDF.
    pub to: PrintTo,
    /// Only the slip of this buyer account (buyer slips only). Missing means every buyer.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub only_buyer_id: Option<i64>,
}

/// What `reports_print` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintOutcome {
    /// The documents that were printed or saved, in order. A document with nothing to print
    /// (no one-time buyer for the پختہ slip) is left out.
    pub printed: Vec<PrintDoc>,
    /// The PDF files written, one per document, when the print went to a PDF. Null for a
    /// printer.
    pub pdf_paths: Option<Vec<String>>,
}

/// Input of `reports_print_voucher` (plan 15): print or save as PDF the slip of one saved voucher
/// (PRD FR-15.2).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrintVoucherInput {
    /// The voucher.
    #[ts(type = "number")]
    pub voucher_id: i64,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// Copies for this print, as typed, 1 to 99. Missing or empty means the Settings value.
    #[serde(default)]
    pub copies: Option<String>,
    /// Printer or PDF.
    pub to: PrintTo,
}

/// Input of `reports_print_collection` (plan 17e): print or save as PDF the collection list of one
/// day (PRD FR-15.6).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrintCollectionInput {
    /// The day, `DD-MM-YYYY` or a shortcut (PRD FR-7.19). Buyers whose collection day is this
    /// weekday are on the list.
    pub date: String,
    /// Only this collector (a staff member). Missing means every collector.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub collector_staff_id: Option<i64>,
    /// Only this city. Missing means every city.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
    /// Only buyers whose code, name or phone holds this text: the print has the rows of the
    /// screen's search (plan 38 R4).
    #[serde(default)]
    pub search: Option<String>,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// Copies for this print, as typed, 1 to 99. Missing or empty means the Settings value.
    #[serde(default)]
    pub copies: Option<String>,
    /// Printer or PDF.
    pub to: PrintTo,
}

/// The names of the collector and the city a collection list is filtered by, in both scripts, for
/// the facts under its title (plan 17e). `None` for a filter that is not used.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectionFilterNames {
    /// The collector's Urdu and English names.
    pub collector: Option<(String, String)>,
    /// The city's Urdu and English names.
    pub city: Option<(String, String)>,
}

/// Reads the number of copies as typed: a whole number from [`MIN_COPIES`] to [`MAX_COPIES`].
///
/// # Errors
/// `ERR_REQUIRED` if empty, `ERR_NOT_A_NUMBER` if not only digits, `ERR_OUT_OF_RANGE` (values
/// `min`, `max`) outside the range.
///
/// ```
/// use mandi_core::printing::parse_copies;
/// assert_eq!(parse_copies("copies", " 2 ").unwrap(), 2);
/// assert!(parse_copies("copies", "0").is_err());
/// ```
pub fn parse_copies(field: &str, text: &str) -> Result<u32, FieldError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(FieldError::new(ErrorCode::ErrRequired, field));
    }
    if !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(FieldError::new(ErrorCode::ErrNotANumber, field));
    }
    let out_of_range = || {
        FieldError::new(ErrorCode::ErrOutOfRange, field)
            .with("min", ErrorValue::Number(i64::from(MIN_COPIES)))
            .with("max", ErrorValue::Number(i64::from(MAX_COPIES)))
    };
    // Digits only, so a parse failure means the number is too big for any range.
    let value = text.parse::<u32>().map_err(|_| out_of_range())?;
    if (MIN_COPIES..=MAX_COPIES).contains(&value) {
        Ok(value)
    } else {
        Err(out_of_range())
    }
}

/// The file name of a PDF: `<document>-<number>-<language>.pdf`, for example
/// `seller-bill-1-ur.pdf` (plan 14).
///
/// ```
/// use mandi_core::auth::Language;
/// use mandi_core::printing::{PrintDoc, pdf_file_name};
/// assert_eq!(pdf_file_name(PrintDoc::SellerBill, 1, Language::Ur), "seller-bill-1-ur.pdf");
/// assert_eq!(pdf_file_name(PrintDoc::PukhtaSlip, 12, Language::En), "pukhta-slip-12-en.pdf");
/// ```
pub fn pdf_file_name(doc: PrintDoc, number: i64, language: Language) -> String {
    format!("{}-{number}-{}.pdf", doc.file_stem(), language.as_str())
}

/// A document list with each document once, in the order first given.
pub fn unique_docs(docs: &[PrintDoc]) -> Vec<PrintDoc> {
    let mut out: Vec<PrintDoc> = Vec::with_capacity(docs.len());
    for doc in docs {
        if !out.contains(doc) {
            out.push(*doc);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_values_are_a21() {
        let setup = PrintSetup::default();
        assert_eq!(setup.paper_of(PrintDoc::SellerBill), Paper::A4);
        assert_eq!(setup.paper_of(PrintDoc::BuyerSlips), Paper::A5);
        assert_eq!(setup.paper_of(PrintDoc::PukhtaSlip), Paper::A5);
        assert_eq!(setup.paper_of(PrintDoc::ReceiptSlip), Paper::A5);
        // Plan 17e: the collection list is a list of rows, so it starts on A4.
        assert_eq!(setup.paper_of(PrintDoc::CollectionList), Paper::A4);
        assert_eq!(setup.copies, 1);
        assert!(!setup.urdu_digits);
        assert!(setup.printer.is_empty());
    }

    #[test]
    fn a_missing_stored_field_takes_its_starting_value() {
        let setup: PrintSetup = serde_json::from_str(r#"{ "copies": 3 }"#).unwrap();
        assert_eq!(setup.copies, 3);
        assert_eq!(setup.paper_seller_bill, Paper::A4);
    }

    #[test]
    fn names_are_stable() {
        assert_eq!(serde_json::to_string(&Paper::Mm80).unwrap(), "\"mm80\"");
        assert_eq!(serde_json::to_string(&Paper::A4).unwrap(), "\"a4\"");
        assert_eq!(
            serde_json::to_string(&PrintDoc::BuyerSlips).unwrap(),
            "\"buyer_slips\""
        );
        assert_eq!(PrintDoc::PukhtaSlip.as_str(), "pukhta_slip");
        assert_eq!(
            serde_json::to_string(&PrintDoc::CollectionList).unwrap(),
            "\"collection_list\""
        );
        assert_eq!(PrintDoc::CollectionList.file_stem(), "collection-list");
        assert_eq!(PrintDoc::ALL.len(), 5);
        assert_eq!(serde_json::to_string(&PrintTo::Pdf).unwrap(), "\"pdf\"");
    }

    #[test]
    fn copies_are_checked() {
        assert_eq!(parse_copies("copies", "1").unwrap(), 1);
        assert_eq!(parse_copies("copies", "99").unwrap(), 99);
        assert_eq!(
            parse_copies("copies", "").unwrap_err().code,
            ErrorCode::ErrRequired
        );
        assert_eq!(
            parse_copies("copies", "two").unwrap_err().code,
            ErrorCode::ErrNotANumber
        );
        assert_eq!(
            parse_copies("copies", "1.5").unwrap_err().code,
            ErrorCode::ErrNotANumber
        );
        for bad in ["0", "100", "99999999999999999999"] {
            let error = parse_copies("copies", bad).unwrap_err();
            assert_eq!(error.code, ErrorCode::ErrOutOfRange, "{bad}");
            assert_eq!(error.field, "copies");
        }
    }

    #[test]
    fn pdf_names_follow_the_plan() {
        assert_eq!(
            pdf_file_name(PrintDoc::BuyerSlips, 7, Language::En),
            "buyer-slips-7-en.pdf"
        );
    }

    #[test]
    fn a_repeated_document_counts_once() {
        let docs = [
            PrintDoc::BuyerSlips,
            PrintDoc::SellerBill,
            PrintDoc::BuyerSlips,
        ];
        assert_eq!(
            unique_docs(&docs),
            vec![PrintDoc::BuyerSlips, PrintDoc::SellerBill]
        );
    }

    #[test]
    fn paper_sizes_match_the_reference() {
        assert_eq!(Paper::A4.width_centi_inches(), 827);
        assert_eq!(Paper::A5.height_centi_inches(), Some(827));
        assert_eq!(Paper::Mm80.height_centi_inches(), None);
    }

    #[test]
    fn margins_match_the_print_css() {
        // The literal `@page` margins of the frontend print layout: A4 `14mm 10mm`, A5
        // `10mm 8mm`, roll `4mm 3mm` (top and bottom first, then the sides).
        assert_eq!(Paper::A4.margins_mm(), [14, 10, 14, 10]);
        assert_eq!(Paper::A5.margins_mm(), [10, 8, 10, 8]);
        assert_eq!(Paper::Mm80.margins_mm(), [4, 3, 4, 3]);
    }

    #[test]
    fn the_screen_form_round_trips_its_parts() {
        let settings = PrintingSettings::from_parts(
            PrintSetup::default(),
            vec![PrintDoc::SellerBill, PrintDoc::PukhtaSlip],
        );
        assert_eq!(settings.setup(), PrintSetup::default());
        assert_eq!(settings.print_after_save.len(), 2);
    }
}
