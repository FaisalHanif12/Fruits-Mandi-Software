# dashboard (plan 19, PRD Section 9)

The screen after sign-in, and the attention list that the bell and the dashboard share.

## What is here

- `pages/DashboardPage.tsx`: greeting, a note when a new version is ready, quick actions, the tiles,
  the partner strip, today's بکری, the oldest buyer dues, today's tasks, the two mini charts.
- `components/`: one file for each part (`DashboardTiles`, `PartnerStrip`,
  `TodayBikriList`, `OldestDues`, `QuickActions`, `DashboardCharts`, `AttentionLines`, `KindIcon`,
  `UpdateNote`). Every part opens the screen behind it (FR-9.1).
- `api.ts`: `dashboard_get` and `assist_attention`, both read only. `hooks.ts`: the two queries.
- `destinations.ts`: where a tile, a line or a name leads, as plain data (`Destination`), and the one
  hook that opens it (`useOpenDestination`). Tests say where a click goes by comparing destinations.
- `attention.ts`: the Fluent message of each kind (`attention-<kind>`, and `attention-<kind>-aged`
  when the backend also names an age). `charts.ts`: what the two charts draw.

## Rules the screen keeps

- Every figure is the backend's. The dashboard adds up nothing: the tiles, the lists and the charts
  are written with `Num` and `formatPaisa`. Heights in a chart are numbers for the drawing only.
- One call (`dashboard_get`) fills everything but the greeting. When it
  fails the page shows one error with a retry; the greeting and the quick actions are still there.
- A tile or a figure that is below zero, or past its date, also says so in words (NFR-USE-12).
- The dashboard and the bell are read again after every save: `app/refresh-after-save.ts` watches
  the mutation cache (300 ms after the last save), and the bell also asks once a minute (NFR-SPD-22).
- The gardens tile (PRD 9.1, FR-12.37) is in `DashboardTiles`: the running balance of the open gardens
  and how many are open, from the same `dashboard_get` call as the other tiles (`gardens_open`,
  `gardens_balance_paisa`). It opens `/gardens`. Under the partners, `components/OpenGardens.tsx` lists
  each open garden (name, running balance, rent due) from `Dashboard.gardens`; a name opens that
  garden. With no open garden the tile says so and the section is not drawn.
- The new-version note comes from `useStartupUpdateCheck` (settings), asked once when the app starts.
  It is only a note, here and in the bell; installing stays in Settings.

## Tests and screenshots

`dashboard-screen.test.tsx`, `dashboard-bell.test.tsx`, `dashboard-words.test.ts`,
`dashboard-charts.test.ts`, `dashboard-api-contract.test.ts`; Windows spec `48-dashboard.e2e.mjs`;
screenshot snippet `tests/e2e/shots-plan26.playwright.txt` on the development page `/dev/dashboard`
(flags in `tests/helpers/dev-bridge-dashboard.ts`).

## The four dues tiles (plan 40, PRD 9.1)

گاہکوں کے ذمے, بیوپاریوں کے ذمے, کل اگراہی and ڈوبی رقم (with the number of people) come from
`DashboardTiles` as the backend sends them. The dead tile opens R38 (`{ kind: 'dead-amount' }` in
`destinations.ts`). The development page flags `?dead` and `?recovered` show the E2E-44 numbers.

## Plan 76: the commission page (FR-9.3) and the buyer dues page (FR-9.4)

- The tile آج کا کمیشن opens `/commission` (`pages/CommissionPage.tsx`): one month at a time, a row for each day
  (seller and buyer commission, لاگا, total, shared income, each partner's share), the month totals, and a day that
  opens in place with the postings behind it. The month is in the address (`?month=`); previous and next month,
  go to month, Page Up and Page Down. The partner columns wait for the partner code (the box shows in their place).
  Commands: `dashboard_commission_month`, `dashboard_commission_day`, `dashboard_commission_print_data`.
- The tile گاہکوں کے ذمے opens `/dues` (`pages/BuyerDuesPage.tsx`): code, name with tag, city, balance with its
  side and the day it last changed, with the backend's total (the tile's figure and the R20 total). Search and
  sort go to the backend; Enter opens his ledger. Commands: `dashboard_buyer_dues`,
  `dashboard_buyer_dues_print_data`.
- Both have a print preview (F9) through `TablePrintPreview` of the reports module. Tests:
  `commission-page.test.tsx`, `buyer-dues-page.test.tsx`.

