// Which days of the window of پچھلی بکری are open (PRD FR-12.46 v1.39, plan 75 7.7). Pure, no money.
// Every day with a بکری starts closed. While the filters are set, or "show every بکری" is ticked, the
// days that have a match open by themselves, so no result is hidden; the Owner can still close one.
// A day with no بکری is never open: it is one line with nothing under it. What the Owner opened or
// closed by hand is kept in two sets and beats the default; a move of the window or a new filter
// starts again (the state is made empty), except for the one day the move asked to open.

import type { BikriDay } from '../../bindings/BikriDay';

/** What the Owner opened and closed by hand since the window or the filter last changed. */
export interface DayOverrides {
  readonly opened: ReadonlySet<string>;
  readonly closed: ReadonlySet<string>;
}

export const NO_OVERRIDES: DayOverrides = { opened: new Set(), closed: new Set() };

/** True when the day is open now. `autoOpen`: the filters are set, or every بکری is shown. */
export function isDayOpen(day: BikriDay, autoOpen: boolean, overrides: DayOverrides): boolean {
  if (day.count === 0) return false;
  if (overrides.closed.has(day.date)) return false;
  if (overrides.opened.has(day.date)) return true;
  return autoOpen;
}

/** The overrides after the arrow of `day` was pressed: an open day closes, a closed day opens. */
export function toggleDay(day: BikriDay, autoOpen: boolean, overrides: DayOverrides): DayOverrides {
  const open = isDayOpen(day, autoOpen, overrides);
  const opened = new Set(overrides.opened);
  const closed = new Set(overrides.closed);
  if (open) {
    opened.delete(day.date);
    closed.add(day.date);
  } else {
    closed.delete(day.date);
    opened.add(day.date);
  }
  return { opened, closed };
}

/** The overrides with `date` open: "go to a date" and the way back from an edit ask for this. */
export function withDayOpen(date: string, overrides: DayOverrides): DayOverrides {
  const opened = new Set(overrides.opened);
  const closed = new Set(overrides.closed);
  closed.delete(date);
  opened.add(date);
  return { opened, closed };
}
