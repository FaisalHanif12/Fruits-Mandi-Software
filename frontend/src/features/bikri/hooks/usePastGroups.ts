// The day headings among the rows of the window of earlier days (PRD 12.7, plan 72 P4, P7, P11). The
// backend gives one entry for each day of the window, empty days too, with its count and خام بکری
// over the whole filter; the page gives only some rows. This picks the headings that belong to the
// rows on screen (`headingDaysOnPage`) and tells the grid which heading each row sits under. The list
// is grouped only in date order: with a sort chosen on a column title the rows stand in that order,
// each with its day in its own column, and no heading is drawn.

import { useMemo } from 'react';
import type { BikriListRow } from '../../../bindings/BikriListRow';
import type { ListGroups } from '../../../shared/grid';
import { dayHeading } from '../day-headings';
import type { BikriList } from '../../../bindings/BikriList';
import { headingDaysOnPage } from '../past-window';

/** The group of a row: its day. A module-level function, so the grid keeps its rows between renders. */
function dayOfRow(row: BikriListRow): string {
  return row.entry_date;
}

export function usePastGroups(
  first: BikriList | undefined,
  page: number,
  pageCount: number,
  sorted: boolean,
): ListGroups<BikriListRow> | undefined {
  return useMemo(() => {
    if (first === undefined || sorted) return undefined;
    const days = new Map(first.days.map((day) => [day.date, day]));
    const dates = headingDaysOnPage(
      first.days.map((day) => day.date),
      first.rows.map((row) => row.entry_date),
      page <= 1,
      page >= pageCount,
    );
    const headings = dates.flatMap((date) => {
      const day = days.get(date);
      return day === undefined ? [] : [dayHeading(day)];
    });
    return { headings, groupOf: dayOfRow };
  }, [first, sorted, page, pageCount]);
}
