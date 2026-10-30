// The opening balances grid by pages (plan 38 R1, R4, PRD FR-10.9): the accounts come 25 at a time
// with a search box and page buttons, what is typed stays when the page turns or the search
// changes, and only the rows that changed are sent, from every page. Urdu and English, against a
// fake backend. The screen works out no money.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES } from '../../src/i18n';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccounts, fakeOpenings } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** 60 buyers with no opening: three pages of 25, 25 and 10. */
const MANY = Array.from({ length: 60 }, (_, index) =>
  account(
    1000 + index,
    'buyer',
    `0020${String(index + 1).padStart(4, '0')}`,
    `خریدار ${String(index + 1)}`,
    `Buyer ${String(index + 1)}`,
    { opening_date: null },
  ),
);

const cellOf = (accountId: number, field: string) =>
  document.querySelector<HTMLElement>(
    `.ag-row[row-id="${String(accountId)}"] .ag-cell[col-id="${field}"]`,
  );
const textOf = (accountId: number, field: string) => visible(cellOf(accountId, field)?.textContent);

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the opening balances by pages (%s)', (language) => {
  beforeEach(() => {
    window.localStorage.clear();
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const store = fakeAccounts(MANY);
    fakeOpenings(store);
    const app = await renderApp('/index/openings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'openings-title') });
    await waitFor(() => {
      expect(cellOf(1000, 'code')).not.toBeNull();
    });
    return { ...app, user: userEvent.setup() };
  }

  it('asks for the first page only, never for every account', async () => {
    await open();
    expect(accountApi.accountsList).toHaveBeenCalledTimes(1);
    expect(vi.mocked(accountApi.accountsList).mock.calls[0]?.slice(1)).toEqual([0, PAGE_ROWS]);
    expect(document.body.textContent.replace(/[⁦-⁩]/g, '')).toContain(
      rangeText(language, 1, 25, 60),
    );
    expect(cellOf(1025, 'code')).toBeNull();
  });

  it('what is typed stays when the page turns, and only the changed rows are sent', async () => {
    const { user } = await open();
    const typed = cellOf(1002, 'opening_dr');
    if (typed === null) throw new Error('no cell');
    await user.click(typed);
    await user.keyboard('4000{Enter}');
    await user.click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(cellOf(1025, 'code')).not.toBeNull();
    });
    expect(cellOf(1002, 'code')).toBeNull();
    const other = cellOf(1030, 'opening_cr');
    if (other === null) throw new Error('no cell');
    await user.click(other);
    await user.keyboard('700{Enter}');
    // Back to the first page: the 4000 is still in its cell.
    await user.click(await pagerButton(language, 'previous'));
    await waitFor(() => {
      expect(textOf(1002, 'opening_dr')).toBe('4,000');
    });
    await waitFor(() => {
      expect(accountApi.accountsOpeningsPreview).toHaveBeenLastCalledWith({
        lines: [
          expect.objectContaining({ account_id: 1002, opening_dr: '4000', opening_cr: '' }),
          expect.objectContaining({ account_id: 1030, opening_dr: '', opening_cr: '700' }),
        ],
      });
    });
    // F10 sends the two changed rows from the two pages, and nothing else.
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsOpeningsSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(accountApi.accountsOpeningsSave).mock.calls[0]?.[0].lines;
    expect(sent?.map((line) => line.account_id)).toEqual([1002, 1030]);
  });

  it('the search box narrows the list from the backend and keeps what was typed', async () => {
    const { user } = await open();
    const typed = cellOf(1002, 'opening_dr');
    if (typed === null) throw new Error('no cell');
    await user.click(typed);
    await user.keyboard('4000{Enter}');
    await user.type(
      screen.getByRole('textbox', { name: message(language, 'common-search-label') }),
      'buyer 5',
    );
    await waitFor(() => {
      expect(vi.mocked(accountApi.accountsList).mock.calls.at(-1)?.[0].text).toBe('buyer 5');
    });
    await waitFor(() => {
      expect(cellOf(1004, 'code')).not.toBeNull();
    });
    expect(cellOf(1002, 'code')).toBeNull();
    // The line typed before the search is still going to be saved.
    await waitFor(() => {
      expect(
        vi
          .mocked(accountApi.accountsOpeningsPreview)
          .mock.calls.at(-1)?.[0]
          .lines.map((l) => l.account_id),
      ).toEqual([1002]);
    });
  });
});
