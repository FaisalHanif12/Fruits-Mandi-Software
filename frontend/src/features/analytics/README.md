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
- `chart.ts` and `components/AnalyticsChart.tsx`: the charts above each table, drawn from the
  `charts` the backend sends with every page (plan 38 R4): they cover the whole filter and never
  change with the page or the sort. A ranked chart is bars with an `others` bar for the rest, a trend
  is one group of bars for each period, view H has one chart for each section (named by it), views F
  and K have none. They are drawn through `shared/charts`. Each bar carries the figure the backend sent
  (`cellText`); the heights are numbers for the drawing only (`chartValueOfPaisa`). A button folds the
  charts (kept for the screen); in a window not higher than 800 (1366 x 768 and 1024 x 600) they start
  folded, so the table, its totals and its page buttons are in view.
- `components/AnalyticsResults.tsx`: the table one page at a time (the shared pager) with the backend's
  totals over the whole filter, the four states, the days covered (and the days compared with), the window buttons.
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

## View I and the garden slice (round 11, PRD 18.2 I, FR-12.37)

- View I (gardens) is one row for each garden: its state (open or closed), what was paid from the
  shop and from pockets, the sales in our shop and in other shops, the other money in, the commission
  and the result, then a percent and a result for each partner of the gardens. The row key is the
  garden's account (`seller_id`), sent back to `analytics_lines` for the sale lines of its goods. The
  result and each partner's part carry "Profit" or "Loss" (red for a loss) only on the row of a CLOSED
  garden (FR-12.38, `signModeOf` in `columns.ts`); an open garden's figure below zero keeps only its sign,
  because a garden starts below zero by design (FR-12.37). The totals row has no word.
- The garden box (`GardenFilterSelect` of the gardens module, loaded whole like the trip box) is a main
  box of view I and one of the "More filters" boxes of A to E, G and H (it narrows the sales to that
  garden's goods). F and J refuse a garden, so the catalogue does not list it there and none is sent.
  The comparison is not offered on I either.
- F has money in its gardens, trips and fruit columns. G has the movement of the garden balances after
  the trips' result (a day of payments is below the line).
- `columns.ts`: view I and J share `PARTNER_PLACES` (a partner's result right after the garden's or
  trip's result, his percent after the first sales column). The state of a garden ranks right after
  its name and is sized as one short word.

## View K, the fruit taken, and the fruit of view F (round 13, PRD 18.2 K, 16.7)

- View K is one row for each partner (or the shop's guests) and item: the quantity, the value, and the
  value from our shop and from other shops (`for`, `item`, `qty`, `value`, `own_shop`, `other_shops`).
  The `for` cell is the partner's name, or the word `fruit-for-guests`. The row key is a partner or
  `guests: true`, with the item, sent back to `analytics_lines`; the lines behind a row are the
  columns of the fruit register R37, so `LINES_PRIORITY` in `columns.ts` lists `for`, `value`, `rate`
  and `source` early. The chart (`fruitChart` in `chart.ts`) is one stack for each row, from our shop
  and from other shops. Its boxes are the days, the item, the partner, the garden and the goods; the
  goods box (`AnalyticsFilterBar`, prop `view`) does not offer trip goods on K, which the backend would
  refuse. There is no comparison and no side.
- The garden, trip and fruit columns of F carry `drill` (the kind of money, `AnalyticsKey.money`):
  `AnalyticsResults` sends it as `column_key` with the row's key, or with the whole filter for the
  totals row, so a number opens the lines behind it (date, document, partner, amount). Nothing else
  changed for them: the screen opens any figure that way. F lists its garden, trip and fruit columns
  before the leftover `other` column.
