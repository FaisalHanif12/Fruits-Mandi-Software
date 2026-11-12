// The two-way link between an entry grid and the "+" button under it (Umar, plan 71 follow-up; PRD FR-7.13,
// FR-7.17, NFR-USE-7). The arrows reach the remove (trash) cell at the end of each row; from the trash cell
// of the LAST row `↓` goes to the "+" button, and `↑` on the button goes back to that cell. The grid and the
// button are two parts of the screen, so they meet through this one object, made once by the screen with
// `useAddRowLink` and given to both: `<EntryGrid addLink={link} />` and `<AddRowButton link={link} />`.
//
// - `buttonRef` is the button's element, so the grid can put the focus on it.
// - `focusLastRemove` goes to the grid's trash cell of the last row (or the last cell of that row when it
//   has no trash button): the grid gives the way with `setFocusLastRemove`, and the button calls it.

import { createRef, useState, type RefObject } from 'react';

export interface AddRowLink {
  /** The "+" button, once it is drawn. */
  buttonRef: RefObject<HTMLButtonElement | null>;
  /** Puts the focus on the last row's trash cell and says whether it did (no grid drawn: false). */
  focusLastRemove: () => boolean;
  /** Called by the grid: the way to put the focus on its last row's trash cell, or null when it goes. */
  setFocusLastRemove: (focus: (() => boolean) | null) => void;
}

/** One link for one grid and its "+" button. It never changes for the life of the screen. */
export function useAddRowLink(): AddRowLink {
  const [link] = useState<AddRowLink>(() => {
    let focus: (() => boolean) | null = null;
    return {
      buttonRef: createRef<HTMLButtonElement>(),
      focusLastRemove: () => focus?.() ?? false,
      setFocusLastRemove: (next) => {
        focus = next;
      },
    };
  });
  return link;
}
