// The seller dues page (plan 79 items 1 and 2a, PRD FR-9.6, FR-19.1, FR-19.11, FR-19.16) in the Urdu and the English
// app, against a fake backend: both seller tiles open it, each on its own side; it opens on سب with every
// seller and the backend's totals (each is a tile's figure); the side filter asks the backend with the side;
// the party filter starts at سب (no party sent); Enter on a seller opens his history; and it has its four
// states. The screen adds nothing up.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as dashboardApi from '../../src/features/dashboard/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeDashboard } from '../helpers/dashboard';
import { SELLERS_ALL, SELLERS_NONE } from '../helpers/dues-pages';
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
  vi.mocked(dashboardApi.dashboardSellerDues).mockResolvedValue(SELLERS_ALL);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the seller dues page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const open = (path = '/dues/sellers') => renderApp(path);
  const rowOf = (id: number) =>
    document.querySelector<HTMLElement>(`.ag-row[row-id="${String(id)}"]`);
  const lastCall = () => vi.mocked(dashboardApi.dashboardSellerDues).mock.calls.at(-1)?.[0];

  it('opens on سب with every seller, and the totals of both sides, the advances and the list are the backend’s', async () => {
    await open();
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('seller-dues-title'),
    });
    await waitFor(() => {
      expect(rowOf(12)).not.toBeNull();
    });
    // The first ask is سب: no side narrowing and no party.
    expect(lastCall()).toMatchObject({ side: 'all', party_id: null, offset: 0 });
    const ehsan = visible(rowOf(12)?.textContent);
    expect(ehsan).toContain(language === 'ur' ? 'احسان' : 'Ehsan');
    expect(ehsan).toContain('120,780');
    expect(ehsan).toContain(m('report-side-cr'));
    expect(visible(rowOf(11)?.textContent)).toContain('72,740');
    expect(visible(rowOf(13)?.textContent)).toContain('9,260');
    // The totals at the top are the tiles' figures: the shop owes the sellers 202,780; they owe it nothing.
    const main = visible(screen.getByRole('main').textContent);
    expect(main).toContain(m('dashboard-tile-payable'));
    expect(main).toContain(m('dashboard-tile-dues-sellers'));
    // The total row under the columns is the backend's, with the side word.
    const totals = visible(document.querySelector('.ag-row-pinned')?.textContent);
    expect(totals).toContain('202,780');
    expect(totals).toContain(m('report-side-cr'));
  });

  it('the address of a tile opens the page on its side, and the side filter asks the backend with the side', async () => {
    const user = userEvent.setup();
    await open('/dues/sellers?side=shop_owes');
    await waitFor(() => {
      expect(lastCall()).toMatchObject({ side: 'shop_owes', party_id: null });
    });
    await waitFor(() => {
      expect(rowOf(11)).not.toBeNull();
    });
    vi.mocked(dashboardApi.dashboardSellerDues).mockResolvedValue(SELLERS_NONE);
    await user.click(screen.getByRole('combobox', { name: m('seller-dues-side-label') }));
    await user.click(
      await screen.findByRole('option', { name: m('seller-dues-side-seller_owes') }),
    );
    await waitFor(() => {
      expect(lastCall()).toMatchObject({ side: 'seller_owes', party_id: null });
    });
    // Nothing matches the side: the page says so and offers سب, which asks for every seller again.
    expect(
      await screen.findByText((text) => visible(text) === m('seller-dues-no-match')),
    ).toBeInTheDocument();
    vi.mocked(dashboardApi.dashboardSellerDues).mockResolvedValue(SELLERS_ALL);
    await user.click(
      screen.getByRole('button', { name: (name) => visible(name) === m('seller-dues-show-all') }),
    );
    await waitFor(() => {
      expect(lastCall()).toMatchObject({ side: 'all', party_id: null });
    });
    await waitFor(() => {
      expect(rowOf(12)).not.toBeNull();
    });
  });

  it('the seller filter starts at سب', async () => {
    await open();
    await waitFor(() => {
      expect(rowOf(12)).not.toBeNull();
    });
    expect(screen.getByPlaceholderText(m('search-all-choice'))).toBeInTheDocument();
  });

  it('Enter on a seller opens his history: the statement, the opening balance first (plan 78 B2)', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await waitFor(() => {
      expect(rowOf(12)).not.toBeNull();
    });
    const cell = rowOf(12)?.querySelector<HTMLElement>('[col-id="seller"]');
    if (cell === null || cell === undefined) throw new Error('no cell');
    await user.click(cell);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/party-ledgers');
    });
    expect(router.state.location.search).toMatchObject({ side: 'seller', party: 12 });
  });

  it('says that no seller has a balance, with the words of the empty state', async () => {
    vi.mocked(dashboardApi.dashboardSellerDues).mockResolvedValue({
      ...SELLERS_NONE,
      shop_owes_paisa: SELLERS_NONE.seller_owes_paisa,
    });
    await open();
    expect(
      await screen.findByText((text) => visible(text) === m('seller-dues-empty')),
    ).toBeInTheDocument();
  });

  it('an error is told with a retry that asks again', async () => {
    vi.mocked(dashboardApi.dashboardSellerDues).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
    );
    const user = userEvent.setup();
    await open();
    const retry = await screen.findByRole('button', {
      name: (name) => visible(name) === m('common-retry'),
    });
    await user.click(retry);
    await waitFor(() => {
      expect(rowOf(12)).not.toBeNull();
    });
    expect(within(document.body).queryByRole('button', { name: m('common-retry') })).toBeNull();
  });
});
