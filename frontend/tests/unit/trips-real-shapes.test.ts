// The canned data of the trip tests has the shape of the backend's real answers (plan 16b, T4a).
// The backend wrote each real answer in its own test (tests/fixtures/*.json, copied, never edited).
// This compares the keys and kinds at every level, so a test that passes on canned data cannot hide
// a field the real backend does not send, or one it sends and the canned data forgot. Then it checks
// that the screens build the request the backend's own test sent for the same entry, and that the
// numbers the screens show are the backend's (E2E-34 and E2E-41).

import { describe, expect, it } from 'vitest';
import {
  buildEntryCost,
  buildEntryPurchase,
  buildEntrySale,
  buildTrip,
  newEntryDraft,
  newTripDraft,
} from '../../src/features/trips/draft';
import type { UiError } from '../../src/bindings/UiError';
import { errorMessageId, messageVars } from '../../src/shared/api';
import { LANGUAGES } from '../../src/i18n';
import { message } from '../helpers/render';
import * as arrivals from '../helpers/arrivals-real';
import * as real from '../helpers/trips-real';
import {
  BOX,
  createTripBooks,
  e34Purchase,
  e34Trip,
  ORANGE,
  tripPartners,
  tripRow,
} from '../helpers/trips-data';

type Shape = string | null | Shape[] | { [key: string]: Shape };

/** The keys and kinds of a value at every level; a list is shown by its first element. */
function shapeOf(value: unknown): Shape {
  if (value === null || value === undefined) return null;
  if (Array.isArray(value)) return value.length === 0 ? [] : [shapeOf(value[0])];
  if (typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, item]) => [key, shapeOf(item)]),
    );
  }
  return typeof value;
}

/** The first difference between two shapes, or null. A null or an empty list can hold anything. */
function firstDifference(a: Shape, b: Shape, path = ''): string | null {
  if (a === null || b === null) return null;
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b)) return `${path}: list against not a list`;
    const [x, y] = [a[0], b[0]];
    return x === undefined || y === undefined ? null : firstDifference(x, y, `${path}[]`);
  }
  if (typeof a === 'string' || typeof b === 'string') {
    return a === b ? null : `${path}: ${JSON.stringify(a)} against ${JSON.stringify(b)}`;
  }
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const key of [...keys].sort()) {
    const [x, y] = [a[key], b[key]];
    if (x === undefined) return `${path}.${key}: the real answer has no such field`;
    if (y === undefined) return `${path}.${key}: the canned data has no such field`;
    const inner = firstDifference(x, y, `${path}.${key}`);
    if (inner !== null) return inner;
  }
  return null;
}

const same = (realValue: unknown, canned: unknown) =>
  firstDifference(shapeOf(realValue), shapeOf(canned));

describe('the comparison itself', () => {
  it('sees a missing field, an extra field and a different kind', () => {
    expect(same({ a: 1, b: 2 }, { a: 1 })).toContain('.b');
    expect(same({ a: 1 }, { a: 1, c: 2 })).toContain('.c');
    expect(same({ a: 1 }, { a: 'x' })).toContain('.a');
    expect(same({ a: [{ k: 1 }] }, { a: [{ j: 1 }] })).not.toBeNull();
    expect(same({ a: null }, { a: 1 })).toBeNull();
  });
});

const TRIP_GET = { id: 1, entries_offset: 0, entries_limit: 0 };
const NO_LIST = {
  from: null,
  to: null,
  status: null,
  partner_id: null,
  search: null,
  offset: 0,
  limit: 0,
};

