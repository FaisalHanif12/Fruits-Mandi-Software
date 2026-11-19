// The stand-in answers of `bikri_list` for the development page /dev/bikri?today (plan 72): today's
// section with Event 1, and a window of earlier days with several بکری, a cancelled one, an old one
// with a truck, and empty days, so the screen can be photographed in both languages and at every
// window size. A development aid only; the real answers are the backend's.

import type { BikriListRow } from '../../src/bindings/BikriListRow';
import type { BikriList } from '../../src/bindings/BikriList';
import type { BikriListInput } from '../../src/bindings/BikriListInput';
import { listRow } from './bikri-saved';
import { IDS } from './bikri-data';
import { paisaForTest } from './paisa';
import { pastAnswer } from './past';

const refOf = (id: number, code: string, ur: string, en: string) => ({
  id,
  code,
  name_ur: ur,
  name_en: en,
});
const RAHIM = refOf(IDS.rahim, '0010', 'رحیم', 'Rahim');
const KARIM = refOf(IDS.karim, '0011', 'کریم', 'Karim');
const BANANA = refOf(IDS.banana, '0001', 'کیلا', 'Banana');
const APPLE = refOf(IDS.apple, '0002', 'سیب', 'Apple');

function row(
  id: number,
  number: number,
  day: string,
  seller: typeof RAHIM,
  item: typeof BANANA,
  counter: number,
  more: Partial<BikriListRow> = {},
): BikriListRow {
  return listRow({
    id,
    number,
    entry_date: day,
    created_at: `${day}T07:${String(10 + (id % 40))}:00Z`,
    truck_no: null,
    sellers: [seller],
    items: [item],
    lots: [{ lot_id: 500 + id, seller, item, counter }],
    gross: paisaForTest(1_200_000 + id * 10_000),
    net: paisaForTest(900_000 + id * 10_000),
    buyer_total: paisaForTest(1_300_000 + id * 10_000),
    ...more,
  });
}

const WINDOW = [
  row(53, 8, '2026-10-09', KARIM, APPLE, 3),
  row(52, 7, '2026-10-09', RAHIM, BANANA, 2, { truck_no: 'LES-1234' }),
  row(51, 6, '2026-10-09', RAHIM, BANANA, 1),
  // A row that cannot be changed (its goods are sold) and a row of a closed day, for the photographs.
  row(55, 5, '2026-10-07', KARIM, APPLE, 2, {
    can_edit: false,
    can_cancel: false,
    locked_reason: 'goods_moved',
  }),
  row(54, 4, '2026-10-07', RAHIM, BANANA, 1, { status: 'cancelled' }),
  row(56, 3, '2026-10-06', KARIM, APPLE, 1, { needs_reopen: true }),
  row(57, 2, '2026-10-03', RAHIM, BANANA, 1),
];

/** The خام بکری of each day in whole rupees, as a stand-in for the backend's day totals (the rows above, cancelled left out). */
const GROSS = {
  '2026-10-09': 51_600,
  '2026-10-07': 17_500,
  '2026-10-06': 17_600,
  '2026-10-03': 17_700,
};

/** The backend's sums for one open day: stored sums of its saved بکری, a cancelled one left out. */
const DAY_TOTALS = {
  '2026-10-09': {
    qty_received: 150,
    qty_sold: 150,
    gross: paisaForTest(5_160_000),
    net: paisaForTest(4_260_000),
    buyer_total: paisaForTest(5_460_000),
  },
  '2026-10-07': {
    qty_received: 50,
    qty_sold: 50,
    gross: paisaForTest(1_750_000),
    net: paisaForTest(1_450_000),
    buyer_total: paisaForTest(1_850_000),
  },
  '2026-10-06': {
    qty_received: 50,
    qty_sold: 50,
    gross: paisaForTest(1_760_000),
    net: paisaForTest(1_460_000),
    buyer_total: paisaForTest(1_860_000),
  },
  '2026-10-03': {
    qty_received: 50,
    qty_sold: 50,
    gross: paisaForTest(1_770_000),
    net: paisaForTest(1_470_000),
    buyer_total: paisaForTest(1_870_000),
  },
};

/** `?many`: 60 بکری over six days (10 a day), to see a window that fills the screen and scrolls. */
const MANY = Array.from({ length: 60 }, (_, index) => {
  const day = 9 - Math.floor(index / 10);
  return row(
    100 + index,
    100 - index,
    `2026-10-${String(day).padStart(2, '0')}`,
    index % 2 === 0 ? RAHIM : KARIM,
    index % 2 === 0 ? BANANA : APPLE,
    1 + Math.floor(index / 2),
  );
});
const MANY_GROSS = Object.fromEntries(
  [9, 8, 7, 6, 5, 4].map((day) => [
    `2026-10-${String(day).padStart(2, '0')}`,
    120_000 + day * 1_000,
  ]),
);

/** What the stand-in answers to `bikri_list`: `empty` gives no بکری at all, `many` a full window. */
export function devPastList(
  input: Record<string, unknown>,
  empty: boolean,
  many = false,
): BikriList {
  const text = (value: unknown): string | null => (typeof value === 'string' ? value : null);
  const number = (value: unknown): number => (typeof value === 'number' ? value : 0);
  const asked: BikriListInput = {
    date: text(input['date']),
    from: text(input['from']),
    to: text(input['to']),
    seller_id: null,
    item_id: null,
    status: null,
    show_finished: false,
    search: null,
    sort: null,
    group_by_month: false,
    group_by_day: input['group_by_day'] === true,
    all: input['all'] === true,
    offset: number(input['offset']),
    limit: number(input['limit']),
  };
  if (empty) return pastAnswer(asked, { today: [], window: [] });
  return pastAnswer(asked, {
    today: [listRow(), listRow({ id: 42, number: 2, status: 'cancelled' })],
    window: many ? MANY : WINDOW,
    gross: many ? MANY_GROSS : GROSS,
    todayTotals: {
      qty_received: 50,
      qty_sold: 50,
      gross: paisaForTest(2_940_000),
      net: paisaForTest(1_931_600),
      buyer_total: paisaForTest(3_156_400),
    },
    windowTotals: {
      gross: paisaForTest(10_440_000),
      net: paisaForTest(8_640_000),
      buyer_total: paisaForTest(11_040_000),
    },
    dayTotals: many ? {} : DAY_TOTALS,
  });
}
