// The days a report covers, in words, as the backend read them (PRD FR-19.1, FR-19.2): a shortcut
// such as "this week" shows the real first and last day, so nobody has to guess what it meant. The
// dates are the backend's echo of the filters, shown as DD-MM-YYYY.

import { Localized } from '@fluent/react';
import type { ReportFilterKind } from '../../../bindings/ReportFilterKind';
import type { ReportResolvedFilters } from '../../../bindings/ReportResolvedFilters';
import { isoToDisplayDate } from '../../../shared/format';

interface ReportRangeProps {
  filters: ReportResolvedFilters;
  takes: readonly ReportFilterKind[];
}

export function ReportRange({ filters, takes }: ReportRangeProps) {
  let line;
  if (takes.includes('as_of')) {
    line =
      filters.as_of === null ? (
        <Localized id="report-as-of-now" />
      ) : (
        <Localized id="report-as-of-shown" vars={{ date: isoToDisplayDate(filters.as_of) }} />
      );
  } else if (takes.includes('date_range')) {
    const from = filters.from === null ? null : isoToDisplayDate(filters.from);
    const to = filters.to === null ? null : isoToDisplayDate(filters.to);
    if (from !== null && to !== null) {
      line = <Localized id="report-range-from-to" vars={{ from, to }} />;
    } else if (from !== null) {
      line = <Localized id="report-range-from" vars={{ from }} />;
    } else if (to !== null) {
      line = <Localized id="report-range-to" vars={{ to }} />;
    } else {
      line = <Localized id="report-range-all" />;
    }
  } else {
    return null;
  }
  return <p className="type-body text-ink-muted">{line}</p>;
}
