// The row at the top of the sale book (PRD FR-12.13, FR-12.11, FR-7.67): the name of the screen, the
// date and the number of the بکری, and at the end the actions of the screen (Print, Credit Details
// and Save on a new بکری; New, Edit and Cancel on a saved one), all in one row, wrapping only when
// the window is too narrow. The same row is on the new sale book and on a saved one opened read
// only, so the user never feels he is on another screen (design system 8 item 14).
//
// - The tabs keep every tab in the row (PRD FR-5.3 v1.57): when the controls leave them too little room the print
//   buttons turn into an icon and a key (`useFitControls`), and only then do the last tabs go behind "More".
// - Find has no button and the four arrows are gone (Umar, 12-11-2026): `F3` opens the find box
//   (FR-12.13, the screen owns the key so a dialog can disable the others), and the find box opens
//   a saved بکری by number, day, seller, buyer or item.
// - Moving off a new بکری that has typed work asks first: the leave guard of the screen stops it
//   (PRD FR-7.9), so opening one from the find box never loses anything.

import { useLayoutEffect, useRef, useState, type ReactNode, type RefObject } from 'react';
import { useLanguage } from '../../../shared/i18n';
import { useSectionTabs } from '../../../shared/ui';
import { BikriFindDialog } from './BikriFindDialog';
import { CompactControlsProvider } from './compact-controls';

/**
 * True when the row of tabs has less room than its tabs want, so the controls beside it must give some up
 * (plan 95, PRD FR-5.3 v1.57: all four tabs stay in the row, "More" only when the window is too narrow). `room` is the
 * width the tabs have now and `natural` the width all of them want, in pixels as fractions. A pixel of slack keeps a
 * rounding from tipping a row that fits.
 */
export function tabsNeedRoom(room: number, natural: number): boolean {
  return natural > 0 && room + 1 < natural;
}

/**
 * Keeps the row of the sale book in one line with every tab in it. When the tabs have less room than they want, the
 * row gets `data-compact`, and the controls that can be smaller (the print buttons, with a class `group-data-
 * [compact]/toolbar:`) show their icon and key only, their words kept for readers and as a tip. Only if the tabs are
 * still short, their own row puts the last ones behind "More". It is measured, not guessed from the window width, so
 * the language, the text size and a longer date or number all count. The decision is made before the row is drawn:
 * the attribute is what the CSS follows, and the state returned only lets a button put its words in its tip.
 */
function useFitControls(row: RefObject<HTMLDivElement | null>, enabled: boolean): boolean {
  const language = useLanguage();
  const [compact, setCompact] = useState(false);
  useLayoutEffect(() => {
    const box = row.current;
    if (box === null || !enabled) return undefined;
    let current = true;
    const measure = () => {
      if (!current) return;
      const tabs = box.querySelector<HTMLElement>('[data-section-tabs]');
      const natural = Number(tabs?.dataset['naturalWidth'] ?? 0);
      if (tabs === null || natural <= 0) return;
      // The controls at their full size first: what room would the tabs have then?
      box.removeAttribute('data-compact');
      const short = tabsNeedRoom(tabs.getBoundingClientRect().width, natural);
      if (short) box.setAttribute('data-compact', '');
      // The attribute is what the CSS follows; the state only tells the buttons to put their words in their tip.
      setCompact(short);
    };
    measure();
    const watcher = typeof ResizeObserver === 'function' ? new ResizeObserver(measure) : null;
    watcher?.observe(box);
    // The tabs measure themselves after the fonts load; this row follows them.
    if ('fonts' in document) void document.fonts.ready.then(measure);
    // A change of words or size changes what the tabs and the controls want without changing this row.
    const tabsBox = box.querySelector('[data-section-tabs]');
    if (tabsBox !== null) watcher?.observe(tabsBox);
    return () => {
      current = false;
      watcher?.disconnect();
    };
  }, [row, enabled, language]);
  return compact;
}

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
  // In a section the row of tabs is the name. On the sale book the tabs and the controls share ONE row (Umar,
  // 22-11-2026, plan 93 T7): the controls keep their size and the tabs take the room that is left, the last ones going
  // behind "More". The other section screens let their buttons wrap under the tabs instead.
  const inSection = useSectionTabs() !== null;
  const row = useRef<HTMLDivElement>(null);
  const compact = useFitControls(row, inSection);
  return (
    <div className="flex flex-col gap-2">
      <div ref={row} className="group/toolbar flex flex-wrap items-center gap-x-2 gap-y-2">
        {title}
        {head}
        {lastSaved}
        <div className={`ms-auto flex flex-wrap items-center gap-2 ${inSection ? 'shrink-0' : ''}`}>
          <CompactControlsProvider value={compact}>{children}</CompactControlsProvider>
        </div>
      </div>
      <BikriFindDialog open={findOpen} onOpenChange={onFindOpenChange} onOpen={onOpen} />
    </div>
  );
}
