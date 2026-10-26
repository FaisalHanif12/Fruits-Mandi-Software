// The canned data of the fruit tests has the shape of the backend's real answers (plan 17d, T2).
// The backend wrote each real answer in its own test (tests/fixtures/fruit_*.json, copied, never
// edited). This compares the keys and kinds at every level, so a test that passes on canned data
// cannot hide a field the real backend does not send, or one it sends and the canned data forgot.
// Then it checks that the screen builds the request the backend's own test sent for the same entry
// (an entry typed the way the munshi types it), that every word the answers need exists in both
// languages, and that the numbers the answers carry are the numbers of E2E-42.

import { describe, expect, it } from 'vitest';
import {
  buildFruit,
  buildSaveRequest,
  chosenLotOf,
  newFruitDraft,
  valueOfRef,
} from '../../src/features/fruit/draft';
import type { FruitDraft } from '../../src/features/fruit/types';
import { LANGUAGES } from '../../src/i18n';
import { errorMessageId, messageVars } from '../../src/shared/api';
import { createFruitBooks, lotRow } from '../helpers/fruit-data';
import * as real from '../helpers/fruit-real';
import { message } from '../helpers/render';
import { same, sendsTheSame } from '../helpers/shapes';

/** The lot of the backend's test: lot 1 of بکری 1, رحیم's کیلا in پیٹی, 5 left. */
const LOT = lotRow({ lot_id: 1, bikri_id: 1 });
const books = () => createFruitBooks({ lots: [{ ...LOT }] });

const home = (rate: string, qty: string): FruitDraft => ({
  ...newFruitDraft('2026-10-10'),
  forWhom: '1',
  lot: chosenLotOf(LOT),
  qty,
  rate,
});
const guests = (qty: string, rate: string, item: number, weight = ''): FruitDraft => ({
  ...newFruitDraft('2026-10-10'),
  forWhom: 'guests',
  source: 'other_shop',
  agent: { id: 31, code: '0012', name_ur: 'حاجی نذیر', name_en: 'Haji Nazeer' },
  item: valueOfRef({ id: item, code: '', name_ur: '', name_en: '' }),
  packing: valueOfRef({ id: 1, code: '', name_ur: '', name_en: '' }),
  qty,
  weight,
  rate,
});

describe('the canned data of the fruit tests has the real shape', () => {
  it('the lots, the list and the row as it is read', () => {
    const fake = books();
    expect(same(real.FRUIT_LOTS_ALL, fake.lots({ search: null, limit: 0 }))).toBeNull();
    expect(same(real.FRUIT_LOTS_SEARCH, fake.lots({ search: 'rahim', limit: 0 }))).toBeNull();
    fake.save(real.FRUIT_SAVE_HOME_REQUEST);
    const request = { ...real.FRUIT_LIST_ALL_REQUEST, offset: 0, limit: 0 };
    expect(same(real.FRUIT_LIST_ALL, fake.list(request))).toBeNull();
    expect(same(real.FRUIT_LIST_ALL.rows[0], fake.list(request).rows[0])).toBeNull();
    expect(same(real.FRUIT_GET_HOME, fake.get({ id: 1 }))).toBeNull();
    expect(same(real.FRUIT_GET_HOME_CANCELLED, fake.get({ id: 1 }))).toBeNull();
  });

  it('the previews: home, guests, another shop, a weight item and the edit', () => {
    const fake = books();
    expect(same(real.FRUIT_PREVIEW_HOME, fake.preview(real.FRUIT_PREVIEW_HOME_REQUEST))).toBeNull();
    expect(
      same(real.FRUIT_PREVIEW_GUESTS, fake.preview(real.FRUIT_PREVIEW_GUESTS_REQUEST)),
    ).toBeNull();
    expect(
      same(
        real.FRUIT_PREVIEW_GUESTS_ONE_RUPEE,
        fake.preview(real.FRUIT_PREVIEW_GUESTS_ONE_RUPEE_REQUEST),
      ),
    ).toBeNull();
    expect(
      same(real.FRUIT_PREVIEW_WEIGHT_ITEM, fake.preview(real.FRUIT_PREVIEW_WEIGHT_ITEM_REQUEST)),
    ).toBeNull();
    fake.save(real.FRUIT_SAVE_HOME_REQUEST);
    expect(
      same(real.FRUIT_PREVIEW_EDIT_HOME, fake.preview(real.FRUIT_PREVIEW_EDIT_HOME_REQUEST)),
    ).toBeNull();
  });

  it('the refusals in a preview carry no numbers and point at their fields', () => {
    const fake = books();
    for (const [one, request] of [
      [real.FRUIT_PREVIEW_QTY_EXCEEDS_LOT, real.FRUIT_PREVIEW_QTY_EXCEEDS_LOT_REQUEST],
      [real.FRUIT_PREVIEW_RATE_ZERO, real.FRUIT_PREVIEW_RATE_ZERO_REQUEST],
    ] as const) {
      const canned = fake.preview(request);
      expect(same(one, canned)).toBeNull();
      expect(one.numbers).toBeNull();
      expect(one.errors.map((item) => item.field)).toEqual(canned.errors.map((item) => item.field));
    }
    expect(real.FRUIT_PREVIEW_WEIGHT_REQUIRED.numbers).toBeNull();
    expect(real.FRUIT_PREVIEW_WEIGHT_REQUIRED.errors[0]?.field).toBe('weight');
  });

  it('the saves, the edit and the cancel', () => {
    const fake = books();
    expect(same(real.FRUIT_SAVE_HOME, fake.save(real.FRUIT_SAVE_HOME_REQUEST))).toBeNull();
    expect(same(real.FRUIT_SAVE_GUESTS, fake.save(real.FRUIT_SAVE_GUESTS_REQUEST))).toBeNull();
    expect(same(real.FRUIT_EDIT_HOME, fake.edit(real.FRUIT_EDIT_HOME_REQUEST))).toBeNull();
    expect(same(real.FRUIT_CANCEL_HOME, fake.cancel(real.FRUIT_CANCEL_HOME_REQUEST))).toBeNull();
  });
});

