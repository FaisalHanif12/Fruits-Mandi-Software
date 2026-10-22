# reports (plans 14 and 20)

The print commands of plan 14 (`reports_bikri_print_data`, `reports_voucher_print_data`,
`reports_print`, `reports_list_printers`) and the report engine of plan 20 (`reports_catalog`,
`reports_run`, `reports_print_data`), with the رپورٹ area built on it.

## The report area (plan 20, PRD 19)

- `pages/ReportsHomePage.tsx`: the catalogue grouped as PRD 19.2 (Sales, Accounts, Collections,
  Business). Only groups that have a report show. ↑ ↓ move, Enter opens.
- `pages/ReportPage.tsx`: the ONE screen every report uses (route `/reports/$id`). The catalogue
  says which filter boxes a report takes; the backend sends its columns, rows and totals. A new
  report needs no new screen: it appears in the catalogue and this page draws it.
- `filters.ts`: what the boxes hold and how it becomes the filters the backend reads (every key is
  sent; a box the report does not take is sent empty). `search.ts`: the address of a report can
  carry `party`, `from`, `to`, `as_of` and `period` (the party card's buttons use it, FR-7.65).
- `cells.ts` and `columns.ts`: a cell and a column of any report as the shared `ListGrid` wants
  them. The screen writes what the backend sent (money through `formatPaisa`, names through
  `displayName`); it adds up nothing.
- `components/`: `ReportFilterBar` (the boxes), `ReportResults` (the four states, the table, the
  totals row, "show more rows"), `ReportSummary` and `ReportRange` (the lines above the table),
  `ReportPreview` (the print preview).
- `print/ReportPrintDocument.tsx`: the printed report on the shared print layout (firm header,
  filters, rows, totals row, amount in words, page footer). The in-app preview and the hidden print
  window (later) draw this same page.

## Rules the screen keeps

- Lists load 100 rows at a time (`REPORT_PAGE_SIZE`, Code Standards B31). The totals row is the
  backend's, over the whole filter, never a sum of the rows on screen.
- A filter change asks the backend once, 250 ms after the last key. A half-typed date is refused
  under its own box and the rows already on screen stay.
- Filters live in the page's state, above the grid, so a language switch loses nothing.
- A row opens its record (Enter): بکری, voucher, or the ledger of an account.

## Not built yet

- Sending a report to the printer and saving it as PDF: the report print DATA exists, the print JOB
  needs a generic report document in the plan 14 print path (`PrintDoc::Report`, plan 17e). The two
  buttons of the preview are dimmed and say so.
- Excel (D5-Q1), the Sales, Collections and Business reports (plan 20 step B3).

## Older notes (plan 14)

- `api.ts` is the only file that knows the command shapes. Every command takes one `{ input }`.
- `hooks.ts`: `useBikriPrintData` (one document, never read again by itself), `usePrinters` (an
  empty list is an answer, not an error: only Windows knows its printers), `usePrintDocs`.
- Printing works on Windows only. Elsewhere `reports_print` answers `ERR_PRINT_WINDOWS_ONLY` and the
  print box offers the in-app preview (`features/bikri/print/PrintPreview.tsx`), which draws the
  same documents.
- The hidden print window may call `reports_bikri_print_data` only (its capability allows no other
  command, NFR-SEC-10). `features/bikri/print/BikriPrintPage.tsx` is that window's route.
