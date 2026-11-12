// An account on a closed day (plan 44, PRD FR-10.11), in the Urdu and the English app: the account
// itself saves on any day, so the wire carries no opening date without opening money; an opening
// balance needs an open date, and when the date is closed the form offers a way out: today's date
// when today is open, or "save without the opening balance" / "reopen the day" when today is closed.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { accountInputOf } from '../../src/shared/accounts/account-input';
import * as editApi from '../../src/shared/edit/api';
import * as accountApi from '../../src/shared/accounts/api';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { EMPTY_PAIRS } from '../helpers/paged';
import { account, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { AUDIT_EDIT_STAMP_BIKRI, DAY_FIX_OPEN_E2E_45 } from '../helpers/edit-real';
import { fakeMasters } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

const named = (text: string) => (name: string) => visible(name) === text;
const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {});

function closedDate(todayOpen: 0 | 1) {
  return new CommandError(
    refusal([
      {
        code: 'ERR_DATE_CLOSED',
        field: 'opening_date',
        values: { closed_up_to: '30-10-2026', today: '31-10-2026', today_open: todayOpen },
      },
    ]),
  );
}

beforeEach(() => {
  vi.clearAllMocks();
});
afterEach(() => {
  leaveAppWindow();
});

describe('what goes to the backend', () => {
  const base = {
    id: null,
    version: null,
    account_type: 'buyer' as const,
    control_id: null,
    code: '0030',
    names: { urdu: 'طارق', english: 'Tariq' },
    tag: '',
    father: { urdu: '', english: '' },
    city: null,
    adda: null,
    phone1: '',
    phone2: '',
    address: '',
    linked_account: null,
    opening_date: '31-10-2026',
    opening_dr: '',
    opening_cr: '',
    opening_is_advance: false,
    credit_allowed: true,
    credit_limit: '',
    default_marka: null,
    collector: null,
    is_active: true,
    note: '',
  };

  it('no opening money: no opening date, so the day is not looked at', () => {
    expect(accountInputOf(base, []).opening_date).toBe('');
  });

  it('opening money keeps its date', () => {
    expect(accountInputOf({ ...base, opening_dr: '1,000' }, []).opening_date).toBe('31-10-2026');
    expect(accountInputOf({ ...base, opening_cr: '500' }, []).opening_date).toBe('31-10-2026');
  });
});

