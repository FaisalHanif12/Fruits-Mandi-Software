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

## Round 7: R12, R29, R31, R32

- R12 (cash book), R29 (expenses), R31 (partner statement) and R32 (money lent) are the same one
  screen. The catalogue says what each takes: a new filter kind `partner` (`ReportFilters.partner_id`)
  and `party_required` meaning the partner for R31 (`canRun` in `filters.ts`). R29 has an "all partners"
  choice, R31 does not and says "pick a partner first" until one is chosen.
- The partner box is `ReportPartnerSelect` (the short list of partners, `usePartnerList`). R32's party
  box is the people who can borrow (control 0103, `ControlAccountPicker`, `partyBoxOf` returns `lent`).
- A new cell and column kind `message` (a Fluent ID with arguments the backend names): `cellText`
  works the arguments out (text, number, money, percent like `formatBps`, names through `displayName`,
  and a message inside a message) and the print writes the same sentence in the print language
  (`printMessageText`). R31 lines are such sentences, so a line reads the same in both languages and
  on paper (PRD FR-17.10).
- `scripts/copy-backend-fixtures.mjs` copies the backend's fixtures byte for byte; run it again when
  the backend changes a type or a fixture. `PENDING_REPORTS` (tests/helpers/reports-real.ts) names the
  reports the backend lists that have no screen or words here yet. It is empty now: all 32 reports
  of the catalogue have a screen, words and a test.
- Tests: `reports-screens-round7.test.tsx`, plus the contract and filter tests. Windows spec:
  `46-reports-month.e2e.mjs`.

## Round 8: R10, R15, R23, R25, R33, R34

- **R10 stock** has no fruit column yet (round 11). **R23** and **R25** have no mode, bank or cheque
  column; a cancelled voucher is listed with its status and left out of the money.
- **R15 vouchers** has the kind box (`ReportVoucherTypeSelect`, filter `voucher_type`). With no kind
  picked the totals row is empty and the backend sends `summary` lines, the saved money of each
  kind. They are drawn above the table (`ReportSummary`) and never added up here. With a kind picked
  there is one total and no summary.
- **R33 trip profit or loss** has the layout box (`ReportTripViewSelect`, filter `trip_view`): trip
  (the default), item, day, month or partner. A trip heading is a group row. The layout is sent every
  time and is not a filter to clear. The partner layout shows the share as a percent (`percentText`)
  and the money columns. A result can be negative: `result` and `net` are signed columns
  (`SIGNED_COLUMNS` in `columns.ts`). They draw as a balance, with the sign, the word (loss or
  profit, `report-result-*`) in the unit slot and red for a loss (`tone`, see `shared/grid`). A loss
  is never shown by colour alone.
- **R34 day summary** is a flat list of rows [date, label, count, amount]: one block for each day.
  A heading row (`report-r34-partner-shares`) comes before the partner rows. It has no totals row:
  `totalsFor` returns no row when every total cell is empty, and the print draws no `tfoot` then.
- A text cell that holds a date or a range ("سفر-1 10-10-2026", the heading of a trip) keeps its
  digits in order in Urdu: `shared/grid/NumberRuns` puts each run in its own isolate, on screen and
  on paper.
- Tests: `reports-screens-round8.test.tsx`, `number-runs.test.tsx`, plus the contract and filter
  tests. Windows spec: `47-reports-round8.e2e.mjs`. Screenshot snippet: `shots-plan25.playwright.txt`.

## Remembered filters (round 9, PRD FR-7.45)

- Each report keeps the filters the Owner last used, under the screen name `report.<id>` (for example
  `report.r20`), through `shared/screen-state`. `remember.ts` says what is kept (never the one-time
  person or any money) and reads it back, leaving out what can no longer be read. An address with
  filters in it (the party card's buttons, the dashboard's partner card) wins over what was kept.
  Clearing the filters forgets them. Tests: `remembered-filters.test.tsx`.

## R35, the party day sheet (round 11, plan 27, PRD FR-19.12, E2E-43)

- One report, two layouts, both on the generic screen. With no party it is the table of every party
  of the day: a group row for the buyers and one for the sellers (each with its own figures), a blank
  column for the collector's note (the wide step and the print), and under the table the shop's total
  اگراہی and the sellers' credit, debit and balance. The table has no pinned totals row (the backend
  sends all its totals empty, like R34): a row under both sections would read as the total of all of
  them. The buyers' total is in their group row and in the line under the table. With a party it is his slip: the lines of his day
  and, right under them, the footer lines in the order the backend sends.
- Enter on a party of the table opens his slip (a one-time person with his key, so the slip is his and
  not the whole account's); "All parties" goes back with the day still typed. The day box is called
  "Day" and the line under the boxes says "Sheet for 10-10-2026" (not "Balances as of").
- The footer is under the lines, like the client's own slip, on screen (`ReportResults`) and in the
  print (`ReportPrintDocument`, `data.report === 'R35'`); every other report has its lines above the
  table. The words of the footer are backend message IDs; the glossary has the four terms the client's
  slip uses (تازہ بنام, سابقہ رقم, ٹوٹل رقم, بقایا). A buyer's slip says "Receipt" (وصولی,
  `report-r35-received`), a seller's says "Payment" (ادائیگی, `report-r35-paid`).
- Tests: `reports-screens-round11.test.tsx`, the round 11 block of `reports-api-contract.test.ts`.
  Windows spec: `49-party-sheet-analytics-round11.e2e.mjs`.

## R36, the gardens report, and the garden rows of R34 (round 11, PRD 19.2 R36, FR-12.37)

