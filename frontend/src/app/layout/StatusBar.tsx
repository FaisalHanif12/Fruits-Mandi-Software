// The bar at the bottom of every screen (PRD FR-5.5, design system 6): the user, today's date,
// the state of the day, and the last cloud backup. Colour always has a word and an icon with it
// (NFR-USE-12). The date arrives as DD-MM-YYYY text from the backend.

import { Localized } from '@fluent/react';
import { Check, Lock } from 'lucide-react';
import { Num } from '../../shared/ui';

export type DayState = 'open' | 'closed';

interface StatusBarProps {
  userName: string;
  /** Today's date as DD-MM-YYYY, from the backend. */
  date: string;
  day: DayState;
}

export function StatusBar({ userName, date, day }: StatusBarProps) {
  return (
    <footer className="type-body flex h-statusbar shrink-0 items-center gap-5 border-t border-border bg-surface-sunken px-4 text-ink">
      <Localized id="layout-status-user" vars={{ name: userName }}>
        <span />
      </Localized>
      <Localized id="layout-status-date" vars={{ date }} elems={{ num: <Num /> }}>
        <span />
      </Localized>
      {day === 'open' ? (
        <span className="flex items-center gap-1 text-money-in">
          <Check aria-hidden="true" className="size-icon-sm" />
          <Localized id="layout-status-day-open" />
        </span>
      ) : (
        <span className="flex items-center gap-1 text-warn">
          <Lock aria-hidden="true" className="size-icon-sm" />
          <Localized id="layout-status-day-closed" />
        </span>
      )}
      <span className="flex-1" />
      <span className="flex items-center gap-2">
        <span aria-hidden="true" className="size-2 rounded-full bg-border-control" />
        <Localized id="layout-status-backup-none" />
      </span>
    </footer>
  );
}
