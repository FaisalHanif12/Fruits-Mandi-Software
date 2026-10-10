import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

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
}
