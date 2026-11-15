// What the address of the commission page carries (PRD FR-9.3, plan 77): the period (one date, a month, a
// year or a range of dates) and the party (a buyer or a seller). `#/commission?period=year&year=2026&
// side=seller&party=...` opens the page for the year 2026 of one seller, the way the button under an answer of
// the question panel does. Everything is optional, and anything that does not read is left out, so an
// address with only `?month=2026-10` (plan 76) opens that month as it always did. The page keeps its own
// period and party in the address, so a language switch, a reload and the way back from a بکری land on the same
// page (like `useWindowParam`). Dates are text as the backend sends them (`YYYY-MM-DD`, `YYYY-MM`, `YYYY`);
// there is no money here.

import { useNavigate, useSearch } from '@tanstack/react-router';
import { useCallback, useMemo } from 'react';
import type { AnswerSource } from '../../bindings/AnswerSource';
import { isIsoDay, isIsoMonth, monthBounds } from '../../shared/format';
import { pickedOrNull } from '../../shared/screen-state';
import type { PickerValue } from '../../shared/ui';

/** The period choices of the page, in the order the Owner sees them (PRD FR-9.3). */
export const COMMISSION_KINDS = ['day', 'month', 'year', 'range'] as const;
export type CommissionKind = (typeof COMMISSION_KINDS)[number];

/** The side of the party filter. No side is every party (the page of plan 76). */
export const COMMISSION_SIDES = ['buyer', 'seller'] as const;
export type CommissionSide = (typeof COMMISSION_SIDES)[number];

/** The kind the page opens on. */
export const DEFAULT_COMMISSION_KIND: CommissionKind = 'month';

/** The address, as it is kept: every key is optional. */
export interface CommissionSearch {
  /** The period choice; none is a month. */
  period?: CommissionKind;
  /** The month, `YYYY-MM`; none is the current month. */
  month?: string;
  /** The one date, `YYYY-MM-DD`. */
  date?: string;
  /** The year, `YYYY`. */
  year?: string;
  /** The range, `YYYY-MM-DD`. */
  from?: string;
  to?: string;
  /** The side and the party. A party without a side is not read. */
  side?: CommissionSide;
  party?: PickerValue;
  /** The person key of a چکتا buyer, when the address came from an answer that names one. */
  person?: string;
}

/** True when the text is a year, `YYYY` (1900 to 2999: a year of the books, not a typo). */
export function isIsoYear(text: unknown): text is string {
  if (typeof text !== 'string' || !/^\d{4}$/.test(text)) return false;
  const year = Number(text);
  return year >= 1900 && year <= 2999;
}

/** The year `years` after `year` (before, when negative). Example: 2026, -1 gives 2025. */
export function addYears(year: string, years: number): string {
  return String(Number(year) + years).padStart(4, '0');
}

/** The year of a day or a month (`YYYY-MM-DD` or `YYYY-MM`): its first four characters. */
export function yearOf(iso: string): string {
  return iso.slice(0, 4);
}

function oneOf<T extends string>(choices: readonly T[], value: unknown): T | undefined {
  return choices.find((choice) => choice === value);
}

/**
 * Reads the address. A word must be one the page knows; a date must be a real one. A party needs its
 * side: a side with no party is kept (the picker is then empty), a party with no side is dropped.
 */
export function parseCommissionSearch(search: Record<string, unknown>): CommissionSearch {
  const side = oneOf(COMMISSION_SIDES, search['side']);
  const party = side === undefined ? null : pickedOrNull(search['party']);
  const person = search['person'];
  return {
    ...parsePeriodSearch(search),
    ...(side === undefined ? {} : { side }),
    ...(party === null ? {} : { party }),
    ...(party !== null && typeof person === 'string' && person.trim() !== ''
      ? { person: person.trim() }
      : {}),
  };
}

/** The period keys of an address (shared with the kharch page): a word must be one we know, a date a real one. */
export function parsePeriodSearch(
  search: Record<string, unknown>,
): Pick<CommissionSearch, 'period' | 'month' | 'date' | 'year' | 'from' | 'to'> {
  const period = oneOf(COMMISSION_KINDS, search['period']);
  const month = search['month'];
  const date = search['date'];
  // A hand-typed `?year=2026` is read as a number by the router; the page keeps a year as text.
  const rawYear = search['year'];
  const year = typeof rawYear === 'number' ? String(rawYear) : rawYear;
  const from = search['from'];
  const to = search['to'];
  return {
    ...(period === undefined || period === DEFAULT_COMMISSION_KIND ? {} : { period }),
    ...(isIsoMonth(month) ? { month } : {}),
    ...(isIsoDay(date) ? { date } : {}),
    ...(isIsoYear(year) ? { year } : {}),
    ...(isIsoDay(from) ? { from } : {}),
    ...(isIsoDay(to) ? { to } : {}),
  };
}

/** What the page shows: the kind with the values of that kind, and the party. Null is "the latest". */
export interface CommissionFilters {
  kind: CommissionKind;
  month: string | null;
  day: string | null;
  year: string | null;
  from: string | null;
  to: string | null;
  side: CommissionSide | null;
  party: PickerValue | null;
  person: string | null;
}

/** The part of the filters that is the period (one date, a month, a year or a range): shared with the kharch page. */
export type PeriodFilters = Pick<
  CommissionFilters,
  'kind' | 'month' | 'day' | 'year' | 'from' | 'to'
