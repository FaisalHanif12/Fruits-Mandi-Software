//! The final answer the model writes (D6-system-prompt Section 4, `answer.schema.json`).
//!
//! The schema fixes the shape; this type is the shape in Rust, read **strictly**: the reply must be
//! a JSON object with exactly the seven fields of the schema, no more and no fewer, each of the
//! right type. A reply that is not (bad JSON, a missing field, an extra field) is "not understood"
//! (PRD FR-7.79, robustness item 5). `check.rs` then checks the content (PRD FR-7.76).
//!
//! The model's words hold references (`{{r2.balance}}`), never figures; this file only reads the
//! shape and finds the references.
//!
//! Example: `{"kind":"answer","text":"{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔",
//! "show":[{"result":"r3","as":"table","rows":7}],"next":[],"choose":null,"entry":null,
//! "nearest":null}`.

use serde::Deserialize;
use serde_json::Value;

/// The seven fields of the answer, in schema order. A reply has exactly these.
pub const FIELDS: [&str; 7] = ["kind", "text", "show", "next", "choose", "entry", "nearest"];

/// What kind of answer it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerKind {
    /// A checked answer.
    Answer,
    /// An answer that uses an estimate.
    Estimate,
    /// A recommendation.
    Advice,
    /// General advice for the shop, marked "عام مشورہ": practical, names no figure, and takes one
    /// model call. For a question the books cannot answer, or a general one (PRD FR-7.88, v1.19).
    GeneralAdvice,
    /// Several names fit: the Owner picks (`choose`).
    WhichOne,
    /// No name fits (`choose`).
    NotFound,
    /// The period is unknown.
    NeedDates,
    /// A party is needed and none was named.
    NeedParty,
    /// Outside the books (`nearest`).
    OutsideBooks,
    /// A change was asked (`entry`).
    ChangeRequest,
    /// The question was not understood.
    NotUnderstood,
}

impl AnswerKind {
    /// True for the kinds whose `text` is written by the model (the others have the app's words).
    pub fn has_text(self) -> bool {
        matches!(
            self,
            Self::Answer | Self::Estimate | Self::Advice | Self::GeneralAdvice
        )
    }
}

/// How a shown result is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowAs {
    /// A table of the result's rows.
    Table,
    /// A chart of a monthly series or an estimate.
    Chart,
}

/// One result the model wants shown.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Show {
    /// The result, for example `r3`.
    pub result: String,
    /// Table or chart.
    #[serde(rename = "as")]
    pub as_: ShowAs,
    /// How many rows, or null for the most (7).
    pub rows: Option<i64>,
}

/// The screens an answer may point to: the entry screens of a change request, and, for a help
/// answer, every main screen (plan 51 B3, PRD FR-7.89).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryScreenName {
    /// بکری.
    Bikri,
    /// وصولی.
    Wasooli,
    /// ادائیگی.
    Adaigi,
    /// پیشگی.
    Peshgi,
    /// خرچہ.
    Kharcha,
    /// ادھار رقم.
    UdharRaqam,
    /// الگ کھاتہ.
    AlagKhata,
    /// سرمایہ.
    Sarmaya,
    /// پھل.
    Phal,
    /// جنرل.
    General,
    /// The account form.
    Account,
    /// ڈیش بورڈ.
    Dashboard,
    /// ترتیبات.
    Settings,
    /// انڈکس.
    Masters,
    /// The accounts list.
    Accounts,
    /// کمیشن اصول.
    CommissionRules,
    /// آج کی بکری.
    TodaySales,
    /// باقی مال.
    UnsoldStock,
    /// چکتا کھاتہ.
    ChuktaKhata,
    /// اگراہی فہرست.
    CollectionList,
    /// روکڑ بک.
    CashBook,
    /// باغات.
    Gardens,
    /// مارکیٹ سفر.
    MarketTrips,
    /// شراکت دار.
    Partners,
    /// مہینہ بند.
    MonthClose,
    /// دن بند کریں.
    DayClose,
    /// رپورٹ.
    Reports,
    /// ڈوبی رقم.
    DeadAmount,
    /// تجزیات.
    Analytics,
    /// بیک اپ.
    Backup,
    // Change requests for garden and trip entries, a later sale, an edit and a cancel (plan 55
    // Section 11.1, D6 v1.5).
    /// قسط: a garden installment.
    GardenInstallment,
    /// باغ خرچ: a garden running cost.
    GardenCost,
    /// دوسری دکان میں فروخت: a sale of a garden's goods in another shop.
    GardenSale,
    /// باغ بند: closing a garden.
    GardenClose,
    /// خریداری: a purchase of a market trip.
    TripPurchase,
    /// سفر خرچ: a cost of a market trip.
    TripCost,
    /// دوسری دکان میں فروخت: a sale of a trip's goods in another shop.
    TripSale,
    /// باقی مال کی بعد کی بکری: a later sale of a lot.
    BikriLaterSale,
    /// بکری کی تدوین: editing a posting of a بکری.
    BikriEdit,
    /// بکری منسوخ: cancelling a بکری.
    BikriCancel,
}