- R36 is on the generic screen. Boxes: the days (the gardens made in them), the item, the partner, one
  garden (`GardenFilterSelect`) and the layout (`ReportGardenViewSelect`): by garden, 15 columns, or by
  partner, 7. The layout is sent only when it is by partner. `columns.ts` chooses the steps by layout
  (`GARDEN_LAYOUT_STEPS`, `GARDEN_PARTNER_LAYOUT_STEPS`): the print has every column. Enter on a garden
  opens its screen. The balance column carries the word "Profit" or "Loss" (red for a loss) only on the
  row of a CLOSED garden (FR-12.38, `CLOSED_ONLY_COLUMNS`, `signModeOf`, `isClosedGardenRow` in
  `columns.ts`). An open garden's running balance starts below zero by design, so it keeps only its
  sign, and the totals row has no word. The by-partner layout has no state cell, so it keeps signs only.
- R34 has two rows more for each day (what was paid into the gardens, what came in); they are
  information and are not added to the cash lines. Tests: the garden block of
  `reports-screens-round11.test.tsx` and of `reports-api-contract.test.ts`.

## R1, the sale book, and R5, the credit buyers' details (round 12, PRD 19.2 R1 and R5, FR-12.12)

- Both are on the generic screen. R5 takes the days and the usual party boxes. Its table writes each
  sale line the way the client's own print does, "(item qty . rate)" (`report-detail-sale-short`,
  `shortSale` in `cells.ts` and `columns.ts`), and has a numbered first column (`report-col-serial`).
  The print draws the same lines (`buyersList` in `ReportPrintDocument.tsx`).
- R1 is the lots of ONE بکری, so it cannot run without one: the new box kind `bikri` shows
  `ReportBikriBox` (the number typed, the backend's own `bikri_find`, the first hit highlighted, so a
  number and Enter takes it). `filters.ts` keeps it in `bikri` and `bikriText`, sends `bikri_id` (always
  in the wire, null when not picked) and `canRun` asks for it. Until one is picked the screen says
  `report-pick-bikri` and draws no rows.
- R1's print is the bill itself: once a بکری is picked the preview button gives way to the shared
  `PrintAction` of the sale book (its box, its F9, its languages and paper). R1 has its own column
  steps in `COLUMN_STEPS`: the lot's number charges stay behind the wider steps, and the print has all.
- R5 "Print all slips" (PRD 19.2 R5): the button next to the print preview opens `BuyerBillsPreview`
  (the same frame as the report preview, with the print language and the paper). It asks the backend's
  own command `reports_buyer_bills_print_data` (`reportsBuyerBillsPrintData`, hook
  `useBuyerBillsPrintData`) for every buyer of the filter, and `print/BuyerBillsPrintDocument.tsx` draws
  one `print-sheet` for each buyer (print.css starts each on a new page): the shared sheet header
  (`print/ReportSheetHeader.tsx`), who the slip is for, his lines "(item qty . rate)", his totals row and
  his total نگ. What all the slips come to is in the preview only (`data-slips-total`), not on a buyer's
  paper. The cells are the shared `print/PrintCell.tsx` that the report print uses too. Sending to the
  printer and saving as PDF wait for the report print job (plan 17e), like the report preview.
- The rate in the detail line is written plain, "2600", through the `plain` option of `formatPaisa`
  (`money(amount, true)` of the list context): the one place that skips the grouping, as on the client's
  own print. Tests: `reports-screens-round12.test.tsx`, the R1 and R5 blocks of
  `reports-api-contract.test.ts`, `money-format.test.ts`.

## R37, the fruit register, and the fruit parts of R10, R34 and R31 (round 13, PRD 16.7, 19.2 R37)

- R37 is on the generic screen. Its boxes are the days, "Partner or guests", the item and the garden.
  The partner box (`ReportPartnerSelect`, props `guests` and `guestsOnly`) is labelled
  `fruit-filter-for` and also offers the shop's guests: a partner OR the guests, never both. The
  choice is kept in `guestsOnly` (remembered like the other boxes) and sent as `guests_only: true`
  only when it is set and no partner is chosen (`filtersToWire`); the backend refuses both together.
  The columns are the date, the document (FRT-n), who it was for (a name, or the word guests), the
  seller of the lot, the item, the packing, the weight (a short figure, only for items sold by
  weight), the quantity, the rate, the value and where it came from; `COLUMN_STEPS.R37` keeps the
  first five at every step. Enter on a line opens the fruit screen (`/rokar/fruit`), which lists the
  entry (it has no page for one entry yet).
- R10 has a `fruit_taken` column between the write-offs and the balance (the backend works the
  balance out); R10 shows it from the normal step, and moved its document column to the wide step to
  make room. R34 has one row after the gardens' two (`report-r34-fruit-taken`: the count is نگ, the
  amount the value). R31 writes the garden and fruit lines with the partners module's own sentences
  (`partners-statement-*`): nothing in the screen changed for them.

## Not built yet

- Sending a report to the printer and saving it as PDF: the report print DATA exists, the print JOB
  needs a generic report document in the plan 14 print path (`PrintDoc::Report`, plan 17e). The two
  buttons of the preview are dimmed and say so.
- (Excel is built: the buttons are `shared/export`.)

## Older notes (plan 14)

- `api.ts` is the only file that knows the command shapes. Every command takes one `{ input }`.
- `hooks.ts`: `useBikriPrintData` (one document, never read again by itself), `usePrinters` (an
  empty list is an answer, not an error: only Windows knows its printers), `usePrintDocs`.
- Printing works on Windows only. Elsewhere `reports_print` answers `ERR_PRINT_WINDOWS_ONLY` and the
  print box offers the in-app preview (`features/bikri/print/PrintPreview.tsx`), which draws the
  same documents.
- The hidden print window may call `reports_bikri_print_data` only (its capability allows no other
  command, NFR-SEC-10). `features/bikri/print/BikriPrintPage.tsx` is that window's route.
