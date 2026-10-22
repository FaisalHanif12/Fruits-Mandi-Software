// Test data: the books of trips and purchases (plan 16b contract, PRD v1.7), in memory, with no
// test runner in it, so the development page can use it too. A test sets the command wrappers from
// it (tests/helpers/trips.ts); the development page /dev/trips answers from it. It answers the way
// a healthy backend answers, from the worked numbers of PRD 27.2 E2E-34 (a trip of three partners at 40, 30
// and 30, a purchase of 20 oranges for 10,400, 15 sold to Akram). The sums in a preview are this
// test double's, written in the fixture with the plan's numbers; the screen only shows what comes
// back and never works out an amount (CLAUDE.md rule 1). Every shape is the generated binding, so
// a change of the contract fails to compile here first.

import type { CashCheck } from '../../src/bindings/CashCheck';
import type { EntryPart } from '../../src/bindings/EntryPart';
import type { ErrorCode } from '../../src/bindings/ErrorCode';
import type { FieldError } from '../../src/bindings/FieldError';
import type { EntryParts } from '../../src/bindings/EntryParts';
import type { MoneySource } from '../../src/bindings/MoneySource';
import type { OtherSaleInput } from '../../src/bindings/OtherSaleInput';
import type { OwnLotPreview } from '../../src/bindings/OwnLotPreview';
import type { OwnStockRow } from '../../src/bindings/OwnStockRow';
import type { PurchaseInput } from '../../src/bindings/PurchaseInput';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { ShareAmount } from '../../src/bindings/ShareAmount';
import type { TripCostInput } from '../../src/bindings/TripCostInput';
import type { TripDefaults } from '../../src/bindings/TripDefaults';
import type { TripEntryRow } from '../../src/bindings/TripEntryRow';
import type { TripInput } from '../../src/bindings/TripInput';
import type { TripPartnerResult } from '../../src/bindings/TripPartnerResult';
import type { TripRow } from '../../src/bindings/TripRow';
import type { TripsList } from '../../src/bindings/TripsList';
import type { VenturePartnerRow } from '../../src/bindings/VenturePartnerRow';
import type { Working } from '../../src/bindings/Working';
import type { OtherSalePreview } from '../../src/bindings/OtherSalePreview';
import type { OtherSaleSaved } from '../../src/bindings/OtherSaleSaved';
import type { PurchaseCancelled } from '../../src/bindings/PurchaseCancelled';
import type { PurchasePreview } from '../../src/bindings/PurchasePreview';
import type { PurchaseSaved } from '../../src/bindings/PurchaseSaved';
import type { PurchaseWriteOffDone } from '../../src/bindings/PurchaseWriteOffDone';
import type { PurchaseWriteOffUndone } from '../../src/bindings/PurchaseWriteOffUndone';
import type { PurchasesList } from '../../src/bindings/PurchasesList';
import type { PurchasesListInput } from '../../src/bindings/PurchasesListInput';
import type { TripCostPreview } from '../../src/bindings/TripCostPreview';
import type { TripCostSaved } from '../../src/bindings/TripCostSaved';
import type { TripDetail } from '../../src/bindings/TripDetail';
import type { TripEntryCancelled } from '../../src/bindings/TripEntryCancelled';
import type { TripGetInput } from '../../src/bindings/TripGetInput';
import type { TripsListInput } from '../../src/bindings/TripsListInput';
import { CommandError } from '../../src/shared/api';
import { refusal, type RefusalItem } from './ui-error';

import type { WorkingArg } from '../../src/bindings/WorkingArg';
import type { PurchaseRow } from '../../src/features/purchases';
import { paisaForTest } from './paisa';

const money = (rupees: number): ReturnType<typeof paisaForTest> => paisaForTest(rupees * 100);

export const TRIP_ID = 1;

