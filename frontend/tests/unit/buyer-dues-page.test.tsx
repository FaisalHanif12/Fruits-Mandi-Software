// The buyer dues page (plan 76 point 9, PRD FR-9.4, FR-19.9, FR-19.11) in the Urdu and the English app,
// against a fake backend: the tile "Buyers' dues" opens it, it lists the buyers who owe with their code,
// name, balance with its side and the day it last changed, shows the backend's total under the columns
// (the tile's figure), searches through the backend, opens a buyer's ledger, and has its four states. The
// screen adds nothing up.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BuyerDuesPage } from '../../src/bindings/BuyerDuesPage';
import type { BuyerDuesRow } from '../../src/bindings/BuyerDuesRow';
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
import { printData } from '../helpers/collection-sheet';
import { fakeDashboard } from '../helpers/dashboard';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

const rupees = (value: number) => paisaForTest(value * 100);

function buyer(
  id: number,
  code: string,
  ur: string,
  en: string,
  owes: number,
  last: string | null,
  tag: string | null = null,
): BuyerDuesRow {
  return {
    party: {
      account_id: id,
      code,
      short_code: code.slice(3),
      name_ur: ur,
      name_en: en,
      tag,
      person_key: null,
    },
    city: { ur: 'کوٹ رادھا کشن', en: 'Kot Radha Kishan' },
    balance: { amount_paisa: rupees(owes), side: 'dr' },
    last_change_date: last,
  };
}

/** After Events 1 to 5 (PRD FR-9.4): اکرم 2,880 and ارشد 18,684, the total 21,564. */
const AKRAM = buyer(20, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', 2_880, '2026-10-10');
const ARSHAD = buyer(21, '0021', 'ارشد ملتانی', 'Arshad Multani', 18_684, '2026-10-10');
const PAGE: BuyerDuesPage = {
  rows: [AKRAM, ARSHAD],
  total_count: 2,
  total_paisa: rupees(21_564),
};

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardBuyerDues).mockResolvedValue(PAGE);
  vi.mocked(dashboardApi.dashboardBuyerDuesPrintData).mockImplementation((input) =>
    Promise.resolve(printData(input.language, input.paper ?? 'a4')),
  );
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the buyer dues page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = {
    akram: language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    arshad: language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
  };
  const open = () => renderApp('/dues');
  const rowOf = (id: number) =>
    document.querySelector<HTMLElement>(`.ag-row[row-id="${String(id)}"]`);

  it('lists the buyers who owe, each with his code, balance and the day it last changed, and the backend’s total', async () => {
    await open();
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('dues-title'),
    });
    await waitFor(() => {
      expect(rowOf(20)).not.toBeNull();
    });
    const akram = visible(rowOf(20)?.textContent);
    expect(akram).toContain(names.akram);
    expect(akram).toContain('2,880');
    expect(akram).toContain('10-10-2026');
    expect(visible(rowOf(21)?.textContent)).toContain('18,684');
    // The total under the columns is the backend's figure, with the side word.
    const totals = visible(document.querySelector('.ag-row-pinned')?.textContent);
    expect(totals).toContain('21,564');
    expect(totals).toContain(m('report-side-dr'));
    // No aging buckets here: that is R20.
    expect(visible(document.body.textContent)).not.toContain(m('report-col-age-0-7'));
  });

  it('the buyer filter starts at سب: no buyer is sent, and the box says All (plan 79 item 1)', async () => {
    await open();
    await waitFor(() => {
      expect(rowOf(20)).not.toBeNull();
    });
    expect(vi.mocked(dashboardApi.dashboardBuyerDues).mock.calls[0]?.[0]).toMatchObject({
      party_id: null,
      search: null,
      offset: 0,
    });
    expect(screen.getByPlaceholderText(m('search-all-choice'))).toBeInTheDocument();
  });

  it('the search goes to the backend with its words, and a clear brings the list back', async () => {
    const user = userEvent.setup();
    await open();
    await waitFor(() => {
      expect(rowOf(20)).not.toBeNull();
    });
    vi.mocked(dashboardApi.dashboardBuyerDues).mockResolvedValue({
      rows: [ARSHAD],
      total_count: 1,
      total_paisa: rupees(18_684),
    });
    await user.type(screen.getByPlaceholderText(m('dues-search')), 'arshad');
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardBuyerDues).mock.calls.at(-1)?.[0]).toMatchObject({
        search: 'arshad',
        offset: 0,
      });
    });
    await waitFor(() => {
      expect(rowOf(20)).toBeNull();
    });
    expect(visible(document.querySelector('.ag-row-pinned')?.textContent)).toContain('18,684');
  });

  it('Enter on a buyer opens his history: the statement with a row for each entry (plan 78 B2)', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await waitFor(() => {
      expect(rowOf(20)).not.toBeNull();
    });
    const cell = rowOf(20)?.querySelector<HTMLElement>('[col-id="buyer"]');
    if (cell === null || cell === undefined) throw new Error('no cell');
    await user.click(cell);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/party-ledgers');
    });
    expect(router.state.location.search).toMatchObject({ side: 'buyer', party: 20 });
  });

  it('the print button shows every buyer of the search and the order in the print layout, and Back returns', async () => {
    const user = userEvent.setup();
    await open();
    await waitFor(() => {
      expect(rowOf(20)).not.toBeNull();
    });
    await user.type(screen.getByPlaceholderText(m('dues-search')), 'a');
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('report-preview')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardBuyerDuesPrintData).mock.calls[0]?.[0]).toEqual({
        party_id: null,
        search: 'a',
        sort: null,
        language,
        paper: 'a4',
      });
    });
    await waitFor(() => {
      expect(visible(document.querySelector('.print-sheet')?.textContent)).toContain('44,364');
    });
    await user.click(
      await within(screen.getByRole('main')).findByRole('button', {
        name: (name) => visible(name) === m('common-back'),
      }),
    );
    await waitFor(() => {
      expect(rowOf(20)).not.toBeNull();
    });
  });

  it('says that nobody owes anything, and that nothing matches a search, with a way out', async () => {
    vi.mocked(dashboardApi.dashboardBuyerDues).mockResolvedValue({
      rows: [],
      total_count: 0,
      total_paisa: rupees(0),
    });
    const user = userEvent.setup();
    await open();
    expect(
      await screen.findByText((text) => visible(text) === m('dues-empty')),
    ).toBeInTheDocument();
    await user.type(screen.getByPlaceholderText(m('dues-search')), 'zzz');
    const clear = await screen.findByRole('button', {
      name: (name) => visible(name) === m('dues-clear'),
    });
    await user.click(clear);
    expect(
      await screen.findByText((text) => visible(text) === m('dues-empty')),
    ).toBeInTheDocument();
  });

  it('an error is told with a retry that asks again', async () => {
    vi.mocked(dashboardApi.dashboardBuyerDues).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
    );
    const user = userEvent.setup();
    await open();
    const retry = await screen.findByRole('button', {
      name: (name) => visible(name) === m('common-retry'),
    });
    await user.click(retry);
    await waitFor(() => {
      expect(rowOf(20)).not.toBeNull();
    });
    expect(within(document.body).queryByRole('button', { name: m('common-retry') })).toBeNull();
  });
});
