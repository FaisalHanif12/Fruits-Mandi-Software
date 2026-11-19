// The real answers and requests of the backend for plan 16 (T4). The backend wrote them in its own
// golden tests (backend/crates/services/tests/fixtures/*.json). They are copied here, never
// edited, and read as the generated types. Money in them is paisa, as on the wire. The contract
// tests that use them show the screens the numbers of PRD E2E-13 and E2E-16b from the backend's
// own answers, and check that what the screens send is what the backend's tests send.

import cancelPreview from '../fixtures/bikri_cancel_preview_event2.json';
import cancelDone from '../fixtures/bikri_cancel_event2.json';
import cancelRequest from '../fixtures/bikri_cancel_event2_request.json';
import editPreview from '../fixtures/bikri_edit_preview_akram_610.json';
import editPreviewRequest from '../fixtures/bikri_edit_preview_akram_610_request.json';
import editPreviewDayChanged from '../fixtures/bikri_edit_preview_day_changed.json';
import editPreviewSellerLocked from '../fixtures/bikri_edit_preview_lot_seller_locked.json';
import editSave from '../fixtures/bikri_edit_save_akram_610.json';
import editSaveRequest from '../fixtures/bikri_edit_save_akram_610_request.json';
import laterPreview from '../fixtures/bikri_later_sale_preview_e2e16b.json';
import laterPreviewRequest from '../fixtures/bikri_later_sale_preview_e2e16b_request.json';
import laterSave from '../fixtures/bikri_later_sale_save_e2e16b.json';
import laterSaveRequest from '../fixtures/bikri_later_sale_save_e2e16b_request.json';
import openForEdit from '../fixtures/bikri_open_for_edit_event1.json';
import stockList from '../fixtures/bikri_stock_list.json';
import undo from '../fixtures/bikri_undo_event1.json';
import undoRequest from '../fixtures/bikri_undo_event1_request.json';
import writeOff from '../fixtures/bikri_write_off_lot1.json';
import writeOffRequest from '../fixtures/bikri_write_off_lot1_request.json';
import writeOffUndo from '../fixtures/bikri_write_off_undo_lot1.json';
import writeOffUndoRequest from '../fixtures/bikri_write_off_undo_lot1_request.json';
import type { BikriCancelled } from '../../src/bindings/BikriCancelled';
import type { BikriEdited } from '../../src/bindings/BikriEdited';
import type { CancelInput } from '../../src/bindings/CancelInput';
import type { CancelPreview } from '../../src/bindings/CancelPreview';
import type { EditableBikri } from '../../src/bindings/EditableBikri';
import type { EditPreviewInput } from '../../src/bindings/EditPreviewInput';
import type { EditSaveInput } from '../../src/bindings/EditSaveInput';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { LaterSaleInput } from '../../src/bindings/LaterSaleInput';
import type { LaterSaleSaved } from '../../src/bindings/LaterSaleSaved';
import type { StockList } from '../../src/bindings/StockList';
import type { UndoInput } from '../../src/bindings/UndoInput';
import type { UndoResult } from '../../src/bindings/UndoResult';
import type { WriteOffDone } from '../../src/bindings/WriteOffDone';
import type { WriteOffInput } from '../../src/bindings/WriteOffInput';
import type { WriteOffUndoInput } from '../../src/bindings/WriteOffUndoInput';
import type { WriteOffUndone } from '../../src/bindings/WriteOffUndone';

/** The fixture files are the backend's own JSON; their text is read as the generated types. */
function stockListOf(value: unknown): StockList {
  return value as StockList;
}

function writeOffOf(value: unknown): WriteOffDone {
  return value as WriteOffDone;
}

function writeOffRequestOf(value: unknown): WriteOffInput {
  return value as WriteOffInput;
}

function writeOffUndoOf(value: unknown): WriteOffUndone {
  return value as WriteOffUndone;
}

function writeOffUndoRequestOf(value: unknown): WriteOffUndoInput {
  return value as WriteOffUndoInput;
}

function laterPreviewOf(value: unknown): BikriPreview {
  return value as BikriPreview;
}

function laterPreviewRequestOf(value: unknown): LaterSaleInput {
  return value as LaterSaleInput;
}

function laterSaveOf(value: unknown): LaterSaleSaved {
  return value as LaterSaleSaved;
}

function laterSaveRequestOf(value: unknown): LaterSaleInput & { confirm_cash_below_zero: boolean } {
  return value as LaterSaleInput & { confirm_cash_below_zero: boolean };
}

function openForEditOf(value: unknown): EditableBikri {
  return value as EditableBikri;
}

function editPreviewOf(value: unknown): BikriPreview {
  return value as BikriPreview;
}

function editPreviewRequestOf(value: unknown): EditPreviewInput {
  return value as EditPreviewInput;
}

function editSaveOf(value: unknown): BikriEdited {
  return value as BikriEdited;
}

function editSaveRequestOf(value: unknown): EditSaveInput {
  return value as EditSaveInput;
}

function cancelPreviewOf(value: unknown): CancelPreview {
  return value as CancelPreview;
}

function cancelDoneOf(value: unknown): BikriCancelled {
  return value as BikriCancelled;
}

function cancelRequestOf(value: unknown): CancelInput {
  return value as CancelInput;
}

function undoOf(value: unknown): UndoResult {
  return value as UndoResult;
}

function undoRequestOf(value: unknown): UndoInput {
  return value as UndoInput;
}

export const STOCK_LIST = stockListOf(stockList);
export const WRITE_OFF = writeOffOf(writeOff);
export const WRITE_OFF_REQUEST = writeOffRequestOf(writeOffRequest);
export const WRITE_OFF_UNDO = writeOffUndoOf(writeOffUndo);
export const WRITE_OFF_UNDO_REQUEST = writeOffUndoRequestOf(writeOffUndoRequest);
export const LATER_PREVIEW = laterPreviewOf(laterPreview);
export const LATER_PREVIEW_REQUEST = laterPreviewRequestOf(laterPreviewRequest);
export const LATER_SAVE = laterSaveOf(laterSave);
export const LATER_SAVE_REQUEST = laterSaveRequestOf(laterSaveRequest);
export const OPEN_FOR_EDIT = openForEditOf(openForEdit);
export const EDIT_PREVIEW = editPreviewOf(editPreview);
export const EDIT_PREVIEW_REQUEST = editPreviewRequestOf(editPreviewRequest);
/** The answers that carry an edit rule (ERR_EDIT_NOT_ALLOWED), as the backend's own tests wrote them. */
export const EDIT_PREVIEW_DAY_CHANGED = editPreviewOf(editPreviewDayChanged);
export const EDIT_PREVIEW_SELLER_LOCKED = editPreviewOf(editPreviewSellerLocked);
export const EDIT_SAVE = editSaveOf(editSave);
export const EDIT_SAVE_REQUEST = editSaveRequestOf(editSaveRequest);
export const CANCEL_PREVIEW = cancelPreviewOf(cancelPreview);
export const CANCEL_DONE = cancelDoneOf(cancelDone);
export const CANCEL_REQUEST = cancelRequestOf(cancelRequest);
export const UNDO = undoOf(undo);
export const UNDO_REQUEST = undoRequestOf(undoRequest);
