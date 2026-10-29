// The real answers and requests of the backend for plan 33 version 1.1 (T2): the date to sell of a
// purchase, the goods arrived of a garden, the list of ready goods for a new sale and a garden lot
// with its arrival. The backend wrote them in its own test (backend/crates/services/tests/
// arrivals_fixtures.rs); they are copied here, never edited, and read as the generated types.

import type { BikriDetail } from '../../src/bindings/BikriDetail';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { StockList } from '../../src/bindings/StockList';
import type { ArrivalCancelInput } from '../../src/bindings/ArrivalCancelInput';
import type { ArrivalCancelled } from '../../src/bindings/ArrivalCancelled';
import type { ArrivalInput } from '../../src/bindings/ArrivalInput';
import type { ArrivalRow } from '../../src/bindings/ArrivalRow';
import type { ArrivalSaved } from '../../src/bindings/ArrivalSaved';
import type { ArrivalsList } from '../../src/bindings/ArrivalsList';
import type { ArrivalsListInput } from '../../src/bindings/ArrivalsListInput';
import type { ArrivalWriteOffDone } from '../../src/bindings/ArrivalWriteOffDone';
import type { ArrivalWriteOffInput } from '../../src/bindings/ArrivalWriteOffInput';
import type { ArrivalWriteOffUndoInput } from '../../src/bindings/ArrivalWriteOffUndoInput';
import type { ArrivalWriteOffUndone } from '../../src/bindings/ArrivalWriteOffUndone';
import type { PurchaseInput } from '../../src/bindings/PurchaseInput';
import type { PurchaseSaved } from '../../src/bindings/PurchaseSaved';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReadyGoods } from '../../src/bindings/ReadyGoods';
import type { SellDateInput } from '../../src/bindings/SellDateInput';
import type { SellDateSet } from '../../src/bindings/SellDateSet';
import type { UiError } from '../../src/bindings/UiError';
import arrivalCancel from '../fixtures/gardens_arrival_cancel.json';
import arrivalCancelRequest from '../fixtures/gardens_arrival_cancel_request.json';
import arrivalCancelHasSales from '../fixtures/gardens_arrival_cancel_has_sales.json';
import arrivalGetAfterSale from '../fixtures/gardens_arrival_get_after_a_sale.json';
import arrivalSave from '../fixtures/gardens_arrival_save.json';
import arrivalSaveRequest from '../fixtures/gardens_arrival_save_request.json';
import arrivalSaveSellDateBefore from '../fixtures/gardens_arrival_save_sell_date_before_entry.json';
import arrivalSetSellDate from '../fixtures/gardens_arrival_set_sell_date.json';
import arrivalSetSellDateRequest from '../fixtures/gardens_arrival_set_sell_date_request.json';
import arrivalWriteOff from '../fixtures/gardens_arrival_write_off.json';
import arrivalWriteOffExceeds from '../fixtures/gardens_arrival_write_off_exceeds.json';
import arrivalWriteOffRequest from '../fixtures/gardens_arrival_write_off_request.json';
import arrivalWriteOffUndo from '../fixtures/gardens_arrival_write_off_undo.json';
import arrivalWriteOffUndoRequest from '../fixtures/gardens_arrival_write_off_undo_request.json';
import arrivalsListAll from '../fixtures/gardens_arrivals_list_all.json';
import arrivalsListAllRequest from '../fixtures/gardens_arrivals_list_all_request.json';
import arrivalsListInStock from '../fixtures/gardens_arrivals_list_in_stock.json';
import arrivalsListInStockRequest from '../fixtures/gardens_arrivals_list_in_stock_request.json';
import gardenLotGet from '../fixtures/bikri_get_garden_lot_with_arrival.json';
import gardenLotPreview from '../fixtures/bikri_preview_garden_lot_with_arrival.json';
import gardenLotPreviewNotFullySold from '../fixtures/bikri_preview_garden_lot_with_arrival_not_fully_sold.json';
import gardenLotPreviewTooMany from '../fixtures/bikri_preview_garden_lot_with_arrival_too_many.json';
import gardenLotSaveOversold from '../fixtures/bikri_save_garden_lot_with_arrival_oversold.json';
import stockWithGardenRows from '../fixtures/bikri_stock_list_with_garden_rows.json';
import r10GardenArrival from '../fixtures/reports_run_r10_with_garden_arrival.json';
import r10GardenArrivalPrint from '../fixtures/reports_print_data_r10_with_garden_arrival_en.json';
import closeHasStock from '../fixtures/gardens_close_has_stock.json';
import purchaseSaveWithSellDate from '../fixtures/purchases_save_with_sell_date.json';
import purchaseSaveWithSellDateRequest from '../fixtures/purchases_save_with_sell_date_request.json';
import purchaseSetSellDate from '../fixtures/purchases_set_sell_date.json';
import purchaseSetSellDateRequest from '../fixtures/purchases_set_sell_date_request.json';
import readyGoods from '../fixtures/bikri_ready_goods.json';
import readyGoodsAfterPartSale from '../fixtures/bikri_ready_goods_after_a_part_sale.json';
import readyGoodsNone from '../fixtures/bikri_ready_goods_none_yet.json';

