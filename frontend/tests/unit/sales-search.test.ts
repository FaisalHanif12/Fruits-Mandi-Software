// The address of the sales page (plan 83, PRD FR-9.9): what it reads, what it drops, and the address the page writes
// back. The plain address is the current month of every party, item and kind of goods (what the tile opens); a party
// needs its side; an item and the goods are kept; only the values of the kind on screen are kept; a word the page
// does not know is dropped.

import { describe, expect, it } from 'vitest';
import {
  salesFilterInput,
  salesInput,
  salesNeedsParty,
} from '../../src/features/dashboard/sales-hooks';
import {
  isPlainSales,
  parseSalesSearch,
  salesFiltersOfSearch,
  searchOfSalesFilters,
} from '../../src/features/dashboard/sales-search';

const RAHIM = { id: 19, name_ur: 'رحیم', name_en: 'Rahim', code: '0010' };
const BANANA = { id: 4, name_ur: 'کیلا', name_en: 'Banana', code: '0040' };

describe('reading the address of the sales page', () => {
  it('the plain address is the current month of everything: what the tile opens', () => {
    const filters = salesFiltersOfSearch(parseSalesSearch({}));
    expect(filters).toMatchObject({
      kind: 'month',
      month: null,
      side: null,
      party: null,
      item: null,
      goods: 'all',
    });
    expect(isPlainSales(filters)).toBe(true);
  });

  it('reads the period of each kind, a party with his side, an item and the goods', () => {
    const search = parseSalesSearch({
      period: 'year',
      year: 2026,
      side: 'seller',
      party: RAHIM,
      item: BANANA,
      goods: 'trip',
    });
    expect(search).toEqual({
      period: 'year',
      year: '2026',
      side: 'seller',
      party: RAHIM,
      item: BANANA,
      goods: 'trip',
    });
    expect(isPlainSales(salesFiltersOfSearch(search))).toBe(false);
  });

  it('a party needs his side, and a side with no party is kept', () => {
    expect(parseSalesSearch({ party: RAHIM })).toEqual({});
    const search = parseSalesSearch({ side: 'buyer' });
    expect(search).toEqual({ side: 'buyer' });
    expect(salesNeedsParty(salesFiltersOfSearch(search))).toBe(true);
  });

  it('drops a word the page does not know, a date that is not real and goods that are all goods', () => {
    expect(
      parseSalesSearch({
        period: 'decade',
        month: '2026-13',
        date: '2026-02-31',
        side: 'partner',
        goods: 'everything',
        item: 'banana',
      }),
    ).toEqual({});
    expect(parseSalesSearch({ goods: 'all' })).toEqual({});
  });
});

describe('writing the address of the sales page', () => {
  it('a plain page writes no key at all', () => {
    expect(searchOfSalesFilters(salesFiltersOfSearch({}))).toEqual({});
  });

  it('keeps only the values of the kind on screen, and the party only with his side', () => {
    const filters = {
      ...salesFiltersOfSearch(parseSalesSearch({ period: 'day', date: '2026-10-10' })),
      month: '2026-09',
      year: '2025',
      item: BANANA,
      goods: 'garden' as const,
      party: RAHIM,
    };
    // No side: the party is not written, and the month and year of other kinds are dropped.
    expect(searchOfSalesFilters(filters)).toEqual({
      period: 'day',
      date: '2026-10-10',
      item: BANANA,
      goods: 'garden',
    });
    expect(searchOfSalesFilters({ ...filters, side: 'buyer' })).toMatchObject({
      side: 'buyer',
      party: RAHIM,
    });
  });
});

describe('what the backend is asked', () => {
  it('a filter at its start is asked as nothing, so the plain page sends the plain request', () => {
    const filters = salesFiltersOfSearch({});
    expect(salesFilterInput(filters)).toEqual({});
    expect(salesInput(filters, '2026-10-10', null)).toEqual({ period: { kind: 'month' } });
  });

  it('sends the party with his side, the item id and the goods, and pages a range', () => {
    const filters = salesFiltersOfSearch(
      parseSalesSearch({
        period: 'range',
        from: '2026-10-01',
        to: '2026-10-05',
        side: 'seller',
        party: RAHIM,
        item: BANANA,
        goods: 'garden',
      }),
    );
    expect(salesInput(filters, '2026-10-10', { offset: 25, limit: 25 })).toEqual({
      period: { kind: 'range', from: '2026-10-01', to: '2026-10-05' },
      party: { side: 'seller', account_id: 19 },
      item_id: 4,
      goods: 'garden',
      offset: 25,
      limit: 25,
    });
  });

  it('waits for today when the period is the latest of a kind that needs it', () => {
    const filters = salesFiltersOfSearch(parseSalesSearch({ period: 'year' }));
    expect(salesInput(filters, null, null)).toBeNull();
    expect(salesInput(filters, '2026-10-10', null)).toEqual({
      period: { kind: 'year', year: 2026 },
    });
  });
});
