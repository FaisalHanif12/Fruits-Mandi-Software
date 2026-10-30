// The پختہ کھاتہ accounts (plan 13, PRD FR-7.29, FR-13.2, FR-13.6): the one merged list of known
// one-time people in the name and phone fields of a lot and a line, and the register screen. In
// the Urdu and the English app, by keyboard, against a fake backend.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { money } from '../helpers/accounts';
import { fakeBikri } from '../helpers/bikri';
import { NASIR, SHAHID, registerRow } from '../helpers/bikri-saved';
import { seedBikriDraft } from '../helpers/bikri-seed';
import { named, openBook } from '../helpers/bikri-screen';
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
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS, pukhtaRegisterOf } from '../helpers/paged';

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

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

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
const PEOPLE = [SHAHID, NASIR];

/** The fake search: a person matches when the text is in a name or in the phone. */
function fakePeople() {
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockImplementation((text) => {
    const needle = text.toLowerCase();
    return Promise.resolve(
      PEOPLE.filter((person) =>
        [person.name_ur, person.name_en.toLowerCase(), person.phone].some((part) =>
          part.includes(needle),
        ),
      ),
    );
  });
}

describe.each(LANGUAGES)('known one-time people in the sale book (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  /**
   * Opens the book on a draft whose first line is for the one-time buyer account, and puts the
   * cursor in his name (the Urdu name in the Urdu app, the English name in the English app).
   */
  async function atTheName() {
    seedBikriDraft({ pukhtaBuyer: true, line: { qty: '20', rate: '600' } });
    const view = await openBook(language);
    // openBook sets the fake backend up, so the people are set after it.
    fakePeople();
    const panel = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('[data-pukhta-for]');
      if (found === null) throw new Error('the one-time person has no fields yet');
      return found;
    }, SLOW_MS);
    const first = panel.querySelector('input');
    if (first === null) throw new Error('no name field');
    await view.user.click(first);
    expect(document.activeElement).toBe(first);
    return view;
  }

  const options = () => screen.findAllByRole('option');
  const lineOfDraft = (view: Awaited<ReturnType<typeof atTheName>>) =>
    view.backend.lastDraft().lots[0]?.sales[0];

  it('while a name is typed the people already recorded come first, with both names, phone and last day', async () => {
    const { user } = await atTheName();
    await user.keyboard('shah');
    await waitFor(() => {
      expect(pukhtaApi.searchPukhtaPeople).toHaveBeenLastCalledWith('shah', 'buyer');
    });
    const rows = (await options()).map((row) => visible(row.textContent));
    expect(rows).toHaveLength(1);
    expect(rows[0]).toContain('شاہد');
    expect(rows[0]).toContain('Shahid');
    expect(rows[0]).toContain('03000000000');
    expect(rows[0]).toContain('10-10-2026');
  });

  it('↓ then Enter takes the person: both names and the phone are his, exactly as recorded', async () => {
    const view = await atTheName();
    await view.user.keyboard('shah');
    await options();
    await view.user.keyboard('{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(lineOfDraft(view)?.pukhta).toEqual({
        name_ur: 'شاہد',
        name_en: 'Shahid',
        phone: '03000000000',
      });
    });
    // Both names are shown in the two fields, and no word is marked "check the spelling".
    expect(pageText()).not.toContain(message(language, 'translit-spelling-check'));
  });

  it('Enter with nothing moved to is a plain Enter: a new person is not replaced by an old one', async () => {
    const view = await atTheName();
    await view.user.keyboard('shah');
    await options();
    await view.user.keyboard('{Enter}');
    // Nobody was taken: the name is what was typed, and focus moved on to the next field.
    await waitFor(() => {
      expect(lineOfDraft(view)?.pukhta?.phone).toBe('');
    });
    expect(lineOfDraft(view)?.pukhta?.name_ur).not.toBe('شاہد');
  });

  it('the phone field lists the people too, and taking one fills the names as well', async () => {
    const view = await atTheName();
    const phone = screen.getByRole('combobox', {
      name: named(message(language, 'bikri-pukhta-phone')),
    });
    await view.user.click(phone);
    await view.user.keyboard('0311');
    await waitFor(() => {
      expect(pukhtaApi.searchPukhtaPeople).toHaveBeenLastCalledWith('0311', 'buyer');
    });
    const rows = (await options()).map((row) => visible(row.textContent));
    expect(rows[0]).toContain('Nasir');
    await view.user.keyboard('{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(lineOfDraft(view)?.pukhta).toEqual({
        name_ur: 'ناصر',
        name_en: 'Nasir',
        phone: '03111111111',
      });
    });
  });

  it('a lot’s one-time seller asks for the sellers’ list', async () => {
    seedBikriDraft({ lot: 'pukhta-seller' });
    const view = await openBook(language);
    fakePeople();
    // The one-time seller account has fields for his name and phone under the lot.
    const panel = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('[data-pukhta-for]');
      if (found === null) throw new Error('the one-time seller has no fields yet');
      return found;
    }, SLOW_MS);
    const first = panel.querySelector('input');
    if (first === null) throw new Error('no name field');
    await view.user.click(first);
    await view.user.keyboard('shah');
    // The seller side is asked: the people of the buyer side are not offered here.
    await waitFor(() => {
      expect(pukhtaApi.searchPukhtaPeople).toHaveBeenLastCalledWith('shah', 'seller');
    });
  });

  it('nothing is asked while the field is empty', async () => {
    await atTheName();
    expect(pukhtaApi.searchPukhtaPeople).not.toHaveBeenCalled();
  });
});