describe('the canned data of the trip tests has the real shape', () => {
  it('the defaults, the list and the trip row', () => {
    const books = createTripBooks();
    expect(same(real.TRIP_DEFAULTS_FIRST, books.defaults())).toBeNull();
    expect(same(real.TRIP_LIST_ALL, books.list(NO_LIST))).toBeNull();
    expect(same(real.TRIP_LIST_ALL.rows[0], e34Trip())).toBeNull();
    expect(same(real.TRIP_CREATE_THREE, tripRow())).toBeNull();
    expect(same(real.TRIP_CREATE_POCKET, tripRow())).toBeNull();
    expect(same(real.TRIPS_SUMMARY_OPEN, real.TRIPS_SUMMARY_OPEN)).toBeNull();
  });

  it('a trip with its purchases, entries and partner results', () => {
    const books = createTripBooks();
    const canned = books.get(TRIP_GET);
    for (const one of [
      real.TRIP_GET_OPEN,
      real.TRIP_GET_CLOSED,
      real.TRIP_GET_REOPENED,
      real.TRIP_GET_E41_CLOSED,
    ]) {
      expect(same(one, canned)).toBeNull();
    }
    expect(same(real.TRIP_GET_OPEN.purchases[0], canned.purchases[0])).toBeNull();
    expect(same(real.TRIP_GET_OPEN.partner_results[0], canned.partner_results[0])).toBeNull();
  });

  it('a purchase row, the list and the card', () => {
    const books = createTripBooks();
    expect(same(real.PURCHASE_GET_AFTER_SALES, e34Purchase())).toBeNull();
    expect(same(real.PURCHASES_LIST_ALL, books.purchaseList({} as never))).toBeNull();
  });

  it('a purchase preview and its save', () => {
    const books = createTripBooks();
    const input = real.PURCHASE_PREVIEW_E34_REQUEST;
    const filled = {
      ...input,
      item_id: ORANGE.id,
      packing_id: BOX.id,
      parts: [],
      qty: input.qty === '' ? '20' : input.qty,
      rate: input.rate === '' ? '500' : input.rate,
    };
    expect(same(real.PURCHASE_PREVIEW_E34, books.purchasePreview(filled))).toBeNull();
    expect(same(real.PURCHASE_SAVE_E34, books.purchaseSave(filled))).toBeNull();
    expect(same(real.PURCHASE_SAVE_E41, books.purchaseSave(filled))).toBeNull();
  });

  it('a trip cost preview and its save', () => {
    const books = createTripBooks();
    const input = { ...real.COST_PREVIEW_TRUCK_REQUEST, parts: [] };
    expect(same(real.COST_PREVIEW_TRUCK, books.costPreview(input))).toBeNull();
    expect(same(real.COST_SAVE_TRUCK, books.costSave(input))).toBeNull();
  });

  it('a sale in another shop: preview, save and cancel', () => {
    const books = createTripBooks();
    expect(same(real.OTHER_PREVIEW, books.salePreview(real.OTHER_PREVIEW_REQUEST))).toBeNull();
    expect(same(real.OTHER_SAVE, books.saleSave(real.OTHER_SAVE_REQUEST))).toBeNull();
    expect(same(real.OTHER_CANCEL, books.saleCancel())).toBeNull();
  });

  it('the cancels, the undo of a write-off and the defaults taken from the last trip', () => {
    const books = createTripBooks();
    expect(same(real.PURCHASE_CANCEL, books.purchaseCancel())).toBeNull();
    expect(same(real.COST_CANCEL, books.costCancel())).toBeNull();
    expect(same(real.WRITE_OFF_UNDO, books.writeOffUndo())).toBeNull();
    expect(same(real.TRIP_DEFAULTS_FROM_LAST, books.defaults())).toBeNull();
    expect(real.TRIP_DEFAULTS_FROM_LAST.from_last_trip).toBe(true);
    expect(real.TRIP_DEFAULTS_FIRST.from_last_trip).toBe(false);
  });

  it('the purchase previews of cash below zero, of a pocket part and of a missing quantity', () => {
    const books = createTripBooks();
    const filled = { ...real.PURCHASE_PREVIEW_E34_REQUEST, item_id: ORANGE.id, packing_id: BOX.id };
    for (const one of [real.PURCHASE_PREVIEW_CASH_BELOW_ZERO, real.PURCHASE_PREVIEW_POCKET]) {
      expect(same(one, books.purchasePreview(filled))).toBeNull();
    }
    // A missing quantity: no numbers, and the problem points at the quantity.
    expect(real.PURCHASE_PREVIEW_NO_QTY.numbers).toBeNull();
    expect(real.PURCHASE_PREVIEW_NO_QTY.errors[0]).toMatchObject({
      code: 'ERR_REQUIRED',
      field: 'qty',
    });
    expect(same(real.PURCHASE_PREVIEW_NO_QTY.cash, real.PURCHASE_PREVIEW_E34.cash)).toBeNull();
  });

  it('the list of cancelled purchases has the rows of any list', () => {
    expect(real.PURCHASES_LIST_CANCELLED_REQUEST.state).toBe('cancelled');
    expect(real.PURCHASES_LIST_CANCELLED.rows[0]?.status).toBe('cancelled');
    expect(same(real.PURCHASES_LIST_CANCELLED, real.PURCHASES_LIST_ALL)).toBeNull();
  });

  it('a write-off', () => {
    const books = createTripBooks();
    expect(same(real.WRITE_OFF_LAST_2, books.writeOff())).toBeNull();
  });

  it('every refusal has the shape of the others', () => {
    for (const one of [
      real.TRIP_CREATE_SHARES_NOT_100,
      real.COST_CANCEL_HAS_SALES,
      real.PURCHASE_SAVE_TRIP_CLOSED,
      real.WRITE_OFF_TOO_MANY,
    ]) {
      expect(one.ok).toBe(false);
      expect(one.errors.length).toBeGreaterThan(0);
      expect(typeof one.ref).toBe('string');
      // `values` holds different keys for each code, so it is left out of the comparison. (The
      // cash question points at no field: a null can hold anything.)
      const plain = (error: UiError) => ({
        ...error,
        errors: error.errors.map((e) => ({ ...e, values: {} })),
      });
      expect(same(plain(real.TRIP_CREATE_SHARES_NOT_100), plain(one))).toBeNull();
    }
  });
});