describe('the screen builds the request the backend’s own test sent', () => {
  it('home fruit: the preview and the save', () => {
    const draft = home('580', '2');
    expect(sendsTheSame(buildFruit(draft, null), real.FRUIT_PREVIEW_HOME_REQUEST)).toBeNull();
    expect(sendsTheSame(buildFruit(draft, null), real.FRUIT_SAVE_HOME_REQUEST)).toBeNull();
  });

  it('guest fruit from another shop, typed the way the test typed it (a rate with its comma)', () => {
    const draft = guests('3', '1,000', 2);
    expect(sendsTheSame(buildFruit(draft, null), real.FRUIT_PREVIEW_GUESTS_REQUEST)).toBeNull();
    expect(sendsTheSame(buildFruit(draft, null), real.FRUIT_SAVE_GUESTS_REQUEST)).toBeNull();
  });

  it('one rupee, and an item sold by weight with its weight in kilograms', () => {
    expect(
      sendsTheSame(
        buildFruit(guests('1', '1', 2), null),
        real.FRUIT_PREVIEW_GUESTS_ONE_RUPEE_REQUEST,
      ),
    ).toBeNull();
    expect(
      sendsTheSame(
        buildFruit(guests('1', '100', 5, '12.345'), null),
        real.FRUIT_PREVIEW_WEIGHT_ITEM_REQUEST,
      ),
    ).toBeNull();
    // A weight item with no weight typed: the weight is left out, and the backend says so.
    expect(
      sendsTheSame(
        buildFruit(guests('1', '100', 5), null),
        real.FRUIT_PREVIEW_WEIGHT_REQUIRED_REQUEST,
      ),
    ).toBeNull();
  });

  it('an entry that is being edited sends its id with the preview, and the edit takes the version and the reason', () => {
    const draft = { ...home('580', '3'), reason: 'wrong quantity' };
    const editing = { id: 1, version: 2, number: 1 };
    expect(sendsTheSame(buildFruit(draft, 1), real.FRUIT_PREVIEW_EDIT_HOME_REQUEST)).toBeNull();
    const request = buildSaveRequest(draft, editing);
    expect(request.edit).toEqual({ id: 1, version: 2, reason: 'wrong quantity' });
    expect(
      sendsTheSame({ ...request.edit, entry: request.entry }, real.FRUIT_EDIT_HOME_REQUEST),
    ).toBeNull();
  });
});

