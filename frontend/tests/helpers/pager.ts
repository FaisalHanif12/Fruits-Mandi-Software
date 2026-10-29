// Helpers for the tests of lists with page buttons (PRD FR-19.11, plan 38 R1): the words of the
// pager under a list in the language of the test, and its buttons.

import { screen } from '@testing-library/react';
import type { Language } from '../../src/i18n';
import { DEFAULT_PAGE_SIZE } from '../../src/shared/hooks';
import { message, visible } from './render';

/** How many rows a list asks for until the Owner chooses another number (plan 38 R1). */
export const PAGE_ROWS = DEFAULT_PAGE_SIZE;

/** "26 to 50 of 340" in the language, as it reads in the page text (`visible(...)`). */
export function rangeText(language: Language, from: number, to: number, total: number): string {
  return message(language, 'common-pager-range', {
    from: String(from),
    to: String(to),
    total: String(total),
  });
}

/** "Page 2 of 14" in the language. */
export function pageText(language: Language, page: number, pages: number): string {
  return message(language, 'common-pager-page', { page: String(page), pages: String(pages) });
}

export type PagerButton = 'first' | 'previous' | 'next' | 'last';

/** A page button of the pager under the list, found by its name. */
export function pagerButton(language: Language, which: PagerButton): Promise<HTMLElement> {
  return screen.findByRole('button', {
    name: (name) => visible(name) === message(language, `common-pager-${which}`),
  });
}
