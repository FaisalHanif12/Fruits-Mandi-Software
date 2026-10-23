// The screens and the backend agree on plan 21 (analytics): each wrapper in features/analytics/api.ts
// sends the command name the backend registers and the input in the shape `{ input }` (the catalogue
// and the saved-views read send only their name). The names are also read from the backend's own
// command file, so a command renamed there fails here first. The fixtures in tests/fixtures are the
// backend's own, copied; here they must still be the same bytes as the backend's, must carry exactly
// the fields of the generated types, and every message ID the backend sends (titles, columns, labels)
// must have words in Urdu and in English.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as api from '../../src/features/analytics/api';
import { bundles, LANGUAGES } from '../../src/i18n';
import * as invoke from '../../src/shared/api/invoke';
import {
  ANALYTICS_CATALOG,
  ANALYTICS_LINES_A_BANANA,
  ANALYTICS_LINES_A_BANANA_REQUEST,
  ANALYTICS_LINES_D_BANANA_OCTOBER,
  ANALYTICS_LINES_E_RAHIM_BANANA,
  ANALYTICS_PRINT_DATA_A_EN,
  ANALYTICS_PRINT_DATA_A_EN_REQUEST,
  ANALYTICS_PRINT_DATA_D_UR,
  ANALYTICS_RUN_A,
  ANALYTICS_RUN_A_COMPARE,
  ANALYTICS_RUN_A_REQUEST,
  ANALYTICS_RUN_B,
  ANALYTICS_RUN_C,
  ANALYTICS_RUN_D_DAY,
  ANALYTICS_RUN_D_MONTH,
  ANALYTICS_RUN_E,
  ANALYTICS_RUN_H,
  VIEWS_GET_TWO,
  VIEWS_SAVE_TWO_REQUEST,
} from '../helpers/analytics-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const BACKEND_FILE = path.resolve(__dirname, '../../../backend/app/src/commands/analytics.rs');
const BACKEND_FIXTURES = path.resolve(__dirname, '../../../backend/crates/services/tests/fixtures');
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

const isAnalyticsFixture = (name: string) =>
  name.startsWith('analytics_') && name.endsWith('.json');

