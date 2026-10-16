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
