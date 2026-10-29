// The answers of the development page /dev/gardens (plan 17c): the commands of the gardens screens,
// from the books of gardens-data.ts (the numbers of E2E-40), plus the partners overview with the
// open gardens under each card. dev-bridge.ts asks this table first while the address is
// /dev/gardens. Flags in the hash: `?list` the list of gardens, `?new` the new garden form,
// `?overview` the partners overview, else one garden; `&empty` (no garden yet), `&closed` (the
// garden closed on 13-10-2026), `&fresh` (nothing paid yet, rent 50,000 still due), `&cash` (cash
// 5,000, so a 30,000 installment asks the cash question). Never part of a build: only the
// development route imports it.

import * as arrivalsReal from './arrivals-real';
import type { ArrivalEntryInput } from '../../src/bindings/ArrivalEntryInput';
import type { GardenCloseInput } from '../../src/bindings/GardenCloseInput';
import type { GardenEntryCancelInput } from '../../src/bindings/GardenEntryCancelInput';
import type { GardenEntryInput } from '../../src/bindings/GardenEntryInput';
import type { GardenGetInput } from '../../src/bindings/GardenGetInput';
import type { GardenInput } from '../../src/bindings/GardenInput';
import type { GardenReopenInput } from '../../src/bindings/GardenReopenInput';
import type { GardensListInput } from '../../src/bindings/GardensListInput';
import type { PartnerVentureCard } from '../../src/bindings/PartnerVentureCard';
import { createGardenBooks, e40Closed, e40Garden, e40History, gardenRow } from './gardens-data';
import { paisaForTest } from './paisa';
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

const books = createGardenBooks(
  flag('empty')
    ? { gardens: [], history: [] }
    : flag('closed')
      ? { gardens: [e40Closed()], history: e40History(true) }
      : flag('fresh')
        ? { gardens: [gardenRow()], history: [] }
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
  gardens_arrivals_list: () => arrivalsReal.ARRIVALS_IN_STOCK,
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
  gardens_arrival_entry_save: (input) => ({
    rows: (input as ArrivalEntryInput).lines.map((_, index) => ({
      ...arrivalsReal.ARRIVAL_SAVE.row,
      id: index + 1,
      number: index + 1,
    })),
    warnings: [],
  }),
  gardens_arrival_cancel: () => arrivalsReal.ARRIVAL_CANCEL,
  gardens_arrival_set_sell_date: () => arrivalsReal.ARRIVAL_SET_SELL_DATE,
  gardens_arrival_write_off: () => arrivalsReal.ARRIVAL_WRITE_OFF,
  gardens_arrival_write_off_undo: () => arrivalsReal.ARRIVAL_WRITE_OFF_UNDO,
  partners_overview: () => overview(),
  partners_list: () => THREE_PARTNERS,
};
