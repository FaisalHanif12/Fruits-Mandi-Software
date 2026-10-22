// The real answers and requests of the backend for plan 16b (T4a): trips, purchases, trip costs and
// sales in another shop. The backend wrote them in its own test (backend/crates/services/tests/
// trips_fixtures.rs); they are copied here, never edited, and read as the generated types. Money in
// them is paisa, as on the wire. The story is the PRD worked numbers of E2E-34 and E2E-41.

import type { OtherSaleInput } from '../../src/bindings/OtherSaleInput';
import type { OtherSalePreview } from '../../src/bindings/OtherSalePreview';
import type { OtherSaleSaved } from '../../src/bindings/OtherSaleSaved';
import type { PurchaseCancelInput } from '../../src/bindings/PurchaseCancelInput';
import type { PurchaseCancelled } from '../../src/bindings/PurchaseCancelled';
import type { PurchaseInput } from '../../src/bindings/PurchaseInput';
import type { PurchasePreview } from '../../src/bindings/PurchasePreview';
import type { PurchaseRow } from '../../src/bindings/PurchaseRow';
import type { PurchaseSaved } from '../../src/bindings/PurchaseSaved';
import type { PurchaseWriteOffDone } from '../../src/bindings/PurchaseWriteOffDone';
import type { PurchaseWriteOffInput } from '../../src/bindings/PurchaseWriteOffInput';
import type { PurchaseWriteOffUndoInput } from '../../src/bindings/PurchaseWriteOffUndoInput';
import type { PurchaseWriteOffUndone } from '../../src/bindings/PurchaseWriteOffUndone';
import type { PurchasesListInput } from '../../src/bindings/PurchasesListInput';
import type { PurchasesList } from '../../src/bindings/PurchasesList';
import type { TripCostInput } from '../../src/bindings/TripCostInput';
import type { TripCostPreview } from '../../src/bindings/TripCostPreview';
import type { TripCostSaved } from '../../src/bindings/TripCostSaved';
import type { TripDefaults } from '../../src/bindings/TripDefaults';
import type { TripDetail } from '../../src/bindings/TripDetail';
import type { TripEntryCancelInput } from '../../src/bindings/TripEntryCancelInput';
import type { TripEntryCancelled } from '../../src/bindings/TripEntryCancelled';
import type { TripInput } from '../../src/bindings/TripInput';
import type { TripRow } from '../../src/bindings/TripRow';
import type { TripsList } from '../../src/bindings/TripsList';
import type { TripsSummary } from '../../src/bindings/TripsSummary';
import type { UiError } from '../../src/bindings/UiError';
import costCancel from '../fixtures/trips_cost_cancel.json';
import costCancelRequest from '../fixtures/trips_cost_cancel_request.json';
import costCancelHasSales from '../fixtures/trips_cost_cancel_has_sales.json';
import costCancelHasSalesRequest from '../fixtures/trips_cost_cancel_has_sales_request.json';
import costPreviewTruck from '../fixtures/trips_cost_preview_truck.json';
import costPreviewTruckRequest from '../fixtures/trips_cost_preview_truck_request.json';
import costSaveTruck from '../fixtures/trips_cost_save_truck.json';
import costSaveTruckRequest from '../fixtures/trips_cost_save_truck_request.json';
import createPocket from '../fixtures/trips_create_pocket.json';
import createPocketRequest from '../fixtures/trips_create_pocket_request.json';
import createShares from '../fixtures/trips_create_shares_not_100.json';
import createSharesRequest from '../fixtures/trips_create_shares_not_100_request.json';
import createThree from '../fixtures/trips_create_three.json';
import createThreeRequest from '../fixtures/trips_create_three_request.json';
import defaultsFirst from '../fixtures/trips_defaults_first.json';
import defaultsFromLast from '../fixtures/trips_defaults_from_last_trip.json';
import getClosed from '../fixtures/trips_get_closed.json';
import getE41Closed from '../fixtures/trips_get_e2e41_closed.json';
import getOpen from '../fixtures/trips_get_open.json';
import getReopened from '../fixtures/trips_get_reopened.json';
import listAll from '../fixtures/trips_list_all.json';
import otherCancel from '../fixtures/trips_other_sale_cancel.json';
import otherCancelRequest from '../fixtures/trips_other_sale_cancel_request.json';
import otherPreview from '../fixtures/trips_other_sale_preview_3.json';
import otherPreviewRequest from '../fixtures/trips_other_sale_preview_3_request.json';
import otherSave from '../fixtures/trips_other_sale_save_3.json';
import otherSaveRequest from '../fixtures/trips_other_sale_save_3_request.json';
import summaryOpen from '../fixtures/trips_summary_open.json';
import purchaseCancel from '../fixtures/purchases_cancel.json';
import purchaseCancelRequest from '../fixtures/purchases_cancel_request.json';
import purchaseGetAfterWriteOff from '../fixtures/purchases_get_after_write_off.json';
import purchaseGet from '../fixtures/purchases_get_after_sales.json';
import purchasesListCancelled from '../fixtures/purchases_list_cancelled.json';
import purchasesListCancelledRequest from '../fixtures/purchases_list_cancelled_request.json';
import purchasePreviewCashBelowZero from '../fixtures/purchases_preview_cash_below_zero.json';
import purchasePreviewCashBelowZeroRequest from '../fixtures/purchases_preview_cash_below_zero_request.json';
import purchasePreviewNoQty from '../fixtures/purchases_preview_no_qty.json';
import purchasePreviewNoQtyRequest from '../fixtures/purchases_preview_no_qty_request.json';
import purchasePreviewPocket from '../fixtures/purchases_preview_e2e41_pocket_part.json';
import purchasePreviewPocketRequest from '../fixtures/purchases_preview_e2e41_pocket_part_request.json';
import purchaseSaveCashNotConfirmed from '../fixtures/purchases_save_cash_below_zero_not_confirmed.json';
import purchaseSaveCashNotConfirmedRequest from '../fixtures/purchases_save_cash_below_zero_not_confirmed_request.json';
import writeOffUndo from '../fixtures/purchases_write_off_undo.json';
import writeOffUndoRequest from '../fixtures/purchases_write_off_undo_request.json';
import purchasesList from '../fixtures/purchases_list_all.json';
import purchasePreview from '../fixtures/purchases_preview_e2e34.json';
import purchasePreviewRequest from '../fixtures/purchases_preview_e2e34_request.json';
import purchaseSave from '../fixtures/purchases_save_e2e34.json';
import purchaseSaveRequest from '../fixtures/purchases_save_e2e34_request.json';
import purchaseSaveE41 from '../fixtures/purchases_save_e2e41.json';
import purchaseSaveE41Request from '../fixtures/purchases_save_e2e41_request.json';
import purchaseSaveClosed from '../fixtures/purchases_save_trip_closed.json';
import writeOffLast2 from '../fixtures/purchases_write_off_last_2.json';
import writeOffLast2Request from '../fixtures/purchases_write_off_last_2_request.json';
import writeOffTooMany from '../fixtures/purchases_write_off_too_many.json';

