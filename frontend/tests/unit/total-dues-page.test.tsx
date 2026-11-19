// The total dues page (plan 79 item 2b, PRD FR-9.7, FR-19.11) in the Urdu and the English app, against a fake
// backend: the tile کل اگراہی opens it; four sections (buyers, sellers, the dead amount, people who
// borrowed), each with its own count and total from `dashboard_total_dues` and its own list; the grand total
// is the last line and is the backend's figure (the tile's); each row opens that person's history; a refused
// read of the totals is an error with a retry while the lists keep working; and a section with nobody says so.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BuyerDuesRow } from '../../src/bindings/BuyerDuesRow';
import type { DeadAmountPage } from '../../src/bindings/DeadAmountPage';
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
import {
  DEAD,
  LENT_OPEN,
  SELLERS_NONE,
  TOTAL_DUES,
  TOTAL_DUES_FULL,
  party,
} from '../helpers/dues-pages';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

const rupees = (value: number) => paisaForTest(value * 100);

function buyer(id: number, code: string, ur: string, en: string, owes: number): BuyerDuesRow {
  return {
    party: party(id, code, ur, en),
    city: null,
    balance: { amount_paisa: rupees(owes), side: 'dr' },
    last_change_date: '2026-10-10',
  };
}

const BUYERS = {
  rows: [
    buyer(20, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', 2_880),
    buyer(21, '0021', 'ارشد ملتانی', 'Arshad Multani', 18_684),
  ],
  total_count: 2,
  total_paisa: rupees(21_564),
};

const EMPTY_LENT = { rows: [], total_count: 0, total_paisa: rupees(0), overdue_paisa: rupees(0) };
const EMPTY_DEAD: DeadAmountPage = {
  rows: [],
  row_count: 0,
  totals: { ...DEAD.totals, still_owed_paisa: rupees(0), owed_then_paisa: rupees(0) },
};

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardTotalDues).mockResolvedValue(TOTAL_DUES);
  vi.mocked(dashboardApi.dashboardBuyerDues).mockResolvedValue(BUYERS);
  vi.mocked(dashboardApi.dashboardSellerDues).mockResolvedValue({ ...SELLERS_NONE });
  vi.mocked(dashboardApi.dashboardDeadAmount).mockResolvedValue(EMPTY_DEAD);
  vi.mocked(dashboardApi.dashboardLentOpen).mockResolvedValue(EMPTY_LENT);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the total dues page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const open = () => renderApp('/dues/total');
  const card = (titleId: string): HTMLElement => {
    const heading = screen.getByRole('heading', { name: (name) => visible(name) === m(titleId) });
    const found = heading.closest('section');
    if (found === null) throw new Error(`no card ${titleId}`);
    return found;
  };

  it('has four sections, each with its count and total, and the grand total is the last line', async () => {
    await open();
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('total-dues-title'),
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="21"]')).not.toBeNull();
    });
    // The buyers' card: two people, 21,564 together (the backend's), and his list.
    const buyers = visible(card('total-dues-buyers').textContent);
    expect(buyers).toContain(
      m('total-dues-section-total', { money: '21,564' }).replace(/\s+/g, ' '),
    );
    expect(buyers).toContain(m('dashboard-tile-dead-note', { count: 2 }));
    // The others say nobody is in them, in words.
    expect(visible(card('total-dues-sellers').textContent)).toContain(
      m('total-dues-sellers-empty'),
    );
    expect(visible(card('total-dues-dead').textContent)).toContain(m('total-dues-dead-empty'));
    expect(visible(card('total-dues-others').textContent)).toContain(m('total-dues-others-empty'));
    // The last line is the tile's figure, the backend's grand total, never a sum of the sections.
    const facts = Array.from(document.querySelectorAll('dl'));
    const last = visible(facts.at(-1)?.textContent);
    expect(last).toContain(m('dashboard-tile-dues-total'));
    expect(last).toContain('21,564');
  });

  it('every section with someone shows its list and total; each row opens that person’s history', async () => {
    vi.mocked(dashboardApi.dashboardTotalDues).mockResolvedValue(TOTAL_DUES_FULL);
    vi.mocked(dashboardApi.dashboardSellerDues).mockResolvedValue({
      ...SELLERS_NONE,
      rows: [
        {
          party: party(50, '0014', 'کریم', 'Karim'),
          city: null,
          balance: { amount_paisa: rupees(1_500), side: 'dr' },
          payable_paisa: rupees(0),
          advance_open_paisa: rupees(0),
          owes_paisa: rupees(1_500),
          last_change_date: '2026-10-10',
        },
      ],
      total_count: 1,
    });
    vi.mocked(dashboardApi.dashboardDeadAmount).mockResolvedValue(DEAD);
    vi.mocked(dashboardApi.dashboardLentOpen).mockResolvedValue({
      ...LENT_OPEN,
      rows: LENT_OPEN.rows.slice(0, 1),
      total_count: 1,
    });
    const user = userEvent.setup();
    const { router } = await open();
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="50"]')).not.toBeNull();
    });
    expect(visible(card('total-dues-dead').textContent)).toContain('18,684');
    expect(visible(card('total-dues-others').textContent)).toContain('2,000');
    const grand = visible(Array.from(document.querySelectorAll('dl')).at(-1)?.textContent);
    expect(grand).toContain('46,748');
    // A seller who owes the shop opens his ledger on the seller side.
    const cell = document
      .querySelector<HTMLElement>('.ag-row[row-id="50"]')
      ?.querySelector<HTMLElement>('[col-id="seller"]');
    if (cell === null || cell === undefined) throw new Error('no cell');
    await user.click(cell);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/party-ledgers');
    });
    expect(router.state.location.search).toMatchObject({ side: 'seller', party: 50 });
  });

  // A dead buyer or seller opens the same history as the buyers and sellers sections: پارٹی کھاتے, one row
  // for each بکری, with his side (PRD FR-19.8 v1.43, FR-9.7, NFR-USE-10; plan 80 R5-3). Not R13. A person who
  // borrowed keeps his loans page.
  describe.each([
    ['buyer', 'buyer', '/bikri/party-ledgers', { side: 'buyer', party: 40 }],
    ['seller', 'seller', '/bikri/party-ledgers', { side: 'seller', party: 40 }],
    ['money_lent', 'a person who borrowed', '/rokar/lent', { type: 'money_lent' }],
  ] as const)('a dead %s row', (partyType, _label, pathname, search) => {
    it(`opens ${pathname}`, async () => {
      vi.mocked(dashboardApi.dashboardTotalDues).mockResolvedValue(TOTAL_DUES_FULL);
      const row = DEAD.rows[0];
      if (row === undefined) throw new Error('no dead row');
      vi.mocked(dashboardApi.dashboardDeadAmount).mockResolvedValue({
        ...DEAD,
        rows: [{ ...row, party_type: partyType }],
      });
      const user = userEvent.setup();
      const { router } = await open();
      await waitFor(() => {
        expect(document.querySelector('.ag-row[row-id="5"]')).not.toBeNull();
      });
      const cell = document
        .querySelector<HTMLElement>('.ag-row[row-id="5"]')
        ?.querySelector<HTMLElement>('[col-id="party"]');
      if (cell === null || cell === undefined) throw new Error('no cell');
      await user.click(cell);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe(pathname);
      });
      expect(router.state.location.search).toMatchObject(search);
    });
  });

  it('a refused read of the totals is an error with a retry, and the lists still work', async () => {
    vi.mocked(dashboardApi.dashboardTotalDues).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
    );
    const user = userEvent.setup();
    await open();
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(m('total-dues-failed'));
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="21"]')).not.toBeNull();
    });
    await user.click(within(alert).getByRole('button', { name: m('common-retry') }));
    await waitFor(() => {
      expect(screen.queryByRole('alert')).toBeNull();
    });
  });
});
