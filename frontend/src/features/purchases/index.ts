// The public face of the purchases module (Tech Stack 5.3). Other modules import only from here.
// The purchases of a trip are made, listed and written off on the trip screen (feature `trips`,
// PRD FR-7.67), so the pieces it needs are exported here: the fields of the form, its draft and
// preview, the card of one purchase and the write-off and cancel boxes. The list of باقی مال and
// the sale book use the write-off boxes, the list of purchases with stock left and the number
// of a purchase too.
export { CancelPurchaseDialog } from './components/CancelPurchaseDialog';
export { OwnWriteOffDialog, UndoOwnWriteOffDialog } from './components/OwnWriteOffDialogs';
export { PurchaseDetail } from './components/PurchaseDetail';
export { PurchaseFieldsForm } from './components/PurchaseFieldsForm';
export { PurchaseNumbersCard } from './components/PurchaseNumbersCard';
export {
  PURCHASE_FIELD_PATHS,
  buildPurchase,
  emptyPurchaseFields,
  nextPurchaseFields,
  purchaseIsFilled,
  textOrNull,
  type PurchaseCommon,
} from './draft';
export {
  readAgainAfterPurchase,
  usePurchase,
  usePurchasePreview,
  usePurchasesList,
  usePurchasesReadyBy,
  useSavePurchase,
} from './hooks';
export { TRIP_PURCHASE_COLUMNS, purchaseNumberText, supplierText } from './list-columns';
export type {
  OwnStockRow,
  OwnStockTarget,
  PurchaseFields,
  PurchaseInput,
  PurchasePreview,
  PurchaseRow,
  PurchaseSaved,
  TripRef,
} from './types';
export { purchasesPreview } from './api';