## Plan 77: a period and a party on the commission page (FR-9.3)

- The page takes a period (one date, a month, a year or a range of dates) and one buyer or one seller
  (`components/CommissionFilterBar.tsx`: a period choice, an all / buyer / seller toggle, and an `AccountPicker` of that
  side only). Everything is in the address (`commission-search.ts`: `?period=year&year=2026&side=seller&party=...`;
  `?month=` alone is the page of plan 76), so a language switch, a reload and the way back land on the same page.
  The Owner moves a month, a year or a date with the buttons under the table (`MonthNav`, `YearNav`, `DayNav`,
  Page Up and Page Down) or with its "go to" box (`components/CommissionNav.tsx`); a range is paged.
- Under a party the columns follow the side (seller: sale, commission charged to him, commission on his goods;
  buyer: purchase value, buyer commission, لاگا, total) and there are no partner columns. A year is one row for each
  month (its date is `YYYY-MM`); a click on a month opens that month for the same party. A side with nobody picked
  asks for a party and reads nothing. `CommissionDays.tsx` draws the rows and the postings behind a day.
- Print and Excel take the same period and party (`commission-hooks.ts`: `commissionInput`); the backend names the
  party in the print header. The answer of the question panel (source `commission`) opens the page with its party
  and days through `commissionSearchOfSource`.
- Tests: `commission-party-filter.test.tsx`, `commission-search.test.ts`, `assistant-commission-button.test.tsx`.

## Plan 77: the kharch page (FR-9.5)

- The tile آج کا خرچہ opens `/kharch` (`pages/KharchPage.tsx`): two parts for one period, the shop's kharch (one row for
  each head with the whole shop's total) and each partner's own kharch (ذاتی خرچہ, with a total), with a partner filter.
  The period box is the commission page's (`components/PeriodFilterBox.tsx`: one date, a month, a year, a range; the
  month, year and date have buttons under the table, Page Up and Page Down). The period and the partner are in the address
  (`kharch-search.ts`: `?period=year&year=2026&partner=2`); `kharch-wire.ts` turns them into the filters of view M.
- The page and the analytics view M read the same thing: `analytics_run` view M, once for each part (`expense_part` shop
  or partner), drawn by one board (`analytics/components/KharchBoard.tsx`), so they can never differ. A figure opens its
  entries (date, entry, name, reason, amount); a line opens its voucher. The partner part waits for the partner code
  (ERR_PARTNER_LOCKED shows the code box in its place; the shop part shows). Print, PDF and Excel are one for each
  part, through the analytics commands.
- Tests: `kharch-page.test.tsx`, `dashboard-screen.test.tsx` (the tile). Development page: `#/dev/kharch`.

## Plan 76 point 10: the tile of garden owners and others

The tile "Garden owners' and others' dues" (`dashboard_get`: `others_dues_paisa`, `others_people_count`) replaced the
last backup tile. It opens `/others-dues` (`features/cashbook/pages/OthersDuesPage.tsx`). The backup state is in the
status bar and the attention list (NFR-BAK-10), so nothing about the backup is lost.

## Plan 79: every tile opens its own page (PRD FR-9.1, FR-9.6 to FR-9.8, FR-19.1)

Each tile opens a page that shows exactly what the tile counts, and the total under its columns is the tile's
figure, the backend's (CLAUDE.md rule 1). The pages load one page of rows at a time with the totals of the whole
filter (FR-19.11). The party filter of every page starts at سب (`allChoice` of the shared picker). Each row opens
that person's history (the opening balance first, FR-19.16).

| Tile                                        | Page (route)                                    | Backend read                                             | A row opens                                               |
| ------------------------------------------- | ----------------------------------------------- | -------------------------------------------------------- | --------------------------------------------------------- |
| بیوپاریوں کو ادائیگی باقی, بیوپاریوں کے ذمے | `SellerDuesPage` (`/dues/sellers?side=shop_owes | seller_owes`)                                            | `dashboard_seller_dues`                                   | party ledgers, side seller |
| کل اگراہی                                   | `TotalDuesPage` (`/dues/total`), four sections  | `dashboard_total_dues` + the four lists                  | party ledgers, R13, or the ادھار رقم screen               |
| ادھار رقم باقی                              | `LentOpenPage` (`/dues/lent`)                   | `dashboard_lent_open`                                    | the ادھار رقم screen, his name chosen                     |
| پیشگی باقی                                  | `AdvancesOpenPage` (`/dues/advances`)           | `advances_report` with `only_open`                       | the پیشگی screen, his name chosen                         |
| آج کی نقدی                                  | `CashSalesPage` (`/cash-sales?month=YYYY-MM`)   | `dashboard_cash_sales_month`, `dashboard_cash_sales_day` | a day opens its buyers in place; the بکری number opens it |

