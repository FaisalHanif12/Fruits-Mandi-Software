// Test helper for one party's statement of 30 days (plan 74, PRD FR-12.47): a statement of a buyer
// with four dates in two windows, in the shape the backend sends, the totals the backend would send for
// each window, and readers of the list of dates (a treegrid, not an AG Grid). The numbers are the ones
// of the plan's worked example (30,000 bought on 20-09, 21,360 bought and paid on 05-10) with a small
// day of 5,000 on 10-10. A double adds no money (F3): the totals below are written out, not worked out.
// Today in the tests is 10-10-2026, so the first window is 11-09-2026 to 10-10-2026.

import type { PartyLedgerDay } from '../../src/bindings/PartyLedgerDay';
import type { PartyLedgerStatement } from '../../src/bindings/PartyLedgerStatement';
import type { ReportCell } from '../../src/bindings/ReportCell';
import type { ReportPage } from '../../src/bindings/ReportPage';
import { STATEMENT_AKRAM } from './party-ledgers-real';
import { paisaForTest } from './paisa';

const day = (
  date: string,
  previous: [number, 'zero' | 'dr'],
  charged: number,
  paid: number,
  balance: [number, 'zero' | 'dr'],
): PartyLedgerDay => ({
  date,
  previous_paisa: paisaForTest(previous[0] * 100),
  previous_side: previous[1],
  charged_paisa: paisaForTest(charged * 100),
  paid_paisa: paisaForTest(paid * 100),
  balance_paisa: paisaForTest(balance[0] * 100),
  balance_side: balance[1],
});

/** Newest first, as the backend sends them. 08-20 is in the window before the first one. */
export const FOUR_DAYS: readonly PartyLedgerDay[] = [
  day('2026-10-10', [30_000, 'dr'], 5_000, 0, [35_000, 'dr']),
  day('2026-10-05', [30_000, 'dr'], 21_360, 21_360, [30_000, 'dr']),
  day('2026-09-20', [0, 'zero'], 30_000, 0, [30_000, 'dr']),
  day('2026-08-20', [0, 'zero'], 10_000, 10_000, [0, 'zero']),
];

/** The statement the double starts from; the rows and totals of a window are set on every read. */
export const FOUR_DAY_STATEMENT: PartyLedgerStatement = {
  ...STATEMENT_AKRAM,
  rows: [...FOUR_DAYS],
};

type Totals = Pick<
  PartyLedgerStatement,
  | 'opening_paisa'
  | 'opening_side'
  | 'charged_total_paisa'
  | 'paid_total_paisa'
  | 'closing_paisa'
  | 'closing_side'
>;

/** 11-09-2026 to 10-10-2026: nothing before it, 56,360 charged, 21,360 paid, 35,000 owed at its end. */
export const WINDOW_NOW_TOTALS: Totals = {
  opening_paisa: paisaForTest(0),
  opening_side: 'zero',
  charged_total_paisa: paisaForTest(5_636_000),
  paid_total_paisa: paisaForTest(2_136_000),
  closing_paisa: paisaForTest(3_500_000),
  closing_side: 'dr',
};

/** 12-08-2026 to 10-09-2026: 10,000 charged and paid, nothing owed before or after. */
export const WINDOW_BEFORE_TOTALS: Totals = {
  opening_paisa: paisaForTest(0),
  opening_side: 'zero',
  charged_total_paisa: paisaForTest(1_000_000),
  paid_total_paisa: paisaForTest(1_000_000),
  closing_paisa: paisaForTest(0),
  closing_side: 'zero',
};

/** What the backend would say for the two windows; any other range keeps the fixture's totals. */
export function windowTotals(input: { from: string | null }): Totals | undefined {
  if (input.from === '11-09-2026') return WINDOW_NOW_TOTALS;
  if (input.from === '12-08-2026') return WINDOW_BEFORE_TOTALS;
  return undefined;
}

// Readers of the list of dates.
export const treegrid = () => document.querySelector<HTMLElement>('[role="treegrid"]');
export const dayRows = () =>
  Array.from(treegrid()?.querySelectorAll<HTMLElement>('[role="row"][aria-expanded]') ?? []);
