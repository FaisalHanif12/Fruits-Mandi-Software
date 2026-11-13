// The days of the window of earlier days of پچھلی بکری, each one closed line (PRD FR-12.46 v1.39, plan 75
// 7.7; Umar 13-11-2026): a small arrow, the date, the بکری count and the خام بکری. The arrow (a click, or
// Enter or Space on the line) opens the day's بکری under its line and closes it again; several days can be
// open. A day with no بکری is one quiet line with no arrow. ↑ ↓ move between the lines, ← → and Page Up and
// Page Down move the window (the arrow that points back in reading goes to the earlier days). What is open
// is the screen's state, not the grid's, so a language switch keeps it (F39). The count and the خام بکری
// are the backend's; nothing is added up here (CLAUDE.md rule 1).

import { useLocalization } from '@fluent/react';
import { ChevronRight } from 'lucide-react';
import { useId, type KeyboardEvent, type ReactNode } from 'react';
import type { BikriDay } from '../../../bindings/BikriDay';
import { elementDirection, readingStep, useRovingKeys } from '../../../shared/keyboard';
import { DayFigures } from '../day-headings';

interface PastDaysProps {
  /** Every day of the window, newest first, empty days too, as the backend sent them. */
  days: readonly BikriDay[];
  isOpen: (day: BikriDay) => boolean;
  onToggle: (day: BikriDay) => void;
  /** The day the Owner went to by date: its line is tinted. */
  highlight: string | null;
  /** The rows of an open day, drawn under its line. */
  children: (day: BikriDay) => ReactNode;
  /** ← → and Page Up and Page Down on a line move the window. */
  onMove: (direction: 'previous' | 'next') => void;
  /** The window's own totals, after the last day. */
  totals: ReactNode;
}

function DayLine({
  day,
  open,
  lit,
  onToggle,
  children,
}: {
  day: BikriDay;
  open: boolean;
  lit: boolean;
  onToggle: () => void;
  children: ReactNode;
}) {
  const panel = useId();
  const line = useId();
  const tone = lit
    ? 'border-primary-chip bg-primary-soft'
    : open
      ? 'border-primary-chip bg-surface'
      : 'border-border bg-surface hover:border-primary-chip hover:bg-row-current';
  return (
    <li className="flex flex-col gap-2" data-day={day.date}>
      <button
        type="button"
        id={line}
        data-nav-item
        data-day-line=""
        data-lit={lit ? '' : undefined}
        aria-expanded={open}
        aria-controls={panel}
        onClick={onToggle}
        className={`flex min-h-control w-full flex-wrap items-center gap-x-3 gap-y-1 rounded-3 border px-3 py-1 text-start transition-calm ${tone}`}
      >
        <ChevronRight
          aria-hidden="true"
          className={`size-icon-md shrink-0 text-ink-muted transition-turn ${open ? 'rotate-90' : 'rtl:rotate-180'}`}
        />
        <DayFigures day={day} />
      </button>
      {open ? (
        // The rows have the whole width, as the list of today has: its columns fit the narrowest window
        // exactly (list-widths), so nothing is taken from its sides.
        <div id={panel} role="region" aria-labelledby={line}>
          {children}
        </div>
      ) : null}
    </li>
  );
}

export function PastDays({
  days,
  isOpen,
  onToggle,
  highlight,
  children,
  onMove,
  totals,
}: PastDaysProps) {
  const { l10n } = useLocalization();
  const roving = useRovingKeys();
  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    const target = event.target;
    if (!(target instanceof HTMLElement) || !target.hasAttribute('data-day-line')) return;
    if (
      event.defaultPrevented ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey ||
      event.shiftKey
    ) {
      return;
    }
    if (event.key === 'PageUp' || event.key === 'PageDown') {
      event.preventDefault();
      onMove(event.key === 'PageUp' ? 'previous' : 'next');
      return;
    }
    const step = readingStep(event.key, elementDirection(event.currentTarget));
    if (step !== null) {
      event.preventDefault();
      onMove(step === 'toStart' ? 'previous' : 'next');
      return;
    }
    roving(event);
  };
  return (
    <ul
      aria-label={l10n.getString('bikri-past-label')}
      onKeyDown={onKeyDown}
      className="flex flex-col gap-2"
    >
      {days.map((day) =>
        day.count === 0 ? (
          <li
            key={day.date}
            data-day={day.date}
            className={`flex flex-wrap items-center gap-x-3 rounded-3 px-3 py-1 ${highlight === day.date ? 'bg-primary-soft' : ''}`}
          >
            <span aria-hidden="true" className="size-icon-md shrink-0" />
            <DayFigures day={day} />
          </li>
        ) : (
          <DayLine
            key={day.date}
            day={day}
            open={isOpen(day)}
            lit={highlight === day.date}
            onToggle={() => {
              onToggle(day);
            }}
          >
            {children(day)}
          </DayLine>
        ),
      )}
      <li data-day-totals="" className="px-3 py-2">
        {totals}
      </li>
    </ul>
  );
}
