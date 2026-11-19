// A screen that starts from what it remembered (PRD FR-7.45). The screen is drawn once the saved
// choices have arrived (or could not be read), so it never flashes its first state and then jumps.
// The saved value is read once, when the screen opens; after that the screen owns its choices.

import type { ReactNode } from 'react';
import { LoadingState } from '../ui';
import { useScreenMemory, type ScreenMemory } from './hooks';

interface RememberedProps {
  /** The screen's name: lower-case letters, digits and `_ - .`, at most 40 (for example `report.r20`). */
  screen: string;
  /** The screen, given what it saved last time (unread, as the screen wrote it) and how to save. */
  children: (memory: { saved: unknown; remember: ScreenMemory['remember'] }) => ReactNode;
}

export function Remembered({ screen, children }: RememberedProps) {
  const memory = useScreenMemory(screen);
  if (!memory.ready) return <LoadingState />;
  return <>{children({ saved: memory.saved, remember: memory.remember })}</>;
}
