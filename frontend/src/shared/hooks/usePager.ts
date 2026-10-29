// The state of the page buttons under a list (PRD FR-19.11, plan 38 R1): which page is on screen,
// how many rows a page has, and the numbers the pager shows ("Page 2 of 14", "26 to 50 of 340").
// The rows per page (25, 50 or 100) is remembered for each screen (PRD FR-7.45). A new filter goes
// back to page 1. When the Owner comes back to a list from a record he opened, the list is on the
// same page again. The counts are row counts, never money, so working them out here is allowed.

import { useCallback, useEffect, useMemo, useState } from 'react';
import { useScreenMemory } from '../screen-state/hooks';
import { choiceOr, isRecord } from '../screen-state/remember';

/** The choices of "rows per page" (plan 38 R1). */
export const PAGE_SIZES = [25, 50, 100] as const;
export type PageSize = (typeof PAGE_SIZES)[number];
/** What a list shows until the Owner chooses another number for that screen. */
export const DEFAULT_PAGE_SIZE: PageSize = 25;

/** The page of a screen is kept under this name, so every screen has its own (FR-7.45). */
const MEMORY_PREFIX = 'pager.';

/** What the pager of a list shows and does. Pages count from 1. */
export interface PagerState {
  /** The page on screen. */
  page: number;
  pageSize: PageSize;
  /** How many rows the whole filter has, as the backend counted them. */
  total: number;
  /** How many pages the whole filter has; at least 1. */
  pageCount: number;
  /** The first and the last row on this page, counted from 1; both 0 when there are no rows. */
  from: number;
  to: number;
  /** A page is on its way. */
  busy: boolean;
  /** Goes to a page. A number outside 1 to `pageCount` is moved inside. */
  goTo: (page: number) => void;
  /** Changes the rows per page, goes back to page 1 and remembers the choice for this screen. */
  setPageSize: (size: PageSize) => void;
}

/**
 * Where a page is, for `total` rows. A page past the end is moved to the last one.
 * Example: page 14 of 340 rows at 25 a page gives from 326 and to 340; 0 rows give 1 page, 0 to 0.
 */
export function pageWindow(page: number, pageSize: number, total: number) {
  const pageCount = Math.max(1, Math.ceil(total / pageSize));
  const shown = Math.min(Math.max(1, page), pageCount);
  return {
    page: shown,
    pageCount,
    from: total === 0 ? 0 : (shown - 1) * pageSize + 1,
    to: Math.min(shown * pageSize, total),
  };
}

/** The page of each screen the Owner was on, for the rest of this run of the app. */
const pageMemory = new Map<string, { filterKey: string; page: number }>();

/** Forgets the pages of every screen. For tests, so one test never starts on the page of another. */
export function resetPagerMemory(): void {
  pageMemory.clear();
}

function sizeFrom(saved: unknown): PageSize {
  const text = isRecord(saved) ? saved['size'] : undefined;
  const wanted = typeof text === 'number' ? String(text) : text;
  const sizes = PAGE_SIZES.map(String);
  return Number(choiceOr(wanted, sizes, String(DEFAULT_PAGE_SIZE))) as PageSize;
}

export interface PagerControl {
  /** False until the saved rows per page of the screen has arrived (or could not be read). */
  ready: boolean;
  page: number;
  pageSize: PageSize;
  setPage: (page: number) => void;
  setPageSize: (size: PageSize) => void;
}

/**
 * The page and the rows per page of one list. `screen` names the list (`cashbook.takes`: lower-case
 * letters, digits and `_ - .`). `filterKey` is the text of everything the list is filtered by; when
 * it changes, the list goes back to page 1.
 */
export function usePagerControl(screen: string, filterKey: string): PagerControl {
  const memory = useScreenMemory(MEMORY_PREFIX + screen);
  const savedSize = useMemo(() => sizeFrom(memory.saved), [memory.saved]);
  const [chosen, setChosen] = useState<PageSize | null>(null);
  const pageSize = chosen ?? savedSize;

  const [page, setPageState] = useState(() => {
    const kept = pageMemory.get(screen);
    return kept?.filterKey === filterKey ? kept.page : 1;
  });
  // A new filter shows its first page. This is set while drawing, so no page of the old filter is
  // asked for with the new filter.
  const [seenKey, setSeenKey] = useState(filterKey);
  let shown = page;
  if (seenKey !== filterKey) {
    setSeenKey(filterKey);
    setPageState(1);
    shown = 1;
  }
  useEffect(() => {
    pageMemory.set(screen, { filterKey, page: shown });
  }, [screen, filterKey, shown]);

  const { remember } = memory;
  const setPageSize = useCallback(
    (size: PageSize) => {
      setChosen(size);
      setPageState(1);
      remember({ size });
    },
    [remember],
  );
  return { ready: memory.ready, page: shown, pageSize, setPage: setPageState, setPageSize };
}

/** What the pager shows, once the backend has said how many rows the filter has. */
export function describePager(control: PagerControl, total: number, busy: boolean): PagerState {
  const window = pageWindow(control.page, control.pageSize, total);
  return {
    ...window,
    pageSize: control.pageSize,
    total,
    busy,
    goTo: (wanted) => {
      control.setPage(pageWindow(wanted, control.pageSize, total).page);
    },
    setPageSize: control.setPageSize,
  };
}

/**
 * The page buttons for a list that is already in memory (a short master list that the screen filters
 * itself): the rows of the page on screen, and the pager. `rows` undefined means the list is not here
 * yet. For a list that is long or filtered by the backend, use `usePagedQuery` instead.
 */
export function useClientPager<T>(
  screen: string,
  filterKey: string,
  rows: readonly T[] | undefined,
): { pageRows: T[] | undefined; pager: PagerState } {
  const control = usePagerControl(screen, filterKey);
  const total = rows?.length ?? 0;
  const pager = describePager(control, total, !control.ready);
  const pageRows =
    rows === undefined || !control.ready
      ? undefined
      : rows.slice((pager.page - 1) * control.pageSize, pager.page * control.pageSize);
  return { pageRows, pager };
}
