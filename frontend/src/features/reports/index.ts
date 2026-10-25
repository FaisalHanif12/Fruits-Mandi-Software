// The public side of the `reports` module (Tech Stack 5.2): what other modules may import.
export {
  reportsBikriPrintData,
  reportsCollectionPrintData,
  reportsListPrinters,
  reportsPrint,
  reportsPrintCollection,
  reportsPrintHeader,
  reportsPrintVoucher,
  reportsVoucherPrintData,
} from './api';
export {
  useBikriPrintData,
  useCollectionPrintData,
  usePrintCollection,
  usePrintDocs,
  usePrintFrame,
  usePrintFrameOf,
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

// ---- Plan 21: what the analytics module borrows (cells of a report are cells of a view). ----
export { cellText, monthText, nameWithTag, partyText, sideText } from './cells';
export { ReportPersonPicker } from './components/ReportPersonPicker';
export { ReportPartnerSelect } from './components/ReportPartnerSelect';
export { PrintPreviewFrame } from './components/PrintPreviewFrame';
export { usePrintJob } from './components/usePrintJob';
export { LANGUAGE_VALUES, isReportAddress, paperOf, readJsonObject } from './print/params';
export { ReportPrintPage } from './print/ReportPrintPage';
export {
  ReportPrintDocument,
  type PrintHeading,
  type PrintTable,
} from './print/ReportPrintDocument';
export {
  listColumnOf,
  totalsFor,
  type ReportRowView,
  type RowLike,
  type SignMode,
} from './columns';
