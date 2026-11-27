// The answers of the development page /dev/gardens (plan 17c): the commands of the gardens screens,
// from the books of gardens-data.ts (the numbers of E2E-40), plus the partners overview with the
// open gardens under each card. dev-bridge.ts asks this table first while the address is
// /dev/gardens. Flags in the hash: `?list` the list of gardens, `?new` the new garden form,
// `?overview` the partners overview, else one garden; `&many` (a garden with about 90 goods arrived
// entries, plan 96: the list is searched, sorted and paged like the backend's, and a saved entry is
// added to it), `&few` (a handful), `&empty` (no garden yet), `&closed` (the
// garden closed on 13-10-2026), `&fresh` (nothing paid yet, rent 50,000 still due), `&cash` (cash
// 5,000, so a 30,000 installment asks the cash question). Never part of a build: only the
// development route imports it.

import * as arrivalsReal from './arrivals-real';
import type { ArrivalEntryInput } from '../../src/bindings/ArrivalEntryInput';
import type { ArrivalRow } from '../../src/bindings/ArrivalRow';
import type { ArrivalsListInput } from '../../src/bindings/ArrivalsListInput';
import type { GardenCloseInput } from '../../src/bindings/GardenCloseInput';
import type { GardenEntryCancelInput } from '../../src/bindings/GardenEntryCancelInput';
import type { GardenEntryInput } from '../../src/bindings/GardenEntryInput';
import type { GardenGetInput } from '../../src/bindings/GardenGetInput';
import type { GardenInput } from '../../src/bindings/GardenInput';
import type { GardenReopenInput } from '../../src/bindings/GardenReopenInput';
import type { GardensListInput } from '../../src/bindings/GardensListInput';
import type { GardensSaleMoneyInput } from '../../src/bindings/GardensSaleMoneyInput';
import type { PartnersOpenVenturesInput } from '../../src/bindings/PartnersOpenVenturesInput';
import type { PartnerVentureCard } from '../../src/bindings/PartnerVentureCard';
import { listArrivals, manyArrivals } from './arrivals-many';
import { createGardenBooks, e40Closed, e40Garden, e40History, gardenRow } from './gardens-data';
import { paisaForTest } from './paisa';
import { openVenturesAnswer } from './partner-ventures';
import { MANI_ROW, SALE_MONEY_ROWS, saleMoneyAnswer } from './sale-money';
import { overviewRow, THREE_PARTNERS } from './partners';

type Input = Record<string, unknown>;
type Handler = (input: Input) => unknown;

/** True on the development page of this area. */
export function isGardensPage(): boolean {
  return window.location.hash.startsWith('#/dev/gardens');
}

const flag = (name: string): boolean =>
  new URLSearchParams(window.location.hash.split('?')[1] ?? '').has(name);

const money = (rupees: number) => paisaForTest(rupees * 100);

/** The entries of the garden of `?many` or `?few`: kept here, so a saved entry joins the list (plan 96). */
const manyRows: ArrivalRow[] = flag('many') ? manyArrivals(90) : flag('few') ? manyArrivals(6) : [];

/** Two entries all sold (one with 5 written off), for the list of the goods sold (plan 99). */
function soldEntries(): ArrivalRow[] {
  return manyArrivals(12)
    .filter((row) => row.qty_sold > 0)
    .slice(0, 2)
    .map((row, index) => {
      const writtenOff = index === 1 ? 5 : 0;
      const other = index === 0 ? 15 : 0;
      const ours = row.qty - writtenOff - other;
      return {
        ...row,
        id: 201 + index,
        number: 201 + index,
        qty_sold: ours + other,
        qty_our_sold: ours,
        qty_other_sold: other,
        qty_written_off: writtenOff,
        qty_left: 0,
        qty_our_left: 0,
        qty_other_left: 0,
        qty_other: other,
        shop_name: other === 0 ? null : 'Haji Shop',
        sale_value_paisa: money(ours * 600),
        commission_paisa: money(ours * 72),
        last_sale_date: '2026-10-09',
        can_change_split: false,
        can_change_sell_date: false,
        can_cancel: false,
        days_waiting: 0,
      };
    });
}
if (manyRows.length > 0) manyRows.push(...soldEntries());

const books = createGardenBooks(
  flag('empty')
    ? { gardens: [], history: [] }
    : flag('closed')
      ? { gardens: [e40Closed()], history: e40History(true) }
      : flag('fresh')
        ? { gardens: [gardenRow()], history: [] }
        : flag('relief')
          ? // After a relief from the owner (plan 78 D3): the rent was 50,000, is 45,000 now, 55,000 is paid, so the
            // owner owes the garden 10,000 less what the 5,000 relief covers: 5,000.
            {
              gardens: [
                e40Garden({
                  rent_paisa: money(45_000),
                  rent_original_paisa: money(50_000),
                  relief_paisa: money(5_000),
                  refunds_paisa: money(0),
                  owner_owes_paisa: money(5_000),
                }),
              ],
              history: e40History(),
            }
          : { gardens: [e40Garden()], history: e40History() },
);
if (flag('cash')) books.store.cashBefore = 5_000;

const card = (
  share: number,
  shop: number,
  pocket: number,
  balance: number,
): PartnerVentureCard => ({
  venture_id: 1,
  kind: 'garden',
  entry_date: '2026-10-10',
  number: 1,
  code: '0080',
  name_ur: 'شاہ پور باغ',
  name_en: 'Shahpur Garden',
  share_bps: share,
  paid_shop_paisa: money(shop),
  paid_pocket_paisa: money(pocket),
  money_back_paisa: money(0),
  balance_paisa: money(balance),
});

