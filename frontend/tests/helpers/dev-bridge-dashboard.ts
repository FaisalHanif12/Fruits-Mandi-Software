// A stand-in for the Tauri bridge for the development page /dev/dashboard (round 9) only. In a plain
// browser tab there is no backend, so the dashboard cannot be looked at. This answers its commands
// with the backend's own fixtures (dashboard-real.ts, b5-real.ts, backup-real.ts): the dashboard of
// the worked day, the attention list, the backup state, the last books check, the update check, and
// the choices a screen remembers (kept in memory while the page is open). It answers nothing else:
// any other command is refused with ERR_NOT_IMPLEMENTED. Never part of a build: only the development
// route imports it.
//
// Flags in the hash, for example `#/dev/dashboard?many`:
//   empty     a new book: no sales, no dues, no tasks   many      the dashboard is full: 12 بکری, all tasks
//   down      the dashboard call fails                   quiet     nothing needs attention
//   cashneg   cash in hand is below zero                 lent      money lent is past its return-by date
//   update    a new version is ready                     bad       the books check found a problem
//   fresh     the first-run guide has not been seen      none      no books check has been run yet
//   cloudold  the last cloud backup is old       garden    the garden world of E2E-40: one open garden

import type { AttentionItem } from '../../src/bindings/AttentionItem';
import type { AttentionList } from '../../src/bindings/AttentionList';
import type { AuthStatus } from '../../src/bindings/AuthStatus';
import type { DailyHealth } from '../../src/bindings/DailyHealth';
import type { Dashboard } from '../../src/bindings/Dashboard';
import type { DashboardBikri } from '../../src/bindings/DashboardBikri';
import type { ScreenState } from '../../src/bindings/ScreenState';
import { HEALTH_LAST, UPDATE_AVAILABLE, UPDATE_NONE } from './b5-real';
import { STATUS_AFTER } from './backup-real';
import { ATTENTION_18_10, DASHBOARD_GARDEN, DASHBOARD_WORKED_DAY } from './dashboard-real';
import { devFlag } from './dev-bridge';
import { paisaForTest } from './paisa';

export type Input = Record<string, unknown>;

/** What screens remembered while the page is open: the screen's name to its text. */
const remembered = new Map<string, string>();

const EMPTY_TILES = {
  sales_gross_paisa: paisaForTest(0),
  sales_count: 0,
  commission_paisa: paisaForTest(0),
  cash_in_hand_paisa: paisaForTest(5_000_000),
  buyers_dues_paisa: paisaForTest(0),
  sellers_dues_paisa: paisaForTest(0),
  total_dues_paisa: paisaForTest(0),
  dead_amount_paisa: paisaForTest(0),
  dead_count: 0,
  sellers_payable_paisa: paisaForTest(0),
  advances_open_paisa: paisaForTest(0),
  stock_lots: 0,
  stock_qty: 0,
  trips_open: 0,
  trips_purchases_with_stock: 0,
  trips_qty_left: 0,
  trips_cost_left_paisa: paisaForTest(0),
  gardens_open: 0,
  gardens_balance_paisa: paisaForTest(0),
  lent_open_paisa: paisaForTest(0),
  lent_overdue_paisa: paisaForTest(0),
  expenses_today_paisa: paisaForTest(0),
} as const;

