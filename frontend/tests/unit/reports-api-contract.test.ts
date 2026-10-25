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
  PRINT_R5_EN,
  PRINT_R5_EN_REQUEST,
  R1_BIKRI_1,
  R1_BIKRI_1_REQUEST,
  R1_NEEDS_A_BIKRI,
  R1_NEEDS_A_BIKRI_REQUEST,
  R5_CREDIT_BUYERS,
  R5_CREDIT_BUYERS_REQUEST,
} from '../helpers/reports-sales-real';
import {
  PRINT_R36_PARTNER_EN,
  PRINT_R36_PARTNER_EN_REQUEST,
  R34_GARDEN_DAY,
  R36_GARDEN,
  R36_GARDEN_REQUEST,
  R36_PARTNER,
  R36_PARTNER_REQUEST,
} from '../helpers/reports-gardens-real';
import {
  PRINT_R35_SLIP_AKRAM_EN,
  PRINT_R35_SLIP_AKRAM_EN_REQUEST,
  R35_SLIP_AKRAM,
  R35_SLIP_AKRAM_REQUEST,
  R35_SLIP_RAHIM,
  R35_SLIP_RAHIM_REQUEST,
  R35_TABLE,
  R35_TABLE_REQUEST,
} from '../helpers/reports-party-real';
import {
  BACKEND_CATALOG,
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
  PENDING_REPORTS,
} from '../helpers/reports-real';
import {
  PRINT_R33_PARTNER_EN,
  PRINT_R33_PARTNER_EN_REQUEST,
  PRINT_R34_EN,
  PRINT_R34_EN_REQUEST,
  R10_STOCK,
  R10_STOCK_REQUEST,
  R15_ALL,
  R15_ALL_REQUEST,
  R15_PAYMENTS,
  R15_PAYMENTS_REQUEST,
  R23_DAY,
  R23_DAY_REQUEST,
  R25_DAY,
  R25_DAY_REQUEST,
  R33_ITEM,
  R33_ITEM_REQUEST,
  R33_MONTH,
  R33_MONTH_REQUEST,
  R33_PARTNER,
  R33_PARTNER_REQUEST,
  R33_TRIP,
  R33_TRIP_REQUEST,
  R34_DAY,
  R34_DAY_REQUEST,
} from '../helpers/reports-stock-real';
import {
  PRINT_R31_BAY_EN,
  PRINT_R31_BAY_EN_REQUEST,
  R12_DAY,
  R12_DAY_REQUEST,
  R29_ALIF,
  R29_ALIF_REQUEST,
  R29_DAY,
  R29_DAY_REQUEST,
  R31_BAY,
  R31_BAY_REQUEST,
  R31_PARTNER_REQUIRED,
  R31_PARTNER_REQUIRED_REQUEST,
  R32_E2E_30B,
  R32_E2E_30B_REQUEST,
} from '../helpers/reports-business-real';

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
    // The catalogue too: it holds all 36 reports of the early track now, as the backend lists them.
    const names = readdirSync(FRONTEND_FIXTURES).filter(isEngineFixture);
    expect(names.length).toBeGreaterThan(100);
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

  it('the backend’s catalogue is the reports of this build and those waiting for their round, no other', () => {
    const backend = BACKEND_CATALOG.map((entry) => entry.id).sort();
    const ours = [...CATALOG.map((entry) => entry.id), ...PENDING_REPORTS].sort();
    expect(backend).toEqual(ours);
    // The ids waiting are still the backend's, so the list does not hold an id it never sent.
    for (const id of PENDING_REPORTS) {
      expect(
        BACKEND_CATALOG.some((entry) => entry.id === id),
        id,
      ).toBe(true);
    }
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
      R12_DAY,
      R29_ALIF,
      R29_DAY,
      R31_BAY,
      R32_E2E_30B,
      R10_STOCK,
      R15_ALL,
      R15_PAYMENTS,
      R23_DAY,
      R25_DAY,
      R33_TRIP,
      R33_PARTNER,
      R33_ITEM,
      R33_MONTH,
      R34_DAY,
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
      // `group_by`, `status`, `voucher_type` and `trip_view` are in the echo only when the report
      // was asked with them.
      const optionalKeys = new Set(['group_by', 'status', 'voucher_type', 'trip_view']);
      expect(keysOf(page.filters).filter((key) => !optionalKeys.has(key))).toEqual([
        'adda',
        'as_of',
        'city',
        'detail',
        'from',
        'item',
        'partner',
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
      PRINT_R31_BAY_EN,
      PRINT_R33_PARTNER_EN,
      PRINT_R34_EN,
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

describe('round 7: the cash book, the expense report, the partner statement and money lent', () => {
  const entryOf = (id: string) => CATALOG.find((entry) => entry.id === id);

  it('the catalogue lists 36 reports, with the boxes the backend says for the newer ones', () => {
    expect(CATALOG).toHaveLength(36);
    expect(entryOf('R12')).toMatchObject({ filters: ['date_range'], party_required: false });
    expect(entryOf('R29')).toMatchObject({
      filters: ['date_range', 'partner'],
      party_required: false,
    });
    // R31 cannot run without its partner: the catalogue says so with `party_required`.
    expect(entryOf('R31')).toMatchObject({
      filters: ['date_range', 'partner'],
      party_required: true,
    });
    expect(entryOf('R32')).toMatchObject({ filters: ['as_of', 'party'], party_required: false });
  });

  it('sends the exact requests the backend wrote for the four reports', async () => {
    const requests = [
      R12_DAY_REQUEST,
      R29_ALIF_REQUEST,
      R29_DAY_REQUEST,
      R31_BAY_REQUEST,
      R31_PARTNER_REQUIRED_REQUEST,
      R32_E2E_30B_REQUEST,
    ];
    for (const request of requests) await api.reportsRun(request);
    await api.reportsPrintData(PRINT_R31_BAY_EN_REQUEST);
    expect(call.mock.calls).toEqual([
      ...requests.map((request) => ['reports_run', { input: request }]),
      ['reports_print_data', { input: PRINT_R31_BAY_EN_REQUEST }],
    ]);
  });

  it('refuses R31 with no partner under the partner box', () => {
    expect(R31_PARTNER_REQUIRED).toMatchObject({
      ok: false,
      errors: [{ code: 'ERR_REQUIRED', field: 'filters.partner_id' }],
    });
    expect(R31_PARTNER_REQUIRED_REQUEST.filters.partner_id).toBeNull();
  });

  it('R31 lines are message cells, in the screen answer and in the print', () => {
    const kinds = R31_BAY.columns.map((column) => column.kind);
    expect(kinds).toEqual(['date', 'label', 'message', 'money', 'money']);
    const screen = R31_BAY.rows.map((row) => row.cells[2]?.type);
    expect(screen.every((type) => type === 'message')).toBe(true);
    const printed = PRINT_R31_BAY_EN.rows.map((row) => row.cells[2]?.type);
    expect(printed.every((type) => type === 'message')).toBe(true);
  });
});

describe('round 8: stock, vouchers, registers, trip profit or loss and the day summary', () => {
  const entryOf = (id: string) => CATALOG.find((entry) => entry.id === id);

  it('the catalogue lists the six new reports with the boxes the backend says', () => {
    expect(entryOf('R10')).toMatchObject({
      group: 'sales',
      filters: ['date_range', 'seller', 'item'],
    });
    expect(entryOf('R15')).toMatchObject({
      group: 'accounts',
      filters: ['date_range', 'party', 'voucher_type'],
    });
    expect(entryOf('R23')).toMatchObject({
      group: 'collections',
      filters: ['date_range', 'party'],
    });
    expect(entryOf('R25')).toMatchObject({
      group: 'collections',
      filters: ['date_range', 'party'],
    });
    expect(entryOf('R33')).toMatchObject({
      group: 'business',
      filters: ['date_range', 'item', 'trip_view'],
    });
    expect(entryOf('R34')).toMatchObject({ group: 'business', filters: ['date_range'] });
    // None of them needs a party or a partner before it can run.
    for (const id of ['R10', 'R15', 'R23', 'R25', 'R33', 'R34']) {
      expect(entryOf(id)?.party_required, id).toBe(false);
    }
  });

  it('sends the exact requests the backend wrote for the six reports', async () => {
    const requests = [
      R10_STOCK_REQUEST,
      R15_ALL_REQUEST,
      R15_PAYMENTS_REQUEST,
      R23_DAY_REQUEST,
      R25_DAY_REQUEST,
      R33_TRIP_REQUEST,
      R33_PARTNER_REQUEST,
      R33_ITEM_REQUEST,
      R33_MONTH_REQUEST,
      R34_DAY_REQUEST,
    ];
    for (const request of requests) await api.reportsRun(request);
    await api.reportsPrintData(PRINT_R33_PARTNER_EN_REQUEST);
    await api.reportsPrintData(PRINT_R34_EN_REQUEST);
    expect(call.mock.calls).toEqual([
      ...requests.map((request) => ['reports_run', { input: request }]),
      ['reports_print_data', { input: PRINT_R33_PARTNER_EN_REQUEST }],
      ['reports_print_data', { input: PRINT_R34_EN_REQUEST }],
    ]);
  });

  it('R15 with no kind has empty totals and one summary line per kind; with a kind, one total', () => {
    expect(R15_ALL.totals.every((cell) => cell.type === 'empty')).toBe(true);
    expect(R15_ALL.summary.map((line) => line.label_id)).toEqual([
      'report-doc-advance',
      'report-doc-payment',
      'report-doc-receipt',
    ]);
    expect(R15_PAYMENTS.summary).toEqual([]);
    expect(R15_PAYMENTS.totals.some((cell) => cell.type === 'money')).toBe(true);
  });

  it('R33 trip and partner layouts have group rows for the trips, and every layout ends in one result', () => {
    for (const page of [R33_TRIP, R33_PARTNER]) {
      expect(page.rows.some((row) => row.kind === 'group')).toBe(true);
    }
    expect(R33_TRIP.columns).toHaveLength(15);
    expect(R33_PARTNER.columns.map((column) => column.key)).toEqual([
      'trip',
      'partner',
      'share',
      'money_in_shop',
      'money_in_pocket',
      'money_back',
      'net',
      'cost_left',
    ]);
    // The share is a percent cell, in hundredths of a percent (4000 is 40%).
    expect(R33_PARTNER.rows[1]?.cells[2]).toEqual({ type: 'percent', hundredths: 4000 });
    for (const page of [R33_ITEM, R33_MONTH]) expect(page.columns).toHaveLength(12);
  });

  it('R34 is a flat list of date, line, count and amount with no totals', () => {
    expect(R34_DAY.columns.map((column) => column.key)).toEqual([
      'date',
      'item',
      'count',
      'amount',
    ]);
    expect(R34_DAY.totals.every((cell) => cell.type === 'empty')).toBe(true);
    // The heading row of the partners' shares comes right before the partner rows.
    const labels = R34_DAY.rows.map((row) => {
      const cell = row.cells[1];
      return cell?.type === 'label' ? cell.id : cell?.type;
    });
    const at = labels.indexOf('report-r34-partner-shares');
    expect(at).toBeGreaterThan(0);
    expect(labels.slice(at + 1, at + 4)).toEqual(['names', 'names', 'names']);
  });

  it('every message of the six reports can be written in both languages with no hole', () => {
    const found = new Set<string>(['report-result-profit', 'report-result-loss']);
    for (const page of [
      R10_STOCK,
      R15_ALL,
      R15_PAYMENTS,
      R23_DAY,
      R25_DAY,
      R33_TRIP,
      R33_PARTNER,
      R33_ITEM,
      R33_MONTH,
      R34_DAY,
    ]) {
      idsIn(page, found);
      found.add(page.title_id.replace(/-title$/, '-help'));
    }
    for (const id of found) {
      for (const language of LANGUAGES) {
        const pattern = bundles[language].getMessage(id)?.value;
        expect(pattern, `${language} ${id}`).toBeTruthy();
        const errors: Error[] = [];
        if (pattern !== null && pattern !== undefined) {
          const text = bundles[language].formatPattern(pattern, {}, errors);
          expect(errors, `${language} ${id}`).toEqual([]);
          expect(text.trim(), `${language} ${id}`).not.toBe('');
        }
      }
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

describe('round 11: the party day sheet (R35)', () => {
  const entry = CATALOG.find((candidate) => candidate.id === 'R35');
  const lines = (page: { summary: readonly { label_id: string }[] }) =>
    page.summary.map((line) => line.label_id);

  it('is in the business group with a day box and a party box, and runs with no party', () => {
    expect(entry).toMatchObject({
      group: 'business',
      title_id: 'report-r35-title',
      filters: ['as_of', 'party'],
      party_required: false,
    });
  });

  it('sends the exact requests the backend wrote: the table, two slips and the print', async () => {
    const requests = [R35_TABLE_REQUEST, R35_SLIP_AKRAM_REQUEST, R35_SLIP_RAHIM_REQUEST];
    for (const request of requests) await api.reportsRun(request);
    await api.reportsPrintData(PRINT_R35_SLIP_AKRAM_EN_REQUEST);
    expect(call.mock.calls).toEqual([
      ...requests.map((request) => ['reports_run', { input: request }]),
      ['reports_print_data', { input: PRINT_R35_SLIP_AKRAM_EN_REQUEST }],
    ]);
    // The table is for no party; a slip is for one account, with no person for an ordinary one.
    expect(R35_TABLE_REQUEST.filters.party).toBeNull();
    expect(R35_SLIP_AKRAM_REQUEST.filters.party).toEqual({ account_id: 20, person_key: null });
  });

  it('the table: a group row for the buyers and one for the sellers, with a blank collector column', () => {
    expect(R35_TABLE.columns.map((column) => column.key)).toEqual([
      'code',
      'party',
      'previous',
      'debit',
      'credit',
      'remaining',
      'collector',
    ]);
    const groups = R35_TABLE.rows.filter((row) => row.kind === 'group');
    expect(groups.map((row) => row.cells[0])).toEqual([
      { type: 'label', id: 'report-r35-buyers' },
      { type: 'label', id: 'report-r35-sellers' },
    ]);
    // The collector writes in the last column by hand: the backend leaves it empty on every row.
    for (const row of R35_TABLE.rows) expect(row.cells[6]).toEqual({ type: 'empty' });
    // A one-time buyer comes with his person key, so his slip is his own.
    const people = R35_TABLE.rows.flatMap((row) =>
      row.cells.flatMap((cell) => (cell.type === 'party' ? [cell.party] : [])),
    );
    expect(people.find((party) => party.code === '03049999')?.person_key).toBe('شاہد|03000000000');
  });

  it('the table has no totals row of its own: the totals are in the group rows and the footer lines', () => {
    // The buyers' total is the heading row of their section and the line "collection" (equal to R20),
    // the sellers' are three footer lines, so a pinned row under both would be read as the total of all.
    expect(R35_TABLE.totals).toHaveLength(R35_TABLE.columns.length);
    for (const total of R35_TABLE.totals) expect(total).toEqual({ type: 'empty' });
    // A slip has its own total: the quantity and the amount of its lines.
    expect(R35_SLIP_AKRAM.totals[0]).toEqual({ type: 'number', value: 20 });
  });

  it('the footer lines come in the order the screen and the print show them', () => {
    expect(lines(R35_TABLE)).toEqual([
      'report-r35-collection',
      'report-r35-sellers-credit',
      'report-r35-sellers-debit',
      'report-r35-sellers-remaining',
    ]);
    // A buyer's slip says "received" (the client's وصولی), a seller's says "paid" (ادائیگی).
    expect(lines(R35_SLIP_AKRAM)).toEqual([
      'report-r35-total-qty',
      'report-r35-previous',
      'report-r35-bought',
      'report-r35-received',
      'report-r35-remaining',
    ]);
    expect(lines(R35_SLIP_RAHIM)).toEqual([
      'report-r35-total-qty',
      'report-r35-previous',
      'report-r35-sold',
      'report-r35-cash-given',
      'report-r35-paid',
      'report-r35-remaining',
    ]);
    expect(PRINT_R35_SLIP_AKRAM_EN.summary.map((line) => line.label_id)).toEqual(
      lines(R35_SLIP_AKRAM),
    );
  });

  it('a slip has the quantity, the item, the rate and the amount of each line, and each opens its بکری', () => {
    for (const slip of [R35_SLIP_AKRAM, R35_SLIP_RAHIM]) {
      expect(slip.columns.map((column) => column.key)).toEqual(['qty', 'item', 'rate', 'amount']);
      expect(slip.rows[0]?.source).toMatchObject({ kind: 'bikri' });
    }
  });
});

describe('the garden slice: R36, the gardens report, and the garden rows of R34', () => {
  const entry = CATALOG.find((candidate) => candidate.id === 'R36');
  const keys = (page: { columns: { key: string }[] }) => page.columns.map((column) => column.key);

  it('is in the business group with the boxes the backend says, and runs with nothing picked', () => {
    expect(entry).toMatchObject({
      group: 'business',
      title_id: 'report-r36-title',
      filters: ['date_range', 'item', 'partner', 'garden', 'garden_view'],
      party_required: false,
    });
  });

  it('sends the exact requests the backend wrote: by garden, by partner and the print', async () => {
    const requests = [R36_GARDEN_REQUEST, R36_PARTNER_REQUEST];
    for (const request of requests) await api.reportsRun(request);
    await api.reportsPrintData(PRINT_R36_PARTNER_EN_REQUEST);
    expect(call.mock.calls).toEqual([
      ...requests.map((request) => ['reports_run', { input: request }]),
      ['reports_print_data', { input: PRINT_R36_PARTNER_EN_REQUEST }],
    ]);
    // By garden is the default and is not sent; by partner is.
    expect(R36_GARDEN_REQUEST.filters.garden_view).toBeUndefined();
    expect(R36_PARTNER_REQUEST.filters.garden_view).toBe('partner');
  });

  it('by garden has the 15 columns of the PRD and a row that opens the garden', () => {
    expect(keys(R36_GARDEN)).toEqual([
      'garden',
      'date',
      'status',
      'rent',
      'installments',
      'costs',
      'paid_shop',
      'paid_pocket',
      'sales_our_shop',
      'sales_other_shops',
      'other_in',
      'commission',
      'money_in',
      'balance',
      'rent_due',
    ]);
    expect(R36_GARDEN.rows[0]?.source).toEqual({ kind: 'venture', id: 1 });
    expect(R36_GARDEN.rows[0]?.cells[2]).toEqual({ type: 'label', id: 'report-garden-closed' });
    // The totals row is the backend's: the sums of the rows, nothing under the name and the state.
    expect(R36_GARDEN.totals.slice(0, 3).every((cell) => cell.type === 'empty')).toBe(true);
  });

  it('by partner has 7 columns, each partner’s percent and money, and the same totals logic', () => {
    expect(keys(R36_PARTNER)).toEqual([
      'garden',
      'partner',
      'share',
      'paid_shop',
      'paid_pocket',
      'money_back',
      'result',
    ]);
    expect(R36_PARTNER.rows).toHaveLength(2);
    expect(R36_PARTNER.rows.map((row) => row.cells[2])).toEqual([
      { type: 'percent', hundredths: 6000 },
      { type: 'percent', hundredths: 4000 },
    ]);
    expect(PRINT_R36_PARTNER_EN.report).toBe('R36');
  });

  it('R34 has two rows more for each day: what was paid into the gardens and what came in', () => {
    const labels = R34_GARDEN_DAY.rows.flatMap((row) =>
      row.cells[1]?.type === 'label' ? [row.cells[1].id] : [],
    );
    expect(labels).toContain('report-r34-gardens-paid');
    expect(labels).toContain('report-r34-gardens-in');
    // Information only: they come after the trips rows and before the partners' shares.
    expect(labels.indexOf('report-r34-gardens-paid')).toBe(
      labels.indexOf('report-r34-trips-sold') + 1,
    );
    expect(labels.indexOf('report-r34-gardens-in')).toBe(
      labels.indexOf('report-r34-gardens-paid') + 1,
    );
  });
});

describe('round 12: R1, the sale book, and R5, the credit buyers’ details', () => {
  const entryOf = (id: string) => CATALOG.find((candidate) => candidate.id === id);
  const keys = (page: { columns: { key: string }[] }) => page.columns.map((column) => column.key);

  it('R1 takes one box, the بکری, which it cannot run without; R5 takes the boxes of R4', () => {
    expect(entryOf('R1')).toMatchObject({
      group: 'sales',
      title_id: 'report-r1-title',
      filters: ['bikri'],
      party_required: true,
    });
    expect(entryOf('R5')).toMatchObject({
      group: 'sales',
      title_id: 'report-r5-title',
      filters: ['date_range', 'party', 'seller', 'zamindar', 'item', 'city', 'adda'],
      party_required: false,
    });
    // The two reports of the sales group that were missing now come before and after R2 to R4.
    expect(CATALOG.filter((candidate) => candidate.group === 'sales').map((c) => c.id)).toEqual(
      expect.arrayContaining(['R1', 'R2', 'R3', 'R4', 'R5', 'R6']),
    );
  });

  it('sends the exact requests the backend wrote: the sale book, its refusal, the credit buyers and the print', async () => {
    const requests = [R1_BIKRI_1_REQUEST, R1_NEEDS_A_BIKRI_REQUEST, R5_CREDIT_BUYERS_REQUEST];
    for (const request of requests) await api.reportsRun(request);
    await api.reportsPrintData(PRINT_R5_EN_REQUEST);
    expect(call.mock.calls).toEqual([
      ...requests.map((request) => ['reports_run', { input: request }]),
      ['reports_print_data', { input: PRINT_R5_EN_REQUEST }],
    ]);
    expect(R1_BIKRI_1_REQUEST.filters.bikri_id).toBe(1);
    expect(R1_NEEDS_A_BIKRI_REQUEST.filters.bikri_id).toBeNull();
  });

  it('with no بکری R1 is refused with ERR_REQUIRED on its own box', () => {
    expect(R1_NEEDS_A_BIKRI).toMatchObject({
      ok: false,
      errors: [{ code: 'ERR_REQUIRED', field: 'filters.bikri_id' }],
    });
  });

  it('R1 is the lots of the بکری with the columns of the seller bill, and its filters name the بکری', () => {
    expect(keys(R1_BIKRI_1)).toEqual([
      'date',
      'doc',
      'seller',
      'item',
      'qty',
      'gross',
      'freight',
      'commission',
      'labour',
      'munshiana',
      'cash_paid',
      'store_rent',
      'phone_charge',
      'deductions',
      'net',
    ]);
    expect(R1_BIKRI_1.filters.bikri_number).toBe(1);
    expect(R1_BIKRI_1.rows[0]?.source).toEqual({ kind: 'bikri', id: 1 });
  });

  it('R5 has a serial, the buyer, his sale lines, the value and the total, and the totals of the whole filter', () => {
    expect(keys(R5_CREDIT_BUYERS)).toEqual(['serial', 'buyer', 'detail', 'value', 'buyer_total']);
    expect(R5_CREDIT_BUYERS.rows.map((row) => row.cells[0])).toEqual([
      { type: 'number', value: 1 },
      { type: 'number', value: 2 },
    ]);
    for (const row of R5_CREDIT_BUYERS.rows) {
      const detail = row.cells[2];
      expect(detail?.type).toBe('detail');
      if (detail?.type === 'detail') {
        expect(detail.parts.every((part) => part.kind === 'sale')).toBe(true);
      }
    }
    expect(PRINT_R5_EN.report).toBe('R5');
  });
});

describe('every message ID the backend sends has words in both languages', () => {
  it('titles, groups, columns, labels, kinds of entry and sides', () => {
    const found = new Set<string>();
    // A report waiting for its round has no words yet: its fixtures (named `..._r35_...`) and its
    // entry in the catalogue are left out until it lands (PENDING_REPORTS, empty now).
    const waiting =
      PENDING_REPORTS.size === 0
        ? null
        : new RegExp(`_(${[...PENDING_REPORTS].map((id) => id.toLowerCase()).join('|')})_`);
    for (const name of readdirSync(FRONTEND_FIXTURES).filter(isEngineFixture)) {
      if (name.endsWith('_request.json') || name === 'reports_catalog.json') continue;
      if (waiting?.test(name) === true) continue;
      idsIn(JSON.parse(readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8')), found);
    }
    idsIn(CATALOG, found);
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
