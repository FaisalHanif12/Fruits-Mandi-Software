// The reopen-and-fix session of the app (plan 44, PRD FR-21.4): the token of the days that were
// reopened for one correction, and the days that stayed open when they were closed again. It lives
// above the screens, so it survives a move between the list and the entry, and the shell can close
// the days when the user leaves. Nothing here is money.

import { create } from 'zustand';
import type { DayFixStayedOpen } from '../../bindings/DayFixStayedOpen';

/** The days reopened for one correction. */
export interface FixSession {
  /** The backend's token for closing exactly these days again. */
  token: string;
  /** The reopened days, `YYYY-MM-DD`, oldest first. */
  days: readonly string[];
  /** The reason typed when the days were reopened. It starts the reason of the edit. */
  reason: string;
  /**
   * The address (`/bikri/book?id=1&edit=2`) of the screen the entry is edited on. The shell closes
   * the days when the user has been there and then goes to another address.
   */
  ownerHref: string;
  /** The user has reached the screen of `ownerHref` (the way there may pass other addresses). */
  entered: boolean;
}

interface FixState {
  session: FixSession | null;
  /** The days that did not close, with the checks that failed, until the user has read them. */
  stayedOpen: readonly DayFixStayedOpen[];
  begin: (session: Omit<FixSession, 'entered'>) => void;
  /** The user reached the screen of the fix. */
  markEntered: () => void;
  /** Forgets the session. The caller has taken its token first. */
  end: () => void;
  setStayedOpen: (days: readonly DayFixStayedOpen[]) => void;
}

export const useFixStore = create<FixState>()((set) => ({
  session: null,
  stayedOpen: [],
  begin: (session) => {
    set({ session: { ...session, entered: false } });
  },
  markEntered: () => {
    // Only the first time: a second call would make a new session object, and the shell, which
    // watches the session, would call this again without end.
    set((state) =>
      state.session === null || state.session.entered
        ? state
        : { session: { ...state.session, entered: true } },
    );
  },

  end: () => {
    set({ session: null });
  },
  setStayedOpen: (stayedOpen) => {
    set({ stayedOpen });
  },
}));
