// When the guide starts (PRD FR-7.12): once, the first time the Owner signs in. After he closes it
// (Skip, Done or Esc) a mark is kept (`prefs_screen_*`, screen `app.guide`), so it does not open by
// itself again. The user menu opens it again at any time (`replayGuide`).

import { useNavigate } from '@tanstack/react-router';
import { useEffect, useRef } from 'react';
import { isRecord, useScreenMemory } from '../../shared/screen-state';
import { useGuideStore } from './guide-store';
import { GUIDE_SCREEN } from './steps';

/** True when the saved mark says the guide was seen. */
function wasSeen(saved: unknown): boolean {
  return isRecord(saved) && saved['done'] === true;
}

/** Starts the guide the first time, and keeps the mark that it was seen. Call once, signed in. */
export function useGuide(active: boolean): { replay: () => void } {
  const navigate = useNavigate();
  const memory = useScreenMemory(GUIDE_SCREEN);
  const step = useGuideStore((state) => state.step);
  const open = useGuideStore((state) => state.open);
  const started = useRef(false);
  const wasOpen = useRef(false);

  useEffect(() => {
    if (!active || !memory.ready || started.current) return;
    started.current = true;
    // A mark that cannot be read is not "never seen": the guide stays shut rather than open over
    // work the Owner has done before.
    if (!memory.failed && !wasSeen(memory.saved)) open(0);
  }, [active, memory.ready, memory.failed, memory.saved, open]);

  // The guide closed: keep that it was seen.
  const { remember } = memory;
  useEffect(() => {
    if (step !== null) {
      wasOpen.current = true;
    } else if (wasOpen.current) {
      wasOpen.current = false;
      remember({ done: true });
    }
  }, [step, remember]);

  return {
    replay: () => {
      void navigate({ to: '/' });
      open(0);
    },
  };
}