describe('every word the real answers need is in both languages', () => {
  const refusals = [
    real.FRUIT_SAVE_QTY_EXCEEDS_LOT,
    real.FRUIT_SAVE_DATE_BEFORE_RECEIVED,
    real.FRUIT_CANCEL_AGAIN,
    real.BIKRI_CANCEL_HAS_FRUIT,
  ];
  const previews = [
    real.FRUIT_PREVIEW_QTY_EXCEEDS_LOT,
    real.FRUIT_PREVIEW_RATE_ZERO,
    real.FRUIT_PREVIEW_WEIGHT_REQUIRED,
  ];

  it.each(LANGUAGES)(
    '%s: each refusal reads as words, with its values and its dates',
    (language) => {
      const items = [
        ...refusals.flatMap((one) => one.errors),
        ...previews.flatMap((one) => one.errors),
      ];
      expect(items.length).toBeGreaterThan(5);
      for (const item of items) {
        const text = message(
          language,
          errorMessageId(item.code),
          messageVars(item.values, 'international'),
        );
        expect(text, item.code).not.toMatch(/[{}$]/);
        expect(text, item.code).not.toContain('undefined');
        expect(text, item.code).not.toMatch(/\d{4}-\d{2}-\d{2}/);
      }
      // What is left of the lot, what was taken from a بکری's lots, and the day the goods came.
      const exceeds = real.FRUIT_SAVE_QTY_EXCEEDS_LOT.errors[0];
      const hasFruit = real.BIKRI_CANCEL_HAS_FRUIT.errors[0];
      const received = real.FRUIT_SAVE_DATE_BEFORE_RECEIVED.errors[0];
      if (exceeds === undefined || hasFruit === undefined || received === undefined) {
        throw new Error('missing refusal');
      }
      expect(
        message(
          language,
          errorMessageId(exceeds.code),
          messageVars(exceeds.values, 'international'),
        ),
      ).toContain('5');
      expect(
        message(
          language,
          errorMessageId(hasFruit.code),
          messageVars(hasFruit.values, 'international'),
        ),
      ).toContain('3');
      expect(messageVars(received.values, 'international')['received_date']).toBe('11-10-2026');
    },
  );
});

describe('the numbers of E2E-42 in the real answers', () => {
  it('home fruit 2 × 580 = 1,160 comes off A, and the lot goes from 5 to 3', () => {
    const numbers = real.FRUIT_PREVIEW_HOME.numbers;
    expect(numbers?.value_paisa).toBe(116_000);
    expect(numbers?.parts).toHaveLength(1);
    expect(numbers?.parts[0]).toMatchObject({
      amount_paisa: 116_000,
      total_before_paisa: 142_400,
      total_after_paisa: 26_400,
    });
    expect(numbers?.lot).toEqual({ lot_id: 1, balance_before: 5, balance_after: 3 });
    expect(real.FRUIT_PREVIEW_HOME.rate_hint?.rate_paisa).toBe(58_000);
  });

  it('guest fruit of 3,000 is shared 1,000 / 1,000 / 1,000 and a rupee goes to the partner made first', () => {
    expect(real.FRUIT_PREVIEW_GUESTS.numbers?.parts.map((part) => part.amount_paisa)).toEqual([
      100_000, 100_000, 100_000,
    ]);
    expect(
      real.FRUIT_PREVIEW_GUESTS_ONE_RUPEE.numbers?.parts.map((part) => part.amount_paisa),
    ).toEqual([100]);
    expect(real.FRUIT_PREVIEW_GUESTS.numbers?.parts_working?.message_id).toBe(
      'working-fruit-guests',
    );
  });

  it('a weight item: 12.345 kg at 100 a kilo is 1,235, shared 413 / 411 / 411', () => {
    const numbers = real.FRUIT_PREVIEW_WEIGHT_ITEM.numbers;
    expect(numbers?.weight_grams).toBe(12_345);
    expect(numbers?.value_paisa).toBe(123_500);
    expect(numbers?.parts.map((part) => part.amount_paisa)).toEqual([41_300, 41_100, 41_100]);
    expect(numbers?.value_working.message_id).toBe('working-fruit-value-weight');
  });

  it('the list and its totals over the whole filter, and the statement of A', () => {
    expect(real.FRUIT_LIST_ALL.totals).toEqual({ qty: 5, value_paisa: 416_000 });
    expect(real.FRUIT_LIST_GUESTS_ONLY.totals).toEqual({ qty: 3, value_paisa: 300_000 });
    expect(real.FRUIT_LIST_CANCELLED.totals).toEqual({ qty: 0, value_paisa: 0 });
    const lines = real.PARTNERS_STATEMENT_FRUIT_ALIF.lines.filter(
      (line) => line.heading === 'fruit',
    );
    expect(lines.map((line) => line.amount_paisa)).toEqual([-116_000, -100_000]);
    expect(lines.map((line) => line.running_paisa)).toEqual([26_400, -73_600]);
  });
});
