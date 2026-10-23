// The روکڑ بک (plan 17, PRD 16.1, FR-16.1, FR-19.10, FR-19.11), in the Urdu and in the English app,
// against a fake backend: the day's cash movements with the opening and closing cash and the
// backend's totals, no box to type counted cash, paging by 100 with the totals over the whole
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
    expect((await labelled(language, 'cashbook-filter-from')).value).toBe('10-10-2026');
    expect((await labelled(language, 'cashbook-filter-to')).value).toBe('10-10-2026');
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-list-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('4,316');
    });
    const text = visible(list.textContent);
    // The kind of each row is written in words, with its number.
    expect(text).toContain(
      message(language, 'cashbook-kind-number', {
        kind: message(language, 'cashbook-kind-receipt'),
        number: 1,
      }),
    );
    // The totals row is the backend's over the whole range: in 10,000, out 4,316 + 3,000 + 2,000.
    expect(text).toContain('10,000');
    expect(text).toContain('9,316');
    expect(pageText()).toContain(message(language, 'cashbook-opening'));
    expect(pageText()).toContain('50,000');
    expect(pageText()).toContain('50,684');
  });

  it('has no box to type counted cash (PRD FR-16.1)', async () => {
    await open();
    const inputs = screen.getAllByRole('textbox');
    // The two dates, and nothing else to type in.
    expect(inputs).toHaveLength(2);
    expect(pageText()).not.toMatch(/count/i);
  });

  it('asks for a range and pages: 100 rows first, the count from the backend, then the rest', async () => {
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
        from: '10-10-2026',
        to: '10-10-2026',
        offset: 0,
        limit: 100,
      });
    });
    const count = (shown: number) =>
      screen.findByText(
        (content) =>
          visible(content) === message(language, 'cashbook-list-count', { shown, total: 154 }),
      );
    expect(await count(100)).toBeInTheDocument();
    // The totals do not come from the 100 rows on screen: they are those of all 154.
    await user.click(
      screen.getByRole('button', { name: startsWith(message(language, 'cashbook-list-more')) }),
    );
    expect(await count(154)).toBeInTheDocument();
    expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: 100, limit: 100 }),
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

  it('the side menu of the area leads to each screen and to the voucher screen', async () => {
    const { user, router } = await open();
    const menu = screen.getByRole('navigation', {
      name: named(message(language, 'layout-rokar-label')),
    });
    const items = within(menu).getAllByRole('button');
    expect(items.map((item) => visible(item.textContent))).toEqual([
      message(language, 'layout-rokar-cashbook'),
      message(language, 'layout-rokar-expense'),
      message(language, 'layout-rokar-lent'),
      message(language, 'layout-rokar-khata'),
      message(language, 'layout-rokar-general'),
      message(language, 'layout-rokar-voucher'),
    ]);
    expect(items[0]).toHaveAttribute('aria-current', 'page');
    for (const [index, path] of [
      [1, '/rokar/expense'],
      [2, '/rokar/lent'],
      [3, '/rokar/khata'],
      [4, '/rokar/general'],
      [5, '/bikri/voucher'],
    ] as const) {
      const item = within(
        screen.getByRole('navigation', { name: named(message(language, 'layout-rokar-label')) }),
      ).getAllByRole('button')[index];
      if (item === undefined) throw new Error('no menu item');
      await user.click(item);
      await waitFor(() => {
        expect(router.state.location.pathname).toBe(path);
      });
      if (index < 5) {
        await waitFor(() => {
          expect(
            within(
              screen.getByRole('navigation', {
                name: named(message(language, 'layout-rokar-label')),
              }),
            ).getAllByRole('button')[index],
          ).toHaveAttribute('aria-current', 'page');
        });
      }
    }
  });

  it('a language switch redraws the book in the other language and keeps the range', async () => {
    const { user } = await open();
    const from = await labelled(language, 'cashbook-filter-from');
    await user.clear(from);
    await user.keyboard('01-10-2026');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'cashbook-filter-from')).value).toBe('01-10-2026');
    expect(
      await screen.findByRole('group', { name: named(message(other, 'cashbook-list-label')) }),
    ).toBeInTheDocument();
    expect(screen.queryByLabelText(named(message(language, 'cashbook-filter-from')))).toBeNull();
    // Alt+L is the same switch from the keyboard.
    press('KeyL', { altKey: true });
    expect(
      await screen.findByRole('heading', { level: 1, name: message(language, 'cashbook-title') }),
    ).toBeInTheDocument();
  });
});
