//! The print data of a report (plan 20, PRD FR-19.3 to FR-19.7, FR-19.10, FR-19.13).
//!
//! It is the same page the screen shows, with every row, formatted for the print language. The
//! template lays it out and calculates nothing:
//!
//! - every number is formatted here: grouping by the number grouping setting, English digits
//!   unless the Urdu digits setting is on and the print language is Urdu (NFR-URD-14);
//! - titles, column headings and labels are Fluent message IDs; names are already in the print
//!   language; the only sentences built here are the amounts in words (FR-19.7);
//! - the header and footer are the shared ones of every print (PRD FR-19.13): the template asks
//!   `reports_print_header` for them with `title_id` and `filters` (the facts under the title,
//!   FR-19.10), so no header is built here.
//!
//! Example (R18 after the worked day, English): the totals row reads `85,048` and `85,048`, the
//! difference line reads `0`.

use mandi_core::accounts::BalanceSide;
use mandi_core::auth::Language;
use mandi_core::english_words::english_words;
use mandi_core::money::{Grouping, Paisa};
use mandi_core::printing::Paper;
use mandi_core::urdu_words::urdu_words;
use serde::Serialize;
use ts_rs::TS;

use super::ids;
use super::model::{
    ArgValue, Cell, ColumnKind, DetailPart, GardenView, GroupBy, MessageArg, Names, PaidFilter,
    PartyRef, ReportId, ReportPage, ResolvedFilters, RowKind, TripView,
};
use crate::error::ReportsError;
use crate::print::format::NumberStyle;
use crate::print::{PrintFact, PrintWords, TextDir};

/// The ID of the label of an amount in words, shared with the other prints.
const IN_WORDS: &str = "print-in-words";

/// An account in a print: short code and name with tag (FR-19.13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintReportParty {
    /// The short code of a seller or buyer. Null for other accounts.
    pub short_code: Option<String>,
    /// The full code.
    pub code: String,
    /// The name in the print language.
    pub name: String,
    /// The tag after the name. Null for none.
    pub tag: Option<String>,
    /// The person inside a پختہ کھاتہ account, as stored. Null for any other line.
    pub person_key: Option<String>,
    /// His phone in the dash form `03XX-XXXXXXX`. Missing when he has none or for any other line.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub phone: Option<String>,
}

/// The goods or reason behind a ledger line, formatted for print.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename = "ReportPrintDetailPart")]
pub enum PrintDetailPart {
    /// A sale line: "کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160".
    Sale {
        /// The بکری number.
        bikri_number: String,
        /// The item in the print language.
        item: String,
        /// The packing in the print language.
        packing: String,
        /// The quantity.
        qty: String,
        /// The rate.
        rate: String,
        /// The value.
        value: String,
        /// The buyer commission.
        commission: String,
        /// لاگا.
        laga: String,
    },
    /// A seller credit.
    Lot {
        /// The بکری number.
        bikri_number: String,
        /// The item in the print language.
        item: String,
        /// The packing in the print language.
        packing: String,
        /// The quantity.
        qty: String,
        /// خام بکری.
        gross: String,
        /// The deductions.
        deductions: String,
    },
    /// Cash lent to a buyer.
    CashLent {
        /// The reason as typed.
        reason: String,
    },
    /// A note as typed.
    Text {
        /// The text.
        text: String,
    },
}

/// One printed cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(rename = "ReportPrintCell")]
pub enum PrintCell {
    /// Nothing.
    Empty,
    /// Text as typed.
    Text {
        /// The text.
        text: String,
    },
    /// A Fluent message ID.
    Label {
        /// The ID.
        id: String,
    },
    /// A whole number.
    Number {
        /// The number as printed.
        text: String,
    },
    /// Money in whole rupees.
    Money {
        /// The amount as printed.
        text: String,
    },
    /// Money with a side.
    Balance {
        /// The amount as printed.
        text: String,
        /// Fluent message ID of the side (بنام, جمع).
        side_id: String,
    },
    /// A date, `DD-MM-YYYY`.
    Date {
        /// The date as printed.
        text: String,
    },
    /// A share of a total, for example `72.10`. The template adds the percent sign.
    Percent {
        /// The share as printed.
        text: String,
    },
    /// A month of a year. The template names the month from Fluent in the print language.
    Month {
        /// The year as printed.
        year: String,
        /// The month, 1 to 12.
        #[ts(type = "number")]
        month: i64,
    },
    /// An account.
    Party {
        /// The account.
        party: PrintReportParty,
    },
    /// A name in the print language.
    Names {
        /// The name.
        text: String,
    },
    /// A document.
    Doc {
        /// Fluent message ID of the kind.
        type_id: String,
        /// The number as printed. Null if the source has none.
        number: Option<String>,
    },
    /// The goods or reason behind a ledger line.
    Detail {
        /// The parts, in order.
        parts: Vec<PrintDetailPart>,
    },
    /// A sentence the template writes from a Fluent message and its arguments, already
    /// formatted (a partner statement line, PRD FR-17.10).
    Message {
        /// The Fluent message ID.
        id: String,
        /// Its arguments.
        args: Vec<PrintMessageArg>,
    },
}

