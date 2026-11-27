// Keeps a screen where it is for a moment after a press (plan 95 item 7, PRD FR-12.52, Umar 23-11-2026: "clicking the نقد
// cell must type in place and the page must never jump"). A press on a cell of the lots moves the focus, which selects the
// lot and re-draws the sale lines and the summary under it; the browser may then scroll the page to keep something in view.
// For `HOLD_SCROLL_MS` after the press the scroll position the user had is put back on every frame. A wheel, a touch, a key
// or another press ends the hold at once, so the user is never fought: only a scroll he did not ask for is undone.

/** How long after a press the scroll position is held, in milliseconds (Code Standards G8). */
export const HOLD_SCROLL_MS = 400;

/** Holds the scroll position of `scroller` as it is now, for `HOLD_SCROLL_MS`. */
export function holdScroll(scroller: HTMLElement): void {
  const top = scroller.scrollTop;
  const end = performance.now() + HOLD_SCROLL_MS;
  let frame = 0;
  const events = ['wheel', 'touchmove', 'keydown', 'pointerdown'] as const;
  const stop = () => {
    cancelAnimationFrame(frame);
    for (const name of events) document.removeEventListener(name, stop, true);
  };
  const tick = () => {
    if (performance.now() >= end) {
      stop();
      return;
    }
    if (scroller.scrollTop !== top) scroller.scrollTop = top;
    frame = requestAnimationFrame(tick);
  };
  // The press that started the hold is already past: only a later one ends it.
  window.setTimeout(() => {
    for (const name of events) document.addEventListener(name, stop, true);
  }, 0);
  frame = requestAnimationFrame(tick);
}
