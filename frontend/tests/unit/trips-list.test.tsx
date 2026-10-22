// مارکیٹ سفر, the list of trips (plan 16b, PRD 12.10, FR-5.2, FR-19.11), in the Urdu and the English
// app, against a fake backend: the main bar item opens a real screen, the list and its totals
// (the backend's), the four states, page-by-page loading, the search, and Enter opening a trip.
// Every figure on the screen is a figure the backend sent.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as tripsApi from '../../src/features/trips/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { e34Trip, fakeTrips, tripRow } from '../helpers/trips';
import { refusal } from '../helpers/ui-error';
import { CommandError } from '../../src/shared/api';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

describe.each(LANGUAGES)('مارکیٹ سفر, the list (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const open = async () => {
    const view = await renderApp('/trips');
    await screen.findByRole('heading', { level: 1, name: named(message(language, 'trips-title')) });
    return { ...view, user: userEvent.setup() };
  };

  it('the main bar has the item and it opens this screen, not a coming soon page', async () => {
    fakeTrips();
    const { router, user } = await open();
    expect(router.state.location.pathname).toBe('/trips');
    const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
    const item = within(nav).getByRole('button', { name: message(language, 'layout-nav-trips') });
    expect(item).toHaveAttribute('aria-current', 'page');
    // The side menu of the بکری area is not here, and the item sits after روکڑ, before the partners.
    expect(
      screen.queryByRole('navigation', { name: message(language, 'layout-module-label') }),
    ).toBeNull();
    const labels = within(nav)
      .getAllByRole('button')
      .map((button) => button.getAttribute('aria-label'));
    expect(labels.indexOf(message(language, 'layout-nav-trips'))).toBe(
      labels.indexOf(message(language, 'layout-nav-rokar')) + 1,
    );
    await user.click(
      within(nav).getByRole('button', { name: message(language, 'layout-nav-bikri') }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
  });

  it('lists the trips and the totals the backend sent, asking for the first page of 100', async () => {
    fakeTrips();
    await open();
    await waitFor(() => {
      expect(cell(1, 'number')).toBe(message(language, 'trips-number', { number: 1 }));
    });
    expect(cell(1, 'market')).toBe('Lahore Mandi');
    expect(cell(1, 'status')).toBe(message(language, 'trips-status-open'));
    expect(cell(1, 'qty_left')).toBe('5');
    const pinned = document.querySelector('.ag-row-pinned');
    expect(pinned).not.toBeNull();
    expect(visible(pinned?.querySelector('[col-id="market"]')?.textContent)).toBe(
      message(language, 'glossary-meezan'),
    );
    expect(visible(pinned?.querySelector('[col-id="qty_left"]')?.textContent)).toBe('5');
    expect(tripsApi.tripsList).toHaveBeenCalledWith({
      from: null,
      to: null,
      status: null,
      partner_id: null,
      search: null,
      offset: 0,
      limit: 100,
    });
    expect(
      screen.getByText(said(message(language, 'trips-list-count', { shown: 1, total: 1 }))),
    ).toBeInTheDocument();
  });

  it('an empty list says why and has a button that starts the first trip (FR-7.46)', async () => {
    fakeTrips({ trips: [] });
    const { router, user } = await open();
    expect(
      await screen.findByText(said(message(language, 'trips-list-empty'))),
    ).toBeInTheDocument();
    const buttons = screen
      .getAllByRole('button')
      .filter((button) => visible(button.textContent).includes(message(language, 'trips-new')));
    expect(buttons.length).toBeGreaterThan(0);
    await user.click(buttons[buttons.length - 1] ?? document.body);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/trips/new');
    });
  });

  it('an error says so in plain words and the retry button asks again', async () => {
    fakeTrips();
    vi.mocked(tripsApi.tripsList).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
    );
    const { user } = await open();
    expect(
      await screen.findByText(said(message(language, 'trips-list-failed'))),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    await waitFor(() => {
      expect(cell(1, 'number')).toBe(message(language, 'trips-number', { number: 1 }));
    });
  });

  it('loads page by page: the next 100 rows come on the button, the count says how many', async () => {
    const many = Array.from({ length: 120 }, (_, index) =>
      tripRow({ id: index + 1, number: index + 1, market: `Market ${String(index + 1)}` }),
    );
    fakeTrips({ trips: many });
    const { user } = await open();
    await waitFor(() => {
      expect(
        screen.getByText(said(message(language, 'trips-list-count', { shown: 100, total: 120 }))),
      ).toBeInTheDocument();
    });
    await user.click(screen.getByRole('button', { name: message(language, 'trips-list-more') }));
    await waitFor(() => {
      expect(
        screen.getByText(said(message(language, 'trips-list-count', { shown: 120, total: 120 }))),
      ).toBeInTheDocument();
    });
    expect(tripsApi.tripsList).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: 100, limit: 100 }),
    );
    expect(screen.queryByRole('button', { name: message(language, 'trips-list-more') })).toBeNull();
  });

  it('the search is sent to the backend, and a search with no match offers to clear it', async () => {
    fakeTrips({ trips: [e34Trip()] });
    const { user } = await open();
    await waitFor(() => {
      expect(cell(1, 'number')).not.toBe('');
    });
    await user.type(
      screen.getByRole('textbox', { name: named(message(language, 'trips-list-search')) }),
      'zzz',
    );
    expect(
      await screen.findByText(said(message(language, 'trips-list-no-match'))),
    ).toBeInTheDocument();
    expect(tripsApi.tripsList).toHaveBeenLastCalledWith(expect.objectContaining({ search: 'zzz' }));
    await user.click(screen.getByRole('button', { name: message(language, 'trips-list-clear') }));
    await waitFor(() => {
      expect(cell(1, 'number')).toBe(message(language, 'trips-number', { number: 1 }));
    });
  });

  it('Enter on a trip opens it', async () => {
    fakeTrips();
    const { router, user } = await open();
    await waitFor(() => {
      expect(cell(1, 'number')).not.toBe('');
    });
    const target = document.querySelector<HTMLElement>('.ag-row[row-id="1"] [col-id="number"]');
    if (target === null) throw new Error('no cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/trips/1');
    });
  });

  it('F2 starts a new trip from the keyboard', async () => {
    fakeTrips();
    const { router } = await open();
    await waitFor(() => {
      expect(cell(1, 'number')).not.toBe('');
    });
    document.body.dispatchEvent(
      new KeyboardEvent('keydown', { code: 'F2', key: 'F2', bubbles: true }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/trips/new');
    });
  });
});