// The fixture files are the backend's own JSON; each is read as its generated type. A reader
// takes `unknown`, because the money in a binding is a branded type that a plain number is not.

function readOtherSaleInput(value: unknown): OtherSaleInput {
  return value as OtherSaleInput;
}
function readOtherSalePreview(value: unknown): OtherSalePreview {
  return value as OtherSalePreview;
}
function readOtherSaleSaved(value: unknown): OtherSaleSaved {
  return value as OtherSaleSaved;
}
function readPurchaseInput(value: unknown): PurchaseInput {
  return value as PurchaseInput;
}
function readPurchaseCancelInput(value: unknown): PurchaseCancelInput {
  return value as PurchaseCancelInput;
}
function readPurchaseCancelled(value: unknown): PurchaseCancelled {
  return value as PurchaseCancelled;
}
function readPurchaseWriteOffUndone(value: unknown): PurchaseWriteOffUndone {
  return value as PurchaseWriteOffUndone;
}
function readPurchaseWriteOffUndoInput(value: unknown): PurchaseWriteOffUndoInput {
  return value as PurchaseWriteOffUndoInput;
}
function readPurchasesListInput(value: unknown): PurchasesListInput {
  return value as PurchasesListInput;
}
function readPurchasePreview(value: unknown): PurchasePreview {
  return value as PurchasePreview;
}
function readPurchaseRow(value: unknown): PurchaseRow {
  return value as PurchaseRow;
}
function readPurchaseSaved(value: unknown): PurchaseSaved {
  return value as PurchaseSaved;
}
function readPurchaseWriteOffDone(value: unknown): PurchaseWriteOffDone {
  return value as PurchaseWriteOffDone;
}
function readPurchaseWriteOffInput(value: unknown): PurchaseWriteOffInput {
  return value as PurchaseWriteOffInput;
}
function readPurchasesList(value: unknown): PurchasesList {
  return value as PurchasesList;
}
function readTripCostInput(value: unknown): TripCostInput {
  return value as TripCostInput;
}
function readTripCostPreview(value: unknown): TripCostPreview {
  return value as TripCostPreview;
}
function readTripCostSaved(value: unknown): TripCostSaved {
  return value as TripCostSaved;
}
function readTripDefaults(value: unknown): TripDefaults {
  return value as TripDefaults;
}
function readTripDetail(value: unknown): TripDetail {
  return value as TripDetail;
}
function readTripEntryCancelInput(value: unknown): TripEntryCancelInput {
  return value as TripEntryCancelInput;
}
function readTripEntryCancelled(value: unknown): TripEntryCancelled {
  return value as TripEntryCancelled;
}
function readTripInput(value: unknown): TripInput {
  return value as TripInput;
}
function readTripRow(value: unknown): TripRow {
  return value as TripRow;
}
function readTripsList(value: unknown): TripsList {
  return value as TripsList;
}
function readTripsSummary(value: unknown): TripsSummary {
  return value as TripsSummary;
}
function readUiError(value: unknown): UiError {
  return value as UiError;
}