export const dayRow = (iso: string) => dayRows().find((row) => row.id.endsWith(`-${iso}`)) ?? null;
export const isOpen = (iso: string) => dayRow(iso)?.getAttribute('aria-expanded') === 'true';
export const footerRow = () => treegrid()?.lastElementChild;
/** The dates of the rows on screen, newest first. */
export const datesShown = () => dayRows().map((row) => row.id.slice(-10));

// The R35 day slip of each date of the four-date statement, written out (a double adds no money, F3):
// the figures a day opened under its row must show are its own, not another day's. A slip is the
// backend's real buyer slip with its summary lines, its one line of goods and its date replaced by these.
interface SlipDay {
  qty: number;
  /** Rupees. */
  rate: number;
  amount: number;
  /** Today's total, what was paid on the day and what was left of the day, in rupees. */
  total: number;
  paid: number;
  left: number;
  previous: [number, 'zero' | 'dr'];
  balance: [number, 'zero' | 'dr'];
}

export const SLIP_DAYS: Readonly<Record<string, SlipDay>> = {
  '2026-10-10': {
    qty: 5,
    rate: 1_000,
    amount: 5_000,
    total: 5_000,
    paid: 0,
    left: 5_000,
    previous: [30_000, 'dr'],
    balance: [35_000, 'dr'],
  },
  '2026-10-05': {
    qty: 20,
    rate: 1_068,
    amount: 21_360,
    total: 21_360,
    paid: 21_360,
    left: 0,
    previous: [30_000, 'dr'],
    balance: [30_000, 'dr'],
  },
  '2026-09-20': {
    qty: 30,
    rate: 1_000,
    amount: 30_000,
    total: 30_000,
    paid: 0,
    left: 30_000,
    previous: [0, 'zero'],
    balance: [30_000, 'dr'],
  },
  '2026-08-20': {
    qty: 10,
    rate: 1_000,
    amount: 10_000,
    total: 10_000,
    paid: 10_000,
    left: 0,
    previous: [0, 'zero'],
    balance: [0, 'zero'],
  },
};

const money = (rupees: number): ReportCell => ({
  type: 'money',
  paisa: paisaForTest(rupees * 100),
});
const balance = (value: [number, 'zero' | 'dr']): ReportCell => ({
  type: 'balance',
  paisa: paisaForTest(value[0] * 100),
  side: value[1],
});

/** The slip of `date` (`YYYY-MM-DD`) from the real buyer slip `base`; any other date is the base as it is. */
export function slipForDate(base: ReportPage, date: string): ReportPage {
  const day = SLIP_DAYS[date];
  if (day === undefined) return base;
  const cells: Record<string, ReportCell> = {
    'report-r35-goods': money(day.amount),
    'report-r35-commission': money(0),
    'report-r35-laga': money(0),
    'report-r35-today-total': money(day.total),
    'report-r35-paid-today': money(day.paid),
    'report-r35-today-left': balance([day.left, day.left === 0 ? 'zero' : 'dr']),
    'report-r35-previous': balance(day.previous),
    'report-r35-new-balance': balance(day.balance),
    'report-r35-total-qty': { type: 'number', value: day.qty },
  };
  return {
    ...base,
    filters: { ...base.filters, as_of: date },
    rows: base.rows.map((row) => ({
      ...row,
      cells: row.cells.map((cell, index) =>
        index === 2
          ? { type: 'number', value: day.qty }
          : index === 3
            ? money(day.rate)
            : index === 4
              ? money(day.amount)
              : cell,
      ),
    })),
    summary: base.summary.map((line) => ({ ...line, cell: cells[line.label_id] ?? line.cell })),
  };
}

/** A typed date `DD-MM-YYYY` as `YYYY-MM-DD`. */
export function isoOfTypedDate(text: string): string {
  const [day = '', month = '', year = ''] = text.split('-');
  return `${year}-${month}-${day}`;
}
