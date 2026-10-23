//! Writes sheets to an `.xlsx` file with `rust_xlsxwriter` (PRD FR-18.4, FR-19.3, FR-20.5).
//!
//! The file is written to a `.partial` file and renamed, so a failed export leaves no half file.
//! Money, counts and shares reach the library only through [`super::numbers`] (Code Standards
//! M13). Dates are real Excel dates. A balance is one signed number with a number format that
//! prints the side word, so the column still adds up.
//!
//! Layout of a sheet, top to bottom: the firm, the title, the filters and the time, a blank row,
//! the lines above the table, the heading row (frozen), every row of the table, the totals row.

use std::fs;
use std::path::{Path, PathBuf};

use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use rust_xlsxwriter::{
    ExcelDateTime, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError,
};

use super::model::{ExportColumn, ExportProgress, ExportRow, ExportSheet, ExportStage, XCell};
use super::numbers::{count_as_f64, rupees_as_f64};
use crate::error::ReportsError;

/// The most characters Excel allows in a tab name.
const SHEET_NAME_LIMIT: usize = 31;
/// Excel holds 1,048,576 rows in a sheet. The header block of ours is small.
const MAX_ROWS: usize = 1_048_000;
/// Widths are in characters.
const MIN_WIDTH: usize = 10;
const MAX_WIDTH: usize = 60;
/// How many rows are written between two progress reports.
const PROGRESS_EVERY: u64 = 500;

fn xlsx(error: XlsxError) -> ReportsError {
    ReportsError::Data(format!("xlsx: {error}"))
}

fn row_number(value: usize) -> Result<u32, ReportsError> {
    u32::try_from(value).map_err(|_| ReportsError::Data("too many rows".to_owned()))
}

fn col_number(value: usize) -> Result<u16, ReportsError> {
    u16::try_from(value).map_err(|_| ReportsError::Data("too many columns".to_owned()))
}

