# Print and export: reference

Check every API name below against the crate and package versions in `Cargo.lock` and `pnpm-lock.yaml`. The WebView2 and `webview2-com` documentation are the final word.

## WebView2 print settings (Rust, Windows only)

The print call runs on the hidden print window's webview, through Tauri's `with_webview`, which gives access to the platform webview on Windows. From there, get `ICoreWebView2` and the environment, and use:

| Need | WebView2 API |
|---|---|
| Create settings | `ICoreWebView2Environment6::CreatePrintSettings` |
| Print to a printer, silently | `ICoreWebView2_16::Print(settings, handler)` |
| Save as PDF | `ICoreWebView2_7::PrintToPdf(path, settings, handler)` |
| Printer name | `ICoreWebView2PrintSettings2::SetPrinterName` |
| Copies | `ICoreWebView2PrintSettings2::SetCopies` |
| Paper size | `SetPageWidth` / `SetPageHeight`, in **inches** |
| Margins | `SetMarginTop/Bottom/Left/Right`, in inches |
| No browser header and footer | `SetShouldPrintHeaderAndFooter(false)`; our template has its own |
| Keep backgrounds and lines | `SetShouldPrintBackgrounds(true)` |

Method names above are the Win32 COM names used through `webview2-com`. (`PrintAsync` and `PrintToPdfAsync` are the .NET names of the same calls.)

Paper sizes in inches: A4 is 8.27 × 11.69, A5 is 5.83 × 8.27, and the 80 mm roll is 3.15 wide. For the roll, measure the rendered content height in the print window (`document.documentElement.scrollHeight` in CSS pixels, at 96 per inch) and pass that height in inches.

Run the print call off the main thread, return a clear error code if the printer is missing or offline (`ERR_PRINTER_NOT_FOUND`), and never block the window. On non-Windows development machines, the print command returns `ERR_PRINT_WINDOWS_ONLY` and the screen offers a PDF preview instead.

## Print CSS (shared print layout)

```css
/* One literal rule per paper class. var() does not work inside @page. */
@page a4  { size: A4;  margin: 12mm 10mm 14mm 10mm; }
@page a5  { size: A5;  margin: 8mm 8mm 10mm 8mm; }
@page r80 { size: 80mm 297mm; margin: 3mm 4mm; }   /* the real roll height comes from the print settings */
.paper-a4 { page: a4; }  .paper-a5 { page: a5; }  .paper-80mm { page: r80; }

/* The page label is literal text. The template writes this small style block per print,
   with the label resolved from Fluent in the print language ("صفحہ" or "Page"). */
@page a4 { @bottom-center { content: "Page " counter(page) " / " counter(pages); } }
@media print {
  html, body { background: #fff; }
  table { border-collapse: collapse; width: 100%; }
  thead { display: table-header-group; }   /* repeat the header on every page */
  tr, .no-split { break-inside: avoid; }    /* never split a row */
  .num { direction: ltr; text-align: end; font-variant-numeric: tabular-nums; }
  .ur { font-family: 'Gulzar', 'Noto Nastaliq Urdu', serif; line-height: 1.9; }
  .en { font-family: 'Inter', sans-serif; }
  .num { font-family: 'Inter', 'Noto Naskh Arabic', sans-serif; }   /* Naskh supplies ۰-۹ */
}
.paper-80mm { width: 72mm; font-size: 11pt; }  /* printable width of an 80 mm roll */
```

Page counters in `@page` margin boxes need a recent Chromium-based WebView2. Confirm "Page x of y" prints correctly on the client's machine during `/windows-verify`. If it does not, the print data must carry the page split and the template prints the footer in each page block.

## Print data shape (example: seller bill, PRD 4.4)

```json
{
  "doc": "seller_bill",
  "language": "ur",
  "dir": "rtl",
  "paper": "A4",
  "firm": { "name": "مہر عدنان اینڈ برادرز", "mandi": "نیو سبزی منڈی کوٹ رادھا کشن" },
  "titleId": "print-seller-bill-title",
  "number": "1",
  "date": "10-10-2026",
  "printedAt": "10-10-2026 17:40",
  "seller": { "code": "0010", "name": "رحیم" },
  "rows": [
    { "buyer": "اکرم ٹول ٹیکس", "qty": "20", "rate": "600", "amount": "12,000" },
    { "buyer": "ارشد ملتانی", "qty": "30", "rate": "580", "amount": "17,400" }
  ],
  "deductions": [
    { "labelId": "bikri-summary-freight", "amount": "3,000" }, { "labelId": "bikri-summary-commission", "amount": "1,764" },
    { "labelId": "bikri-summary-labour", "amount": "250" }, { "labelId": "bikri-summary-munshiana", "amount": "50" },
    { "labelId": "bikri-summary-cash-paid", "amount": "5,000" }, { "labelId": "bikri-summary-phone", "amount": "20" }
  ],
  "gross": "29,400", "totalDeductions": "10,084", "net": "19,316",
  "netInWords": "انیس ہزار تین سو سولہ روپے"
}
```

Every value is already formatted text, and every label is a Fluent message ID. The template resolves the IDs in the print language and places the values. Names (firm, seller, buyers) are data, already in the print language. The amount in words is the one sentence the backend builds.

## Excel (rust_xlsxwriter)

```rust
let mut workbook = Workbook::new();
let sheet = workbook.add_worksheet();
sheet.set_name(&sheet_name)?;                      // 31 characters at most
if language == Language::Ur { sheet.set_right_to_left(true); }

let money = Format::new().set_num_format("#,##0");
let date = Format::new().set_num_format("dd-mm-yyyy");
let head = Format::new().set_bold();

sheet.write_string_with_format(0, 0, &l10n.get("reports-filters"), &head)?;
// ... headings from Fluent, then rows:
sheet.write_number_with_format(row, col, rupees_as_f64(amount_paisa)?, &money)?;
```

**Status: approved by Umar on 23-10-2026 (D5-Q1), recorded in Code Standards M13 and the money agent.** Excel stores every number as a floating-point value, so `rupees_as_f64` is the one allowed exception; the money agent's forbidden-pattern search flags it, and that one hit is expected.

`rupees_as_f64` is the one place where money becomes a floating-point number. It divides whole paisa by 100 with integer division first, checks the value is below 2^53 (so the conversion is exact), and only then converts. A unit test covers the maximum amount. Never read an Excel value back into the app.