/** Every kind of the attention list once, with figures like a busy book (for the `many` flag). */
function manyTasks(): AttentionList {
  const items: AttentionItem[] = [
    { kind: 'drafts', count: 2, amount_paisa: null, qty: null, days: null, first: null },
    { kind: 'open_days', count: 8, amount_paisa: null, qty: null, days: 8, first: null },
    {
      kind: 'stock_waiting',
      count: 3,
      amount_paisa: null,
      qty: 42,
      days: 4,
      first: { target: 'bikri', id: 1, person_key: null, name_ur: null, name_en: null },
    },
    {
      kind: 'over_credit_limit',
      count: 1,
      amount_paisa: null,
      qty: null,
      days: null,
      first: {
        target: 'account',
        id: 20,
        person_key: null,
        name_ur: 'اکرم ٹول ٹیکس',
        name_en: 'Akram Toll Tax',
      },
    },
    {
      kind: 'dues_old',
      count: 2,
      amount_paisa: paisaForTest(2_156_400),
      qty: null,
      days: 41,
      first: {
        target: 'account',
        id: 21,
        person_key: null,
        name_ur: 'ارشد ملتانی',
        name_en: 'Arshad Multani',
      },
    },
    {
      kind: 'payables_old',
      count: 1,
      amount_paisa: paisaForTest(1_016_000),
      qty: null,
      days: 10,
      first: { target: 'account', id: 19, person_key: null, name_ur: 'کریم', name_en: 'Karim' },
    },
    { kind: 'backup_old', count: 1, amount_paisa: null, qty: null, days: 2, first: null },
    {
      kind: 'loans_overdue',
      count: 1,
      amount_paisa: paisaForTest(200_000),
      qty: null,
      days: 1,
      first: { target: 'account', id: 31, person_key: null, name_ur: 'اسلم', name_en: 'Aslam' },
    },
    {
      kind: 'takes_overdue',
      count: 1,
      amount_paisa: paisaForTest(200_000),
      qty: null,
      days: 3,
      first: { target: 'partner', id: 1, person_key: null, name_ur: 'الف', name_en: 'Partner A' },
    },
    { kind: 'names_missing', count: 4, amount_paisa: null, qty: null, days: null, first: null },
  ];
  return { items, total_count: items.reduce((sum, item) => sum + item.count, 0) };
}

function attention(): AttentionList {
  if (devFlag('empty') || devFlag('quiet')) return { items: [], total_count: 0 };
  return devFlag('many') ? manyTasks() : ATTENTION_18_10;
}

/** More rows of today's بکری than fit, to see the list with its count (the `many` flag). */
function manyBikri(): { rows: DashboardBikri[]; count: number } {
  const first = DASHBOARD_WORKED_DAY.bikri_today;
  const rows: DashboardBikri[] = [];
  for (let at = 0; at < 12; at += 1) {
    const base = first[at % first.length];
    if (base === undefined) continue;
    rows.push({
      ...base,
      bikri_id: 100 + at,
      number: 3 + at,
      seller_count: at % 4 === 3 ? 2 : 1,
      cancelled: at === 5,
    });
  }
  return { rows, count: 31 };
}

function dashboard(): Dashboard {
  if (devFlag('empty')) {
    return {
      ...DASHBOARD_WORKED_DAY,
      tiles: EMPTY_TILES,
      partners: DASHBOARD_WORKED_DAY.partners.map((partner) => ({
        ...partner,
        today_paisa: paisaForTest(0),
        month_paisa: paisaForTest(0),
        year_paisa: paisaForTest(0),
        total_paisa: paisaForTest(0),
      })),
      gardens: [],
      bikri_today: [],
      bikri_today_count: 0,
      oldest_dues: [],
      chart_days: DASHBOARD_WORKED_DAY.chart_days.map((day) => ({
        ...day,
        gross_paisa: paisaForTest(0),
        commission_paisa: paisaForTest(0),
      })),
      chart_items: [],
      attention: attention(),
    };
  }
  // The garden world of E2E-40 has the same day with one open garden and its balance.
  const base = devFlag('garden') ? DASHBOARD_GARDEN : DASHBOARD_WORKED_DAY;
  let result: Dashboard = { ...base, attention: attention() };
  if (devFlag('many')) {
    const many = manyBikri();
    result = {
      ...result,
      bikri_today: many.rows,
      bikri_today_count: many.count,
      tiles: {
        ...result.tiles,
        stock_lots: 3,
        stock_qty: 42,
        trips_open: 1,
        trips_purchases_with_stock: 1,
        trips_qty_left: 5,
        trips_cost_left_paisa: paisaForTest(260_000),
        lent_open_paisa: paisaForTest(500_000),
        lent_overdue_paisa: paisaForTest(200_000),
      },
    };
  }
  if (devFlag('lent')) {
    result = {
      ...result,
      tiles: {
        ...result.tiles,
        lent_open_paisa: paisaForTest(500_000),
        lent_overdue_paisa: paisaForTest(200_000),
      },
    };
  }
  // Plan 40 (E2E-44): ارشد is in the dead amount; `dead` is after the move, `recovered` after the 4,000.
  if (devFlag('dead') || devFlag('recovered')) {
    const recovered = devFlag('recovered');
    result = {
      ...result,
      tiles: {
        ...result.tiles,
        buyers_dues_paisa: paisaForTest(288_000),
        sellers_dues_paisa: paisaForTest(0),
        total_dues_paisa: paisaForTest(recovered ? 1_756_400 : 2_156_400),
        dead_amount_paisa: paisaForTest(recovered ? 1_468_400 : 1_868_400),
        dead_count: 1,
      },
    };
  }
  if (devFlag('cashneg')) {
    result = { ...result, tiles: { ...result.tiles, cash_in_hand_paisa: paisaForTest(-500_000) } };
  }
  return result;
}

