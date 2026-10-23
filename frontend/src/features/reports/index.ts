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
export {
  PEOPLE_RESULT_LIMIT,
  REPORT_PAGE_SIZE,
  useReportCatalog,
  useReportPages,
  useReportPrintData,
  usePeopleSearch,
} from './hooks';
export { reportsFindPeople } from './api';
export { PartyReportLinks } from './components/PartyReportLinks';
export { useOpenReport, partyReportSearch, type OpenReport } from './open-report';
export { parseReportSearch, type ReportSearch } from './search';
export { personFromHit, type ReportPerson } from './filters';