impl EntryScreenName {
    /// Every name the answer schema offers, in the order of the schema. A test keeps this list equal
    /// to the `entry.screen` enum of `answer.schema.json`, both ways.
    pub const ALL: [&'static str; 40] = [
        "bikri",
        "wasooli",
        "adaigi",
        "peshgi",
        "kharcha",
        "udhar_raqam",
        "alag_khata",
        "sarmaya",
        "phal",
        "general",
        "account",
        "dashboard",
        "settings",
        "masters",
        "accounts",
        "commission_rules",
        "today_sales",
        "unsold_stock",
        "chukta_khata",
        "collection_list",
        "cash_book",
        "gardens",
        "market_trips",
        "partners",
        "month_close",
        "day_close",
        "reports",
        "dead_amount",
        "analytics",
        "backup",
        "garden_installment",
        "garden_cost",
        "garden_sale",
        "garden_close",
        "trip_purchase",
        "trip_cost",
        "trip_sale",
        "bikri_later_sale",
        "bikri_edit",
        "bikri_cancel",
    ];

    /// True for the entry screens, which open empty with a party chosen when there is one. The
    /// others are main screens, which open as they are and take no party (plan 51 B3, PRD FR-7.89).
    pub fn is_entry_screen(self) -> bool {
        matches!(
            self,
            Self::Bikri
                | Self::Wasooli
                | Self::Adaigi
                | Self::Peshgi
                | Self::Kharcha
                | Self::UdharRaqam
                | Self::AlagKhata
                | Self::Sarmaya
                | Self::Phal
                | Self::General
                | Self::Account
                | Self::GardenInstallment
                | Self::GardenCost
                | Self::GardenSale
                | Self::GardenClose
                | Self::TripPurchase
                | Self::TripCost
                | Self::TripSale
                | Self::BikriLaterSale
                | Self::BikriEdit
                | Self::BikriCancel
        )
    }
}

/// The screen a change request opens, and the party to choose on it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// The screen.
    pub screen: EntryScreenName,
    /// A reference to the party (a find match), or null.
    pub party: Option<String>,
}

/// The closest source for a question outside the books.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Nearest {
    /// A source id: `R13`, `A`, `party_card` ...
    pub source: String,
    /// A reference to the party, or null.
    pub party: Option<String>,
}

/// The answer of the model.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelAnswer {
    /// The kind.
    pub kind: AnswerKind,
    /// The words, with references. Empty for the kinds that have no text.
    pub text: String,
    /// Up to 3 results to show.
    pub show: Vec<Show>,
    /// Up to 3 suggested questions, with references.
    pub next: Vec<String>,
    /// For `which_one` and `not_found`: the `find` result.
    pub choose: Option<String>,
    /// For a change request.
    pub entry: Option<Entry>,
    /// For a question outside the books.
    pub nearest: Option<Nearest>,
}

/// Why a reply is not an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Not JSON.
    NotJson,
    /// JSON, but not an object with exactly the seven fields (a missing or an extra one).
    WrongFields,
    /// The fields are there but one has the wrong type or an unknown value.
    WrongShape,
}

impl ModelAnswer {
    /// Reads the model's reply strictly (see the module doc).
    ///
    /// # Errors
    /// [`ParseError`].
    pub fn parse(reply: &str) -> Result<Self, ParseError> {
        let value: Value = serde_json::from_str(reply).map_err(|_| ParseError::NotJson)?;
        let Some(object) = value.as_object() else {
            return Err(ParseError::WrongFields);
        };
        let exact = object.len() == FIELDS.len() && FIELDS.iter().all(|f| object.contains_key(*f));
        if !exact {
            return Err(ParseError::WrongFields);
        }
        let mut answer: ModelAnswer =
            serde_json::from_value(value).map_err(|_| ParseError::WrongShape)?;
        answer.choose = answer.choose.map(|id| clean_result_id(&id));
        Ok(answer)
    }
}

/// The result id of a `choose` the model wrote with braces or a path: `{{r1}}` and `r1.matches`
/// are `r1` (Q158 of the live run 2: `"choose":"{{r1}}"` failed the check and the question ended
/// as plain tables). Anything else is left as it is, so the check refuses it.
///
/// Example: `clean_result_id(" {{ f1 }} ")` is `f1`.
pub fn clean_result_id(text: &str) -> String {
    let inner = text
        .trim()
        .trim_start_matches("{{")
        .trim_end_matches("}}")
        .trim();
    match inner.split_once('.') {
        Some((id, _)) => id.trim().to_owned(),
        None => inner.to_owned(),
    }
}

