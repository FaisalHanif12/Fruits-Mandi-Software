// Arrow keys in menus, lists and reports (PRD FR-7.17): ↑ ↓ move between items, Enter opens the
// focused one, Page Up and Page Down move a page. In the main menu and the بکری side menu the
// arrow that points into the screen moves focus to the screen. Items are marked
// `data-nav-item`; a disabled item is skipped.

import { useMemo, type KeyboardEvent } from 'react';
import { arrowSide, elementDirection, intoScreenSide } from './direction';
import { focusRegion } from './focusRegions';

/** The index an item key moves to, or null for a key that does not move. Does not wrap. */
export function moveIndex(
  current: number,
  count: number,
  key: string,
  pageSize: number,
): number | null {
  if (count === 0) return null;
  const last = count - 1;
  switch (key) {
    case 'ArrowDown':
      return Math.min(current + 1, last);
    case 'ArrowUp':
      return Math.max(current - 1, 0);
    case 'PageDown':
      return Math.min(current + pageSize, last);
    case 'PageUp':
      return Math.max(current - pageSize, 0);
    case 'Home':
      return 0;
    case 'End':
      return last;
    default:
      return null;
  }
}

/** How many items one Page Up or Page Down moves, when the list does not say. */
export const DEFAULT_PAGE_SIZE = 10;

const NATIVE_ACTIVATION = new Set(['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA', 'SUMMARY']);

function itemsOf(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>('[data-nav-item]')).filter(
    (item) => !item.hasAttribute('disabled') && item.getAttribute('aria-disabled') !== 'true',
  );
}

export interface RovingOptions {
  /** Items per Page Up or Page Down. */
  pageSize?: number | undefined;
  /** True for the main menu and the side menu: the arrow into the screen moves focus there. */
  isMenu?: boolean | undefined;
}

/** The key handler of a menu or list. Put it on the element that holds the items. */
export function rovingKeyHandler({ pageSize = DEFAULT_PAGE_SIZE, isMenu = false }: RovingOptions) {
  return (event: KeyboardEvent<HTMLElement>): void => {
    if (
      event.defaultPrevented ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey ||
      event.shiftKey
    ) {
      return;
    }
    const container = event.currentTarget;
    const items = itemsOf(container);
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;
    const current = items.findIndex((item) => item === target || item.contains(target));
    if (current === -1) return;

    if (event.key === 'Enter') {
      const item = items[current];
      if (item === undefined || NATIVE_ACTIVATION.has(target.tagName)) return;
      event.preventDefault();
      item.click();
      return;
    }

    const side = arrowSide(event.key);
    if (side !== null) {
      if (isMenu && side === intoScreenSide(elementDirection(container)) && focusRegion('screen')) {
        event.preventDefault();
      }
      return;
    }

    const to = moveIndex(current, items.length, event.key, pageSize);
    const item = to === null ? undefined : items[to];
    if (item === undefined) return;
    event.preventDefault();
    item.focus();
  };
}

/** The same handler, kept between renders. */
export function useRovingKeys(options: RovingOptions = {}) {
  const { pageSize, isMenu } = options;
  return useMemo(() => rovingKeyHandler({ pageSize, isMenu }), [pageSize, isMenu]);
}
