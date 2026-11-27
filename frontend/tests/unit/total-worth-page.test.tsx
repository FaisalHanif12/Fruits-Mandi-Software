// The page of کل مالیت (plan 100 B, PRD FR-9.10) in the Urdu and the English app, against a fake backend
// holding the PRD worked day after Events 1 to 5: the net 65,488 big, money to have 80,048 and money to
// give 14,560, each side with the total of every source and its rows; a row opens that party's history;
// and a refusal shows an error with a retry. Every figure is the backend's: the screen adds nothing up.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as dashboardApi from '../../src/features/dashboard/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { totalWorthPage } from '../helpers/attention-panel';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeDashboard } from '../helpers/dashboard';
import { must } from '../helpers/must';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardTotalWorth).mockImplementation((input) =>
    Promise.resolve(totalWorthPage(input.side)),
  );
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the total worth page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const rowOf = (id: string) => document.querySelector<HTMLElement>(`.ag-row[row-id="${id}"]`);

  it('shows the net big with both sides, every source with its total, and one row for each party', async () => {
    await renderApp('/total-worth');
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('total-worth-title'),
    });
    await waitFor(() => {
      expect(rowOf('buyers_dues-21')).not.toBeNull();
      expect(rowOf('sellers_payable-11')).not.toBeNull();
    });
    const totals = visible(document.querySelector('[data-total-worth]')?.textContent);
    for (const figure of ['65,488', '80,048', '14,560']) expect(totals).toContain(figure);
    const page = visible(screen.getByRole('main').textContent);
    // The parts: cash 58,484 and buyers 21,564; the sellers 10,160 and the payables 4,400.
    for (const figure of ['58,484', '21,564', '10,160', '4,400']) expect(page).toContain(figure);
    expect(visible(rowOf('buyers_dues-21')?.textContent)).toContain(
      language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
    );
    expect(visible(rowOf('buyers_dues-21')?.textContent)).toContain('18,684');
    expect(vi.mocked(dashboardApi.dashboardTotalWorth).mock.calls.map((c) => c[0].side)).toEqual(
      expect.arrayContaining(['have', 'give']),
    );
  });

  it.each([
    ['buyers_dues-21', '/bikri/party-ledgers', { side: 'buyer', party: 21 }],
    ['sellers_payable-11', '/bikri/party-ledgers', { side: 'seller', party: 11 }],
    ['payables-2', '/reports/R13', { party: 2 }],
  ] as const)('Enter on %s opens that history', async (id, path, search) => {
    const user = userEvent.setup();
    const { router } = await renderApp('/total-worth');
    await waitFor(() => {
      expect(rowOf(id)).not.toBeNull();
    });
    await user.click(must(rowOf(id)?.querySelector<HTMLElement>('.ag-cell')));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(path);
    });
    expect(router.state.location.search).toMatchObject(search);
  });

  it('a refusal shows in words with a retry', async () => {
    vi.mocked(dashboardApi.dashboardTotalWorth).mockRejectedValue(
      new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
    );
    await renderApp('/total-worth');
    const alerts = await screen.findAllByRole('alert');
    expect(visible(must(alerts[0]).textContent)).not.toContain('ERR_');
    expect(
      screen.getAllByRole('button', { name: m('common-retry') }).length,
    ).toBeGreaterThanOrEqual(1);
  });
});