/** The overview of E2E-40 before the closing: A and B are in the garden, C is not. */
function overview() {
  const [a, b, c] = THREE_PARTNERS;
  if (a === undefined || b === undefined || c === undefined) throw new Error('three partners');
  return [
    overviewRow(a, {
      month_ventures_paisa: money(-33_000),
      total_paisa: money(-33_000),
      open_ventures: [card(6000, 33_000, 0, 11_400)],
    }),
    overviewRow(b, {
      open_ventures: [card(4000, 0, 22_000, 11_400)],
    }),
    overviewRow(c),
  ];
}

export const gardensHandlers: Record<string, Handler> = {
  gardens_defaults: () => books.defaults(),
  gardens_create_preview: (input) => books.createPreview(input as unknown as GardenInput),
  gardens_create: (input) => books.create(input as unknown as GardenInput),
  gardens_list: (input) => books.list(input as unknown as GardensListInput),
  gardens_get: (input) => books.get(input as unknown as GardenGetInput),
  gardens_summary: () => books.summary(),
  gardens_entry_preview: (input) => books.entryPreview(input as unknown as GardenEntryInput),
  gardens_entry_save: (input) => books.entrySave(input as unknown as GardenEntryInput),
  gardens_entry_cancel: (input) => books.entryCancel(input as unknown as GardenEntryCancelInput),
  gardens_close_preview: (input) => books.closePreview(input as unknown as GardenCloseInput),
  gardens_close: (input) => books.close(input as unknown as GardenCloseInput),
  gardens_reopen: (input) => books.reopen(input as unknown as GardenReopenInput),
  // Goods arrived of the garden (plan 33 version 1.1): the backend's real list, one entry in stock.
  // With `?other` (plan 90) two entries: one has goods for another shop (40, "Haji Shop"), one has none.
  gardens_arrivals_list: (input) => {
    const asked = input as unknown as ArrivalsListInput;
    if (manyRows.length > 0) return listArrivals(manyRows, asked);
    if (asked.state === 'sold_out') return listArrivals(soldEntries(), asked);
    const base = listArrivals(manyArrivals(3), asked);
    const first = base.rows[0];
    const wanted = new URLSearchParams(window.location.hash.split('?')[1] ?? '').has('other');
    if (!wanted || first === undefined) return base;
    const rows = [
      {
        ...first,
        qty_left: 100,
        qty_other: 40,
        qty_other_left: 40,
        qty_our_left: 60,
        shop_name: 'Haji Shop',
      },
      {
        ...first,
        id: 2,
        number: 2,
        qty_left: 50,
        qty_other: 0,
        qty_other_left: 0,
        qty_our_left: 50,
      },
    ];
    return { ...base, rows, total_count: rows.length };
  },
  // Where the sale money of the garden is (plan 99, PRD FR-12.53): the worked example's rows, or the
  // first of them as the sale lines of one goods arrived entry.
  gardens_sale_money: (input) => {
    const asked = input as unknown as GardensSaleMoneyInput;
    return saleMoneyAnswer(asked.arrival_id === null ? SALE_MONEY_ROWS : [MANI_ROW], asked);
  },
  windows_bounds: () => ({ first_date: '2026-08-01', last_date: '2026-10-10' }),
  partners_open_ventures: (input) => {
    const asked = input as unknown as PartnersOpenVenturesInput;
    const row = overview().find((one) => one.partner_id === asked.partner_id);
    return openVenturesAnswer(row?.open_ventures ?? [], asked);
  },
  gardens_arrival_save: () => arrivalsReal.ARRIVAL_SAVE,
  gardens_arrival_entry_preview: (input) => ({
    // The key of each row comes back with what is left for our shop: qty less the other part.
    lines: (input as ArrivalEntryInput).lines.map((line) => {
      const qty = Number(line.qty);
      const other = Number(line.other_qty ?? '0');
      const known = line.qty !== '' && !Number.isNaN(qty) && !Number.isNaN(other);
      return { key: line.key, qty: known ? qty : null, qty_our: known ? qty - other : null };
    }),
    errors: [],
    warnings: [],
  }),
  gardens_arrival_entry_save: (input) => {
    const lines = (input as ArrivalEntryInput).lines;
    if (manyRows.length === 0) {
      return {
        rows: lines.map((_, index) => ({
          ...arrivalsReal.ARRIVAL_SAVE.row,
          id: index + 1,
          number: index + 1,
        })),
        warnings: [],
      };
    }
    // A saved entry joins the list as the newest: the next numbers, today's day, nothing sold yet.
    const rows = lines.map((line, index) => {
      const number = Math.max(...manyRows.map((row) => row.number)) + 1 + index;
      const qty = Number(line.qty);
      const made: ArrivalRow = {
        ...arrivalsReal.ARRIVAL_SAVE.row,
        id: number,
        number,
        entry_date: '2026-10-30',
        sell_date: '2026-10-30',
        qty,
        qty_left: qty,
        qty_our_left: qty,
      };
      manyRows.push(made);
      return made;
    });
    return { rows, warnings: [] };
  },
  gardens_arrival_cancel: () => arrivalsReal.ARRIVAL_CANCEL,
  gardens_arrival_set_sell_date: () => arrivalsReal.ARRIVAL_SET_SELL_DATE,
  gardens_arrival_write_off: () => arrivalsReal.ARRIVAL_WRITE_OFF,
  gardens_arrival_write_off_undo: () => arrivalsReal.ARRIVAL_WRITE_OFF_UNDO,
  partners_overview: () => overview(),
  partners_list: () => ({ partners: THREE_PARTNERS, partners_locked: false }),
};
