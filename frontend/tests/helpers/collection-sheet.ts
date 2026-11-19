// Test helper: the اگراہی sheet of 10-10-26 after Events 1 to 5 (PRD R5, plan 76 point 2) as the backend
// answers it, and the fakes of its two commands. The page test and the screens that visit every report
// use it. The sums are a test double's: the screen shows what comes back and adds nothing up
// (CLAUDE.md rule 1).

import { vi } from 'vitest';
import type { AdhariSheet } from '../../src/bindings/AdhariSheet';
import type { AdhariSheetRow } from '../../src/bindings/AdhariSheetRow';
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
      { key: 'party', title_id: 'print-adhari-sheet-col-party', kind: 'text' },
      { key: 'bill', title_id: 'print-adhari-sheet-col-bill', kind: 'money' },
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

/** One buyer of ادھاری تفصیل (PRD R5 v1.43): the bill, the cash paid on the spot, today's credit, the balance before and now. */
export function adhariBuyer(
  id: number,
  code: string,
  ur: string,
  en: string,
  bill: number,
  cash: number,
  previous: number,
  options: { dead?: boolean; personKey?: string } = {},
): AdhariSheetRow {
  const base = buyer(id, code, ur, en, bill, cash, previous, options);
  return {
    party: base.party,
    lines: base.lines,
    bill_paisa: base.bill_paisa,
    cash_paisa: base.paid_paisa,
    today_credit_paisa: rupees(bill - cash),
    previous_due: base.previous_due,
    total_due: base.total_due,
    is_dead: base.is_dead,
  };
}

/** After Events 1 to 5 on 10-10-26 (PRD R5 v1.43): اکرم 12,880 / 0 / 12,880; ارشد 18,684; شاہد (چکتا) paid in full. */
export const ADHARI_AKRAM = adhariBuyer(
  20,
  '0020',
  'اکرم ٹول ٹیکس',
  'Akram Toll Tax',
  12_880,
  0,
  0,
);
export const ADHARI_ARSHAD = adhariBuyer(21, '0021', 'ارشد ملتانی', 'Arshad Multani', 18_684, 0, 0);
export const ADHARI_SHAHID = adhariBuyer(30, '0030', 'شاہد', 'Shahid', 12_800, 12_800, 0, {
  personKey: 'p-1',
});
const sums = (bill: number, cash: number, previous: number) => ({
  bill_paisa: rupees(bill),
  cash_paisa: rupees(cash),
  today_credit_paisa: rupees(bill - cash),
  previous_due: dr(previous),
  total_due: dr(previous + bill - cash),
});
export const ADHARI_SHEET: AdhariSheet = {
  date: '2026-10-10',
  rows: [ADHARI_AKRAM, ADHARI_ARSHAD, ADHARI_SHAHID],
  total_count: 3,
  totals: sums(44_364, 12_800, 0),
  others: null,
  others_count: 0,
  all_totals: sums(44_364, 12_800, 0),
  dead_totals: null,
  credit_totals: null,
};

/** Makes ادھاری تفصیل (R5) answer. Its print is `fakeDaySlips`'s. */
export function fakeAdhariSheet(): void {
  vi.mocked(reportsApi.reportsAdhariSheet).mockImplementation((input) =>
    Promise.resolve({
      ...ADHARI_SHEET,
      date: input.date === '' ? ADHARI_SHEET.date : input.date,
    }),
  );
  // Its print data is `fakeDaySlips`'s: the backend's own ادھاری تفصیل of the worked day (`day-slips.ts`).
}

/** Makes the sheet and its print data answer; the search narrows the buyers by name or code. */
export function fakeCollectionSheet(): void {
  fakeDaySlips();
  fakeAdhariSheet();
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