// The files are the wire shapes the backend sent. These readers are the one place that says so.
function readArrivalCancelInput(value: unknown): ArrivalCancelInput {
  return value as ArrivalCancelInput;
}
function readArrivalCancelled(value: unknown): ArrivalCancelled {
  return value as ArrivalCancelled;
}
function readArrivalInput(value: unknown): ArrivalInput {
  return value as ArrivalInput;
}
function readArrivalRow(value: unknown): ArrivalRow {
  return value as ArrivalRow;
}
function readArrivalSaved(value: unknown): ArrivalSaved {
  return value as ArrivalSaved;
}
function readArrivalWriteOffDone(value: unknown): ArrivalWriteOffDone {
  return value as ArrivalWriteOffDone;
}
function readArrivalWriteOffInput(value: unknown): ArrivalWriteOffInput {
  return value as ArrivalWriteOffInput;
}
function readArrivalWriteOffUndoInput(value: unknown): ArrivalWriteOffUndoInput {
  return value as ArrivalWriteOffUndoInput;
}
function readArrivalWriteOffUndone(value: unknown): ArrivalWriteOffUndone {
  return value as ArrivalWriteOffUndone;
}
function readArrivalsList(value: unknown): ArrivalsList {
  return value as ArrivalsList;
}
function readArrivalsListInput(value: unknown): ArrivalsListInput {
  return value as ArrivalsListInput;
}
function readBikriDetail(value: unknown): BikriDetail {
  return value as BikriDetail;
}
function readBikriPreview(value: unknown): BikriPreview {
  return value as BikriPreview;
}
function readPurchaseInput(value: unknown): PurchaseInput {
  return value as PurchaseInput;
}
function readPurchaseSaved(value: unknown): PurchaseSaved {
  return value as PurchaseSaved;
}
function readReportPage(value: unknown): ReportPage {
  return value as ReportPage;
}
function readReportPrintData(value: unknown): ReportPrintData {
  return value as ReportPrintData;
}
function readReadyGoods(value: unknown): ReadyGoods {
  return value as ReadyGoods;
}
function readSellDateInput(value: unknown): SellDateInput {
  return value as SellDateInput;
}
function readSellDateSet(value: unknown): SellDateSet {
  return value as SellDateSet;
}
function readStockList(value: unknown): StockList {
  return value as StockList;
}
function readUiError(value: unknown): UiError {
  return value as UiError;
}

export const READY_GOODS = readReadyGoods(readyGoods);
export const READY_GOODS_AFTER_A_PART_SALE = readReadyGoods(readyGoodsAfterPartSale);
export const READY_GOODS_NONE = readReadyGoods(readyGoodsNone);

export const ARRIVAL_SAVE = readArrivalSaved(arrivalSave);
export const ARRIVAL_SAVE_REQUEST = readArrivalInput(arrivalSaveRequest);
export const ARRIVAL_SAVE_SELL_DATE_BEFORE = readUiError(arrivalSaveSellDateBefore);
export const ARRIVALS_IN_STOCK = readArrivalsList(arrivalsListInStock);
export const ARRIVALS_IN_STOCK_REQUEST = readArrivalsListInput(arrivalsListInStockRequest);
export const ARRIVALS_ALL = readArrivalsList(arrivalsListAll);
export const ARRIVALS_ALL_REQUEST = readArrivalsListInput(arrivalsListAllRequest);
export const ARRIVAL_AFTER_A_SALE = readArrivalRow(arrivalGetAfterSale);
export const ARRIVAL_CANCEL = readArrivalCancelled(arrivalCancel);
export const ARRIVAL_CANCEL_REQUEST = readArrivalCancelInput(arrivalCancelRequest);
export const ARRIVAL_CANCEL_HAS_SALES = readUiError(arrivalCancelHasSales);
export const ARRIVAL_SET_SELL_DATE = readSellDateSet(arrivalSetSellDate);
export const ARRIVAL_SET_SELL_DATE_REQUEST = readSellDateInput(arrivalSetSellDateRequest);
export const ARRIVAL_WRITE_OFF = readArrivalWriteOffDone(arrivalWriteOff);
export const ARRIVAL_WRITE_OFF_REQUEST = readArrivalWriteOffInput(arrivalWriteOffRequest);
export const ARRIVAL_WRITE_OFF_EXCEEDS = readUiError(arrivalWriteOffExceeds);
export const ARRIVAL_WRITE_OFF_UNDO = readArrivalWriteOffUndone(arrivalWriteOffUndo);
export const ARRIVAL_WRITE_OFF_UNDO_REQUEST = readArrivalWriteOffUndoInput(
  arrivalWriteOffUndoRequest,
);
export const PURCHASE_SET_SELL_DATE = readSellDateSet(purchaseSetSellDate);
export const PURCHASE_SET_SELL_DATE_REQUEST = readSellDateInput(purchaseSetSellDateRequest);
export const GARDEN_CLOSE_HAS_STOCK = readUiError(closeHasStock);

export const GARDEN_LOT_PREVIEW = readBikriPreview(gardenLotPreview);
export const GARDEN_LOT_PREVIEW_NOT_FULLY_SOLD = readBikriPreview(gardenLotPreviewNotFullySold);
export const GARDEN_LOT_PREVIEW_TOO_MANY = readBikriPreview(gardenLotPreviewTooMany);
export const GARDEN_LOT_SAVE_OVERSOLD = readUiError(gardenLotSaveOversold);
export const GARDEN_LOT_SAVED = readBikriDetail(gardenLotGet);
export const STOCK_WITH_GARDEN_ROWS = readStockList(stockWithGardenRows);

export const PURCHASE_SAVE_WITH_SELL_DATE_REQUEST = readPurchaseInput(
  purchaseSaveWithSellDateRequest,
);
export const PURCHASE_SAVE_WITH_SELL_DATE = readPurchaseSaved(purchaseSaveWithSellDate);

/** R10, the goods left, with a garden's goods arrived as a row (backend: reports_run_r10_with_garden_arrival). */
export const R10_GARDEN_ARRIVAL = readReportPage(r10GardenArrival);
export const R10_GARDEN_ARRIVAL_PRINT = readReportPrintData(r10GardenArrivalPrint);
