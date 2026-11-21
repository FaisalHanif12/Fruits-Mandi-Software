// The box every dropdown list is drawn in (plan 86, NFR-USE-16, Code Standards F7): the open list of
// a picker, the suggestions under a text field, the key map under an Urdu field, the results under
// the search box. It replaces the lists that were drawn under their field by hand.
//
// - Drawn in a portal, on the page or inside the dialog the field is in, as a fixed box, so a grid cell, a
//   card or a dialog with `overflow: hidden` never cuts it. It is positioned from the field it hangs from
//   (`anchorRef`).
// - Below the field when the whole list fits, above it when there is more room above (see
//   `dropdown-position.ts`). Its height is the room on that side and it scrolls inside itself.
// - Inside the window sideways, in both directions, and clear of the status bar, the top bar and
//   the assistant button (elements marked `data-dropdown-avoid`).
// - It moves with its field: on scroll in any container, on resize, when the field moves by itself
//   (a grid popup repositioned) and when the list changes size. Closed, it takes no room.
//
// A list inside it that must be able to shrink (the rows of a picker) is a flex item with its own
// `overflow-y: auto`; the rest (the add row, the notes) is `shrink-0`. Keys, focus and Escape are
// the field's, as before: this box only places the list.
//
// Placement is set on the element directly (`inset-inline-start` and the block offset, no physical
// `left` or `right`), so scrolling a page re-places it without drawing the screen again.

import { useEffect, useLayoutEffect, useState, type ReactNode, type RefObject } from 'react';
import { createPortal } from 'react-dom';
import { useUiLanguage } from '../i18n';
import {
  DROPDOWN_AVOID_ATTRIBUTE,
  placeDropdown,
  type Box,
  type DropdownSide,
} from './dropdown-position';

interface DropdownPopupProps {
  /** Whether the list is open. Closed, the box stays in the page (a combobox needs its menu element) but takes no room. */
  open: boolean;
  /** The field the list hangs from. */
  anchorRef: RefObject<HTMLElement | null>;
  /** The side to use when both fit; default below. */
  prefer?: DropdownSide | undefined;
  /**
   * `anchor`: exactly as wide as the field (a picker). `content`: as wide as its words, at least as
   * wide as the field (a list of numbers and notes).
   */
  fit?: 'anchor' | 'content';
  /** The direction of the words in the list when it is not the app's (the Urdu spellings under an Urdu field in the English app). */
  direction?: 'ltr' | 'rtl' | undefined;
  /** Extra classes, for example `gap-1` between rows. */
  className?: string;
  children: ReactNode;
}

// `ag-custom-component-popup` is the class AG Grid looks for: a click on an element that has it, or sits in one,
// is a click inside the cell's popup, so the grid does not end the edit before the click picks the row (the
// list of a picker cell is drawn on the page, outside the grid's own popup; FormDialog does the same).
const BASE_CLASSES =
  'ag-custom-component-popup pointer-events-auto fixed z-50 flex flex-col overflow-y-auto rounded-2 border border-border bg-surface p-1 shadow-overlay';

/** The part of the window a page is drawn in. */
function viewportSize(): { width: number; height: number } {
  const root = document.documentElement;
  return {
    width: root.clientWidth > 0 ? root.clientWidth : window.innerWidth,
    height: root.clientHeight > 0 ? root.clientHeight : window.innerHeight,
  };
}

/** A rectangle of the page as the box the placement works with. */
function boxOf(rect: DOMRect): Box {
  return { top: rect.top, bottom: rect.bottom, minX: rect.left, maxX: rect.right };
}

/** The bars of the shell a list must stay clear of, as boxes in window coordinates. */
function obstacleBoxes(): Box[] {
  const boxes: Box[] = [];
  for (const element of document.querySelectorAll(`[${DROPDOWN_AVOID_ATTRIBUTE}]`)) {
    const rect = element.getBoundingClientRect();
    if (rect.width > 0 && rect.height > 0) boxes.push(boxOf(rect));
  }
  return boxes;
}

/** Whether the field is written right to left (the nearest `dir` above it, the page's at the end). */
function isRtl(anchor: HTMLElement): boolean {
  return anchor.closest('[dir]')?.getAttribute('dir') === 'rtl';
}

/** A string that changes when the field or the window moves, so a frame with no change does no work. */
function placementKey(anchor: HTMLElement): string {
  const rect = anchor.getBoundingClientRect();
  const view = viewportSize();
  return [rect.top, rect.left, rect.width, rect.height, view.width, view.height].join('|');
}

/**
 * How far the list and the things in it are scrolled. The list is measured with no limit on its height, which
 * makes a list that was cut to the room as tall as it wants for a moment, and a browser answers a taller box
 * by moving the scroll back to the new end. The positions are put back when the measure is done.
 */
