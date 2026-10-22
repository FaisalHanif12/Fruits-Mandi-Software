// The public side of the `reports` module (Tech Stack 5.2): what other modules may import.
export {
  reportsBikriPrintData,
  reportsListPrinters,
  reportsPrint,
  reportsPrintVoucher,
  reportsVoucherPrintData,
} from './api';
export {
  useBikriPrintData,
  usePrintDocs,
  usePrintVoucher,
  usePrinters,
  useVoucherPrintData,
} from './hooks';

// ---- Plan 20: the report engine. ----
export { reportsCatalog, reportsPrintData, reportsRun } from './api';
export { REPORT_PAGE_SIZE, useReportCatalog, useReportPages, useReportPrintData } from './hooks';
export { parseReportSearch, type ReportSearch } from './search';
