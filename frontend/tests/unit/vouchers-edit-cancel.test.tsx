// Edit and cancel of a saved voucher (plan 15, PRD FR-15.3, FR-7.8, FR-6.7, FR-6.6): the list under
// the form, Enter on a row to change it, the preview taking the old posting back first, the save of
// the change, the cancel box with its reason, the cash line of a cancel, a cancelled voucher marked
// in the list and opened read only, and a voucher of a closed day that cannot be changed. In the
// Urdu and in the English app, against a fake backend.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { paisaForTest } from '../helpers/paisa';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
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
import { fakeMasters } from '../helpers/masters';
import { PAGE_ROWS } from '../helpers/pager';
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
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));
const pageText = () => visible(document.body.textContent);

function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

/** Puts focus on a row of the list, the way a click on its number does. */
async function focusRow(user: ReturnType<typeof userEvent.setup>, rowId: number) {
  await waitFor(() => {
    expect(document.querySelector(`.ag-row[row-id="${String(rowId)}"]`)).not.toBeNull();
  });
  const target = document.querySelector<HTMLElement>(
    `.ag-row[row-id="${String(rowId)}"] [col-id="number"]`,
  );
  if (target === null) throw new Error('no cell');
  await user.click(target);
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

describe.each(LANGUAGES)('editing and cancelling a voucher (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    store = fakeVouchers();
    store.rows.push(voucherRow({ id: 501, number: 1 }), voucherRow({ id: 502, number: 2 }));
  });

  async function open(path = '/bikri/voucher?type=receipt') {
    const app = await renderApp(path);
    await screen.findByRole('heading', { level: 1 });
    return { ...app, user: userEvent.setup() };
  }

  it("lists the day's vouchers of the type, each with its number, account, amount and state", async () => {
    await open();
    await waitFor(() => {
      expect(cell(501, 'number')).toBe(message(language, 'vouchers-number-receipt', { number: 1 }));
    });
    expect(cell(501, 'account')).toBe(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(cell(501, 'amount')).toBe('10,000');
    expect(cell(501, 'status')).toBe(message(language, 'vouchers-state-saved'));
    expect(vouchersApi.vouchersList).toHaveBeenCalledWith({
      voucher_type: 'receipt',
      from: '10-10-2026',
      to: '10-10-2026',
      account_id: null,
      person_key: null,
      offset: 0,
      limit: PAGE_ROWS,
      // Cancelled vouchers step aside until the tick is on; today needs no month heading (plan 38).
      show_finished: false,
      search: null,
      sort: null,
      group_by_month: false,
    });
  });

  it("shows the backend's total under the amount, and leaves a cancelled voucher out of it", async () => {
    store.rows.push(voucherRow({ id: 503, number: 3, status: 'cancelled', cancel_reason: 'x' }));
    await open();
    await waitFor(() => {
      expect(cell(503, 'status')).toBe(message(language, 'vouchers-state-cancelled'));
    });
    // Three rows of 10,000, one cancelled: the total of the two saved ones, as the backend sums it.
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="amount"]')?.textContent)).toBe('20,000');
    expect(visible(pinned?.querySelector('[col-id="account"]')?.textContent)).toBe(
      message(language, 'glossary-meezan'),
    );
  });

  it('shows only the word under the list when the backend sends no total (a list of all types)', async () => {
    const real = store.rows.map((row) => ({ ...row }));
    vi.mocked(vouchersApi.vouchersList).mockResolvedValue({
      rows: real,
      total_count: real.length,
      totals: { cancelled_count: 0, total_paisa: null },
      months: [],
    });
    await open();
    await waitFor(() => {
      expect(cell(501, 'number')).toBe(message(language, 'vouchers-number-receipt', { number: 1 }));
    });
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="account"]')?.textContent)).toBe(
      message(language, 'glossary-meezan'),
    );
    expect(visible(pinned?.querySelector('[col-id="amount"]')?.textContent)).toBe('');
  });

  it('says so when there is nothing today, with what to do', async () => {
    store.rows.length = 0;
    await open();
    expect(
      await screen.findByText(message(language, 'vouchers-list-empty-receipt')),
    ).toBeInTheDocument();
  });

  it('a list that cannot be read says so with a retry', async () => {
    vi.mocked(vouchersApi.vouchersList).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    const { user } = await open();
    const failed = await screen.findByText(message(language, 'vouchers-list-failed'));
    const alert = failed.closest<HTMLElement>('[role="alert"]');
    if (alert === null) throw new Error('the failed list has no alert');
    await user.click(
      within(alert).getByRole('button', { name: message(language, 'common-retry') }),
    );
    await waitFor(() => {
      expect(cell(501, 'number')).not.toBe('');
    });
  });

  it('Enter on a row opens it in the form: its values, the old posting taken back in the preview', async () => {
    const { user, router } = await open();
    await focusRow(user, 501);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ type: 'receipt', id: 501 });
    });
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('10,000');
    expect(
      await screen.findByRole('combobox', {
        name: named(message(language, 'vouchers-field-buyer')),
      }),
    ).toHaveValue(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(pageText()).toContain(
      message(language, 'vouchers-editing-note', {
        kind: 'receipt',
        number: message(language, 'vouchers-number-receipt', { number: 1 }),
      }),
    );
    // The type cannot change while a saved voucher is changed.
    const group = screen.getByRole('group', { name: message(language, 'vouchers-field-type') });
    for (const button of within(group).getAllByRole('button')) expect(button).toBeDisabled();
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ type: 'receipt', account_id: 21, amount: '10,000' }),
        501,
      );
    });
  });

  it('changing the amount and pressing F10 sends the edit with the version, then a new form is ready', async () => {
    const { user, router } = await open('/bikri/voucher?type=receipt&id=501');
    const amount = await labelled(language, 'vouchers-field-amount');
    await user.click(amount);
    await user.keyboard('{Control>}a{/Control}9000');
    await user.click(await labelled(language, 'vouchers-field-edit-reason'));
    await user.keyboard('r');
    press('F10');
    await waitFor(() => {
      expect(store.edits).toHaveLength(1);
    });
    expect(store.edits[0]).toMatchObject({
      id: 501,
      version: 1,
      reason: 'r',
      confirm: false,
      entry: { type: 'receipt', account_id: 21, amount: '9000' },
    });
    const toast = message(language, 'vouchers-edited', {
      kind: 'receipt',
      number: message(language, 'vouchers-number-receipt', { number: 1 }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('id');
    });
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('');
  });

  it('cancel asks what will happen and for a reason; an empty reason is told at its field and nothing is sent', async () => {
    const { user } = await open();
    await focusRow(user, 501);
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'vouchers-cancel')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    const words = visible(dialog.textContent);
    expect(words).toContain(message(language, 'vouchers-cancel-title'));
    expect(words).toContain(message(language, 'vouchers-number-receipt', { number: 1 }));
    expect(words).toContain('10,000');
    expect(words).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    await user.click(
      within(dialog).getByRole('button', {
        name: named(message(language, 'vouchers-cancel-confirm')),
      }),
    );
    expect(within(dialog).getByText(message(language, 'err-required'))).toBeInTheDocument();
    expect(store.cancels).toHaveLength(0);
  });

  it('cancel with a reason reverses the voucher: sent with the version, marked cancelled in the list', async () => {
    const { user } = await open();
    await focusRow(user, 501);
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'vouchers-cancel')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.keyboard('wrong buyer');
    await user.click(
      within(dialog).getByRole('button', {
        name: named(message(language, 'vouchers-cancel-confirm')),
      }),
    );
    await waitFor(() => {
      expect(store.cancels).toEqual([
        { id: 501, version: 1, reason: 'wrong buyer', confirm: false },
      ]);
    });
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    await waitFor(() => {
      expect(cell(501, 'status')).toBe(message(language, 'vouchers-state-cancelled'));
    });
    // A cancelled voucher can neither be changed nor cancelled again.
    expect(
      screen.getByRole('button', { name: named(message(language, 'vouchers-edit')) }),
    ).toBeDisabled();
  });

  it('a cancel that takes cash below zero shows the cash line, and the next press confirms it', async () => {
    const { user } = await open();
    vi.mocked(vouchersApi.vouchersCancel).mockRejectedValueOnce(
      fail({ code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', values: { cash_after_paisa: -500000 } }),
    );
    await focusRow(user, 501);
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'vouchers-cancel')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.keyboard('x');
    await user.click(
      within(dialog).getByRole('button', {
        name: named(message(language, 'vouchers-cancel-confirm')),
      }),
    );
    const cashLine = message(language, 'err-cash-below-zero-not-confirmed', {
      cash_after: '-5,000',
    });
    await within(dialog).findByText((content) => visible(content) === cashLine);
    const yes = within(dialog).getByRole('button', {
      name: named(message(language, 'vouchers-cancel-confirm-cash')),
    });
    await user.click(yes);
    await waitFor(() => {
      expect(store.cancels).toEqual([{ id: 501, version: 1, reason: 'x', confirm: true }]);
    });
  });

  it('a cancelled voucher opens read only, in words, with its reason and no field to type in', async () => {
    store.rows.push(
      voucherRow({
        id: 503,
        number: 3,
        status: 'cancelled',
        editable: false,
        cancel_reason: 'wrong buyer',
      }),
    );
    await open('/bikri/voucher?type=receipt&id=503');
    await screen.findByText((content) =>
      visible(content).startsWith(message(language, 'vouchers-view-cancelled')),
    );
    expect(pageText()).toContain('wrong buyer');
    const screenArea = within(screen.getByRole('main'));
    expect(screenArea.queryByRole('textbox')).toBeNull();
    expect(screenArea.queryByRole('combobox')).toBeNull();
  });

  it('a voucher of a closed day cannot be changed and says what to do', async () => {
    store.rows.push(voucherRow({ id: 504, number: 4, editable: false }));
    await open('/bikri/voucher?type=receipt&id=504');
    expect(await screen.findByText(message(language, 'vouchers-view-closed'))).toBeInTheDocument();
    expect(within(screen.getByRole('main')).queryByRole('textbox')).toBeNull();
  });

  it('Ctrl+K finds a voucher by its number and opens it on the voucher screen with its type and id', async () => {
    vi.mocked(accountApi.searchGlobal).mockResolvedValue([
      {
        kind: 'voucher',
        id: 501,
        voucher_type: 'receipt',
        number: 1,
        entry_date: '2026-10-10',
        account: { id: 21, code: '0020', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' },
        amount_paisa: paisaForTest(1_000_000),
        status: 'saved',
      },
    ]);
    const { user, router } = await renderApp('/').then((app) => ({
      ...app,
      user: userEvent.setup(),
    }));
    await user.keyboard('{Control>}k{/Control}');
    await user.keyboard('1');
    await user.click(await screen.findByRole('option'));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/voucher');
    });
    expect(router.state.location.search).toMatchObject({ type: 'receipt', id: 501 });
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('10,000');
  });

  it('a voucher of a kind that belongs to another screen goes to its own screen in the cash book area', async () => {
    vi.mocked(accountApi.searchGlobal).mockResolvedValue([
      {
        kind: 'voucher',
        id: 900,
        voucher_type: 'personal_expense',
        number: 3,
        entry_date: '2026-10-10',
        account: null,
        amount_paisa: paisaForTest(200_000),
        status: 'saved',
      },
    ]);
    const { router } = await renderApp('/');
    const user = userEvent.setup();
    await user.keyboard('{Control>}k{/Control}');
    await user.keyboard('3');
    await user.click(await screen.findByRole('option'));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/expense');
    });
  });

  it('a voucher that is not there says so with a retry', async () => {
    await renderApp('/bikri/voucher?type=receipt&id=999');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-found'));
  });
});
