// The shared print layout (plan 14, Tech Stack 5.3). A module's print template builds on these
// pieces and never styles a printed page itself.
export { AFTER_SAVE_DOCS, BIKRI_PRINT_DOCS, DOC_NAME_ID, PRINT_DOCS } from './docs';
export { printingInputOf } from './printing-input';
export { PrintLanguageTo, PrintOutcomeNotice, useDocNames } from './PrintBoxParts';
export { PrintDocument, type PrintDocumentProps } from './PrintDocument';
export { PrintProvider } from './PrintProvider';
export { PrintRoot, type PrintRootProps } from './PrintRoot';
export { PrintSheet } from './PrintSheet';
export { PrintGoods } from './PrintGoods';
export { PrintSummary } from './PrintSummary';
export {
  PrintView,
  describePrintData,
  type PrintDescription,
  type PrintViewProps,
} from './PrintView';
export { PrintHeader } from './PrintHeader';
export { PrintName } from './PrintName';
export { PrintText } from './PrintText';
export { PrintR5 } from './PrintR5';
export { PrintScreenFooter } from './PrintScreenFooter';
export { PrintPartyBlock } from './PrintPartyBlock';
export { FOOTER_PROPERTY, applyPrintRoot, cssString, footerContent, stripIsolates } from './footer';
export { fontFamiliesOf, waitForPrintReady, type PrintReady } from './wait-ready';
export {
  COLUMN_HEAD_ID,
  FOOTER_LABEL_IDS,
  LAYOUT_IDS,
  PAPERS,
  PAYMENT_PAID_ID,
  R5_AMOUNT_IDS,
  type AmountRole,
  type Paper,
  type PrintAmount,
  type PrintColumn,
  type PrintContact,
  type PrintData,
  type PrintFact,
  type PrintFirm,
  type PrintFooter,
  type PrintFrame,
  type PrintHeaderData,
  type PrintLine,
  type PrintPage,
  type PrintParty,
  type PrintWords,
} from './types';
export { jobOf, positiveInt, printErrorEvent, printReadyEvent, type PrintFailed } from './window';
