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
  /**
   * The user chose "leave without saving". A screen that keeps a draft of its own throws it away
   * here, so the work he gave up is not offered back later (plan 13).
   */
  onLeave?: (() => void) | undefined;
  /**
   * A way out that is not leaving: the address the screen is going to is the same screen with
   * another choice on it (the voucher screen with another type, which keeps what was typed). Asked
   * with the address the user is going to. True lets him through without the prompt.
   */
  allowNavigation?:
    ((next: { pathname: string; search: Record<string, unknown> }) => boolean) | undefined;
}

export function LeaveGuard({ dirty, onSave, onLeave, allowNavigation }: LeaveGuardProps) {
  // True while the form is being saved for the user who chose "save": the prompt closes at once,
  // and that must not cancel the way out he asked for.
  const deciding = useRef(false);
  const blocker = useBlocker({
    shouldBlockFn: ({ next }) =>
      dirty &&
      !deciding.current &&
      allowNavigation?.({
        pathname: next.pathname,
        search: next.search as Record<string, unknown>,
      }) !== true,
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
        onLeave?.();
        if (blocker.status === 'blocked') blocker.proceed();
        deciding.current = false;
      }}
    />
  );
}
