// Opens the ⓘ next to a label or a heading (plan 38 G5): the words that used to sit under the field
// are in a bubble now. The label is found by its words, and the button is the ⓘ in the same row (the
// nearest parent that holds one).

import { screen, within } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';
import type { Language } from '../../src/i18n';
import { message, visible } from './render';

type User = ReturnType<typeof userEvent.setup>;

/** Clicks the ⓘ beside the text `near` (the words of a label or a heading). */
export async function openInfo(user: User, language: Language, near: string): Promise<void> {
  const label = await screen.findByText((content) => visible(content) === near);
  const infoName = message(language, 'common-info-label');
  let row: HTMLElement | null = label;
  for (let step = 0; step < 4 && row !== null; step += 1) {
    const found = within(row).queryByRole('button', { name: (name) => visible(name) === infoName });
    if (found !== null) {
      await user.click(found);
      return;
    }
    row = row.parentElement;
  }
  throw new Error(`no ⓘ near "${near}"`);
}
