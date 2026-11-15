// The kharch page's period and partner as the filters of view M (PRD FR-9.5, plan 77): the days of the period as
// `from` and `to` (a month, a year or one date, never past today), the grouping that suits the kind (a date by
// day, everything else by month), and the partner. The page and view M read the same thing (`analytics_run`, view
// M), so this only builds the filters; the backend reads the days and works out every figure.

import { EMPTY_ANALYTICS_FILTERS, filtersToWire, type AnalyticsFilters } from '../analytics';
import { monthBounds, monthOf } from '../../shared/format';
import { yearOf } from './commission-search';
import type { KharchFilters } from './kharch-search';

/** The days of the period, `YYYY-MM-DD`, or null while today is not known. */
export function kharchDays(
  filters: KharchFilters,
  today: string | null,
): { from: string; to: string } | null {
  if (today === null) return null;
  const upToToday = (day: string) => (day > today ? today : day);
  switch (filters.kind) {
    case 'day': {
      const day = filters.day ?? today;
      return { from: day, to: day };
    }
    case 'month': {
      const bounds = monthBounds(filters.month ?? monthOf(today));
      return { from: bounds.from, to: upToToday(bounds.to) };
    }
    case 'year': {
      const year = filters.year ?? yearOf(today);
      return { from: `${year}-01-01`, to: upToToday(`${year}-12-31`) };
    }
    case 'range':
      return {
        from: filters.from ?? monthBounds(monthOf(today)).from,
        to: filters.to ?? today,
      };
  }
}

/** The filters of view M for the page, or null while today is not known. */
export function kharchWire(filters: KharchFilters, today: string | null): AnalyticsFilters | null {
  const days = kharchDays(filters, today);
  if (days === null) return null;
  return filtersToWire(
    {
      ...EMPTY_ANALYTICS_FILTERS,
      period: 'custom',
      from: days.from,
      to: days.to,
      // A date by day; a month, a year and a range by month (a year is one column for each month).
      groupBy: filters.kind === 'day' ? 'day' : 'month',
      partner: filters.partner,
    },
    ['period', 'group_by', 'partner'],
    'M',
  );
}