describe('what the backend sends is what the screens show', () => {
  it('E2E-34: the purchase costs 10,400 and the shop parts are 4,160 / 3,120 / 3,120', () => {
    const numbers = real.PURCHASE_PREVIEW_E34.numbers;
    expect(numbers).not.toBeNull();
    expect(numbers?.parts.parts.map((part) => part.amount_paisa)).toEqual([
      416_000, 312_000, 312_000,
    ]);
    expect(numbers?.paid_now).toBe(1_040_000);
    expect(numbers?.total_cost).toBe(1_040_000);
    expect(numbers?.avg_cost).toBe(52_000);
    expect(numbers?.parts.shop_total_paisa).toBe(1_040_000);
    expect(numbers?.parts.pocket_total_paisa).toBe(0);
    // The opening position has 50,000 in cash; the shop parts take 10,400: 39,600 are left.
    expect(real.PURCHASE_PREVIEW_E34.cash.cash_before_paisa).toBe(5_000_000);
    expect(real.PURCHASE_PREVIEW_E34.cash.cash_after_paisa).toBe(3_960_000);
  });

  it('every part has a working with the rule id the screen can show', () => {
    const numbers = real.PURCHASE_PREVIEW_E34.numbers;
    for (const part of numbers?.parts.parts ?? []) {
      expect(part.working.message_id).toBe('working-venture-part');
    }
    expect(numbers?.workings.paid_now.message_id).toBe('working-purchase-paid-now');
    expect(numbers?.workings.total_cost.message_id).toBe('working-purchase-total-cost');
    expect(numbers?.workings.avg_cost.message_id).toBe('working-purchase-avg-cost');
  });

  it('a closed trip is refused with the code the screen turns into words', () => {
    expect(real.PURCHASE_SAVE_TRIP_CLOSED.errors[0]?.code).toBe('ERR_TRIP_CLOSED');
    expect(real.TRIP_CREATE_SHARES_NOT_100.errors[0]?.code).toBe('ERR_SHARES_NOT_100');
    expect(real.COST_CANCEL_HAS_SALES.errors[0]?.code).toBe('ERR_TRIP_COST_HAS_SALES');
    expect(real.WRITE_OFF_TOO_MANY.errors[0]?.code).toBe('ERR_WRITE_OFF_EXCEEDS_BALANCE');
  });
});

describe('every refusal the backend really sent reads as words in both languages', () => {
  const refusals: [string, UiError][] = [
    ['trip create, shares 90%', real.TRIP_CREATE_SHARES_NOT_100],
    ['trip cost cancel after sales', real.COST_CANCEL_HAS_SALES],
    ['purchase on a closed trip', real.PURCHASE_SAVE_TRIP_CLOSED],
    ['write off more than is left', real.WRITE_OFF_TOO_MANY],
    ['purchase that takes cash below zero', real.PURCHASE_SAVE_CASH_NOT_CONFIRMED],
  ];

  it.each(LANGUAGES)('%s: no placeholder is left open and the real numbers show', (language) => {
    const texts = refusals.map(([, error]) => {
      const item = error.errors[0];
      if (item === undefined) throw new Error('refusal without an error');
      return message(
        language,
        errorMessageId(item.code),
        messageVars(item.values, 'international'),
      );
    });
    for (const text of texts) {
      expect(text).not.toMatch(/[{}$]/);
      expect(text).not.toContain('undefined');
    }
    // 9000 bps is 90%; 80 and 0 and 2 are the real values of the other refusals.
    expect(texts[0]).toContain('90');
    expect(texts[1]).toContain('80');
    expect(texts[3]).toContain('2');
    // Cash after the entry: −5,000,000 paisa is written −50,000 rupees, never as paisa.
    expect(texts[4]).toContain('50,000');
  });
});