/// One reference in a text: `{{r2.balance}}` is the path `r2.balance`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// The path inside the braces, trimmed.
    pub path: String,
    /// Where it starts in the text, in bytes.
    pub start: usize,
    /// Where it ends (after `}}`), in bytes.
    pub end: usize,
}

/// A piece of a text: words, or a reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// Words as written.
    Words(String),
    /// A reference.
    Ref(Reference),
}

/// Splits a text into words and references. A `{{` with no closing `}}` stays words (and is then
/// refused by the checks, which find a brace).
pub fn pieces(text: &str) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut rest = 0usize;
    let mut at = 0usize;
    while let Some(open) = text.get(at..).and_then(|t| t.find("{{")) {
        let start = at.saturating_add(open);
        let Some(close) = text.get(start..).and_then(|t| t.find("}}")) else {
            break;
        };
        let end = start.saturating_add(close).saturating_add(2);
        let inside = text
            .get(start.saturating_add(2)..end.saturating_sub(2))
            .unwrap_or("");
        if let Some(words) = text.get(rest..start).filter(|w| !w.is_empty()) {
            out.push(Piece::Words(words.to_owned()));
        }
        out.push(Piece::Ref(Reference {
            path: inside.trim().to_owned(),
            start,
            end,
        }));
        rest = end;
        at = end;
    }
    if let Some(words) = text.get(rest..).filter(|w| !w.is_empty()) {
        out.push(Piece::Words(words.to_owned()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{"kind":"answer","text":"{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔","show":[{"result":"r3","as":"table","rows":7}],"next":["پچھلے مہینے کا حساب؟"],"choose":null,"entry":null,"nearest":null}"#;

    #[test]
    fn a_good_answer_is_read() {
        let a = ModelAnswer::parse(GOOD).unwrap();
        assert_eq!(a.kind, AnswerKind::Answer);
        assert_eq!(a.show[0].as_, ShowAs::Table);
        assert_eq!(a.show[0].rows, Some(7));
        assert!(a.choose.is_none() && a.entry.is_none() && a.nearest.is_none());
        assert!(a.kind.has_text());
    }

    #[test]
    fn a_reply_that_is_not_the_schema_is_refused() {
        assert_eq!(ModelAnswer::parse("not json"), Err(ParseError::NotJson));
        assert_eq!(ModelAnswer::parse("[]"), Err(ParseError::WrongFields));
        // A missing field and an extra field.
        let missing = GOOD.replace(r#","nearest":null"#, "");
        assert_eq!(ModelAnswer::parse(&missing), Err(ParseError::WrongFields));
        let extra = GOOD.replace(r#""nearest":null"#, r#""nearest":null,"why":"x""#);
        assert_eq!(ModelAnswer::parse(&extra), Err(ParseError::WrongFields));
        // A wrong kind, a wrong type, an unknown field inside show.
        let kind = GOOD.replace(r#""kind":"answer""#, r#""kind":"guess""#);
        assert_eq!(ModelAnswer::parse(&kind), Err(ParseError::WrongShape));
        let ty = GOOD.replace(r#""next":["پچھلے مہینے کا حساب؟"]"#, r#""next":"x""#);
        assert_eq!(ModelAnswer::parse(&ty), Err(ParseError::WrongShape));
        let inner = GOOD.replace(r#""rows":7"#, r#""rows":7,"x":1"#);
        assert_eq!(ModelAnswer::parse(&inner), Err(ParseError::WrongShape));
    }

    #[test]
    fn a_choose_written_with_braces_or_a_path_is_the_result_id() {
        for (written, id) in [
            ("{{r1}}", "r1"),
            (" {{ f1 }} ", "f1"),
            ("r2.matches", "r2"),
            ("{{r3.matches.0}}", "r3"),
            ("r4", "r4"),
            ("", ""),
        ] {
            assert_eq!(clean_result_id(written), id, "{written:?}");
        }
        let reply = r#"{"kind":"which_one","text":"","show":[],"next":[],"choose":"{{r1}}","entry":null,"nearest":null}"#;
        assert_eq!(
            ModelAnswer::parse(reply).unwrap().choose.as_deref(),
            Some("r1")
        );
    }

    #[test]
    fn references_are_found_in_order() {
        let found = pieces("A {{ r2.balance }} b {{r3.rows.je1.debit}}");
        assert_eq!(found.len(), 4);
        assert!(matches!(&found[0], Piece::Words(w) if w == "A "));
        assert!(matches!(&found[1], Piece::Ref(r) if r.path == "r2.balance"));
        assert!(matches!(&found[3], Piece::Ref(r) if r.path == "r3.rows.je1.debit"));
        // No braces: all words. An open brace with no end stays words.
        assert_eq!(pieces("plain"), vec![Piece::Words("plain".to_owned())]);
        assert_eq!(pieces("a {{ b").len(), 1);
        assert!(pieces("").is_empty());
    }
}
