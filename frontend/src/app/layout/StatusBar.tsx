// The bar at the bottom of every screen (PRD FR-5.5, design system 6): the user, today's date,
// the state of the day, and the last cloud backup. Colour always has a word and an icon with it
// (NFR-USE-12). The date arrives as DD-MM-YYYY text from the backend.

import { Localized } from '@fluent/react';
import { Check, Lock } from 'lucide-react';
import { Num, ScriptText } from '../../shared/ui';

export type DayState = 'open' | 'closed';

interface StatusBarProps {
  userName: string;
  /** Today's date as DD-MM-YYYY, from the backend. Empty until it has arrived. */
  date: string;
  /** Null until the backend has said. */
  day: DayState | null;
  /** The last closed day as DD-MM-YYYY, or null when no day is closed (PRD FR-21.1). */
  closedUpTo?: string | null;
  /** The day state opens the screen where days are closed and reopened. */
  onOpenDay?: () => void;
}

export function StatusBar({ userName, date, day, closedUpTo = null, onOpenDay }: StatusBarProps) {
  return (
    <footer className="type-body flex min-h-statusbar shrink-0 items-center gap-5 border-t border-border bg-surface-sunken px-4 text-ink">
      <Localized id="layout-status-user" vars={{ name: userName }} elems={{ name: <ScriptText /> }}>
        <span />
      </Localized>
      {date === '' ? null : (
        <Localized id="layout-status-date" vars={{ date }} elems={{ num: <Num /> }}>
          <span />
        </Localized>
      )}
      {day === null ? null : <DayStatusItem day={day} onOpenDay={onOpenDay} />}
      {closedUpTo === null ? null : (
        <Localized
          id="layout-status-closed-up-to"
          vars={{ date: closedUpTo }}
          elems={{ num: <Num /> }}
        >
          <span />
        </Localized>
      )}
      <span className="flex-1" />
      <span className="flex items-center gap-2">
        <span aria-hidden="true" className="size-2 rounded-full bg-border-control" />
        <Localized id="layout-status-backup-none" />
      </span>
    </footer>
  );
}

/** The state of the day with its icon and word; a button when it can open the day screen. */
function DayStatusItem({ day, onOpenDay }: { day: DayState; onOpenDay: (() => void) | undefined }) {
  const content =
    day === 'open' ? (
      <>
        <Check aria-hidden="true" className="size-icon-sm" />
        <Localized id="layout-status-day-open" />
      </>
    ) : (
      <>
        <Lock aria-hidden="true" className="size-icon-sm" />
        <Localized id="layout-status-day-closed" />
      </>
    );
  const tone = day === 'open' ? 'text-money-in' : 'text-warn';
  if (onOpenDay === undefined) {
    return <span className={`flex items-center gap-1 ${tone}`}>{content}</span>;
  }
  return (
    <button
      type="button"
      onClick={onOpenDay}
      className={`flex min-h-target items-center gap-1 rounded-1 underline ${tone}`}
    >
      {content}
    </button>
  );
}
