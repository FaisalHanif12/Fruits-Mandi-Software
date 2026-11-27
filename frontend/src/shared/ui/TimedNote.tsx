// One short warning or error of an entry screen that is not a list of its own (plan 97 item 4, PRD FR-12.52): it shows when it
// appears, hides after `NOTICE_MS`, and shows again when the user repeats the action (the screen's `NoticePulseContext`). The
// note is this one box, named by its own id, so an answer that changes its words is not a new note. The red mark of the field
// stays. A notice that carries a button (the closed-day notice) is not wrapped in it.

import { useId, type ReactNode } from 'react';
import { NOTICE_MS, useNoticePulse, useTimedNotices } from '../hooks';

interface TimedNoteProps {
  children: ReactNode;
}

export function TimedNote({ children }: TimedNoteProps) {
  const pulse = useNoticePulse();
  const key = useId();
  const shown = useTimedNotices([key], pulse, NOTICE_MS).has(key);
  return shown ? <>{children}</> : null;
}