>;

/** The period the address asks for. The kind is a month unless the address says another. */
export function periodFiltersOfSearch(
  search: Pick<CommissionSearch, 'period' | 'month' | 'date' | 'year' | 'from' | 'to'>,
): PeriodFilters {
  return {
    kind: search.period ?? DEFAULT_COMMISSION_KIND,
    month: search.month ?? null,
    day: search.date ?? null,
    year: search.year ?? null,
    from: search.from ?? null,
    to: search.to ?? null,
  };
}

/** The filters the address asks for. */
export function filtersOfSearch(search: CommissionSearch): CommissionFilters {
  return {
    ...periodFiltersOfSearch(search),
    side: search.side ?? null,
    party: search.party ?? null,
    person: search.person ?? null,
  };
}

/**
 * The address for the period. A key that holds nothing is left out; the month kind is no `period` key, and a
 * month that is the current one is no `month` key, so a reload on it is always the latest month. Only the values
 * of the kind on screen are kept: a date typed for "one date" does not travel to "a year".
 */
export function searchOfPeriod(
  filters: PeriodFilters,
): Pick<CommissionSearch, 'period' | 'month' | 'date' | 'year' | 'from' | 'to'> {
  const { kind } = filters;
  return {
    ...(kind === DEFAULT_COMMISSION_KIND ? {} : { period: kind }),
    ...(kind === 'month' && filters.month !== null ? { month: filters.month } : {}),
    ...(kind === 'day' && filters.day !== null ? { date: filters.day } : {}),
    ...(kind === 'year' && filters.year !== null ? { year: filters.year } : {}),
    ...(kind === 'range' && filters.from !== null ? { from: filters.from } : {}),
    ...(kind === 'range' && filters.to !== null ? { to: filters.to } : {}),
  };
}

/** The address for the filters: the period, and the party with his side. */
export function searchOfFilters(filters: CommissionFilters): CommissionSearch {
  return {
    ...searchOfPeriod(filters),
    ...(filters.side === null ? {} : { side: filters.side }),
    ...(filters.side !== null && filters.party !== null ? { party: filters.party } : {}),
    ...(filters.side !== null && filters.party !== null && filters.person !== null
      ? { person: filters.person }
      : {}),
  };
}

/** True when the filters are the page as plan 76 had it: the current month of every party. */
export function isPlainCommission(filters: CommissionFilters): boolean {
  return filters.kind === DEFAULT_COMMISSION_KIND && filters.side === null;
}

/**
 * The filters of the page, kept in its address, and the way to change some of them. The new address replaces the entry in
 * the history, so Back leaves the page and does not step through periods.
 */
export function useCommissionFilters(): readonly [
  CommissionFilters,
  (change: Partial<CommissionFilters>) => void,
] {
  const navigate = useNavigate();
  const raw: Record<string, unknown> = useSearch({ strict: false });
  const filters = useMemo(() => filtersOfSearch(parseCommissionSearch(raw)), [raw]);
  const change = useCallback(
    (part: Partial<CommissionFilters>) => {
      const next: CommissionFilters = { ...filters, ...part };
      void navigate({
        to: '.',
        // A plain record, like `useWindowParam`: the route types of the other screens share some key names.
        search: (): Record<string, unknown> => ({ ...searchOfFilters(next) }),
        replace: true,
      });
    },
    [filters, navigate],
  );
  return [filters, change];
}

/**
 * The period of two days as the page shows it (PRD FR-9.3): one day is a date, a whole calendar month is that
 * month, a whole year is that year, anything else is a range. A question that named no dates (an empty start)
 * is every date from `firstDay`, the first day with entries (the backend's `DayStatus.first_entry_date`), up to
 * today (an empty end); until that day is known the current month opens instead of a guess. Example:
 * 2026-10-01 to 2026-10-31 is the month 2026-10; 2026-10-01 to 2026-10-10 is a range (the month so far).
 */
export function periodSearchOf(
  from: string,
  to: string,
  firstDay: string | null,
): Pick<CommissionSearch, 'period' | 'month' | 'date' | 'year' | 'from' | 'to'> {
  if (from === '' || to === '') {
    const start = from === '' ? firstDay : from;
    if (start === null) return {};
    return { period: 'range', from: start, ...(to === '' ? {} : { to }) };
  }
  if (from === to) return { period: 'day', date: from };
  const month = from.slice(0, 7);
  if (from === `${month}-01` && to === monthBounds(month).to) return { month };
  const year = from.slice(0, 4);
  if (from === `${year}-01-01` && to === `${year}-12-31`) return { period: 'year', year };
  return { period: 'range', from, to };
}

/**
 * The address of the commission page for the source an answer of the question panel read (PRD FR-7.74, FR-7.90,
 * plan 77): the party of that side and the days it was read for. `firstDay` is the first day with entries, when it
 * is known. Nothing is worked out here.
 */
export function commissionSearchOfSource(
  source: Extract<AnswerSource, { kind: 'commission' }>,
  firstDay: string | null,
): CommissionSearch {
  const { record, person_key: person } = source.party;
  return {
    ...periodSearchOf(source.from, source.to, firstDay),
    side: source.side,
    party: {
      id: record.id,
      name_ur: record.name_ur,
      name_en: record.name_en,
      ...(record.code === null ? {} : { code: record.code }),
    },
    ...(person === null ? {} : { person }),
  };
}