// Trips
export const TRIP_DEFAULTS_FIRST = readTripDefaults(defaultsFirst);
export const TRIP_CREATE_THREE = readTripRow(createThree);
export const TRIP_CREATE_THREE_REQUEST = readTripInput(createThreeRequest);
export const TRIP_CREATE_POCKET = readTripRow(createPocket);
export const TRIP_CREATE_POCKET_REQUEST = readTripInput(createPocketRequest);
export const TRIP_CREATE_SHARES_NOT_100 = readUiError(createShares);
export const TRIP_CREATE_SHARES_NOT_100_REQUEST = readTripInput(createSharesRequest);
export const TRIP_LIST_ALL = readTripsList(listAll);
export const TRIP_GET_OPEN = readTripDetail(getOpen);
export const TRIP_GET_CLOSED = readTripDetail(getClosed);
export const TRIP_GET_REOPENED = readTripDetail(getReopened);
export const TRIP_GET_E41_CLOSED = readTripDetail(getE41Closed);
export const TRIPS_SUMMARY_OPEN = readTripsSummary(summaryOpen);

// Trip costs
export const COST_PREVIEW_TRUCK = readTripCostPreview(costPreviewTruck);
export const COST_PREVIEW_TRUCK_REQUEST = readTripCostInput(costPreviewTruckRequest);
export const COST_SAVE_TRUCK = readTripCostSaved(costSaveTruck);
export const COST_SAVE_TRUCK_REQUEST = readTripCostInput(costSaveTruckRequest);
export const COST_CANCEL_HAS_SALES = readUiError(costCancelHasSales);
export const COST_CANCEL_HAS_SALES_REQUEST = readTripEntryCancelInput(costCancelHasSalesRequest);

