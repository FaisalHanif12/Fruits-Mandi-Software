// Test helper for one party's statement with a row for each entry (plan 78 A7, B1 to B3, PRD FR-12.47,
// FR-19.16): a buyer with five entries on four dates (the plan 74 worked example, with a receipt on the day
// of the last sale so one date holds two rows), a seller after Events 1 to 5, the totals the backend would
// send for each window, and readers of the table (a treegrid, not an AG Grid). A double adds no money (F3):
// every figure below is written out, never worked out. Today in the tests is 10-10-2026, so the first
// window is 11-09-2026 to 10-10-2026. The rows are oldest first, the way the screen asks for them.

import type { Balance } from '../../src/bindings/Balance';
import type { BalanceSide } from '../../src/bindings/BalanceSide';
import type { PartyLedgerHead } from '../../src/bindings/PartyLedgerHead';
import type { ReportDetailPart } from '../../src/bindings/ReportDetailPart';
import type { ReportRowSource } from '../../src/bindings/ReportRowSource';
import type { StatementEdited } from '../../src/bindings/StatementEdited';
import type { StatementRows } from '../../src/bindings/StatementRows';
import type { ReportPrintCell } from '../../src/bindings/ReportPrintCell';
import type { StatementRowsPrintInput } from '../../src/bindings/StatementRowsPrintInput';
import type { StatementRowsRow } from '../../src/bindings/StatementRowsRow';
import type { TablePrintData } from '../../src/bindings/TablePrintData';
import { paisaForTest } from './paisa';
import { printFrameFor } from './print-frame-real';

const rupees = (value: number) => paisaForTest(value * 100);

const balance = (value: number, side: BalanceSide): Balance => ({
  amount_paisa: rupees(value),
  side,
});

/** A name in both scripts. */
const names = (ur: string, en: string) => ({ ur, en });

interface RowSpec {
  date: string;
  id: number;
  type: string;
  number: number | null;
  source: ReportRowSource;
  dr: number;
  cr: number;
  spot?: number;
  balance: [number, BalanceSide];
  items?: readonly [ur: string, en: string, qty: number][];
  parts?: readonly ReportDetailPart[];
  narration?: string;
  edited?: StatementEdited;
  replaced?: boolean;
}

function row(spec: RowSpec): StatementRowsRow {
  return {
    date: spec.date,
    entry_id: spec.id,
    type_id: spec.type,
    number: spec.number,
    source: spec.source,
    dr_paisa: rupees(spec.dr),
    cr_paisa: rupees(spec.cr),
    on_spot_paisa: rupees(spec.spot ?? 0),
    balance: balance(spec.balance[0], spec.balance[1]),
    short: (spec.items ?? []).map(([ur, en, qty]) => ({ item: names(ur, en), qty })),
    parts: [...(spec.parts ?? [])],
    narration: spec.narration ?? null,
    edited: spec.edited ?? null,
    is_reversal: false,
    is_replaced: spec.replaced ?? false,
  };
}

const sale = (
  bikri: number,
  item: [string, string],
  qty: number,
  rate: number,
): ReportDetailPart => ({
  kind: 'sale',
  bikri_number: bikri,
  item: names(item[0], item[1]),
  packing: names('پیٹی', 'Box'),
  qty,
  rate_paisa: rupees(rate),
  value_paisa: rupees(qty * rate),
  commission_paisa: rupees(0),
  laga_paisa: rupees(0),
});

const bikriSource = (id: number): ReportRowSource => ({ kind: 'bikri', id });

/** The owner's reason, in an edit stamp. */
export const EDIT_STAMP: StatementEdited = {
  at: '2026-10-10T11:30:00Z',
  by: 'Owner',
  count: 1,
  reason: 'rate was wrong',
  sign: null,
};

/** The head of the buyer of the worked example, account 20. */
export const HEAD_BUYER: PartyLedgerHead = {
  account_id: 20,
  code: '0020',
  name_ur: 'اکرم ٹول ٹیکس',
  name_en: 'Akram Toll Tax',
  kind: 'buyer',
  is_active: true,
  is_dead: false,
};

