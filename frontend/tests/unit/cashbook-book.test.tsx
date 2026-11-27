// The روکڑ بک (plan 17, PRD 16.1, FR-16.1, FR-19.10, FR-19.11), in the Urdu and in the English app,
// against a fake backend: the day's cash movements with the opening and closing cash and the
// backend's totals, no box to type counted cash, paging by 25 with the totals over the whole
// range, the four states, the side menu of the area and the language switch.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import type { CashRec } from '../helpers/cashbook-books';
import { labelled, named, pageText, prepareCashbook, startsWith } from '../helpers/cashbook-screen';
import type { FakeCashbook } from '../helpers/cashbook';
import { fail, leaveAppWindow, press, renderApp } from '../helpers/app';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
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

/** The worked day's cash after Events 3 and 4 and the expenses of Event 5 (PRD 4.9). */
const DAY: CashRec[] = [
  {
    date: '2026-10-10',
    type: 'receipt',
    number: 1,
    narration: '',
    inRupees: 10000,
    outRupees: 0,
    running: 60000,
  },
  {
    date: '2026-10-10',
    type: 'payment',
    number: 1,
    narration: '',
    inRupees: 0,
    outRupees: 4316,
    running: 55684,
  },
  {
    date: '2026-10-10',
    type: 'shop_expense',
    number: 1,
    narration: '',
    inRupees: 0,
    outRupees: 3000,
    running: 52684,
  },
  {
    date: '2026-10-10',
    type: 'personal_expense',
    number: 1,
    narration: '',
    inRupees: 0,
    outRupees: 2000,
    running: 50684,
  },
];

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the روکڑ بک (%s)', (language) => {
  let books: FakeCashbook;

  beforeEach(() => {
    books = prepareCashbook(language, [], { cash: 50684, cashRows: DAY });
  });

  async function open(path = '/rokar') {
    const app = await renderApp(path);
    await screen.findByRole('heading', { level: 1, name: message(language, 'cashbook-title') });
    return { ...app, user: userEvent.setup() };
  }

  it("shows the day: the opening and closing cash, every movement and the backend's totals", async () => {
    await open();
    // The book shows 30 days at a time and opens on the latest, the days that end today (PRD FR-19.11).
    expect(
      await screen.findByText(message(language, 'common-days-latest-note')),
    ).toBeInTheDocument();
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-list-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('4,316');
    });
    const text = visible(list.textContent);
    // The kind of each row is written in words, with its number.
    expect(text).toContain(`${message(language, 'cashbook-row-kind-receipt')} 1`);
    // The totals row is the backend's over the whole range: in 10,000, out 4,316 + 3,000 + 2,000.
    expect(text).toContain('10,000');
    expect(text).toContain('9,316');
    // The cash at the start has no card any more (plan 100); the cash at the end has.
    expect(pageText()).toContain(message(language, 'cashbook-closing'));
    expect(pageText()).toContain('50,684');
  });

  it('has no box to type counted cash (PRD FR-16.1)', async () => {
    await open();
    await screen.findByText(message(language, 'common-days-latest-note'));
    // The only box to type in is the one that goes to a day: there is no box for counted cash.
    const boxes = screen.queryAllByRole('textbox');
    expect(boxes).toHaveLength(1);
    expect(boxes[0]).toBe(await labelled(language, 'common-goto-day'));
    expect(pageText()).not.toMatch(/count/i);
  });

  it('asks for a range and pages: 25 rows first, the count from the backend, then the next 25', async () => {
    const many: CashRec[] = Array.from({ length: 150 }, (_, index) => ({
      date: '2026-10-10',
      type: 'receipt',
      number: index + 1,
      narration: '',
      inRupees: 100,
      outRupees: 0,
      running: 1000 + (index + 1) * 100,
    }));
    books.store.cashRows.push(...many);
    const { user } = await open();
    await waitFor(() => {
      expect(cashbookApi.cashbookGet).toHaveBeenCalledWith({
        from: '11-09-2026',
        to: '10-10-2026',
        offset: 0,
        limit: PAGE_ROWS,
        sort: null,
        group_by_month: true,
        show_reversed: false,
      });
    });
    const range = (from: number, to: number) =>
      waitFor(() => {
        expect(visible(pageText())).toContain(rangeText(language, from, to, 154));
      });
    await range(1, 25);
    // The totals do not come from the 25 rows on screen: they are those of all 154.
    await user.click(await pagerButton(language, 'next'));
    await range(26, 50);
    expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: PAGE_ROWS, limit: PAGE_ROWS }),
    );
  });

  it('a day with no cash movement says so and offers the expense screen', async () => {
    books.store.cashRows.length = 0;
    const { user, router } = await open();
    expect(await screen.findByText(message(language, 'cashbook-empty'))).toBeInTheDocument();
    // The header has the button too; the one in the empty message is the last.
    const buttons = screen.getAllByRole('button', {
      name: startsWith(message(language, 'cashbook-new-expense')),
    });
    const last = buttons.at(-1);
    if (last === undefined) throw new Error('no button');
    await user.click(last);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/expense');
    });
  });

  it('a month row heads its lines with the count the backend sent for the whole range', async () => {
    const real = vi.mocked(cashbookApi.cashbookGet).getMockImplementation();
    if (real === undefined) throw new Error('the fake book is not set up');
    vi.mocked(cashbookApi.cashbookGet).mockImplementation(async (input) => ({
      ...(await real(input)),
      months: [{ month: '2026-10', count: 42, figures: [] }],
    }));
    await open();
    await waitFor(() => {
      expect(visible(document.querySelector('.month-heading')?.textContent)).toContain(
        message(language, 'common-month-count', { count: 42 }),
      );
    });
  });

  it('a click on a title sorts by the backend; a sort by anything but the day has no running cash and says so', async () => {
    const real = vi.mocked(cashbookApi.cashbookGet).getMockImplementation();
    if (real === undefined) throw new Error('the fake book is not set up');
    // As the backend: only the book in date order has the cash in hand after each line.
    vi.mocked(cashbookApi.cashbookGet).mockImplementation(async (input) => {
      const page = await real(input);
      return input.sort === null
        ? page
        : { ...page, rows: page.rows.map((row) => ({ ...row, running_paisa: null })) };
    });
    const { user } = await open();
    await waitFor(() => {
      expect(visible(pageText())).toContain('60,000');
    });
    expect(pageText()).not.toContain(message(language, 'cashbook-running-date-only'));
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-list-label')),
    });
    await user.click(await within(list).findByText(named(message(language, 'cashbook-col-in'))));
    await waitFor(() => {
      expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
        expect.objectContaining({ sort: { column: 'in_paisa', direction: 'asc', language } }),
      );
    });
    expect(
      await screen.findByText(message(language, 'cashbook-running-date-only')),
    ).toBeInTheDocument();
    // The cash after each line is not shown for a list in another order.
    await waitFor(() => {
      expect(visible(pageText())).not.toContain('60,000');
    });
  });

  it('a failed load says so with a retry that works', async () => {
    vi.mocked(cashbookApi.cashbookGet).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await renderApp('/rokar');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'cashbook-load-failed'));
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent
      .setup()
      .click(
        within(alert).getByRole('button', { name: startsWith(message(language, 'common-retry')) }),
      );
    expect(
      await screen.findByRole('group', { name: named(message(language, 'cashbook-list-label')) }),
    ).toBeInTheDocument();
  });

  it('the tabs of the area lead to each screen (the voucher tabs are in rokar-tabs.test.tsx)', async () => {
    const { user, router } = await open();
    const menu = screen.getByRole('navigation', {
      name: named(message(language, 'layout-rokar-label')),
    });
    const items = within(menu).getAllByRole('button');
    expect(items.map((item) => visible(item.textContent))).toEqual([
      message(language, 'layout-rokar-cashbook'),
      message(language, 'layout-rokar-receipt'),
      message(language, 'layout-rokar-payment'),
      message(language, 'layout-rokar-advance'),
      message(language, 'layout-rokar-expense'),
      message(language, 'layout-rokar-lent'),
      message(language, 'layout-rokar-khata'),
      message(language, 'layout-rokar-general'),
    ]);
    expect(items[0]).toHaveAttribute('aria-current', 'page');
    for (const [index, path] of [
      [4, '/rokar/expense'],
      [5, '/rokar/lent'],
      [6, '/rokar/khata'],
      [7, '/rokar/general'],
    ] as const) {
      const item = within(
        screen.getByRole('navigation', { name: named(message(language, 'layout-rokar-label')) }),
      ).getAllByRole('button')[index];
      if (item === undefined) throw new Error('no menu item');
      await user.click(item);
      // Each screen of the area is a lazy route, and the path changes once its code has loaded. In a
      // full run on a busy computer the first load took more than 8 s.
      await waitFor(
        () => {
          expect(router.state.location.pathname).toBe(path);
        },
        { timeout: 20_000 },
      );
      await waitFor(
        () => {
          expect(
            within(
              screen.getByRole('navigation', {
                name: named(message(language, 'layout-rokar-label')),
              }),
            ).getAllByRole('button')[index],
          ).toHaveAttribute('aria-current', 'page');
        },
        { timeout: 8000 },
      );
    }
  }, 90_000);

  it('a language switch redraws the book in the other language and keeps the window', async () => {
    const { user } = await open();
    // One window back: the 30 days before the latest.
    await user.click(
      await screen.findByRole('button', {
        name: (name) =>
          visible(name).includes(message(language, 'common-days-earlier', { days: '30' })),
      }),
    );
    await waitFor(() => {
      expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
        expect.objectContaining({ from: '12-08-2026', to: '10-09-2026' }),
      );
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    await waitFor(() => {
      expect(visible(pageText())).toContain('12-08-2026');
    });
    // The 30 days before have no cash movement: the page says so, in the other language.
    expect(await screen.findByText(message(other, 'cashbook-empty'))).toBeInTheDocument();
    // Alt+L is the same switch from the keyboard.
    press('KeyL', { altKey: true });
    expect(
      await screen.findByRole('heading', { level: 1, name: message(language, 'cashbook-title') }),
    ).toBeInTheDocument();
  });
});