// Sales in another shop
export const OTHER_PREVIEW = readOtherSalePreview(otherPreview);
export const OTHER_PREVIEW_REQUEST = readOtherSaleInput(otherPreviewRequest);
export const OTHER_SAVE = readOtherSaleSaved(otherSave);
export const OTHER_SAVE_REQUEST = readOtherSaleInput(otherSaveRequest);
export const OTHER_CANCEL = readTripEntryCancelled(otherCancel);
export const OTHER_CANCEL_REQUEST = readTripEntryCancelInput(otherCancelRequest);

// Purchases
export const PURCHASE_PREVIEW_E34 = readPurchasePreview(purchasePreview);
export const PURCHASE_PREVIEW_E34_REQUEST = readPurchaseInput(purchasePreviewRequest);
export const PURCHASE_SAVE_E34 = readPurchaseSaved(purchaseSave);
export const PURCHASE_SAVE_E34_REQUEST = readPurchaseInput(purchaseSaveRequest);
export const PURCHASE_SAVE_E41 = readPurchaseSaved(purchaseSaveE41);
export const PURCHASE_SAVE_E41_REQUEST = readPurchaseInput(purchaseSaveE41Request);
export const PURCHASE_SAVE_TRIP_CLOSED = readUiError(purchaseSaveClosed);
export const PURCHASE_GET_AFTER_SALES = readPurchaseRow(purchaseGet);
export const PURCHASES_LIST_ALL = readPurchasesList(purchasesList);
export const WRITE_OFF_LAST_2 = readPurchaseWriteOffDone(writeOffLast2);
export const WRITE_OFF_LAST_2_REQUEST = readPurchaseWriteOffInput(writeOffLast2Request);
export const WRITE_OFF_TOO_MANY = readUiError(writeOffTooMany);

// Added with the backend's later files: the cancel of a purchase and of a trip cost, the undo of a
// write-off, the defaults taken from the last trip, a purchase that takes cash below zero, one with
// a part paid from a pocket, one with no quantity, and the list of cancelled purchases
export const PURCHASE_CANCEL = readPurchaseCancelled(purchaseCancel);
export const PURCHASE_CANCEL_REQUEST = readPurchaseCancelInput(purchaseCancelRequest);
export const COST_CANCEL = readTripEntryCancelled(costCancel);
export const COST_CANCEL_REQUEST = readTripEntryCancelInput(costCancelRequest);
export const WRITE_OFF_UNDO = readPurchaseWriteOffUndone(writeOffUndo);
export const WRITE_OFF_UNDO_REQUEST = readPurchaseWriteOffUndoInput(writeOffUndoRequest);
export const TRIP_DEFAULTS_FROM_LAST = readTripDefaults(defaultsFromLast);
export const PURCHASE_GET_AFTER_WRITE_OFF = readPurchaseRow(purchaseGetAfterWriteOff);
export const PURCHASES_LIST_CANCELLED = readPurchasesList(purchasesListCancelled);
export const PURCHASES_LIST_CANCELLED_REQUEST = readPurchasesListInput(
  purchasesListCancelledRequest,
);
export const PURCHASE_PREVIEW_CASH_BELOW_ZERO = readPurchasePreview(purchasePreviewCashBelowZero);
export const PURCHASE_PREVIEW_CASH_BELOW_ZERO_REQUEST = readPurchaseInput(
  purchasePreviewCashBelowZeroRequest,
);
export const PURCHASE_PREVIEW_POCKET = readPurchasePreview(purchasePreviewPocket);
export const PURCHASE_PREVIEW_POCKET_REQUEST = readPurchaseInput(purchasePreviewPocketRequest);
export const PURCHASE_PREVIEW_NO_QTY = readPurchasePreview(purchasePreviewNoQty);
export const PURCHASE_PREVIEW_NO_QTY_REQUEST = readPurchaseInput(purchasePreviewNoQtyRequest);
export const PURCHASE_SAVE_CASH_NOT_CONFIRMED = readUiError(purchaseSaveCashNotConfirmed);
export const PURCHASE_SAVE_CASH_NOT_CONFIRMED_REQUEST = readPurchaseInput(
  purchaseSaveCashNotConfirmedRequest,
);