describe('the screens build the request the backend tested', () => {
  const partners = tripPartners();

  /** The parts the screen sends are one row for every partner; the backend's test sent none. */
  const withoutParts = (input: object) =>
    Object.fromEntries(Object.entries(input).filter(([key]) => key !== 'parts'));

  it('a trip of three partners at 40, 30 and 30 from the shop', () => {
    const draft = newTripDraft({ partners, from_last_trip: false }, '');
    draft.date = '10-10-2026';
    draft.market = 'Lahore';
    draft.sentPerson = 'Aslam';
    expect(buildTrip(draft)).toEqual(real.TRIP_CREATE_THREE_REQUEST);
  });

  it('a trip with a partner paying from his own pocket', () => {
    const request = real.TRIP_CREATE_POCKET_REQUEST;
    const draft = newTripDraft({ partners, from_last_trip: false }, '');
    draft.date = request.date;
    draft.market = request.market ?? '';
    draft.sentPerson = request.sent_person ?? '';
    // The shares and sources the backend's test sent, typed into the rows as a user would.
    draft.partners = draft.partners.map((row) => {
      const sent = request.partners.find((one) => one.partner_id === row.partner.id);
      return {
        ...row,
        share: sent?.share === '0' ? '' : (sent?.share ?? ''),
        source: sent?.source ?? 'shop',
      };
    });
    expect(buildTrip(draft)).toEqual(request);
  });

  it('a purchase: the fields as typed, the item and packing by id', () => {
    const request = real.PURCHASE_SAVE_E34_REQUEST;
    const draft = newEntryDraft('purchase', '');
    draft.date = request.date;
    draft.purchase.supplier = request.supplier ?? '';
    draft.purchase.item = { id: request.item_id ?? 0, code: '', name_ur: '', name_en: '' };
    draft.purchase.packing = { id: request.packing_id ?? 0, code: '', name_ur: '', name_en: '' };
    draft.purchase.qty = request.qty;
    draft.purchase.rate = request.rate;
    draft.purchase.freight = request.freight ?? '';
    const sent = buildEntryPurchase(draft, request.trip_id ?? 0, partners, false);
    // The backend's file was written before the date to sell: a missing one is read as empty, the
    // day of the purchase (PRD FR-12.39), and the screen sends that as null.
    expect(withoutParts(sent)).toEqual({ ...withoutParts(request), sell_date: null });
    expect(sent.parts).toHaveLength(3);
  });

  it('a purchase with a date to sell: the day typed goes as typed', () => {
    const request = arrivals.PURCHASE_SAVE_WITH_SELL_DATE_REQUEST;
    const draft = newEntryDraft('purchase', '');
    draft.date = request.date;
    draft.purchase.item = { id: request.item_id ?? 0, code: '', name_ur: '', name_en: '' };
    draft.purchase.packing = { id: request.packing_id ?? 0, code: '', name_ur: '', name_en: '' };
    draft.purchase.qty = request.qty;
    draft.purchase.rate = request.rate;
    draft.purchase.sellDate = request.sell_date ?? '';
    const sent = buildEntryPurchase(draft, request.trip_id ?? 0, partners, false);
    expect(withoutParts(sent)).toEqual(withoutParts(request));
  });

  it('a trip cost: the amount stays as typed, commas and all', () => {
    const request = real.COST_SAVE_TRUCK_REQUEST;
    const draft = newEntryDraft('cost', '');
    draft.date = request.date;
    draft.amount = request.amount;
    draft.cost.reason = request.reason;
    const sent = buildEntryCost(draft, request.trip_id ?? 0, partners, false);
    expect(withoutParts(sent)).toEqual(withoutParts(request));
  });

  it('a sale in another shop', () => {
    const request = real.OTHER_SAVE_REQUEST;
    const draft = newEntryDraft('other_sale', '');
    draft.date = request.date;
    draft.amount = request.amount;
    draft.sale.shop = request.shop ?? '';
    draft.sale.purchaseId = String(request.purchase_id ?? '');
    draft.sale.qty = request.qty;
    expect(buildEntrySale(draft, request.trip_id ?? 0)).toEqual(request);
  });
});