function scrollPositions(element: HTMLElement): [HTMLElement, number][] {
  const scrolled: [HTMLElement, number][] = [];
  for (const node of [element, ...element.querySelectorAll<HTMLElement>('*')]) {
    if (node.scrollTop > 0) scrolled.push([node, node.scrollTop]);
  }
  return scrolled;
}

/** Measures the list, places it, and writes the result on the element. */
function place(
  element: HTMLElement,
  anchor: HTMLElement,
  prefer: DropdownSide | undefined,
  fit: 'anchor' | 'content',
): void {
  const box = anchor.getBoundingClientRect();
  const scrolled = scrollPositions(element);
  // The size the list wants: measured with no limit on it, then limited to the room that is there.
  element.style.maxHeight = '';
  element.style.maxWidth = '';
  if (box.width > 0) {
    if (fit === 'anchor') element.style.width = `${String(box.width)}px`;
    else element.style.minWidth = `${String(box.width)}px`;
  }
  const wanted = element.getBoundingClientRect();
  const placement = placeDropdown({
    anchor: boxOf(box),
    wanted: { width: wanted.width, height: wanted.height },
    viewport: viewportSize(),
    obstacles: obstacleBoxes(),
    rtl: isRtl(anchor),
    ...(prefer === undefined ? {} : { prefer }),
  });
  const { style } = element;
  style.maxHeight = `${String(placement.maxHeight)}px`;
  style.maxWidth = `${String(placement.maxWidth)}px`;
  style.insetInlineStart = `${String(placement.inlineStart)}px`;
  // Above the field the list is tied to its bottom edge, so it grows upward from the field.
  style.insetBlockStart = placement.side === 'bottom' ? `${String(placement.block)}px` : 'auto';
  style.insetBlockEnd = placement.side === 'top' ? `${String(placement.block)}px` : 'auto';
  element.dataset['side'] = placement.side;
  for (const [node, top] of scrolled) {
    if (node.scrollTop !== top) node.scrollTop = top;
  }
}

export function DropdownPopup({
  open,
  anchorRef,
  prefer,
  fit = 'anchor',
  direction: ownDirection,
  className,
  children,
}: DropdownPopupProps) {
  const { language, direction: appDirection } = useUiLanguage();
  const direction = ownDirection ?? appDirection;
  // The element, kept in state so the effects run again when it appears.
  const [element, setElement] = useState<HTMLDivElement | null>(null);
  // Where the box is drawn: the page, or the dialog the field is in, so the list belongs to the dialog's
  // own tree (its focus, its screen reader view, a test that looks inside it). A fixed box is not cut by the
  // dialog's own scrolling either way. Known once the field is on the page.
  const [dialog, setDialog] = useState<HTMLElement | null>(null);
  useLayoutEffect(() => {
    setDialog(anchorRef.current?.closest<HTMLElement>('[role="dialog"]') ?? null);
  }, [element, anchorRef]);

  useLayoutEffect(() => {
    const anchor = anchorRef.current;
    if (!open || element === null || anchor === null) return;
    place(element, anchor, prefer, fit);
  });

  // While open the list follows its field: any scroll, a resize, a change of its own size (rows
  // arrive), and a field that moves by itself (a grid popup that was repositioned).
  useEffect(() => {
    const anchor = anchorRef.current;
    if (!open || element === null || anchor === null) return undefined;
    let last = placementKey(anchor);
    let frame = 0;
    const replace = () => {
      place(element, anchor, prefer, fit);
      last = placementKey(anchor);
    };
    // The list scrolling inside itself moves nothing: only a scroll of the page or of a grid does.
    const onScroll = (event: Event) => {
      if (event.target instanceof Node && element.contains(event.target)) return;
      replace();
    };
    const onFrame = () => {
      frame = window.requestAnimationFrame(onFrame);
      const now = placementKey(anchor);
      if (now !== last) replace();
    };
    frame = window.requestAnimationFrame(onFrame);
    window.addEventListener('resize', replace);
    window.addEventListener('scroll', onScroll, { capture: true, passive: true });
    // A test window has no ResizeObserver: the list is then placed on each draw and on each frame.
    const observer = typeof ResizeObserver === 'function' ? new ResizeObserver(replace) : null;
    observer?.observe(element);
    return () => {
      window.cancelAnimationFrame(frame);
      window.removeEventListener('resize', replace);
      window.removeEventListener('scroll', onScroll, { capture: true });
      observer?.disconnect();
    };
  }, [open, element, anchorRef, prefer, fit]);

  return createPortal(
    <div
      ref={setElement}
      lang={language}
      dir={direction}
      data-dropdown-popup=""
      className={
        open ? `${BASE_CLASSES} ${fit === 'content' ? 'w-max' : ''} ${className ?? ''}` : 'hidden'
      }
    >
      {children}
    </div>,
    dialog ?? document.body,
  );
}
