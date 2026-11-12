// The row at the top of the sale book (PRD FR-12.13, FR-12.11, FR-7.67): the name of the screen, the
// date and the number of the بکری, and at the end the actions of the screen (Print, Credit Details
// and Save on a new بکری; New, Edit and Cancel on a saved one), all in one row, wrapping only when
// the window is too narrow. The same row is on the new sale book and on a saved one opened read
// only, so the user never feels he is on another screen (design system 8 item 14).
//
// - Find has no button and the four arrows are gone (Umar, 12-11-2026): `F3` opens the find box
//   (FR-12.13, the screen owns the key so a dialog can disable the others), and the find box opens
//   a saved بکری by number, day, seller, buyer or item.
// - Moving off a new بکری that has typed work asks first: the leave guard of the screen stops it
//   (PRD FR-7.9), so opening one from the find box never loses anything.

import type { ReactNode } from 'react';
import { BikriFindDialog } from './BikriFindDialog';

interface BikriToolbarProps {
  /** Open this saved بکری (from the find box). */
  onOpen: (id: number) => void;
  /** The find box is open (F3 opens it; the screen owns the key so a dialog can disable others). */
  findOpen: boolean;
  onFindOpenChange: (open: boolean) => void;
  /** The actions of the screen, at the end of the row. */
  children?: ReactNode;
  /** What was saved last, with a way to open it. */
  lastSaved?: ReactNode;
  /** The name of the screen, at the start of the row (it replaces a line of its own above). */
  title?: ReactNode;
  /** The date and the number of the بکری, right after the name (plan 71 H4). */
  head?: ReactNode;
}

export function BikriToolbar({
  onOpen,
  findOpen,
  onFindOpenChange,
  children,
  lastSaved,
  title,
  head,
}: BikriToolbarProps) {
  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-x-2 gap-y-2">
        {title}
        {head}
        {lastSaved}
        <div className="ms-auto flex flex-wrap items-center gap-2">{children}</div>
      </div>
      <BikriFindDialog open={findOpen} onOpenChange={onFindOpenChange} onOpen={onOpen} />
    </div>
  );
}
