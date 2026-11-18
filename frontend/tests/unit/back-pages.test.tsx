// One real page of each kind that is opened from another page gets the Back button and Esc (plan 78 F1, PRD
// FR-7.92, FR-7.18), in the whole app with its real router and shell, in Urdu and in English: a page opened by
// its address goes back to its parent with the button and with Esc; a page reached from a menu has no button.
// The pages' own data is not set up here (they show their error or loading state): Back and Esc belong to the
// shell, so they work whatever the page shows.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeDashboard } from '../helpers/dashboard';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

/** A page of each kind: the dashboard's drill pages, the partners area, a report, a new record, a form. */
const CHILD_PAGES: readonly (readonly [string, string])[] = [
  ['/commission', '/'],
  ['/dues', '/'],
  ['/partners/capital', '/partners'],
  ['/partners/statement', '/partners'],
  ['/reports/R10', '/reports'],
  ['/analytics/A', '/analytics'],
  ['/trips/new', '/trips'],
  ['/gardens/new', '/gardens'],
  ['/index/accounts/new', '/index/accounts'],
  ['/index/rules/new', '/index/rules'],
  ['/day', '/'],
];

/** The pages a menu opens: no Back button. */
const MENU_PAGES: readonly string[] = ['/partners', '/reports', '/trips', '/rokar', '/'];

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('Back and Esc on the pages opened from another page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const backButton = () => screen.queryByRole('button', { name: message(language, 'common-back') });

  it.each(CHILD_PAGES)('%s has the Back button, and it goes to %s', async (path, parent) => {
    const { router } = await renderApp(path);
    const button = await waitFor(() => {
      const found = backButton();
      if (found === null) throw new Error('no Back button yet');
      return found;
    });
    // It stands in the top bar, never inside the page.
    expect(button.closest('header')).not.toBeNull();
    await userEvent.setup().click(button);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(parent);
    });
  });

  it.each(CHILD_PAGES)('Esc on %s goes to %s, like the button', async (path, parent) => {
    const { router } = await renderApp(path);
    await waitFor(() => {
      expect(backButton()).not.toBeNull();
    });
    press('Escape');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(parent);
    });
  });

  it.each(MENU_PAGES)('%s, reached from a menu, has no Back button', async (path) => {
    await renderApp(path);
    await screen.findByRole('main');
    expect(backButton()).toBeNull();
    // Nothing inside the page is a second Back either.
    expect(
      within(screen.getByRole('main')).queryByRole('button', {
        name: message(language, 'common-back'),
      }),
    ).toBeNull();
  });
});
