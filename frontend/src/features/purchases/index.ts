// The public face of the purchases module (Tech Stack 5.3). Other modules import only from here.
// The purchases of a trip are made, listed and written off on the trip screen (feature `trips`,
// PRD FR-7.67), so the pieces it needs are exported here: the fields of the form, its draft and
// preview, the card of one purchase and the write-off and cancel boxes. The list of باقی مال and
// the sale book use the write-off boxes, the list of purchases with stock left and the number
// of a purchase too.
export { CancelPurchaseDialog } from './components/CancelPurchaseDialog';
export { OwnWriteOffDialog, UndoOwnWriteOffDialog } from './components/OwnWriteOffDialogs';
export { PurchaseDetail } from './components/PurchaseDetail';
export { PurchaseEntryFieldsForm } from './components/PurchaseEntryFieldsForm';
export { PurchaseNumbersCard } from './components/PurchaseNumbersCard';
export {
  PurchaseEntryNumbersCard,
  type EntryLineNumbers,
} from './components/PurchaseEntryNumbersCard';
export {
  PURCHASE_FIELD_PATHS,
  buildPurchaseEntry,
  emptyPurchaseFields,
  nextPurchaseFields,
  purchaseIsFilled,
  textOrNull,
  type PurchaseCommon,
} from './draft';
export {
  readAgainAfterPurchase,
  useChangePurchaseSplit,
  usePurchase,
  usePurchaseEntryPreview,
  usePurchasesList,
  useSavePurchaseEntry,
} from './hooks';
export { TRIP_PURCHASE_COLUMNS, purchaseNumberText, supplierText } from './list-columns';
export type {
  OwnStockRow,
  OwnStockTarget,
  PurchaseEntryFields,
  PurchaseEntryInput,
  PurchaseEntryPreview,
  PurchaseEntrySaved,
  PurchaseRow,
  PurchaseSaved,
  SplitInput,
  SplitSet,
  TripRef,
} from './types';
export { purchasesEntryPreview } from './api';
export { GoodsLinesGrid, addGoodsLine } from './components/GoodsLinesGrid';
export {
  LINE,
  filledLines,
  goodsLineColumns,
  lineCellErrors,
  lineIsBlank,
  lineOfPath,
  lineText,
  lineTextOrNull,
  arrivalLineInputs,
  linesAreFilled,
  linesText,
  purchaseLineInputs,
  resetLines,
  type GoodsKind,
} from './lines';
