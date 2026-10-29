// The cash book screens show what the REAL backend answers (plan 17, T4). The backend wrote each
// answer in its own test (tests/fixtures/*.json, copied, never edited). The screens are driven by
// those answers and by the refusals the backend really sent: the cash book page by page, the heads,
// the shares and workings of a shop expense, the partner total of a personal one, the cash
// question, the refusals at their fields, a partner's money taken, last month's proposals and a
// person's loans. In the Urdu and in the English app. The story is the PRD worked day of
// 10-10-2026 (Partner A takes 2,000, Aslam is lent 5,000).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { ASLAM_ROW } from '../helpers/cashbook';
import { ASLAM } from '../helpers/cashbook-books';
import { paisaForTest as paisaOf } from '../helpers/paisa';
import * as real from '../helpers/cashbook-real';
import {
  choose,
  labelled,
  named,
  pageText,
  pickAccount,
  prepareCashbook,
  startsWith,
  typeAmount,
} from '../helpers/cashbook-screen';
import { leaveAppWindow, press, renderApp } from '../helpers/app';
import { PAGE_ROWS, pagerButton } from '../helpers/pager';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

/** The backend's refusal as the command wrapper rejects with it. */
const refused = (error: typeof real.REFUSED_SHOP_MISTAKES) => new CommandError(error);

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the روکڑ بک with the real answers (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
    vi.mocked(cashbookApi.cashbookGet).mockImplementation((input) =>
      Promise.resolve(input.offset === 0 ? real.CASHBOOK_DAY : real.CASHBOOK_SECOND_PAGE),
    );
  });

  it('shows the opening, the closing, a minus balance, the words of each kind and the totals of the whole range', async () => {
    await renderApp('/rokar');
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-list-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('50,000');
    });
    const text = visible(list.textContent);
    // The first row is the opening entry; its other side is a named account.
    expect(text).toContain(message(language, 'cashbook-kind-opening'));
    expect(text).toContain(nameOf(language, 'ابتدائی فرق', 'Opening Difference'));
    expect(text).toContain(
      message(language, 'cashbook-kind-number', {
        kind: message(language, 'cashbook-kind-bikri-first'),
        number: 1,
      }),
    );
    // The totals are the backend's over all six rows, not the five on screen.
    expect(text).toContain('50,500');
    expect(text).toContain('15,000');
    expect(pageText()).toContain('32,700');
    // The opening cash of the range is below zero: the minus sign shows.
    expect(pageText()).toContain('-2,800');
    expect(pageText()).not.toContain('ERR_');
  });

  it('asks for the next page by the offset of the page size, and shows the loan rows with their person', async () => {
    // The real files hold a page of five rows. The screen pages by 25, so the first page says the
    // range has 30 rows (the other fields are the backend's own) and the next page is the real second one.
    vi.mocked(cashbookApi.cashbookGet).mockImplementation((input) =>
      Promise.resolve(
        input.offset === 0 ? { ...real.CASHBOOK_DAY, total_count: 30 } : real.CASHBOOK_SECOND_PAGE,
      ),
    );
    const { user } = { user: userEvent.setup() };
    await renderApp('/rokar');
    await user.click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
        expect.objectContaining({ offset: PAGE_ROWS, limit: PAGE_ROWS }),
      );
    });
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-list-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain(nameOf(language, 'اسلم', 'Aslam'));
    });
    expect(visible(list.textContent)).toContain(
      message(language, 'cashbook-kind-money-lent-return'),
    );
  });
});

