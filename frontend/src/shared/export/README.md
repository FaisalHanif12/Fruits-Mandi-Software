# export (D5-Q1, backend round 10)

The Excel export of a report or an analytics view: one shared hook, one button, one set of notices, so the
two modules behave the same (NFR-USE-10).

- `useExcelExport.ts`: `start(language)` opens the system save box (`chooseSavePath`, a name from the report
  id and the day, `.xlsx` added if the box did not), then runs the module's command with the screen's
  filters AS THEY ARE IN THE BOXES NOW and a new job id, and listens to the event `export-progress` for
  that job only (reading, writing with done and total, done). The states are idle, choosing, running,
  done (path and row count) and failed (the backend's refusal). Esc answers at the overlay level: it does
  nothing while the file is written (leaving the screen would lose the result) and closes a finished
  notice. The export goes on in the backend if the screen is left.
- `ExcelExport.tsx`: `ExcelExportButton` (beside Print preview, and in the print preview beside PDF and
  Print, where the sheet takes the print language chosen there) and `ExcelExportNotices` (progress with the
  browser's own bar, the result with the path and the rows, or the refusal). Refusals: `path` and
  `filters` (ERR_OUT_OF_RANGE, no range to show) have words of their own; `job_id` and ERR_OVERFLOW are the
  plain "something went wrong"; ERR_DISK_FULL and a filter problem use the common messages.
- `export.css`: the bar's look from the tokens (imported by `styles/globals.css`).
- The module wrappers are `reportsExportExcel` (features/reports/api.ts) and `analyticsExportExcel`
  (features/analytics/api.ts). Texts: `i18n/*/export.ftl`. Nothing here works out a figure: the counts are
  the backend's, and the app never opens or reads the file back.
