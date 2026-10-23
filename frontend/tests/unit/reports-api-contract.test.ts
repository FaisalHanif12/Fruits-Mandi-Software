// The screens and the backend agree on plan 20 (the report engine): each wrapper in
// features/reports/api.ts sends the command name the backend registers and the input in the shape
// `{ input }` (the catalogue sends only its name). The names are also read from the backend's own
// command file, so a command renamed there fails here first. The fixtures in tests/fixtures are the
// backend's own, copied; here they must still be the same bytes as the backend's, must carry exactly
// the fields of the generated types, and every message ID the backend sends (titles, groups,
// columns, labels, kinds of entry, sides) must have words in Urdu and in English.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as api from '../../src/features/reports/api';
import { bundles, LANGUAGES } from '../../src/i18n';
import * as invoke from '../../src/shared/api/invoke';
import {
  CATALOG,
  PEOPLE_LIMIT_TOO_BIG,
  PEOPLE_NONE,
  PEOPLE_SHAHID,
  PEOPLE_SHAHID_REQUEST,
  PRINT_R13_AKRAM_UR,
  PRINT_R13_AKRAM_UR_REQUEST,
  PRINT_R18_EN,
  PRINT_R18_EN_REQUEST,
  R11_TODAY,
  R13_AKRAM,
  R13_AKRAM_REQUEST,
  R14_BALANCE,
  R14_DETAIL,
  R16_PAGE1,
  R17,
  R18_PAGE1,
  R18_PAGE1_REQUEST,
  R19,
  R2_ALL,
  R20_DUES,
  R21_BY_CITY,
  R22_BY_COLLECTOR,
  R24_PAYABLES,
  R26_ADVANCES,
  R27_SERIES,
  R28_BY_DAY,
  R28_BY_MONTH,
  R3_CASH,
  R30_PROFIT,
  R4_CREDIT,
  R6_LOTS,
  R7_ITEMS,
  R8_ADDA,
  R9_ALL,
  R9_DUE,
  PRINT_R20_UR,
  PRINT_R28_MONTH_EN,
  PRINT_R7_EN,
  PRINT_R9_DUE_EN,
} from '../helpers/reports-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const BACKEND_FILE = path.resolve(__dirname, '../../../backend/app/src/commands/reports_engine.rs');
const BACKEND_PEOPLE_FILE = path.resolve(
  __dirname,
  '../../../backend/app/src/commands/reports_people.rs',
);
const BACKEND_FIXTURES = path.resolve(__dirname, '../../../backend/crates/services/tests/fixtures');
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

/** The fixtures of the report engine (the voucher slips of plan 15 have the same file prefix). */
const isEngineFixture = (name: string) =>
  name.endsWith('.json') &&
  (name === 'reports_catalog.json' ||
    name.startsWith('reports_run_') ||
    name.startsWith('reports_find_people_') ||
    name.startsWith('reports_print_data_r'));

