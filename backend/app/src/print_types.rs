//! The plain data of a print job, shared by the print action and the Windows driver (plan 14).
//! No platform code here, so it builds everywhere.

use std::path::PathBuf;

use mandi_services::core::printing::{Paper, PrintDoc};

/// True on the computers that can print or save a PDF: Windows (WebView2, plan 14) and, in the Mac
/// phase, a Mac (the native print window of the web view, plan 69, decision M1 of plan 60). On any
/// other system a print is answered with `ERR_PRINT_WINDOWS_ONLY` and the screen offers the preview.
pub const fn prints_here() -> bool {
    cfg!(any(windows, target_os = "macos"))
}

/// True when a print opens the system's own print window (the Mac): the user picks the printer, the
/// copies or "Save as PDF" there, so the Printing settings of printer and copies do not apply and
/// the app writes no PDF file itself (plan 69, PRD FR-19.3 Mac-phase note).
pub const fn opens_system_print_window() -> bool {
    cfg!(target_os = "macos")
}

/// Where one document goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobTarget {
    /// The printer by name; empty means the default printer.
    Printer {
        /// The printer name from the Printing settings.
        name: String,
        /// Copies, 1 to 99.
        copies: u32,
    },
    /// A PDF file at this path.
    Pdf(PathBuf),
}

/// One document to print: the hidden print window shows it, then the web view prints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// Which of the five documents (plan 14, 17e), or none for a report or an analytics view
    /// (plan d5-late): those are not one of the documents of the Printing settings.
    pub doc: Option<PrintDoc>,
    /// Its paper class, from the Printing settings.
    pub paper: Paper,
    /// The query of the print route without the job number: `bikri=1&doc=seller_bill&language=ur`.
    pub query: String,
    /// Printer or PDF.
    pub target: JobTarget,
}

/// Why a job did not print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobError {
    /// The printer was not found or is offline.
    PrinterNotFound,
    /// Anything else, with the cause for the log.
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_print_or_a_pdf_works_on_windows_and_on_a_mac_only() {
        // Plan 69, decision M1 of plan 60: the Mac phase prints through the native print window.
        assert_eq!(prints_here(), cfg!(any(windows, target_os = "macos")));
        assert_eq!(opens_system_print_window(), cfg!(target_os = "macos"));
    }
}
