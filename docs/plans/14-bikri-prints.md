# Plan 14: Seller bill, buyer slips and the پختہ slip

Deliverable: D3, plan file 14. Branch: `feature/bikri` (goal run D3+D4). Version 1.0, 15-10-2026.

## 1. Goal

A saved بکری prints three documents in Urdu or English, on the paper size Settings says: the seller bill (R1, A4), the buyer slips (R5, one slip per buyer, "print all", A5 or 80 mm) and the پختہ کھاتہ slip. Every printed number comes from the backend and equals the screen to the rupee. `Ctrl+Enter` saves, prints the ticked documents and opens the next بکری. The print foundation (shared layout, print window, Printing settings) is built here for all later prints.

## 2. Requirements

PRD FR-12.12, FR-7.36, FR-7.57, FR-19.3 to FR-19.7, R1 and R5 (PRD 19.2), Section 21 "Printing", PRD 12.2 (print count), FR-13.1 to FR-13.3 (the پختہ slip), A21, A34. NFR-URD-14 to NFR-URD-16, NFR-URD-28, NFR-SPD-14, NFR-SEC-13, NFR-USE-20. Tech Stack 1 and 4.5. The `print-and-export` skill (read it and its `reference.md`). E2E-22 (Windows). Acceptance: D3-A15, D3-A16, D3-A17 (print preview speed). The money-calculations agent reviews the amounts in words and the bill totals.

## 3. Worked numbers (the snapshots use exactly these)

The بکری 1 seller bill (PRD 4.4), Urdu and English:

| Line | Value |
|---|---|
| خام بکری / Gross Sale | 29,400 |
| کرایہ / Freight | 3,000 |
| کمیشن / Commission | 1,764 |
| مزدوری / Labour | 250 |
| منشیانہ / Munshiana | 50 |
| نقد / Cash Paid | 5,000 |
| کرایہ سٹور / Store Rent | 0 |
| فون / Phone Charge | 20 |
| کل اخراجات / Total deductions | 10,084 |
| صافی بکری / Net Sale | 19,316 |
| In words, Urdu | انیس ہزار تین سو سولہ روپے |
| In words, English | Nineteen thousand three hundred sixteen rupees only |

Buyer slips: اکرم کیلا 20 پیٹی @ 600 = 12,000, کمیشن 720, لاگا 160, total 12,880; ارشد 30 @ 580 = 17,400, 1,044, 240, total 18,684. Print all gives two slips on two pages, each starting a new page. The پختہ slip: شاہد (0300-0000000), سیب 10 @ 1,200 = 12,000, کمیشن 720, لاگا 80, total 12,800, paid (نقدی), in words "بارہ ہزار آٹھ سو روپے" / "Twelve thousand eight hundred rupees only". Dates print as DD-MM-YYYY (NFR-URD-16).

## 4. Contract

New error codes (`errors.ftl` in `ur` and `en`, `# review: pending`): `ERR_PRINT_WINDOWS_ONLY` (printing needs Windows; PDF preview is offered instead), `ERR_PRINTER_NOT_FOUND`, `ERR_PRINT_FAILED`.

### Backend (`mandi-reports` crate for print data; commands in `app/src/commands/reports.rs` and `settings.rs`; sign-in needed, NFR-SEC-13)

- `reports_bikri_print_data { bikri_id, doc: seller_bill | buyer_slips | pukhta_slip, language: ur | en, only_buyer_id? }` -> `PrintData`: the firm header in the print language (name, mandi, address, phones from Settings "Firm"), the title and every label as Fluent message IDs, the date and number, rows with values already formatted for the print (grouping by the number grouping setting, English digits unless the Urdu-digits setting is on), totals, the amount in words from `core::urdu_words` or `core::english_words` by language and grouping (FR-19.7), names through the print language (`name_ur` or `name_en`), notes and truck number as typed (FR-7.25), the paper class, `dir` and `lang`. Read-only. Built from the stored values of the saved بکری, never recalculated (B20); a test checks every printed total equals `bikri_get` and the journal.
- `reports_print { bikri_id, docs: [..], language, copies?, to: printer | pdf }` -> `PrintOutcome { printed: [doc], pdf_paths? }`. On Windows it calls the WebView2 `Print` or `PrintToPdf` on the hidden print window through `with_webview` and `webview2-com` (reference.md of the skill). On any other system it returns `ERR_PRINT_WINDOWS_ONLY` (the Mac phase; the screen then offers the PDF preview in the app). Raises `print_count` of the بکری in the same transaction as the audit row (PRD 12.2). File name for a PDF: `<document>-<number>-<language>.pdf`, for example `seller-bill-1-ur.pdf`.
- Printing settings (PRD Section 21): `settings_get_printing` / `settings_set_printing` with paper per document (A4, A5 or 80 mm; defaults: seller bill A4, buyer slip A5, پختہ slip A5, A21), printer name (empty = the default printer), copies (1), Urdu digits on Urdu prints (off, NFR-URD-14), and the list "print after save" (seller bill, buyer slips, پختہ slip; remembered per user, FR-7.57, in `user_prefs`). Each change writes an audit row (S7). `reports_list_printers()` -> `Vec<String>` (Windows; empty elsewhere).
- Amounts in words are tested for 19,316 and 12,800 in both languages (plus the existing boundary tests). Money agent reviews them.
- Tests: `insta` snapshots of the print data for the three documents in both languages from the worked day (the بکری 1 and 2 numbers above); the totals-equal-screen test; print count raised once per print; sign-in refusal; settings audit.

