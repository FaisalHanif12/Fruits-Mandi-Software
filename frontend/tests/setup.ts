import '@testing-library/jest-dom/vitest';
import { cleanup, configure } from '@testing-library/react';
import { afterEach, vi } from 'vitest';
import { resetPagerMemory } from '../src/shared/hooks/usePager';

// A test waits for what the screen shows with waitFor and findBy. The default one second is
// enough on a quiet machine and not on a loaded one: 49 files run at the same time, and the
// Windows runner is slower still (D1+D2 review, finding 26). A test that passes still ends at
// once; only a test that fails waits this long.
const ASYNC_UTIL_TIMEOUT_MS = 10_000;
configure({ asyncUtilTimeout: ASYNC_UTIL_TIMEOUT_MS });

afterEach(() => {
  cleanup();
  // A list remembers its page while the app runs; a test never starts on the page of another.
  resetPagerMemory();
});

// What a screen remembers (PRD FR-7.45) is read from the backend when a screen opens. In a test there
// is no backend and nothing is kept: every screen starts as a first visit, at once. The one exception
// is the first-run guide (PRD FR-7.12), which counts as seen, so it does not open over every test of
// the whole app and take its first Esc. A test about the remembering or the guide mocks this module
// itself and says what was kept.
vi.mock('../src/shared/screen-state/api', () => ({
  prefsScreenGet: vi.fn(({ screen }: { screen: string }) =>
    Promise.resolve({ screen, value: screen === 'app.guide' ? '{"done":true}' : null }),
  ),
  prefsScreenSet: vi.fn(({ screen, value }: { screen: string; value: string }) =>
    Promise.resolve({ screen, value: value === '' ? null : value }),
  ),
}));

// jsdom lacks the pointer-capture and scrolling methods Radix Select calls when it opens.
// Some test files run in plain Node (no DOM), where there is nothing to add.
if (typeof Element !== 'undefined') {
  if (!('hasPointerCapture' in Element.prototype)) {
    Object.assign(Element.prototype, {
      hasPointerCapture: () => false,
      releasePointerCapture: () => undefined,
    });
  }
  if (!('scrollIntoView' in Element.prototype)) {
    Object.assign(Element.prototype, { scrollIntoView: () => undefined });
  }
  // The router scrolls to the top after it opens a page; jsdom prints a notice for that.
  Object.assign(window, { scrollTo: () => undefined });
  // jsdom has no layout, so `offsetParent` is always null. AG Grid places the popup of a cell
  // editor (the picker cells of the بکری grids) against it and fails on null. Without layout the
  // parent element is the nearest thing there is.
  Object.defineProperty(HTMLElement.prototype, 'offsetParent', {
    configurable: true,
    get(this: HTMLElement) {
      return this.parentElement;
    },
  });
  // jsdom has no layout either, so every element's rectangle is all zeros. AG Grid looks at the
  // cell of an open popup editor (the picker cells of the بکری grids) every 200 ms and closes the
  // editor when the cell's rectangle is all zeros, because it takes the cell for one that has left
  // the page (ag-stack `keepPopupPositionedRelativeTo`). With zeros, a picker cell closed itself
  // 200 ms after it opened: a code typed in under 200 ms was fine, and on a busy computer the cell
  // closed in the middle of the code, so the one-time buyer test failed in full parallel runs
  // (found in plan 14). In a real window a cell has a size and the editor stays open, so nothing
  // in the app is wrong. Every element gets a small rectangle that is not all zeros.
  const RECT_WIDTH = 100;
  const RECT_HEIGHT = 20;
  Element.prototype.getBoundingClientRect = function getBoundingClientRect(): DOMRect {
    return new DOMRect(0, 0, RECT_WIDTH, RECT_HEIGHT);
  };
  // jsdom has no canvas. ECharts (the analytics charts) asks for a 2D context to measure text, and
  // jsdom answers null and prints "Not implemented" for each call, which buried the real test
  // output. The answer stays null (the chart copes with it); only the notice is gone.
  Object.assign(HTMLCanvasElement.prototype, { getContext: () => null });
}
