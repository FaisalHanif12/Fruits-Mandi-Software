//! The header and footer of a print that is not a document of a بکری or a voucher (plan 17e), for
//! example the اگراہی فہرست (PRD FR-15.6). It reads the firm, the contacts and the trade mark
//! from the Settings and builds the same header and footer as every other print (PRD FR-19.13).

use mandi_core::auth::Language;
use mandi_db::Db;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::format::NumberStyle;
use super::header::{PrintFooter, PrintHeader, footer, header};
use super::load::{load_firm, load_grouping, load_setup};
use super::{PrintFact, TextDir};
use crate::error::ReportsError;

/// Input of `reports_print_header` (plan 17e): the header and footer of a print that is not a
/// document of a بکری or a voucher, for example the اگراہی فہرست (PRD FR-15.6).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrintFrameInput {
    /// The print language (PRD FR-7.36): names and digits follow it.
    pub language: Language,
    /// Fluent message ID of the title (defined by the screen's print, in `print.ftl`).
    pub title_id: String,
    /// The facts under the title: the date and the filters (PRD FR-19.10), each a label ID and
    /// its value as it is to be printed. A value is shown as typed, so the screen formats a date
    /// and a filter name. A fact never carries money: an amount or a total for print is always
    /// formatted by the backend from the stored paisa (CLAUDE.md rule 1).
    #[serde(default)]
    pub facts: Vec<PrintFact>,
}

/// What `reports_print_header` gives back: the shared header and footer, with the language, the
/// direction and the digit style of the print (plan 17e).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintFrame {
    /// The print language.
    pub language: Language,
    /// The writing direction, from the language.
    pub dir: TextDir,
    /// True when Urdu digits are used on this print (an Urdu print with the setting on).
    pub urdu_digits: bool,
    /// The header (PRD FR-19.13).
    pub header: PrintHeader,
    /// The footer (PRD FR-19.13).
    pub footer: PrintFooter,
}

/// Builds the header and footer for a title and its facts.
///
/// Example: the اگراہی فہرست for 10-10-26 sends its title ID and one fact (the date). The answer
/// holds the firm, up to 3 contacts, the trade mark when set, and the print time.
///
/// # Errors
/// A database or data error.
pub fn print_frame(db: &Db, input: &PrintFrameInput) -> Result<PrintFrame, ReportsError> {
    let (firm, grouping, setup) = db.read(|conn| {
        Ok::<_, ReportsError>((load_firm(conn)?, load_grouping(conn)?, load_setup(conn)?))
    })?;
    let style = NumberStyle::new(input.language, grouping, setup.urdu_digits);
    Ok(PrintFrame {
        language: input.language,
        dir: TextDir::of(input.language),
        urdu_digits: style.urdu_digits(),
        header: header(input.language, &firm, &input.title_id, input.facts.clone()),
        footer: footer(style.printed_at(db.clock().now_utc())),
    })
}