const person = (id: number, ur: string, en: string): RecordRef => ({
  id,
  code: '',
  name_ur: ur,
  name_en: en,
});
export const PARTNER_A = person(1, 'الف', 'Partner A');
export const PARTNER_B = person(2, 'ب', 'Partner B');
export const PARTNER_C = person(3, 'ج', 'Partner C');
export const ORANGE: RecordRef = { id: 61, code: '0061', name_ur: 'مالٹا', name_en: 'Orange' };
export const BOX: RecordRef = { id: 71, code: '0071', name_ur: 'پیٹی', name_en: 'Box' };

export const TRIP_REF = { id: TRIP_ID, number: 1, name_ur: 'سفر-1', name_en: 'Trip 1' };

/** A working the way the backend names it: a message and its values. */
export function working(id: string, args: readonly WorkingArg[]): Working {
  return { message_id: id, args: [...args], rule_id: null };
}
const moneyArg = (name: string, rupees: number): WorkingArg => ({
  name,
  value: { kind: 'money', paisa: money(rupees) },
});
const countArg = (name: string, value: number): WorkingArg => ({
  name,
  value: { kind: 'count', value },
});

/** The partners of a trip at 40, 30, 30, each with the source given. */
export function tripPartners(
  sources: readonly [MoneySource, MoneySource, MoneySource] = ['shop', 'shop', 'shop'],
): VenturePartnerRow[] {
  return [
    { partner: PARTNER_A, share_bps: 4000, source: sources[0] },
    { partner: PARTNER_B, share_bps: 3000, source: sources[1] },
    { partner: PARTNER_C, share_bps: 3000, source: sources[2] },
  ];
}

const SHARES = [4000, 3000, 3000] as const;
const PARTNERS = [PARTNER_A, PARTNER_B, PARTNER_C] as const;

function resultWorking(
  id: string,
  sales: number,
  other: number,
  sold: number,
  off: number,
  result: number,
): Working {
  return working(id, [
    moneyArg('sales', sales),
    moneyArg('other_shop', other),
    moneyArg('cost_sold', sold),
    moneyArg('cost_written_off', off),
    moneyArg('result', result),
  ]);
}

/** A new trip with nothing bought yet. */
export function tripRow(over: Partial<TripRow> = {}): TripRow {
  return {
    id: TRIP_ID,
    version: 1,
    number: 1,
    name_ur: 'سفر-1',
    name_en: 'Trip 1',
    entry_date: '2026-10-10',
    market: 'Lahore Mandi',
    sent_person: 'Aslam',
    note: null,
    status: 'open',
    closed_date: null,
    shares_locked: false,
    partners: tripPartners(),
    purchase_count: 0,
    qty_bought: 0,
    qty_sold: 0,
    qty_written_off: 0,
    qty_left: 0,
    cost_paisa: money(0),
    cost_left_paisa: money(0),
    sales_paisa: money(0),
    other_shop_paisa: money(0),
    cost_sold_paisa: money(0),
    cost_written_off_paisa: money(0),
    result_paisa: money(0),
    result_working: resultWorking('working-trip-result', 0, 0, 0, 0, 0),
    ...over,
  };
}

/** E2E-34 after the sale: 20 bought for 10,400, 15 sold, result 660, 5 left with cost 2,600. */
export function e34Trip(over: Partial<TripRow> = {}): TripRow {
  return tripRow({
    purchase_count: 1,
    qty_bought: 20,
    qty_sold: 15,
    qty_left: 5,
    cost_paisa: money(10_400),
    cost_left_paisa: money(2_600),
    sales_paisa: money(8_460),
    cost_sold_paisa: money(7_800),
    result_paisa: money(660),
    result_working: resultWorking('working-trip-result', 8_460, 0, 7_800, 0, 660),
    shares_locked: true,
    ...over,
  });
}

function entryPart(index: 0 | 1 | 2, rupees: number, source: MoneySource = 'shop'): EntryPart {
  return {
    partner: PARTNERS[index],
    share_bps: SHARES[index],
    source,
    amount_paisa: money(rupees),
    working: working('working-venture-part', [
      moneyArg('amount', 10_400),
      countArg('bps', SHARES[index]),
      moneyArg('result', rupees),
    ]),
  };
}

