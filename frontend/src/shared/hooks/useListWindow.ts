// The window a list shows (PRD FR-19.11, plan 76 point 8): a list with many rows a day shows 7 days at a time, a
// list with few rows a day shows one month at a time. The window opens on the latest one (the 7 days that end
// today, or this month), moves with the buttons, Page Up and Page Down, and a day or month typed in the go-to
// box, and never passes today nor goes before the first day that has data in the list (`windows_bounds`).
// What the screen asks the backend for is `range`, the first and last day of the window as `DD-MM-YYYY`
// (the shape of a period); the screen's own filters stay as they are. The window is kept above the grid, so a
// language switch loses nothing. Pure dates and one read-only command; no money.

import { useQuery } from '@tanstack/react-query';
import { useCallback, useState } from 'react';
import type { WindowSource } from '../../bindings/WindowSource';
import {
  addMonths,
  canDaysWindowComeNearer,
  canDaysWindowGoBack,
  canMonthComeNearer,
  canMonthGoBack,
  daysWindowEnd,
  daysWindowOf,
  isoToDisplayDate,
  monthBounds,
  monthOf,
  stepDaysWindow,
  windowMonth,
  type PeriodRange,
} from '../format';
import { windowsBounds } from '../windows/api';

export type ListWindowKind = 'days' | 'month';

interface ListWindowOptions {
  kind: ListWindowKind;
  /** What the list is, for its first day of data. Null: the first day of the books is not asked for. */
  source: WindowSource | null;
  /** The day the app is on, `YYYY-MM-DD` (the day status). Until it is known the list waits. */
  today: string | undefined;
}

export interface ListWindow {
  kind: ListWindowKind;
  /** The day is known: the list can ask. */
  ready: boolean;
  today: string | null;
  /** The first day with data in the list, `YYYY-MM-DD`, or null while it is not known (or there is none). */
  first: string | null;
  /** The first month with data, `YYYY-MM`, or null. */
  firstMonth: string | null;
  /** The days of the window, `YYYY-MM-DD`. Empty until the day is known. */
  from: string;
  to: string;
  /** The month of a month window, `YYYY-MM`, or null for a window of days. */
  month: string | null;
  /** The days to ask the backend for, as `DD-MM-YYYY` (the same shape as a period's range). */
  range: PeriodRange;
  /** The window is the latest one: the later button is off. */
  isLatest: boolean;
  /** The window can go further back: the earlier button is on. */
  canGoBack: boolean;
  /** One window back (-1) or nearer (+1), inside the limits. */
  move: (direction: -1 | 1) => void;
  /** Goes to the window that ends on a day (days) or holds a month (month). */
  goTo: (value: string) => void;
  /** Back to the latest window. */
  latest: () => void;
  /** Page Up and Page Down of the list: the window moves, not a page of rows. */
  onPageKey: (direction: 'previous' | 'next') => void;
}

/** The first month of data from its first day. */
function monthOfFirst(first: string | null): string | null {
  return first === null ? null : monthOf(first);
}

export function useListWindow({ kind, source, today }: ListWindowOptions): ListWindow {
  // The window asked for: the end day of 7 days, or the month; null is the latest.
  const [asked, setAsked] = useState<string | null>(null);
  const bounds = useQuery({
    queryKey: ['windows', 'bounds', source],
    queryFn: () => {
      if (source === null) throw new Error('no source');
      return windowsBounds({ source });
    },
    enabled: source !== null && today !== undefined,
    // The first day of data moves only when something older is saved: reading it again on a return is enough.
    staleTime: 60_000,
  });
  const first = bounds.data?.first_date ?? null;
  const firstMonth = monthOfFirst(first);
  const known = today !== undefined;
  const now = today ?? '';
  const end = known && kind === 'days' ? daysWindowEnd(asked, now, first) : null;
  const month = known && kind === 'month' ? windowMonth(asked, now, firstMonth) : null;

  const days = end === null ? null : daysWindowOf(end);
  const bounded = month === null ? null : monthBounds(month);
  const from = days?.from ?? bounded?.from ?? '';
  const to = days?.to ?? bounded?.to ?? '';
  const range: PeriodRange =
    from === '' ? { from: '', to: '' } : { from: isoToDisplayDate(from), to: isoToDisplayDate(to) };

  const isLatest =
    end !== null
      ? !canDaysWindowComeNearer(end, now)
      : month === null || !canMonthComeNearer(month, now);
  const canGoBack =
    end !== null
      ? canDaysWindowGoBack(end, first)
      : month !== null && canMonthGoBack(month, firstMonth);

  const move = useCallback(
    (direction: -1 | 1) => {
      if (!known) return;
      if (kind === 'days') {
        if (end !== null) setAsked(stepDaysWindow(end, direction, now, first));
      } else if (month !== null) {
        setAsked(windowMonth(addMonths(month, direction), now, firstMonth));
      }
    },
    [known, kind, end, month, now, first, firstMonth],
  );
  const goTo = useCallback((value: string) => {
    setAsked(value);
  }, []);
  const latest = useCallback(() => {
    setAsked(null);
  }, []);
  const onPageKey = useCallback(
    (direction: 'previous' | 'next') => {
      move(direction === 'previous' ? -1 : 1);
    },
    [move],
  );
  return {
    kind,
    ready: known,
    today: today ?? null,
    first,
    firstMonth,
    from,
    to,
    month,
    range,
    isLatest,
    canGoBack,
    move,
    goTo,
    latest,
    onPageKey,
  };
}