/// The tab name Excel accepts: at most 31 characters, none of `: \ / ? * [ ]`, not empty.
pub fn tab_name(wanted: &str, taken: &[String]) -> String {
    let cleaned: String = wanted
        .chars()
        .map(|c| match c {
            ':' | '\\' | '/' | '?' | '*' | '[' | ']' => ' ',
            other => other,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('\'').trim();
    let base: String = if trimmed.is_empty() {
        "Sheet".to_owned()
    } else {
        trimmed.chars().take(SHEET_NAME_LIMIT).collect()
    };
    let same = |candidate: &str| {
        taken
            .iter()
            .any(|t| t.eq_ignore_ascii_case(candidate) || t == candidate)
    };
    if !same(&base) {
        return base;
    }
    // A second sheet with the same name gets a number, still inside the limit.
    (2_u32..)
        .map(|n| {
            let suffix = format!(" {n}");
            let room = SHEET_NAME_LIMIT.saturating_sub(suffix.chars().count());
            let head: String = base.chars().take(room).collect();
            format!("{head}{suffix}")
        })
        .find(|candidate| !same(candidate))
        .unwrap_or(base)
}

/// The formats of one sheet.
struct Styles {
    title: Format,
    label: Format,
    header: Format,
    text: Format,
    whole: Format,
    money: Format,
    balance: Format,
    date: Format,
    bold_text: Format,
    bold_whole: Format,
    bold_money: Format,
    bold_balance: Format,
    bold_date: Format,
    total_text: Format,
    total_whole: Format,
    total_money: Format,
    total_balance: Format,
    total_date: Format,
}

fn quoted(word: &str) -> String {
    // These signs have a meaning inside a number format (a literal's end, a section break, an
    // escape, a condition or a text slot), so they are left out of the side word.
    word.chars()
        .filter(|c| !matches!(c, '"' | ';' | '\\' | '[' | ']' | '@'))
        .collect()
}

impl Styles {
    fn new(sheet: &ExportSheet) -> Self {
        let whole_format = "#,##0";
        let balance_format = format!(
            "#,##0\" {dr}\";#,##0\" {cr}\";0",
            dr = quoted(&sheet.debit_word),
            cr = quoted(&sheet.credit_word)
        );
        let date_format = "dd-mm-yyyy";
        let base = Format::new();
        let bold = Format::new().set_bold();
        let total = Format::new().set_bold().set_border_top(FormatBorder::Thin);
        Self {
            title: Format::new().set_bold().set_font_size(14_u8),
            label: bold.clone(),
            header: Format::new()
                .set_bold()
                .set_text_wrap()
                .set_align(FormatAlign::Center)
                .set_background_color(0x00E7_E6E6)
                .set_border_bottom(FormatBorder::Thin),
            text: base.clone(),
            whole: base.clone().set_num_format(whole_format),
            money: base.clone().set_num_format(whole_format),
            balance: base.clone().set_num_format(balance_format.clone()),
            date: base.set_num_format(date_format),
            bold_text: bold.clone(),
            bold_whole: bold.clone().set_num_format(whole_format),
            bold_money: bold.clone().set_num_format(whole_format),
            bold_balance: bold.clone().set_num_format(balance_format.clone()),
            bold_date: bold.set_num_format(date_format),
            total_text: total.clone(),
            total_whole: total.clone().set_num_format(whole_format),
            total_money: total.clone().set_num_format(whole_format),
            total_balance: total.clone().set_num_format(balance_format),
            total_date: total.set_num_format(date_format),
        }
    }

    fn pick(&self, cell: &XCell, weight: Weight) -> &Format {
        match (cell, weight) {
            (XCell::Whole(_), Weight::Normal) => &self.whole,
            (XCell::Whole(_), Weight::Bold) => &self.bold_whole,
            (XCell::Whole(_), Weight::Total) => &self.total_whole,
            (XCell::Money(_), Weight::Normal) => &self.money,
            (XCell::Money(_), Weight::Bold) => &self.bold_money,
            (XCell::Money(_), Weight::Total) => &self.total_money,
            (XCell::Signed(_), Weight::Normal) => &self.balance,
            (XCell::Signed(_), Weight::Bold) => &self.bold_balance,
            (XCell::Signed(_), Weight::Total) => &self.total_balance,
            (XCell::Date { .. }, Weight::Normal) => &self.date,
            (XCell::Date { .. }, Weight::Bold) => &self.bold_date,
            (XCell::Date { .. }, Weight::Total) => &self.total_date,
            (_, Weight::Normal) => &self.text,
            (_, Weight::Bold) => &self.bold_text,
            (_, Weight::Total) => &self.total_text,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Weight {
    Normal,
    Bold,
    Total,
}

fn write_cell(
    sheet: &mut Worksheet,
    row: u32,
    col: u16,
    cell: &XCell,
    format: &Format,
) -> Result<(), ReportsError> {
    match cell {
        XCell::Empty => {
            // A blank cell still carries the format, so the border of a totals row is unbroken.
            sheet.write_blank(row, col, format).map_err(xlsx)?;
        }
        XCell::Text(text) => {
            sheet
                .write_string_with_format(row, col, text, format)
                .map_err(xlsx)?;
        }
        XCell::Whole(value) => {
            sheet
                .write_number_with_format(row, col, count_as_f64(*value)?, format)
                .map_err(xlsx)?;
        }
        XCell::Money(amount) | XCell::Signed(amount) => {
            sheet
                .write_number_with_format(row, col, rupees_as_f64(*amount)?, format)
                .map_err(xlsx)?;
        }
        XCell::Date { year, month, day } => {
            let date = ExcelDateTime::from_ymd(
                u16::try_from(*year).map_err(|_| ReportsError::Data("year".to_owned()))?,
                u8::try_from(*month).map_err(|_| ReportsError::Data("month".to_owned()))?,
                u8::try_from(*day).map_err(|_| ReportsError::Data("day".to_owned()))?,
            )
            .map_err(xlsx)?;
            sheet
                .write_datetime_with_format(row, col, &date, format)
                .map_err(xlsx)?;
        }
    }
    Ok(())
}

/// How many characters a cell needs, for the column width.
fn width_of(cell: &XCell) -> usize {
    match cell {
        XCell::Empty => 0,
        XCell::Text(text) => text.chars().count(),
        XCell::Whole(_) | XCell::Money(_) | XCell::Signed(_) => 16,
        XCell::Date { .. } => 12,
    }
}

fn column_widths(columns: &[ExportColumn], rows: &[ExportRow], totals: &[XCell]) -> Vec<u16> {
    columns
        .iter()
        .enumerate()
        .map(|(at, column)| {
            let head = column.title.chars().count();
            let body = rows
                .iter()
                .filter_map(|row| row.cells.get(at))
                .chain(totals.get(at))
                .map(width_of)
                .max()
                .unwrap_or(0);
            let chars = head.max(body).saturating_add(2).clamp(MIN_WIDTH, MAX_WIDTH);
            u16::try_from(chars).unwrap_or(0)
        })
        .collect()
}

fn write_sheet(
    sheet: &mut Worksheet,
    data: &ExportSheet,
    name: &str,
    done: &mut u64,
    total: u64,
    progress: &mut dyn FnMut(ExportProgress),
) -> Result<(), ReportsError> {
    sheet.set_name(name).map_err(xlsx)?;
    sheet.set_right_to_left(data.right_to_left);
    let styles = Styles::new(data);
    let mut at: usize = 0;

    sheet
        .write_string_with_format(row_number(at)?, 0, &data.firm, &styles.title)
        .map_err(xlsx)?;
    at += 1;
    sheet
        .write_string_with_format(row_number(at)?, 0, &data.title, &styles.title)
        .map_err(xlsx)?;
    at += 1;
    for (label, value) in &data.facts {
        let row = row_number(at)?;
        sheet
            .write_string_with_format(row, 0, label, &styles.label)
            .map_err(xlsx)?;
        sheet
            .write_string_with_format(row, 1, value, &styles.text)
            .map_err(xlsx)?;
        at += 1;
    }
    at += 1;
    for (label, cell) in &data.summary {
        let row = row_number(at)?;
        sheet
            .write_string_with_format(row, 0, label, &styles.label)
            .map_err(xlsx)?;
        write_cell(sheet, row, 1, cell, styles.pick(cell, Weight::Bold))?;
        at += 1;
    }
    if !data.summary.is_empty() {
        at += 1;
    }

    let header_row = row_number(at)?;
    for (col, column) in data.columns.iter().enumerate() {
        sheet
            .write_string_with_format(header_row, col_number(col)?, &column.title, &styles.header)
            .map_err(xlsx)?;
    }
    sheet
        .set_freeze_panes(header_row.saturating_add(1), 0)
        .map_err(xlsx)?;
    at += 1;

    for row in &data.rows {
        let weight = if row.bold {
            Weight::Bold
        } else {
            Weight::Normal
        };
        let row_at = row_number(at)?;
        for (col, cell) in row.cells.iter().enumerate() {
            write_cell(
                sheet,
                row_at,
                col_number(col)?,
                cell,
                styles.pick(cell, weight),
            )?;
        }
        at += 1;
        *done += 1;
        if (*done).is_multiple_of(PROGRESS_EVERY) {
            progress(ExportProgress {
                stage: ExportStage::Writing,
                done: *done,
                total,
            });
        }
    }

    if !data.totals.is_empty() {
        let row_at = row_number(at)?;
        // The label goes in the first column the totals leave empty. A totals row that fills
        // every column has no place for it, and the label must never cover a total.
        let label_at = data
            .totals
            .iter()
            .position(|cell| matches!(cell, XCell::Empty))
            .ok_or_else(|| {
                ReportsError::Data("the totals row has no cell for its label".to_owned())
            })?;
        for (col, cell) in data.totals.iter().enumerate() {
            if col == label_at {
                sheet
                    .write_string_with_format(
                        row_at,
                        col_number(col)?,
                        &data.totals_label,
                        &styles.total_text,
                    )
                    .map_err(xlsx)?;
            } else {
                write_cell(
                    sheet,
                    row_at,
                    col_number(col)?,
                    cell,
                    styles.pick(cell, Weight::Total),
                )?;
            }
        }
    }

    for (col, width) in column_widths(&data.columns, &data.rows, &data.totals)
        .into_iter()
        .enumerate()
    {
        sheet
            .set_column_width(col_number(col)?, width)
            .map_err(xlsx)?;
    }
    Ok(())
}

/// The path must be absolute, end in `.xlsx`, and sit in a folder that exists.
///
/// # Errors
/// `ReportsError::Rejected` with `ERR_OUT_OF_RANGE` on field `path`.
pub fn check_path(path: &str) -> Result<PathBuf, ReportsError> {
    let candidate = PathBuf::from(path);
    let good = candidate.is_absolute()
        && candidate
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("xlsx"))
        && candidate.file_stem().is_some_and(|stem| !stem.is_empty())
        && candidate.parent().is_some_and(Path::is_dir);
    if good {
        Ok(candidate)
    } else {
        Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "path",
        )]))
    }
}