/** The parts of E2E-34 (4,160 / 3,120 / 3,120), each from the source given. */
export function e34Parts(
  sources: readonly [MoneySource, MoneySource, MoneySource] = ['shop', 'shop', 'shop'],
): EntryPart[] {
  return [
    entryPart(0, 4_160, sources[0]),
    entryPart(1, 3_120, sources[1]),
    entryPart(2, 3_120, sources[2]),
  ];
}

/**
 * The two sums of a payment's parts, from the whole rupees the fixture wrote them with (this test
 * double adds rupees it holds itself; the lint rule keeps tests from reading a Paisa).
 */
function partsTotals(
  parts: EntryPart[],
  rupees: readonly number[],
): { view: EntryParts; shop: number } {
  let shop = 0;
  let pocket = 0;
  parts.forEach((part, index) => {
    const amount = rupees[index] ?? 0;
    if (part.source === 'shop') shop += amount;
    else pocket += amount;
  });
  return {
    view: { parts, shop_total_paisa: money(shop), pocket_total_paisa: money(pocket) },
    shop,
  };
}

const E34_RUPEES = [4_160, 3_120, 3_120] as const;
const COST_RUPEES = [1_600, 1_200, 1_200] as const;

/** The purchase of E2E-34 as the list and the card show it after 15 of 20 were sold. */
export function e34Purchase(over: Partial<PurchaseRow> = {}): PurchaseRow {
  return {
    id: 7,
    version: 1,
    number: 1,
    trip: TRIP_REF,
    entry_date: '2026-10-10',
    status: 'saved',
    cancel_reason: null,
    supplier: 'Haji Nazir',
    item: ORANGE,
    packing: BOX,
    qty: 20,
    weight_grams: null,
    rate_paisa: money(500),
    value_paisa: money(10_000),
    freight_paisa: money(400),
    labour_paisa: money(0),
    other_cost_paisa: money(0),
    paid_now_paisa: money(10_400),
    trip_cost_paisa: money(0),
    total_cost_paisa: money(10_400),
    avg_cost_paisa: money(520),
    note: null,
    entry_id: 31,
    parts: e34Parts(),
    qty_sold: 15,
    qty_written_off: 0,
    qty_left: 5,
    cost_released_paisa: money(7_800),
    cost_left_paisa: money(2_600),
    sales_paisa: money(8_460),
    other_shop_paisa: money(0),
    cost_sold_paisa: money(7_800),
    cost_written_off_paisa: money(0),
    result_paisa: money(660),
    result_working: resultWorking('working-purchase-result', 8_460, 0, 7_800, 0, 660),
    days_waiting: 2,
    can_cancel: false,
    lots: [],
    write_offs: [],
    ...over,
  };
}

/** The partners' money in the trip of E2E-34 before the write-off (E2E-34b has −776 / −582 / −582 too). */
export function e34Results(): TripPartnerResult[] {
  const row = (index: 0 | 1 | 2, inShop: number, back: number, net: number): TripPartnerResult => ({
    partner: PARTNERS[index],
    share_bps: SHARES[index],
    money_in_shop_paisa: money(inShop),
    money_in_pocket_paisa: money(0),
    money_back_paisa: money(back),
    net_paisa: money(net),
    working: working('working-trip-partner-net', [
      moneyArg('in_shop', inShop),
      moneyArg('in_pocket', 0),
      moneyArg('back', back),
      moneyArg('result', net),
    ]),
  });
  return [row(0, 4_160, 3_384, -776), row(1, 3_120, 2_538, -582), row(2, 3_120, 2_538, -582)];
}

