// Starting where the Owner left off (round 9, PRD FR-7.45): the app opens on the screen it was left
// on, keeps the screen that is open, forgets it when the Owner goes back to the dashboard, never
// offers a screen made for one record, and never takes him away from a screen he has already moved to.

import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LAST_SCREEN, isRestorablePath } from '../../src/app/use-last-screen';
import * as screenApi from '../../src/shared/screen-state/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeDashboard } from '../helpers/dashboard';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

const get = vi.mocked(screenApi.prefsScreenGet);
const set = vi.mocked(screenApi.prefsScreenSet);

/** What the screens kept: the guide counts as seen, and the last screen is what a test says. */
function kept(lastScreen: string | null) {
  get.mockImplementation(({ screen }) => {
    if (screen === 'app.guide') return Promise.resolve({ screen, value: '{"done":true}' });
    if (screen === LAST_SCREEN) {
      return Promise.resolve({
        screen,
        value: lastScreen === null ? null : JSON.stringify(lastScreen),
      });
    }
    return Promise.resolve({ screen, value: null });
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  resetClientState('en');
  signedIn(sessionUser({ language: 'en' }));
  set.mockImplementation(({ screen, value }) =>
    Promise.resolve({ screen, value: value === '' ? null : value }),
  );
});

afterEach(() => {
  leaveAppWindow();
});

describe('which screens are offered again', () => {
  it.each(['/bikri/stock', '/reports/R20', '/partners', '/rokar/expense', '/settings'])(
    '%s is',
    (path) => {
      expect(isRestorablePath(path)).toBe(true);
    },
  );

  it.each([
    '/',
    '',
    'bikri',
    '/print',
    '/print?bikri=3',
    '/index/accounts/12',
    '/trips/5',
    '/trips/new',
    '/index/accounts/new',
  ])('%s is not', (path) => {
    expect(isRestorablePath(path)).toBe(false);
  });
});

describe('starting where he left off', () => {
  it('opens the screen the app was left on', async () => {
    kept('/bikri/stock');
    const { router } = await renderApp('/');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/stock');
    });
  });

  it('starts on the dashboard when nothing was kept', async () => {
    kept(null);
    const { router } = await renderApp('/');
    await new Promise((resolve) => setTimeout(resolve, 300));
    expect(router.state.location.pathname).toBe('/');
  });

  it('does not open a screen made for one record, which may be gone', async () => {
    kept('/index/accounts/12');
    const { router } = await renderApp('/');
    await new Promise((resolve) => setTimeout(resolve, 300));
    expect(router.state.location.pathname).toBe('/');
  });

  it('does not take him away from a screen he has already moved to', async () => {
    let answer: (state: { screen: string; value: string | null }) => void = () => undefined;
    get.mockImplementation(({ screen }) => {
      if (screen === 'app.guide') return Promise.resolve({ screen, value: '{"done":true}' });
      if (screen === LAST_SCREEN) {
        return new Promise((resolve) => {
          answer = resolve;
        });
      }
      return Promise.resolve({ screen, value: null });
    });
    const { router } = await renderApp('/');
    await router.navigate({ to: '/bikri/today' });
    answer({ screen: LAST_SCREEN, value: '"/bikri/stock"' });
    await new Promise((resolve) => setTimeout(resolve, 300));
    expect(router.state.location.pathname).toBe('/bikri/today');
  });
});

describe('keeping the screen that is open', () => {
  it('saves the path of each screen he opens, once he settles on it', async () => {
    kept(null);
    const { router } = await renderApp('/');
    await router.navigate({ to: '/bikri/today' });
    await waitFor(() => {
      expect(set).toHaveBeenCalledWith({ screen: LAST_SCREEN, value: '"/bikri/today"' });
    });
    // Only the path: never what was typed on the screen.
    expect(set.mock.calls.every(([input]) => !input.value.includes('?'))).toBe(true);
  });

  it('forgets it when he goes back to the dashboard, so the dashboard is the start next time', async () => {
    kept('/bikri/stock');
    const { router } = await renderApp('/');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/stock');
    });
    await router.navigate({ to: '/' });
    await waitFor(() => {
      expect(set).toHaveBeenLastCalledWith({ screen: LAST_SCREEN, value: '' });
    });
  });

  it('keeps nothing for a screen made for one record', async () => {
    kept(null);
    const { router } = await renderApp('/');
    await router.navigate({ to: '/trips/new' });
    await new Promise((resolve) => setTimeout(resolve, 800));
    expect(set.mock.calls.some(([input]) => input.value.includes('trips'))).toBe(false);
  });
});
