// The capital, partner overview, partner statement, الگ کھاتہ and جنرل screens show what the REAL
// backend answers (plan 17, T4, second half). The backend wrote each answer in its own test
// (tests/fixtures/*.json, copied, never edited). The screens are driven by those answers and by the
// refusals the backend really sent, in the Urdu and in the English app. The story is the PRD worked
// day of 10-10-2026 with partner ب's 50,000 of capital on 11-10-2026 (E2E-35, E2E-37), the family
// khata (E2E-38) and the E2E-34 statement of partner الف (-4,160, +480, +3,384).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as partnersApi from '../../src/features/partners/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { cellOf, focusedIn, gridOf, textOf, typeCodeAndEnter } from '../helpers/bikri-screen';
import { KHATA_ROW } from '../helpers/cashbook';
import {
  dropdown,
  labelled,
  named,
  pageText,
  pickAccount,
  prepareCashbook,
  startsWith,
  typeAmount,
} from '../helpers/cashbook-screen';
import { paisaForTest } from '../helpers/paisa';
import { leaveAppWindow, press, renderApp } from '../helpers/app';
import { preparePartnerMoney } from '../helpers/partner-money';
import * as real from '../helpers/partner-money-real';
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
const refused = (error: typeof real.REFUSED_KHATA_MISTAKES) => new CommandError(error);

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the سرمایہ screen with the real answers (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
    vi.mocked(partnersApi.capitalHistory).mockResolvedValue(real.CAPITAL_HISTORY);
  });

  async function open(search: string) {
    const app = await renderApp(`/partners/capital${search}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'partners-capital-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('shows the history the backend sent: the kind, the number of the series and the balance', async () => {
    await open('?type=capital_in&partner=2');
    const list = await screen.findByRole('group', {
      name: named(message(language, 'partners-capital-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('50,000');
    });
    const text = visible(list.textContent);
    expect(text).toContain(message(language, 'partners-capital-kind-invested'));
    expect(text).toContain(message(language, 'vouchers-number-capital-in', { number: 1 }));
    expect(text).toContain('11-10-2026');
    expect(pageText()).toContain(message(language, 'partners-capital-balance'));
    expect(pageText()).not.toContain('ERR_');
  });

  it('taking back 60,000 of 50,000 shows the backend’s warning, with the balance and the part above it, and still saves', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_CAPITAL_OUT);
    vi.mocked(partnersApi.saveCapitalOut).mockResolvedValue({
      ...real.SAVED_CAPITAL_IN,
      voucher_type: 'capital_out',
    });
    const { user } = await open('?type=capital_out&partner=2');
    await typeAmount(user, language, '60000');
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'err-capital-exceeds-balance', { balance: '50,000', over: '10,000' }),
      );
    });
    press('F10');
    await waitFor(() => {
      expect(partnersApi.saveCapitalOut).toHaveBeenCalledTimes(1);
    });
    // The request is exactly the text typed, with the type tag taken off (the backend's own shape).
    expect(vi.mocked(partnersApi.saveCapitalOut).mock.calls[0]?.[0]).toMatchObject({
      date: '10-10-2026',
      partner_id: 2,
      amount: '60000',
      confirm_cash_below_zero: false,
    });
  });

  it('the cash question opens the box with the cash after the entry, from a fresh preview', async () => {
    let tried = false;
    const below = {
      ...real.PREVIEW_CAPITAL_OUT,
      warnings: [],
      cash: {
        ...real.PREVIEW_CAPITAL_OUT.cash,
        cash_after_paisa: paisaForTest(-3_000_000),
        below_zero: true,
        needs_confirm: true,
      },
    };
    vi.mocked(vouchersApi.vouchersPreview).mockImplementation(() =>
      Promise.resolve(tried ? below : real.PREVIEW_CAPITAL_OUT),
    );
    vi.mocked(partnersApi.saveCapitalOut).mockImplementation(() => {
      tried = true;
      return Promise.reject(refused(real.REFUSED_CAPITAL_OUT_CASH));
    });
    const { user } = await open('?type=capital_out&partner=2');
    await typeAmount(user, language, '120000');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenCalled();
    });
    press('F10');
    const no = await screen.findByRole('button', {
      name: startsWith(message(language, 'common-cash-confirm-no')),
    });
    await waitFor(() => {
      expect(no).toHaveFocus();
    });
    expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-30,000' }));
    expect(pageText()).not.toContain('ERR_');
  });
});

describe.each(LANGUAGES)('the partner overview with the real answers (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
    vi.mocked(partnersApi.partnersOverview).mockResolvedValue(real.OVERVIEW);
  });

  const cardOf = async (name: string) => {
    const heading = await screen.findByText(name);
    const card = heading.closest('section');
    if (!(card instanceof HTMLElement)) throw new Error('no card');
    return card;
  };

  it('shows the figures of E2E-37 and E2E-35 for each partner, with the total in the due colour when minus', async () => {
    await renderApp('/partners');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-title')),
    });
    const a = await cardOf(nameOf(language, 'الف', 'Partner A'));
    // The real fixture of the backend: A took 2,000 and has -1,620 in all; B has 50,284; C 284.
    expect(visible(a.textContent)).toContain('2,000');
    const total = within(a)
      .getByText(named(message(language, 'partners-metric-total')))
      .closest('div')
      ?.querySelector('dd');
    expect(visible(total?.textContent)).toBe('-1,620');
    expect(total?.querySelector('.text-due')).not.toBeNull();
    const b = await cardOf(nameOf(language, 'ب', 'Partner B'));
    expect(visible(b.textContent)).toContain('50,284');
    expect(visible(b.textContent)).toContain('50,000');
    const c = await cardOf(nameOf(language, 'ج', 'Partner C'));
    expect(visible(c.textContent)).toContain('284');
    expect(pageText()).not.toContain('ERR_');
  });
});

describe.each(LANGUAGES)('the partner statement with the real answers (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
  });

  const grid = () => gridOf(language, 'partners-statement-label');
  const rows = () => grid().querySelectorAll('.ag-row[row-index]').length;

  async function open(search: string) {
    const app = await renderApp(`/partners/statement${search}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-statement-title')),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('partner ب: the three lines and the running total ending at 50,284, with the series of each voucher', async () => {
    vi.mocked(partnersApi.partnersStatement).mockResolvedValue(real.STATEMENT_B);
    await open('?partner=2');
    await waitFor(() => {
      expect(rows()).toBe(3);
    });
    expect([0, 1, 2].map((row) => textOf(grid(), row, 'amount'))).toEqual([
      '1,184',
      '-900',
      '50,000',
    ]);
    expect([0, 1, 2].map((row) => textOf(grid(), row, 'running'))).toEqual([
      '1,184',
      '284',
      '50,284',
    ]);
    expect(textOf(grid(), 1, 'detail')).toBe(
      message(language, 'partners-statement-detail-expense-share', {
        voucher: message(language, 'vouchers-number-shop-expense', { number: 1 }),
        subject: nameOf(language, 'بجلی', 'Electricity'),
      }),
    );
    expect(textOf(grid(), 2, 'detail')).toBe(
      message(language, 'partners-statement-detail-capital-in', {
        voucher: message(language, 'vouchers-number-capital-in', { number: 1 }),
      }),
    );
    expect(textOf(grid(), 2, 'date')).toBe('11-10-2026');
    expect(partnersApi.partnersStatement).toHaveBeenCalledWith(
      expect.objectContaining({ partner_id: 2, from: null, to: null, offset: 0 }),
    );
    expect(pageText()).not.toContain('ERR_');
  });

  it('E2E-34: partner الف has -4,160, +480 and +3,384, ending at -296, with the trip and the purchase named', async () => {
    vi.mocked(partnersApi.partnersStatement).mockResolvedValue(real.STATEMENT_A_E2E34);
    await open('?partner=1');
    await waitFor(() => {
      expect(rows()).toBe(3);
    });
    expect([0, 1, 2].map((row) => textOf(grid(), row, 'amount'))).toEqual([
      '-4,160',
      '480',
      '3,384',
    ]);
    expect([0, 1, 2].map((row) => textOf(grid(), row, 'running'))).toEqual([
      '-4,160',
      '-3,680',
      '-296',
    ]);
    expect(textOf(grid(), 0, 'detail')).toBe(
      message(language, 'partners-statement-detail-trip-paid', {
        kind: 'purchase',
        trip: message(language, 'trips-number', { number: 1 }),
        secondary: message(language, 'purchases-number', { number: 1 }),
      }),
    );
    const credit = textOf(grid(), 2, 'detail');
    // The sale line names the item, 15 box, the sales money, the cost and his 40%.
    expect(credit).toContain(nameOf(language, 'مالٹا', 'Orange'));
    for (const figure of ['15', '8,460', '7,800', '660', '40']) expect(credit).toContain(figure);
    expect(pageText()).toContain('-296');
  });

  it('a salary deduction and a جنرل keep the series of their own voucher (LR- and G-)', async () => {
    const [income, expense] = real.STATEMENT_B.lines;
    if (income === undefined || expense === undefined) throw new Error('no lines');
    vi.mocked(partnersApi.partnersStatement).mockResolvedValue({
      ...real.STATEMENT_B,
      lines: [
        { ...expense, number: 5, values: { kind: 'salary' } },
        { ...expense, number: 2, values: { kind: 'general' } },
      ],
      total_count: 2,
    });
    await open('?partner=2');
    await waitFor(() => {
      expect(rows()).toBe(2);
    });
    expect(textOf(grid(), 0, 'detail')).toContain(
      message(language, 'vouchers-number-money-lent-return', { number: 5 }),
    );
    expect(textOf(grid(), 1, 'detail')).toContain(
      message(language, 'vouchers-number-general', { number: 2 }),
    );
    expect(cellOf(grid(), 0, 'detail')).toBeInTheDocument();
  });

  it('a page of two of three: the next page starts at the offset the backend counts', async () => {
    vi.mocked(partnersApi.partnersStatement).mockImplementation((input) =>
      Promise.resolve(input.offset === 0 ? real.STATEMENT_A_PAGE : real.STATEMENT_B),
    );
    const { user } = await open('?partner=1');
    await waitFor(() => {
      expect(rows()).toBe(2);
    });
    expect(pageText()).toContain(message(language, 'common-list-count', { shown: 2, total: 3 }));
    await user.click(
      screen.getByRole('button', { name: startsWith(message(language, 'common-list-more')) }),
    );
    await waitFor(() => {
      expect(partnersApi.partnersStatement).toHaveBeenLastCalledWith(
        expect.objectContaining({ offset: 2 }),
      );
    });
  });
});

