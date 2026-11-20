// Test helper for the dropdown box (plan 86): jsdom has no layout, so every rectangle is the same
// small box (tests/setup.ts). A test of where a list goes says where the field is and how big the
// list wants to be, and the window and the bars of the shell it sits in. Anything not named keeps the
// small box of the setup.

import { vi } from 'vitest';

export interface RectSpec {
  top: number;
  left: number;
  width: number;
  height: number;
}

interface Layout {
  window: { width: number; height: number };
  /** The element the list hangs from, found again on each measure. */
  field: () => Element | null;
  fieldRect: RectSpec;
  /** The size the open list wants. */
  list: { width: number; height: number };
  /** Parts of the shell, by the test id of the element carrying `data-dropdown-avoid`. */
  bars?: Record<string, RectSpec>;
}

const toRect = ({ top, left, width, height }: RectSpec) => new DOMRect(left, top, width, height);

export interface DropdownLayout {
  /** The field moves (the page scrolled, the window changed). */
  moveField: (rect: Partial<RectSpec>) => void;
  restore: () => void;
}

/** Fixes the window and the rectangles of the field, the list and the bars until `restore`. */
export function mockDropdownLayout(layout: Layout): DropdownLayout {
  const field = { ...layout.fieldRect };
  // The rectangle every other element has (tests/setup.ts), read without taking the method off its object.
  const original = Object.getOwnPropertyDescriptor(Element.prototype, 'getBoundingClientRect')
    ?.value as (this: Element) => DOMRect;
  const innerWidth = Object.getOwnPropertyDescriptor(window, 'innerWidth');
  const innerHeight = Object.getOwnPropertyDescriptor(window, 'innerHeight');
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: layout.window.width });
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: layout.window.height });
  const spy = vi
    .spyOn(Element.prototype, 'getBoundingClientRect')
    .mockImplementation(function measure(this: Element) {
      if (this === layout.field()) return toRect(field);
      if (this.hasAttribute('data-dropdown-popup')) {
        return toRect({ top: 0, left: 0, ...layout.list });
      }
      const bar = layout.bars?.[this.getAttribute('data-testid') ?? ''];
      if (bar !== undefined) return toRect(bar);
      return original.call(this);
    });
  return {
    moveField: (rect) => {
      Object.assign(field, rect);
    },
    restore: () => {
      spy.mockRestore();
      if (innerWidth !== undefined) Object.defineProperty(window, 'innerWidth', innerWidth);
      if (innerHeight !== undefined) Object.defineProperty(window, 'innerHeight', innerHeight);
    },
  };
}