fn file_error(error: &std::io::Error) -> ReportsError {
    ReportsError::File {
        full: error.kind() == std::io::ErrorKind::StorageFull,
        detail: error.to_string(),
    }
}

/// Writes the sheets to `path` (through a `.partial` file) and returns the size in bytes.
///
/// # Errors
/// `ReportsError::Rejected` if a sheet has too many rows for Excel; a number that does not fit;
/// `ReportsError::File` if the file cannot be written.
pub fn write_workbook(
    sheets: &[ExportSheet],
    path: &Path,
    progress: &mut dyn FnMut(ExportProgress),
) -> Result<u64, ReportsError> {
    if sheets.iter().any(|s| s.rows.len() > MAX_ROWS) {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters",
        )]));
    }
    let total = sheets.iter().map(|s| s.rows.len() as u64).sum::<u64>();
    let mut workbook = Workbook::new();
    let mut names: Vec<String> = Vec::new();
    let mut done = 0_u64;
    progress(ExportProgress {
        stage: ExportStage::Writing,
        done,
        total,
    });
    for data in sheets {
        let name = tab_name(&data.name, &names);
        names.push(name.clone());
        let sheet = workbook.add_worksheet();
        write_sheet(sheet, data, &name, &mut done, total, progress)?;
    }
    let partial = {
        let mut name = path.as_os_str().to_owned();
        name.push(".partial");
        PathBuf::from(name)
    };
    let saved = workbook.save(&partial).map_err(xlsx);
    let result = saved.and_then(|()| fs::rename(&partial, path).map_err(|e| file_error(&e)));
    if result.is_err() {
        let _ = fs::remove_file(&partial);
    }
    result?;
    progress(ExportProgress {
        stage: ExportStage::Done,
        done,
        total,
    });
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| file_error(&e))
}