/// One argument of a [`PrintCell::Message`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportPrintMessageArg")]
pub struct PrintMessageArg {
    /// The variable name in the message.
    pub name: String,
    /// Its value.
    pub value: PrintArgValue,
}

/// The value of a print message argument. Numbers, money, rates and names are already text in
/// the print language and number style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(rename = "ReportPrintArgValue")]
pub enum PrintArgValue {
    /// Text as it is.
    Text {
        /// The text.
        text: String,
    },
    /// Another message (a word such as a number series).
    Message {
        /// The Fluent message ID.
        id: String,
        /// Its arguments.
        args: Vec<PrintMessageArg>,
    },
}

/// One printed column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintReportColumn {
    /// A stable key.
    pub key: String,
    /// Fluent message ID of the heading.
    pub title_id: String,
    /// What it holds. The template aligns numbers and money to the right edge.
    pub kind: ColumnKind,
}

/// One printed row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintReportRow {
    /// One cell for each column.
    pub cells: Vec<PrintCell>,
    /// How it is drawn.
    pub kind: RowKind,
}

/// A line above the table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportPrintSummaryLine")]
pub struct PrintSummaryLine {
    /// Fluent message ID of the label.
    pub label_id: String,
    /// The value.
    pub cell: PrintCell,
}

/// Everything a template needs to print one report with every row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ReportPrintData {
    /// The report.
    pub report: ReportId,
    /// The print language.
    pub language: Language,
    /// The writing direction.
    pub dir: TextDir,
    /// The paper class.
    pub paper: Paper,
    /// True when Urdu digits are used on this print.
    pub urdu_digits: bool,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The filters of the run, the facts under the title: dates and party (FR-19.10). They go to
    /// `reports_print_header` with the title.
    pub filters: Vec<PrintFact>,
    /// The columns.
    pub columns: Vec<PrintReportColumn>,
    /// Lines above the table (opening balance and so on).
    pub summary: Vec<PrintSummaryLine>,
    /// Every row of the filter.
    pub rows: Vec<PrintReportRow>,
    /// How many rows there are, as printed.
    pub row_count: String,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals row, one cell for each column.
    pub totals: Vec<PrintCell>,
    /// The closing balance of a ledger or statement in words (FR-19.7). Null otherwise.
    pub words: Option<PrintWords>,
}

/// What a print needs besides the report page.
#[derive(Debug, Clone)]
pub struct PrintOptions {
    /// The print language.
    pub language: Language,
    /// The number grouping setting (PRD FR-19.7).
    pub grouping: Grouping,
    /// The Urdu digits switch of the Printing settings.
    pub urdu_digits_setting: bool,
    /// The paper class.
    pub paper: Paper,
}

/// The formatting of one print: names in the print language, numbers by the grouping and digits
/// setting. Shared with the analytics print (`crate::analytics`).
pub(crate) struct Fmt {
    pub(crate) language: Language,
    grouping: Grouping,
    pub(crate) style: NumberStyle,
}

impl Fmt {
    /// The formatting a print with these options uses.
    pub(crate) fn new(options: &PrintOptions) -> Self {
        Self {
            language: options.language,
            grouping: options.grouping,
            style: NumberStyle::new(
                options.language,
                options.grouping,
                options.urdu_digits_setting,
            ),
        }
    }

