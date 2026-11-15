// Test helper: the اگراہی sheet of 10-10-26 after Events 1 to 5 (PRD R5, plan 76 point 2) as the backend
// answers it, and the fakes of its two commands. The page test and the screens that visit every report
// use it. The sums are a test double's: the screen shows what comes back and adds nothing up
// (CLAUDE.md rule 1).

import { vi } from 'vitest';
import type { CollectionSheet } from '../../src/bindings/CollectionSheet';
import type { CollectionSheetRow } from '../../src/bindings/CollectionSheetRow';
import type { Paper } from '../../src/bindings/Paper';
import type { TablePrintData } from '../../src/bindings/TablePrintData';
import * as reportsApi from '../../src/features/reports/api';
import { paisaForTest } from './paisa';
import { fakeDaySlips } from './day-slips';
import { printFrameFor } from './print-frame-real';

export const rupees = (value: number) => paisaForTest(value * 100);
export const dr = (value: number) =>
  ({ amount_paisa: rupees(value), side: value === 0 ? 'zero' : 'dr' }) as const;

export function buyer(
  id: number,
  code: string,
  ur: string,
  en: string,
  bill: number,
  paid: number,
  previous: number,
  options: { dead?: boolean; personKey?: string } = {},
): CollectionSheetRow {
  return {
    party: {
      account_id: id,
      code,
      short_code: code.slice(3),
      name_ur: ur,
      name_en: en,
      tag: null,
      person_key: options.personKey ?? null,
    },
    lines:
      bill === 0
        ? []
        : [
            {
              item: { ur: 'کیلا', en: 'Banana' },
              packing: { ur: 'پیٹی', en: 'Box' },
              qty: 20,
              weight_grams: null,
              rate_paisa: rupees(600),
            },
          ],
    previous_due: dr(previous),
    bill_paisa: rupees(bill),
    paid_paisa: rupees(paid),
    total_due: dr(previous + bill - paid),
    is_dead: options.dead ?? false,
  };
}

/** After Events 1 to 5 on 10-10-26 (PRD R5): اکرم 0, 12,880, 0, 12,880; ارشد 0, 18,684, 0, 18,684; شاہد (چکتا) paid in full. */
export const AKRAM = buyer(20, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', 12_880, 0, 0);
export const ARSHAD = buyer(21, '0021', 'ارشد ملتانی', 'Arshad Multani', 18_684, 0, 0);
export const SHAHID = buyer(30, '0030', 'شاہد', 'Shahid', 12_800, 12_800, 0, { personKey: 'p-1' });
export const SHEET: CollectionSheet = {
  date: '2026-10-10',
  rows: [AKRAM, ARSHAD, SHAHID],
  total_count: 3,
  totals: {
    previous_due: dr(0),
    bill_paisa: rupees(44_364),
    paid_paisa: rupees(12_800),
    total_due: dr(31_564),
  },
  dead_totals: null,
  credit_totals: null,
};

const PRINT_ROW: TablePrintData['rows'][number] = {
  kind: 'normal',
  cells: [
    { type: 'text', text: '0020 Akram Toll Tax' },
    { type: 'money', text: '12,880' },
  ],
};

export function printData(language: 'ur' | 'en', paper: Paper = 'a4'): TablePrintData {
  return {
    frame: printFrameFor({ language, title_id: 'report-r5-title', facts: [] }),
    paper,
    columns: [
      { key: 'party', title_id: 'collection-col-party', kind: 'text' },
      { key: 'bill', title_id: 'collection-col-bill', kind: 'money' },
    ],
    column_labels: [null, null],
    rows: [PRINT_ROW],
    row_count: '1',
    totals_label_id: 'glossary-meezan',
    totals: [{ type: 'empty' }, { type: 'money', text: '44,364' }],
    words: null,
    extra_totals: [],
  };
}

/** Makes the sheet and its print data answer; the search narrows the buyers by name or code. */
export function fakeCollectionSheet(): void {
  fakeDaySlips();
  vi.mocked(reportsApi.reportsCollectionSheet).mockImplementation((input) => {
    const search = (input.search ?? '').toLowerCase();
    const rows = SHEET.rows.filter(
      (row) =>
        search === '' ||
        row.party.name_en.toLowerCase().includes(search) ||
        row.party.code.includes(search),
    );
    return Promise.resolve({
      ...SHEET,
      date: input.date === '' ? SHEET.date : input.date.split('-').reverse().join('-'),
      rows,
      total_count: rows.length,
    });
  });
  vi.mocked(reportsApi.reportsCollectionSheetPrintData).mockImplementation((input) =>
    Promise.resolve(printData(input.language, input.paper ?? 'a4')),
  );
}
