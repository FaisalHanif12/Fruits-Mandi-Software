// The days a report covers, in words, as the backend read them (PRD FR-19.1, FR-19.2): a shortcut
// such as "this week" shows the real first and last day, so nobody has to guess what it meant. The
// dates are the backend's echo of the filters, shown as DD-MM-YYYY.

import { Localized } from '@fluent/react';
import type { ReportFilterKind } from '../../../bindings/ReportFilterKind';
import type { ReportResolvedFilters } from '../../../bindings/ReportResolvedFilters';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { RUNNING_NUMBER } from '../../../shared/ui';

interface ReportRangeProps {
  filters: ReportResolvedFilters;
  takes: readonly ReportFilterKind[];
  /** The one day is a day sheet (R35), not the day balances are read at: the words say so. */
  day?: boolean;
}

export function ReportRange({ filters, takes, day = false }: ReportRangeProps) {
  const language = useLanguage();
  let line;
  if (takes.includes('as_of') && day) {
    line =
      filters.as_of === null ? (
        <Localized id="report-day-now" />
      ) : (
        <Localized
          id="report-day-shown"
          vars={{ date: isoToDisplayDate(filters.as_of) }}
          elems={RUNNING_NUMBER}
        >
          <span />
        </Localized>
      );
  } else if (takes.includes('as_of')) {
    line =
      filters.as_of === null ? (
        <Localized id="report-as-of-now" />
      ) : (
        <Localized
          id="report-as-of-shown"
          vars={{ date: isoToDisplayDate(filters.as_of) }}
          elems={RUNNING_NUMBER}
        >
          <span />
        </Localized>
      );
  } else if (takes.includes('date_range')) {
    const from = filters.from === null ? null : isoToDisplayDate(filters.from);
    const to = filters.to === null ? null : isoToDisplayDate(filters.to);
    if (from !== null && to !== null) {
      line = (
        <Localized id="report-range-from-to" vars={{ from, to }} elems={RUNNING_NUMBER}>
          <span />
        </Localized>
      );
    } else if (from !== null) {
      line = (
        <Localized id="report-range-from" vars={{ from }} elems={RUNNING_NUMBER}>
          <span />
        </Localized>
      );
    } else if (to !== null) {
      line = (
        <Localized id="report-range-to" vars={{ to }} elems={RUNNING_NUMBER}>
          <span />
        </Localized>
      );
    } else {
      line = <Localized id="report-range-all" />;
    }
  } else {
    return null;
  }
  // One lot picked by the assistant's button (PRD FR-7.77, plan 71 L9): its name, from the backend's
  // echo, so the report says plainly that it shows that lot alone.
  const lot = filters.lot;
  return (
    <>
      <p className="type-body text-ink-muted">{line}</p>
      {lot === undefined ? null : (
        <Localized
          id="report-lot-shown"
          vars={{ name: displayName({ name_ur: lot.ur, name_en: lot.en }, language) }}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
    </>
  );
}
