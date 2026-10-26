// The opening balances grid (plan 07, PRD FR-10.9): one row per account, the cells typed by
// keyboard, only the changed rows sent as the text typed, the backend's totals and ابتدائی فرق
// shown, a refused line pointing at its cell, and nothing lost on a language switch, in the Urdu and
// in the English app, against a fake backend. The screen works out no money.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccounts, fakeOpenings, money } from '../helpers/accounts';
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
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const CASH = account(51, 'other', '01010001', 'روکڑ', 'Cash in Hand', {
  opening_date: '2026-08-31',
  opening_dr_paisa: money(65000),
});
const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  opening_date: null,
});
const ARSHAD = account(22, 'buyer', '0021', 'ارشد ملتانی', 'Arshad Multani', {
  opening_date: null,
});
const RAHIM = account(31, 'seller', '0010', 'رحیم', 'Rahim', {
  opening_date: '2026-08-31',
  opening_dr_paisa: money(15000),
  opening_is_advance: true,
});
const ROWS = [CASH, AKRAM, ARSHAD, RAHIM];

const named = (text: string) => (name: string) => visible(name) === text;

/** The grid cell of an account's row: the rows are in the order of the list. */
function cell(accountId: number, field: string): HTMLElement {
  const index = ROWS.findIndex((row) => row.id === accountId);
  const found = document.querySelector<HTMLElement>(
    `.ag-row[row-index="${String(index)}"] .ag-cell[col-id="${field}"]`,
  );
  if (found === null) throw new Error(`no cell ${String(accountId)}:${field}`);
  return found;
}
const text = (accountId: number, field: string) => visible(cell(accountId, field).textContent);

const focused = () => {
  const cellElement = document.activeElement?.closest<HTMLElement>('.ag-cell');
  const rowElement = cellElement?.closest<HTMLElement>('.ag-row');
  if (cellElement == null || rowElement == null) return null;
  const index = Number(rowElement.getAttribute('row-index'));
  return `${String(ROWS[index]?.id)}:${cellElement.getAttribute('col-id') ?? '?'}`;
};

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

