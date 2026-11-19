// The pills of a day of the window of earlier days (PRD 12.7, FR-12.46; Umar 12-11-2026 and 13-11-2026): the
// date in bold, then two pills, how many بکری the day has ("بکری 3") and their خام بکری, each figure written
// left to right by `<Num>` (F25). A day with no بکری has the date and "no sales" and nothing to open. The
// count and the total are the backend's, over the whole filter on that day (never added up from rows,
// FR-19.11, NFR-SPD-24).

import type { ReactNode } from 'react';
import { useLocalization } from '@fluent/react';
import type { BikriDay } from '../../bindings/BikriDay';
import { isoToDisplayDate } from '../../shared/format';
import { Num } from '../../shared/ui';

/** One pill: its words, then its figure. The words are the name of the figure. */
export function DayPill({ label, children }: { label: string; children: ReactNode }) {
  return (
    <span
      data-heading-chip=""
      className="inline-flex items-baseline gap-2 rounded-full bg-surface-sunken px-3 type-chip text-ink"
    >
      <span className="text-ink-muted">{label}</span>
      {children}
    </span>
  );
}

/** The date of a day, in bold. */
export function DayDate({ date }: { date: string }) {
  return (
    <span className="month-heading-title">
      <Num inherit>{isoToDisplayDate(date)}</Num>
    </span>
  );
}

/** What a day says about itself: its date and, when it has بکری, the count and the خام بکری. */
export function DayFigures({ day }: { day: BikriDay }) {
  const { l10n } = useLocalization();
  return (
    <>
      <DayDate date={day.date} />
      {day.count === 0 ? (
        <span className="type-chip text-ink-muted">{l10n.getString('bikri-past-day-empty')}</span>
      ) : (
        <>
          <DayPill label={l10n.getString('bikri-past-day-sales')}>
            <Num inherit>{String(day.count)}</Num>
          </DayPill>
          <DayPill label={l10n.getString('glossary-kham-bikri')}>
            <Num inherit paisa={day.gross_paisa} />
          </DayPill>
        </>
      )}
    </>
  );
}