/** The history entry of the purchase of E2E-34. */
export function purchaseEntry(over: Partial<TripEntryRow> = {}): TripEntryRow {
  return {
    id: 11,
    version: 1,
    kind: 'purchase',
    number: 1,
    entry_date: '2026-10-10',
    amount_paisa: money(10_400),
    reason: null,
    shop_name: null,
    qty: null,
    purchase_id: 7,
    purchase_number: 1,
    cost_paisa: money(0),
    journal_entry_id: 31,
    cancelled: false,
    cancel_reason: null,
    parts: e34Parts(),
    credits: [],
    spread: [],
    can_cancel: false,
    ...over,
  };
}

/** A truck cost of 4,000 on the trip, spread over the one purchase. */
export function costEntry(over: Partial<TripEntryRow> = {}): TripEntryRow {
  return {
    ...purchaseEntry(),
    id: 12,
    kind: 'cost',
    number: 1,
    amount_paisa: money(4_000),
    reason: 'Truck',
    purchase_id: null,
    purchase_number: null,
    journal_entry_id: 32,
    parts: [entryPart(0, 1_600), entryPart(1, 1_200), entryPart(2, 1_200)],
    spread: [
      {
        purchase_id: 7,
        purchase_number: 1,
        item: ORANGE,
        qty: 20,
        amount_paisa: money(4_000),
        working: working('working-trip-cost-spread', [
          moneyArg('cost', 4_000),
          countArg('qty', 20),
          countArg('total_qty', 20),
          moneyArg('result', 4_000),
        ]),
      },
    ],
    can_cancel: true,
    ...over,
  };
}

/** Each trip partner's part of 8,460 (E2E-34: 3,384 / 2,538 / 2,538), as a list of credits. */
export function e34Credits(): ShareAmount[] {
  return [3_384, 2_538, 2_538].map((rupees, index) => {
    const place = index as 0 | 1 | 2;
    return {
      partner: PARTNERS[place],
      share_bps: SHARES[place],
      amount_paisa: money(rupees),
      working: working('working-venture-credit', [
        moneyArg('amount', 8_460),
        countArg('bps', SHARES[place]),
        moneyArg('result', rupees),
      ]),
    };
  });
}

/**
 * What the preview of a lot of trip goods says about the lot (E2E-34): 15 نگ at cost 7,800 from the
 * purchase, the seller credit 8,460 shared 3,384 / 2,538 / 2,538, and the result. The working
 * names are the ones the backend sends (`working-own-lot-cost`, `working-own-lot-result`).
 */
export function e34OwnLotPart(resultRupees = 660, purchaseId = 7): OwnLotPreview {
  return {
    purchase_id: purchaseId,
    purchase_number: 1,
    trip: TRIP_REF,
    cost_paisa: money(7_800),
    cost_working: working('working-own-lot-cost', [countArg('qty', 15), moneyArg('result', 7_800)]),
    credit_paisa: money(8_460),
    result_paisa: money(resultRupees),
    result_working: working('working-own-lot-result', [
      moneyArg('credit', 8_460),
      moneyArg('cost', 7_800),
      moneyArg('result', resultRupees),
    ]),
    credits: e34Credits(),
  };
}

/** The purchase of E2E-34 as a row of باقی مال: 20 bought, 15 sold, 5 left with cost 2,600. */
export function e34OwnStockRow(): OwnStockRow {
  return {
    purchase_id: 7,
    number: 1,
    received_date: '2026-10-10',
    trip: TRIP_REF,
    supplier: 'Haji Nazir',
    item: ORANGE,
    packing: BOX,
    qty_bought: 20,
    qty_sold: 15,
    qty_written_off: 0,
    balance: 5,
    total_cost_paisa: money(10_400),
    cost_left_paisa: money(2_600),
    days_waiting: 2,
    write_offs: [],
  };
}

