// The partner statement, one month at a time (plan 76 point 11, PRD FR-17.12, FR-17.10, FR-19.11, E2E-37,
// D4-A33), in the Urdu and in the English app, by keyboard, against a fake backend: the opening کل رقم of
// the month on top, one row for each day with a movement (commission, expense share, gardens and trips,
// fruit, money taken, capital, the day's net and the کل رقم at the end of the day), the month totals and
// the closing, a day that opens in place with every movement and its source, the month buttons and the box
// that go to another month (kept in the address), and the four states. The screen shows what the backend
// sends and adds nothing up (CLAUDE.md rule 1).

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { choose, named, startsWith } from '../helpers/cashbook-screen';
import { printData } from '../helpers/collection-sheet';
import { fakePartnerMonth, openDays } from '../helpers/partner-month';
import * as real from '../helpers/partner-money-real';
import { DAY, fail, leaveAppWindow, renderApp } from '../helpers/app';
import {
  preparePartnerMoney,
  PARTNER_MONTH_TODAY,
  type PartnerMoney,
} from '../helpers/partner-money';
import { message, visible } from '../helpers/render';

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

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

/** The texts of the cells of every row of the list: the lead row, a row for each day, the totals. */
function rowsOf(list: HTMLElement): string[][] {
  return Array.from(list.querySelectorAll('[role="row"]'))
    .map((row) =>
      Array.from(row.querySelectorAll(':scope > div > [role="gridcell"]')).map((cell) =>
        visible(cell.textContent),
      ),
    )
    .filter((cells) => cells.length > 0);
}

