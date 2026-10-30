// آج کی بکری, the list of بکری (plan 13, PRD FR-12.7, FR-7.46), in the Urdu and the English app,
// against a fake backend: the columns, the filters, the four states, and opening a row by keyboard.
// All amounts are the stored sums the backend sends.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { listRow } from '../helpers/bikri-saved';
import { named } from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
import { money, ref } from '../helpers/accounts';
import { EMPTY_PAIRS, bikriListOf } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  // Reset, not only clear: a rejection queued with mockRejectedValueOnce by a test that failed early
  // must not reach the next test.
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const pageText = () => visible(document.body.textContent);

function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

describe.each(LANGUAGES)('آج کی بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const open = async () => {
    const view = await renderApp('/bikri/today');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'layout-module-today')),
    });
    return view;
  };

  it('asks for today by default, with the other filters empty', async () => {
    await open();
    await waitFor(() => {
      expect(bikriApi.bikriList).toHaveBeenCalledWith({
        date: '10-10-2026',
        from: null,
        to: null,
        seller_id: null,
        item_id: null,
        status: null,
        offset: 0,
        limit: PAGE_ROWS,
        show_finished: false,
        search: null,
        sort: null,
        group_by_month: false,
      });
    });
  });

  it('shows number, sellers, gross, net and status as the backend stored them', async () => {
    await open();
    await waitFor(() => {
      expect(cell(41, 'number')).toBe('1');
    });
    expect(cell(41, 'sellers')).toBe(language === 'ur' ? 'رحیم' : 'Rahim');
    expect(cell(41, 'gross')).toBe('29,400');
    expect(cell(41, 'net')).toBe('19,316');
    expect(cell(41, 'status')).toBe(message(language, 'bikri-status-saved'));
  });

  it('two sellers of one truck are in one cell, and a one-time seller is shown by his own name', async () => {
    vi.mocked(bikriApi.bikriList).mockResolvedValue(
      bikriListOf([
        listRow({
          sellers: [
            ref(17, '0010', 'رحیم', 'Rahim'),
            ref(22, '0031', 'پختہ کھاتہ (بیوپاری)', 'Pukhta Khata (Seller)'),
          ],
          pukhta_sellers: [
            { name_ur: 'شاہد', name_en: 'Shahid', phone: '03000000000', person_key: 'shahid|0300' },
          ],
        }),
      ]),
    );
    await open();
    await waitFor(() => {
      expect(cell(41, 'sellers')).toBe(language === 'ur' ? 'رحیم، شاہد' : 'Rahim, Shahid');
    });
  });

  it('a cancelled بکری says so in words', async () => {
    vi.mocked(bikriApi.bikriList).mockResolvedValue(
      bikriListOf([listRow({ status: 'cancelled' })]),
    );
    await open();
    await waitFor(() => {
      expect(cell(41, 'status')).toBe(message(language, 'bikri-status-cancelled'));
    });
  });

  it('Enter on a row opens that بکری read-only', async () => {
    const { router } = await open();
    const user = userEvent.setup();
    await waitFor(() => {
      expect(cell(41, 'number')).toBe('1');
    });
    const target = document.querySelector<HTMLElement>('.ag-row[row-id="41"] [col-id="number"]');
    if (target === null) throw new Error('no cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 41 });
  });

  it('a day with no بکری says why and what to do, with the key on the button', async () => {
    vi.mocked(bikriApi.bikriList).mockResolvedValue(bikriListOf([]));
    const { router } = await open();
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-list-empty'));
    });
    expect(pageText()).toContain(message(language, 'bikri-list-empty-help', { key: 'Alt+B' }));
    const user = userEvent.setup();
    const button = screen.getByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'bikri-new')),
    });
    expect(visible(button.textContent)).toContain('Alt+B');
    await user.click(button);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
  });

  it('another day sends the day as typed; nothing found offers to clear the filters', async () => {
    vi.mocked(bikriApi.bikriList).mockResolvedValue(bikriListOf([]));
    await open();
    const user = userEvent.setup();
    const day = await screen.findByRole('textbox', {
      name: named(message(language, 'bikri-header-date')),
    });
    await user.clear(day);
    await user.type(day, '09-10-2026');
    await waitFor(() => {
      expect(bikriApi.bikriList).toHaveBeenLastCalledWith(
        expect.objectContaining({ date: '09-10-2026' }),
      );
    });
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-list-no-match'));
    });
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'bikri-list-clear')) }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriList).toHaveBeenLastCalledWith(
        expect.objectContaining({ date: '10-10-2026' }),
      );
    });
  });

  it('the status filter, behind "More filters", sends cancelled', async () => {
    await open();
    const user = userEvent.setup();
    // The seller, the item and the status are the rare filters: they are behind "More filters".
    expect(
      screen.queryByRole('combobox', { name: named(message(language, 'bikri-list-col-status')) }),
    ).not.toBeInTheDocument();
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'common-filters-more')) }),
    );
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'bikri-list-col-status')),
      }),
    );
    await user.click(
      await screen.findByRole('option', {
        name: named(message(language, 'bikri-status-cancelled')),
      }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriList).toHaveBeenLastCalledWith(
        expect.objectContaining({ status: 'cancelled' }),
      );
    });
  });

  it('the search box sends the text as `search`, and the tick sends show_finished (off by default)', async () => {
    await open();
    const user = userEvent.setup();
    await waitFor(() => {
      expect(bikriApi.bikriList).toHaveBeenLastCalledWith(
        expect.objectContaining({ search: null, show_finished: false, sort: null }),
      );
    });
    await user.type(
      await screen.findByRole('textbox', { name: named(message(language, 'common-search-label')) }),
      'LES',
    );
    await waitFor(() => {
      expect(bikriApi.bikriList).toHaveBeenLastCalledWith(
        expect.objectContaining({ search: 'LES' }),
      );
    });
    await user.click(
      await screen.findByRole('checkbox', { name: named(message(language, 'common-show-closed')) }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriList).toHaveBeenLastCalledWith(
        expect.objectContaining({ show_finished: true }),
      );
    });
  });

  it('the titles of the columns the backend can sort by are sortable, the others are not', async () => {
    await open();
    await waitFor(() => {
      expect(cell(41, 'number')).toBe('1');
    });
    const header = (colId: string) =>
      document.querySelector(`.ag-header-cell[col-id="${colId}"]`)?.classList;
    // The backend sorts by number, day, truck, status and the money columns (BikriListInput.sort).
    for (const colId of ['number', 'gross', 'net', 'status']) {
      expect(header(colId)).toContain('ag-header-cell-sortable');
    }
    expect(header('sellers')).not.toContain('ag-header-cell-sortable');
    // The click on a title is tested with the grid itself (list-sort-months.test.tsx) and was run in
    // a real browser on this page; jsdom does not deliver the header click through the whole page.
  });

  it('an error is told in words and has a retry', async () => {
    vi.mocked(bikriApi.bikriList).mockRejectedValueOnce(
      fail({ code: 'ERR_DB_BUSY', field: null, values: {} }),
    );
    await open();
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-list-failed'));
    });
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'common-retry')) }),
    );
    await waitFor(() => {
      expect(cell(41, 'number')).toBe('1');
    });
  });

  it('the side menu item “today’s sale” opens it', async () => {
    const { router } = await renderApp('/bikri/book');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'layout-module-today')) }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
  });

  it('150 بکری come one page at a time: 25 at once, the count and the totals are the backend’s, Next page loads the next 25', async () => {
    const all = Array.from({ length: 150 }, (_, index) =>
      listRow({ id: 1000 + index, number: index + 1 }),
    );
    vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
      Promise.resolve({
        ...bikriListOf(all, input.offset, input.limit),
        // The totals of the whole filter, not the sum of the rows on screen.
        totals: {
          qty_received: 7_500,
          qty_sold: 7_400,
          gross: money(4_410_000),
          net: money(2_897_400),
          buyer_total: money(4_734_600),
        },
      }),
    );
    await open();
    // The day is known a moment after the screen opens, and then the list asks for that day.
    await waitFor(() => {
      expect(vi.mocked(bikriApi.bikriList).mock.calls.at(-1)?.[0].date).toBe('10-10-2026');
    });
    await waitFor(() => {
      expect(pageText()).toContain(rangeText(language, 1, 25, 150));
    });
    const pinned = (colId: string) =>
      visible(document.querySelector(`.ag-row-pinned [col-id="${colId}"]`)?.textContent);
    await waitFor(() => {
      expect(pinned('gross')).toBe('4,410,000');
    });
    expect(pinned('net')).toBe('2,897,400');
    const user = userEvent.setup();
    await user.click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(pageText()).toContain(rangeText(language, 26, 50, 150));
    });
    expect(vi.mocked(bikriApi.bikriList).mock.calls.at(-1)?.[0]).toMatchObject({
      date: '10-10-2026',
      offset: PAGE_ROWS,
      limit: PAGE_ROWS,
    });
    // The totals are over the whole filter on every page.
    expect(pinned('gross')).toBe('4,410,000');
  });
});
