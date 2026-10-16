// The frame of every screen after login (PRD FR-5.2 to FR-5.6): the main menu on the start side,
// the top bar, the screen, the status bar, and optionally the side menu of an area. It registers
// the menu and the screen for the arrow keys (PRD FR-7.17), and mounts the text size keys once.

import type { ReactNode } from 'react';
import { useFocusRegion } from '../../shared/keyboard';
import { StatusBar, type DayState } from './StatusBar';
import { TopBar } from './TopBar';
import { useTextSize } from './text-size';

interface AppShellProps {
  /** The main menu, the top bar's neighbour on the start side. */
  nav: ReactNode;
  /** The side menu of an area (the بکری side menu), between the main menu and the screen. */
  moduleMenu?: ReactNode;
  userName: string;
  onLock: () => void;
  onSignOut: () => void;
  /** A result of the top bar's search was chosen: open that account. */
  onOpenAccount: (id: number) => void;
  date: string;
  day: DayState | null;
  /** The last closed day as DD-MM-YYYY, or null (PRD FR-21.1). */
  closedUpTo?: string | null;
  /** The state of the day opens the screen where days are closed and reopened. */
  onOpenDay?: () => void;
  children: ReactNode;
}

export function AppShell({
  nav,
  moduleMenu,
  userName,
  onLock,
  onSignOut,
  onOpenAccount,
  date,
  day,
  closedUpTo = null,
  onOpenDay,
  children,
}: AppShellProps) {
  useTextSize();
  const screen = useFocusRegion<HTMLElement>('screen');
  // The shell is exactly as tall as the window and never scrolls itself; each screen scrolls in
  // its own area. Without the clip, Gulzar's tall letter metrics in the status bar pushed the page
  // 8 px past the window (D1+D2 review, finding 29).
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-bg text-ink">
      <div className="flex min-h-0 flex-1">
        {nav}
        {moduleMenu}
        <div className="flex min-w-0 flex-1 flex-col">
          <TopBar
            userName={userName}
            onLock={onLock}
            onSignOut={onSignOut}
            onOpenAccount={onOpenAccount}
          />
          {/* `relative` makes the screen the containing block of the hidden native <select> that
              Radix Select keeps beside each drop-down. Without it that element sits against the
              window, outside this scroll area, and the whole page scrolled (finding 29). */}
          <main
            ref={screen}
            className="relative min-h-0 flex-1 scroll-pb-strip overflow-auto p-6 wide:mx-auto wide:w-full wide:max-w-page"
          >
            {children}
          </main>
        </div>
      </div>
      <StatusBar
        userName={userName}
        date={date}
        day={day}
        closedUpTo={closedUpTo}
        {...(onOpenDay === undefined ? {} : { onOpenDay })}
      />
    </div>
  );
}
