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
  return (
    <div className="flex h-screen flex-col bg-bg text-ink">
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
          <main
            ref={screen}
            className="min-h-0 flex-1 overflow-auto p-6 wide:mx-auto wide:w-full wide:max-w-page"
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
