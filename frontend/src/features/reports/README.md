# reports (plan 14)

The print commands (module `reports`, Tech Stack 5.2): `reports_bikri_print_data`, `reports_print`
and `reports_list_printers`. The report screens R2 onward come in D5.

- `api.ts` is the only file that knows the command shapes. Every command takes one `{ input }`.
- `hooks.ts`: `useBikriPrintData` (one document, never read again by itself), `usePrinters` (an
  empty list is an answer, not an error: only Windows knows its printers), `usePrintDocs`.
- Printing works on Windows only. Elsewhere `reports_print` answers `ERR_PRINT_WINDOWS_ONLY` and the
  print box offers the in-app preview (`features/bikri/print/PrintPreview.tsx`), which draws the
  same documents.
- The hidden print window may call `reports_bikri_print_data` only (its capability allows no other
  command, NFR-SEC-10). `features/bikri/print/BikriPrintPage.tsx` is that window's route.

Plan 17e adds `reports_collection_print_data`, `reports_print_collection` and `reports_print_header`
(`reportsCollectionPrintData`, `reportsPrintCollection`, `reportsPrintHeader`; hooks
`useCollectionPrintData`, `usePrintCollection`, `usePrintFrame`). `reports_print_header` is the ONE
command every report print asks for its header and footer (title and facts in, `PrintFrame` out); the
layout is `shared/print` (`PrintRoot`, `PrintHeader`). The collection list template and its print box
are in `features/vouchers/print/`.
