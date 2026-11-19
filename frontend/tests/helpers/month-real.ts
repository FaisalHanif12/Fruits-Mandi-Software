// The real answers and requests of the backend for the month close (round 7, PRD FR-17.6, FR-21.2).
// The backend wrote them in its own test (backend/crates/services/tests/fixtures/month_*.json); they
// are copied here by `scripts/copy-backend-fixtures.mjs`, never edited, and read as the
// generated types. Money in them is paisa, as on the wire. A reader takes `unknown`, because the
// money in a binding is a branded type that a plain number is not. The month is the worked day's
// October 2026, closed on 02-11-2026 (the "today" of those tests).

import type { MonthClosed } from '../../src/bindings/MonthClosed';
import type { MonthInput } from '../../src/bindings/MonthInput';
import type { MonthList } from '../../src/bindings/MonthList';
import type { MonthPreview } from '../../src/bindings/MonthPreview';
import type { MonthReverseInput } from '../../src/bindings/MonthReverseInput';
import type { MonthReversed } from '../../src/bindings/MonthReversed';
import type { UiError } from '../../src/bindings/UiError';
import closePreview from '../fixtures/month_close_preview.json';
import closePreviewBadMonth from '../fixtures/month_close_preview_bad_month.json';
import closePreviewBadMonthRequest from '../fixtures/month_close_preview_bad_month_request.json';
import closePreviewNotFinished from '../fixtures/month_close_preview_not_finished.json';
import closePreviewNotFinishedRequest from '../fixtures/month_close_preview_not_finished_request.json';
import closePreviewRequest from '../fixtures/month_close_preview_request.json';
import closeReverse from '../fixtures/month_close_reverse.json';
import closeReverseNoReason from '../fixtures/month_close_reverse_no_reason.json';
import closeReverseNoReasonRequest from '../fixtures/month_close_reverse_no_reason_request.json';
import closeReverseNotInForce from '../fixtures/month_close_reverse_not_in_force.json';
import closeReverseNotInForceRequest from '../fixtures/month_close_reverse_not_in_force_request.json';
import closeReverseRequest from '../fixtures/month_close_reverse_request.json';
import closeSave from '../fixtures/month_close_save.json';
import closeSaveDaysOpen from '../fixtures/month_close_save_days_open.json';
import closeSaveDaysOpenRequest from '../fixtures/month_close_save_days_open_request.json';
import closeSaveNothingToClose from '../fixtures/month_close_save_nothing_to_close.json';
import closeSaveNothingToCloseRequest from '../fixtures/month_close_save_nothing_to_close_request.json';
import closeSaveRequest from '../fixtures/month_close_save_request.json';
import closeSaveTwice from '../fixtures/month_close_save_twice.json';
import closeSaveTwiceRequest from '../fixtures/month_close_save_twice_request.json';
import listClosed from '../fixtures/month_list_closed.json';
import listReady from '../fixtures/month_list_ready.json';

function readList(value: unknown): MonthList {
  return value as MonthList;
}
function readPreview(value: unknown): MonthPreview {
  return value as MonthPreview;
}
function readInput(value: unknown): MonthInput {
  return value as MonthInput;
}
function readClosed(value: unknown): MonthClosed {
  return value as MonthClosed;
}
function readReverseInput(value: unknown): MonthReverseInput {
  return value as MonthReverseInput;
}
function readReversed(value: unknown): MonthReversed {
  return value as MonthReversed;
}
function readRefusal(value: unknown): UiError {
  return value as UiError;
}

/** Four months: 2026-11 not finished, 2026-10 to 2026-08 ready. `month_list` takes no input. */
export const MONTH_LIST_READY = readList(listReady);
/** The same list after October was closed on 02-11-2026. */
export const MONTH_LIST_CLOSED = readList(listClosed);

export const MONTH_PREVIEW = readPreview(closePreview);
export const MONTH_PREVIEW_REQUEST = readInput(closePreviewRequest);
/** October asked while it is not finished: `ERR_DATE_IN_FUTURE` on `month`. */
export const MONTH_PREVIEW_NOT_FINISHED = readRefusal(closePreviewNotFinished);
export const MONTH_PREVIEW_NOT_FINISHED_REQUEST = readInput(closePreviewNotFinishedRequest);
/** A month in the wrong form (`2026-13`): `ERR_FORMAT_INVALID` on `month`. */
export const MONTH_PREVIEW_BAD_MONTH = readRefusal(closePreviewBadMonth);
export const MONTH_PREVIEW_BAD_MONTH_REQUEST = readInput(closePreviewBadMonthRequest);

export const MONTH_SAVE = readClosed(closeSave);
export const MONTH_SAVE_REQUEST = readInput(closeSaveRequest);
/** Days closed only up to 09-10-2026: `ERR_DAY_NOT_CLOSED` with `closed_up_to` on `month`. */
export const MONTH_SAVE_DAYS_OPEN = readRefusal(closeSaveDaysOpen);
export const MONTH_SAVE_DAYS_OPEN_REQUEST = readInput(closeSaveDaysOpenRequest);
/** September has no income and no expense: `ERR_NOTHING_TO_CLOSE` on `month`. */
export const MONTH_SAVE_NOTHING_TO_CLOSE = readRefusal(closeSaveNothingToClose);
export const MONTH_SAVE_NOTHING_TO_CLOSE_REQUEST = readInput(closeSaveNothingToCloseRequest);
/** October closed a second time: `ERR_MONTH_CLOSED` on `month`. */
export const MONTH_SAVE_TWICE = readRefusal(closeSaveTwice);
export const MONTH_SAVE_TWICE_REQUEST = readInput(closeSaveTwiceRequest);

export const MONTH_REVERSE = readReversed(closeReverse);
export const MONTH_REVERSE_REQUEST = readReverseInput(closeReverseRequest);
/** A blank reason: `ERR_REASON_REQUIRED` on `reason`. */
export const MONTH_REVERSE_NO_REASON = readRefusal(closeReverseNoReason);
export const MONTH_REVERSE_NO_REASON_REQUEST = readReverseInput(closeReverseNoReasonRequest);
/** A month whose close is not in force: `ERR_NOT_FOUND` on `month`. */
export const MONTH_REVERSE_NOT_IN_FORCE = readRefusal(closeReverseNotInForce);
export const MONTH_REVERSE_NOT_IN_FORCE_REQUEST = readReverseInput(closeReverseNotInForceRequest);
