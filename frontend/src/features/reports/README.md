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

## Round 4 (plan 20, step F3): the other early reports, search and the party card

- The Sales (R2, R3, R4, R6, R7, R8, R9), Collections (R20, R21, R22, R24, R26, R27) and Business
  (R28, R30) reports use the same one screen. Nothing was added to `ReportPage`: the catalogue lists
  them and the filters come from it.
- `columns.ts` holds `COLUMN_STEPS`: a report has more columns than a 1024 px window holds, and a
  table never scrolls sideways (NFR-USE-15), so the less used columns wait for the normal or the
  wide step (F22). The print has every column. Checked at 1024, 1366 and 1920 in both languages with
  no sideways scroll and no cut cell. A name or a code with a name wraps instead of being cut.
- Filter boxes (`components/ReportFilterBar.tsx`): a report with more than five boxes folds its
  record pickers behind "More filters" in a compact window; from the normal width every box shows.
  New boxes: paid or due (R9), by month or by day (R28), and the person box.
- The person box (`ReportPersonPicker`, `reports_find_people`, PRD FR-13.9): R9 is for one-time
  people only, so its party box IS the person box. On any other report it appears when the party
  picked is a پختہ کھاتہ account, and narrows the report to one person inside it (`party.person_key`).
  The text is searched as typed and never turned into Urdu. A person can come in the address
  (`?person={...}`), whole or not at all.
- Money cells use the grid type `figure` (120 px) and counts the type `count`; a phone is `short`.
- R28 names the month from Fluent (`report-month-1` ... `12`, `monthText`). R21, R22 and R8 name the
  rows the backend leaves with an empty name (no city, no collector, no اڈا, `emptyNameMessageId`).
  R30 shows a minus sign where the backend sends a negative amount.
- Party card buttons (FR-7.65): `PartyReportLinks` (in `AppShell`) gives the shared `PartyCard` a
  function that opens R13 or R14 for the account on the card (`open-report.ts`, `partyReportSearch`).
  The range is the whole history: an empty `from` is the first day of data. Change it in one place.
- Global search (FR-5.4): `app/layout/global-search-extras.tsx` adds the one-time people and the
  screens and reports (found by their name in either language, or a report by its number) to the
  backend's accounts, بکری and vouchers. `GlobalSearch.tsx` has a few lines marked "Run B".

## Not built yet

- Sending a report to the printer and saving it as PDF: the report print DATA exists, the print JOB
  needs a generic report document in the plan 14 print path (`PrintDoc::Report`, plan 17e). The two
  buttons of the preview are dimmed and say so.
- Excel (D5-Q1).

## Older notes (plan 14)

- `api.ts` is the only file that knows the command shapes. Every command takes one `{ input }`.
- `hooks.ts`: `useBikriPrintData` (one document, never read again by itself), `usePrinters` (an
  empty list is an answer, not an error: only Windows knows its printers), `usePrintDocs`.
- Printing works on Windows only. Elsewhere `reports_print` answers `ERR_PRINT_WINDOWS_ONLY` and the
  print box offers the in-app preview (`features/bikri/print/PrintPreview.tsx`), which draws the
  same documents.
- The hidden print window may call `reports_bikri_print_data` only (its capability allows no other
  command, NFR-SEC-10). `features/bikri/print/BikriPrintPage.tsx` is that window's route.