    /// An account for a header fact: short code (or code), name and tag.
    pub(crate) fn party_text(&self, p: &PartyRef) -> String {
        let mut text = String::new();
        if let Some(short) = &p.short_code {
            text.push_str(short);
        } else {
            text.push_str(&p.code);
        }
        text.push(' ');
        text.push_str(&self.name(&p.name_ur, &p.name_en));
        if let Some(tag) = &p.tag {
            text.push(' ');
            text.push_str(tag);
        }
        // A one-time person is told from another of the same name by his phone.
        if let Some(phone) = &p.phone {
            text.push_str(" (");
            text.push_str(phone);
            text.push(')');
        }
        text
    }

    pub(crate) fn name(&self, ur: &str, en: &str) -> String {
        let (first, second) = match self.language {
            Language::Ur => (ur, en),
            Language::En => (en, ur),
        };
        if first.trim().is_empty() {
            second.to_owned()
        } else {
            first.to_owned()
        }
    }

    pub(crate) fn party(&self, party: &PartyRef) -> PrintReportParty {
        PrintReportParty {
            short_code: party.short_code.clone(),
            code: party.code.clone(),
            name: self.name(&party.name_ur, &party.name_en),
            tag: party.tag.clone(),
            person_key: party.person_key.clone(),
            phone: party.phone.clone(),
        }
    }

    pub(crate) fn names(&self, names: &Names) -> String {
        self.name(&names.ur, &names.en)
    }

    fn side_id(side: BalanceSide) -> String {
        match side {
            BalanceSide::Dr => ids::SIDE_DR,
            BalanceSide::Cr => ids::SIDE_CR,
            BalanceSide::Zero => ids::SIDE_ZERO,
        }
        .to_owned()
    }

    pub(crate) fn detail(&self, part: &DetailPart) -> Result<PrintDetailPart, ReportsError> {
        Ok(match part {
            DetailPart::Sale {
                bikri_number,
                item,
                packing,
                qty,
                rate_paisa,
                value_paisa,
                commission_paisa,
                laga_paisa,
            } => PrintDetailPart::Sale {
                bikri_number: self.style.count(*bikri_number),
                item: self.names(item),
                packing: self.names(packing),
                qty: self.style.count(*qty),
                rate: self.style.money(*rate_paisa)?,
                value: self.style.money(*value_paisa)?,
                commission: self.style.money(*commission_paisa)?,
                laga: self.style.money(*laga_paisa)?,
            },
            DetailPart::Lot {
                bikri_number,
                item,
                packing,
                qty,
                gross_paisa,
                deductions_paisa,
            } => PrintDetailPart::Lot {
                bikri_number: self.style.count(*bikri_number),
                item: self.names(item),
                packing: self.names(packing),
                qty: self.style.count(*qty),
                gross: self.style.money(*gross_paisa)?,
                deductions: self.style.money(*deductions_paisa)?,
            },
            DetailPart::CashLent { reason } => PrintDetailPart::CashLent {
                reason: reason.clone(),
            },
            DetailPart::Text { text } => PrintDetailPart::Text { text: text.clone() },
        })
    }

