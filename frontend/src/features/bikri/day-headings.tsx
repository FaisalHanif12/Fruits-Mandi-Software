// The heading of a day in the window of earlier days (PRD 12.7, plan 72 P4, P7; Umar 12-11-2026): the date
// in bold, then two chips, how many بکری the day has and their خام بکری, each figure written left to
// right by `<Num>` (F25). A day with no بکری is one quiet line, the date and "no sales", with nothing to
// read in it but the words. The count and the total are the backend's, over the whole filter on that day
// (never added up from the rows on the page, FR-19.11, NFR-SPD-24). The band is the shared heading row of
// the list grid (`month-heading`), set a little apart from the rows above it.

import type { BikriDay } from '../../bindings/BikriDay';
import { isoToDisplayDate } from '../../shared/format';
import type { ListContext, ListHeading } from '../../shared/grid';
import { Num } from '../../shared/ui';

/** One chip: its words, then its figure. The words are the name of the figure. */
function Chip({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <span
      data-heading-chip=""
      className="inline-flex items-baseline gap-2 rounded-full border border-border bg-surface px-3 type-chip text-ink"
    >
      <span className="text-ink-muted">{label}</span>
      {children}
    </span>
  );
}

/** The words and figures of one day's heading. */
function DayHeadingBody({ day, context }: { day: BikriDay; context: ListContext }) {
  return (
    <>
      <span className="month-heading-title">
        <Num inherit>{isoToDisplayDate(day.date)}</Num>
      </span>
      {day.count === 0 ? (
        <span className="month-heading-count">{context.text('bikri-past-day-empty')}</span>
      ) : (
        <>
          <Chip label={context.text('bikri-past-day-sales')}>
            <Num inherit>{String(day.count)}</Num>
          </Chip>
          <Chip label={context.text('glossary-kham-bikri')}>
            <Num inherit paisa={day.gross_paisa} />
          </Chip>
        </>
      )}
    </>
  );
}

/** The heading of a day. Its key is the day, so no two headings share one; an empty day is quiet. */
export function dayHeading(day: BikriDay): ListHeading {
  return {
    key: day.date,
    quiet: day.count === 0,
    render: (context) => <DayHeadingBody day={day} context={context} />,
  };
}
