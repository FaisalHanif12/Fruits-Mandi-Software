//! The Windows print and PDF calls and the printer list (plan 14, Tech Stack 1, reference.md of
//! the `print-and-export` skill). Compiled on Windows only, and only through `webview2.rs`,
//! which is the one place where `unsafe` is allowed (plan 01 T1.6).
//!
//! The functions here are small and do one COM call each. They know nothing about Tauri,
//! windows or the database: `print_windows.rs` opens the hidden window, waits until the page is
//! ready and then calls [`start`] on that window's web view.
//!
//! Method names are the Win32 COM names of WebView2 (`ICoreWebView2_16::Print`,
//! `ICoreWebView2_7::PrintToPdf`), as in `webview2-com` 0.39. This code cannot run on a Mac; it
//! is type-checked against the Windows target and checked on real Windows in the Windows phase
//! (`windows-verify`, LOCAL-ONLY until then).

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_PRINT_MEDIA_SIZE_CUSTOM, COREWEBVIEW2_PRINT_STATUS_PRINTER_UNAVAILABLE,
    COREWEBVIEW2_PRINT_STATUS_SUCCEEDED, ICoreWebView2_7, ICoreWebView2_16,
    ICoreWebView2Controller, ICoreWebView2Environment, ICoreWebView2Environment6,
    ICoreWebView2PrintSettings, ICoreWebView2PrintSettings2,
};
use webview2_com::{PrintCompletedHandler, PrintToPdfCompletedHandler};
use windows::Win32::Graphics::Printing::{
    EnumPrintersW, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL, PRINTER_INFO_4W,
};
use windows_core::{Interface, PCWSTR};

/// Where the page goes.
#[derive(Debug, Clone)]
pub enum Target {
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

/// What to print and on which paper. Sizes are in inches, as the web view wants them.
#[derive(Debug, Clone)]
pub struct Request {
    /// Printer or PDF.
    pub target: Target,
    /// Page width.
    pub width_in: f64,
    /// Page height. For the 80 mm roll this is the height of the rendered content.
    pub height_in: f64,
    /// Margins in inches: top, right, bottom, left. They equal the `@page` margins of the print
    /// CSS, so the two never add up.
    pub margins_in: [f64; 4],
}

/// Why a print did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The printer was not found or is offline (`ERR_PRINTER_NOT_FOUND`).
    PrinterUnavailable,
    /// Anything else, with the cause for the log (`ERR_PRINT_FAILED`).
    Other(String),
}

fn other(error: impl std::fmt::Display) -> Failure {
    Failure::Other(error.to_string())
}

/// A string as the null-terminated UTF-16 that COM wants. Keep the vector alive for the call.
fn wide(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain(std::iter::once(0)).collect()
}

/// Starts the print or the PDF of the page in this web view and returns at once. WebView2 calls
/// back on the thread that owns the web view when it is done; the answer goes through `done`.
/// Call it from the closure of `with_webview`, which runs on that thread.
///
/// # Errors
/// `Failure::Other` if a COM call fails before the job started (then `done` gets nothing).
pub fn start(
    controller: &ICoreWebView2Controller,
    environment: &ICoreWebView2Environment,
    request: &Request,
    done: Sender<Result<(), Failure>>,
) -> Result<(), Failure> {
    // SAFETY: `controller` and `environment` are live WebView2 objects given to us by Tauri on
    // the web view's own thread. Every call below is a plain COM method with numbers, booleans
    // or a null-terminated string that outlives the call (`printer` and `path` stay alive until
    // the end of the block). The handlers own what they use; they hold only a `Sender`.
    unsafe {
        let webview = controller.CoreWebView2().map_err(other)?;
        let settings = print_settings(environment, request)?;
        match &request.target {
            Target::Printer { name, copies } => {
                let extra: ICoreWebView2PrintSettings2 = settings.cast().map_err(other)?;
                extra
                    .SetCopies(i32::try_from(*copies).unwrap_or(1))
                    .map_err(other)?;
                // An empty name leaves the printer out, so the default printer is used.
                let printer = wide(OsStr::new(name));
                if !name.is_empty() {
                    extra
                        .SetPrinterName(PCWSTR(printer.as_ptr()))
                        .map_err(other)?;
                }
                let handler = PrintCompletedHandler::create(Box::new(move |result, status| {
                    let outcome = if status == COREWEBVIEW2_PRINT_STATUS_SUCCEEDED {
                        result.map_err(other)
                    } else if status == COREWEBVIEW2_PRINT_STATUS_PRINTER_UNAVAILABLE {
                        Err(Failure::PrinterUnavailable)
                    } else {
                        Err(other(format!("print status {}", status.0)))
                    };
                    // The receiver may have given up (timeout); nothing to do then.
                    done.send(outcome).ok();
                    Ok(())
                }));
                let webview16: ICoreWebView2_16 = webview.cast().map_err(other)?;
                webview16.Print(&settings, &handler).map_err(other)
            }
            Target::Pdf(path) => {
                let file = wide(path.as_os_str());
                let handler = PrintToPdfCompletedHandler::create(Box::new(move |result, ok| {
                    let outcome = match result {
                        Ok(()) if ok => Ok(()),
                        Ok(()) => Err(other("the PDF was not written")),
                        Err(error) => Err(other(error)),
                    };
                    done.send(outcome).ok();
                    Ok(())
                }));
                let webview7: ICoreWebView2_7 = webview.cast().map_err(other)?;
                webview7
                    .PrintToPdf(PCWSTR(file.as_ptr()), &settings, &handler)
                    .map_err(other)
            }
        }
    }
}

