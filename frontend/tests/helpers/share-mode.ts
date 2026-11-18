// Test helper: the choice "عام حصے / اپنے حصے" at the top of the shares of a new trip or garden (plan 78 D1). A new
// form starts on the default shares, which are shown and cannot be typed in; a test that types a share chooses
// the custom shares first, the way the Owner does.

import { screen } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';
import type { Language } from '../../src/i18n';
import { message, visible } from './render';

type User = ReturnType<typeof userEvent.setup>;

/** Presses "Custom shares", which opens every share for typing. */
export async function chooseCustomShares(user: User, language: Language) {
  await user.click(
    screen.getByRole('button', {
      name: (name) => visible(name) === visible(message(language, 'ventures-share-mode-custom')),
    }),
  );
}