/// Paisa of a balance with its side as one signed amount: debit positive, credit negative
/// (PRD C13: balance = debit − credit).
///
/// # Errors
/// `ReportsError::Data` for a negative amount, or a side `Zero` with an amount (the side would
/// hide the amount);
/// `ReportsError::Money` if the amount cannot be negated.
pub fn signed_balance(
    amount: Paisa,
    side: mandi_core::accounts::BalanceSide,
) -> Result<Paisa, ReportsError> {
    use mandi_core::accounts::BalanceSide;
    // A balance is the size of what is owed plus the side it stands on, never a negative size.
    if amount.is_negative() {
        return Err(ReportsError::Data(
            "a balance with a side has a negative amount".to_owned(),
        ));
    }
    let size = amount;
    match side {
        BalanceSide::Dr => Ok(size),
        BalanceSide::Cr => Ok(size.checked_neg()?),
        BalanceSide::Zero if size == Paisa::ZERO => Ok(Paisa::ZERO),
        BalanceSide::Zero => Err(ReportsError::Data(
            "a balance with no side has an amount".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandi_core::accounts::BalanceSide;

    fn paisa(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    #[test]
    fn a_balance_is_one_signed_amount() {
        // Akram owes 2,880 (debit); Karim is owed 10,160 (credit); a nil balance is 0.
        assert_eq!(
            signed_balance(paisa(2_880), BalanceSide::Dr).unwrap(),
            paisa(2_880)
        );
        assert_eq!(
            signed_balance(paisa(10_160), BalanceSide::Cr).unwrap(),
            paisa(-10_160)
        );
        assert_eq!(
            signed_balance(Paisa::ZERO, BalanceSide::Zero).unwrap(),
            Paisa::ZERO
        );
        assert!(signed_balance(paisa(5), BalanceSide::Zero).is_err());
        // A negative size would flip the side without a word: refused.
        assert!(signed_balance(paisa(-5), BalanceSide::Dr).is_err());
        assert!(signed_balance(paisa(-5), BalanceSide::Cr).is_err());
    }

    #[test]
    fn tab_names_fit_excel() {
        assert_eq!(tab_name("Full Sales Register", &[]), "Full Sales Register");
        let long = "x".repeat(60);
        assert_eq!(tab_name(&long, &[]).chars().count(), 31);
        assert_eq!(tab_name("a:b/c?d*e[f]g", &[]), "a b c d e f g");
        assert_eq!(tab_name("   ", &[]), "Sheet");
        let first = tab_name("Report", &[]);
        let second = tab_name("report", &[first]);
        assert_eq!(second, "report 2");
        // The suffix never pushes the name over the limit.
        let taken = vec!["y".repeat(31)];
        let fixed = tab_name(&"y".repeat(40), &taken);
        assert!(fixed.chars().count() <= 31 && fixed.ends_with(" 2"));
    }

    #[test]
    fn the_side_word_cannot_break_a_number_format() {
        assert_eq!(quoted("بنام"), "بنام");
        assert_eq!(quoted("D\"r;[x]@"), "Drx");
    }

    #[test]
    fn a_path_must_be_absolute_xlsx_in_a_real_folder() {
        let dir = std::env::temp_dir();
        let good = dir.join("export-test.xlsx");
        assert!(check_path(&good.display().to_string()).is_ok());
        for bad in [
            "relative.xlsx".to_owned(),
            dir.join("report.csv").display().to_string(),
            dir.join("no-such-folder")
                .join("a.xlsx")
                .display()
                .to_string(),
            dir.display().to_string(),
            String::new(),
        ] {
            assert!(
                matches!(check_path(&bad), Err(ReportsError::Rejected(_))),
                "{bad}"
            );
        }
    }
}
