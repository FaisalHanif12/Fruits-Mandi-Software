// The real answers and requests of the backend for a lot of trip goods in the sale book (plan 16b,
// T4b). The backend wrote them in its own golden tests (backend/crates/services/tests/fixtures/
// bikri_*_own_lot_*.json and bikri_stock_list_own_rows.json). They are copied here, never edited,
// and read as the generated types. Money in them is paisa, as on the wire. The story is the PRD
// worked numbers of E2E-34 (15 oranges to Akram at 600) and E2E-41 (80 oranges to Arshad at 1,400).

import type { BikriCancelled } from '../../src/bindings/BikriCancelled';
import type { BikriDetail } from '../../src/bindings/BikriDetail';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { BikriPreviewInput } from '../../src/bindings/BikriPreviewInput';
import type { BikriSaveInput } from '../../src/bindings/BikriSaveInput';
import type { BikriSaved } from '../../src/bindings/BikriSaved';
import type { CancelInput } from '../../src/bindings/CancelInput';
import type { EditableBikri } from '../../src/bindings/EditableBikri';
import type { StockList } from '../../src/bindings/StockList';
import cancelOwn from '../fixtures/bikri_cancel_own_lot.json';
import cancelOwnRequest from '../fixtures/bikri_cancel_own_lot_request.json';
import getE34 from '../fixtures/bikri_get_own_lot_e2e34.json';
import getE41 from '../fixtures/bikri_get_own_lot_e2e41.json';
import openForEditOwn from '../fixtures/bikri_open_for_edit_own_lot.json';
import previewE34 from '../fixtures/bikri_preview_own_lot_e2e34.json';
import previewE34Request from '../fixtures/bikri_preview_own_lot_e2e34_request.json';
import previewNotFullySold from '../fixtures/bikri_preview_own_lot_not_fully_sold.json';
import previewNotFullySoldRequest from '../fixtures/bikri_preview_own_lot_not_fully_sold_request.json';
import saveE34 from '../fixtures/bikri_save_own_lot_e2e34.json';
import saveE34Request from '../fixtures/bikri_save_own_lot_e2e34_request.json';
import saveE41 from '../fixtures/bikri_save_own_lot_e2e41.json';
import saveE41Request from '../fixtures/bikri_save_own_lot_e2e41_request.json';
import stockOwnRows from '../fixtures/bikri_stock_list_own_rows.json';

// The fixture files are the backend's own JSON; each is read as its generated type. A reader takes
// `unknown`, because the money in a binding is a branded type that a plain number is not.

function readBikriPreview(value: unknown): BikriPreview {
  return value as BikriPreview;
}
function readBikriPreviewInput(value: unknown): BikriPreviewInput {
  return value as BikriPreviewInput;
}
function readBikriSaved(value: unknown): BikriSaved {
  return value as BikriSaved;
}
function readBikriSaveInput(value: unknown): BikriSaveInput {
  return value as BikriSaveInput;
}
function readBikriDetail(value: unknown): BikriDetail {
  return value as BikriDetail;
}
function readEditableBikri(value: unknown): EditableBikri {
  return value as EditableBikri;
}
function readBikriCancelled(value: unknown): BikriCancelled {
  return value as BikriCancelled;
}
function readCancelInput(value: unknown): CancelInput {
  return value as CancelInput;
}
function readStockList(value: unknown): StockList {
  return value as StockList;
}

// A new own lot: preview, save, and the refusal for a lot that is not sold in full
export const OWN_PREVIEW_E34 = readBikriPreview(previewE34);
export const OWN_PREVIEW_E34_REQUEST = readBikriPreviewInput(previewE34Request);
export const OWN_PREVIEW_NOT_FULLY_SOLD = readBikriPreview(previewNotFullySold);
export const OWN_PREVIEW_NOT_FULLY_SOLD_REQUEST = readBikriPreviewInput(previewNotFullySoldRequest);
export const OWN_SAVE_E34 = readBikriSaved(saveE34);
export const OWN_SAVE_E34_REQUEST = readBikriSaveInput(saveE34Request);
export const OWN_SAVE_E41 = readBikriSaved(saveE41);
export const OWN_SAVE_E41_REQUEST = readBikriSaveInput(saveE41Request);

// The saved view, the edit and the cancel
export const OWN_GET_E34 = readBikriDetail(getE34);
export const OWN_GET_E41 = readBikriDetail(getE41);
export const OWN_OPEN_FOR_EDIT = readEditableBikri(openForEditOwn);
export const OWN_CANCEL = readBikriCancelled(cancelOwn);
export const OWN_CANCEL_REQUEST = readCancelInput(cancelOwnRequest);

// The goods left list with a trip purchase row
export const STOCK_OWN_ROWS = readStockList(stockOwnRows);
