// The filters of the list under the voucher form (plan 38 R4, G3, G4, G6): it opens on today, the
// search box sends `search`, the tick for cancelled vouchers sends `show_finished` (off by default),
// a month period asks for month heading rows, a column title sends the backend a sort, and the
// account filter is behind "More filters", reached by the keyboard. Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { paisaForTest } from '../helpers/paisa';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
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
import {
  AKRAM,
  ARSHAD,
  KARIM,
  PUKHTA_BUYER,
  RAHIM,
  fakeVouchers,
  voucherRow,
  type FakeVouchers,
} from '../helpers/vouchers';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;

function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

let store: FakeVouchers;

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  const rows = [AKRAM, ARSHAD, PUKHTA_BUYER, RAHIM, KARIM];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({});
});

afterEach(() => {
  leaveAppWindow();
});

const lastCall = () => vi.mocked(vouchersApi.vouchersList).mock.calls.at(-1)?.[0];

describe.each(LANGUAGES)('the filters of the voucher list (%s)', (language) => {
  beforeEach(() => {
    window.localStorage.clear();
    resetClientState(language);
    signedIn(sessionUser({ language }));
    store = fakeVouchers();
    store.rows.push(voucherRow({ id: 501, number: 1 }), voucherRow({ id: 502, number: 2 }));
  });

  async function open() {
    const app = await renderApp('/bikri/voucher?type=receipt');
    await screen.findByRole('heading', { level: 1 });
    await waitFor(() => {
      expect(cell(501, 'number')).not.toBe('');
    });
    return { ...app, user: userEvent.setup() };
  }

  it('opens on today with the cancelled vouchers left out', async () => {
    await open();
    expect(lastCall()).toMatchObject({
      from: '10-10-2026',
      to: '10-10-2026',
      show_finished: false,
      search: null,
      sort: null,
      group_by_month: false,
    });
  });

  it('the search box sends the text, and the tick sends show_finished', async () => {
    const { user } = await open();
    await user.type(
      await screen.findByRole('textbox', { name: named(message(language, 'common-search-label')) }),
      'akram',
    );
    await waitFor(() => {
      expect(lastCall()).toMatchObject({ search: 'akram' });
    });
    await user.click(
      screen.getByRole('checkbox', { name: named(message(language, 'common-show-closed')) }),
    );
    await waitFor(() => {
      expect(lastCall()).toMatchObject({ show_finished: true, search: 'akram' });
    });
  });

  it('a month period asks for month heading rows and draws them with the backend’s subtotals', async () => {
    const { user } = await open();
    vi.mocked(vouchersApi.vouchersList).mockImplementation((input) => {
      const rows = store.rows.slice(0, 2);
      return Promise.resolve({
        rows,
        total_count: 2,
        totals: { cancelled_count: 0, total_paisa: paisaForTest(2_000_000) },
        months: input.group_by_month
          ? [
              {
                month: '2026-10',
                count: 2,
                figures: [{ key: 'amount', is_money: true, value: 2_000_000 }],
              },
            ]
          : [],
      });
    });
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'common-period-label')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'common-period-month')) }),
    );
    await waitFor(() => {
      expect(lastCall()).toMatchObject({ from: '01-10-2026', group_by_month: true });
    });
    await waitFor(() => {
      expect(document.querySelectorAll('.month-heading')).toHaveLength(1);
    });
    expect(visible(document.querySelector('.month-heading')?.textContent)).toContain(
      `${message(language, 'report-month-10')} 2026`,
    );
    expect(visible(document.querySelector('.month-heading')?.textContent)).toContain('20,000');
  });

  it('a click on the amount title asks the backend for that order', async () => {
    const { user } = await open();
    const title = document.querySelector<HTMLElement>(
      '.ag-header-cell[col-id="amount"] .ag-header-cell-text',
    );
    if (title === null) throw new Error('no title');
    await user.click(title);
    await waitFor(() => {
      // The app language goes with the sort: names then sort in the alphabet of the screen.
      expect(lastCall()?.sort).toEqual({ column: 'amount_paisa', direction: 'asc', language });
    });
  });

  it('the account filter is behind "More filters", opened with the keyboard', async () => {
    const { user } = await open();
    expect(
      screen.queryByRole('combobox', { name: named(message(language, 'vouchers-list-account')) }),
    ).not.toBeInTheDocument();
    screen.getByRole('button', { name: named(message(language, 'common-filters-more')) }).focus();
    await user.keyboard('{Enter}');
    expect(
      await screen.findByRole('combobox', {
        name: named(message(language, 'vouchers-list-account')),
      }),
    ).toBeInTheDocument();
  });
});
