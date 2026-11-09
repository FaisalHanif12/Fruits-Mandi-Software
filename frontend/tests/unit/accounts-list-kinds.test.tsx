// Plan 32, R6 and plan 64: Index → Accounts shows only sellers and buyers. The type filter is
// exactly All, Seller, Buyer; cash, payables, capital, چکتا کھاتہ and the other system accounts
// never show here (the chart of accounts still has them). The new-account form offers Seller and
// Buyer only. Both languages, against a fake backend.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { accountChoicesFrom } from '../../src/features/masters/remember';
import { LANGUAGES, type Language } from '../../src/i18n';
import { account, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { PAGE_ROWS } from '../helpers/pager';
import { message, visible } from '../helpers/render';
import { useMasterFocus } from '../../src/features/masters/store';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const RAHIM = account(31, 'seller', '0010', 'رحیم', 'Rahim');
const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const CHUKTA_KHATA = account(23, 'buyer', '0030', 'چکتا کھاتہ', 'Chukta Khata', {
  is_system: true,
  is_chukta_khata: true,
});
const NAQDI = account(24, 'buyer', '03040000', 'نقدی منڈی', 'Naqdi Mandi', { is_system: true });
const CASH = account(51, 'other', '01010001', 'روکڑ', 'Cash in Hand', { is_system: true });
const FREIGHT = account(52, 'other', '02010002', 'کرایہ کھاتہ', 'Freight Payable', {
  is_system: true,
});
const LENT = account(53, 'money_lent', '01030001', 'اسلم', 'Aslam');
const ALL_ROWS = [RAHIM, AKRAM, CHUKTA_KHATA, NAQDI, CASH, FREIGHT, LENT];

const named = (text: string) => (name: string) => visible(name) === text;
const rowIds = () =>
  Array.from(document.querySelectorAll('.ag-row[row-id]')).map((row) => row.getAttribute('row-id'));

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the Accounts list (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    act(() => {
      useMasterFocus.setState({ rows: {} });
    });
  });

  async function openFilter() {
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'accounts-filter-type')),
      }),
    );
    return user;
  }

  it('shows only sellers and buyers: no cash, payable, نقدی منڈی, چکتا کھاتہ or lent person', async () => {
    fakeAccounts(ALL_ROWS);
    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['31', '21']);
    });
    expect(accountApi.accountsList).toHaveBeenLastCalledWith(
      { kind: 'all', text: '', include_inactive: false, has_opening: null },
      0,
      PAGE_ROWS,
    );
  });

  it('the type filter has exactly All, Seller, Buyer', async () => {
    fakeAccounts(ALL_ROWS);
    await renderApp('/index/accounts');
    await openFilter();
    const shown = (await screen.findAllByRole('option')).map((option) =>
      visible(option.textContent),
    );
    expect(shown).toEqual([
      message(language, 'accounts-type-all'),
      message(language, 'accounts-type-seller'),
      message(language, 'accounts-type-buyer'),
    ]);
  });

  it('Seller lists the sellers only', async () => {
    fakeAccounts(ALL_ROWS);
    await renderApp('/index/accounts');
    const user = await openFilter();
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'accounts-type-seller')) }),
    );
    await waitFor(() => {
      expect(rowIds()).toEqual(['31']);
    });
  });

  it('a kind kept from an earlier version (money lent, other) is not read: the list starts at All', () => {
    expect(accountChoicesFrom({ type: 'money_lent' }).type).toBe('all');
    expect(accountChoicesFrom({ type: 'other' }).type).toBe('all');
    expect(accountChoicesFrom({ type: 'buyer' }).type).toBe('buyer');
  });

  it('the new-account form offers Seller and Buyer only', async () => {
    fakeAccounts([RAHIM, AKRAM]);
    fakeMasters();
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'accounts-field-type')),
      }),
    );
    const shown = (await screen.findAllByRole('option')).map((option) =>
      visible(option.textContent),
    );
    expect(shown).toEqual([
      message(language, 'accounts-type-seller'),
      message(language, 'accounts-type-buyer'),
    ]);
  });

  it('an account hidden from the list (cash, a person who borrows) still opens in the account form, and its type cannot be changed', async () => {
    fakeAccounts(ALL_ROWS);
    fakeMasters();
    await renderApp('/index/accounts/53');
    const lentType = await screen.findByRole('combobox', {
      name: named(message(language, 'accounts-field-type')),
    });
    expect(lentType).toBeDisabled();
    expect(lentType).toHaveTextContent(message(language, 'accounts-type-money_lent'));
  });

  it('cash opens in the account form from its address, with its own type', async () => {
    fakeAccounts(ALL_ROWS);
    fakeMasters();
    await renderApp('/index/accounts/51');
    const cashType = await screen.findByRole('combobox', {
      name: named(message(language, 'accounts-field-type')),
    });
    expect(cashType).toBeDisabled();
    expect(cashType).toHaveTextContent(message(language, 'accounts-type-other'));
  });
});