describe.each(LANGUAGES)('the opening balances grid (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open(rows = ROWS) {
    const store = fakeAccounts(rows);
    fakeOpenings(store);
    const app = await renderApp('/index/openings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'openings-title') });
    await waitFor(() => {
      expect(document.querySelectorAll('.ag-row[row-index]')).toHaveLength(rows.length);
    });
    return { ...app, store, user: userEvent.setup() };
  }

  it('shows one row per account with its code, name, date and opening, and the advance word', async () => {
    await open();
    expect(text(51, 'code')).toBe('01010001');
    expect(text(51, 'name')).toBe(language === 'ur' ? 'روکڑ' : 'Cash in Hand');
    expect(text(51, 'opening_date')).toBe('31-08-2026');
    expect(text(51, 'opening_dr')).toBe('65,000');
    expect(text(51, 'opening_cr')).toBe('');
    // An account with no opening starts empty, with the common date (today).
    expect(text(21, 'opening_date')).toBe('10-10-2026');
    expect(text(21, 'opening_dr')).toBe('');
    // A seller's opening that counts as an advance says so.
    expect(text(31, 'name')).toBe(
      message(language, 'openings-name-advance', { name: language === 'ur' ? 'رحیم' : 'Rahim' }),
    );
    expect(text(21, 'name')).toBe(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    // Nothing changed yet: nothing to save.
    expect(screen.getByText(message(language, 'openings-none-changed'))).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) }),
    ).toBeDisabled();
  });

  it('Enter goes date, debit, credit and then to the next row, and no key adds a row', async () => {
    const { user } = await open([AKRAM, ARSHAD]);
    const first = document.querySelector<HTMLElement>(
      '.ag-row[row-index="0"] .ag-cell[col-id="opening_date"]',
    );
    if (first === null) throw new Error('no first cell');
    await user.click(first);
    await user.keyboard('{Enter}');
    expect(document.activeElement?.closest('.ag-cell')?.getAttribute('col-id')).toBe('opening_dr');
    await user.keyboard('5000{Enter}');
    expect(document.activeElement?.closest('.ag-cell')?.getAttribute('col-id')).toBe('opening_cr');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      const active = document.activeElement?.closest<HTMLElement>('.ag-cell');
      expect(active?.getAttribute('col-id')).toBe('opening_date');
      expect(active?.closest('.ag-row')?.getAttribute('row-index')).toBe('1');
    });
    // On the last row's last cell Enter and ↓ stay put.
    await user.keyboard('{Enter}{Enter}{Enter}{ArrowDown}');
    expect(document.querySelectorAll('.ag-row[row-index]')).toHaveLength(2);
  });

  it('sends only the changed rows, as the text typed, and shows the totals and ابتدائی فرق of the backend', async () => {
    const { user } = await open();
    await user.click(cell(21, 'opening_dr'));
    await user.keyboard('5000{Enter}');
    await user.click(cell(22, 'opening_cr'));
    await user.keyboard('2,500{Enter}');
    await waitFor(() => {
      expect(accountApi.accountsOpeningsPreview).toHaveBeenLastCalledWith({
        lines: [
          {
            account_id: 21,
            version: 1,
            opening_date: '10-10-2026',
            opening_dr: '5000',
            opening_cr: '',
            opening_is_advance: false,
          },
          {
            account_id: 22,
            version: 1,
            opening_date: '10-10-2026',
            opening_dr: '',
            opening_cr: '2,500',
            opening_is_advance: false,
          },
        ],
      });
    });
    const summary = await screen.findByRole('heading', {
      name: named(message(language, 'openings-summary-title')),
    });
    const panel = summary.closest('section');
    await waitFor(() => {
      expect(visible(panel?.textContent)).toContain('5,000');
    });
    const words = visible(panel?.textContent);
    expect(words).toContain('2,500');
    expect(words).toContain(message(language, 'openings-changed', { count: '2' }));
    // Debit 65,000 + 15,000 + 5,000 and credit 2,500: the difference is the backend's, 82,500 credit.
    expect(words).toContain(message(language, 'openings-difference'));
    expect(words).toContain('82,500');
    expect(words).toContain(message(language, 'common-side-cr'));
  });

  it('F10 saves the changed rows once, says so, and shows what was saved', async () => {
    const { user, store } = await open();
    await user.click(cell(21, 'opening_dr'));
    await user.keyboard('5000{Enter}');
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) }),
      ).toBeEnabled();
    });
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsOpeningsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsOpeningsSave).mock.calls[0]?.[0].lines).toHaveLength(1);
    expect(
      (await screen.findAllByText(message(language, 'openings-saved'))).length,
    ).toBeGreaterThan(0);
    expect(store.rows.find((row) => row.id === 21)?.version).toBe(2);
    // The grid starts again from what was saved: nothing left to save.
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) }),
      ).toBeDisabled();
    });
    expect(text(21, 'opening_dr')).toBe('5,000');
  });

  it('a date changed for all follows only the accounts with no opening', async () => {
    const { user } = await open();
    const common = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'openings-date-all')),
    );
    expect(common.value).toBe('10-10-2026');
    await user.clear(common);
    await user.type(common, '31-08-2026');
    await waitFor(() => {
      expect(text(21, 'opening_date')).toBe('31-08-2026');
    });
    expect(text(22, 'opening_date')).toBe('31-08-2026');
    // An account that had an opening keeps its own date.
    expect(text(31, 'opening_date')).toBe('31-08-2026');
    expect(text(51, 'opening_date')).toBe('31-08-2026');
    // A date typed on one row is that row's own, and the next change of the common date skips it.
    await user.click(cell(22, 'opening_date'));
    await user.keyboard('15-09-2026{Enter}');
    await user.clear(common);
    await user.type(common, '30-09-2026');
    await waitFor(() => {
      expect(text(21, 'opening_date')).toBe('30-09-2026');
    });
    expect(text(22, 'opening_date')).toBe('15-09-2026');
  });

  it('an account that cannot take an opening has locked cells, no date, and never goes to the backend', async () => {
    const income = account(61, 'other', '04010001', 'کمیشن آمدن', 'Commission Income', {
      opening_date: null,
      can_take_opening: false,
    });
    ROWS.push(income);
    try {
      const { user } = await open([...ROWS]);
      expect(text(61, 'opening_date')).toBe('');
      expect(cell(61, 'opening_dr').getAttribute('aria-readonly') ?? '').not.toBe('false');
      // The help line says why a row is locked.
      expect(visible(document.body.textContent)).toContain(
        message(language, 'openings-locked-help'),
      );
      // Typing on a locked cell changes nothing.
      await user.click(cell(61, 'opening_dr'));
      await user.keyboard('5000');
      expect(text(61, 'opening_dr')).toBe('');
      // The date for all does not touch it, and nothing is a line to save.
      const common = await screen.findByLabelText<HTMLInputElement>(
        named(message(language, 'openings-date-all')),
      );
      await user.clear(common);
      await user.type(common, '31-08-2026');
      await waitFor(() => {
        expect(text(21, 'opening_date')).toBe('31-08-2026');
      });
      expect(text(61, 'opening_date')).toBe('');
      const asked = vi
        .mocked(accountApi.accountsOpeningsPreview)
        .mock.calls.flatMap((call) => call[0].lines.map((line) => line.account_id));
      expect(asked).not.toContain(61);
    } finally {
      ROWS.pop();
    }
  });

  it('a line the backend refuses is listed with its account and marked on its cell, and F10 shows the refusal of the save', async () => {
    const { user } = await open();
    await user.click(cell(21, 'opening_dr'));
    await user.keyboard('100{Enter}');
    await user.click(cell(21, 'opening_cr'));
    await user.keyboard('50{Enter}');
    const heading = await screen.findByRole('heading', {
      name: message(language, 'openings-errors-title'),
    });
    const list = heading.closest('section');
    const words = visible(list?.textContent);
    expect(words).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(words).toContain(message(language, 'err-opening-both-sides'));
    expect(words).toContain(message(language, 'accounts-col-opening-dr'));
    expect(cell(21, 'opening_dr')).toHaveClass('cell-error');
    expect(cell(21, 'opening_cr')).not.toHaveClass('cell-error');
    // Saving is refused by the backend the same way: the line goes to its cell, nothing is lost.
    vi.mocked(accountApi.accountsOpeningsSave).mockRejectedValueOnce(
      fail({ code: 'ERR_OPENING_BOTH_SIDES', field: 'lines[0].opening_dr' }),
    );
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsOpeningsSave).toHaveBeenCalledTimes(1);
    });
    expect(cell(21, 'opening_dr')).toHaveClass('cell-error');
    expect(text(21, 'opening_dr')).toBe('100');
    expect(text(21, 'opening_cr')).toBe('50');
  });

  it('a refusal that is not about one line is shown as a notice', async () => {
    const { user } = await open();
    await user.click(cell(21, 'opening_dr'));
    await user.keyboard('100{Enter}');
    vi.mocked(accountApi.accountsOpeningsSave).mockRejectedValueOnce(
      fail({ code: 'ERR_DATE_CLOSED', values: { closed_up_to: '09-10-2026' } }),
    );
    press('F10');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(
      message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }),
    );
  });

  it('switching the language in the middle of an entry keeps what was typed', async () => {
    const { user } = await open();
    await user.click(cell(21, 'opening_dr'));
    await user.keyboard('5000{Enter}');
    await user.click(cell(22, 'opening_cr'));
    await user.keyboard('7');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    await waitFor(() => {
      expect(document.querySelector(other === 'ur' ? '.ag-rtl' : '.ag-ltr')).not.toBeNull();
    });
    await waitFor(() => {
      expect(text(21, 'opening_dr')).toBe('5,000');
    });
    expect(text(21, 'name')).toBe(other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    // The typing in the open cell was kept, and the same cell has focus to carry on.
    await waitFor(() => {
      expect(text(22, 'opening_cr')).toBe('7');
    });
    await waitFor(() => {
      expect(focused()).toBe('22:opening_cr');
    });
    await user.keyboard('700{Enter}');
    await waitFor(() => {
      expect(text(22, 'opening_cr')).toBe('700');
    });
  });

  it('a failed load shows the message with a retry that works', async () => {
    fakeAccounts(ROWS);
    vi.mocked(accountApi.accountsList).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await renderApp('/index/openings');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(
      await screen.findByRole('heading', { level: 1, name: message(language, 'openings-title') }),
    ).toBeInTheDocument();
  });

  it('with no accounts it says why and offers to make one', async () => {
    fakeAccounts([]);
    await renderApp('/index/openings');
    expect(await screen.findByText(message(language, 'openings-empty'))).toBeInTheDocument();
    expect(
      within(document.body).getByRole('button', {
        name: new RegExp(message(language, 'accounts-new')),
      }),
    ).toBeInTheDocument();
  });

  it('the account list opens it with a button', async () => {
    fakeAccounts(ROWS);
    const { router } = await renderApp('/index/accounts');
    await userEvent
      .setup()
      .click(await screen.findByRole('button', { name: message(language, 'openings-open') }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/openings');
    });
  });
});
