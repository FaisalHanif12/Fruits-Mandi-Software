// Keeps the party card under the sale lines in view while the user is typing in them (PRD FR-7.38).
// The card is drawn under the grid, and in a short window the page has already scrolled to the cell
// being typed in, so the card, with its links to the ledger and the statement, would sit below the
// bottom edge. When the card appears or grows, the page scrolls just enough to show all of it, but
// never so far that the cell with the focus leaves the top of the page. Nothing happens when the
// focus is not in the sale lines, so the card never moves a page the user is reading elsewhere.

import { useEffect, type RefObject } from 'react';

/** Space left under the card when it is scrolled into view, in pixels. */
const CARD_GAP_PX = 8;

/** The nearest ancestor that scrolls up and down, or null. */
function scrollerOf(element: HTMLElement): HTMLElement | null {
  for (let at = element.parentElement; at !== null; at = at.parentElement) {
    const overflow = getComputedStyle(at).overflowY;
    if (overflow === 'auto' || overflow === 'scroll') return at;
  }
  return null;
}

export function useKeepCardInView(box: RefObject<HTMLElement | null>, region: HTMLElement | null) {
  useEffect(() => {
    const card = box.current;
    if (card === null || region === null) return undefined;
    const keep = () => {
      const active = document.activeElement;
      if (!(active instanceof HTMLElement) || !region.contains(active)) return;
      const scroller = scrollerOf(card);
      if (scroller === null) return;
      const cardBottom = card.getBoundingClientRect().bottom + CARD_GAP_PX;
      const view = scroller.getBoundingClientRect();
      const overflow = cardBottom - view.bottom;
      if (overflow <= 0) return;
      // The cell that has the focus stays on the page: the page goes up only as far as that allows.
      const cell = active.closest<HTMLElement>('.ag-cell') ?? active;
      const room = Math.max(0, cell.getBoundingClientRect().top - view.top - CARD_GAP_PX);
      scroller.scrollTop += Math.min(overflow, room);
    };
    // A test window has no ResizeObserver: the card is then looked at once.
    const Observer = (globalThis as { ResizeObserver?: typeof ResizeObserver }).ResizeObserver;
    if (Observer === undefined) {
      keep();
      return undefined;
    }
    const observer = new Observer(keep);
    observer.observe(card);
    keep();
    return () => {
      observer.disconnect();
    };
  }, [box, region]);
}
