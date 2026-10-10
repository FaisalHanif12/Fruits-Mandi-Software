---
name: print-and-export
description: Builds or changes any printed document, PDF or Excel export in the Mandi Software (seller bill, buyer slips, پختہ کھاتہ slip, receipt slip, reports R1 to R31, the daily backup pack) with Urdu or English print language, A4, A5 or 80 mm paper, amounts in words, and numbers that never flip. Use whenever something must be printed, saved as PDF or exported to Excel.
---

# Print, PDF and Excel export

A printed bill is what the seller and buyer take home. It must match the screen to the rupee, read naturally in Urdu or English, and fit the paper. PRD 19.1 (FR-19.3 to FR-19.7), FR-7.36, FR-7.57, FR-12.12, FR-15.2, FR-20.5 and Section 21 "Printing".

Code sketches for the WebView2 print call, the print CSS and the Excel writer are in [reference.md](reference.md).

## How printing works (Tech Stack 1, Architecture 14)

```
Screen: print button, F9, or Ctrl+Enter print-after-save
   |  reports_print { doc, params, language, copies? }
   v
Backend (mandi-reports): builds the print data. Every number is calculated and formatted here.
   |  print data (JSON) + print settings (printer, paper, copies) from Settings
   v
Hidden print window: a frontend print route renders the HTML template with print CSS.
   |  page ready signal
   v
Backend: WebView2 Print (printer) or PrintToPdf (PDF), through with_webview + webview2-com
```

- **Backend owns** the print data, the settings lookup and the print and PDF calls.
- **Frontend owns** the print templates in `frontend/src/features/<module>/print/` and the shared print layout in `frontend/src/shared/print/`.
- The template only lays out what it receives. **It never calculates.** Totals, amounts in words and page totals all come in the print data.
- **Words follow CLAUDE.md rule 7.** The print data carries Fluent message IDs for titles and labels (for example `print-seller-bill-title`, `bikri-summary-freight`). The template resolves them with the Fluent bundle of the print language. The backend sends values, not sentences. The only words it builds itself are the amounts in words.
- The hidden print window has its own window label. If it calls any command, add that label to the capability (see the `tauri-command` skill).

## Before you start

- Find the document in the plan and the PRD: its columns (Section 19.2 for reports), its paper size (Settings "Printing", A21: bills A4, slips A5 or 80 mm), and who receives it.
- Check its language rule: the app language by default, with a choice in the print box (FR-7.36). The backup pack uses the default-language setting.

## Build steps

1. **Print data type** in `mandi-reports` (use the `tauri-command` skill for the command). Include:
   - the firm header: name in the print language, mandi, address and phones (Settings "Firm");
   - the title as a Fluent message ID, the date range or document number, and the print time;
   - rows with values already formatted for printing;
   - totals (میزان);
   - the amount in words from `core::urdu_words` or `core::english_words`, chosen by print language and number grouping (FR-19.7);
   - the language, the direction and the paper size.
2. **Names and words in the print language.** Use `name_ur` or `name_en` through `displayName(record, printLanguage)`, not the app language (F38). The template resolves every title, label and month name from Fluent in the print language.
3. **Digits.** English digits by default. Urdu prints use Urdu digits only if the Printing setting is on (NFR-URD-14). The backend number formatter does this, not the template. The font stack for numbers includes Noto Naskh Arabic, which has the Urdu digits (Tech Stack 4.1).
4. **Template.**
   - Use the shared print layout: header, body table, totals, words, and a footer with "Page x of y" and the print time.
   - `dir` and `lang` come from the print language.
   - Use the bundled fonts only: Gulzar or Noto Nastaliq Urdu for Urdu, Inter for numbers and English. Nastaliq line height is at least 1.8, so tall letters are not clipped in table rows (NFR-URD-11).
   - Numbers sit on the physical right edge with tabular digits (F17). Mixed text uses `<Num>` and `<Code>` so `(16 @ 900)` never flips (FR-19.6, NFR-URD-28).
   - Table headers repeat on every page, and rows never split across pages.
5. **Paper.** Set the page size in the print CSS (one literal `@page` rule per paper class) **and** in the WebView2 print settings, so both agree (see reference.md):
   - A4: 210 × 297 mm;
   - A5: 148 × 210 mm;
   - 80 mm roll: 80 mm wide, with the height taken from the rendered content.
6. **Print Settings.** Printer, paper per document and copies come from Settings (Section 21). The print box can change the language and copies for one print.
7. **Print all** (R5 buyer slips): one print job, one slip per buyer, each starting on a new page.
8. **PDF.** Use the same template with PrintToPdf. The file name is `<document>-<number or date>-<language>.pdf`, for example `seller-bill-1-ur.pdf`.
9. **Excel** (`rust_xlsxwriter`, in `mandi-reports`):
   - headings from Fluent in the export language;
   - the sheet set right-to-left for Urdu;
   - money written as numbers in whole rupees with the format `#,##0`, so Excel can add them. **This needs one approved exception to the no-float rule** (see reference.md). Until Umar approves it and Code Standards and the money agent record it, ask `tech-lead` before building any Excel export with money;
   - dates written as real dates in `DD-MM-YYYY`;
   - a totals row;
   - the filters used, written above the table.
10. **Audit and count.** Printing a bill raises its print count (PRD 12.2 system fields).

## Tests

- **Print data snapshot** (`insta`) for each document, in both languages, built from the PRD worked day. For example, the بکری 1 seller bill shows خام بکری 29,400, کل اخراجات 10,084, صافی بکری 19,316 and the words "انیس ہزار تین سو سولہ روپے" / "Nineteen thousand three hundred sixteen rupees only".
- **Totals check:** every printed total equals the matching screen and ledger value. A print never has its own sum.
- **Excel check:** open the file in a test and confirm the headings, the number cells, the totals and the right-to-left flag.
- **Visual check on Windows** (`windows-verify` skill): print preview and PDF in Urdu and English, on A4, A5 and 80 mm. Look for clipped Nastaliq, flipped numbers, rows split across pages, and the page numbers.
- **Speed:** print preview of a 15-page report in under 2 seconds (NFR-SPD-14).

## Done when

- [ ] All values come from the backend print data. The template calculates nothing.
- [ ] Print language, names, words, month names and digits follow FR-7.36, FR-19.7 and NFR-URD-14/16.
- [ ] The paper size matches in CSS and in the print settings. Headers repeat; rows never split.
- [ ] Printer, paper and copies come from Settings.
- [ ] Excel: real numbers and dates, Fluent headings, right-to-left for Urdu, totals row.
- [ ] Snapshot tests in both languages pass. Checked on Windows.