describe.each(LANGUAGES)('the partner statement by month (%s)', (language) => {
  let books: PartnerMoney;

  beforeEach(() => {
    books = preparePartnerMoney(language);
    // The app opens on the last month of the year, so the days of the books (October and November 2026)
    // need a month in the address.
    vi.mocked(appApi.dayStatus).mockResolvedValue({ ...DAY, today: PARTNER_MONTH_TODAY });
    vi.mocked(partnersApi.partnersStatementMonthPrintData).mockImplementation((input) =>
      Promise.resolve(printData(input.language, input.paper ?? 'a4')),
    );
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const list = () => screen.getByRole('treegrid');

  async function open(search = '') {
    const app = await renderApp(`/partners/statement${search}`);
    await screen.findByRole('heading', { level: 1, name: named(m('partners-statement-title')) });
    return { ...app, user: userEvent.setup() };
  }

  it('without a partner it says to choose one, and asks nothing', async () => {
    await open();
    expect(await screen.findByText(named(m('partners-statement-pick')))).toBeInTheDocument();
    expect(partnersApi.partnersStatementMonth).not.toHaveBeenCalled();
  });

  it('E2E-37: partner ب in October has one row for 10-10-26 and the کل رقم ends at 746', async () => {
    await open('?partner=2&month=2026-10');
    await screen.findByRole('treegrid');
    await waitFor(() => {
      expect(rowsOf(list()).length).toBe(3);
    });
    const [lead, day, totals] = rowsOf(list());
    // Opening: the کل رقم before the month (0), under the running column.
    expect(lead?.[0]).toBe(m('partners-month-opening'));
    expect(lead?.[8]).toBe('0');
    // The day: 1,184 + 462 of commission, -900 of expense, net 746 and 746 at the end of the day.
    expect(day).toEqual(['10-10-2026', '1,646', '-900', '', '', '', '', '746', '746']);
    // The totals and the closing are the backend's, under the same columns.
    expect(totals?.[0]).toBe(m('glossary-meezan'));
    expect(totals?.slice(1)).toEqual(['1,646', '-900', '', '', '', '', '746', '746']);
    expect(partnersApi.partnersStatementMonth).toHaveBeenCalledWith({
      partner_id: 2,
      month: '2026-10',
    });
  });

  it('a click on the day opens every movement of it in place, with the کل رقم before and after, and a second click shuts it', async () => {
    const { user } = await open('?partner=2&month=2026-10');
    await screen.findByRole('treegrid');
    const dayRow = await screen.findByRole('row', { name: (name) => name.includes('10-10-2026') });
    await user.click(dayRow);
    const card = await waitFor(() => {
      const found = list().querySelector('[data-partner-day="2026-10-10"]');
      if (!(found instanceof HTMLElement)) throw new Error('the day is not open');
      return found;
    });
    await waitFor(() => {
      expect(within(card).getAllByRole('listitem').length).toBe(3);
    });
    const text = visible(card.textContent);
    // The sentences of the shared statement words, and each amount as the backend sent it.
    expect(text).toContain(m('partners-statement-detail-income-share', { number: 1 }));
    expect(text).toContain(m('partners-statement-detail-income-share', { number: 2 }));
    expect(text).toContain(
      m('partners-statement-detail-expense-share', {
        voucher: m('vouchers-number-shop-expense', { number: 1 }),
        subject: nameOf(language, 'بجلی', 'Electricity'),
      }),
    );
    for (const figure of ['1,184', '462', '-900']) expect(text).toContain(figure);
    // Before the day 0, after it 746.
    expect(text).toContain(m('partners-day-before'));
    expect(text).toContain(m('partners-day-after'));
    expect(text).toContain('746');
    expect(partnersApi.partnersStatementDay).toHaveBeenCalledWith({
      partner_id: 2,
      date: '2026-10-10',
    });
    await user.click(dayRow);
    await waitFor(() => {
      expect(list().querySelector('[data-partner-day]')).toBeNull();
    });
  });

  it('a share line says what it is a share of: his 30% of 3,948 (PRD FR-17.12)', async () => {
    const income = real.STATEMENT_B.lines[0];
    if (income === undefined) throw new Error('no line');
    fakePartnerMonth(
      real.STATEMENT_B.partner,
      [{ ...income, values: { base_paisa: 394_800, share_bps: 3_000 } }],
      PARTNER_MONTH_TODAY,
    );
    const { user } = await open('?partner=2&month=2026-10');
    await screen.findByRole('treegrid');
    const lines = (await openDays(user, ['2026-10-10'])).get('2026-10-10') ?? [];
    expect(lines[0]?.detail).toBe(
      m('partners-statement-detail-income-share-part', { number: 1, share: '30', base: '3,948' }),
    );
    // "Sale 1: ..." already says the group, so no chip repeats it in front.
    expect(lines[0]?.heading).toBe('');
  });

  it('Enter on the day opens it, from the keyboard', async () => {
    const { user } = await open('?partner=2&month=2026-10');
    const grid = await screen.findByRole('treegrid');
    grid.focus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(list().querySelector('[data-partner-day="2026-10-10"]')).not.toBeNull();
    });
  });

  it('a capital line joins the month it was saved in, and the کل رقم goes to 50,746 (E2E-37)', async () => {
    // Partner ب puts in 50,000 on 11-10-26, through the same books the capital screen saves to.
    books.money.saveCapital('capital_in', {
      date: '11-10-2026',
      partner_id: 2,
      amount: '50000',
      narration: null,
      confirm_cash_below_zero: false,
    });
    await open('?partner=2&month=2026-10');
    await screen.findByRole('treegrid');
    await waitFor(() => {
      expect(rowsOf(list()).length).toBe(4);
    });
    const rows = rowsOf(list());
    // 11-10-26: only the capital column and the net, the running total 746 + 50,000.
    expect(rows[2]).toEqual(['11-10-2026', '', '', '', '', '', '50,000', '50,000', '50,746']);
    expect(rows[3]?.[8]).toBe('50,746');
  });

  it('D4-A33: partner الف in October has the trip part, the commission and the trip credit on one day, ending at -296', async () => {
    await open('?partner=1&month=2026-10');
    await screen.findByRole('treegrid');
    await waitFor(() => {
      expect(rowsOf(list()).length).toBe(3);
    });
    const [, day, totals] = rowsOf(list());
    // 10-10-26: commission 480, gardens and trips -4,160 + 3,384 = -776, net -296.
    expect(day).toEqual(['10-10-2026', '480', '', '-776', '', '', '', '-296', '-296']);
    expect(totals?.[8]).toBe('-296');
  });

  it('a day that has only notes (a write-off and a trip closing) has no figures and the کل رقم stays at -296', async () => {
    await open('?partner=1&month=2026-11');
    await screen.findByRole('treegrid');
    await waitFor(() => {
      expect(rowsOf(list()).length).toBe(3);
    });
    const [lead, day] = rowsOf(list());
    // The opening is the -296 carried from October.
    expect(lead?.[8]).toBe('-296');
    expect(day?.[0]).toBe('11-11-2026');
    expect(day?.slice(1, 7)).toEqual(['', '', '', '', '', '']);
    expect(day?.slice(7)).toEqual(['0', '-296']);
  });

  it('the notes of that day show without an amount when the day is opened', async () => {
    const { user } = await open('?partner=1&month=2026-11');
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('row', { name: (name) => name.includes('11-11-2026') }),
    );
    const card = await waitFor(() => {
      const found = list().querySelector('[data-partner-day="2026-11-11"]');
      if (!(found instanceof HTMLElement)) throw new Error('the day is not open');
      return found;
    });
    await waitFor(() => {
      expect(within(card).getAllByRole('listitem').length).toBe(2);
    });
    const text = visible(card.textContent);
    expect(text).toContain('0070');
    expect(text).toContain('10,400');
    expect(text).toContain('-776');
  });

  it('the previous-month button goes back one month, keeps it in the address and asks for it', async () => {
    const { user, router } = await open('?partner=1&month=2026-11');
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('button', { name: startsWith(m('common-month-earlier')) }),
    );
    await waitFor(() => {
      expect(partnersApi.partnersStatementMonth).toHaveBeenLastCalledWith({
        partner_id: 1,
        month: '2026-10',
      });
    });
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ month: '2026-10' });
    });
    // October is the first month with a movement of his: the earlier button says so and is off.
    expect(await screen.findByText(m('common-month-first-note'))).toBeInTheDocument();
  });

  it('the box goes to a typed month, 11-2026', async () => {
    const { user } = await open('?partner=1&month=2026-10');
    await screen.findByRole('treegrid');
    const box = await screen.findByLabelText<HTMLInputElement>(named(m('common-goto-month')));
    await user.type(box, '11-2026{Enter}');
    await waitFor(() => {
      expect(partnersApi.partnersStatementMonth).toHaveBeenLastCalledWith({
        partner_id: 1,
        month: '2026-11',
      });
    });
  });

  it('opens on the current month when the address has none', async () => {
    await open('?partner=2');
    await screen.findByRole('treegrid');
    expect(vi.mocked(partnersApi.partnersStatementMonth).mock.calls[0]?.[0]).toEqual({
      partner_id: 2,
      month: null,
    });
    // The current month, December, has no movement.
    expect(await screen.findByText(named(m('partners-month-empty')))).toBeInTheDocument();
  });

  it('the partner is chosen from the drop-down and the statement follows', async () => {
    const { user } = await open('?month=2026-10');
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'ب', 'Partner B'));
    await waitFor(() => {
      expect(partnersApi.partnersStatementMonth).toHaveBeenCalledWith({
        partner_id: 2,
        month: '2026-10',
      });
    });
    await waitFor(() => {
      expect(rowsOf(list()).length).toBe(3);
    });
  });

  it('a month with no movement says so and the buttons still work', async () => {
    await open('?partner=3&month=2026-10');
    expect(await screen.findByText(named(m('partners-month-empty')))).toBeInTheDocument();
    expect(screen.getByText(m('partners-month-empty-help'))).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: startsWith(m('common-month-earlier')) }),
    ).toBeInTheDocument();
  });

  it('the print button shows the month of the partner on screen in the print layout, and Back returns', async () => {
    const { user } = await open('?partner=2&month=2026-10');
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('report-preview')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(partnersApi.partnersStatementMonthPrintData).mock.calls[0]?.[0]).toEqual({
        partner_id: 2,
        month: '2026-10',
        language,
        paper: 'a4',
      });
    });
    await waitFor(() => {
      expect(visible(document.querySelector('.print-sheet')?.textContent)).toContain('44,364');
    });
    await user.click(
      await screen.findByRole('button', { name: (name) => visible(name) === m('common-back') }),
    );
    expect(await screen.findByRole('treegrid')).toBeInTheDocument();
  });

  it('the month failing to load says so with a retry that works', async () => {
    vi.mocked(partnersApi.partnersStatementMonth).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    const { user } = await open('?partner=2&month=2026-10');
    const title = await screen.findByText(named(m('partners-statement-failed')));
    const alert = title.closest('[role="alert"]');
    if (!(alert instanceof HTMLElement)) throw new Error('no alert');
    await user.click(within(alert).getByRole('button', { name: startsWith(m('common-retry')) }));
    await waitFor(() => {
      expect(rowsOf(list()).length).toBe(3);
    });
  });

  it('a day that cannot be read says so, and the rest of the month stays', async () => {
    vi.mocked(partnersApi.partnersStatementDay).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    const { user } = await open('?partner=2&month=2026-10');
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('row', { name: (name) => name.includes('10-10-2026') }),
    );
    expect(await screen.findByText(named(m('partners-day-failed')))).toBeInTheDocument();
    expect(rowsOf(list()).length).toBe(3);
  });

  it('switching the language rebuilds the list in the other language with the same figures', async () => {
    await open('?partner=1&month=2026-10');
    await screen.findByRole('treegrid');
    await waitFor(() => {
      expect(rowsOf(list()).length).toBe(3);
    });
    const before = rowsOf(list()).map((cells) => cells.slice(1));
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    await waitFor(() => {
      expect(visible(list().textContent)).toContain(message(other, 'partners-month-opening'));
    });
    expect(rowsOf(list()).map((cells) => cells.slice(1))).toEqual(before);
  });
});