/** The five entries of the buyer, oldest first. 08-20 is in the window before the first one. */
export const ENTRIES: readonly StatementRowsRow[] = [
  row({
    date: '2026-08-20',
    id: 101,
    type: 'report-doc-bikri-first',
    number: 4,
    source: bikriSource(4),
    dr: 10_000,
    cr: 10_000,
    spot: 10_000,
    balance: [0, 'zero'],
    items: [['کیلا', 'Banana', 10]],
    parts: [sale(4, ['کیلا', 'Banana'], 10, 1_000)],
  }),
  row({
    date: '2026-09-20',
    id: 107,
    type: 'report-doc-bikri-first',
    number: 7,
    source: bikriSource(7),
    dr: 30_000,
    cr: 0,
    balance: [30_000, 'dr'],
    items: [['سیب', 'Apple', 30]],
    parts: [sale(7, ['سیب', 'Apple'], 30, 1_000)],
  }),
  row({
    date: '2026-10-05',
    id: 109,
    type: 'report-doc-bikri-first',
    number: 9,
    source: bikriSource(9),
    dr: 21_360,
    cr: 21_360,
    spot: 21_360,
    balance: [30_000, 'dr'],
    items: [['سیب', 'Apple', 20]],
    parts: [sale(9, ['سیب', 'Apple'], 20, 1_068)],
  }),
  row({
    date: '2026-10-10',
    id: 112,
    type: 'report-doc-bikri-first',
    number: 12,
    source: bikriSource(12),
    dr: 5_000,
    cr: 0,
    balance: [35_000, 'dr'],
    items: [
      ['مالٹا', 'Orange', 3],
      ['کیلا', 'Banana', 2],
    ],
    parts: [sale(12, ['مالٹا', 'Orange'], 3, 1_000), sale(12, ['کیلا', 'Banana'], 2, 1_000)],
    edited: EDIT_STAMP,
  }),
  row({
    date: '2026-10-10',
    id: 113,
    type: 'report-doc-receipt',
    number: 3,
    source: { kind: 'voucher', id: 3 },
    dr: 0,
    cr: 2_000,
    balance: [33_000, 'dr'],
    narration: 'cash at the shop',
  }),
];

/** Every entry of the party, with the figures of the whole statement. */
export const BUYER_STATEMENT: StatementRows = {
  head: HEAD_BUYER,
  from: null,
  to: null,
  brought_forward: balance(0, 'zero'),
  rows: [...ENTRIES],
  count: ENTRIES.length,
  dr_total_paisa: rupees(66_360),
  cr_total_paisa: rupees(33_360),
  on_spot_total_paisa: rupees(31_360),
  closing: balance(33_000, 'dr'),
};

type Totals = Pick<
  StatementRows,
  'brought_forward' | 'dr_total_paisa' | 'cr_total_paisa' | 'on_spot_total_paisa' | 'closing'
>;

/** 11-09-2026 to 10-10-2026: nothing owed before it, 56,360 charged, 23,360 paid, 33,000 owed at its end. */
export const WINDOW_NOW_TOTALS: Totals = {
  brought_forward: balance(0, 'zero'),
  dr_total_paisa: rupees(56_360),
  cr_total_paisa: rupees(23_360),
  on_spot_total_paisa: rupees(21_360),
  closing: balance(33_000, 'dr'),
};

/** 12-08-2026 to 10-09-2026: 10,000 charged and paid, nothing owed before or after. */
export const WINDOW_BEFORE_TOTALS: Totals = {
  brought_forward: balance(0, 'zero'),
  dr_total_paisa: rupees(10_000),
  cr_total_paisa: rupees(10_000),
  on_spot_total_paisa: rupees(10_000),
  closing: balance(0, 'zero'),
};

/** From 01-10-2026: 30,000 was owed before it (the entry of 20-09), so the table opens with that balance. */
export const RANGE_FROM_OCTOBER_TOTALS: Totals = {
  brought_forward: balance(30_000, 'dr'),
  dr_total_paisa: rupees(26_360),
  cr_total_paisa: rupees(23_360),
  on_spot_total_paisa: rupees(21_360),
  closing: balance(33_000, 'dr'),
};

