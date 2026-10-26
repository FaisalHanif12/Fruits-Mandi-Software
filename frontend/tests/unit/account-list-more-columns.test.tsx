// The "More columns" tick of the account list (PRD FR-10.5, Code Standards F22; D1+D2 final
// recheck, N3). Below the wide step the opening debit and credit are hidden; the tick adds them,
// is reached and changed by keyboard, is remembered for next time, and the list stays inside the
// room it has (the widths are added up in list-widths.test.ts), in the Urdu and the English app.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, fakeAccounts, money, ref } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const STORAGE_KEY = 'mandi.filters.accounts.more_columns';

const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  city: ref(1, '0001', 'کوٹ رادھا کشن', 'Kot Radha Kishan'),
  balance: balance(2880, 'dr'),
});
const RAHIM = account(31, 'seller', '0010', 'رحیم', 'Rahim', {
  opening_dr_paisa: money(15000),
  opening_is_advance: true,
});

// The pinned totals row is not an account, so it is left out.
const rowIds = () =>
  Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map((row) =>
    row.getAttribute('row-id'),
  );
const hasColumn = (field: string) =>
  document.querySelector(`.ag-header-cell[col-id="${field}"]`) !== null;
const cell = (rowId: number, field: string) =>
  visible(
    document
      .querySelector(`.ag-row[row-id="${String(rowId)}"]`)
      ?.querySelector(`[col-id="${field}"]`)?.textContent,
  );
const moreTick = (language: Language) =>
  screen.queryByRole('checkbox', { name: message(language, 'accounts-more-columns') });
/** The tick, or a failed test when it is not there. */
const tickOf = (language: Language) =>
  screen.getByRole('checkbox', { name: message(language, 'accounts-more-columns') });

/** The width of the window as the three steps see it: media queries that match up to a step. */
function stubStep(step: 'compact' | 'normal' | 'wide') {
  const matchesFor = (query: string) => {
    if (query.includes('1920')) return step === 'wide';
    if (query.includes('1366')) return step !== 'compact';
    return false;
  };
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: matchesFor(query),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  }));
  document.documentElement.style.setProperty('--breakpoint-normal', '1366px');
  document.documentElement.style.setProperty('--breakpoint-wide', '1920px');
}

beforeEach(() => {
  vi.clearAllMocks();
  window.localStorage.removeItem(STORAGE_KEY);
  healthyBackend();
  fakeMasters();
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.documentElement.style.removeProperty('--breakpoint-normal');
  document.documentElement.style.removeProperty('--breakpoint-wide');
  window.localStorage.removeItem(STORAGE_KEY);
  leaveAppWindow();
});

describe.each(LANGUAGES)('the account list, More columns (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeAccounts([AKRAM, RAHIM]);
  });

  it('at the normal step the tick adds the opening debit and credit, with the totals under them', async () => {
    stubStep('normal');
    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31']);
    });
    // Not shown until asked for, and the totals are the line under the list.
    expect(hasColumn('opening_dr')).toBe(false);
    expect(hasColumn('opening_cr')).toBe(false);
    expect(hasColumn('city')).toBe(true);
    expect(visible(document.body.textContent)).toContain(message(language, 'accounts-total-dr'));

    const tick = tickOf(language);
    expect(tick).not.toBeChecked();
    await userEvent.setup().click(tick);

    await waitFor(() => {
      expect(hasColumn('opening_dr')).toBe(true);
    });
    expect(hasColumn('opening_cr')).toBe(true);
    expect(cell(31, 'opening_dr')).toBe('15,000');
    // The room for the two columns comes from city and credit allowed; the date stays with them.
    expect(hasColumn('city')).toBe(false);
    expect(hasColumn('credit_allowed')).toBe(false);
    expect(hasColumn('opening_date')).toBe(true);
    expect(hasColumn('balance')).toBe(true);
    // The totals now stand under the columns, in a pinned row, with no second line.
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="opening_dr"]')?.textContent)).toBe('65,000');
    expect(visible(document.body.textContent)).not.toContain(
      message(language, 'accounts-total-dr'),
    );
  });

  it('is reached with Tab and changed with Space, and unticking takes the columns away again', async () => {
    stubStep('normal');
    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31']);
    });
    const user = userEvent.setup();
    const tick = tickOf(language);
    // A native tick box is in the Tab order and answers Space.
    expect(tick.tabIndex).toBeGreaterThanOrEqual(0);
    tick.focus();
    await user.keyboard(' ');
    await waitFor(() => {
      expect(hasColumn('opening_dr')).toBe(true);
    });
    expect(tick).toHaveFocus();
    await user.keyboard(' ');
    await waitFor(() => {
      expect(hasColumn('opening_dr')).toBe(false);
    });
    expect(hasColumn('city')).toBe(true);
  });

  it('is remembered for the next time the list is opened', async () => {
    stubStep('normal');
    const first = await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31']);
    });
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('opening_dr')).toBe(true);
    });
    expect(window.localStorage.getItem(STORAGE_KEY)).toBe('1');
    first.unmount();

    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(hasColumn('opening_dr')).toBe(true);
    });
    expect(moreTick(language)).toBeChecked();
  });

  it('at the compact step the tick adds them too, and the narrow columns stay out', async () => {
    stubStep('compact');
    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31']);
    });
    expect(hasColumn('opening_dr')).toBe(false);
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('opening_dr')).toBe(true);
    });
    expect(hasColumn('opening_cr')).toBe(true);
    expect(hasColumn('opening_date')).toBe(false);
    expect(hasColumn('city')).toBe(false);
  });

  it('at the wide step every column is already there, so there is no tick', async () => {
    stubStep('wide');
    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31']);
    });
    expect(hasColumn('opening_dr')).toBe(true);
    expect(hasColumn('city')).toBe(true);
    expect(moreTick(language)).toBeNull();
  });

  it('keeps the tick and its columns through a language switch, with the same rows', async () => {
    stubStep('normal');
    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31']);
    });
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('opening_dr')).toBe(true);
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    // The grid is made again for the other direction; nothing the user chose is lost.
    await waitFor(() => {
      expect(moreTick(other)).toBeChecked();
    });
    await waitFor(() => {
      expect(hasColumn('opening_dr')).toBe(true);
    });
    expect(rowIds()).toEqual(['21', '31']);
  });
});
