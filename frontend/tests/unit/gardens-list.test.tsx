// باغات, the list of gardens (plan 17c, PRD 12.11, FR-5.2, FR-19.11), in the Urdu and the English
// app, against a fake backend: the main bar item opens a real screen before مارکیٹ سفر, the list
// and its totals (the backend's), a closed garden that stays in the list marked with its day, the
// four states, page-by-page loading, the search, and Enter opening a garden. Every figure on the
// screen is a figure the backend sent.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { e40Closed, e40Garden, fakeGardens, gardenRow } from '../helpers/gardens';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';
import * as urduApi from '../../src/shared/urdu/api';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
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
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

describe.each(LANGUAGES)('باغات, the list (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const open = async () => {
    const view = await renderApp('/gardens');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'gardens-title')),
    });
    return { ...view, user: userEvent.setup() };
  };

  it('the main bar has the item, before مارکیٹ سفر, and it opens this screen, not a coming soon page', async () => {
    fakeGardens();
    const { router, user } = await open();
    expect(router.state.location.pathname).toBe('/gardens');
    const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
    const item = within(nav).getByRole('button', { name: message(language, 'layout-nav-gardens') });
    expect(item).toHaveAttribute('aria-current', 'page');
    const labels = within(nav)
      .getAllByRole('button')
      .map((button) => button.getAttribute('aria-label'));
    expect(labels.indexOf(message(language, 'layout-nav-gardens'))).toBe(
      labels.indexOf(message(language, 'layout-nav-rokar')) + 1,
    );
    expect(labels.indexOf(message(language, 'layout-nav-trips'))).toBe(
      labels.indexOf(message(language, 'layout-nav-gardens')) + 1,
    );
    await user.click(
      within(nav).getByRole('button', { name: message(language, 'layout-nav-trips') }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/trips');
    });
  });

  it('lists the gardens and the totals the backend sent, asking for the first page of 100', async () => {
    fakeGardens();
    await open();
    await waitFor(() => {
      expect(cell(1, 'number')).toBe('0080');
    });
    expect(cell(1, 'name')).toBe(language === 'ur' ? 'شاہ پور باغ' : 'Shahpur Garden');
    expect(cell(1, 'status')).toBe(message(language, 'gardens-status-open'));
    expect(cell(1, 'balance')).toBe('11,400');
    const pinned = document.querySelector('.ag-row-pinned');
    expect(pinned).not.toBeNull();
    expect(visible(pinned?.querySelector('[col-id="name"]')?.textContent)).toBe(
      message(language, 'glossary-meezan'),
    );
    expect(visible(pinned?.querySelector('[col-id="balance"]')?.textContent)).toBe('11,400');
    expect(gardensApi.gardensList).toHaveBeenCalledWith({
      from: null,
      to: null,
      status: null,
      partner_id: null,
      search: null,
      offset: 0,
      limit: PAGE_ROWS,
      sort: null,
      // The list is in date order, so the month rows are asked for.
      group_by_month: true,
    });
    expect(visible(document.body.textContent)).toContain(rangeText(language, 1, 1, 1));
  });

  it('the search box sends the text, and the period opens on every garden and can be narrowed', async () => {
    fakeGardens();
    const { user } = await open();
    const period = await screen.findByRole('combobox', {
      name: named(message(language, 'common-period-label')),
    });
    expect(period).toHaveTextContent(message(language, 'common-period-all'));
    await user.type(
      await screen.findByRole('textbox', { name: named(message(language, 'gardens-list-search')) }),
      'shah',
    );
    await waitFor(() => {
      expect(gardensApi.gardensList).toHaveBeenLastCalledWith(
        expect.objectContaining({ search: 'shah', from: null, to: null }),
      );
    });
    await user.click(period);
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'common-period-year')) }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensList).toHaveBeenLastCalledWith(
        expect.objectContaining({ from: '01-01-2026', to: '10-10-2026' }),
      );
    });
  });

  it('a closed garden stays in the list, marked closed with the day it was closed', async () => {
    fakeGardens({
      gardens: [e40Garden(), e40Closed(), { ...e40Closed(), id: 2, number: 2 }],
    });
    await open();
    await waitFor(() => {
      expect(cell(2, 'number')).not.toBe('');
    });
    expect(cell(1, 'status')).toBe(message(language, 'gardens-status-open'));
    expect(cell(2, 'status')).toBe(
      message(language, 'gardens-status-closed-on', { date: '13-10-2026' }),
    );
  });

  it('an empty list says why and has a button that starts the first garden (FR-7.46)', async () => {
    fakeGardens({ gardens: [] });
    const { router, user } = await open();
    expect(
      await screen.findByText(said(message(language, 'gardens-list-empty'))),
    ).toBeInTheDocument();
    const buttons = screen
      .getAllByRole('button')
      .filter((button) => visible(button.textContent).includes(message(language, 'gardens-new')));
    expect(buttons.length).toBeGreaterThan(0);
    await user.click(buttons[buttons.length - 1] ?? document.body);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/gardens/new');
    });
  });

  it('an error says so in plain words and the retry button asks again', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensList).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
    );
    const { user } = await open();
    expect(
      await screen.findByText(said(message(language, 'gardens-list-failed'))),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    await waitFor(() => {
      expect(cell(1, 'number')).toBe('0080');
    });
  });

  it('loads page by page: the next 25 rows come on the page button, the count says where it is', async () => {
    const many = Array.from({ length: 60 }, (_, index) =>
      gardenRow({
        id: index + 1,
        number: index + 1,
        name_en: `Garden ${String(index + 1)}`,
        name_ur: `باغ ${String(index + 1)}`,
      }),
    );
    fakeGardens({ gardens: many });
    const { user } = await open();
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 1, 25, 60));
    });
    await user.click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 26, 50, 60));
    });
    expect(gardensApi.gardensList).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: PAGE_ROWS, limit: PAGE_ROWS }),
    );
    await user.click(await pagerButton(language, 'last'));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 51, 60, 60));
    });
    expect(await pagerButton(language, 'next')).toBeDisabled();
  });

  it('the search is sent to the backend, and a search with no match offers to clear it', async () => {
    fakeGardens();
    const { user } = await open();
    await waitFor(() => {
      expect(cell(1, 'number')).not.toBe('');
    });
    await user.type(
      screen.getByRole('textbox', { name: named(message(language, 'gardens-list-search')) }),
      'zzz',
    );
    expect(
      await screen.findByText(said(message(language, 'gardens-list-no-match'))),
    ).toBeInTheDocument();
    expect(gardensApi.gardensList).toHaveBeenLastCalledWith(
      expect.objectContaining({ search: 'zzz' }),
    );
    await user.click(screen.getByRole('button', { name: message(language, 'gardens-list-clear') }));
    await waitFor(() => {
      expect(cell(1, 'number')).toBe('0080');
    });
  });

  it('Enter on a garden opens it', async () => {
    fakeGardens();
    const { router, user } = await open();
    await waitFor(() => {
      expect(cell(1, 'number')).not.toBe('');
    });
    const target = document.querySelector<HTMLElement>('.ag-row[row-id="1"] [col-id="number"]');
    if (target === null) throw new Error('no cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/gardens/1');
    });
  });

  it('F2 starts a new garden from the keyboard', async () => {
    fakeGardens();
    const { router } = await open();
    await waitFor(() => {
      expect(cell(1, 'number')).not.toBe('');
    });
    document.body.dispatchEvent(
      new KeyboardEvent('keydown', { code: 'F2', key: 'F2', bubbles: true }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/gardens/new');
    });
  });
});