/** What the backend would say for the windows the tests ask for; any other range keeps the fixture's totals. */
export function windowTotals(input: {
  account_id?: number;
  from?: string | undefined;
}): Totals | undefined {
  // The totals above are the buyer's (account 20, and 21 in the dev page); the seller keeps his own.
  if (input.account_id === 19) return undefined;
  if (input.from === '11-09-2026') return WINDOW_NOW_TOTALS;
  if (input.from === '12-08-2026') return WINDOW_BEFORE_TOTALS;
  if (input.from === '01-10-2026') return RANGE_FROM_OCTOBER_TOTALS;
  return undefined;
}

/** An entry that an edit replaced: it shows only with "show cancelled too" (plan 78 B2). */
export const REPLACED_ENTRY: StatementRowsRow = row({
  date: '2026-10-10',
  id: 111,
  type: 'report-doc-bikri-first',
  number: 12,
  source: bikriSource(12),
  dr: 4_000,
  cr: 0,
  balance: [34_000, 'dr'],
  items: [['مالٹا', 'Orange', 3]],
  replaced: true,
});

// The seller after Events 1 to 5: an advance of 15,000 on 01-09 (before the window), then بکری 1 and the payment.
export const HEAD_SELLER: PartyLedgerHead = {
  account_id: 19,
  code: '0010',
  name_ur: 'رحیم',
  name_en: 'Rahim',
  kind: 'seller',
  is_active: true,
  is_dead: false,
};

export const SELLER_ENTRIES: readonly StatementRowsRow[] = [
  row({
    date: '2026-09-01',
    id: 201,
    type: 'report-doc-advance',
    number: 1,
    source: { kind: 'voucher', id: 1 },
    dr: 15_000,
    cr: 0,
    balance: [15_000, 'dr'],
  }),
  row({
    date: '2026-10-10',
    id: 202,
    type: 'report-doc-bikri-first',
    number: 1,
    source: bikriSource(1),
    dr: 5_000,
    cr: 24_316,
    balance: [4_316, 'cr'],
    items: [['کیلا', 'Banana', 50]],
  }),
  row({
    date: '2026-10-10',
    id: 203,
    type: 'report-doc-payment',
    number: 1,
    source: { kind: 'voucher', id: 2 },
    dr: 4_316,
    cr: 0,
    balance: [0, 'zero'],
  }),
];

/** رحیم's whole statement; the first window starts after the advance, so it opens with 15,000 brought forward. */
export const SELLER_STATEMENT: StatementRows = {
  head: HEAD_SELLER,
  from: null,
  to: null,
  brought_forward: balance(15_000, 'dr'),
  rows: [...SELLER_ENTRIES],
  count: SELLER_ENTRIES.length,
  dr_total_paisa: rupees(9_316),
  cr_total_paisa: rupees(24_316),
  on_spot_total_paisa: rupees(0),
  closing: balance(0, 'zero'),
};

/** اکرم of the worked day (Events 1 to 5), account 21: بکری 1 charged 12,880 and he paid 10,000 (E2E-67). */
export const HEAD_AKRAM: PartyLedgerHead = { ...HEAD_BUYER, account_id: 21 };

export const AKRAM_ENTRIES: readonly StatementRowsRow[] = [
  row({
    date: '2026-10-10',
    id: 301,
    type: 'report-doc-bikri-first',
    number: 1,
    source: bikriSource(1),
    dr: 12_880,
    cr: 0,
    balance: [12_880, 'dr'],
    items: [['کیلا', 'Banana', 20]],
    parts: [sale(1, ['کیلا', 'Banana'], 20, 600)],
  }),
  row({
    date: '2026-10-10',
    id: 302,
    type: 'report-doc-receipt',
    number: 1,
    source: { kind: 'voucher', id: 5 },
    dr: 0,
    cr: 10_000,
    balance: [2_880, 'dr'],
  }),
];

export const AKRAM_STATEMENT: StatementRows = {
  head: HEAD_AKRAM,
  from: null,
  to: null,
  brought_forward: balance(0, 'zero'),
  rows: [...AKRAM_ENTRIES],
  count: AKRAM_ENTRIES.length,
  dr_total_paisa: rupees(12_880),
  cr_total_paisa: rupees(10_000),
  on_spot_total_paisa: rupees(0),
  closing: balance(2_880, 'dr'),
};

