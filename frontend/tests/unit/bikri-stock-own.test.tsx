// Trip goods in باقی مال (plan 16b, PRD 12.25, FR-12.28), in the Urdu and the English app, against
// a fake backend: the purchases of open trips with goods left are listed apart, each marked with its
// purchase number and its trip; the card has the figures and the write-off and its undo, which post
// nothing; "Open the trip" leads to the trip screen with that purchase chosen. Every figure on the
// screen is a figure the backend sent.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { OwnStockRow } from '../../src/bindings/OwnStockRow';
import * as bikriApi from '../../src/features/bikri/api';
import * as purchasesApi from '../../src/features/purchases/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { money } from '../helpers/accounts';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { stockList } from '../helpers/bikri-plan16-data';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { e34OwnStockRow, fakeTrips } from '../helpers/trips';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const OWN_ROW: OwnStockRow = e34OwnStockRow();

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

describe.each(LANGUAGES)('trip goods in باقی مال (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    fakePlan16({ rows: [] });
    fakeTrips();
    vi.mocked(bikriApi.bikriStockList).mockResolvedValue({
      ...stockList([]),
      own_rows: [OWN_ROW],
      own_balance_total: 5,
      own_cost_left_paisa: money(2_600),
    });
  });

  const open = async () => {
    const view = await renderApp('/bikri/stock');
    await screen.findByText(said(message(language, 'bikri-stock-own-title')));
    return { ...view, user: userEvent.setup() };
  };

  it('lists the trip’s purchase apart, with its number and trip, and the backend’s totals', async () => {
    await open();
    const row = await waitFor(() => {
      const found = document.querySelector('.ag-row[row-id="7"]');
      if (found === null) throw new Error('no row');
      return found;
    });
    expect(visible(row.querySelector('[col-id="number"]')?.textContent)).toBe(
      message(language, 'purchases-number', { number: 1 }),
    );
    expect(visible(row.querySelector('[col-id="trip"]')?.textContent)).toBe(
      message(language, 'trips-number', { number: 1 }),
    );
    expect(visible(row.querySelector('[col-id="balance"]')?.textContent)).toBe('5');
    // The list of lots is empty but the screen is not "empty": there is trip stock.
    expect(screen.queryByText(said(message(language, 'bikri-stock-empty')))).toBeNull();
    const pinned = document.querySelectorAll('.ag-row-pinned');
    expect(pinned.length).toBeGreaterThan(0);
  });

  it('the card has the figures, and the write-off asks for a reason and posts nothing', async () => {
    const { user } = await open();
    const cell = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="7"] [col-id="number"]');
      if (found === null) throw new Error('no cell');
      return found;
    });
    await user.click(cell);
    const card = (
      await screen.findByText(
        said(
          message(language, 'bikri-stock-own-card-title', {
            number: message(language, 'purchases-number', { number: 1 }),
            item: language === 'ur' ? 'مالٹا' : 'Orange',
          }),
        ),
      )
    ).closest('section');
    if (card === null) throw new Error('no card');
    const text = visible(card.textContent);
    for (const figure of ['10,400', '2,600', 'Haji Nazir']) expect(text).toContain(figure);
    await user.click(
      within(card).getByRole('button', {
        name: message(language, 'purchases-write-off'),
      }),
    );
    expect(
      await screen.findByText(said(message(language, 'purchases-write-off-title'))),
    ).toBeInTheDocument();
    await user.type(
      screen.getByRole('textbox', {
        name: (name) => visible(name) === message(language, 'purchases-write-off-qty'),
      }),
      '5',
    );
    await user.type(
      screen.getByRole('combobox', {
        name: (name) => visible(name) === message(language, 'purchases-write-off-reason'),
      }),
      'Spoiled',
    );
    await user.click(
      screen.getByRole('button', { name: message(language, 'purchases-write-off-confirm') }),
    );
    await waitFor(() => {
      expect(purchasesApi.purchasesWriteOff).toHaveBeenCalledWith(7, '5', 'Spoiled');
    });
  });

  it('"open the trip" leads to the trip with this purchase chosen', async () => {
    const { router, user } = await open();
    const cell = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="7"] [col-id="number"]');
      if (found === null) throw new Error('no cell');
      return found;
    });
    await user.click(cell);
    await user.click(
      await screen.findByRole('button', { name: message(language, 'bikri-stock-own-open') }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/trips/1');
    });
    expect(router.state.location.search).toMatchObject({ purchase: 7 });
  });
});
