// The partner statement (plan 17 F2, PRD FR-17.9, FR-17.10, FR-19.11, E2E-37, D4-A33), in the Urdu
// and in the English app, by keyboard, against a fake backend: one dated line for every amount that
// reaches a partner, the heading and the sentence from Fluent, the signed amount and the running
// کل رقم, note lines without an amount, the totals over the whole range, paging, and the four
// states. The screen shows what the backend sends and adds nothing up (CLAUDE.md rule 1).

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { cellOf, gridOf, textOf } from '../helpers/bikri-screen';
import { choose, named, pageText, startsWith } from '../helpers/cashbook-screen';
import { fail, leaveAppWindow, renderApp } from '../helpers/app';
import { preparePartnerMoney, type PartnerMoney } from '../helpers/partner-money';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
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

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the partner statement (%s)', (language) => {
  let books: PartnerMoney;

  beforeEach(() => {
    books = preparePartnerMoney(language);
  });

  const grid = () => gridOf(language, 'partners-statement-label');
  const rows = () => grid().querySelectorAll('.ag-row[row-index]').length;

  async function open(search = '') {
    const app = await renderApp(`/partners/statement${search}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-statement-title')),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('without a partner it says to choose one, and asks nothing', async () => {
    await open();
    expect(
      await screen.findByText(named(message(language, 'partners-statement-pick'))),
    ).toBeInTheDocument();
    expect(partnersApi.partnersStatement).not.toHaveBeenCalled();
  });

  it('E2E-37: partner ب has one dated line for each amount, with the running total ending at 746', async () => {
    await open('?partner=2');
    await waitFor(() => {
      expect(rows()).toBe(3);
    });
    // 10-10-26 بکری 1 share 1,184, بکری 2 share 462, the electricity share -900: 1,184, 1,646, 746.
    expect([0, 1, 2].map((row) => textOf(grid(), row, 'running'))).toEqual([
      '1,184',
      '1,646',
      '746',
    ]);
    expect([0, 1, 2].map((row) => textOf(grid(), row, 'amount'))).toEqual(['1,184', '462', '-900']);
    expect(textOf(grid(), 0, 'date')).toBe('10-10-2026');
    expect(textOf(grid(), 0, 'detail')).toBe(
      message(language, 'partners-statement-detail-income-share', { number: 1 }),
    );
    expect(textOf(grid(), 2, 'detail')).toBe(
      message(language, 'partners-statement-detail-expense-share', {
        voucher: message(language, 'vouchers-number-shop-expense', { number: 1 }),
        subject: nameOf(language, 'بجلی', 'Electricity'),
      }),
    );
    // The totals above the list are the backend's over the whole range.
    const totals = visible(
      screen.getByText(named(message(language, 'partners-statement-opening'))).closest('dl')
        ?.textContent,
    );
    expect(totals).toContain('1,646');
    expect(totals).toContain('900');
    expect(totals).toContain('746');
  });

  it('a capital line joins the statement once saved, and the total goes to 50,746 (E2E-37)', async () => {
    // Partner ب puts in 50,000 on 11-10-26, through the same books the capital screen saves to.
    books.money.saveCapital('capital_in', {
      date: '11-10-2026',
      partner_id: 2,
      amount: '50000',
      narration: null,
      confirm_cash_below_zero: false,
    });
    await open('?partner=2');
    await waitFor(() => {
      expect(rows()).toBe(4);
    });
    expect(textOf(grid(), 3, 'running')).toBe('50,746');
    expect(textOf(grid(), 3, 'detail')).toBe(
      message(language, 'partners-statement-detail-capital-in', {
        voucher: message(language, 'vouchers-number-capital-in', { number: 1 }),
      }),
    );
  });

  it('D4-A33: partner الف has -4,160, +480 and +3,384 ending at -296, then two note lines without an amount', async () => {
    await open('?partner=1');
    await waitFor(() => {
      expect(rows()).toBe(5);
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
    // The trip line names the trip, the purchase and the money from the shop (PRD FR-17.10).
    expect(textOf(grid(), 0, 'detail')).toBe(
      message(language, 'partners-statement-detail-trip-paid', {
        kind: 'purchase',
        trip: '0070',
        secondary: message(language, 'purchases-number', { number: 1 }),
      }),
    );
    // The sale line of a trip carries the figures of the backend: 8,460, 7,800, 660 and 40%.
    const credit = textOf(grid(), 2, 'detail');
    for (const figure of ['8,460', '7,800', '660', '40']) expect(credit).toContain(figure);
    // The notes: no amount, and the running total stays -296.
    for (const row of [3, 4]) {
      expect(textOf(grid(), row, 'amount')).toBe('');
      expect(textOf(grid(), row, 'running')).toBe('-296');
    }
    expect(textOf(grid(), 3, 'detail')).toContain('0070');
    const closed = textOf(grid(), 4, 'detail');
    expect(closed).toContain('10,400');
    expect(closed).toContain('-776');
  });

  it('a date range limits the lines and the opening total is the balance before it', async () => {
    const { user } = await open('?partner=2');
    await waitFor(() => {
      expect(rows()).toBe(3);
    });
    const from = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'cashbook-filter-from')),
    );
    await user.click(from);
    await user.keyboard('11-10-2026');
    await waitFor(() => {
      expect(partnersApi.partnersStatement).toHaveBeenLastCalledWith(
        expect.objectContaining({ partner_id: 2, from: '11-10-2026' }),
      );
    });
    expect(
      await screen.findByText(named(message(language, 'partners-statement-empty'))),
    ).toBeInTheDocument();
    // The opening total is the 746 of the days before; no line is drawn.
    const totals = visible(
      screen.getByText(named(message(language, 'partners-statement-opening'))).closest('dl')
        ?.textContent,
    );
    expect(totals).toContain('746');
  });

  it('the partner is chosen from the drop-down and the statement follows', async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'ب', 'Partner B'));
    await waitFor(() => {
      expect(rows()).toBe(3);
    });
    expect(partnersApi.partnersStatement).toHaveBeenCalledWith(
      expect.objectContaining({ partner_id: 2, offset: 0 }),
    );
  });

  it('the statement failing to load says so with a retry that works', async () => {
    vi.mocked(partnersApi.partnersStatement).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    const { user } = await open('?partner=2');
    const title = await screen.findByText(named(message(language, 'partners-statement-failed')));
    const alert = title.closest('[role="alert"]');
    if (!(alert instanceof HTMLElement)) throw new Error('no alert');
    await user.click(
      within(alert).getByRole('button', { name: startsWith(message(language, 'common-retry')) }),
    );
    await waitFor(() => {
      expect(rows()).toBe(3);
    });
  });

  it('a partner with no lines gets a helpful empty message', async () => {
    await open('?partner=3');
    expect(
      await screen.findByText(named(message(language, 'partners-statement-empty'))),
    ).toBeInTheDocument();
    expect(pageText()).toContain(message(language, 'partners-statement-empty-help'));
  });

  it('switching the language rebuilds the list in the other language with the same figures', async () => {
    await open('?partner=1');
    await waitFor(() => {
      expect(rows()).toBe(5);
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const rebuilt = gridOf(other, 'partners-statement-label');
    await waitFor(() => {
      expect(textOf(rebuilt, 2, 'running')).toBe('-296');
    });
    expect(textOf(rebuilt, 0, 'heading')).toBe(message(other, 'partners-statement-heading-trip'));
    expect(cellOf(rebuilt, 0, 'detail')).toBeInTheDocument();
  });

  it("pages 25 lines at a time and the count is the backend's over the whole range", async () => {
    const full = books.money.statement({
      partner_id: 2,
      from: null,
      to: null,
      offset: 0,
      limit: 0,
      sort: null,
      group_by_month: false,
    });
    // Thirty lines in all: the three real ones again and again, each with its own id.
    const lines = Array.from({ length: 30 }, (_, index) => {
      const base = full.lines[index % full.lines.length];
      return base === undefined ? [] : [{ ...base, id: `line-${String(index)}` }];
    }).flat();
    vi.mocked(partnersApi.partnersStatement).mockImplementation((input) =>
      Promise.resolve({
        ...full,
        lines: lines.slice(input.offset, input.offset + input.limit),
        total_count: 30,
      }),
    );
    const { user } = await open('?partner=2');
    await waitFor(() => {
      expect(visible(pageText())).toContain(rangeText(language, 1, 25, 30));
    });
    expect(partnersApi.partnersStatement).toHaveBeenCalledWith(
      expect.objectContaining({ offset: 0, limit: PAGE_ROWS }),
    );
    await user.click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(partnersApi.partnersStatement).toHaveBeenLastCalledWith(
        expect.objectContaining({ offset: PAGE_ROWS, limit: PAGE_ROWS }),
      );
    });
    await waitFor(() => {
      expect(visible(pageText())).toContain(rangeText(language, 26, 30, 30));
    });
  });
});
