// A stand-in for the Tauri bridge for the development pages /dev/analytics (plan 21) only. In a plain
// browser tab there is no backend, so the analytics screens cannot be looked at. This answers the six
// analytics commands with the backend's own fixtures (analytics-real.ts), so the screens can be
// opened in a browser and photographed in both languages and at every window size. It answers
// nothing else: any other command is refused with ERR_NOT_IMPLEMENTED. Never part of a build: only
// the development route imports it.
//
// Flags in the hash, for example `#/dev/analytics/A?many`:
//   empty   the view has no rows          many   230 rows (page by page)
//   down    the backend fails              wide   view D with 12 months, view E with 8 items
// A comparison without both dates is refused, as the backend refuses it (ERR_REQUIRED under the two
// date boxes), so the refusal can be looked at.

import type { AnalyticsColumn } from '../../src/bindings/AnalyticsColumn';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsSavedView } from '../../src/bindings/AnalyticsSavedView';
import type { AnalyticsView } from '../../src/bindings/AnalyticsView';
import type { ReportCell } from '../../src/bindings/ReportCell';
import type { GardensListInput } from '../../src/bindings/GardensListInput';
import type { TripsListInput } from '../../src/bindings/TripsListInput';
import { devFlag } from './dev-bridge';
import { devExport, exportPluginCommand, transformCallback } from './dev-bridge-export';
import { rokarHandlers } from './dev-bridge-rokar';
import { createGardenBooks } from './gardens-data';
import { createTripBooks } from './trips-data';
import {
  ANALYTICS_CATALOG,
  ANALYTICS_LINES_A_BANANA,
  ANALYTICS_PRINT_DATA_A_EN,
  ANALYTICS_PRINT_DATA_D_UR,
  ANALYTICS_RUN_A,
  ANALYTICS_RUN_A_COMPARE,
  ANALYTICS_RUN_A_COMPARE_NEEDS_DATES,
  ANALYTICS_RUN_B,
  ANALYTICS_RUN_C,
  ANALYTICS_RUN_D_DAY,
  ANALYTICS_RUN_D_MONTH,
  ANALYTICS_RUN_E,
  ANALYTICS_RUN_H,
  VIEWS_GET_TWO,
} from './analytics-real';
import {
  ANALYTICS_PRINT_DATA_I_EN,
  ANALYTICS_RUN_I_GARDEN,
  ANALYTICS_RUN_I_GARDEN_SELLER_SIDE,
} from './analytics-gardens-real';
import {
  ANALYTICS_PRINT_DATA_J_EN,
  ANALYTICS_RUN_F,
  ANALYTICS_RUN_G_DAY,
  ANALYTICS_RUN_G_MONTH,
  ANALYTICS_RUN_J_TRIP,
  ANALYTICS_RUN_J_TRIP_BY_PURCHASE,
} from './analytics-trips-real';

type Input = Record<string, unknown>;

const PAGES: Record<AnalyticsView, AnalyticsPage> = {
  A: ANALYTICS_RUN_A,
  B: ANALYTICS_RUN_B,
  C: ANALYTICS_RUN_C,
  D: ANALYTICS_RUN_D_MONTH,
  E: ANALYTICS_RUN_E,
  F: ANALYTICS_RUN_F,
  G: ANALYTICS_RUN_G_MONTH,
  H: ANALYTICS_RUN_H,
  I: ANALYTICS_RUN_I_GARDEN,
  J: ANALYTICS_RUN_J_TRIP,
};

/** Views D and E with more columns than a window holds, to show the window and its buttons. */
function wide(page: AnalyticsPage): AnalyticsPage {
  const headed = page.columns.findIndex((column) => column.heading !== null);
  const template = page.columns[headed];
  if (headed < 0 || template === undefined) return page;
  const copies = page.view === 'D' ? 12 : 8;
  const extra: AnalyticsColumn[] = Array.from({ length: copies - 1 }, (_, index) => ({
    ...template,
    key: `${template.key}_x${String(index)}`,
    heading:
      template.heading?.kind === 'month'
        ? {
            kind: 'month',
            year: template.heading.year,
            month: ((template.heading.month + index) % 12) + 1,
          }
        : template.heading,
  }));
  const columns = [
    ...page.columns.slice(0, headed + 1),
    ...extra,
    ...page.columns.slice(headed + 1),
  ];
  const grow = (cells: ReportCell[]) => {
    const own = cells[headed];
    return own === undefined
      ? cells
      : [...cells.slice(0, headed + 1), ...extra.map(() => own), ...cells.slice(headed + 1)];
  };
  return {
    ...page,
    columns,
    rows: page.rows.map((row) => ({ ...row, cells: grow(row.cells) })),
    totals: grow(page.totals),
  };
}