describe.each(LANGUAGES)('the پختہ کھاتہ register (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    vi.mocked(pukhtaApi.pukhtaRegister).mockResolvedValue(
      pukhtaRegisterOf([
        registerRow(),
        registerRow({
          bikri_id: 43,
          bikri_number: 3,
          name_ur: NASIR.name_ur,
          name_en: NASIR.name_en,
          phone: NASIR.phone,
          amount: money(2_000),
          paid: false,
          open: money(2_000),
        }),
        registerRow({
          bikri_id: 44,
          bikri_number: 4,
          cancelled: true,
          paid: false,
          open: money(0),
        }),
      ]),
    );
  });

  const open = async () => {
    const view = await renderApp('/bikri/pukhta');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'glossary-pukhta-khata')),
    });
    return view;
  };

  const rowOf = (index: number) => document.querySelector(`.ag-row[row-id="${String(index)}"]`);
  const cell = (index: number, colId: string) =>
    visible(rowOf(index)?.querySelector(`[col-id="${colId}"]`)?.textContent);

  it('opens on this month, with the other filters empty and month rows asked for', async () => {
    await open();
    await waitFor(() => {
      expect(pukhtaApi.pukhtaRegister).toHaveBeenCalledWith({
        from: '01-10-2026',
        to: '10-10-2026',
        side: null,
        name: null,
        phone: null,
        state: null,
        offset: 0,
        limit: PAGE_ROWS,
        show_finished: false,
        search: null,
        sort: null,
        group_by_month: true,
      });
    });
  });

  it('shows date, number, name, amount and paid or due with the amount still open', async () => {
    await open();
    await waitFor(() => {
      expect(cell(0, 'bikri')).toBe('2');
    });
    expect(cell(0, 'date')).toBe('10-10-2026');
    expect(cell(0, 'name')).toBe(language === 'ur' ? 'شاہد' : 'Shahid');
    expect(cell(0, 'amount')).toBe('12,800');
    expect(cell(0, 'state')).toBe(message(language, 'pukhta-state-paid'));
    expect(cell(1, 'name')).toBe(language === 'ur' ? 'ناصر' : 'Nasir');
    expect(cell(1, 'state')).toBe(message(language, 'pukhta-state-due', { amount: '2,000' }));
  });

  it('a cancelled line is marked in words', async () => {
    await open();
    await waitFor(() => {
      expect(cell(2, 'state')).toBe(message(language, 'pukhta-state-cancelled'));
    });
  });

  it('the name, the phone, the item or the number are one search box, sent as typed, never converted', async () => {
    await open();
    const user = userEvent.setup();
    const search = await screen.findByRole('textbox', {
      name: named(message(language, 'common-search-label')),
    });
    await user.click(search);
    await user.keyboard('shah');
    await waitFor(() => {
      expect(pukhtaApi.pukhtaRegister).toHaveBeenLastCalledWith(
        expect.objectContaining({ search: 'shah', name: null, phone: null }),
      );
    });
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
    await user.keyboard('0300');
    await waitFor(() => {
      expect(pukhtaApi.pukhtaRegister).toHaveBeenLastCalledWith(
        expect.objectContaining({ search: 'shah0300' }),
      );
    });
  });

  it('the period is a choice (remembered), and "due" is a filter behind "More filters"', async () => {
    await open();
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'common-period-label')),
      }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'common-period-all')) }),
    );
    await waitFor(() => {
      expect(pukhtaApi.pukhtaRegister).toHaveBeenLastCalledWith(
        expect.objectContaining({ from: '', to: '' }),
      );
    });
    // "Paid or due" is the rare filter: it is behind the button.
    expect(
      screen.queryByRole('combobox', { name: named(message(language, 'pukhta-col-state')) }),
    ).not.toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'common-filters-more')) }),
    );
    await user.click(
      await screen.findByRole('combobox', { name: named(message(language, 'pukhta-col-state')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'pukhta-filter-due')) }),
    );
    await waitFor(() => {
      expect(pukhtaApi.pukhtaRegister).toHaveBeenLastCalledWith(
        expect.objectContaining({ state: 'due' }),
      );
    });
  });

  it('the tick "show closed and cancelled" sends show_finished, off by default', async () => {
    await open();
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('checkbox', { name: named(message(language, 'common-show-closed')) }),
    );
    await waitFor(() => {
      expect(pukhtaApi.pukhtaRegister).toHaveBeenLastCalledWith(
        expect.objectContaining({ show_finished: true }),
      );
    });
  });

  it('the backend’s month rows come above their lines, with its count', async () => {
    vi.mocked(pukhtaApi.pukhtaRegister).mockResolvedValue({
      ...pukhtaRegisterOf([registerRow()]),
      months: [{ month: '2026-10', count: 31, figures: [] }],
    });
    await open();
    await waitFor(() => {
      expect(visible(document.querySelector('.month-heading')?.textContent)).toContain(
        message(language, 'common-month-count', { count: 31 }),
      );
    });
  });

  it('a row opens its بکری read-only', async () => {
    const { router } = await open();
    const user = userEvent.setup();
    await waitFor(() => {
      expect(cell(0, 'bikri')).toBe('2');
    });
    const target = rowOf(0)?.querySelector<HTMLElement>('[col-id="bikri"]');
    if (target === null || target === undefined) throw new Error('no cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 42 });
  });

  it('with no line it says why; with a filter and no match it offers to clear the filters', async () => {
    vi.mocked(pukhtaApi.pukhtaRegister).mockResolvedValue(pukhtaRegisterOf([]));
    await open();
    const user = userEvent.setup();
    // This month has no line: that is a filter, so it offers a way out.
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'pukhta-no-match'));
    });
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'pukhta-clear')) }),
    );
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'pukhta-empty'));
    });
  });

  it('an error is told in words and has a retry', async () => {
    vi.mocked(pukhtaApi.pukhtaRegister).mockRejectedValueOnce(
      fail({ code: 'ERR_DATE_INVALID', field: 'from', values: {} }),
    );
    await open();
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'pukhta-load-failed'));
    });
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'common-retry')) }),
    );
    await waitFor(() => {
      expect(cell(0, 'bikri')).toBe('2');
    });
  });

  it('the side menu item opens it', async () => {
    const { router } = await renderApp('/bikri/book');
    const user = userEvent.setup();
    const menu = await screen.findByRole('navigation', {
      name: named(message(language, 'layout-module-label')),
    });
    await user.click(
      within(menu).getByRole('button', { name: named(message(language, 'layout-module-pukhta')) }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/pukhta');
    });
  });

  it('150 lines come one page at a time: 25 at once, the count and the totals are the backend’s, Next page loads the next 25', async () => {
    const all = Array.from({ length: 150 }, (_, index) =>
      registerRow({ bikri_id: 500 + index, bikri_number: index + 1 }),
    );
    vi.mocked(pukhtaApi.pukhtaRegister).mockImplementation((input) =>
      Promise.resolve(
        pukhtaRegisterOf(
          all,
          input.offset,
          input.limit,
          // As the backend: the sums are given only when a side is chosen, over the whole filter
          // and not over the rows on screen.
          input.side === null
            ? {}
            : {
                qty: 900,
                amount: money(1_800_000),
                commission: money(216_000),
                open: money(65_000),
              },
        ),
      ),
    );
    await open();
    await waitFor(() => {
      expect(visible(pageText())).toContain(rangeText(language, 1, 25, 150));
    });
    // With no side chosen there are no sums, so there is no totals row at all.
    expect(document.querySelector('.ag-row-pinned')).toBeNull();
    // Choosing a side gives them, over the whole filter.
    const picker = userEvent.setup();
    await picker.click(
      await screen.findByRole('combobox', { name: named(message(language, 'pukhta-col-side')) }),
    );
    await picker.click(
      await screen.findByRole('option', { name: named(message(language, 'pukhta-side-buyer')) }),
    );
    await waitFor(() => {
      expect(visible(document.querySelector('.ag-row-pinned [col-id="amount"]')?.textContent)).toBe(
        '1,800,000',
      );
    });
    expect(visible(document.querySelector('.ag-row-pinned [col-id="state"]')?.textContent)).toBe(
      message(language, 'pukhta-state-due', { amount: '65,000' }),
    );
    expect(pukhtaApi.pukhtaRegister).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: 0, limit: PAGE_ROWS }),
    );
    await userEvent.setup().click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(visible(pageText())).toContain(rangeText(language, 26, 50, 150));
    });
    expect(pukhtaApi.pukhtaRegister).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: PAGE_ROWS, limit: PAGE_ROWS }),
    );
  });
});
