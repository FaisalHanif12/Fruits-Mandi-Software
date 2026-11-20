// Plan 88, PRD 15.4, FR-15.11 (v1.51), FR-7.58, FR-7.60, FR-7.69: the پیشگی tab of the voucher screen
// takes a buyer as well as a seller. The account box lists both, each marked in words; a seller
// goes on as before (his advances, the expected goods); a buyer is cash lent: a reason is asked and
// required, his open bills show under the form, and what is saved carries the reason and no
// expected goods. Both languages, by keyboard, against a fake backend. The screen works out no
// money: every figure shown is the fake backend's.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import * as windowsApi from '../../src/shared/windows/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
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
  CHUKTA_KHATA,
  KARIM,
  RAHIM,
  TRANSPORT,
  fakeVouchers,
  type FakeVouchers,
} from '../helpers/vouchers';
import { EMPTY_PAIRS } from '../helpers/paged';
import { refOf, voucherRow } from '../helpers/voucher-books';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/windows/api');

const named = (text: string) => (name: string) => visible(name) === text;

const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

const dropdown = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });

const pageText = () => visible(document.body.textContent);

/** The words of one cell of a list row. */
function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

/** The text of a panel, by the heading of its card. */
async function panelText(language: Language, titleId: string): Promise<string> {
  const title = await screen.findByRole('heading', { name: named(message(language, titleId)) });
  return visible(title.closest('section')?.textContent);
}

/** Types in the account box and takes the only match. */
async function pick(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  typed: string,
  shown: string,
) {
  await user.click(await dropdown(language, 'vouchers-field-party'));
  await user.keyboard(typed);
  // The recent list is open before the search answers: wait until the matches are the only rows.
  await waitFor(() => {
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(1);
    expect(visible(options[0]?.textContent)).toContain(shown);
  });
  await user.keyboard('{Enter}');
}