describe.each(LANGUAGES)('the خرچہ screen with the real answers (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
    vi.mocked(cashbookApi.expenseHeads).mockResolvedValue(real.EXPENSE_HEADS);
  });

  const electricity = nameOf(language, 'بجلی', 'Electricity');

  async function open(type = 'shop_expense') {
    const app = await renderApp(`/rokar/expense?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-expense-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('lists the heads by the names inside `head`, in the order the backend sent', async () => {
    const { user } = await open();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'cashbook-field-head')),
      }),
    );
    const options = await screen.findAllByRole('option');
    expect(options.map((option) => visible(option.textContent))).toEqual([
      electricity,
      nameOf(language, 'تنخواہ', 'Salaries'),
    ]);
  });

  it('a shop expense: the three shares and their workings are the backend’s, and the note offers ذاتی خرچہ', async () => {
    vi.mocked(cashbookApi.expensePreview).mockResolvedValue(real.PREVIEW_SHOP);
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', electricity);
    await typeAmount(user, language, '3000');
    await user.click(await labelled(language, 'cashbook-field-paid-to'));
    await user.keyboard('x');
    const card = (
      await screen.findByRole('heading', { name: message(language, 'cashbook-shares-title') })
    ).closest('section');
    if (card === null) throw new Error('no shares card');
    await waitFor(() => {
      const text = visible(card.textContent);
      expect(text).toContain('1,200');
      expect(text).toContain('900');
    });
    // The working of the first share is written with the message of the backend.
    const infos = within(card).getAllByRole('button', {
      name: startsWith(message(language, 'calc-info-label')),
    });
    expect(infos).toHaveLength(3);
    const firstInfo = infos[0];
    if (firstInfo === undefined) throw new Error('no working');
    await user.click(firstInfo);
    const working = await screen.findByRole('dialog');
    expect(visible(working.textContent)).toContain('3,000');
    expect(visible(working.textContent)).toContain('40');
    expect(visible(working.textContent)).toContain('1,200');
    expect(visible(working.textContent)).not.toContain('working-expense-share');
    // The backend found a partner's name in "paid to": the note with its button shows.
    await user.keyboard('{Escape}');
    expect(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'cashbook-personal-make')),
      }),
    ).toBeInTheDocument();
  });

  it('a personal expense: the partner total before and after, and the others are not affected', async () => {
    vi.mocked(cashbookApi.expensePreview).mockResolvedValue(real.PREVIEW_PERSONAL);
    vi.mocked(cashbookApi.partnerTakes).mockResolvedValue(real.PARTNER_TAKES);
    const { user } = await open('personal_expense');
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    await typeAmount(user, language, '2000');
    const card = (
      await screen.findByRole('heading', { name: message(language, 'cashbook-effect-title') })
    ).closest('section');
    if (card === null) throw new Error('no effect card');
    await waitFor(() => {
      const text = visible(card.textContent);
      expect(text).toContain('-740');
      expect(text).toContain('-2,740');
    });
    const info = within(card).getAllByRole('button', {
      name: startsWith(message(language, 'calc-info-label')),
    })[0];
    if (info === undefined) throw new Error('no working');
    await user.click(info);
    const working = await screen.findByRole('dialog');
    expect(visible(working.textContent)).toContain('-740');
    expect(visible(working.textContent)).toContain('2,000');
    expect(visible(working.textContent)).not.toContain('working-partner-total');
    // His money taken is listed with its return-by day and what is still open.
    await user.keyboard('{Escape}');
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-takes-label')),
    });
    await waitFor(() => {
      const text = visible(list.textContent);
      expect(text).toContain('20-10-2026');
      expect(text).toContain('2,000');
    });
  });

  it('two mistakes together: each is told at its own field, in words', async () => {
    vi.mocked(cashbookApi.expensePreview).mockResolvedValue(real.PREVIEW_SHOP);
    vi.mocked(cashbookApi.saveShopExpense).mockRejectedValue(refused(real.REFUSED_SHOP_MISTAKES));
    const { user } = await open();
    await typeAmount(user, language, '100');
    press('F10');
    const amount = await labelled(language, 'cashbook-field-amount');
    await waitFor(() => {
      expect(amount).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-decimals-not-allowed'));
    expect(pageText()).toContain(message(language, 'err-expense-head-required'));
    expect(pageText()).not.toContain('ERR_');
    expect(amount.value).toBe('100');
  });

  it('the cash question opens the box with the cash after the entry, from a fresh preview', async () => {
    let tried = false;
    const below = {
      ...real.PREVIEW_PERSONAL,
      cash: {
        ...real.PREVIEW_PERSONAL.cash,
        cash_after_paisa: paisaOf(-2_230_000),
        below_zero: true,
        needs_confirm: true,
      },
    };
    vi.mocked(cashbookApi.expensePreview).mockImplementation(() =>
      Promise.resolve(tried ? below : real.PREVIEW_PERSONAL),
    );
    vi.mocked(cashbookApi.savePersonalExpense).mockImplementation(() => {
      tried = true;
      return Promise.reject(refused(real.REFUSED_PERSONAL_CASH_CONFIRM));
    });
    const { user } = await open('personal_expense');
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'ب', 'Partner B'));
    await typeAmount(user, language, '60000');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('plot');
    await waitFor(() => {
      expect(cashbookApi.expensePreview).toHaveBeenCalled();
    });
    press('F10');
    const no = await screen.findByRole('button', {
      name: startsWith(message(language, 'common-cash-confirm-no')),
    });
    await waitFor(() => {
      expect(no).toHaveFocus();
    });
    expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-22,300' }));
    expect(pageText()).not.toContain('ERR_');
  });

  it('a return above what is open is refused at the amount with the most he can return', async () => {
    vi.mocked(cashbookApi.expensePreview).mockResolvedValue(real.PREVIEW_PERSONAL);
    vi.mocked(cashbookApi.savePersonalReturn).mockRejectedValue(
      refused(real.REFUSED_PERSONAL_RETURN_EXCEEDS),
    );
    const { user } = await open('personal_return');
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    await typeAmount(user, language, '2500');
    press('F10');
    const amount = await labelled(language, 'cashbook-field-amount');
    await waitFor(() => {
      expect(amount).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-return-exceeds-open', { open: '2,000' }));
  });

  it('last month’s proposals show the head, the amount and the total as sent', async () => {
    vi.mocked(cashbookApi.repeatLastMonth).mockResolvedValue(real.REPEAT_LAST_MONTH);
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'cashbook-repeat-open')),
      }),
    );
    const panel = (
      await screen.findByRole('heading', { name: message(language, 'cashbook-repeat-title') })
    ).closest('section');
    if (panel === null) throw new Error('no panel');
    await waitFor(() => {
      expect(visible(panel.textContent)).toContain('2,800');
    });
    expect(visible(panel.textContent)).toContain(electricity);
    await user.click(
      within(panel).getByRole('button', {
        name: startsWith(message(language, 'cashbook-repeat-fill')),
      }),
    );
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('2,800');
    // The text typed last month comes into the form as it was, for the user to check.
    expect((await labelled(language, 'cashbook-field-paid-to')).value).toBe('WAPDA');
  });
});

describe.each(LANGUAGES)('the ادھار رقم screen with the real answers (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language, [ASLAM_ROW]);
    vi.mocked(cashbookApi.expenseHeads).mockResolvedValue(real.EXPENSE_HEADS);
    vi.mocked(cashbookApi.lentList).mockResolvedValue(real.LENT_LIST);
  });

  const aslam = nameOf(language, ASLAM.name_ur, ASLAM.name_en);

  async function open(type = 'money_lent') {
    const app = await renderApp(`/rokar/lent?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-lent-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('the loans of a person: what was lent, returned and is open, with the balance as sent', async () => {
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-lent-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('5,000');
    });
    const text = visible(list.textContent);
    expect(text).toContain('3,000');
    expect(text).toContain('2,000');
    expect(text).toContain('15-10-2026');
    expect(text).toContain('گھریلو ضرورت');
    expect(pageText()).not.toContain('ERR_');
  });

  it('an opening debit on his account is a loan with no voucher: the word, not a number 0', async () => {
    const [first] = real.LENT_LIST.rows;
    if (first === undefined) throw new Error('no row');
    vi.mocked(cashbookApi.lentList).mockResolvedValue({
      ...real.LENT_LIST,
      rows: [{ ...first, voucher_id: 0, number: 0, reason: '' }],
    });
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-lent-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain(message(language, 'cashbook-take-opening'));
    });
    expect(visible(list.textContent)).not.toContain(
      message(language, 'vouchers-number-money-lent', { number: 0 }),
    );
  });

  it('a return-by day before the entry is told at that field, in words', async () => {
    vi.mocked(cashbookApi.saveMoneyLent).mockRejectedValue(
      refused(real.REFUSED_RETURN_BY_BEFORE_ENTRY),
    );
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    await typeAmount(user, language, '5000');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('x');
    await user.click(await labelled(language, 'cashbook-field-return-by'));
    await user.keyboard('15-10-2026');
    press('F10');
    const field = await labelled(language, 'cashbook-field-return-by');
    await waitFor(() => {
      expect(field).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-return-by-before-entry'));
  });

  it('a salary return names the head from the heads the backend sent (Salaries)', async () => {
    const { user } = await open('money_lent_return');
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    await choose(user, language, 'cashbook-field-mode', message(language, 'cashbook-mode-salary'));
    await choose(
      user,
      language,
      'cashbook-field-salary-head',
      nameOf(language, 'تنخواہ', 'Salaries'),
    );
    await typeAmount(user, language, '2000');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveMoneyLentReturn).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.saveMoneyLentReturn).mock.calls[0]?.[0]).toEqual({
      date: '10-10-2026',
      person_account_id: 61,
      amount: '2000',
      mode: 'salary',
      salary_head_id: 2,
      narration: null,
    });
  });
});
