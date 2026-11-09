// A saved entry of the روکڑ area open for edit (plan 44, PRD FR-7.70, FR-7.67), in the Urdu and the
// English app, against a fake backend: the خرچہ screen opens it with `?id=` filled as it was saved,
// the type list shut, the amber band, the change list from the backend, the reason when the entry is
// not of today, and the save goes to `vouchers_edit`; an entry of another screen opens on its own.
// The ادھار رقم and الگ کھاتہ screens do the same.

import { fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES } from '../../src/i18n';
import { HEADS } from '../helpers/cashbook-books';
import { labelled, named, prepareCashbook, startsWith } from '../helpers/cashbook-screen';
import { leaveAppWindow, press, renderApp } from '../helpers/app';
import { noChangeReview } from '../helpers/edit-real';
import { message, visible } from '../helpers/render';
import { paisaForTest } from '../helpers/paisa';
import { voucherRow } from '../helpers/voucher-books';
import { CLOSED_DAY_ENTRY, lockedEntry } from '../helpers/entry-flags';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.clearAllMocks();
});
afterEach(() => {
  leaveAppWindow();
});

/** A shop expense of today as saved: electricity, 3,000, paid to the board, a note. */
function shopExpense(overrides: Parameters<typeof voucherRow>[0] = {}) {
  return {
    row: voucherRow({
      id: 77,
      number: 3,
      voucher_type: 'shop_expense',
      amount_paisa: paisaForTest(300_000),
      account: null,
      ...overrides,
    }),
    entry: {
      type: 'shop_expense' as const,
      date: '10-10-2026',
      expense_head_id: 1,
      amount: '3000',
      paid_to: 'Wapda',
      narration: 'October bill',
      confirm_cash_below_zero: false,
    },
  };
}

