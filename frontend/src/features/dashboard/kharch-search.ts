// What the address of the kharch page carries (PRD FR-9.5, plan 77): the period (one date, a month, a year or a
// range of dates, as the commission page keeps it) and one partner. `#/kharch?period=year&year=2026&partner=2`
// opens the year 2026 with only partner 2 in the partner part. Everything is optional and anything that does not
// read is left out, so `#/kharch` is the current month of every partner. The page keeps its filters in the address,
// so a language switch, a reload and the way back land on the same page. There is no money here.

import { useNavigate, useSearch } from '@tanstack/react-router';
import { useCallback, useMemo } from 'react';
import {
  parsePeriodSearch,
  periodFiltersOfSearch,
  searchOfPeriod,
  type CommissionSearch,
  type PeriodFilters,
} from './commission-search';

/** The address, as it is kept: every key is optional. */
export interface KharchSearch extends Pick<
  CommissionSearch,
  'period' | 'month' | 'date' | 'year' | 'from' | 'to'
> {
  /** The partner of the partner part, by id. */
  partner?: number;
}

/** A positive whole number, from a number or from the text of one (an address carries text). */
function positiveId(value: unknown): number | undefined {
  const id = typeof value === 'string' && value.trim() !== '' ? Number(value) : value;
  return typeof id === 'number' && Number.isSafeInteger(id) && id > 0 ? id : undefined;
}

/** Reads the address: a word must be one the page knows, a date a real one, a partner a positive whole number. */
export function parseKharchSearch(search: Record<string, unknown>): KharchSearch {
  const partner = positiveId(search['partner']);
  return { ...parsePeriodSearch(search), ...(partner === undefined ? {} : { partner }) };
}

/** What the page shows: the period, and the partner (null is every partner). */
export interface KharchFilters extends PeriodFilters {
  partner: number | null;
}

export function kharchFiltersOfSearch(search: KharchSearch): KharchFilters {
  return { ...periodFiltersOfSearch(search), partner: search.partner ?? null };
}

export function searchOfKharchFilters(filters: KharchFilters): KharchSearch {
  return {
    ...searchOfPeriod(filters),
    ...(filters.partner === null ? {} : { partner: filters.partner }),
  };
}

/**
 * The filters of the page, kept in its address, and the way to change some of them. The new address replaces the
 * entry in the history, so Back leaves the page and does not step through periods.
 */
export function useKharchFilters(): readonly [
  KharchFilters,
  (change: Partial<KharchFilters>) => void,
] {
  const navigate = useNavigate();
  const raw: Record<string, unknown> = useSearch({ strict: false });
  const filters = useMemo(() => kharchFiltersOfSearch(parseKharchSearch(raw)), [raw]);
  const change = useCallback(
    (part: Partial<KharchFilters>) => {
      const next: KharchFilters = { ...filters, ...part };
      void navigate({
        to: '.',
        // A plain record, like `useWindowParam`: the route types of the other screens share some key names.
        search: (): Record<string, unknown> => ({ ...searchOfKharchFilters(next) }),
        replace: true,
      });
    },
    [filters, navigate],
  );
  return [filters, change];
}
