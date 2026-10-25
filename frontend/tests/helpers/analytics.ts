// Test helper: sets the answers of the analytics command wrappers (plan 21) from the backend's own
// answers in analytics-real.ts. The wrappers are mocked in the test file with vi.mock. A view is
// answered the way the real engine does it: the rows of the whole filter are sliced into the page
// that was asked for, with the same total and totals on every page. A refusal is a rejected promise
// (`CommandError`), the way `invoke` fails. The saved views are kept in a list that a save replaces,
// the way the backend keeps them.

import { vi } from 'vitest';
import type { AnalyticsLinesPage } from '../../src/bindings/AnalyticsLinesPage';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsPrintData } from '../../src/bindings/AnalyticsPrintData';
import type { AnalyticsSavedView } from '../../src/bindings/AnalyticsSavedView';
import type { AnalyticsView } from '../../src/bindings/AnalyticsView';
import type { AnalyticsRunInput } from '../../src/bindings/AnalyticsRunInput';
import type { UiError } from '../../src/bindings/UiError';
import * as analyticsApi from '../../src/features/analytics/api';
import { CommandError } from '../../src/shared/api';
import {
  ANALYTICS_CATALOG,
  ANALYTICS_LINES_A_BANANA,
  ANALYTICS_LINES_D_BANANA_OCTOBER,
  ANALYTICS_LINES_E_RAHIM_BANANA,
  ANALYTICS_PRINT_DATA_A_EN,
  ANALYTICS_PRINT_DATA_D_UR,
  ANALYTICS_RUN_A,
  ANALYTICS_RUN_A_COMPARE,
  ANALYTICS_RUN_A_COMPARE_NEEDS_DATES,
  ANALYTICS_RUN_A_GROUP_BY_REFUSED,
  ANALYTICS_RUN_A_SELLER_SIDE,
  ANALYTICS_RUN_A_UNKNOWN_ITEM,
  ANALYTICS_RUN_B,
  ANALYTICS_RUN_C,
  ANALYTICS_RUN_D_DAY,
  ANALYTICS_RUN_D_MONTH,
  ANALYTICS_RUN_E,
  ANALYTICS_RUN_H,
  VIEWS_GET_EMPTY,
  VIEWS_SAVE_BAD,
} from './analytics-real';
import {
  ANALYTICS_LINES_I_GARDEN,
  ANALYTICS_PRINT_DATA_I_EN,
  ANALYTICS_RUN_A_GARDEN_GOODS,
  ANALYTICS_RUN_G_GARDEN_DAY,
  ANALYTICS_RUN_I_GARDEN,
  ANALYTICS_RUN_I_GARDEN_SELLER_SIDE,
} from './analytics-gardens-real';
import {
  ANALYTICS_LINES_F_FRUIT_ALIF,
  ANALYTICS_LINES_K_GUESTS,
  ANALYTICS_PRINT_DATA_K_EN,
  ANALYTICS_RUN_F_FRUIT,
  ANALYTICS_RUN_K_FRUIT,
} from './analytics-fruit-real';
import {
  ANALYTICS_LINES_F_PARTNER_A,
  ANALYTICS_LINES_J_TRIP,
  ANALYTICS_PRINT_DATA_J_EN,
  ANALYTICS_RUN_A_GOODS_CONSIGNED,
  ANALYTICS_RUN_A_GOODS_TRIP,
  ANALYTICS_RUN_F,
  ANALYTICS_RUN_F_COMPARE_REFUSED,
  ANALYTICS_RUN_G_DAY,
  ANALYTICS_RUN_G_MONTH,
  ANALYTICS_RUN_J_TRIP,
  ANALYTICS_RUN_J_TRIP_BY_PURCHASE,
  ANALYTICS_RUN_J_TRIP_PARTNER,
} from './analytics-trips-real';

import { fakePrintFrame } from './print-frame';

export * from './analytics-real';
export * from './analytics-trips-real';
export * from './analytics-gardens-real';
export * from './analytics-fruit-real';

/** A backend refusal, as a rejected promise. */
export function refusedWith(error: UiError): CommandError {
  return new CommandError(error);
}

