// The screens that need today's date say so when the day status cannot be read, with a retry,
// instead of opening a form with an empty date (Code Standards F8; D1+D2 review, finding 38).
// New account, new rule and the opening balances grid, in the Urdu and in the English app.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import { LANGUAGES } from '../../src/i18n';
import { fakeAccounts } from '../helpers/accounts';
import {
  DAY,
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { fakeRules } from '../helpers/rules';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/rules/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const SCREENS = [
  { name: 'new account', path: '/index/accounts/new', titleId: 'accounts-load-failed' },
  { name: 'new rule', path: '/index/rules/new', titleId: 'rules-load-failed' },
  { name: 'opening balances', path: '/index/openings', titleId: 'openings-load-failed' },
] as const;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeAccounts([]);
  fakeMasters();
  fakeRules();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the day status cannot be read (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it.each(SCREENS)(
    '$name shows the error with a retry, and the retry opens the screen',
    async ({ path, titleId }) => {
      vi.mocked(appApi.dayStatus).mockRejectedValue(fail({ code: 'ERR_DB_BUSY' }));
      await renderApp(path);
      const title = (text: string) => (content: string) => visible(content) === text;
      expect(await screen.findByText(title(message(language, titleId)))).toBeInTheDocument();
      expect(document.body.textContent).not.toContain('ERR_');
      // The retry asks again; this time the day is there.
      vi.mocked(appApi.dayStatus).mockResolvedValue(DAY);
      await userEvent
        .setup()
        .click(screen.getByRole('button', { name: new RegExp(message(language, 'common-retry')) }));
      await waitFor(() => {
        expect(screen.queryByText(title(message(language, titleId)))).not.toBeInTheDocument();
      });
    },
  );
});