describe.each(LANGUAGES)('the opening date of a closed day (%s)', (language) => {
  beforeEach(() => {
    healthyBackend();
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeMasters();
    fakeAccounts([AKRAM]);
    vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
    vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
    vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
    vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
    vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
  });

  const field = (id: string) =>
    screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

  async function typeOpening() {
    await renderApp('/index/accounts/21');
    const user = userEvent.setup();
    await user.type(await field('accounts-field-opening-dr'), '1000');
    return user;
  }

  it('today is open: the form offers today’s date, and the next save carries it', async () => {
    vi.mocked(accountApi.accountsSave).mockRejectedValueOnce(closedDate(1));
    const user = await typeOpening();
    press('F10');
    await user.click(
      await screen.findByRole('button', {
        name: named(
          message(language, 'accounts-opening-closed-use-today', { today: '31-10-2026' }),
        ),
      }),
    );
    await waitFor(async () => {
      expect((await field('accounts-field-opening-date')).value).toBe('31-10-2026');
    });
    vi.mocked(accountApi.accountsSave).mockImplementation(() => Promise.resolve(AKRAM));
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(2);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[1]?.[0]).toMatchObject({
      opening_date: '31-10-2026',
      opening_dr: '1000',
    });
  });

  it('the offer scrolls itself into view when the save is refused, so the reason is never out of sight', async () => {
    const scrolled = vi.fn();
    Object.defineProperty(Element.prototype, 'scrollIntoView', {
      configurable: true,
      value: scrolled,
    });
    try {
      vi.mocked(accountApi.accountsSave).mockRejectedValueOnce(closedDate(1));
      await typeOpening();
      expect(scrolled).not.toHaveBeenCalled();
      press('F10');
      const button = await screen.findByRole('button', {
        name: named(
          message(language, 'accounts-opening-closed-use-today', { today: '31-10-2026' }),
        ),
      });
      await waitFor(() => {
        expect(scrolled).toHaveBeenCalledWith({ block: 'nearest' });
      });
      // The element that scrolled holds the offer's own words and buttons.
      const target = scrolled.mock.contexts.at(-1);
      expect(target).toBeInstanceOf(HTMLElement);
      expect((target as HTMLElement).contains(button)).toBe(true);
    } finally {
      Reflect.deleteProperty(Element.prototype, 'scrollIntoView');
    }
  });

  it('today is closed: the day is said to be closed, there is no "date after" line and no way but reopening', async () => {
    vi.mocked(accountApi.accountsSave).mockRejectedValueOnce(closedDate(0));
    await typeOpening();
    press('F10');
    expect(
      visible(
        (await screen.findByText(message(language, 'accounts-opening-closed-today-closed')))
          .textContent,
      ),
    ).toContain(message(language, 'accounts-opening-closed-today-closed'));
    // There is no "use today" here: today cannot take it.
    expect(
      screen.queryByRole('button', {
        name: (name) =>
          visible(name).startsWith(
            message(language, 'accounts-opening-closed-use-today', { today: '' }).slice(0, 4),
          ),
      }),
    ).toBeNull();
    // Neither "enter a date after X" (X is today here, so no later date is allowed) nor "save without".
    const shown = visible(document.body.textContent);
    expect(shown).not.toContain(
      message(language, 'err-date-closed', { closed_up_to: '30-10-2026' }),
    );
    expect(shown).toContain(message(language, 'edit-action-fix-day'));
    expect(
      screen.getAllByRole('button').filter((button) => {
        const label = visible(button.textContent);
        return (
          label === message(language, 'edit-action-fix-day') ||
          label.startsWith(
            message(language, 'accounts-opening-closed-use-today', { today: '' }).slice(0, 4),
          )
        );
      }),
    ).toHaveLength(1);
  });

  it('today is open: the plain "date after X" line is not shown either; today and the reopening are both offered', async () => {
    vi.mocked(accountApi.accountsSave).mockRejectedValueOnce(closedDate(1));
    await typeOpening();
    press('F10');
    expect(
      await screen.findByText(message(language, 'accounts-opening-closed-today-open')),
    ).toBeInTheDocument();
    // Today's date helps when the new date is the closed one; the reopening helps when the OLD opening is.
    expect(
      screen.getByRole('button', {
        name: named(
          message(language, 'accounts-opening-closed-use-today', { today: '31-10-2026' }),
        ),
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: named(message(language, 'edit-action-fix-day')) }),
    ).toBeInTheDocument();
    expect(visible(document.body.textContent)).not.toContain(
      message(language, 'err-date-closed', { closed_up_to: '30-10-2026' }),
    );
  });

  it('today is closed: "reopen the day and fix" asks why and reopens', async () => {
    vi.mocked(accountApi.accountsSave).mockRejectedValueOnce(closedDate(0));
    vi.mocked(editApi.dayFixOpen).mockResolvedValue(DAY_FIX_OPEN_E2E_45);
    const user = await typeOpening();
    press('F10');
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'edit-action-fix-day')),
      }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', { name: named(message(language, 'edit-fix-label')) }),
      'opening balance{Enter}',
    );
    await waitFor(() => {
      expect(editApi.dayFixOpen).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(editApi.dayFixOpen).mock.calls[0]?.[1]).toBe('opening balance');
    // The opening date is typed as DD-MM-YYYY, which is what `day_fix_open` reads (Umar, 13-11-2026).
    expect(vi.mocked(editApi.dayFixOpen).mock.calls[0]?.[0]).toMatch(/^\d{2}-\d{2}-\d{4}$/);
  });

  it('an account that was changed shows when and by whom (PRD FR-7.71)', async () => {
    const stamp = AUDIT_EDIT_STAMP_BIKRI[0]?.stamp ?? null;
    vi.mocked(editApi.auditEditStamp).mockResolvedValue([{ id: 21, stamp }]);
    await renderApp('/index/accounts/21');
    await screen.findByRole('heading', { level: 1 });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain('Maher Adnan');
    });
    expect(editApi.auditEditStamp).toHaveBeenCalledWith('account', [21]);
  });
});
