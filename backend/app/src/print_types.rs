//! The plain data of a print job, shared by the print action and the Windows driver (plan 14).
//! No platform code here, so it builds everywhere.

use std::path::PathBuf;

use mandi_services::core::printing::{Paper, PrintDoc};

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
    /// Which document.
    pub doc: PrintDoc,
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
