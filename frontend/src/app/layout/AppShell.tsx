// The frame of every screen after login (PRD FR-5.2 to FR-5.6): the main menu on the start side,
// the top bar, the screen, the status bar, and optionally the side menu of an area. It registers
// the menu and the screen for the arrow keys (PRD FR-7.17), and mounts the Esc order and the
// text size keys once. The screens and the routes come in plan 04's next step.

import type { ReactNode } from 'react';
import { useEscapeDispatcher, useFocusRegion } from '../../shared/keyboard';
import { StatusBar, type DayState } from './StatusBar';
import { TopBar } from './TopBar';
import { useTextSize } from './text-size';

interface AppShellProps {
  /** The main menu, the top bar's neighbour on the start side. */
  nav: ReactNode;
  /** The side menu of an area (the بکری side menu), between the main menu and the screen. */
  moduleMenu?: ReactNode;
  userName: string;
  date: string;
  day: DayState;
  children: ReactNode;
}

export function AppShell({ nav, moduleMenu, userName, date, day, children }: AppShellProps) {
  useEscapeDispatcher();
  useTextSize();
  const screen = useFocusRegion<HTMLElement>('screen');
  return (
    <div className="flex h-screen flex-col bg-bg text-ink">
      <div className="flex min-h-0 flex-1">
        {nav}
        {moduleMenu}
        <div className="flex min-w-0 flex-1 flex-col">
          <TopBar userName={userName} />
          <main
            ref={screen}
            className="min-h-0 flex-1 overflow-auto p-6 wide:mx-auto wide:w-full wide:max-w-page"
          >
            {children}
          </main>
        </div>
      </div>
      <StatusBar userName={userName} date={date} day={day} />
    </div>
  );
}
