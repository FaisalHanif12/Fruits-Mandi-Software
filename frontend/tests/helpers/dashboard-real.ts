// The real answers of the backend for round 9 of the app (plan 19): the dashboard of the worked day
// (PRD 4.9), the attention list of 18-10 (8 earlier days open, a payable more than 7 days old, the
// last backup old), and what a screen remembers. The backend wrote them in its own test
// (backend/crates/services/tests/fixtures); they are copied here by `scripts/copy-backend-fixtures.mjs`,
// never edited, and read as the generated types. Money in them is paisa, as on the wire. A reader
// takes `unknown`, because the money in a binding is a branded type that a plain number is not.

import type { AttentionList } from '../../src/bindings/AttentionList';
import type { Dashboard } from '../../src/bindings/Dashboard';
import type { ScreenState } from '../../src/bindings/ScreenState';
import type { ScreenStateSetInput } from '../../src/bindings/ScreenStateSetInput';
import attention1810 from '../fixtures/assist_attention_18_10.json';
import dashboardWorkedDay from '../fixtures/dashboard_get_worked_day.json';
import prefsScreenSet from '../fixtures/prefs_screen_set.json';
import prefsScreenSetRequest from '../fixtures/prefs_screen_set_request.json';

function readDashboard(value: unknown): Dashboard {
  return value as Dashboard;
}
function readAttention(value: unknown): AttentionList {
  return value as AttentionList;
}

/** The dashboard after Events 1 to 5 of the worked day: sales 41,400 in 2 بکری, cash 58,484. */
export const DASHBOARD_WORKED_DAY = readDashboard(dashboardWorkedDay);

/** Eight earlier days not closed, a payable of 10,160 to کریم, the last cloud backup a day old. */
export const ATTENTION_18_10 = readAttention(attention1810);

/** What a screen saved: the screen `report.r20` kept `{"city_id":3}`. */
export const PREFS_SCREEN_SET: ScreenState = prefsScreenSet;
export const PREFS_SCREEN_SET_REQUEST: ScreenStateSetInput = prefsScreenSetRequest;