### Frontend (shared print layout in `frontend/src/shared/print/`, templates in `frontend/src/features/bikri/print/`)

- The shared print layout: firm header, body table, totals, amount in words, footer with "Page x of y" and the print time; `dir` and `lang` from the print language; bundled fonts only (Gulzar or Noto Nastaliq Urdu, Inter, Noto Naskh Arabic for Urdu digits); Nastaliq line height at least 1.8; numbers on the physical right edge with tabular digits through `<Num>` and `<Code>`; table headers repeat; rows never split; one literal `@page` rule per paper class (A4, A5, 80 mm). The template only lays out what it receives and calculates nothing (rule 1).
- Three templates: the A4 seller bill, the A5 and 80 mm buyer slip (print all = one slip per buyer, each on a new page), the پختہ slip.
- The hidden print window and its route; the print box (print language اردو / English, copies, the documents to print, print or PDF), opened from the بکری screen with `F9`; `Ctrl+Enter` = save, print the ticked documents of the "print after save" list, open a new بکری (FR-7.57); the ticked list is remembered. On a computer that is not Windows the box offers the PDF preview.
- A Printing tab in Settings: paper per document, printer list, copies, Urdu digits, with the four states.
- Fluent: `print.ftl` (document titles and labels) in `ur` and `en`; all labels resolved from the print language bundle; new Urdu words `# review: pending`.
- Screenshots in headless Chrome of each template in both languages (print CSS emulated) and a print-preview timing for a 15-page report layout (NFR-SPD-14, using fixture rows).
- The Windows suite gets a spec that prints the bill to PDF in both languages (written and syntax-checked; run LOCAL-ONLY).

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: print data types, command stubs, settings types, error codes, `pnpm gen:types` | backend | plan 13 built |
| T2 | Print data for three documents, words, print count, Windows print and PDF behind `cfg(windows)`, Printing settings, snapshots, money agent review | backend | T1 |
| T3 | Shared print layout, three templates, print box, `F9`, `Ctrl+Enter`, Printing settings tab, Fluent, screenshots | frontend | T1 |
| T4 | Wire to the real commands; E2E-22 flow; PDF preview path | frontend | T2, T3 |

## 6. Acceptance checklist

1. The بکری 1 seller bill snapshot shows 29,400, 10,084, 19,316 and the words in Urdu and in English (D3-A15; `cargo test -p mandi-reports`).
2. Buyer slips print all as two slips on two pages; the پختہ slip shows 12,800 paid and its words (D3-A15).
3. Every printed total equals the screen and the journal (the totals test).
4. Paper size matches in the CSS and in the print settings for A4, A5 and 80 mm; Printing settings work; print count rises (D3-A15).
5. Print preview speed for a 15-page layout under NFR-SPD-14 at the runner threshold (D3-A17).
6. E2E-22: the English and Urdu bill snapshots and the Windows spec are written; the Windows run (PDFs, WebView2 print) is LOCAL-ONLY (D3-A16, D3-A15).
7. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` exit 0; every earlier test still passes.

## 7. Out of scope

Report prints R2 onward and Excel exports (D5); the receipt slip (plan 15, it reuses this layout); the daily backup pack (D5); real printing and Urdu shaping on WebView2 (Windows phase).

## 8. Open questions

None for Umar. Team defaults: bill A4, slips A5, پختہ slip A5, one copy, English digits (A21).
