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

export { useOpenRowSource } from './open-source';

// ---- Plan 20: the report engine. ----
export { reportsCatalog, reportsPrintData, reportsRun } from './api';
export { useReportCatalog, useReportPages, useReportPrintData } from './hooks';
export { PartyReportLinks } from './components/PartyReportLinks';
export { useOpenReport, partyReportSearch, type OpenReport } from './open-report';
export { parseReportSearch, type ReportSearch } from './search';

// ---- Plan 21: what the analytics module borrows (cells of a report are cells of a view). ----
export { cellText, monthText, nameWithTag, partyText, sideText } from './cells';
// ---- Plan 49 (ask the app): the answer's tables are drawn with the report screens' own cell
// renderer, so a cell reads the same in the panel as on its report. ----
export { emptyNameMessageId, messageText, percentText } from './cells';
// The print of a report with the filters an answer read it with (plan 49).
export { reportsPrintReport } from './api';
export { ReportPartnerSelect } from './components/ReportPartnerSelect';
// Plan 68: the print of ادھاری تفصیل (R5), opened from the sale book's second print button.
export { BuyerBillsPreview } from './components/BuyerBillsPreview';
export { PrintPreviewFrame } from './components/PrintPreviewFrame';
export { usePrintJob } from './components/usePrintJob';
export { LANGUAGE_VALUES, isReportAddress, paperOf, readJsonObject, sortOf } from './print/params';
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

// ---- Plan 66 (party ledgers): level 3 is the R35 slip of one party and one date, with its print. ----
export { PartyDayPreview, PartyDaySlip } from './components/PartyDaySlip';
export { partyDayFilters } from './party-day';
