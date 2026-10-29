// The pages of the اگراہی فہرست joined into one list (plan 38 B1): a group that continues on the next
// page gets its buyers added, the group total and the grand total are the backend's, and nothing is
// added up here.

import { describe, expect, it } from 'vitest';
import type { CollectionBuyer } from '../../src/bindings/CollectionBuyer';
import type { CollectionList } from '../../src/bindings/CollectionList';
import { joinCollectionPages } from '../../src/features/vouchers/collection-pages';
import { money } from '../helpers/accounts';

const ASLAM = { id: 81, code: '0001', name_ur: 'اسلم', name_en: 'Aslam' };
const KOT = { id: 91, code: '0001', name_ur: 'کوٹ', name_en: 'Kot' };

function buyer(id: number): CollectionBuyer {
  return {
    account: {
      id,
      code: String(id),
      name_ur: `خریدار ${String(id)}`,
      name_en: `Buyer ${String(id)}`,
    },
    phone: '',
    dues_paisa: money(id),
    oldest_bill_date: null,
    oldest_days: null,
  };
}

function page(offset: number, groups: CollectionList['groups']): CollectionList {
  const count = groups.reduce((sum, group) => sum + group.buyers.length, 0);
  return {
    date: '2026-10-10',
    weekday: 5,
    groups,
    group_totals: [],
    total_count: 5,
    offset,
    limit: count,
    total_paisa: money(15),
  };
}

describe('joinCollectionPages', () => {
  it('is nothing for no page', () => {
    expect(joinCollectionPages([])).toBeUndefined();
  });

  it('adds the buyers of a group that continues on the next page, in order, with its whole total', () => {
    const first = page(0, [
      {
        collector: ASLAM,
        city: KOT,
        buyers: [buyer(1), buyer(2), buyer(3)],
        total_paisa: money(15),
      },
    ]);
    const second = page(3, [
      { collector: ASLAM, city: KOT, buyers: [buyer(4), buyer(5)], total_paisa: money(15) },
    ]);
    const joined = joinCollectionPages([first, second]);
    expect(joined?.groups).toHaveLength(1);
    expect(joined?.groups[0]?.buyers.map((row) => row.account.id)).toEqual([1, 2, 3, 4, 5]);
    // The group total and the grand total are the backend's, not added here.
    expect(joined?.groups[0]?.total_paisa).toEqual(money(15));
    expect(joined?.total_paisa).toEqual(money(15));
    expect(joined?.total_count).toBe(5);
  });

  it('keeps a different collector or city as its own group', () => {
    const first = page(0, [
      { collector: ASLAM, city: KOT, buyers: [buyer(1)], total_paisa: money(1) },
    ]);
    const second = page(1, [
      { collector: null, city: KOT, buyers: [buyer(2)], total_paisa: money(2) },
    ]);
    const joined = joinCollectionPages([first, second]);
    expect(joined?.groups.map((group) => group.collector?.id ?? null)).toEqual([81, null]);
  });
});
