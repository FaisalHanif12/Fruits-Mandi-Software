// A test double of the print data of پچھلی بکری (plan 72 P8), shaped like the data backend-past sends
// (`bikri_list_print_data`): the facts of the header, the twelve columns, today's section (null without
// `with_today`), each day of the window newest first (empty days included) and the totals, all as printed
// text. A double adds no money (the frontend never does, F3): the count and the خام بکری of a day, and
// the totals, are given by the test. The real counts and totals are the backend's. The names are in the
// print language the call asked for.

import type { PrintReportColumn } from '../../src/bindings/PrintReportColumn';
import type { PrintReportRow } from '../../src/bindings/PrintReportRow';
import type { ReportPrintCell } from '../../src/bindings/ReportPrintCell';
import type { BikriListPrintData } from '../../src/bindings/BikriListPrintData';
import type { BikriListPrintDay } from '../../src/bindings/BikriListPrintDay';
import type { BikriListPrintInput } from '../../src/bindings/BikriListPrintInput';
import { addDays } from '../../src/features/bikri/past-window';
import { isoOf } from './past';

export const PAST_PRINT_COLUMNS: PrintReportColumn[] = [
  { key: 'number', title_id: 'bikri-list-col-number', kind: 'number' },
  { key: 'date', title_id: 'report-col-date', kind: 'date' },
  { key: 'time', title_id: 'bikri-list-col-time', kind: 'text' },
  { key: 'truck', title_id: 'bikri-list-col-truck', kind: 'text' },
  { key: 'sellers', title_id: 'bikri-list-col-sellers', kind: 'names' },
  { key: 'items', title_id: 'bikri-list-col-items', kind: 'names' },
  { key: 'qty_sold', title_id: 'bikri-saved-qty-sold', kind: 'number' },
  { key: 'qty_received', title_id: 'bikri-col-qty-received', kind: 'number' },
  { key: 'gross', title_id: 'glossary-kham-bikri', kind: 'money' },
  { key: 'net', title_id: 'glossary-safi-bikri', kind: 'money' },
  { key: 'buyer_total', title_id: 'bikri-list-col-buyer-total', kind: 'money' },
  { key: 'status', title_id: 'bikri-list-col-status', kind: 'label' },
];

const text = (value: string): ReportPrintCell => ({ type: 'text', text: value });
const number = (value: string): ReportPrintCell => ({ type: 'number', text: value });
const money = (value: string): ReportPrintCell => ({ type: 'money', text: value });
const names = (value: string): ReportPrintCell => ({ type: 'names', text: value });

/** The status cell: a label, or the message of an edited بکری with its status as an argument. */
function statusCell(status: 'saved' | 'cancelled', edited: boolean): ReportPrintCell {
  const id = status === 'cancelled' ? 'bikri-status-cancelled' : 'bikri-status-saved';
  if (!edited) return { type: 'label', id };
  return {
    type: 'message',
    id: 'bikri-list-status-edited',
    args: [{ name: 'status', value: { type: 'message', id, args: [] } }],
  };
}

/** One بکری as a printed row: Rahim's bananas, the numbers of Event 1, with the number and the day given. */
export function pastPrintRow(
  language: 'ur' | 'en',
  numberText: string,
  dateText: string,
  extra: { truck?: string; status?: 'saved' | 'cancelled'; edited?: boolean } = {},
): PrintReportRow {
  return {
    kind: 'normal',
    cells: [
      number(numberText),
      { type: 'date', text: dateText },
      text('12:30'),
      extra.truck === undefined ? { type: 'empty' } : text(extra.truck),
      names(language === 'ur' ? 'رحیم، کیلا-1' : 'Rahim, Banana-1'),
      names(language === 'ur' ? 'کیلا' : 'Banana'),
      number('50'),
      number('50'),
      money('29,400'),
      money('19,316'),
      money('31,564'),
      statusCell(extra.status ?? 'saved', extra.edited ?? false),
    ],
  };
}

/** What the double is given: which days hold بکری, and the figures the backend would say. */
export interface PastPrintFake {
  /** Today, `YYYY-MM-DD`. */
  today?: string;
  /** Rows of today's section (number, truck). */
  todayRows?: readonly { number: string; truck?: string; edited?: boolean }[];
  /** The rows of the window by day `YYYY-MM-DD`: numbers (and the cancelled ones). */
  windowRows?: Readonly<
    Record<string, readonly { number: string; status?: 'saved' | 'cancelled' }[]>
  >;
}

const display = (iso: string) => {
  const [year = '', month = '', day = ''] = iso.split('-');
  return `${day}-${month}-${year}`;
};

/** The print data for a call: the days come from the call's `from` and `to`, the rows from the fake. */
export function pastPrintDataFor(
  input: BikriListPrintInput,
  {
    today = '2026-10-10',
    todayRows = [{ number: '1', truck: 'LES-1234' }],
    windowRows = { '2026-10-09': [{ number: '2' }, { number: '3' }] },
  }: PastPrintFake = {},
): BikriListPrintData {
  const { language } = input;
  const from = isoOf(input.from);
  const to = isoOf(input.to);
  const days: BikriListPrintDay[] = [];
  for (let date = to; date >= from; date = addDays(date, -1)) {
    const rows = (windowRows[date] ?? [])
      // Without "show every بکری" a cancelled one is left out, as on the screen.
      .filter((row) => input.all || row.status !== 'cancelled')
      .map((row) =>
        pastPrintRow(language, row.number, display(date), { status: row.status ?? 'saved' }),
      );
    days.push({
      date: display(date),
      date_iso: date,
      count: String(rows.length),
      // Given by the test, never worked out from the rows.
      gross: rows.length === 0 ? '0' : rows.length === 2 ? '58,800' : '29,400',
      rows,
    });
  }
  const todaySection: BikriListPrintDay = {
    date: display(today),
    date_iso: today,
    count: String(todayRows.length),
    gross: '29,400',
    rows: todayRows.map((row) =>
      pastPrintRow(language, row.number, display(today), {
        ...(row.truck === undefined ? {} : { truck: row.truck }),
        ...(row.edited === true ? { edited: true } : {}),
      }),
    ),
  };
  const withToday = input.with_today;
  const facts = input.all
    ? [{ label_id: 'bikri-list-print-all', value: '' }]
    : [
        { label_id: 'report-filter-from', value: display(from) },
        { label_id: 'report-filter-to', value: display(to) },
        ...(input.search === null
          ? []
          : [{ label_id: 'bikri-list-print-search', value: input.search }]),
      ];
  const empty: ReportPrintCell = { type: 'empty' };
  return {
    language,
    dir: language === 'ur' ? 'rtl' : 'ltr',
    paper: input.paper ?? 'a4',
    title_id: 'bikri-list-print-title',
    filters: facts,
    columns: PAST_PRINT_COLUMNS,
    today: withToday ? todaySection : null,
    days,
    row_count: String(
      (withToday ? todayRows.length : 0) + days.reduce((sum, day) => sum + day.rows.length, 0),
    ),
    totals_label_id: 'report-total',
    // The totals of the whole filter, as the backend would say them.
    totals: [
      empty,
      empty,
      empty,
      empty,
      empty,
      empty,
      number('200'),
      number('200'),
      money('117,600'),
      money('77,264'),
      money('126,256'),
      empty,
    ],
  };
}
