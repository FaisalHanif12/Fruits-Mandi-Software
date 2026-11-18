// The trail of pages and the parents (plan 78 F1, PRD FR-7.18): what Back remembers, what a menu clears,
// and where each child page goes when the trail is empty.

import { beforeEach, describe, expect, it } from 'vitest';
import { backTarget, parentOf } from '../../src/shared/navigation';
import {
  MAX_TRAIL,
  cutTrailTo,
  pathOf,
  promiseNavigation,
  pushStop,
  resetTrail,
  takeNavigationMode,
  useTrailStore,
} from '../../src/shared/navigation/page-trail';

beforeEach(() => {
  resetTrail();
});

describe('the trail', () => {
  it('keeps the address of each page left, with its search, oldest first', () => {
    pushStop('/');
    pushStop('/commission?month=2026-10');
    expect(useTrailStore.getState().stops).toEqual(['/', '/commission?month=2026-10']);
  });

  it('keeps only the latest state of the same page twice in a row', () => {
    pushStop('/commission?month=2026-09');
    pushStop('/commission?month=2026-10');
    expect(useTrailStore.getState().stops).toEqual(['/commission?month=2026-10']);
  });

  it('forgets the oldest stops after the limit', () => {
    for (let index = 0; index < MAX_TRAIL + 5; index += 1) pushStop(`/p${String(index)}`);
    const { stops } = useTrailStore.getState();
    expect(stops).toHaveLength(MAX_TRAIL);
    expect(stops[0]).toBe('/p5');
  });

  it('is cut to the stops before the one the user goes to', () => {
    pushStop('/a');
    pushStop('/b');
    pushStop('/c');
    cutTrailTo(1);
    expect(useTrailStore.getState().stops).toEqual(['/a']);
  });

  it('reads the path of an address', () => {
    expect(pathOf('/partners/capital?x=1')).toBe('/partners/capital');
    expect(pathOf('/dues')).toBe('/dues');
    expect(pathOf('/a#b')).toBe('/a');
  });
});

describe('what a change of page is', () => {
  it('is a link unless a way in said otherwise', () => {
    expect(takeNavigationMode('/dues')).toBe('link');
  });

  it('is what was promised for that page, once', () => {
    promiseNavigation('fresh', '/bikri/book');
    expect(takeNavigationMode('/bikri/book')).toBe('fresh');
    expect(takeNavigationMode('/bikri/book')).toBe('link');
  });

  it('drops a promise when another page opens (the way was blocked, the user went elsewhere)', () => {
    promiseNavigation('back', '/dues');
    expect(takeNavigationMode('/rokar')).toBe('link');
    expect(takeNavigationMode('/dues')).toBe('link');
  });
});

describe('where Back goes', () => {
  it('goes to the last stop that is another page', () => {
    expect(backTarget('/dues', ['/partners', '/'])).toEqual({
      kind: 'trail',
      index: 1,
      href: '/',
    });
    // A stop of the same page is skipped: the user cannot go back to where he is.
    expect(backTarget('/dues', ['/', '/dues?x=1'])).toEqual({ kind: 'trail', index: 0, href: '/' });
  });

  it('falls back to the parent when the trail has nothing', () => {
    expect(backTarget('/partners/capital', [])).toEqual({ kind: 'parent', to: '/partners' });
  });

  it('has nowhere to go from a menu page with an empty trail', () => {
    expect(backTarget('/partners', [])).toBeNull();
    expect(backTarget('/', [])).toBeNull();
  });
});

describe('the parent of a page opened from another page', () => {
  it.each([
    ['/dues', '/'],
    ['/others-dues', '/'],
    ['/kharch', '/'],
    ['/commission', '/'],
    ['/day', '/'],
    ['/partners/setup', '/partners'],
    ['/partners/capital', '/partners'],
    ['/partners/statement', '/partners'],
    ['/month', '/partners'],
    ['/index/accounts', '/index'],
    ['/index/accounts/new', '/index/accounts'],
    ['/index/accounts/12', '/index/accounts'],
    ['/index/rules', '/index'],
    ['/index/rules/new', '/index/rules'],
    ['/index/rules/test', '/index/rules'],
    ['/index/rules/7', '/index/rules'],
    ['/index/chart', '/index'],
    ['/index/openings', '/index'],
    ['/index/cities', '/index'],
    ['/index/cities/new', '/index/cities'],
    ['/index/items/3', '/index/items'],
    ['/gardens/new', '/gardens'],
    ['/gardens/5', '/gardens'],
    ['/trips/new', '/trips'],
    ['/trips/9', '/trips'],
    ['/reports/R10', '/reports'],
    ['/reports/dead-amount', '/reports'],
    ['/reports/collection-sheet', '/reports'],
    ['/analytics/A', '/analytics'],
    ['/bikri/collection', '/bikri/voucher'],
    ['/bikri/later', '/bikri/stock'],
  ])('%s goes back to %s', (path, parent) => {
    expect(parentOf(path)).toBe(parent);
  });

  it.each([
    '/',
    '/index',
    '/bikri/book',
    '/bikri/today',
    '/bikri/stock',
    '/bikri/party-ledgers',
    '/bikri/voucher',
    '/rokar',
    '/rokar/expense',
    '/gardens',
    '/trips',
    '/partners',
    '/analytics',
    '/reports',
    '/backup',
    '/settings',
  ])('%s is a menu page and has no parent', (path) => {
    expect(parentOf(path)).toBeNull();
  });
});
