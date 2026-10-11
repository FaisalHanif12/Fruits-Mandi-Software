// Leaving a screen with unsaved work (PRD FR-7.9): any way out of the screen, a menu item, Esc or
// the browser's own back, stops first and asks "Save changes?" with three buttons. Put it on a
// screen that has a form, with `dirty` true while the form has changes that are not saved. The
// screen itself decides what saving means (`onSave`, true when it worked).

import { useBlocker } from '@tanstack/react-router';
import { useRef } from 'react';
import { SaveChangesPrompt } from '../shared/ui';

interface LeaveGuardProps {
  dirty: boolean;
  /** Saves the form. Resolves true when it was saved and the user may go on. */
  onSave: () => Promise<boolean>;
}

export function LeaveGuard({ dirty, onSave }: LeaveGuardProps) {
  // True while the form is being saved for the user who chose "save": the prompt closes at once,
  // and that must not cancel the way out he asked for.
  const deciding = useRef(false);
  const blocker = useBlocker({
    shouldBlockFn: () => dirty && !deciding.current,
    enableBeforeUnload: () => dirty,
    withResolver: true,
  });
  return (
    <SaveChangesPrompt
      open={blocker.status === 'blocked'}
      onOpenChange={(open) => {
        if (!open && !deciding.current && blocker.status === 'blocked') blocker.reset();
      }}
      onSave={() => {
        deciding.current = true;
        void onSave().then((saved) => {
          if (saved && blocker.status === 'blocked') {
            blocker.proceed();
          } else if (blocker.status === 'blocked') {
            blocker.reset();
          }
          deciding.current = false;
        });
      }}
      onLeave={() => {
        deciding.current = true;
        if (blocker.status === 'blocked') blocker.proceed();
        deciding.current = false;
      }}
    />
  );
}