function health(): DailyHealth | null {
  if (devFlag('none')) return null;
  const checkedAt = '2026-10-10T09:05:00Z';
  if (!devFlag('bad')) return { ...HEALTH_LAST, checked_at: checkedAt };
  return { ...HEALTH_LAST, checked_at: checkedAt, ok: false };
}

const AUTH: AuthStatus = {
  first_run: false,
  session: { id: 1, username: 'owner', display_name: 'Adnan', language: 'ur', text_size: 100 },
  locked: false,
  login_language: 'ur',
  firm: {
    name_ur: 'مہر عدنان اینڈ برادرز',
    name_en: 'Maher Adnan and Brothers',
    mandi_ur: 'نئی سبزی منڈی، کوٹ رادھا کشن',
    mandi_en: 'New Sabzi Mandi, Kot Radha Kishan',
  },
  app_version: '0.1.12',
};

function screenOf(input: Input): string {
  return typeof input['screen'] === 'string' ? input['screen'] : '';
}

function stateOf(screen: string): ScreenState {
  // The first-run guide counts as seen, unless the `fresh` flag asks to see it.
  if (screen === 'app.guide' && !devFlag('fresh') && !remembered.has(screen)) {
    return { screen, value: '{"done":true}' };
  }
  return { screen, value: remembered.get(screen) ?? null };
}

export const dashboardHandlers: Record<string, (input: Input) => unknown> = {
  auth_status: () => AUTH,
  day_status: () => ({ today: '2026-10-10', closed_up_to: null, today_open: true }),
  dashboard_get: () => {
    if (devFlag('down')) throw refusal('ERR_INTERNAL');
    return dashboard();
  },
  assist_attention: () => attention(),
  backup_status: () =>
    devFlag('cloudold')
      ? { ...STATUS_AFTER, cloud_level: 'red' as const, cloud_age_hours: 60 }
      : STATUS_AFTER,
  health_last: () => health(),
  updates_check: () => (devFlag('update') ? UPDATE_AVAILABLE : UPDATE_NONE),
  prefs_screen_get: (input) => stateOf(screenOf(input)),
  prefs_screen_set: (input) => {
    const screen = screenOf(input);
    const value = typeof input['value'] === 'string' ? input['value'] : '';
    if (value === '') remembered.delete(screen);
    else remembered.set(screen, value);
    return stateOf(screen);
  },
  prefs_set: () => null,
};

/** What Tauri rejects with for a command this page does not answer, or for a failure. */
export function refusal(code: string): Error {
  return Object.assign(new Error(code), {
    ok: false,
    errors: [{ code, field: null, values: {} }],
    ref: 'E-20261010-0001',
  });
}

export function inputOf(args: unknown): Input {
  if (typeof args !== 'object' || args === null) return {};
  const inner = (args as Input)['input'];
  return typeof inner === 'object' && inner !== null ? (inner as Input) : {};
}

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installDashboardDevBridge(): void {
  remembered.clear();
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      transformCallback: () => 0,
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const handler = dashboardHandlers[command];
            if (handler === undefined) {
              reject(refusal('ERR_NOT_IMPLEMENTED'));
              return;
            }
            try {
              resolve(handler(inputOf(args)));
            } catch (error) {
              reject(error instanceof Error ? error : refusal('ERR_INTERNAL'));
            }
          }, 20);
        }),
    },
  });
}