describe('the commands the screens call are the ones the backend declares', () => {
  it('every report engine command used in api.ts exists in the backend file', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    const declared = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    for (const name of ['reports_catalog', 'reports_run', 'reports_print_data']) {
      expect(declared).toContain(name);
    }
  });

  it('the people search used in api.ts exists in the backend’s people file', () => {
    const text = readFileSync(BACKEND_PEOPLE_FILE, 'utf8');
    const declared = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    expect(declared).toContain('reports_find_people');
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  it('the catalogue sends only its name', async () => {
    await api.reportsCatalog();
    expect(call.mock.calls).toEqual([['reports_catalog']]);
  });

  it('the others send the exact request the backend wrote', async () => {
    await api.reportsRun(R18_PAGE1_REQUEST);
    await api.reportsRun(R13_AKRAM_REQUEST);
    await api.reportsPrintData(PRINT_R18_EN_REQUEST);
    await api.reportsPrintData(PRINT_R13_AKRAM_UR_REQUEST);
    await api.reportsFindPeople(PEOPLE_SHAHID_REQUEST);
    expect(call.mock.calls).toEqual([
      ['reports_run', { input: R18_PAGE1_REQUEST }],
      ['reports_run', { input: R13_AKRAM_REQUEST }],
      ['reports_print_data', { input: PRINT_R18_EN_REQUEST }],
      ['reports_print_data', { input: PRINT_R13_AKRAM_UR_REQUEST }],
      ['reports_find_people', { input: PEOPLE_SHAHID_REQUEST }],
    ]);
  });
});

describe('the fixtures are the backend’s own', () => {
  it('every report engine fixture is the same text as the backend’s file', () => {
    // The catalogue too: it holds all 22 reports of the early track now, as the backend lists them.
    const names = readdirSync(FRONTEND_FIXTURES).filter(isEngineFixture);
    expect(names.length).toBeGreaterThan(90);
    for (const name of names) {
      expect(readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8'), name).toBe(
        readFileSync(path.join(BACKEND_FIXTURES, name), 'utf8'),
      );
    }
  });

  it('has every report the backend’s fixture folder has, none more, none less', () => {
    const names = (dir: string) => readdirSync(dir).filter(isEngineFixture).sort();
    expect(names(FRONTEND_FIXTURES)).toEqual(names(BACKEND_FIXTURES));
  });

  it('every report of the copied catalogue is, unchanged, in the backend’s catalogue', () => {
    const backend = JSON.parse(
      readFileSync(path.join(BACKEND_FIXTURES, 'reports_catalog.json'), 'utf8'),
    ) as { id: string }[];
    for (const entry of CATALOG) {
      expect(
        backend.find((candidate) => candidate.id === entry.id),
        entry.id,
      ).toEqual(entry);
    }
  });
});

const keysOf = (value: object) => Object.keys(value).sort();

describe('the answers carry exactly the fields of the generated types', () => {
  it('ReportCatalogEntry', () => {
    for (const entry of CATALOG) {
      expect(keysOf(entry)).toEqual([
        'filters',
        'group',
        'group_title_id',
        'id',
        'party_required',
        'title_id',
      ]);
    }
  });

  it('ReportPage, its rows, columns and filters', () => {
    for (const page of [
      R11_TODAY,
      R13_AKRAM,
      R14_BALANCE,
      R14_DETAIL,
      R16_PAGE1,
      R17,
      R18_PAGE1,
      R19,
      R2_ALL,
      R3_CASH,
      R4_CREDIT,
      R6_LOTS,
      R7_ITEMS,
      R8_ADDA,
      R9_ALL,
      R9_DUE,
      R20_DUES,
      R21_BY_CITY,
      R22_BY_COLLECTOR,
      R24_PAYABLES,
      R26_ADVANCES,
      R27_SERIES,
      R28_BY_DAY,
      R28_BY_MONTH,
      R30_PROFIT,
    ]) {
      expect(keysOf(page)).toEqual([
        'columns',
        'filters',
        'id',
        'limit',
        'offset',
        'rows',
        'summary',
        'title_id',
        'total_count',
        'totals',
        'totals_label_id',
      ]);
      // `group_by` and `status` are in the echo only when the report was asked with them.
      expect(keysOf(page.filters).filter((key) => key !== 'group_by' && key !== 'status')).toEqual([
        'adda',
        'as_of',
        'city',
        'detail',
        'from',
        'item',
        'party',
        'seller',
        'shortcut',
        'to',
        'zamindar',
      ]);
      for (const column of page.columns)
        expect(keysOf(column)).toEqual(['key', 'kind', 'title_id']);
      for (const row of page.rows) {
        expect(keysOf(row)).toEqual(['cells', 'kind', 'source']);
        // One cell for each column, in order (the screen reads cell i under column i).
        expect(row.cells.length).toBe(page.columns.length);
      }
      expect(page.totals.length).toBe(page.columns.length);
    }
  });

  it('ReportParty', () => {
    const cell = R13_AKRAM.filters.party;
    expect(cell).not.toBeNull();
    expect(keysOf(cell ?? {})).toEqual([
      'account_id',
      'code',
      'name_en',
      'name_ur',
      'person_key',
      'short_code',
      'tag',
    ]);
  });

  it('PukhtaPersonHit', () => {
    for (const hit of PEOPLE_SHAHID) {
      expect(keysOf(hit)).toEqual([
        'accounts',
        'last_date',
        'name_en',
        'name_ur',
        'person_key',
        'phone',
      ]);
      for (const account of hit.accounts) expect(keysOf(account)).toEqual(['account_id', 'side']);
    }
    expect(PEOPLE_NONE).toEqual([]);
    expect(PEOPLE_LIMIT_TOO_BIG).toMatchObject({ ok: false });
  });

  it('ReportPrintData', () => {
    for (const data of [
      PRINT_R18_EN,
      PRINT_R13_AKRAM_UR,
      PRINT_R7_EN,
      PRINT_R20_UR,
      PRINT_R28_MONTH_EN,
      PRINT_R9_DUE_EN,
    ]) {
      expect(keysOf(data)).toEqual([
        'columns',
        'dir',
        'filters',
        'firm',
        'footer_id',
        'language',
        'paper',
        'printed_at',
        'report',
        'row_count',
        'rows',
        'summary',
        'title_id',
        'totals',
        'totals_label_id',
        'urdu_digits',
        'words',
      ]);
      expect(keysOf(data.firm)).toEqual(['address', 'mandi', 'name', 'phone']);
    }
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
      [
        'title_id',
        'label_id',
        'type_id',
        'side_id',
        'group_title_id',
        'totals_label_id',
        'footer_id',
      ].includes(key)
    ) {
      found.add(inner);
    } else if (key === 'id' && typeof inner === 'string' && /^[a-z]+(-[a-z0-9]+)+$/.test(inner)) {
      // A `label` cell holds an ID under `id`; a report's own id (R18) is not one.
      found.add(inner);
    } else {
      idsIn(inner, found);
    }
  }
}

describe('every message ID the backend sends has words in both languages', () => {
  it('titles, groups, columns, labels, kinds of entry and sides', () => {
    const found = new Set<string>();
    for (const name of readdirSync(FRONTEND_FIXTURES).filter(isEngineFixture)) {
      if (name.endsWith('_request.json')) continue;
      idsIn(JSON.parse(readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8')), found);
    }
    expect(found.size).toBeGreaterThan(100);
    for (const id of found) {
      for (const language of LANGUAGES) {
        expect(bundles[language].hasMessage(id), `${language} ${id}`).toBe(true);
      }
    }
  });

  it('has a kind-of-entry message for each posting type the contract names', () => {
    const types = [
      'bikri-first',
      'bikri-later',
      'reversal',
      'receipt',
      'payment',
      'advance',
      'shop-expense',
      'personal-expense',
      'personal-return',
      'money-lent',
      'money-lent-return',
      'capital-in',
      'capital-out',
      'khata-out',
      'khata-in',
      'purchase',
      'trip-cost',
      'garden-payment',
      'venture-receipt',
      'garden-close',
      'fruit-taken',
      'general',
      'opening',
      'month-close',
      'other',
    ];
    for (const type of types) {
      for (const language of LANGUAGES) {
        expect(bundles[language].hasMessage(`report-doc-${type}`), `${language} ${type}`).toBe(
          true,
        );
      }
    }
  });
});
