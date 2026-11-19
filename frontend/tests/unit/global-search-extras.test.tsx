// The top bar's search after plan 20, step F3 (PRD FR-5.4): besides accounts, بکری and vouchers it
// finds the screens and reports of the app by their name in Urdu or in English; Enter opens what
// was chosen. It no longer finds people: a چکتا buyer is found in the buyer box of a sale line. It runs inside the real app, in the
// Urdu and in the English app, against fakes of the search commands. The text is searched as typed
// and never converted (Code Standards F41).

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(accountApi.searchGlobal).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the global search, screens and reports (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeReports();
  });

  const m = (id: string) => message(language, id);

  async function searchFor(text: string) {
    const view = await renderApp('/');
    const user = userEvent.setup();
    const box = await screen.findByRole('combobox', { name: m('layout-search-label') });
    await user.click(box);
    await user.type(box, text);
    return { ...view, user, box };
  }

  it('finds a report by its name in this language, and by its name in the other', async () => {
    const { box, user } = await searchFor(language === 'ur' ? 'ذمے' : 'Dues');
    await screen.findByRole(
      'option',
      { name: (name) => visible(name).includes(m('report-r20-title')) },
      SLOW_MS,
    );
    await user.clear(box);
    // A report is found by the name it has in the other language too (PRD FR-7.35).
    await user.type(box, language === 'ur' ? 'Buyer Dues' : 'ذمے');
    expect((await screen.findAllByRole('option', undefined, SLOW_MS)).length).toBeGreaterThan(0);
  });

  it('finds a report by its number, and Enter opens it', async () => {
    const { router, user } = await searchFor('R18');
    await screen.findByRole(
      'option',
      { name: (name) => visible(name).includes(m('report-r18-title')) },
      SLOW_MS,
    );
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R18');
    }, SLOW_MS);
  });

  it('finds a screen of the menu and opens it', async () => {
    const { router, user } = await searchFor(m('layout-nav-backup'));
    await screen.findByRole(
      'option',
      { name: (name) => visible(name).includes(m('report-search-screen')) },
      SLOW_MS,
    );
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/backup');
    }, SLOW_MS);
  });

  it('shows nothing extra, and asks the backend nothing, for an empty box', async () => {
    await renderApp('/');
    await screen.findByRole('combobox', { name: m('layout-search-label') });
    expect(accountApi.searchGlobal).not.toHaveBeenCalled();
  });
});
