// A garden with about 90 goods arrived entries (plan 96, PRD FR-12.40, FR-12.41, FR-19.11, NFR-USE-16),
// in the Urdu and the English app: the list is 30 rows a page, newest first, searched by a number, an
// item or another shop, with the backend's totals; the details of the entry that has focus are beside
// the list (a window of their own on a narrow screen), with the buttons first; Enter goes to the
// buttons and Esc back to the list; a saved entry shows first, tinted, and the page stays where it was;
// the figures of the garden fold. The list answers like the backend's (tests/helpers/arrivals-many.ts).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ArrivalRow } from '../../src/bindings/ArrivalRow';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import * as arrivals from '../helpers/arrivals-real';
import { listArrivals, manyArrivals } from '../helpers/arrivals-many';
import { named } from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import { typeArrivalLine } from '../helpers/goods-lines';
import { stubLayoutStep } from '../helpers/layout-step';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { rangeText } from '../helpers/pager';
import { fakeItemPickers } from '../helpers/trips';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/shared/edit/api');
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
  fakeItemPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  vi.unstubAllGlobals();
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

const rowCell = (id: string, field: string) =>
  visible(document.querySelector(`.ag-row[row-id="${id}"] [col-id="${field}"]`)?.textContent);
/** The cell of a row of the list, or a clear failure when it is not drawn. */
const cellOf = (id: string, field: string): HTMLElement => {
  const found = document.querySelector<HTMLElement>(`.ag-row[row-id="${id}"] [col-id="${field}"]`);
  if (found === null) throw new Error(`no cell ${field} in row ${id}`);
  return found;
};
const rowOf = (id: string) => document.querySelector(`.ag-row[row-id="${id}"]`);