The other tiles keep the pages of plans 76 to 78 (`/bikri/today`, `/commission`, `/rokar`, `/dues`, R38, `/others-dues`,
`/kharch`, `/bikri/stock`, `/trips`, `/gardens`). The books check tile is gone: its result is the line of the Day tab
of Settings (`BooksCheck`) and an item of the attention list when the last check found a problem. Files:
`seller-dues-*`, `other-dues-columns.ts`, `dues-hooks.ts`, `dues-links.ts`, `components/TotalDuesSections.tsx`,
`components/CashSalesDay.tsx`. Words: `i18n/*/dues.ftl` and `cash_sales.ftl`. Tests: `seller-dues-page`,
`total-dues-page`, `lent-advances-open-pages`, `cash-sales-page`, `dashboard-screen`. Development pages:
`#/dev/dues/sellers|total|lent|advances|cash` (flags `?empty`, `?down`, `?full`).

### Print, PDF and Excel of these pages (PRD FR-19.10, FR-19.3)

Every page has a Print button (`F9`) and an Excel button (`print-view.tsx`: `usePrintView`, `PrintButtons`). Print shows the
preview in place of the table (`TablePrintPreview`): every row of the filter on screen, read from the backend in the print
language by `dashboard_*_print_data`, printed or saved as a PDF by `dashboard_*_print`, and made into a sheet by
`tables_export_excel` (kinds `seller_dues`, `total_dues`, `lent_open`, `advances_open`, `cash_sales_month`,
`cash_sales_day`). `Esc` leaves the preview first. The hidden print window knows the kinds `seller-dues`, `total-dues`,
`lent-open`, `advances-open`, `cash-sales-month` and `cash-sales-day` (`app/print-kinds`). The cash sales page prints its
month, and one day from the Print button inside the open row. Tests: `dues-pages-print.test.tsx`, `print-kinds.test.tsx`.

## Plan 83: the sales page (FR-9.9)

- The tile آج کی بکری opens `/sales` (`pages/SalesPage.tsx`), built like the commission page, not پچھلی بکری (that stays in the
  بکری menu, FR-12.46). One month at a time (previous and next, go to month, Page Up and Page Down), or one date, a year or
  a range. A row for each day (a month for each row of a year): the number of بکری, نگ, خام بکری, the buyers' bill, نقدی and
  ادھاری, with the totals row from the backend. Today's row carries the word "Today". A day opens in place
  (`components/SalesDays.tsx`) with its بکری, their lot names and figures; the بکری number opens that بکری. A month row of a
  year opens that month. The page adds nothing up (CLAUDE.md rule 1).
- Filters (`components/SalesFilterBar.tsx`): the period box of the commission page, All / buyer / seller and an
  `AccountPicker` of that side (every party box starts at All, FR-19.1), one item (`Picker`, scope item) and the goods
  (all, consigned, trip, garden). All of it is in the address (`sales-search.ts`: `?period=year&year=2026&side=seller&party=...&item=...&goods=trip`).
  A side with nobody picked asks for him and reads nothing. A filter at its start is asked as nothing, so the plain page sends
  `{ period: { kind: 'month' } }`.
- Commands: `dashboard_sales_period`, `dashboard_sales_day`, `dashboard_sales_print_data`, `dashboard_sales_print`, and the Excel kind
  `sales_period`. The hidden print window knows the kind `sales` (`app/print-kinds`). The Back parent of `/sales` is
  the dashboard (`shared/navigation/back-parents.ts`).
- Files: `sales-search.ts`, `sales-hooks.ts`, `components/SalesDays.tsx`, `components/SalesFilterBar.tsx`,
  `pages/SalesPage.tsx`. Words: `i18n/*/sales.ftl`. Tests: `sales-page.test.tsx`, `sales-search.test.ts`, `dashboard-screen`,
  `back-pages`, `print-kinds`. Development page: `#/dev/sales` (flags `?empty`, `?down`).
