// Plan 95 item 7 (PRD FR-12.52): after a press on a cell of the lots the page stays where it was for a moment, so clicking
// the نقد ادائیگی cell types in place and never jumps. A scroll the user asks for (wheel, touch, key, another press) is
// never undone.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { HOLD_SCROLL_MS, holdScroll } from '../../src/shared/hooks/holdScroll';

/** A box whose scroll position is a plain number, as a real scroller's is. */
function scroller(top: number): HTMLElement {
  const box = document.createElement('div');
  let value = top;
  Object.defineProperty(box, 'scrollTop', {
    get: () => value,
    set: (next: number) => {
      value = next;
    },
  });
  return box;
}

beforeEach(() => {
  vi.useFakeTimers({
    toFake: [
      'setTimeout',
      'clearTimeout',
      'requestAnimationFrame',
      'cancelAnimationFrame',
      'performance',
    ],
  });
});

afterEach(() => {
  vi.useRealTimers();
});

describe('holding the scroll after a press', () => {
  it('puts the page back where it was when something moves it, for a short time', () => {
    const box = scroller(240);
    holdScroll(box);
    box.scrollTop = 0;
    vi.advanceTimersByTime(50);
    expect(box.scrollTop).toBe(240);
    box.scrollTop = 600;
    vi.advanceTimersByTime(50);
    expect(box.scrollTop).toBe(240);
  });

  it('lets go after the hold, so the user can scroll as he likes', () => {
    const box = scroller(240);
    holdScroll(box);
    vi.advanceTimersByTime(HOLD_SCROLL_MS + 100);
    box.scrollTop = 10;
    vi.advanceTimersByTime(100);
    expect(box.scrollTop).toBe(10);
  });

  it.each(['wheel', 'touchmove', 'keydown', 'pointerdown'])(
    'a %s by the user ends the hold at once',
    (name) => {
      const box = scroller(240);
      holdScroll(box);
      vi.advanceTimersByTime(20);
      document.dispatchEvent(new Event(name, { bubbles: true }));
      box.scrollTop = 90;
      vi.advanceTimersByTime(100);
      expect(box.scrollTop).toBe(90);
    },
  );
});