describe.each(LANGUAGES)('a garden with many goods arrived entries (%s)', (language) => {
  let all: ArrivalRow[];
  const itemName = (row: ArrivalRow) => (language === 'ur' ? row.item.name_ur : row.item.name_en);
  /** An entry of the garden by its number, or a clear failure when the test asks for one that is not there. */
  const entryOf = (number: number): ArrivalRow => {
    const found = all.find((row) => row.number === number);
    if (found === undefined) throw new Error(`no entry ${String(number)}`);
    return found;
  };

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    all = manyArrivals(90);
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockImplementation((input) =>
      Promise.resolve(listArrivals(all, input)),
    );
  });

  const open = async () => {
    const view = await renderApp('/gardens/1');
    await screen.findByText(said(message(language, 'gardens-detail-title', { code: '0080' })));
    await waitFor(() => {
      expect(rowIds().length).toBeGreaterThan(0);
    });
    return { ...view, user: userEvent.setup() };
  };
  /**
   * The rows of the goods arrived list that are drawn, by their id. A grid with no layout (jsdom) draws only the
   * rows at its top, about ten; the page has 30 (the request and the pager say so).
   */
  const rowIds = () =>
    Array.from(
      screen
        .queryByRole('group', { name: named(message(language, 'gardens-arrival-label')) })
        ?.querySelectorAll('.ag-row[row-id]') ?? [],
    )
      .filter((row) => row.closest('.ag-grid-pinned-bottom-rows-container') === null)
      .map((row) => row.getAttribute('row-id') ?? '');
  /** The words of the pager under the list: "1 to 30 of 90" and the page. */
  const pagerText = () =>
    visible(
      screen.getAllByRole('navigation', { name: named(message(language, 'common-pager-label')) })[0]
        ?.textContent,
    );
  const details = () => screen.getByRole('complementary');
  const searchBox = () =>
    screen.getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'gardens-arrival-search-label')),
    });
  const cardTitle = (number: number) =>
    message(language, 'gardens-arrival-card-title', {
      number: message(language, 'gardens-arrival-number', { number }),
      item: itemName(entryOf(number)),
    });
  const lastAsk = () => vi.mocked(gardensApi.gardensArrivalsList).mock.calls.at(-1)?.[0];

  it('the list is 30 rows a page, the newest first, and the totals and the count are over all 90', async () => {
    await open();
    const ids = rowIds();
    expect(ids[0]).toBe('90');
    expect(ids.slice(0, 3)).toEqual(['90', '89', '88']);
    expect(lastAsk()).toMatchObject({
      garden_id: 1,
      offset: 0,
      limit: 30,
      search: null,
      sort: { column: 'entry_date', direction: 'desc' },
    });
    // The count and the page of the pager are the backend's.
    await waitFor(() => {
      expect(pagerText()).toContain(rangeText(language, 1, 30, 90));
    });
    // The total of "left" is the backend's, over the whole filter, never over the 30 rows on screen.
    const pinned = document.querySelector('.ag-row-pinned');
    const left = all.reduce((sum, row) => sum + row.qty_left, 0);
    expect(visible(pinned?.querySelector('[col-id="qty_left"]')?.textContent)).toBe(String(left));
  });

  it('the next page asks the backend for the next 30', async () => {
    const { user } = await open();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'common-pager-next')) }),
    );
    await waitFor(() => {
      expect(rowIds()[0]).toBe('60');
    });
    expect(lastAsk()).toMatchObject({ offset: 30, limit: 30 });
  });

  it('a search by item narrows the whole list, and a search that finds nothing says so and clears', async () => {
    const { user } = await open();
    await user.type(searchBox(), language === 'ur' ? 'آم' : 'mango');
    await waitFor(() => {
      expect(lastAsk()?.search).toBe(language === 'ur' ? 'آم' : 'mango');
    });
    await waitFor(() => {
      expect(pagerText()).toContain(rangeText(language, 1, 18, 18));
    });
    for (const id of rowIds()) expect(rowCell(id, 'item')).toContain(itemName(entryOf(Number(id))));
    await user.clear(searchBox());
    await user.type(searchBox(), 'zzz');
    expect(
      await screen.findByText(said(message(language, 'gardens-arrival-no-match'))),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: said(message(language, 'gardens-arrival-search-clear')) }),
    );
    await waitFor(() => {
      expect(pagerText()).toContain(rangeText(language, 1, 30, 90));
    });
    expect(searchBox().value).toBe('');
  });

  it('a search by number finds one entry, and its details show', async () => {
    const { user } = await open();
    await user.type(searchBox(), 'GA-12');
    await waitFor(() => {
      expect(rowIds()).toEqual(['12']);
    });
    expect(within(details()).getByText(said(cardTitle(12)))).toBeInTheDocument();
  });

  it('the details are beside the list: a click on another entry shows that one, with its sales, and one at a time', async () => {
    const { user } = await open();
    expect(within(details()).getByText(said(cardTitle(90)))).toBeInTheDocument();
    await user.click(cellOf('87', 'number'));
    expect(await within(details()).findByText(said(cardTitle(87)))).toBeInTheDocument();
    expect(within(details()).queryByText(said(cardTitle(90)))).toBeNull();
    // Its two sales, with the buttons of the entry in one block above its figures.
    const sales = within(details()).getByRole('table', {
      name: named(message(language, 'gardens-arrival-lots')),
    });
    expect(within(sales).getAllByRole('row')).toHaveLength(3);
    expect(document.querySelectorAll('[data-arrival-actions]')).toHaveLength(1);
    expect(screen.getAllByRole('complementary')).toHaveLength(1);
  });

  it('a long history of sales shows the first four and the rest on a button', async () => {
    const lots = Array.from({ length: 7 }, (_, index) => ({
      bikri_id: 500 + index,
      bikri_number: 500 + index,
      lot_id: 900 + index,
      lot_no: 1,
      lot_counter: index + 1,
      entry_date: '2026-10-20',
      qty_sold: 5,
    }));
    all = all.map((row) => (row.number === 90 ? { ...row, lots } : row));
    const { user } = await open();
    const table = () =>
      within(details()).getByRole('table', {
        name: named(message(language, 'gardens-arrival-lots')),
      });
    expect(within(table()).getAllByRole('row')).toHaveLength(5);
    await user.click(
      within(details()).getByRole('button', {
        name: said(message(language, 'gardens-arrival-lots-more', { count: 7 })),
      }),
    );
    expect(within(table()).getAllByRole('row')).toHaveLength(8);
  });

  it('Enter on an entry goes to the first button of its details, and Esc goes back to the list', async () => {
    const { user } = await open();
    await user.click(cellOf('88', 'number'));
    await within(details()).findByText(said(cardTitle(88)));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(details().contains(document.activeElement)).toBe(true);
    });
    expect(document.activeElement?.tagName).toBe('BUTTON');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(document.querySelector('.list-grid')?.contains(document.activeElement)).toBe(true);
    });
    // Still on the same entry.
    expect(within(details()).getByText(said(cardTitle(88)))).toBeInTheDocument();
  });

  it('on a narrow window the details are a window of their own: Enter opens it, Esc closes it', async () => {
    stubLayoutStep('compact');
    const { user } = await open();
    expect(screen.queryByRole('complementary')).toBeNull();
    await user.click(cellOf('88', 'number'));
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByText(said(cardTitle(88)))).toBeInTheDocument();
    // The buttons of the entry are in the window, in one block.
    expect(dialog.querySelectorAll('[data-arrival-actions]')).toHaveLength(1);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });

  it(
    'a saved goods arrived entry shows first in the list, tinted and in the details; the search is cleared and the page stays',
    { timeout: 30_000 },
    async () => {
      vi.mocked(gardensApi.gardensArrivalEntryPreview).mockResolvedValue({
        lines: [],
        errors: [],
        warnings: [],
      });
      vi.mocked(gardensApi.gardensArrivalEntrySave).mockImplementation(() => {
        const made: ArrivalRow = {
          ...arrivals.ARRIVAL_SAVE.row,
          id: 91,
          number: 91,
          entry_date: '2026-10-31',
          sell_date: '2026-10-31',
          qty: 100,
          qty_left: 100,
          qty_our_left: 100,
        };
        all = [...all, made];
        return Promise.resolve({ rows: [made], warnings: [] });
      });
      const { user } = await open();
      await user.type(searchBox(), 'GA-12');
      await waitFor(() => {
        expect(rowIds()).toEqual(['12']);
      });
      await user.click(
        within(
          screen.getByRole('group', { name: named(message(language, 'gardens-type-label')) }),
        ).getByRole('button', { name: named(message(language, 'gardens-type-arrival')) }),
      );
      await typeArrivalLine(user, 0, {
        item: language === 'ur' ? 'م' : 'Ora',
        packing: language === 'ur' ? 'پ' : 'Box',
        qty: '100',
      });
      press('F10');
      // A save, the list asked again and drawn: slow on a busy computer. The new entry is the first row of the
      // list (the grid places rows by their index, not by their place in the page).
      await waitFor(
        () => {
          expect(rowOf('91')?.getAttribute('row-index')).toBe('0');
        },
        { timeout: 8000 },
      );
      expect(searchBox().value).toBe('');
      await waitFor(() => {
        expect(pagerText()).toContain(rangeText(language, 1, 30, 91));
      });
      expect(rowOf('91')?.classList.contains('row-marked')).toBe(true);
      expect(
        await within(details()).findByText(
          (text) => visible(text).includes('GA-91') || visible(text).includes('91'),
        ),
      ).toBeInTheDocument();
      // The form is ready for the next entry, where it was.
      expect(document.querySelectorAll('.list-grid')).not.toHaveLength(0);
    },
  );

  it('an error says so in plain words and the retry button asks again; a garden with no entry says what to do', async () => {
    // The form's own list of the entries asks too, so every ask fails until the backend is back.
    vi.mocked(gardensApi.gardensArrivalsList).mockRejectedValue(
      fail({ code: 'ERR_INTERNAL', field: null, values: {} }),
    );
    const view = await renderApp('/gardens/1');
    await screen.findByText(said(message(language, 'gardens-detail-title', { code: '0080' })));
    expect(
      await screen.findByText(said(message(language, 'gardens-arrival-failed'))),
    ).toBeInTheDocument();
    vi.mocked(gardensApi.gardensArrivalsList).mockImplementation((input) =>
      Promise.resolve(listArrivals(all, input)),
    );
    await userEvent.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    await waitFor(() => {
      expect(rowIds()[0]).toBe('90');
    });
    view.unmount();
    all = [];
    await renderApp('/gardens/1');
    expect(
      await screen.findByText(said(message(language, 'gardens-arrival-empty'))),
    ).toBeInTheDocument();
    // Nothing to search in: no search box until the first entry is saved.
    expect(
      screen.queryByRole('textbox', {
        name: named(message(language, 'gardens-arrival-search-label')),
      }),
    ).toBeNull();
  });

  it('on a narrow window a button of the details closes their window and opens its own box', async () => {
    stubLayoutStep('compact');
    const { user } = await open();
    await user.click(cellOf('88', 'number'));
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog');
    await user.click(
      within(dialog).getByRole('button', {
        name: said(message(language, 'purchases-change-sell-date')),
      }),
    );
    // The box of the day to sell is the only window left.
    expect(
      await screen.findByText(said(message(language, 'gardens-arrival-sell-date-title'))),
    ).toBeInTheDocument();
    expect(screen.getAllByRole('dialog')).toHaveLength(1);
  });

  it('the key figures are always in view; the full figures and each partner fold under one heading', async () => {
    const { user } = await open();
    const fold = screen
      .getByRole('button', { name: named(message(language, 'gardens-figures-title')) })
      .closest('section');
    if (fold === null) throw new Error('the figures have no section');
    // The four figures of the head of the garden.
    for (const id of [
      'gardens-balance',
      'gardens-rent-due',
      'glossary-commission',
      'gardens-fact-money-in',
    ]) {
      expect(screen.getAllByText(said(message(language, id))).length).toBeGreaterThan(0);
    }
    expect(
      within(fold).queryByRole('table', {
        name: named(message(language, 'gardens-partner-results-title')),
      }),
    ).toBeNull();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'gardens-figures-title')) }),
    );
    expect(
      await within(fold).findByRole('table', {
        name: named(message(language, 'gardens-partner-results-title')),
      }),
    ).toBeInTheDocument();
  });
});
