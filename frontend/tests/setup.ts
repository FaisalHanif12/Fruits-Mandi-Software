import '@testing-library/jest-dom/vitest';
import { cleanup, configure } from '@testing-library/react';
import { afterEach } from 'vitest';

// A test waits for what the screen shows with waitFor and findBy. The default one second is
// enough on a quiet machine and not on a loaded one: 49 files run at the same time, and the
// Windows runner is slower still (D1+D2 review, finding 26). A test that passes still ends at
// once; only a test that fails waits this long.
const ASYNC_UTIL_TIMEOUT_MS = 10_000;
configure({ asyncUtilTimeout: ASYNC_UTIL_TIMEOUT_MS });

afterEach(() => {
  cleanup();
});

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
}