function page(input: Input): AnalyticsPage {
  const view = input['view'] as AnalyticsView;
  const filters = (input['filters'] ?? {}) as Record<string, unknown>;
  const compared = filters['compare'] !== 'off';
  const hasDates = filters['period'] !== null || (filters['from'] !== '' && filters['to'] !== '');
  if (compared && !hasDates) {
    throw Object.assign(new Error('ERR_REQUIRED'), ANALYTICS_RUN_A_COMPARE_NEEDS_DATES);
  }
  let base = PAGES[view];
  if (view === 'A' && compared) base = ANALYTICS_RUN_A_COMPARE;
  if (view === 'D' && filters['group_by'] === 'day') base = ANALYTICS_RUN_D_DAY;
  if (view === 'G' && filters['group_by'] === 'day') base = ANALYTICS_RUN_G_DAY;
  if (view === 'J' && filters['trip_view'] === 'purchase') base = ANALYTICS_RUN_J_TRIP_BY_PURCHASE;
  if (view === 'I' && filters['side'] === 'seller') base = ANALYTICS_RUN_I_GARDEN_SELLER_SIDE;
  if (devFlag('wide')) base = wide(base);
  const firstRow = base.rows[0];
  if (devFlag('many') && firstRow !== undefined) {
    base = { ...base, rows: Array.from({ length: 230 }, () => firstRow), total_count: 230 };
  }
  if (devFlag('empty')) base = { ...base, rows: [], total_count: 0 };
  const offset = Number(input['offset'] ?? 0);
  const limit = Number(input['limit'] ?? 100) || 100;
  return { ...base, rows: base.rows.slice(offset, offset + limit), offset, limit };
}

let saved: AnalyticsSavedView[] = [...VIEWS_GET_TWO];

/** The trips of the trip box (view J, the trip filter): the worked trip of the trips module's books. */
const tripBooks = createTripBooks();

/** The gardens of the garden box (view I, the garden filter): the garden world of the gardens module. */
const gardenBooks = createGardenBooks();

const handlers: Record<string, (input: Input) => unknown> = {
  analytics_export_excel: (input) => devExport(input),
  analytics_catalog: () => ANALYTICS_CATALOG,
  analytics_run: (input) => page(input),
  analytics_lines: (input) => {
    const offset = Number(input['offset'] ?? 0);
    const limit = Number(input['limit'] ?? 100) || 100;
    return {
      ...ANALYTICS_LINES_A_BANANA,
      rows: ANALYTICS_LINES_A_BANANA.rows.slice(offset, offset + limit),
      offset,
      limit,
    };
  },
  analytics_print_data: (input) =>
    input['view'] === 'D'
      ? ANALYTICS_PRINT_DATA_D_UR
      : input['view'] === 'J'
        ? ANALYTICS_PRINT_DATA_J_EN
        : input['view'] === 'I'
          ? ANALYTICS_PRINT_DATA_I_EN
          : ANALYTICS_PRINT_DATA_A_EN,
  // The partner box (F, G, J) and the trip box read these, so the boxes work in the browser.
  partners_list: (input) => rokarHandlers['partners_list']?.(input),
  trips_list: (input) => tripBooks.list(input as TripsListInput),
  gardens_list: (input) => gardenBooks.list(input as GardensListInput),
  analytics_views_get: () => saved,
  analytics_views_save: (input) => {
    saved = (input['views'] ?? []) as AnalyticsSavedView[];
    return saved;
  },
};

/** What Tauri rejects with for a command this page does not answer, or for the `down` flag. */
function refusal(code: string): Error {
  return Object.assign(new Error(code), {
    ok: false,
    errors: [{ code, field: null, values: {} }],
    ref: 'E-20261010-0001',
  });
}

function inputOf(args: unknown): Input {
  if (typeof args !== 'object' || args === null) return {};
  const inner = (args as Input)['input'];
  return typeof inner === 'object' && inner !== null ? (inner as Input) : {};
}

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installAnalyticsDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      transformCallback,
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const plugin = exportPluginCommand(command, args);
            if (plugin.handled) {
              resolve(plugin.value);
              return;
            }
            const handler = handlers[command];
            if (handler === undefined) {
              reject(refusal('ERR_NOT_IMPLEMENTED'));
              return;
            }
            if (devFlag('down')) {
              reject(refusal('ERR_INTERNAL'));
              return;
            }
            try {
              resolve(handler(inputOf(args)));
            } catch (error) {
              // A refusal of the handler (a comparison without dates) is already an Error.
              reject(error instanceof Error ? error : refusal('ERR_INTERNAL'));
            }
          }, 20);
        }),
    },
  });
}