describe.each(LANGUAGES)('an expense open for edit (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(noChangeReview());
  });

  async function open(id: number) {
    const app = await renderApp(`/rokar/expense?id=${String(id)}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-expense-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('fills the form as it was saved, shows the band and shuts the type list', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(shopExpense());
    await open(77);
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('3,000');
    expect((await labelled(language, 'cashbook-field-paid-to')).value).toBe('Wapda');
    const head = HEADS[0];
    expect(
      await screen.findByDisplayValue(
        language === 'ur' ? (head?.name_ur ?? '') : (head?.name_en ?? ''),
      ),
    ).toBeInTheDocument();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-band', {
        what: message(language, 'glossary-kharcha'),
        number: '3',
        date: '10-10-2026',
      }),
    );
    for (const button of screen
      .getByRole('group', { name: named(message(language, 'cashbook-expense-type')) })
      .querySelectorAll('button')) {
      expect(button).toBeDisabled();
    }
  });

  it('saves with vouchers_edit: the id, the version and the entry as typed, no reason for today', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(shopExpense());
    vi.mocked(vouchersApi.vouchersEdit).mockResolvedValue({
      voucher_id: 77,
      number: 3,
      version: 2,
      entry_id: 9,
      cash_after: paisaForTest(0),
    } as never);
    const { user, router } = await open(77);
    const amount = await labelled(language, 'cashbook-field-amount');
    await user.clear(amount);
    await user.type(amount, '3500');
    await waitFor(() => {
      expect(vouchersApi.vouchersEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersEdit).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(vouchersApi.vouchersEdit).mock.calls[0]?.[0];
    expect(sent).toMatchObject({
      id: 77,
      version: 1,
      reason: '',
      entry: { type: 'shop_expense', amount: '3500', expense_head_id: 1, paid_to: 'Wapda' },
    });
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('id');
    });
  });

  it('an entry of an earlier day asks why before it saves', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      shopExpense({ entry_date: '2026-10-09' }),
    );
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(
      noChangeReview('2026-10-09', true),
    );
    vi.mocked(vouchersApi.vouchersEdit).mockResolvedValue({
      voucher_id: 77,
      number: 3,
      version: 2,
      entry_id: 9,
      cash_after: paisaForTest(0),
    } as never);
    const { user } = await open(77);
    const amount = await labelled(language, 'cashbook-field-amount');
    await user.clear(amount);
    await user.type(amount, '3500');
    await waitFor(() => {
      expect(vouchersApi.vouchersEditPreview).toHaveBeenCalled();
    });
    press('F10');
    const reason = await screen.findByRole('combobox', {
      name: named(message(language, 'edit-reason-label')),
    });
    expect(vouchersApi.vouchersEdit).not.toHaveBeenCalled();
    await user.type(reason, 'wrong bill{Enter}');
    await waitFor(() => {
      expect(vouchersApi.vouchersEdit).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersEdit).mock.calls[0]?.[0]).toMatchObject({
      reason: 'wrong bill',
    });
  });

  it('a loan opened on the expense address moves to its own screen', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue({
      row: voucherRow({ id: 88, number: 2, voucher_type: 'money_lent' }),
      entry: {
        type: 'money_lent',
        date: '10-10-2026',
        person_account_id: 21,
        amount: '5000',
        reason: 'home',
        return_by: null,
        narration: null,
        confirm_cash_below_zero: false,
      },
    });
    const { router } = await renderApp('/rokar/expense?id=88');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/lent');
    });
    expect(router.state.location.search).toMatchObject({ id: 88, type: 'money_lent' });
  });
});

/** A loan as saved: 5,000 lent to Akram, for the home, with a note. */
function moneyLent(overrides: Parameters<typeof voucherRow>[0] = {}) {
  return {
    row: voucherRow({ id: 88, number: 2, voucher_type: 'money_lent', ...overrides }),
    entry: {
      type: 'money_lent' as const,
      date: '10-10-2026',
      person_account_id: 21,
      amount: '5000',
      reason: 'home',
      return_by: null,
      narration: 'for the house',
      confirm_cash_below_zero: false,
    },
  };
}

const EDITED = {
  voucher_id: 88,
  number: 2,
  version: 2,
  entry_id: 9,
  cash_after: paisaForTest(0),
} as never;

describe.each(LANGUAGES)('a loan open for edit (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(noChangeReview());
  });

  async function open(id: number) {
    const app = await renderApp(`/rokar/lent?id=${String(id)}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-lent-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('fills the form as it was saved, shows the band and shuts the type list', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(moneyLent());
    await open(88);
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('5,000');
    expect((await labelled(language, 'cashbook-field-reason')).value).toBe('home');
    expect(
      await screen.findByDisplayValue(startsWith(language === 'ur' ? 'اکرم' : 'Akram')),
    ).toBeInTheDocument();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-band', {
        what: message(language, 'glossary-naqd-udhar'),
        number: '2',
        date: '10-10-2026',
      }),
    );
    for (const button of screen
      .getByRole('group', { name: named(message(language, 'cashbook-lent-type')) })
      .querySelectorAll('button')) {
      expect(button).toBeDisabled();
    }
  });

  it('saves with vouchers_edit: the id, the version and the entry as typed', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(moneyLent());
    vi.mocked(vouchersApi.vouchersEdit).mockResolvedValue(EDITED);
    const { user, router } = await open(88);
    const amount = await labelled(language, 'cashbook-field-amount');
    await user.clear(amount);
    await user.type(amount, '6000');
    await waitFor(() => {
      expect(vouchersApi.vouchersEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersEdit).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersEdit).mock.calls[0]?.[0]).toMatchObject({
      id: 88,
      version: 1,
      reason: '',
      entry: { type: 'money_lent', amount: '6000', person_account_id: 21, reason: 'home' },
    });
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('id');
    });
  });

  it('the preview of the change is asked as a replacement of this voucher', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(moneyLent());
    const { user } = await open(88);
    const amount = await labelled(language, 'cashbook-field-amount');
    await user.clear(amount);
    await user.type(amount, '6000');
    await waitFor(() => {
      const asked = vi.mocked(vouchersApi.vouchersPreview).mock.calls.at(-1);
      expect(asked?.[0]).toMatchObject({ type: 'money_lent', amount: '6000' });
      expect(asked?.[1]).toBe(88);
    });
  });

  it('a khata entry opened on the loan address moves to its own screen', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(khataOut());
    const { router } = await renderApp('/rokar/lent?id=91');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/khata');
    });
    expect(router.state.location.search).toMatchObject({ id: 91, type: 'khata_out' });
  });
});