describe('the commands the screens call are the ones the backend declares', () => {
  it('every analytics command used in api.ts exists in the backend file', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    const declared = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    for (const name of [
      'analytics_catalog',
      'analytics_run',
      'analytics_lines',
      'analytics_print_data',
      'analytics_views_get',
      'analytics_views_save',
    ]) {
      expect(declared).toContain(name);
    }
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  it('the catalogue and the saved-views read send only their name', async () => {
    await api.analyticsCatalog();
    await api.analyticsViewsGet();
    expect(call.mock.calls).toEqual([['analytics_catalog'], ['analytics_views_get']]);
  });

  it('the others send the exact request the backend wrote', async () => {
    await api.analyticsRun(ANALYTICS_RUN_A_REQUEST);
    await api.analyticsLines(ANALYTICS_LINES_A_BANANA_REQUEST);
    await api.analyticsPrintData(ANALYTICS_PRINT_DATA_A_EN_REQUEST);
    await api.analyticsViewsSave(VIEWS_SAVE_TWO_REQUEST);
    expect(call.mock.calls).toEqual([
      ['analytics_run', { input: ANALYTICS_RUN_A_REQUEST }],
      ['analytics_lines', { input: ANALYTICS_LINES_A_BANANA_REQUEST }],
      ['analytics_print_data', { input: ANALYTICS_PRINT_DATA_A_EN_REQUEST }],
      ['analytics_views_save', { input: VIEWS_SAVE_TWO_REQUEST }],
    ]);
  });
});

describe('the fixtures are the backend’s own', () => {
  it('every analytics fixture is the same text as the backend’s file, and none is missing', () => {
    const names = (dir: string) => readdirSync(dir).filter(isAnalyticsFixture).sort();
    expect(names(FRONTEND_FIXTURES)).toEqual(names(BACKEND_FIXTURES));
    expect(names(FRONTEND_FIXTURES).length).toBeGreaterThan(40);
    for (const name of names(FRONTEND_FIXTURES)) {
      expect(readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8'), name).toBe(
        readFileSync(path.join(BACKEND_FIXTURES, name), 'utf8'),
      );
    }
  });
});

const keysOf = (value: object) => Object.keys(value).sort();

describe('the answers carry exactly the fields of the generated types', () => {
  it('AnalyticsCatalogEntry: the six views, each with the filters it takes', () => {
    expect(ANALYTICS_CATALOG.map((entry) => entry.id)).toEqual(['A', 'B', 'C', 'D', 'E', 'H']);
    for (const entry of ANALYTICS_CATALOG) {
      expect(keysOf(entry)).toEqual(['filters', 'id', 'title_id']);
      // Only view D takes a grouping.
      expect(entry.filters.includes('group_by')).toBe(entry.id === 'D');
    }
  });

  it('AnalyticsPage, its rows, columns and filters', () => {
    for (const page of [
      ANALYTICS_RUN_A,
      ANALYTICS_RUN_A_COMPARE,
      ANALYTICS_RUN_B,
      ANALYTICS_RUN_C,
      ANALYTICS_RUN_D_MONTH,
      ANALYTICS_RUN_D_DAY,
      ANALYTICS_RUN_E,
      ANALYTICS_RUN_H,
    ]) {
      expect(keysOf(page)).toEqual([
        'columns',
        'filters',
        'limit',
        'offset',
        'rows',
        'title_id',
        'total_count',
        'totals',
        'totals_label_id',
        'view',
      ]);
      expect(keysOf(page.filters)).toEqual([
        'adda',
        'buyer',
        'city',
        'compare',
        'compare_from',
        'compare_to',
        'from',
        'group_by',
        'item',
        'period',
        'seller',
        'side',
        'to',
      ]);
      for (const column of page.columns) {
        expect(keysOf(column)).toEqual(['drill', 'heading', 'key', 'kind', 'title_id']);
      }
      for (const row of page.rows) {
        expect(keysOf(row)).toEqual(['cells', 'key', 'kind']);
        // One cell for each column, in order (the screen reads cell i under column i).
        expect(row.cells.length).toBe(page.columns.length);
      }
      expect(page.totals.length).toBe(page.columns.length);
    }
  });

  it('AnalyticsKey carries the six filters and the items left out', () => {
    const key = ANALYTICS_RUN_A.rows[0]?.key;
    expect(key).not.toBeNull();
    expect(keysOf(key ?? {})).toEqual([
      'buyer_id',
      'from',
      'item_id',
      'not_item_ids',
      'person_key',
      'seller_id',
      'to',
    ]);
  });

  it('the headings of the columns of D and E say what each column stands for', () => {
    const month = ANALYTICS_RUN_D_MONTH.columns.find((column) => column.heading !== null);
    expect(month?.heading).toEqual({ kind: 'month', year: 2026, month: 10 });
    expect(month?.drill).toMatchObject({ from: '2026-10-01', to: '2026-10-31' });
    const day = ANALYTICS_RUN_D_DAY.columns.find((column) => column.heading !== null);
    expect(day?.heading).toEqual({ kind: 'day', iso: '2026-10-10' });
    const item = ANALYTICS_RUN_E.columns.find((column) => column.heading !== null);
    expect(item?.heading?.kind).toBe('item');
  });

  it('AnalyticsLinesPage', () => {
    for (const page of [
      ANALYTICS_LINES_A_BANANA,
      ANALYTICS_LINES_D_BANANA_OCTOBER,
      ANALYTICS_LINES_E_RAHIM_BANANA,
    ]) {
      expect(keysOf(page)).toEqual([
        'columns',
        'filters',
        'limit',
        'offset',
        'rows',
        'title_id',
        'total_count',
        'totals',
        'totals_label_id',
        'view',
      ]);
      for (const row of page.rows) {
        expect(keysOf(row)).toEqual(['cells', 'kind', 'source']);
        expect(row.source).toMatchObject({ kind: 'bikri' });
      }
    }
  });

  it('AnalyticsPrintData', () => {
    for (const data of [ANALYTICS_PRINT_DATA_A_EN, ANALYTICS_PRINT_DATA_D_UR]) {
      expect(keysOf(data)).toEqual([
        'columns',
        'dir',
        'filters',
        'firm',
        'footer_id',
        'language',
        'paper',
        'printed_at',
        'row_count',
        'rows',
        'title_id',
        'totals',
        'totals_label_id',
        'urdu_digits',
        'view',
      ]);
      for (const column of data.columns) {
        expect(keysOf(column)).toEqual(['heading', 'key', 'kind', 'title_id']);
      }
    }
  });

  it('AnalyticsSavedView', () => {
    for (const saved of VIEWS_GET_TWO) {
      expect(keysOf(saved)).toEqual(['filters', 'name', 'view']);
      expect(keysOf(saved.filters)).toEqual([
        'adda_id',
        'buyer',
        'city_id',
        'compare',
        'from',
        'group_by',
        'item_id',
        'period',
        'seller_id',
        'side',
        'to',
      ]);
    }
  });
});

describe('the share is sent as hundredths of a percent', () => {
  it('7210 and 2790 for the two items of E2E-11, and 10000 in the total', () => {
    const share = ANALYTICS_RUN_A.columns.findIndex((column) => column.key === 'share');
    const hundredths = (cell: { type: string; hundredths?: number } | undefined) =>
      cell?.type === 'percent' ? cell.hundredths : null;
    expect(ANALYTICS_RUN_A.rows.map((row) => hundredths(row.cells[share]))).toEqual([7210, 2790]);
    expect(hundredths(ANALYTICS_RUN_A.totals[share])).toBe(10000);
  });
});

/** Every Fluent message ID the backend names in an answer. */
function idsIn(value: unknown, found: Set<string>): void {
  if (Array.isArray(value)) {
    for (const item of value) idsIn(item, found);
    return;
  }
  if (typeof value !== 'object' || value === null) return;
  for (const [key, inner] of Object.entries(value)) {
    if (
      typeof inner === 'string' &&
      ['title_id', 'label_id', 'type_id', 'side_id', 'totals_label_id', 'footer_id'].includes(key)
    ) {
      found.add(inner);
    } else if (key === 'id' && typeof inner === 'string' && /^[a-z]+(-[a-z0-9]+)+$/.test(inner)) {
      // A `label` cell holds an ID under `id`.
      found.add(inner);
    } else {
      idsIn(inner, found);
    }
  }
}

describe('every message ID the backend sends has words in both languages', () => {
  it('titles, columns, the three lists of H and the kinds of entry', () => {
    const found = new Set<string>();
    for (const name of readdirSync(FRONTEND_FIXTURES).filter(isAnalyticsFixture)) {
      if (name.endsWith('_request.json')) continue;
      idsIn(JSON.parse(readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8')), found);
    }
    expect(found.size).toBeGreaterThan(25);
    for (const id of found) {
      for (const language of LANGUAGES) {
        expect(bundles[language].hasMessage(id), `${language} ${id}`).toBe(true);
      }
    }
  });
});
