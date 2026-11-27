// A garden with many goods arrived entries (plan 96): about 90 of them, in several items, some sold in
// part with their sales, some written off, some with goods for another shop, one cancelled. The list
// answers like the backend's `gardens_arrivals_list`: the search over the number, the item and the other
// shop, the sort of a column title, 30 rows a page and the totals over the whole filter. For the
// development page /dev/gardens?many and for the tests of the list; never part of a build.

import type { ArrivalRow } from '../../src/bindings/ArrivalRow';
import type { ArrivalsList } from '../../src/bindings/ArrivalsList';
import type { ArrivalsListInput } from '../../src/bindings/ArrivalsListInput';
import type { RecordRef } from '../../src/bindings/RecordRef';
import * as arrivals from './arrivals-real';
import { paisaForTest } from './paisa';
import { commissionWorking } from './trips-data';

/** The items of the fruit the garden grows: the first is the banana of the backend's own list. */
const ITEMS: readonly RecordRef[] = [
  { id: 1, code: '0040', name_ur: 'کیلا', name_en: 'Banana' },
  { id: 2, code: '0041', name_ur: 'مالٹا', name_en: 'Orange' },
  { id: 3, code: '0042', name_ur: 'آم', name_en: 'Mango' },
  { id: 4, code: '0043', name_ur: 'سیب', name_en: 'Apple' },
  { id: 5, code: '0044', name_ur: 'انار', name_en: 'Pomegranate' },
];
const PACKINGS: readonly RecordRef[] = [
  { id: 1, code: '0050', name_ur: 'پیٹی', name_en: 'Box' },
  { id: 2, code: '0051', name_ur: 'کریٹ', name_en: 'Crate' },
];
/** The 30 days the entries came over, counted back from this day. */
const LAST_DAY = Date.UTC(2026, 9, 30);
const DAY_MS = 86_400_000;

function iso(daysBack: number): string {
  return new Date(LAST_DAY - daysBack * DAY_MS).toISOString().slice(0, 10);
}

function template(): ArrivalRow {
  const base = arrivals.ARRIVALS_IN_STOCK.rows[0];
  if (base === undefined) throw new Error('the backend file has no arrival');
  return base;
}

/** The nth entry, 1-based: its sales, its write-offs and its other-shop part follow from the number. */
function entry(number: number): ArrivalRow {
  const base = template();
  const item = ITEMS[number % ITEMS.length] ?? ITEMS[0];
  const packing = PACKINGS[number % PACKINGS.length] ?? PACKINGS[0];
  if (item === undefined || packing === undefined) throw new Error('no item');
  const qty = 40 + ((number * 7) % 9) * 20;
  // Every third entry sold a part in our shop in two sales; every ninth also wrote some off.
  const writtenOff = number % 9 === 0 ? 10 : 0;
  const sold = number % 3 === 0 ? Math.min(20 + ((number * 3) % 5) * 10, qty - writtenOff - 10) : 0;
  const left = qty - sold - writtenOff;
  // Every fifth entry keeps 30 نگ for another shop, when it has that many left.
  const other = number % 5 === 0 && left >= 30 ? 30 : 0;
  const day = Math.max(0, 30 - Math.floor((number * 30) / 90));
  const lots =
    sold === 0
      ? []
      : [
          {
            bikri_id: number,
            bikri_number: 100 + number,
            lot_id: number * 10,
            lot_no: 1,
            lot_counter: 1 + Math.floor(number / 5),
            entry_date: iso(Math.max(0, day - 1)),
            qty_sold: sold - 10,
          },
          {
            bikri_id: number + 1,
            bikri_number: 101 + number,
            lot_id: number * 10 + 1,
            lot_no: 1,
            lot_counter: 2 + Math.floor(number / 5),
            entry_date: iso(Math.max(0, day - 2)),
            qty_sold: 10,
          },
        ];
  return {
    ...base,
    id: number,
    number,
    version: 1,
    entry_date: iso(day),
    sell_date: iso(Math.max(0, day - 1)),
    item,
    packing,
    qty,
    qty_sold: sold,
    qty_written_off: writtenOff,
    qty_left: left,
    qty_other: other,
    shop_name: other === 0 ? null : 'Haji Shop',
    qty_our_sold: sold,
    qty_other_sold: 0,
    qty_our_left: left - other,
    qty_other_left: other,
    can_cancel: sold === 0 && writtenOff === 0,
    can_change_split: left > 0,
    can_change_sell_date: left > 0,
    days_waiting: left > 0 ? day : 0,
    locked_reason: null,
    lots,
    write_offs:
      writtenOff === 0
        ? []
        : [{ move_id: number * 100, move_date: iso(day), qty: writtenOff, reason: 'Spoiled' }],
    commission_paisa: paisaForTest(sold * 150 * 100),
    commission_working: commissionWorking(sold * 60, sold * 60, sold * 30),
    // The value of its sales and the last day of one (plan 99, PRD FR-12.53).
    sale_value_paisa: paisaForTest(sold * 500 * 100),
    last_sale_date: sold === 0 ? null : iso(Math.max(0, day - 2)),
    note: number === 4 ? 'First picking of the season' : null,
  };
}