export interface AnalyticsOptions {
  catalog?: typeof ANALYTICS_CATALOG | Error;
  /** Pages by view; the default is the backend's own answer for each. */
  pages?: Partial<Record<AnalyticsView, AnalyticsPage>>;
  /** Called for every `analytics_run`; return a refusal to refuse that run. */
  refuse?: (input: AnalyticsRunInput) => UiError | null;
  print?: AnalyticsPrintData;
  /** The saved views to start with. */
  saved?: AnalyticsSavedView[] | Error;
  lines?: AnalyticsLinesPage | Error;
  /** View F answers with the fruit column filled (the fruit world of E2E-42). */
  fruitInF?: boolean;
}

const DEFAULT_PAGES: Record<AnalyticsView, AnalyticsPage> = {
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
  K: ANALYTICS_RUN_K_FRUIT,
};

/** The views that have no comparison (the catalogue says so): the backend refuses one. */
const NO_COMPARE: readonly AnalyticsView[] = ['F', 'G', 'I', 'J', 'K'];
/** The views that refuse a garden: the backend answers ERR_OUT_OF_RANGE on `filters.garden_id`. */
const NO_GARDEN: readonly AnalyticsView[] = ['F', 'J'];
/** The views that take a grouping. */
const GROUPED: readonly AnalyticsView[] = ['D', 'G'];

/** The refusal of a garden on a view that takes none. */
const GARDEN_REFUSED: UiError = {
  ok: false,
  errors: [{ code: 'ERR_OUT_OF_RANGE', field: 'filters.garden_id', values: {} }],
  ref: 'E-20261010-0002',
};

function slice(page: AnalyticsPage, offset: number, limit: number): AnalyticsPage {
  const size = limit > 0 ? limit : 100;
  return { ...page, rows: page.rows.slice(offset, offset + size), offset, limit };
}

export interface FakeAnalytics {
  /** The saved views the backend keeps now (a save replaces them). */
  saved: () => AnalyticsSavedView[];
}