/// The print settings of one job: our page size, our margins, no browser header and footer (the
/// template has its own), and backgrounds on (lines and shading print).
///
/// # Safety
/// `environment` must be a live WebView2 environment.
unsafe fn print_settings(
    environment: &ICoreWebView2Environment,
    request: &Request,
) -> Result<ICoreWebView2PrintSettings, Failure> {
    // SAFETY: plain COM property setters with numbers and booleans, on objects we just made.
    unsafe {
        let factory: ICoreWebView2Environment6 = environment.cast().map_err(other)?;
        let settings = factory.CreatePrintSettings().map_err(other)?;
        settings.SetPageWidth(request.width_in).map_err(other)?;
        settings.SetPageHeight(request.height_in).map_err(other)?;
        let [top, right, bottom, left] = request.margins_in;
        settings.SetMarginTop(top).map_err(other)?;
        settings.SetMarginRight(right).map_err(other)?;
        settings.SetMarginBottom(bottom).map_err(other)?;
        settings.SetMarginLeft(left).map_err(other)?;
        settings
            .SetShouldPrintHeaderAndFooter(false)
            .map_err(other)?;
        settings.SetShouldPrintBackgrounds(true).map_err(other)?;
        let extra: ICoreWebView2PrintSettings2 = settings.cast().map_err(other)?;
        // Our width and height, not the printer's default paper.
        extra
            .SetMediaSize(COREWEBVIEW2_PRINT_MEDIA_SIZE_CUSTOM)
            .map_err(other)?;
        Ok(settings)
    }
}

/// The names of the printers installed on this computer (local and network), sorted, each once.
///
/// # Errors
/// `Failure::Other` if Windows refuses the list.
pub fn list_printers() -> Result<Vec<String>, Failure> {
    let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
    // PRINTER_INFO_4W is the lightest level: the name, the server and some flags.
    const LEVEL: u32 = 4;
    let (mut needed, mut returned) = (0_u32, 0_u32);
    // SAFETY: the first call has no buffer and only asks how many bytes are needed (it "fails"
    // with insufficient buffer by design). The second call gets a buffer of that size, made of
    // u64 so its start is aligned for the structs. Windows fills the buffer with
    // `returned` structs whose string pointers point inside the same buffer, which stays alive
    // while we read them. Reads are done with `read_unaligned`, one struct at a time.
    unsafe {
        EnumPrintersW(
            flags,
            PCWSTR::null(),
            LEVEL,
            None,
            &mut needed,
            &mut returned,
        )
        .ok();
        if needed == 0 {
            return Ok(Vec::new());
        }
        let bytes = usize::try_from(needed).map_err(other)?;
        let mut buffer = vec![0_u64; bytes.div_ceil(8)];
        let slice = std::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast::<u8>(), bytes);
        EnumPrintersW(
            flags,
            PCWSTR::null(),
            LEVEL,
            Some(slice),
            &mut needed,
            &mut returned,
        )
        .map_err(other)?;
        let first = buffer.as_ptr().cast::<PRINTER_INFO_4W>();
        let mut names = Vec::new();
        for index in 0..usize::try_from(returned).map_err(other)? {
            let info = first.add(index).read_unaligned();
            if let Ok(name) = info.pPrinterName.to_string() {
                names.push(name);
            }
        }
        names.sort();
        names.dedup();
        Ok(names)
    }
}
