// Plan 90 (Umar, 21-11-2026): goods for another shop on a garden or a trip. The three small rules a sale in
// another shop uses (PRD FR-12.41, FR-12.32, FR-12.36), and the words of the screen in Urdu and in English.
// The screens themselves are tested in gardens-arrivals and trips-real-answers (the sale) and trips-trip (the list).

import { describe, expect, it } from 'vitest';
import {
  offeredForOtherShop,
  qtyAboveOther,
  shopAfterPicking,
} from '../../src/features/purchases/other-shop';
import { LANGUAGES } from '../../src/i18n';
import { message, visible } from '../helpers/render';

const line = (id: number, left: number, shop: string | null = null) => ({
  id,
  qty_other_left: left,
  shop_name: shop,
});

describe('which lines a sale in another shop offers', () => {
  it('only the lines with goods left for another shop, in the order they came', () => {
    const lines = [line(1, 10), line(2, 0), line(3, 4)];
    expect(offeredForOtherShop(lines, '').map((one) => one.id)).toEqual([1, 3]);
  });

  it('the line already chosen stays even with nothing left (an entry open for edit)', () => {
    const lines = [line(1, 10), line(2, 0)];
    expect(offeredForOtherShop(lines, '2').map((one) => one.id)).toEqual([1, 2]);
  });

  it('no line has goods for another shop: nothing is offered', () => {
    expect(offeredForOtherShop([line(1, 0), line(2, 0)], '')).toEqual([]);
  });
});

describe('the shop name after a line is picked', () => {
  it('an empty box takes the name of the line', () => {
    expect(shopAfterPicking('', null, 'Haji Shop')).toBe('Haji Shop');
  });

  it('a box that only holds the name of the line picked before takes the new line’s name, or none', () => {
    expect(shopAfterPicking('Haji Shop', 'Haji Shop', 'Noor Shop')).toBe('Noor Shop');
    expect(shopAfterPicking('Haji Shop', 'Haji Shop', null)).toBe('');
  });

  it('a name typed by hand is never taken away', () => {
    expect(shopAfterPicking('My Shop', 'Haji Shop', 'Noor Shop')).toBe('My Shop');
    expect(shopAfterPicking('My Shop', null, null)).toBe('My Shop');
  });
});

describe('more نگ than the line has for another shop', () => {
  it('a whole number is compared with what is left', () => {
    expect(qtyAboveOther('40', 40, null)).toBe(false);
    expect(qtyAboveOther('41', 40, null)).toBe(true);
    expect(qtyAboveOther('1,000', 40, null)).toBe(true);
  });

  it('a sum is left to the backend: its preview says how many would come from our shop', () => {
    expect(qtyAboveOther('30+14', 40, null)).toBe(false);
    expect(qtyAboveOther('30+14', 40, 4)).toBe(true);
    expect(qtyAboveOther('30+14', 40, 0)).toBe(false);
  });

  it('nothing typed, or a half-typed sum, is not above', () => {
    expect(qtyAboveOther('', 40, undefined)).toBe(false);
    expect(qtyAboveOther('30+', 40, null)).toBe(false);
  });
});

describe.each(LANGUAGES)('the words of goods for another shop (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  it('the column says whose نگ they are', () => {
    expect(m('goods-line-col-other-qty')).toBe(
      language === 'ur' ? 'دوسری دکان کے نگ' : 'Nag for Another Shop',
    );
  });

  it('the two help lines of the item grids quote the same column words', () => {
    expect(m('goods-lines-help-purchase')).toContain(m('goods-line-col-other-qty'));
    expect(m('goods-lines-help-arrival')).toContain(m('goods-line-col-other-qty'));
  });

  it('the refusal names the number that is left', () => {
    expect(visible(m('purchases-qty-above-other', { remaining: 8 }))).toContain('8');
  });

  it('the option names only what is left for another shop', () => {
    const garden = m('gardens-sale-arrival-option', {
      number: 'A-2',
      item: 'x',
      packing: 'y',
      other_left: 10,
    });
    expect(garden).toContain('10');
    const trip = m('trips-sale-purchase-option', {
      number: 'P-2',
      item: 'x',
      packing: 'y',
      other_left: 10,
    });
    expect(trip).toContain('10');
  });
});
