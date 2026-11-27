// Test helper: what every cancel asks beside its reason since v1.61 (PRD FR-7.93, plan 100 C): the Owner's
// sign and the 4-digit partner code, typed into the open cancel box. The tests type the reason as before,
// then call this.

import { screen, within } from '@testing-library/react';
import type { UserEvent } from '@testing-library/user-event';
import type { Language } from '../../src/i18n';
import { message, visible } from './render';

/** What the tests type, and what the request must then hold. */
export const CANCEL_SECRETS = { sign: 'adnan', partner_code: '4826' } as const;

/** Types the sign and the code into the open cancel box. */
export async function typeSignAndCode(user: UserEvent, language: Language): Promise<void> {
  const dialog = screen.getByRole('alertdialog');
  const field = (id: string) =>
    within(dialog).getByLabelText((label) => visible(label) === message(language, id), {
      selector: 'input',
    });
  await user.type(field('cancel-sign-label'), CANCEL_SECRETS.sign);
  await user.type(field('cancel-code-label'), CANCEL_SECRETS.partner_code);
}
