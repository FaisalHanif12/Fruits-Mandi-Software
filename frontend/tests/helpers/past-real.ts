// The real answers of the backend for پچھلی بکری (plan 72 T1). The backend wrote them in its own tests
// (backend/crates/services/tests/fixtures/bikri_past_*.json). They are copied here with
// `node scripts/copy-backend-fixtures.mjs bikri_past_`, never edited, and read as the types of the list.
// Money in them is paisa, as on the wire. Today in these files is 12-11-2026, so the first window is
// 05-11-2026 to 11-11-2026.

import today from '../fixtures/bikri_past_today.json';
import todayRequest from '../fixtures/bikri_past_today_request.json';
import window from '../fixtures/bikri_past_window.json';
import windowRequest from '../fixtures/bikri_past_window_request.json';
import windowAll from '../fixtures/bikri_past_window_all.json';
import windowAfterEdit from '../fixtures/bikri_past_window_after_edit.json';
import windowBack from '../fixtures/bikri_past_window_back.json';
import windowBackRequest from '../fixtures/bikri_past_window_back_request.json';
import windowEmpty from '../fixtures/bikri_past_window_empty.json';
import windowSecondPage from '../fixtures/bikri_past_window_second_page.json';
import windowSeller from '../fixtures/bikri_past_window_seller.json';
import futureRefused from '../fixtures/bikri_past_future_refused.json';
import type { BikriList } from '../../src/bindings/BikriList';

/** The fixture files are the backend's own JSON; their text is read as the types of the list. */
function listOf(value: unknown): BikriList {
  return value as BikriList;
}

/** The day these files were written for, as the backend's day status says it. */
export const PAST_TODAY_ISO = '2026-11-12';

export const PAST_TODAY: BikriList = listOf(today);
export const PAST_TODAY_REQUEST: Record<string, unknown> = todayRequest;
export const PAST_WINDOW: BikriList = listOf(window);
export const PAST_WINDOW_REQUEST: Record<string, unknown> = windowRequest;
export const PAST_WINDOW_ALL: BikriList = listOf(windowAll);
export const PAST_WINDOW_AFTER_EDIT: BikriList = listOf(windowAfterEdit);
export const PAST_WINDOW_BACK: BikriList = listOf(windowBack);
export const PAST_WINDOW_BACK_REQUEST: Record<string, unknown> = windowBackRequest;
export const PAST_WINDOW_EMPTY: BikriList = listOf(windowEmpty);
export const PAST_WINDOW_SECOND_PAGE: BikriList = listOf(windowSecondPage);
export const PAST_WINDOW_SELLER: BikriList = listOf(windowSeller);

/** The refusal of a window that ends after today: the code, the field and the day it says. */
export const PAST_FUTURE_REFUSED = futureRefused as {
  errors: { code: string; field: string | null; values: Record<string, string> }[];
};
