// Reopen and fix, the shell's part (plan 44, PRD FR-21.4, E2E-45), in the Urdu and the English app:
// the days that were reopened for a correction close again when the user leaves the screen of the
// fix; the days that close are told; a day that does not close stays open and says why (the checks
// of FR-21.1); and a close is asked once, never twice.

import { act, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import { useFixStore } from '../../src/shared/edit';
import { fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { DAY_FIX_CLOSE_E2E_45, DAY_FIX_CLOSE_STAYED_OPEN } from '../helpers/edit-real';
import { fakeMasters } from '../helpers/masters';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

afterEach(() => {
  leaveAppWindow();
  useFixStore.setState({ session: null, stayedOpen: [] });
});

describe.each(LANGUAGES)('the days of a fix (%s)', (language) => {
  beforeEach(() => {
    vi.clearAllMocks();
    healthyBackend();
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeMasters();
    fakeAccounts([]);
  });

  /** Opens the app and waits until the shell is drawn: the host is part of it. */
  async function openApp(path: string) {
    const app = await renderApp(path);
    await screen.findByRole('main');
    await screen.findAllByRole('heading', { level: 1 });
    return app;
  }

  const begin = (ownerHref: string) => {
    act(() => {
      useFixStore.getState().begin({
        token: 'fix-20261012-0001',
        days: ['2026-10-10', '2026-10-11', '2026-10-12'],
        reason: 'wrong rate',
        ownerHref,
      });
    });
  };

  it('staying on the screen of the fix keeps the days open; leaving it closes them, once', async () => {
    vi.mocked(editApi.dayFixClose).mockResolvedValue(DAY_FIX_CLOSE_E2E_45);
    const { router } = await openApp('/index/accounts');
    begin('/index/accounts');
    await router.navigate({ to: '/index/accounts' });
    expect(editApi.dayFixClose).not.toHaveBeenCalled();
    await router.navigate({ to: '/index' });
    await waitFor(() => {
      expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
    });
    expect(editApi.dayFixClose).toHaveBeenCalledWith('fix-20261012-0001');
    // The success is told in words: the days closed again.
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'edit-fix-closed-toast', {
          days: DAY_FIX_CLOSE_E2E_45.closed
            .map((day) => day.split('-').reverse().join('-'))
            .join(message(language, 'edit-list-separator')),
        }),
      );
    });
    expect(useFixStore.getState().session).toBeNull();
    // Another move asks nothing more.
    await router.navigate({ to: '/index/accounts' });
    expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
  });

  it('a day that cannot close stays open and the box says which day and why', async () => {
    vi.mocked(editApi.dayFixClose).mockResolvedValue(DAY_FIX_CLOSE_STAYED_OPEN);
    const { router } = await openApp('/index/accounts');
    begin('/index/accounts');
    await router.navigate({ to: '/index/accounts' });
    await router.navigate({ to: '/index' });
    const dialog = await screen.findByRole('dialog');
    const text = visible(dialog.textContent);
    expect(text).toContain(message(language, 'edit-fix-stayed-text', { date: '11-10-2026' }));
    expect(text).toContain(message(language, 'err-trial-not-balanced'));
  });

  it('the way there passes other screens: nothing closes before the user has reached the fix', async () => {
    vi.mocked(editApi.dayFixClose).mockResolvedValue(DAY_FIX_CLOSE_E2E_45);
    const { router } = await openApp('/index');
    begin('/index/accounts');
    await router.navigate({ to: '/settings' });
    expect(editApi.dayFixClose).not.toHaveBeenCalled();
  });
});