let store: FakeVouchers;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  const rows = [AKRAM, ARSHAD, CHUKTA_KHATA, RAHIM, KARIM, TRANSPORT];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({ staff: [] });
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  vi.mocked(windowsApi.windowsBounds).mockResolvedValue({
    first_date: '2026-09-01',
    last_date: '2026-10-10',
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the پیشگی tab takes a buyer too (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    store = fakeVouchers();
  });

  async function open() {
    await renderApp('/bikri/voucher?type=advance');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'vouchers-advance-title'),
    });
    return userEvent.setup();
  }

  it('the account box says who it is for and lists sellers and buyers, each marked, and nothing else', async () => {
    const user = await open();
    await user.click(await dropdown(language, 'vouchers-field-party'));
    // Rahim and Karim (sellers), Akram and Arshad (buyers): not the transporter, not چکتا کھاتہ.
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(4);
    });
    const rows = screen.getAllByRole('option').map((option) => visible(option.textContent));
    const sellerWord = message(language, 'accounts-type-seller');
    const buyerWord = message(language, 'accounts-type-buyer');
    const rahim = rows.find((row) => row.includes('0010'));
    const akram = rows.find((row) => row.includes('0020'));
    expect(rahim).toContain(sellerWord);
    expect(rahim).not.toContain(buyerWord);
    expect(akram).toContain(buyerWord);
    expect(akram).not.toContain(sellerWord);
    expect(rows.some((row) => row.includes('02010002'))).toBe(false);
    expect(rows.some((row) => row.includes('0030'))).toBe(false);
  });

  it('choosing a buyer shows his open bills and his card, asks for the reason and has no expected goods', async () => {
    const user = await open();
    // Before anyone is chosen the field for a seller's goods is there.
    expect(
      screen.queryByLabelText(named(message(language, 'vouchers-field-expected'))),
    ).not.toBeNull();
    await pick(user, language, '0020', '0020');
    const bills = await panelText(language, 'vouchers-bills-title');
    expect(bills).toContain('12,880');
    expect(await screen.findByText(message(language, 'assist-card-balance'))).toBeInTheDocument();
    // The reason of cash lent is asked; a seller's expected goods are not.
    expect(await labelled(language, 'vouchers-field-reason-lent')).toBeInTheDocument();
    expect(screen.queryByLabelText(named(message(language, 'vouchers-field-expected')))).toBeNull();
    // The seller's advances do not show for him.
    expect(
      screen.queryByRole('heading', { name: named(message(language, 'vouchers-advances-title')) }),
    ).toBeNull();
  });

  it('the reason is required: told at its field, and saving with it sends the buyer, the reason and no goods', async () => {
    const user = await open();
    await pick(user, language, '0020', '0020');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('3000');
    const reason = await labelled(language, 'vouchers-field-reason-lent');
    await waitFor(() => {
      expect(reason).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-payment-to-buyer-needs-reason'));
    await user.click(reason);
    await user.keyboard('x');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0]).toMatchObject({
      type: 'advance',
      input: {
        seller_account_id: 21,
        amount: '3000',
        date: '10-10-2026',
        reason: 'x',
        expected_goods: '',
        confirm_cash_below_zero: false,
      },
    });
    // The saved voucher is a bill of his: the books give him 3,000 more.
    await waitFor(() => {
      expect(store.rows).toHaveLength(1);
    });
    expect(store.bills.get(21)?.reduce((sum, one) => sum + one.open, 0)).toBe(12880 + 3000);
  });

  it('the preview shows his balance before and after as the backend worked it out', async () => {
    const user = await open();
    await pick(user, language, '0020', '0020');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('3000');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ type: 'advance', seller_account_id: 21, amount: '3000' }),
        null,
      );
    });
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'vouchers-new-balance'));
    });
    // 12,880 now; the fake backend says 15,880 after (the screen adds nothing).
    expect(pageText()).toContain('12,880');
    expect(pageText()).toContain('15,880');
  });

  it('a seller goes on as before: his advances, the expected goods, no reason, and the saved input says so', async () => {
    const user = await open();
    await pick(user, language, 'rahim', '0010');
    const advances = await panelText(language, 'vouchers-advances-title');
    expect(advances).toContain('15,000');
    expect(await labelled(language, 'vouchers-field-expected')).toBeInTheDocument();
    expect(
      screen.queryByLabelText(named(message(language, 'vouchers-field-reason-lent'))),
    ).toBeNull();
    expect(
      screen.queryByRole('heading', { name: named(message(language, 'vouchers-bills-title')) }),
    ).toBeNull();
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('1000');
    await user.click(await labelled(language, 'vouchers-field-expected'));
    await user.keyboard('mango');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0]).toMatchObject({
      type: 'advance',
      input: {
        seller_account_id: 31,
        amount: '1000',
        expected_goods: 'mango',
        confirm_cash_below_zero: false,
      },
    });
    // A seller's advance is the request it always was: no reason key at all.
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0].input).not.toHaveProperty(
      'reason',
    );
    expect(store.advances.get(31)).toBe(15000 + 1000);
  });

  it('changing from a buyer to a seller drops the reason and brings back the expected goods', async () => {
    const user = await open();
    await pick(user, language, '0020', '0020');
    await user.click(await labelled(language, 'vouchers-field-reason-lent'));
    await user.keyboard('urgent');
    // The other kind of account: the picker is emptied and a seller is chosen.
    const field = await dropdown(language, 'vouchers-field-party');
    await user.clear(field);
    await user.keyboard('rahim');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    expect(await labelled(language, 'vouchers-field-expected')).toBeInTheDocument();
    expect(
      screen.queryByLabelText(named(message(language, 'vouchers-field-reason-lent'))),
    ).toBeNull();
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('500');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0];
    expect(sent).toMatchObject({ input: { seller_account_id: 31 } });
    expect(sent?.input).not.toHaveProperty('reason');
  });

  it('saved, it says cash lent with its number in the payment series, and the advance list shows it with his type', async () => {
    const user = await open();
    await pick(user, language, '0020', '0020');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('3000');
    await user.click(await labelled(language, 'vouchers-field-reason-lent'));
    await user.keyboard('x');
    press('F10');
    // The backend saved it as a payment to him (cash lent): P-1, and the message names the cash lent.
    const toast = message(language, 'vouchers-saved', {
      kind: 'lent',
      number: message(language, 'vouchers-number-payment', { number: 1 }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    // The advance list asked for the cash lent too, and shows the new row with his type.
    await waitFor(() => {
      expect(vouchersApi.vouchersList).toHaveBeenLastCalledWith(
        expect.objectContaining({ voucher_type: 'advance', advance_view: true }),
      );
    });
    await waitFor(() => {
      expect(cell(501, 'number')).toBe(message(language, 'vouchers-number-payment', { number: 1 }));
    });
    expect(cell(501, 'kind')).toBe(message(language, 'accounts-type-buyer'));
    expect(cell(501, 'amount')).toBe('3,000');
  });

  it('the advance list holds the advances of sellers and the cash lent to buyers, each marked, and not a plain payment', async () => {
    store.rows.push(
      voucherRow({
        id: 601,
        voucher_type: 'advance',
        number: 1,
        account: refOf(31),
        account_type: 'seller',
      }),
      voucherRow({
        id: 602,
        voucher_type: 'payment',
        number: 2,
        account: refOf(21),
        account_type: 'buyer',
        reason: 'x',
      }),
      voucherRow({
        id: 603,
        voucher_type: 'payment',
        number: 3,
        account: refOf(32),
        account_type: 'seller',
      }),
    );
    await open();
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="601"]')).not.toBeNull();
    });
    expect(document.querySelector('.ag-row[row-id="602"]')).not.toBeNull();
    // A payment to a seller is the payment list's, not this one's.
    expect(document.querySelector('.ag-row[row-id="603"]')).toBeNull();
    expect(cell(601, 'number')).toBe(message(language, 'vouchers-number-advance', { number: 1 }));
    expect(cell(601, 'kind')).toBe(message(language, 'accounts-type-seller'));
    expect(cell(602, 'number')).toBe(message(language, 'vouchers-number-payment', { number: 2 }));
    expect(cell(602, 'kind')).toBe(message(language, 'accounts-type-buyer'));
  });

  it("opening a buyer's cash lent from the advance list opens it on the payment tab, which edits a payment", async () => {
    store.rows.push(
      voucherRow({
        id: 602,
        voucher_type: 'payment',
        number: 2,
        account: refOf(21),
        account_type: 'buyer',
        reason: 'x',
      }),
    );
    const { router } = await (async () => {
      const app = await renderApp('/bikri/voucher?type=advance');
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-advance-title'),
      });
      return app;
    })();
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="602"]')).not.toBeNull();
    });
    const target = document.querySelector<HTMLElement>('.ag-row[row-id="602"] [col-id="number"]');
    if (target === null) throw new Error('no cell');
    await userEvent.setup().dblClick(target);
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ type: 'payment', id: 602 });
    });
  });

  it("the list window of the advance tab is the backend's advances window, and the other tabs keep their own", async () => {
    const user = await open();
    await waitFor(() => {
      expect(windowsApi.windowsBounds).toHaveBeenCalledWith({ source: { kind: 'advances' } });
    });
    vi.mocked(windowsApi.windowsBounds).mockClear();
    const group = screen.getByRole('group', { name: message(language, 'vouchers-field-type') });
    await user.click(
      within(group).getByRole('button', { name: message(language, 'vouchers-type-payment') }),
    );
    await waitFor(() => {
      expect(windowsApi.windowsBounds).toHaveBeenCalledWith({
        source: { kind: 'vouchers', voucher_type: 'payment' },
      });
    });
  });

  it('a saved advance of a seller that is changed takes sellers only, as the backend keeps its type', async () => {
    store.rows.push(
      voucherRow({
        id: 601,
        voucher_type: 'advance',
        number: 1,
        account: refOf(31),
        account_type: 'seller',
        expected_goods: 'mango',
      }),
    );
    await renderApp('/bikri/voucher?id=601');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'vouchers-advance-title'),
    });
    const field = await dropdown(language, 'vouchers-field-seller');
    expect(screen.queryByLabelText(named(message(language, 'vouchers-field-party')))).toBeNull();
    const user = userEvent.setup();
    await user.click(field);
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(2);
    });
    for (const option of screen.getAllByRole('option')) {
      expect(visible(option.textContent)).not.toContain(message(language, 'accounts-type-buyer'));
    }
  });

  it('cash lent to a buyer that takes cash below zero asks first, like any advance', async () => {
    store = fakeVouchers({ cash: 2000 });
    const user = await open();
    await pick(user, language, '0020', '0020');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('3000');
    await user.click(await labelled(language, 'vouchers-field-reason-lent'));
    await user.keyboard('x');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-1,000' }));
    });
    press('F10');
    const no = await screen.findByRole('button', {
      name: new RegExp(message(language, 'common-cash-confirm-no')),
    });
    await waitFor(() => {
      expect(no).toHaveFocus();
    });
    expect(store.rows).toHaveLength(0);
    await userEvent.setup().click(no);
    expect(store.rows).toHaveLength(0);
  });

  it('the language switch in the middle keeps the buyer and the reason', async () => {
    const user = await open();
    await pick(user, language, '0020', '0020');
    await user.click(await labelled(language, 'vouchers-field-reason-lent'));
    await user.keyboard('urgent');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    const { act } = await import('@testing-library/react');
    const { useLanguageStore } = await import('../../src/shared/i18n');
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'vouchers-field-reason-lent')).value).toBe('urgent');
    expect(await dropdown(other, 'vouchers-field-party')).toHaveValue(
      other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    );
    const group = screen.getByRole('group', { name: message(other, 'vouchers-field-type') });
    expect(
      within(group).getByRole('button', { name: message(other, 'vouchers-type-advance') }),
    ).toHaveAttribute('aria-pressed', 'true');
  });
});