export interface TripStore {
  trips: TripRow[];
  purchases: PurchaseRow[];
  entries: TripEntryRow[];
  results: TripPartnerResult[];
  /** The cash in hand before an entry, in rupees (50,000 in the opening position). */
  cashBefore: number;
  /** Every input that was saved, as sent. */
  saved: {
    trips: TripInput[];
    purchases: PurchaseInput[];
    costs: TripCostInput[];
    sales: OtherSaleInput[];
  };
  /** Set to refuse the next save of a kind with these problems. */
  refuse: {
    purchase?: RefusalItem[];
    cost?: RefusalItem[];
    sale?: RefusalItem[];
    trip?: RefusalItem[];
  };
}

export interface TripsOptions {
  trips?: TripRow[];
  purchases?: PurchaseRow[];
  entries?: TripEntryRow[];
  results?: TripPartnerResult[];
  cashBefore?: number;
  defaults?: TripDefaults;
}

/** The backend refusing: thrown by the books, so a caller can turn it into a rejected promise. */
function refuse(items: RefusalItem[]): never {
  throw new CommandError(refusal(items));
}

function cashOf(before: number, shopRupees: number): CashCheck {
  const after = before - shopRupees;
  return {
    cash_before_paisa: money(before),
    change_paisa: money(-shopRupees),
    cash_after_paisa: money(after),
    below_zero: after < 0,
    lowers_cash: shopRupees > 0,
    needs_confirm: after < 0 && shopRupees > 0,
  };
}

const problem = (code: ErrorCode, field: string): FieldError => ({ code, field, values: {} });

const sourcesOf = (
  parts: readonly { partner_id: number; source: MoneySource }[],
): [MoneySource, MoneySource, MoneySource] => [
  parts.find((part) => part.partner_id === 1)?.source ?? 'shop',
  parts.find((part) => part.partner_id === 2)?.source ?? 'shop',
  parts.find((part) => part.partner_id === 3)?.source ?? 'shop',
];

/**
 * The commands of trips and purchases as a healthy backend answers them, in memory. Every method
 * answers at once or throws the refusal (a `CommandError`); the test wrapper turns that into a
 * promise, and the development page's stand-in does the same.
 */