/** Sets the answers of the analytics commands, and returns what the backend would keep. */
export function fakeAnalytics(options: AnalyticsOptions = {}): FakeAnalytics {
  const pages: Record<AnalyticsView, AnalyticsPage> = { ...DEFAULT_PAGES, ...options.pages };
  let saved: AnalyticsSavedView[] = Array.isArray(options.saved) ? [...options.saved] : [];
  vi.mocked(analyticsApi.analyticsCatalog).mockImplementation(() =>
    options.catalog instanceof Error
      ? Promise.reject(options.catalog)
      : Promise.resolve([...(options.catalog ?? ANALYTICS_CATALOG)]),
  );
  vi.mocked(analyticsApi.analyticsRun).mockImplementation((input) => {
    const refused = options.refuse?.(input) ?? null;
    if (refused !== null) return Promise.reject(refusedWith(refused));
    const { filters, view } = input;
    // The refusals of the backend's own fixtures: a grouping on a view that has none, an unknown
    // item, and a comparison without both dates.
    if (!GROUPED.includes(view) && filters.group_by !== null) {
      return Promise.reject(refusedWith(ANALYTICS_RUN_A_GROUP_BY_REFUSED));
    }
    if (NO_COMPARE.includes(view) && filters.compare !== 'off') {
      return Promise.reject(refusedWith(ANALYTICS_RUN_F_COMPARE_REFUSED));
    }
    if (NO_GARDEN.includes(view) && filters.garden_id !== null) {
      return Promise.reject(refusedWith(GARDEN_REFUSED));
    }
    if (filters.item_id === 9999) {
      return Promise.reject(refusedWith(ANALYTICS_RUN_A_UNKNOWN_ITEM));
    }
    if (
      filters.compare !== 'off' &&
      (filters.from === '' || filters.to === '') &&
      filters.period === null
    ) {
      return Promise.reject(refusedWith(ANALYTICS_RUN_A_COMPARE_NEEDS_DATES));
    }
    let answer = pages[view];
    if (view === 'A' && filters.compare !== 'off') answer = ANALYTICS_RUN_A_COMPARE;
    if (view === 'A' && filters.side === 'seller') answer = ANALYTICS_RUN_A_SELLER_SIDE;
    if (view === 'A' && filters.garden_id !== null) answer = ANALYTICS_RUN_A_GARDEN_GOODS;
    if (view === 'G' && filters.garden_id !== null) answer = ANALYTICS_RUN_G_GARDEN_DAY;
    if (view === 'I' && filters.side === 'seller') answer = ANALYTICS_RUN_I_GARDEN_SELLER_SIDE;
    if (view === 'A' && filters.goods === 'consigned') answer = ANALYTICS_RUN_A_GOODS_CONSIGNED;
    if (view === 'A' && filters.goods === 'trip') answer = ANALYTICS_RUN_A_GOODS_TRIP;
    if (view === 'D' && filters.group_by === 'day') answer = ANALYTICS_RUN_D_DAY;
    if (view === 'G' && filters.group_by === 'day') answer = ANALYTICS_RUN_G_DAY;
    if (view === 'J' && filters.trip_view === 'purchase') {
      answer = ANALYTICS_RUN_J_TRIP_BY_PURCHASE;
    }
    if (view === 'J' && filters.partner_id !== null) answer = ANALYTICS_RUN_J_TRIP_PARTNER;
    // View F with the fruit of E2E-42 answers when the test asks for it by the fruit option.
    if (view === 'F' && options.fruitInF === true) answer = ANALYTICS_RUN_F_FRUIT;
    return Promise.resolve(slice(answer, input.offset, input.limit));
  });
  vi.mocked(analyticsApi.analyticsLines).mockImplementation((input) => {
    if (options.lines instanceof Error) return Promise.reject(options.lines);
    const page =
      options.lines ??
      (input.view === 'D'
        ? ANALYTICS_LINES_D_BANANA_OCTOBER
        : input.view === 'E'
          ? ANALYTICS_LINES_E_RAHIM_BANANA
          : input.view === 'F'
            ? input.column_key?.money === 'fruit'
              ? ANALYTICS_LINES_F_FRUIT_ALIF
              : ANALYTICS_LINES_F_PARTNER_A
            : input.view === 'J'
              ? ANALYTICS_LINES_J_TRIP
              : input.view === 'I'
                ? ANALYTICS_LINES_I_GARDEN
                : input.view === 'K'
                  ? ANALYTICS_LINES_K_GUESTS
                  : ANALYTICS_LINES_A_BANANA);
    const size = input.limit > 0 ? input.limit : 100;
    return Promise.resolve({
      ...page,
      rows: page.rows.slice(input.offset, input.offset + size),
      offset: input.offset,
      limit: input.limit,
    });
  });
  fakePrintFrame();
  vi.mocked(analyticsApi.analyticsPrintData).mockImplementation((input) =>
    Promise.resolve(
      options.print ??
        (input.view === 'D'
          ? ANALYTICS_PRINT_DATA_D_UR
          : input.view === 'J'
            ? ANALYTICS_PRINT_DATA_J_EN
            : input.view === 'I'
              ? ANALYTICS_PRINT_DATA_I_EN
              : input.view === 'K'
                ? ANALYTICS_PRINT_DATA_K_EN
                : ANALYTICS_PRINT_DATA_A_EN),
    ),
  );
  vi.mocked(analyticsApi.analyticsViewsGet).mockImplementation(() =>
    options.saved instanceof Error ? Promise.reject(options.saved) : Promise.resolve([...saved]),
  );
  vi.mocked(analyticsApi.analyticsViewsSave).mockImplementation((input) => {
    // A name with nothing in it is refused, as the backend's own fixture shows.
    if (input.views.some((view) => view.name.trim() === '')) {
      return Promise.reject(refusedWith(VIEWS_SAVE_BAD));
    }
    saved = input.views.map((view) => ({ ...view, name: view.name.trim() }));
    return Promise.resolve([...saved]);
  });
  return { saved: () => [...saved] };
}

export { VIEWS_GET_EMPTY };
