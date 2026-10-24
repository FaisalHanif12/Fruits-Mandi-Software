# analytics (plan 21, PRD 18)

The تجزیات area: the views A to H and J on one screen, in tabs, against the commands
`analytics_catalog`, `analytics_run`, `analytics_lines`, `analytics_print_data`,
`analytics_views_get` and `analytics_views_save`. The views I (gardens) and K (fruit taken) and the
garden and fruit columns of F wait for D4R (the columns exist and stay empty). Analytics writes
nothing (FR-18.8) except the user's list of saved views.

- `pages/AnalyticsPage.tsx`: the one screen (routes `/analytics` and `/analytics/$view`). The
  catalogue says which filter boxes a view takes. Tabs move between the views and keep the filters.
- `store.ts`: the filters, the window of columns and the open saved view, kept above the screens so
  nothing is lost when the language changes (the grid and the chart are built again).
- `filters.ts`: what the boxes hold and how it becomes the request (every key is sent; the grouping
  only when it is not the default month). `stateFromWire` brings a saved view back into the boxes.
- `columns.ts` and `fit.ts`: a table never scrolls sideways (NFR-USE-15), so the columns shown are
  the ones that fit the window, most useful first. Views D and E have a column for each period or
  item: a window of them is shown, with "Earlier" and "Later" buttons. The print has every column.
  When a window cannot hold every column of A, B, C or H, a line says "8 of 9 columns fit" and a
  button swaps in the columns that were waiting (the average rate of E2E-11 at 1,366 px), so every
  column of PRD 18.2 can be read on screen. The choice is kept in `store.ts`.
- `chart.ts` and `components/AnalyticsChart.tsx`: the chart above each table, drawn from the rows the
  screen already has (the backend sends no chart data) through `shared/charts`. Each bar carries the
  figure the backend sent (`cellText`); the heights are numbers for the drawing only
  (`chartValueOfPaisa`). The chart shows the first ten rows and says so when it cut.
- `components/AnalyticsResults.tsx`: the table page by page with the backend's totals over the whole
  filter, the four states, the days covered (and the days compared with), the window buttons.
- Drill-down (FR-18.2): Enter or a double click on a figure sends the row's key (and the column's
  `drill` in views D and E) to `analytics_lines`; Enter on the totals row sends the whole filter.
  `components/AnalyticsLines.tsx` shows the lines with the بکری behind each; Esc comes back.
- `components/SavedViews.tsx` (FR-18.5): a list of saved views, "Save this view" asks for a name, "Delete"
  asks first. The backend keeps the list for the user and a save replaces all of it.
- `components/AnalyticsPreview.tsx`: the print preview on the shared report page (`PrintTable`). Print
  and PDF are dimmed until the print job exists; no Excel here.
- The share is a `percent` cell in hundredths (7210 is "72.10%"): written from integer text, no
  division.
- Fixtures: the backend's `analytics_*.json` are copied into `tests/fixtures` byte for byte;
  `tests/unit/analytics-api-contract.test.ts` compares them with the backend's folder.

## Views F, G and J (round 11, plan 27, PRD 18.2)

- F (by partner), G (trend) and J (trips) come from the catalogue like the others. New filter boxes:
  partner (F, G, J: one partner or all, `ReportPartnerSelect` of the reports module), the trip
  (`components/AnalyticsTripSelect.tsx`, the trips module's own list loaded whole) and the goods
  (all, consigned, trip, garden) and the trip layout (J: trip, purchase, item, day). Every new key is
  always sent (`partner_id`, `trip_id`, `goods`; `trip_view` only when it is not "trip"). The
  comparison is not offered on F, G and J, because the backend refuses it there (the catalogue does
  not list it). G and J fold their less used boxes behind "More filters" at the normal step too.
- `columns.ts`: the headed columns of J are one pair for each partner (his percent and his part of
  the result). Only D and E are windowed; J and the others show the columns that fit, the partner
  parts first, and the "other columns" button swaps the waiting ones in. A partner column is headed
  "Partner A: Part of Result". The figures that can be a loss (the result of a trip and each
  partner's part, G's trips result) are red with their sign and the word "Loss" or "Profit" (PRD E50).
- `chart.ts`: F draws each partner's profit share and net change side by side, G the money of each
  period (many thin bars for the days of a month), J the sales money and the result of each row. A
  column with no figure in any row has no bar; a loss is a bar below the line.
- Drill-down: a row of F sends `partner_id`, a row of J sends its `purchase_ids` (the totals row sends
  the whole filter).
- Tests: `analytics-views-round11.test.tsx`, `analytics-logic.test.ts`, the contract test. Windows
  spec: `49-party-sheet-analytics-round11.e2e.mjs`. Screenshot snippet: `shots-plan27.playwright.txt`.