// Readers of the table.
export const treegrid = () => document.querySelector<HTMLElement>('[role="treegrid"]');
export const entryRows = () =>
  Array.from(treegrid()?.querySelectorAll<HTMLElement>('[role="row"][aria-expanded]') ?? []);
/** The row of one entry (its id in the double). */
export const entryRow = (id: number) =>
  entryRows().find((candidate) => candidate.id.endsWith(`-${String(id)}`)) ?? null;
export const isOpen = (id: number) => entryRow(id)?.getAttribute('aria-expanded') === 'true';
/**
 * The row under the headings that stands for the balance brought forward, when there is one. The table's own
 * rows are the headings, the lead row (only when there is a balance to bring), and the totals.
 */
export const leadRow = () => {
  const own = Array.from(treegrid()?.querySelectorAll<HTMLElement>(':scope > [role="row"]') ?? []);
  return own.length >= 3 ? (own[1] ?? null) : null;
};
export const footerRow = () => treegrid()?.lastElementChild;
/** The ids of the entries on screen, in the order of the table. */
export const entriesShown = () =>
  entryRows().map((candidate) => Number(candidate.id.slice(candidate.id.lastIndexOf('-') + 1)));

/** A typed date `DD-MM-YYYY` as `YYYY-MM-DD`. */
export function isoOfTypedDate(text: string): string {
  const [day = '', month = '', year = ''] = text.split('-');
  return `${year}-${month}-${day}`;
}

/**
 * What the backend sends for the print of the statement: a table with a row for each entry, the totals and the
 * closing balance in words. A double adds no money: the rows are the buyer's two entries of 10-10-2026, written
 * out. A period that holds nothing (it starts in 2000) has no row, which the preview says in words.
 */
export function statementRowsPrintAnswer(input: StatementRowsPrintInput): TablePrintData {
  const empty = input.from === '01-01-2000';
  const text = (value: string): ReportPrintCell => ({ type: 'text', text: value });
  const money = (value: string): ReportPrintCell => ({ type: 'money', text: value });
  return {
    frame: printFrameFor({
      language: input.language,
      title_id: 'print-statement-rows-title',
      facts: [],
    }),
    paper: input.paper ?? 'a4',
    columns: [
      { key: 'date', title_id: 'print-statement-rows-col-date', kind: 'date' },
      { key: 'entry', title_id: 'print-statement-rows-col-entry', kind: 'text' },
      { key: 'detail', title_id: 'print-statement-rows-col-detail', kind: 'text' },
      { key: 'debit', title_id: 'print-statement-rows-col-debit', kind: 'money' },
      { key: 'credit', title_id: 'print-statement-rows-col-credit', kind: 'money' },
      { key: 'balance', title_id: 'print-statement-rows-col-balance', kind: 'balance' },
    ],
    column_labels: [null, null, null, null, null, null],
    rows: empty
      ? []
      : [
          {
            kind: 'normal',
            cells: [
              { type: 'date', text: '10-10-2026' },
              text('Sale 12'),
              text(input.items ? 'Orange 3, Banana 2' : ''),
              money('5,000'),
              money('0'),
              { type: 'balance', text: '35,000', side_id: 'common-side-dr' },
            ],
          },
          {
            kind: 'normal',
            cells: [
              { type: 'date', text: '10-10-2026' },
              text('Receipt 3'),
              text(''),
              money('0'),
              money('2,000'),
              { type: 'balance', text: '33,000', side_id: 'common-side-dr' },
            ],
          },
        ],
    row_count: empty ? '0' : '2',
    totals_label_id: 'glossary-meezan',
    totals: [
      { type: 'empty' },
      { type: 'empty' },
      { type: 'empty' },
      money('5,000'),
      money('2,000'),
      { type: 'balance', text: '33,000', side_id: 'common-side-dr' },
    ],
    words: empty ? null : { label_id: 'print-in-words', text: 'thirty-three thousand rupees only' },
    extra_totals: [],
  };
}
