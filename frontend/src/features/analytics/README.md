# analytics (plan 21, PRD 18)

The تجزیات area: the views A to E and H on one screen, in tabs, against the commands
`analytics_catalog`, `analytics_run`, `analytics_lines`, `analytics_print_data`,
`analytics_views_get` and `analytics_views_save`. Garden, trip, goods and partner filters and the
views F, G, I, J and K wait for plans 16b and 17 (D5 late part). Analytics writes nothing (FR-18.8)
except the user's list of saved views.

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
