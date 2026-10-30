// The catalogue of reports of plan 38: a search box finds a report by its name or its number, nothing
// found says so with a way out, a "Go to" choice takes the user to the first report of a group and
// every card stays reachable by the keyboard. Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { CATALOG, fakeReports } from '../helpers/reports';

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
  window.localStorage.clear();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the catalogue of reports, search and jump (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);

  async function open() {
    fakeReports();
    await renderApp('/reports');
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('reports-title'),
    });
    return userEvent.setup();
  }

  const links = () =>
    screen
      .queryAllByRole('link')
      .filter((link) => link.getAttribute('href')?.includes('/reports/R') === true);

  it('finds a report by its name, and by its number', async () => {
    const user = await open();
    const search = await screen.findByRole('textbox', {
      name: (name) => visible(name) === m('reports-search-label'),
    });
    expect(links()).toHaveLength(CATALOG.length);
    const first = CATALOG.find((entry) => entry.id === 'R20');
    if (first === undefined) throw new Error('no R20');
    await user.type(search, m(first.title_id));
    await waitFor(() => {
      expect(links().map((link) => link.getAttribute('href')?.split('/').at(-1))).toContain('R20');
    }, SLOW_MS);
    expect(links().length).toBeLessThan(CATALOG.length);
    await user.clear(search);
    await user.type(search, 'R33');
    await waitFor(() => {
      expect(links().map((link) => link.getAttribute('href')?.split('/').at(-1))).toEqual(['R33']);
    }, SLOW_MS);
  });

  it('says nothing matches, and the button clears the search', async () => {
    const user = await open();
    const search = await screen.findByRole('textbox', {
      name: (name) => visible(name) === m('reports-search-label'),
    });
    await user.type(search, 'qzxq');
    expect(
      await screen.findByText((text) => visible(text) === m('reports-search-none')),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: (name) => visible(name) === m('reports-search-clear') }),
    );
    await waitFor(() => {
      expect(links()).toHaveLength(CATALOG.length);
    }, SLOW_MS);
  });

  it('"Go to" takes the focus to the first report of the group chosen', async () => {
    const user = await open();
    await user.click(
      await screen.findByRole('combobox', {
        name: (name) => visible(name) === m('reports-jump-label'),
      }),
    );
    await user.click(await screen.findByRole('option', { name: m('report-group-business') }));
    await waitFor(() => {
      expect(document.activeElement?.getAttribute('href')).toMatch(/R28$/);
    }, SLOW_MS);
  });
});