/** Every entry of the garden that is listed, oldest number first. */
export function manyArrivals(count = 90): ArrivalRow[] {
  return Array.from({ length: count }, (_, index) => entry(index + 1));
}

function textOf(row: ArrivalRow): string {
  return [
    `GA-${String(row.number)}`,
    String(row.number),
    row.item.name_ur,
    row.item.name_en,
    row.item.code,
    row.shop_name ?? '',
    row.note ?? '',
  ]
    .join(' ')
    .toLowerCase();
}

const SORTS: Record<string, (row: ArrivalRow) => string | number> = {
  number: (row) => row.number,
  entry_date: (row) => row.entry_date,
  sell_date: (row) => row.sell_date,
  item: (row) => row.item.name_en,
  qty: (row) => row.qty,
  qty_sold: (row) => row.qty_sold,
  qty_left: (row) => row.qty_left,
};

/** True when the row is in the state asked for (the backend's `ArrivalStateFilter`). */
function inState(row: ArrivalRow, state: ArrivalsListInput['state']): boolean {
  if (state === 'in_stock') return row.status === 'saved' && row.qty_left > 0;
  if (state === 'sold_out') return row.status === 'saved' && row.qty_left === 0;
  return true;
}

/** The answer of `gardens_arrivals_list` for `rows`: filtered, sorted, one page, with the totals of the filter. */
export function listArrivals(all: readonly ArrivalRow[], input: ArrivalsListInput): ArrivalsList {
  const wanted = input.search?.trim().toLowerCase() ?? '';
  const found = all.filter(
    (row) => inState(row, input.state) && (wanted === '' || textOf(row).includes(wanted)),
  );
  const key = SORTS[input.sort?.column ?? 'entry_date'] ?? SORTS['entry_date'];
  const sign = input.sort === null || input.sort.direction === 'asc' ? 1 : -1;
  const sorted = [...found].sort((a, b) => {
    const first = key?.(a) ?? 0;
    const second = key?.(b) ?? 0;
    if (first === second) return (a.number - b.number) * sign;
    return (first < second ? -1 : 1) * sign;
  });
  const limit = input.limit === 0 ? 100 : input.limit;
  const sum = (pick: (row: ArrivalRow) => number) =>
    found.reduce((total, row) => total + pick(row), 0);
  return {
    rows: sorted.slice(input.offset, input.offset + limit),
    total_count: found.length,
    totals: {
      qty: sum((row) => row.qty),
      qty_sold: sum((row) => row.qty_sold),
      qty_written_off: sum((row) => row.qty_written_off),
      qty_left: sum((row) => row.qty_left),
      qty_our_sold: sum((row) => row.qty_our_sold),
      qty_other_sold: sum((row) => row.qty_other_sold),
      qty_our_left: sum((row) => row.qty_our_left),
      qty_other_left: sum((row) => row.qty_other_left),
      sale_value_paisa: paisaForTest(sum((row) => row.qty_sold) * 500 * 100),
      commission_paisa: paisaForTest(sum((row) => row.qty_sold) * 150 * 100),
    },
    months: [],
  };
}