    pub(crate) fn cell(&self, cell: &Cell) -> Result<PrintCell, ReportsError> {
        Ok(match cell {
            Cell::Empty => PrintCell::Empty,
            Cell::Text { text } => PrintCell::Text { text: text.clone() },
            Cell::Label { id } => PrintCell::Label { id: id.clone() },
            Cell::Number { value } => PrintCell::Number {
                text: self.style.count(*value),
            },
            Cell::Money { paisa } => PrintCell::Money {
                text: self.style.money(*paisa)?,
            },
            Cell::Balance { paisa, side } => PrintCell::Balance {
                text: self.style.money(*paisa)?,
                side_id: Self::side_id(*side),
            },
            Cell::Date { iso } => PrintCell::Date {
                text: self.style.date(iso)?,
            },
            Cell::Percent { hundredths } => PrintCell::Percent {
                text: self.style.hundredths(*hundredths),
            },
            Cell::Month { year, month } => PrintCell::Month {
                year: self.style.plain(*year),
                month: *month,
            },
            Cell::Party { party } => PrintCell::Party {
                party: self.party(party),
            },
            Cell::Names { names } => PrintCell::Names {
                text: self.names(names),
            },
            Cell::Doc { type_id, number } => PrintCell::Doc {
                type_id: type_id.clone(),
                number: number.map(|n| self.style.count(n)),
            },
            Cell::Detail { parts } => PrintCell::Detail {
                parts: parts
                    .iter()
                    .map(|p| self.detail(p))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            Cell::Message { id, args } => PrintCell::Message {
                id: id.clone(),
                args: self.message_args(args)?,
            },
        })
    }

    /// The arguments of a message, each written the way the print writes it.
    fn message_args(&self, args: &[MessageArg]) -> Result<Vec<PrintMessageArg>, ReportsError> {
        args.iter()
            .map(|arg| {
                Ok(PrintMessageArg {
                    name: arg.name.clone(),
                    value: match &arg.value {
                        ArgValue::Text { text } => PrintArgValue::Text { text: text.clone() },
                        ArgValue::Number { value } => PrintArgValue::Text {
                            text: self.style.count(*value),
                        },
                        ArgValue::Money { paisa } => PrintArgValue::Text {
                            text: self.style.money(*paisa)?,
                        },
                        ArgValue::Percent { hundredths } => PrintArgValue::Text {
                            text: self.style.bps_text(*hundredths),
                        },
                        ArgValue::Names { names } => PrintArgValue::Text {
                            text: self.names(names),
                        },
                        ArgValue::Message { id, args } => PrintArgValue::Message {
                            id: id.clone(),
                            args: self.message_args(args)?,
                        },
                    },
                })
            })
            .collect()
    }

    pub(crate) fn cells(&self, cells: &[Cell]) -> Result<Vec<PrintCell>, ReportsError> {
        cells.iter().map(|c| self.cell(c)).collect()
    }

    fn words(&self, amount: Paisa) -> Result<PrintWords, ReportsError> {
        let text = match self.language {
            Language::Ur => urdu_words(amount)?,
            Language::En => english_words(amount, self.grouping)?,
        };
        Ok(PrintWords {
            label_id: IN_WORDS.to_owned(),
            text,
        })
    }
}

pub(crate) fn fact(label: &str, value: String) -> PrintFact {
    PrintFact {
        label_id: label.to_owned(),
        value,
    }
}

/// The filters of the run as header facts: dates and party first (FR-19.10).
pub(crate) fn filter_facts(
    fmt: &Fmt,
    id: ReportId,
    filters: &ResolvedFilters,
) -> Result<Vec<PrintFact>, ReportsError> {
    let mut facts = Vec::new();
    if let Some(from) = &filters.from {
        facts.push(fact(ids::FILTER_FROM, fmt.style.date(from)?));
    }
    if let Some(to) = &filters.to {
        facts.push(fact(ids::FILTER_TO, fmt.style.date(to)?));
    }
    if let Some(as_of) = &filters.as_of {
        facts.push(fact(ids::FILTER_AS_OF, fmt.style.date(as_of)?));
    }
    let party_text = |p: &PartyRef| fmt.party_text(p);
    if let Some(party) = &filters.party {
        facts.push(fact(ids::FILTER_PARTY, party_text(party)));
    }
    if let Some(party) = &filters.seller {
        facts.push(fact(ids::FILTER_SELLER, party_text(party)));
    }
    if let Some(party) = &filters.zamindar {
        facts.push(fact(ids::FILTER_ZAMINDAR, party_text(party)));
    }
    if let Some(names) = &filters.item {
        facts.push(fact(ids::FILTER_ITEM, fmt.names(names)));
    }
    if let Some(names) = &filters.city {
        facts.push(fact(ids::FILTER_CITY, fmt.names(names)));
    }
    if let Some(names) = &filters.adda {
        facts.push(fact(ids::FILTER_ADDA, fmt.names(names)));
    }
    if let Some(names) = &filters.partner {
        facts.push(fact(ids::FILTER_PARTNER, fmt.names(names)));
    }
    // The statement says whether it is balances only or with its lines. The label is the whole
    // fact, so the value is empty.
    // R9 says whether it lists only paid or only due lines; R28 how it groups. The label is the
    // whole fact, so the value is empty.
    if let Some(status) = filters.status {
        let label = match status {
            PaidFilter::Paid => ids::FILTER_PAID,
            PaidFilter::Due => ids::FILTER_DUE,
        };
        facts.push(fact(label, String::new()));
    }
    if let Some(group_by) = filters.group_by {
        let label = match group_by {
            GroupBy::Day => ids::FILTER_BY_DAY,
            GroupBy::Month => ids::FILTER_BY_MONTH,
        };
        facts.push(fact(label, String::new()));
    }
    // R15 says which kind of voucher it lists; R33 how it is laid out. The label is the whole fact.
    if let Some(kind) = filters.voucher_type {
        facts.push(fact(&ids::doc_id(kind.as_str()), String::new()));
    }
    if let Some(view) = filters.trip_view {
        let label = match view {
            TripView::Trip => ids::FILTER_TRIP_BY_TRIP,
            TripView::Item => ids::FILTER_TRIP_BY_ITEM,
            TripView::Day => ids::FILTER_TRIP_BY_DAY,
            TripView::Month => ids::FILTER_TRIP_BY_MONTH,
            TripView::Partner => ids::FILTER_TRIP_BY_PARTNER,
        };
        facts.push(fact(label, String::new()));
    }
    // R1: the بکری picked, by its number.
    if let Some(number) = filters.bikri_number {
        facts.push(fact(ids::FILTER_BIKRI, number.to_string()));
    }
    // R36: the garden picked, and how the gardens are laid out. The label is the whole fact.
    if let Some(names) = &filters.garden {
        facts.push(fact(ids::FILTER_GARDEN, fmt.names(names)));
    }
    if let Some(view) = filters.garden_view {
        let label = match view {
            GardenView::Garden => ids::FILTER_GARDEN_BY_GARDEN,
            GardenView::Partner => ids::FILTER_GARDEN_BY_PARTNER,
        };
        facts.push(fact(label, String::new()));
    }
    // R37: only the fruit for the shop's guests. The label is the whole fact.
    if filters.guests_only == Some(true) {
        facts.push(fact(ids::FRUIT_FOR_GUESTS, String::new()));
    }
    if id == ReportId::R14 {
        let label = if filters.detail {
            ids::FILTER_DETAIL
        } else {
            ids::FILTER_BALANCE_ONLY
        };
        facts.push(fact(label, String::new()));
    }
    Ok(facts)
}

/// Builds the print data from a page that holds every row. Pure: it reads nothing else.
///
/// # Errors
/// `ReportsError::Money` for an amount that does not fit, `ReportsError::Words` for a closing
/// balance that cannot be written in words, `ReportsError::Data` for a stored date that is not
/// a date.
pub fn build_print(
    page: &ReportPage,
    options: &PrintOptions,
) -> Result<ReportPrintData, ReportsError> {
    let fmt = Fmt::new(options);
    let language = options.language;

    // A ledger or a statement is signed off with its closing balance in words (FR-19.7).
    let words = if matches!(page.id, ReportId::R13 | ReportId::R14) {
        let closing = page
            .summary
            .iter()
            .find(|s| s.label_id == ids::CLOSING_BALANCE)
            .and_then(|s| match &s.cell {
                Cell::Balance { paisa, side } if *side != BalanceSide::Zero => Some(*paisa),
                _ => None,
            });
        closing.map(|amount| fmt.words(amount)).transpose()?
    } else {
        None
    };

    let mut rows = Vec::with_capacity(page.rows.len());
    for row in &page.rows {
        rows.push(PrintReportRow {
            cells: fmt.cells(&row.cells)?,
            kind: row.kind,
        });
    }
    let mut summary = Vec::with_capacity(page.summary.len());
    for line in &page.summary {
        summary.push(PrintSummaryLine {
            label_id: line.label_id.clone(),
            cell: fmt.cell(&line.cell)?,
        });
    }
    Ok(ReportPrintData {
        report: page.id,
        language,
        dir: TextDir::of(language),
        paper: options.paper,
        urdu_digits: fmt.style.urdu_digits(),
        title_id: page.title_id.clone(),
        filters: filter_facts(&fmt, page.id, &page.filters)?,
        columns: page
            .columns
            .iter()
            .map(|c| PrintReportColumn {
                key: c.key.clone(),
                title_id: c.title_id.clone(),
                kind: c.kind,
            })
            .collect(),
        summary,
        rows,
        row_count: fmt.style.count(page.total_count),
        totals_label_id: page.totals_label_id.clone(),
        totals: fmt.cells(&page.totals)?,
        words,
    })
}
