//! Turns the typed cells of a report or analytics page into sheet cells (PRD FR-19.3).
//!
//! Names come in the sheet language, labels from Fluent, dates as real dates, money as `Paisa`
//! for the writer to convert once (Code Standards M13). A share of a total is text, because the
//! only numbers a sheet holds are money and whole counts.

use mandi_core::dates::BusinessDate;

use super::l10n::L10n;
use super::model::{ColumnShape, XCell};
use super::writer::signed_balance;
use crate::error::ReportsError;
use crate::reports::print_data::Fmt;
use crate::reports::{ArgValue, Cell, ColumnKind, DetailPart, MessageArg};

/// What the cells of a sheet need: the formatter (names, account text, date text) and the words.
pub(crate) struct CellContext<'a> {
    pub(crate) fmt: &'a Fmt,
    pub(crate) l10n: &'a L10n,
}

/// A stored date as a real date cell.
pub(crate) fn date_cell(iso: &str) -> Result<XCell, ReportsError> {
    let day = BusinessDate::parse_iso(iso)
        .ok_or_else(|| ReportsError::Data(format!("a stored date is not a date: {iso}")))?;
    Ok(XCell::Date {
        year: day.year(),
        month: day.month(),
        day: day.day(),
    })
}

/// The shape of a column, which sets its width.
pub(crate) fn shape_of(kind: ColumnKind) -> ColumnShape {
    match kind {
        ColumnKind::Number => ColumnShape::Whole,
        ColumnKind::Money => ColumnShape::Money,
        ColumnKind::Balance => ColumnShape::Balance,
        ColumnKind::Date => ColumnShape::Date,
        ColumnKind::Text
        | ColumnKind::Label
        | ColumnKind::Doc
        | ColumnKind::Month
        | ColumnKind::Percent
        | ColumnKind::Party
        | ColumnKind::Names
        | ColumnKind::Detail
        | ColumnKind::Message => ColumnShape::Text,
    }
}

impl CellContext<'_> {
    /// The words of a month: `October 2026`.
    pub(crate) fn month_text(&self, year: i64, month: i64) -> String {
        format!(
            "{} {}",
            self.l10n.text(&format!("report-month-{month}")),
            self.fmt.style.plain(year)
        )
    }

    fn detail_text(&self, part: &DetailPart) -> Result<String, ReportsError> {
        use crate::reports::print_data::PrintDetailPart as P;
        Ok(match self.fmt.detail(part)? {
            P::Sale {
                bikri_number,
                item,
                packing,
                qty,
                rate,
                value,
                commission,
                laga,
            } => self.l10n.text_with(
                "report-detail-sale",
                &[
                    ("number", &bikri_number),
                    ("item", &item),
                    ("qty", &qty),
                    ("packing", &packing),
                    ("rate", &rate),
                    ("value", &value),
                    ("commission", &commission),
                    ("laga", &laga),
                ],
            ),
            P::Lot {
                bikri_number,
                item,
                packing,
                qty,
                gross,
                deductions,
            } => self.l10n.text_with(
                "report-detail-lot",
                &[
                    ("number", &bikri_number),
                    ("item", &item),
                    ("qty", &qty),
                    ("packing", &packing),
                    ("gross", &gross),
                    ("deductions", &deductions),
                ],
            ),
            P::CashLent { reason } => self
                .l10n
                .text_with("report-detail-cash-lent", &[("reason", &reason)]),
            P::Text { text } => text,
        })
    }

    /// One cell of a page.
    pub(crate) fn cell(&self, cell: &Cell) -> Result<XCell, ReportsError> {
        Ok(match cell {
            Cell::Empty => XCell::Empty,
            Cell::Text { text } => XCell::Text(text.clone()),
            Cell::Label { id } => XCell::Text(self.l10n.text(id)),
            Cell::Number { value } => XCell::Whole(*value),
            Cell::Money { paisa } => XCell::Money(*paisa),
            Cell::Balance { paisa, side } => XCell::Signed(signed_balance(*paisa, *side)?),
            Cell::Date { iso } => date_cell(iso)?,
            Cell::Percent { hundredths } => {
                XCell::Text(format!("{}%", self.fmt.style.hundredths(*hundredths)))
            }
            Cell::Month { year, month } => XCell::Text(self.month_text(*year, *month)),
            Cell::Party { party } => XCell::Text(self.fmt.party_text(party)),
            Cell::Names { names } => XCell::Text(self.fmt.names(names)),
            Cell::Doc { type_id, number } => {
                let label = self.l10n.text(type_id);
                XCell::Text(match number {
                    Some(n) => format!("{label} {}", self.fmt.style.count(*n)),
                    None => label,
                })
            }
            Cell::Detail { parts } => {
                let texts = parts
                    .iter()
                    .map(|p| self.detail_text(p))
                    .collect::<Result<Vec<_>, _>>()?;
                XCell::Text(texts.join("; "))
            }
            Cell::Message { id, args } => XCell::Text(self.message_text(id, args)?),
        })
    }

    /// A message written out: each argument is formatted as the sheet formats it, then the
    /// message is read from Fluent in the sheet language.
    fn message_text(&self, id: &str, args: &[MessageArg]) -> Result<String, ReportsError> {
        let mut texts: Vec<(String, String)> = Vec::with_capacity(args.len());
        for arg in args {
            let text = match &arg.value {
                ArgValue::Text { text } => text.clone(),
                ArgValue::Number { value } => self.fmt.style.count(*value),
                ArgValue::Money { paisa } => self.fmt.style.money(*paisa)?,
                ArgValue::Percent { hundredths } => self.fmt.style.bps_text(*hundredths),
                ArgValue::Names { names } => self.fmt.names(names),
                ArgValue::Message { id, args } => self.message_text(id, args)?,
            };
            texts.push((arg.name.clone(), text));
        }
        let pairs: Vec<(&str, &str)> = texts
            .iter()
            .map(|(name, text)| (name.as_str(), text.as_str()))
            .collect();
        Ok(self.l10n.text_with(id, &pairs))
    }

    pub(crate) fn cells(&self, cells: &[Cell]) -> Result<Vec<XCell>, ReportsError> {
        cells.iter().map(|c| self.cell(c)).collect()
    }
}