export function createTripBooks(options: TripsOptions = {}) {
  const store: TripStore = {
    trips: options.trips ?? [e34Trip()],
    purchases: options.purchases ?? [e34Purchase()],
    entries: options.entries ?? [purchaseEntry()],
    results: options.results ?? e34Results(),
    cashBefore: options.cashBefore ?? 50_000,
    saved: { trips: [], purchases: [], costs: [], sales: [] },
    refuse: {},
  };
  const defaults: TripDefaults = options.defaults ?? {
    partners: tripPartners(),
    from_last_trip: false,
  };

  return {
    store,
    defaults: (): TripDefaults => defaults,
    list: (input: TripsListInput): TripsList => {
      let rows = store.trips;
      if (input.status !== null) rows = rows.filter((row) => row.status === input.status);
      if (input.search !== null) {
        const needle = input.search.toLowerCase();
        rows = rows.filter((row) => (row.market ?? '').toLowerCase().includes(needle));
      }
      const limit = input.limit === 0 ? 100 : input.limit;
      return {
        rows: rows.slice(input.offset, input.offset + limit),
        total_count: rows.length,
        totals: {
          qty_left: 5,
          cost_left_paisa: money(2_600),
          cost_paisa: money(10_400),
          sales_paisa: money(8_460),
          result_paisa: money(660),
        },
      };
    },
    get: (input: TripGetInput): TripDetail => {
      const row = store.trips.find((trip) => trip.id === input.id);
      if (row === undefined) return refuse([{ code: 'ERR_NOT_FOUND', field: 'id' }]);
      const limit = input.entries_limit === 0 ? 100 : input.entries_limit;
      return {
        row,
        purchases: store.purchases.filter((purchase) => purchase.trip.id === row.id),
        entries: store.entries.slice(input.entries_offset, input.entries_offset + limit),
        entries_count: store.entries.length,
        partner_results: store.results,
      };
    },
    create: (input: TripInput): TripRow => {
      if (store.refuse.trip !== undefined) return refuse(store.refuse.trip);
      store.saved.trips.push(input);
      const row = tripRow({
        id: 50 + store.trips.length,
        number: store.trips.length + 1,
        market: input.market,
        sent_person: input.sent_person,
      });
      store.trips.push(row);
      return row;
    },

    purchasePreview: (input: PurchaseInput): PurchasePreview => {
      const errors: FieldError[] = [];
      if (input.item_id === null) errors.push(problem('ERR_REQUIRED', 'item_id'));
      if (input.packing_id === null) errors.push(problem('ERR_REQUIRED', 'packing_id'));
      if (input.qty === '') errors.push(problem('ERR_REQUIRED', 'qty'));
      if (input.rate === '') errors.push(problem('ERR_REQUIRED', 'rate'));
      if (errors.length > 0) {
        return { numbers: null, cash: cashOf(store.cashBefore, 0), errors, warnings: [] };
      }
      const { view: parts, shop } = partsTotals(e34Parts(sourcesOf(input.parts)), E34_RUPEES);
      return {
        numbers: {
          value: money(10_000),
          freight: money(400),
          labour: money(0),
          other_cost: money(0),
          paid_now: money(10_400),
          total_cost: money(10_400),
          avg_cost: money(520),
          parts,
          workings: {
            value: working('working-line-value', [
              countArg('qty', 20),
              moneyArg('rate', 500),
              moneyArg('result', 10_000),
            ]),
            paid_now: working('working-purchase-paid-now', [
              moneyArg('value', 10_000),
              moneyArg('freight', 400),
              moneyArg('labour', 0),
              moneyArg('other_cost', 0),
              moneyArg('result', 10_400),
            ]),
            total_cost: working('working-purchase-total-cost', [
              moneyArg('paid_now', 10_400),
              moneyArg('trip_cost', 0),
              moneyArg('result', 10_400),
            ]),
            avg_cost: working('working-purchase-avg-cost', [
              moneyArg('total', 10_400),
              countArg('qty', 20),
              moneyArg('result', 520),
            ]),
          },
        },
        cash: cashOf(store.cashBefore, shop),
        errors: [],
        warnings: [],
      };
    },
    purchaseSave: (input: PurchaseInput): PurchaseSaved => {
      if (store.refuse.purchase !== undefined) return refuse(store.refuse.purchase);
      const { shop } = partsTotals(e34Parts(sourcesOf(input.parts)), E34_RUPEES);
      if (cashOf(store.cashBefore, shop).needs_confirm && !input.confirm_cash_below_zero) {
        return refuse([
          { code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', field: 'confirm_cash_below_zero' },
        ]);
      }
      store.saved.purchases.push(input);
      return {
        id: 8,
        number: 2,
        version: 1,
        entry_id: 33,
        total_cost: money(10_400),
        preview: {
          numbers: null,
          cash: cashOf(store.cashBefore, shop),
          errors: [],
          warnings: [],
        },
      };
    },

    costPreview: (input: TripCostInput): TripCostPreview => {
      if (input.amount === '' || input.reason === '') {
        const errors = [
          ...(input.amount === '' ? [problem('ERR_REQUIRED', 'amount')] : []),
          ...(input.reason === '' ? [problem('ERR_REASON_REQUIRED', 'reason')] : []),
        ];
        return { numbers: null, cash: cashOf(store.cashBefore, 0), errors, warnings: [] };
      }
      const [a, b, c] = sourcesOf(input.parts);
      const { view: parts, shop } = partsTotals(
        [entryPart(0, 1_600, a), entryPart(1, 1_200, b), entryPart(2, 1_200, c)],
        COST_RUPEES,
      );
      return {
        numbers: {
          amount: money(4_000),
          parts,
          spread: [
            {
              purchase_id: 7,
              purchase_number: 1,
              item: ORANGE,
              qty: 20,
              amount_paisa: money(4_000),
              working: working('working-trip-cost-spread', [
                moneyArg('cost', 4_000),
                countArg('qty', 20),
                countArg('total_qty', 20),
                moneyArg('result', 4_000),
              ]),
            },
          ],
        },
        cash: cashOf(store.cashBefore, shop),
        errors: [],
        warnings: [],
      };
    },
    costSave: (input: TripCostInput): TripCostSaved => {
      if (store.refuse.cost !== undefined) return refuse(store.refuse.cost);
      store.saved.costs.push(input);
      return {
        id: 21,
        number: 1,
        version: 1,
        entry_id: 34,
        preview: { numbers: null, cash: cashOf(store.cashBefore, 0), errors: [], warnings: [] },
      };
    },

    salePreview: (input: OtherSaleInput): OtherSalePreview => {
      if (input.purchase_id === null || input.qty === '' || input.amount === '') {
        return { numbers: null, errors: [], warnings: [] };
      }
      return {
        numbers: {
          amount: money(8_460),
          qty: 15,
          cost_paisa: money(7_800),
          cost_working: working('working-cost-of-goods', [
            moneyArg('cost_left', 10_400),
            countArg('qty', 15),
            countArg('qty_left', 20),
            moneyArg('result', 7_800),
          ]),
          result_paisa: money(660),
          result_working: working('working-other-sale-result', [
            moneyArg('amount', 8_460),
            moneyArg('cost', 7_800),
            moneyArg('result', 660),
          ]),
          credits: e34Credits(),
          qty_left_after: 5,
          cost_left_after_paisa: money(2_600),
        },
        errors: [],
        warnings: [],
      };
    },
    saleSave: (input: OtherSaleInput): OtherSaleSaved => {
      if (store.refuse.sale !== undefined) return refuse(store.refuse.sale);
      store.saved.sales.push(input);
      return {
        id: 22,
        number: 1,
        version: 1,
        entry_id: 35,
        trip_closed: false,
        preview: { numbers: null, errors: [], warnings: [] },
      };
    },
    costCancel: (): TripEntryCancelled => ({
      id: 12,
      number: 1,
      reversal_entry_id: 36,
      trip_reopened: false,
    }),
    saleCancel: (): TripEntryCancelled => ({
      id: 22,
      number: 1,
      reversal_entry_id: 37,
      trip_reopened: false,
    }),

    purchaseGet: (id: number): PurchaseRow => {
      const found = store.purchases.find((purchase) => purchase.id === id);
      return found ?? refuse([{ code: 'ERR_NOT_FOUND', field: 'id' }]);
    },
    purchaseList: (input: PurchasesListInput): PurchasesList => {
      let rows = store.purchases;
      if (input.state === 'in_stock') rows = rows.filter((row) => row.qty_left > 0);
      if (input.trip_id !== null) rows = rows.filter((row) => row.trip.id === input.trip_id);
      return {
        rows,
        total_count: rows.length,
        totals: {
          total_cost_paisa: money(10_400),
          cost_left_paisa: money(2_600),
          qty_left: 5,
          result_paisa: money(660),
        },
      };
    },
    purchaseCancel: (): PurchaseCancelled => ({ id: 7, number: 1, reversal_entry_id: 38 }),
    writeOff: (): PurchaseWriteOffDone => ({
      purchase_id: 7,
      move_id: 90,
      qty: 5,
      cost_paisa: money(2_600),
      working: working('working-cost-of-goods-last', [
        moneyArg('cost_left', 2_600),
        countArg('qty', 5),
        moneyArg('result', 2_600),
      ]),
      qty_written_off: 5,
      qty_left: 0,
      cost_left_paisa: money(0),
      trip_closed: true,
    }),
    writeOffUndo: (): PurchaseWriteOffUndone => ({
      purchase_id: 7,
      reversal_move_id: 91,
      qty_written_off: 0,
      qty_left: 5,
      cost_left_paisa: money(2_600),
      trip_reopened: true,
    }),
  };
}