/** A khata entry as saved: 10,000 paid out of the family khata for the family's costs. */
function khataOut(overrides: Parameters<typeof voucherRow>[0] = {}) {
  return {
    row: voucherRow({ id: 91, number: 4, voucher_type: 'khata_out', ...overrides }),
    entry: {
      type: 'khata_out' as const,
      date: '10-10-2026',
      khata_account_id: 21,
      amount: '10000',
      reason: 'family costs',
      narration: null,
      confirm_cash_below_zero: false,
    },
  };
}

describe.each(LANGUAGES)('a khata entry open for edit (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(noChangeReview());
  });

  async function open(id: number) {
    const app = await renderApp(`/rokar/khata?id=${String(id)}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-khata-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('fills the form as it was saved, shows the band and shuts the type list', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(khataOut());
    await open(91);
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('10,000');
    expect((await labelled(language, 'cashbook-field-reason')).value).toBe('family costs');
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-band', {
        what: message(language, 'glossary-khata'),
        number: '4',
        date: '10-10-2026',
      }),
    );
    for (const button of screen
      .getByRole('group', { name: named(message(language, 'cashbook-khata-type')) })
      .querySelectorAll('button')) {
      expect(button).toBeDisabled();
    }
  });

  it('an entry of an earlier day asks why, then saves with the reason', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      khataOut({ entry_date: '2026-10-09' }),
    );
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(
      noChangeReview('2026-10-09', true),
    );
    vi.mocked(vouchersApi.vouchersEdit).mockResolvedValue(EDITED);
    const { user } = await open(91);
    const amount = await labelled(language, 'cashbook-field-amount');
    await user.clear(amount);
    await user.type(amount, '12000');
    await waitFor(() => {
      expect(vouchersApi.vouchersEditPreview).toHaveBeenCalled();
    });
    press('F10');
    const reason = await screen.findByRole('combobox', {
      name: named(message(language, 'edit-reason-label')),
    });
    expect(vouchersApi.vouchersEdit).not.toHaveBeenCalled();
    await user.type(reason, 'typed wrong{Enter}');
    await waitFor(() => {
      expect(vouchersApi.vouchersEdit).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersEdit).mock.calls[0]?.[0]).toMatchObject({
      id: 91,
      reason: 'typed wrong',
      entry: { type: 'khata_out', amount: '12000', khata_account_id: 21 },
    });
  });
});

describe.each(LANGUAGES)('an entry that cannot be edited yet (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(noChangeReview());
  });

  it('a closed day shows the entry as stored and a button that reopens the day first', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      moneyLent({ ...CLOSED_DAY_ENTRY, entry_date: '2026-10-05' }),
    );
    await renderApp('/rokar/lent?id=88');
    expect(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'edit-action-fix-day')),
      }),
    ).toBeEnabled();
    // No form: the amount is not an input, it is the stored figure.
    expect(screen.queryByLabelText(named(message(language, 'cashbook-field-amount')))).toBeNull();
    expect(visible(document.body.textContent)).toContain(message(language, 'vouchers-view-closed'));
  });

  it('a locked entry says why in words and its Edit button is off', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      khataOut({ ...lockedEntry('goods_moved') }),
    );
    await renderApp('/rokar/khata?id=91');
    expect(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'edit-action-edit')),
      }),
    ).toBeDisabled();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-locked-goods-moved'),
    );
  });
});

describe.each(LANGUAGES)('opening an entry from the lists under the forms (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
  });

  it('the day’s shop expenses: the row that has focus is edited with the Edit button', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(shopExpense());
    vi.mocked(vouchersApi.vouchersList).mockResolvedValue({
      rows: [shopExpense().row],
      total_count: 1,
      totals: { cancelled_count: 0, total_paisa: paisaForTest(300_000) },
      months: [],
    });
    const { router } = await renderApp('/rokar/expense?type=shop_expense');
    const user = userEvent.setup();
    const edit = () =>
      screen.getByRole('button', { name: startsWith(message(language, 'edit-action-edit')) });
    await user.click(
      await screen.findByText(
        named(message(language, 'vouchers-number-shop-expense', { number: 3 })),
      ),
    );
    await waitFor(() => {
      expect(edit()).toBeEnabled();
    });
    await user.click(edit());
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ type: 'shop_expense', id: 77 });
    });
  });

  it('a person’s loans: Enter or a double click on a loan opens it', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(moneyLent());
    vi.mocked(cashbookApi.lentList).mockResolvedValue({
      person: { id: 21, code: '0020', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' },
      balance: { amount_paisa: paisaForTest(500_000), side: 'dr' },
      rows: [
        {
          voucher_id: 89,
          number: 3,
          entry_date: '2026-10-10',
          amount_paisa: paisaForTest(500_000),
          reason: 'older loan',
          return_by: null,
          returned_paisa: paisaForTest(0),
          open_paisa: paisaForTest(500_000),
          days_past_return_by: null,
        },
      ],
      total_count: 1,
      totals: {
        lent_paisa: paisaForTest(500_000),
        returned_paisa: paisaForTest(0),
        open_paisa: paisaForTest(500_000),
      },
      months: [],
    } as never);
    const { router } = await renderApp('/rokar/lent?id=88');
    fireEvent.doubleClick(await screen.findByText('older loan'));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ id: 89 });
    });
  });

  it('a khata’s statement: a payment opens its voucher, a sale of its goods opens nothing', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(khataOut());
    vi.mocked(cashbookApi.khataStatement).mockResolvedValue({
      khata: { id: 21, code: '0020', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' },
      opening_paisa: paisaForTest(0),
      closing_paisa: paisaForTest(-1_000_000),
      rows: [
        {
          kind: 'paid_out',
          entry_id: 5,
          voucher_id: 92,
          number: 5,
          entry_date: '2026-10-10',
          reason: 'earlier payment',
          narration: '',
          amount_paisa: paisaForTest(-1_000_000),
          running_paisa: paisaForTest(-1_000_000),
        },
        {
          kind: 'sale',
          entry_id: 6,
          voucher_id: null,
          number: 1,
          entry_date: '2026-10-10',
          reason: 'goods sold',
          narration: '',
          amount_paisa: paisaForTest(0),
          running_paisa: paisaForTest(-1_000_000),
        },
      ],
      total_count: 2,
      totals: { paid_out_paisa: paisaForTest(1_000_000), received_paisa: paisaForTest(0) },
      months: [],
    } as never);
    const { router } = await renderApp('/rokar/khata?id=91');
    fireEvent.doubleClick(await screen.findByText('goods sold'));
    expect(router.state.location.search).toMatchObject({ id: 91 });
    fireEvent.doubleClick(await screen.findByText('earlier payment'));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ id: 92 });
    });
  });
});

describe.each(LANGUAGES)('opening an entry from the cash book (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
  });

  /** The cash book answers with these rows. */
  function bookWith(...rows: object[]) {
    vi.mocked(cashbookApi.cashbookGet).mockResolvedValue({
      opening_paisa: paisaForTest(0),
      closing_paisa: paisaForTest(0),
      rows: rows.map((row, index) => ({
        entry_id: 900 + index,
        entry_date: '2026-10-10',
        posting_type: 'shop_expense',
        number: 3,
        account: null,
        narration: 'October bill',
        in_paisa: paisaForTest(0),
        out_paisa: paisaForTest(300_000),
        running_paisa: paisaForTest(-300_000),
        ...row,
      })),
      total_count: rows.length,
      totals: { in_paisa: paisaForTest(0), out_paisa: paisaForTest(300_000) },
      months: [],
    } as never);
  }

  /** The kind cell of the first row bookWith makes. */
  const kindCell = () =>
    waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        '.ag-row[row-id="900-shop_expense"] [col-id="kind"]',
      );
      if (found === null) throw new Error('no cash book row');
      return found;
    });

  it('a voucher row opens its entry on the screen of its kind, ready to edit', async () => {
    bookWith({ source: { kind: 'voucher', id: 77 } });
    vi.mocked(vouchersApi.vouchersGet).mockResolvedValue(shopExpense().row);
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(shopExpense());
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(noChangeReview());
    const { router } = await renderApp('/rokar');
    fireEvent.doubleClick(await kindCell());
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/expense');
    });
    expect(router.state.location.search).toMatchObject({ type: 'shop_expense', id: 77 });
    expect(
      await screen.findByLabelText<HTMLInputElement>(
        named(message(language, 'cashbook-field-amount')),
      ),
    ).toHaveValue('3,000');
  });

  it('a row with no source opens nothing', async () => {
    bookWith({ source: null });
    const { router } = await renderApp('/rokar');
    fireEvent.doubleClick(await kindCell());
    expect(router.state.location.pathname).toBe('/rokar');
  });
});