describe.each(LANGUAGES)('the الگ کھاتہ screen with the real answers (%s)', (language) => {
  const family = nameOf(language, 'خاندانی کھاتہ', 'Family Khata');

  beforeEach(() => {
    prepareCashbook(language, [KHATA_ROW]);
    vi.mocked(cashbookApi.khataStatement).mockResolvedValue(real.KHATA_STATEMENT);
  });

  async function open(type = 'khata_out') {
    const app = await renderApp(`/rokar/khata?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-khata-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('lists the entries as the backend sent them: minus for paid out, plus for received, and the balance at the end', async () => {
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-khata', 'family', family);
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-khata-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('-10,000');
    });
    const text = visible(list.textContent);
    expect(text).toContain('4,000');
    expect(text).toContain(message(language, 'cashbook-khata-kind-paid-out'));
    expect(text).toContain(message(language, 'cashbook-khata-kind-received'));
    // The closing balance above the list is the backend's: -6,000, from the khata's side.
    expect(pageText()).toContain('-6,000');
    expect(cashbookApi.khataStatement).toHaveBeenCalledWith(
      expect.objectContaining({ khata_account_id: 71, from: null, to: null, offset: 0 }),
    );
    expect(pageText()).not.toContain('ERR_');
  });

  it('three mistakes together are told at their own fields, in words: decimals, no khata, no reason', async () => {
    vi.mocked(cashbookApi.saveKhataOut).mockRejectedValue(refused(real.REFUSED_KHATA_MISTAKES));
    const { user } = await open();
    await typeAmount(user, language, '10');
    press('F10');
    const amount = await labelled(language, 'cashbook-field-amount');
    await waitFor(() => {
      expect(amount).toHaveAttribute('aria-invalid', 'true');
    });
    expect(await dropdown(language, 'cashbook-field-khata')).toHaveAttribute(
      'aria-invalid',
      'true',
    );
    expect(await labelled(language, 'cashbook-field-reason')).toHaveAttribute(
      'aria-invalid',
      'true',
    );
    expect(pageText()).toContain(message(language, 'err-decimals-not-allowed'));
    expect(pageText()).toContain(message(language, 'err-khata-required'));
    expect(pageText()).toContain(message(language, 'err-reason-required'));
    expect(pageText()).not.toContain('ERR_');
    // What was typed stays.
    expect(amount.value).toBe('10');
  });
});

describe.each(LANGUAGES)('the جنرل screen with the real answers (%s)', (language) => {
  const ELECTRICITY = '05010001';
  const CASH = '01010001';
  const grid = () => gridOf(language, 'cashbook-general-grid-label');
  const saveButton = () =>
    screen.getByRole('button', { name: startsWith(message(language, 'layout-save-save')) });

  beforeEach(() => {
    preparePartnerMoney(language);
  });

  async function open() {
    const app = await renderApp('/rokar/general');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'cashbook-general-screen-title')),
    });
    await waitFor(() => {
      expect(cellOf(grid(), 1, 'cr')).toBeInTheDocument();
    });
    return { ...app, user: userEvent.setup() };
  }

  async function account(user: ReturnType<typeof userEvent.setup>, row: number, code: string) {
    await user.click(cellOf(grid(), row, 'account'));
    await typeCodeAndEnter(user, code);
    await waitFor(() => {
      expect(focusedIn(grid())).toBe(`${String(row)}:dr`);
    });
  }

  /** Electricity 1,000 on the بنام side and cash on the جمع side. */
  async function typeEntry(user: ReturnType<typeof userEvent.setup>, credit: string) {
    await account(user, 0, ELECTRICITY);
    await user.keyboard('1000{Enter}');
    await waitFor(() => {
      expect(focusedIn(grid())).toBe('0:cr');
    });
    await user.keyboard('{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(grid())).toBe('1:account');
    });
    await account(user, 1, CASH);
    await user.keyboard(`{Enter}${credit}{Enter}`);
  }

  it('a balanced entry: the totals are the backend’s, Save opens, and F10 saves with the G- number', async () => {
    vi.mocked(cashbookApi.generalPreview).mockResolvedValue(real.GENERAL_BALANCED);
    vi.mocked(cashbookApi.generalSave).mockResolvedValue(real.SAVED_GENERAL);
    const { user } = await open();
    await typeEntry(user, '1000');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'cashbook-general-balanced'));
    });
    const totals = visible(
      screen.getByText(named(message(language, 'cashbook-general-total-dr'))).closest('dl')
        ?.textContent,
    );
    expect(totals).toContain('1,000');
    await waitFor(() => {
      expect(saveButton()).toBeEnabled();
    });
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.generalSave).toHaveBeenCalledTimes(1);
    });
    const toast = message(language, 'cashbook-saved', {
      kind: 'general',
      number: message(language, 'vouchers-number-general', { number: 1 }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
  });

  it('an entry that does not balance: Save stays off, and after the try the backend’s words show with both totals', async () => {
    vi.mocked(cashbookApi.generalPreview).mockResolvedValue(real.GENERAL_UNBALANCED);
    const { user } = await open();
    await typeEntry(user, '900');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'cashbook-general-not-balanced'));
    });
    expect(saveButton()).toBeDisabled();
    press('F10');
    expect(cashbookApi.generalSave).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'err-general-not-balanced', { dr: '1,000', cr: '900' }),
      );
    });
    expect(pageText()).not.toContain('ERR_');
  });

  it('a save the backend refuses as not balanced writes nothing and says why', async () => {
    vi.mocked(cashbookApi.generalPreview).mockResolvedValue(real.GENERAL_BALANCED);
    vi.mocked(cashbookApi.generalSave).mockRejectedValue(
      refused(real.REFUSED_GENERAL_NOT_BALANCED),
    );
    const { user } = await open();
    await typeEntry(user, '1000');
    await waitFor(() => {
      expect(saveButton()).toBeEnabled();
    });
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'err-general-not-balanced', { dr: '1,000', cr: '900' }),
      );
    });
    // The lines stay as typed.
    expect(textOf(grid(), 0, 'dr')).toBe('1,000');
  });

  it('the split of an entry shows the parts, and the working of a part is written with the backend’s message', async () => {
    const split = {
      partner: { id: 1, code: '', name_ur: 'الف', name_en: 'Partner A' },
      share_bps: 4000,
      amount_paisa: paisaForTest(40_000),
      working: {
        message_id: 'working-general-share',
        args: [
          { name: 'bps', value: { kind: 'percent' as const, bps: 4000 } },
          { name: 'result', value: { kind: 'money' as const, paisa: paisaForTest(40_000) } },
        ],
        rule_id: null,
      },
    };
    vi.mocked(cashbookApi.generalPreview).mockResolvedValue({
      ...real.GENERAL_BALANCED,
      split: [split],
    });
    const { user } = await open();
    await typeEntry(user, '1000');
    const card = (
      await screen.findByText(named(message(language, 'cashbook-general-split-title')))
    ).closest('section');
    if (card === null) throw new Error('no split card');
    await waitFor(() => {
      expect(visible(card.textContent)).toContain('400');
    });
    const info = within(card).getAllByRole('button', {
      name: startsWith(message(language, 'calc-info-label')),
    })[0];
    if (info === undefined) throw new Error('no working');
    await user.click(info);
    const working = await screen.findByRole('dialog');
    expect(visible(working.textContent)).toContain('40');
    expect(visible(working.textContent)).toContain('400');
    expect(visible(working.textContent)).not.toContain('working-general-share');
  });

  it('the screen sends the lines the way the backend’s own test did: text, the account, no person for an ordinary account', async () => {
    vi.mocked(cashbookApi.generalPreview).mockResolvedValue(real.GENERAL_BALANCED);
    const { user } = await open();
    await typeEntry(user, '1000');
    await waitFor(() => {
      const last = vi.mocked(cashbookApi.generalPreview).mock.calls.at(-1)?.[0];
      expect(last?.lines).toHaveLength(2);
    });
    const last = vi.mocked(cashbookApi.generalPreview).mock.calls.at(-1)?.[0];
    expect(Object.keys(last ?? {}).sort()).toEqual(
      Object.keys(real.REQUEST_GENERAL_BALANCED).sort(),
    );
    expect(Object.keys(last?.lines[0] ?? {}).sort()).toEqual(
      Object.keys(real.REQUEST_GENERAL_BALANCED.lines[0] ?? {}).sort(),
    );
  });
});
