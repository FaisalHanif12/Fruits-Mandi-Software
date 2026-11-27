// The "user did the action again" count of an entry screen, for every short note on it (plan 97 item 4, PRD FR-12.52).
// The screen counts each try to save, each change typed in its lots or lines and each Enter, and gives the count to this
// context. A note that has hidden (`useTimedNotices`, after `NOTICE_MS`) shows again when the count goes up. A note outside
// such a screen sees 0 and never shows again by itself.

import { createContext, useContext } from 'react';

export const NoticePulseContext = createContext(0);

/** The count of repeated actions of the screen the note is on. */
export function useNoticePulse(): number {
  return useContext(NoticePulseContext);
}
