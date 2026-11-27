// The box of an entry screen that counts what the user repeats, for every short note inside it (plan 97 item 4, PRD
// FR-12.52). A change typed inside, an Enter inside, and each try to save (`pulse`, bumped by the screen) go up the count that
// `NoticePulseContext` gives to the notes (`useTimedNotices`): a note that has hidden after `NOTICE_MS` shows again.

import type { ReactNode, Ref } from 'react';
import { NoticePulseContext } from '../../../shared/hooks';

interface NoticeScopeProps {
  /** The count of the user's repeated actions that the screen keeps (each try to save and each change or Enter inside). */
  pulse: number;
  /** A change or an Enter happened inside: the screen goes the count up. */
  onRepeat: () => void;
  className?: string;
  ref?: Ref<HTMLDivElement>;
  children: ReactNode;
}

export function NoticeScope({ pulse, onRepeat, className, ref, children }: NoticeScopeProps) {
  return (
    <NoticePulseContext.Provider value={pulse}>
      <div
        ref={ref}
        {...(className === undefined ? {} : { className })}
        onInputCapture={onRepeat}
        onKeyDownCapture={(event) => {
          if (event.key === 'Enter') onRepeat();
        }}
      >
        {children}
      </div>
    </NoticePulseContext.Provider>
  );
}
