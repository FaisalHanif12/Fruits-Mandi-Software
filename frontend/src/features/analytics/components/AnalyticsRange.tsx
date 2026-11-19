// The days a view covers, in words, as the backend read them (PRD FR-18.1): a shortcut such as "this
// month" shows the real first and last day, and a comparison shows the days it is compared with, so
// nobody has to guess. The dates are the backend's echo of the filters, shown as DD-MM-YYYY.

import { Localized } from '@fluent/react';
import type { AnalyticsResolvedFilters } from '../../../bindings/AnalyticsResolvedFilters';
import { isoToDisplayDate } from '../../../shared/format';
import { RUNNING_NUMBER } from '../../../shared/ui';

export function AnalyticsRange({ filters }: { filters: AnalyticsResolvedFilters }) {
  const from = filters.from === null ? null : isoToDisplayDate(filters.from);
  const to = filters.to === null ? null : isoToDisplayDate(filters.to);
  let line;
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
  const compareFrom = filters.compare_from === null ? null : isoToDisplayDate(filters.compare_from);
  const compareTo = filters.compare_to === null ? null : isoToDisplayDate(filters.compare_to);
  return (
    <p className="type-body text-ink-muted">
      {line}
      {compareFrom !== null && compareTo !== null ? (
        <>
          {' · '}
          <Localized id="analytics-compare-range" vars={{ from: compareFrom, to: compareTo }} />
        </>
      ) : null}
    </p>
  );
}
