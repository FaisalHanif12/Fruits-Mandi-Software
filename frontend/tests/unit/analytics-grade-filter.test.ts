// The quality filter of analytics view A (PRD FR-12.42, plan 59): the backend's catalogue lists `grade` on
// view A only, so view A has the box, sends the quality only when one is chosen, and no other view ever
// sends a quality left over from view A. The quality is a filter to clear, comes back from a saved view,
// and is kept for the Owner like the other boxes.

import { describe, expect, it } from 'vitest';
import {
  EMPTY_ANALYTICS_FILTERS,
  filtersToWire,
  isUnfiltered,
  stateFromWire,
} from '../../src/features/analytics/filters';
import {
  analyticsFromRemembered,
  rememberedAnalytics,
} from '../../src/features/analytics/remember';
import { ANALYTICS_CATALOG } from '../helpers/analytics-real';

const takes = (view: string) => {
  const found = ANALYTICS_CATALOG.find((entry) => entry.id === view);
  if (found === undefined) throw new Error(`no view ${view}`);
  return found.filters;
};

describe('the quality filter of view A', () => {
  it('is listed on view A and on no other view', () => {
    expect(takes('A')).toContain('grade');
    for (const entry of ANALYTICS_CATALOG.filter((candidate) => candidate.id !== 'A')) {
      expect(entry.filters, entry.id).not.toContain('grade');
    }
  });

  it('is sent only when one is chosen, and never "all"', () => {
    expect(filtersToWire(EMPTY_ANALYTICS_FILTERS, takes('A'))).not.toHaveProperty('grade');
    expect(
      filtersToWire({ ...EMPTY_ANALYTICS_FILTERS, grade: 'all' }, takes('A')),
    ).not.toHaveProperty('grade');
    for (const grade of ['first', 'second', 'third'] as const) {
      expect(filtersToWire({ ...EMPTY_ANALYTICS_FILTERS, grade }, takes('A'))).toMatchObject({
        grade,
      });
    }
  });

  it('a view without the box never sends a quality left over from view A', () => {
    const state = { ...EMPTY_ANALYTICS_FILTERS, grade: 'second' as const };
    expect(filtersToWire(state, takes('B'))).not.toHaveProperty('grade');
    expect(filtersToWire(state, takes('G'))).not.toHaveProperty('grade');
  });

  it('is a filter to clear on view A only', () => {
    const state = { ...EMPTY_ANALYTICS_FILTERS, grade: 'third' as const };
    expect(isUnfiltered(EMPTY_ANALYTICS_FILTERS, takes('A'))).toBe(true);
    expect(isUnfiltered(state, takes('A'))).toBe(false);
    expect(isUnfiltered(state, takes('B'))).toBe(true);
  });

  it('comes back from a saved view into the box, and a view saved without one has none', () => {
    const saved = filtersToWire({ ...EMPTY_ANALYTICS_FILTERS, grade: 'first' }, takes('A'));
    expect(stateFromWire(saved).grade).toBe('first');
    expect(stateFromWire(filtersToWire(EMPTY_ANALYTICS_FILTERS, takes('A'))).grade).toBe('all');
  });

  it('is kept for the Owner and read back, and an unreadable one starts as "all"', () => {
    expect(rememberedAnalytics(EMPTY_ANALYTICS_FILTERS)).toBeNull();
    const kept = rememberedAnalytics({ ...EMPTY_ANALYTICS_FILTERS, grade: 'second' });
    expect(kept).toEqual({ grade: 'second' });
    expect(analyticsFromRemembered(JSON.parse(JSON.stringify(kept)))?.grade).toBe('second');
    expect(analyticsFromRemembered({ grade: 'fourth' })?.grade).toBe('all');
  });
});
