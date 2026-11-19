# Plan 69: Feature 9, printing on the Mac

Deliverable D8, item 9. Requirements: PRD v1.32 FR-12.12, FR-19.3 Mac-phase note. Decision M1 in plan 60.

## 1. Goal

On the Mac, print preview, the print dialog and saving as PDF work from the same templates as Windows. The Windows path does not change. No new `unsafe` (CLAUDE.md rule 5); no new dependency.

## 2. What is known

`print_job.rs` and `print_report.rs` return `ERR_PRINT_WINDOWS_ONLY` when not on Windows. `print_windows.rs` opens a hidden window on the print route, waits for `print-ready-<job>`, prints or saves, closes it. Tauri's `WebviewWindow::print()` opens the native dialog on macOS with no `unsafe`; the dialog's PDF button saves a PDF.

## 3. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | `print_mac.rs`, compiled on macOS only: same ready handshake, then `print()` on a window of the print route. Print jobs and report prints stop answering `ERR_PRINT_WINDOWS_ONLY` on the Mac. `list_printers` stays empty on the Mac, and the screen hides the printer choice there. Audit row as on Windows. | backend |
| T2 | Screens: Print and PDF on a Mac open the dialog; a short hint says "choose PDF, then Save as PDF" in both languages. Remove the "works only on Windows" text where it no longer applies. | frontend |
| T3 | A script `scripts/print/pdf-from-route.mjs` that turns print routes into PDFs through headless Chrome, for R1, R5 and R35 in Urdu. Evidence for plan 63 T4 and D8-A6. | frontend |

## 4. Acceptance (D8-A12)

Unit tests on the Mac path. The PDFs of T3 shown with their extracted text. The hand check of the real dialog (one click, one PDF) is listed for Umar in the report, since no automatic test can click a native dialog.
