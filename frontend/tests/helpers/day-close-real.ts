// The real answer and request of the backend for the day close preview with its full summary
// (plan 17, F5, PRD FR-21.1, 4.9). The backend wrote them in its own test
// (backend/crates/services/tests/day_money_fixtures.rs); they are copied here, never edited, and
// read as the generated types. Money in them is paisa, as on the wire. The story: بکری 1, the
// electricity bill and الف's 2,000 on 10-10-2026, so 1 بکری, خام 29,400, commission 3,928, shared
// income 3,948, and shares 1,580 / 1,184 / 1,184 against expenses 1,200 / 900 / 900.

import dayClosePreviewSummary from '../fixtures/day_close_preview_summary.json';
import dayClosePreviewSummaryClosedDay from '../fixtures/day_close_preview_summary_closed_day.json';
import dayClosePreviewSummaryClosedDayRequest from '../fixtures/day_close_preview_summary_closed_day_request.json';
import dayClosePreviewSummaryRequest from '../fixtures/day_close_preview_summary_request.json';
import dayClosePreviewSummaryWorkedDay from '../fixtures/day_close_preview_summary_worked_day.json';
import dayClosePreviewSummaryWorkedDayRequest from '../fixtures/day_close_preview_summary_worked_day_request.json';
import dayClosePreviewSummaryStockLeft from '../fixtures/day_close_preview_summary_stock_left.json';
import dayClosePreviewSummaryStockLeftRequest from '../fixtures/day_close_preview_summary_stock_left_request.json';
import type { DayClosePreview } from '../../src/bindings/DayClosePreview';
import type { DayClosePreviewInput } from '../../src/bindings/DayClosePreviewInput';
import type { UiError } from '../../src/bindings/UiError';

// A fixture file is the backend's own JSON; each is read as its generated type.
function readDayClosePreview(value: unknown): DayClosePreview {
  return value as DayClosePreview;
}
function readDayClosePreviewInput(value: unknown): DayClosePreviewInput {
  return value as DayClosePreviewInput;
}

function readUiError(value: unknown): UiError {
  return value as UiError;
}

export const DAY_CLOSE_PREVIEW: DayClosePreview = readDayClosePreview(dayClosePreviewSummary);

/** Exactly the `input` the command took: the empty date asks for the oldest open day. */
export const REQUEST_DAY_CLOSE_PREVIEW: DayClosePreviewInput = readDayClosePreviewInput(
  dayClosePreviewSummaryRequest,
);

/** The E2E-16 state: 5 نگ of کیلا are still in باقی مال (1 lot, 5 نگ), خام 26,500. */
export const DAY_CLOSE_PREVIEW_STOCK_LEFT: DayClosePreview = readDayClosePreview(
  dayClosePreviewSummaryStockLeft,
);
export const REQUEST_DAY_CLOSE_PREVIEW_STOCK_LEFT: DayClosePreviewInput = readDayClosePreviewInput(
  dayClosePreviewSummaryStockLeftRequest,
);

/** The refusal for a day that is closed already: ERR_DATE_CLOSED on the field `up_to`. */
export const REFUSED_DAY_CLOSED: UiError = readUiError(dayClosePreviewSummaryClosedDay);
export const REQUEST_DAY_CLOSE_PREVIEW_CLOSED_DAY: DayClosePreviewInput = readDayClosePreviewInput(
  dayClosePreviewSummaryClosedDayRequest,
);

/**
 * The WHOLE worked day of PRD 4.9 after Events 1 to 5 (written by golden_day): 2 بکری, خام 41,400,
 * commission 5,448, shared income 5,488, shares 996 / 746 / 746, cash 58,484, trial 85,048.
 */
export const DAY_CLOSE_PREVIEW_WORKED_DAY: DayClosePreview = readDayClosePreview(
  dayClosePreviewSummaryWorkedDay,
);
export const REQUEST_DAY_CLOSE_PREVIEW_WORKED_DAY: DayClosePreviewInput = readDayClosePreviewInput(
  dayClosePreviewSummaryWorkedDayRequest,
);
