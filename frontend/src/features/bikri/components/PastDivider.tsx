// The divider between today's بکری and the days before them (PRD 12.7, plan 72 P4; Umar 12-11-2026):
// no box and no buttons, only a thin rule across the page with a small pill in the middle that says what
// comes next: a calendar, "پچھلے 7 دن" and the days the window covers, "03-10-2026 تا 09-10-2026". The
// buttons that move the window are at the end of the page (`PastWeekNav`). The pill has the words of
// the window and the rule only separates, so a screen reader hears one line (`role="separator"` with its
// name). Quiet by design: tokens only, no motion.

import { Localized, useLocalization } from '@fluent/react';
import { CalendarDays } from 'lucide-react';
import { isoToDisplayDate } from '../../../shared/format';
import { RUNNING_NUMBER } from '../../../shared/ui';
import { WINDOW_DAYS } from '../past-window';

interface PastDividerProps {
  /** The first and the last day of the window, `YYYY-MM-DD`. */
  from: string;
  to: string;
}

export function PastDivider({ from, to }: PastDividerProps) {
  const { l10n } = useLocalization();
  const first = isoToDisplayDate(from);
  const last = isoToDisplayDate(to);
  return (
    <div
      role="separator"
      aria-label={l10n.getString('bikri-past-divider-label', { from: first, to: last })}
      className="flex shrink-0 items-center gap-3 py-1"
    >
      <span aria-hidden="true" className="h-px flex-1 bg-border" />
      <span className="inline-flex items-center gap-2 rounded-full border border-border bg-surface px-3 py-1 type-chip text-ink-muted">
        <CalendarDays aria-hidden="true" className="size-icon-md" />
        <Localized
          id="bikri-past-divider-title"
          vars={{ days: String(WINDOW_DAYS) }}
          elems={RUNNING_NUMBER}
        >
          <span />
        </Localized>
        <span aria-hidden="true" className="size-1 rounded-full bg-border-control" />
        <Localized id="bikri-past-range" vars={{ from: first, to: last }} elems={RUNNING_NUMBER}>
          <span />
        </Localized>
      </span>
      <span aria-hidden="true" className="h-px flex-1 bg-border" />
    </div>
  );
}
