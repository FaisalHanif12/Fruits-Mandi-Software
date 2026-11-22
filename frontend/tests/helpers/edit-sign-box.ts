// The box that asks the reason and the Owner's sign when a saved وصولی, ادائیگی or پیشگی is saved after a change
// (PRD FR-15.3, plan 92), as a test uses it: F10 opens it, the reason and Enter, the sign and Enter saves.

import { screen, waitFor, within } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';
import { expect } from 'vitest';
import type { Language } from '../../src/i18n';
import { press } from './app';
import { message, visible } from './render';

type User = ReturnType<typeof userEvent.setup>;

const named = (text: string) => (name: string) => visible(name) === text;

/** The two fields of the open box. */
export function signBoxFields(language: Language, dialog: HTMLElement) {
  return {
    reason: within(dialog).getByRole('combobox', {
      name: named(message(language, 'vouchers-field-edit-reason')),
    }),
    sign: within(dialog).getByRole('textbox', {
      name: named(message(language, 'vouchers-edit-sign-label')),
    }),
  };
}

/** F10, then the box: opens it and waits for it. */
export async function openSignBox(language: Language): Promise<HTMLElement> {
  press('F10');
  const dialog = await screen.findByRole('alertdialog');
  await waitFor(() => {
    expect(signBoxFields(language, dialog).reason).toHaveFocus();
  });
  return dialog;
}

/** F10, the reason, Enter, the sign, Enter: the way the Owner saves a change with the keyboard only. */
export async function saveChange(
  user: User,
  language: Language,
  reason: string,
  sign = 'adnan',
): Promise<HTMLElement> {
  const dialog = await openSignBox(language);
  await user.keyboard(`${reason}{Enter}`);
  await waitFor(() => {
    expect(signBoxFields(language, dialog).sign).toHaveFocus();
  });
  await user.keyboard(`${sign}{Enter}`);
  return dialog;
}

/** The words a field shows under itself (its note, found through `aria-describedby`), as the user reads them. */
export function noteOf(field: HTMLElement): string {
  const id = field.getAttribute('aria-describedby') ?? '';
  return visible(id === '' ? '' : document.getElementById(id)?.textContent);
}
