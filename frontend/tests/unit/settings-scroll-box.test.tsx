// Plan 92 (Umar's report, found in WebKit): on Settings, a panel scrolls inside its own box. The hidden native
// <select> that Radix Select keeps beside each drop-down (the role of a print-header person on the firm tab) is
// placed against its nearest positioned ancestor. If that is the screen (`main`) and not the panel's scroll box, `main`
// gets a tall overflow: the title and the tabs scroll away and a blank band shows under the panel. So every panel's
// scroll box must be positioned (`relative`), in the Urdu and in the English app. A browser measures the overflow
// itself (the lead checks in WebKit); this test keeps the class from being dropped.

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)(
  'the scroll box of a Settings panel is a containing block (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    it.each(['settings-tab-firm', 'settings-tab-printing', 'settings-tab-lock'])(
      'the panel of %s scrolls in a box that is positioned',
      async (tab) => {
        await renderApp('/settings');
        await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
        const user = userEvent.setup();
        await user.click(screen.getByRole('tab', { name: named(message(language, tab)) }));
        const panel = await screen.findByRole('tabpanel');
        // The box that scrolls the panel: the one with `overflow-auto` around its cards.
        const box = panel.querySelector('.overflow-auto') ?? panel.closest('.overflow-auto');
        expect(box).not.toBeNull();
        expect(box?.classList.contains('relative')).toBe(true);
        // And it shows no scroll line (plan 93 T4).
        expect(box?.classList.contains('no-scrollbar')).toBe(true);
      },
    );
  },
);
