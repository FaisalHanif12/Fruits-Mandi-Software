// Which way is "left" for the user (PRD FR-7.13, FR-7.34). Arrow keys follow what the user sees,
// in both layouts, so the code looks at the arrow's physical side and at the direction the
// element is written in, never at the language.

import type { Direction } from '../i18n';

export type Side = 'left' | 'right';

/** The writing direction of an element: the nearest `dir` attribute, else the page, else ltr. */
export function elementDirection(element: Element | null): Direction {
  const marked = element?.closest('[dir]')?.getAttribute('dir') ?? document.documentElement.dir;
  return marked === 'rtl' ? 'rtl' : 'ltr';
}

/** The side an arrow key points to, or null for any other key. */
export function arrowSide(key: string): Side | null {
  if (key === 'ArrowLeft') return 'left';
  if (key === 'ArrowRight') return 'right';
  return null;
}

/**
 * The side of the screen where the main menu sits: the start side, so the right in Urdu and the
 * left in English (PRD FR-5.2).
 */
export function menuSide(direction: Direction): Side {
  return direction === 'rtl' ? 'right' : 'left';
}

/** The side that leads from the menu into the screen (PRD FR-7.17). */
export function intoScreenSide(direction: Direction): Side {
  return menuSide(direction) === 'right' ? 'left' : 'right';
}
