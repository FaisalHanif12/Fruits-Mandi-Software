// The روکڑ بک of PRD FR-16.14 v1.60 (plan 99), in the Urdu and in the English app, against a fake backend: the
// three cards, every row named for what it is with its party, garden or trip (the worked day's rows of the PRD
// example), a buyer's پیشگی named پیشگی, old and cancelled postings hidden until the switch is on, the type filter
// with the backend's kinds, the summary by type, and the one-time fix of old opening balances with its reason, sign
// and cash question. The screen works out no money: every figure shown is the fake backend's.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import type { OpeningsToCashPreview } from '../../src/bindings/OpeningsToCashPreview';
import { money, ref } from '../helpers/accounts';
import type { CashRec } from '../helpers/cashbook-books';
import { choose, named, pageText, prepareCashbook, startsWith } from '../helpers/cashbook-screen';
import { nothingToMove, type FakeCashbook } from '../helpers/cashbook';
import { leaveAppWindow, renderApp } from '../helpers/app';
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

const name = (id: number, code: string, ur: string, en: string) => ref(id, code, ur, en);
const RAHIM = name(10, '0010', 'رحیم', 'Rahim');
const SHAHID = name(30, '0030', 'شاہد', 'Shahid');
const AKRAM = name(20, '0020', 'اکرم', 'Akram');
const MANI = name(24, '0024', 'مانی', 'Mani');
const ANAS = name(25, '0025', 'انس', 'Anas');
const HAROON = name(26, '0026', 'ہارون', 'Haroon');
const BIJLI = name(501, '05010001', 'بجلی', 'Electricity');
const ALIF = name(1, '', 'الف', 'Partner A');
const GARDEN = name(8, '0080', 'پومپ والا باغ', 'Pump Garden');

/** One line of the fake book, with the running cash after it. */
const line = (
  rec: Omit<CashRec, 'date' | 'narration' | 'inRupees' | 'outRupees'> &
    Partial<Pick<CashRec, 'inRupees' | 'outRupees'>>,
): CashRec => ({
  date: '2026-10-10',
  narration: '',
  inRupees: 0,
  outRupees: 0,
  ...rec,
});

/** The worked day's cash as the PRD example names it (FR-16.14): closing 58,484. */
const DAY: CashRec[] = [
  line({
    type: 'payment',
    number: 1,
    kind: 'bikri_seller_cash',
    refs: { parties: [RAHIM], party_count: 1 },
    outRupees: 5000,
    running: 45000,
  }),
  line({
    type: 'receipt',
    number: 2,
    kind: 'bikri_paid',
    refs: { parties: [SHAHID], party_count: 1 },
    inRupees: 12800,
    running: 57800,
  }),
  line({
    type: 'receipt',
    number: 1,
    refs: { parties: [AKRAM], party_count: 1 },
    inRupees: 10000,
    running: 67800,
  }),
  line({
    type: 'payment',
    number: 1,
    refs: { parties: [RAHIM], party_count: 1 },
    outRupees: 4316,
    running: 63484,
  }),
  line({
    type: 'shop_expense',
    number: 1,
    refs: { expense_head: BIJLI },
    outRupees: 3000,
    running: 60484,
  }),
  line({
    type: 'personal_expense',
    number: 1,
    refs: { partner: ALIF },
    outRupees: 2000,
    running: 58484,
  }),
];

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the روکڑ بک of v1.60 (%s)', (language) => {
  let books: FakeCashbook;
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const pick = (ur: string, en: string) => (language === 'ur' ? ur : en);

  beforeEach(() => {
    books = prepareCashbook(language, [], { cash: 58484, cashRows: DAY });
  });

  async function open() {
    const app = await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-title') });
    return { ...app, user: userEvent.setup() };
  }

  const listText = async (waitedFor: string) => {
    const list = await screen.findByRole('group', { name: named(m('cashbook-list-label')) });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain(waitedFor);
    });
    return visible(list.textContent);
  };

  it('has the two cards: capital and cash at the end (the cash at the start is gone, plan 100)', async () => {
    books.store.cashRows.push(
      line({
        type: 'capital_in',
        number: 1,
        refs: { partner: ALIF },
        inRupees: 50000,
        running: 108484,
      }),
    );
    await open();
    await listText('50,000');
    const cards = visible(
      screen.getByText((text) => visible(text) === m('cashbook-card-capital')).closest('dl')
        ?.textContent,
    );
    expect(cards).toContain(m('cashbook-closing'));
    for (const figure of ['50,000', '108,484']) expect(cards).toContain(figure);
  });

  it('names every row with its number and its party, partner or head, as the PRD example does', async () => {
    await open();
    const text = await listText('4,316');
    for (const label of [
      pick('بکری 1، نقد ادائیگی، رحیم', 'Sale 1, Cash Paid, Rahim'),
      pick('بکری 2، موقع پر جمع، شاہد', 'Sale 2, Paid on the Spot, Shahid'),
      pick('وصولی 1، اکرم', 'Receipt 1, Akram'),
      pick('ادائیگی 1، رحیم', 'Payment 1, Rahim'),
      pick('دکان خرچہ 1، بجلی', 'Shop Expense 1, Electricity'),
      pick('ذاتی خرچہ 1، الف', 'Personal Drawing 1, Partner A'),
    ]) {
      expect(text).toContain(label);
    }
    // No message id is left on the screen.
    expect(text).not.toContain('cashbook-row-');
    // No isolate holds only isolates: that would turn an Urdu label left to right (the pieces are joined flat).
    for (const cell of document.querySelectorAll('.ag-row [col-id="kind"]')) {
      expect(cell.textContent).not.toMatch(/\u2068\u2068/);
    }
  });

  it("names a buyer's پیشگی پیشگی, a garden cost with its garden, and the rest of the buyers as more", async () => {
    books.store.cashRows.length = 0;
    books.store.cashRows.push(
      line({
        type: 'payment',
        number: 2,
        kind: 'buyer_advance',
        refs: { parties: [MANI], party_count: 1 },
        outRupees: 50000,
        running: 8484,
      }),
      line({
        type: 'payment',
        number: 1,
        kind: 'garden_cost',
        refs: { garden: GARDEN },
        outRupees: 5000,
        running: 3484,
      }),
      line({
        type: 'receipt',
        number: 3,
        kind: 'bikri_paid',
        refs: { parties: [MANI, ANAS, HAROON], party_count: 5 },
        inRupees: 900,
        running: 4384,
      }),
    );
    await open();
    const text = await listText('50,000');
    expect(text).toContain(pick('پیشگی 2، مانی', 'Advance 2, Mani'));
    expect(text).toContain(pick('باغ خرچ 1، پومپ والا باغ', 'Garden cost 1, Pump Garden'));
    expect(text).toContain(
      pick(
        'بکری 3، موقع پر جمع، مانی، انس، ہارون اور 2 مزید',
        'Sale 3, Paid on the Spot, Mani, Anas, Haroon and 2 more',
      ),
    );
  });

  it('hides old and cancelled postings until the switch is on, and then says what they are', async () => {
    books.store.cashRows.push(
      line({
        type: 'receipt',
        number: 9,
        refs: { parties: [AKRAM], party_count: 1 },
        inRupees: 700,
        running: 59184,
        status: 'reversed',
      }),
      line({
        type: 'receipt',
        number: 9,
        refs: { parties: [AKRAM], party_count: 1 },
        outRupees: 700,
        running: 58484,
        status: 'reversal',
      }),
    );
    const { user } = await open();
    const reversed = `${m('cashbook-row-reversed').trim()} ${pick('وصولی 9، اکرم', 'Receipt 9, Akram')}`;
    expect(await listText('4,316')).not.toContain(reversed);
    expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
      expect.objectContaining({ show_reversed: false }),
    );
    await user.click(screen.getByRole('checkbox', { name: named(m('cashbook-show-reversed')) }));
    await waitFor(() => {
      expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
        expect.objectContaining({ show_reversed: true }),
      );
    });
    const text = await listText(reversed);
    expect(text).toContain(
      `${m('cashbook-row-reversal').trim()} ${pick('وصولی 9، اکرم', 'Receipt 9, Akram')}`,
    );
  });

  it('filters by a type with the kinds the backend names, says the running cash is not shown, and goes back', async () => {
    const { user } = await open();
    await listText('4,316');
    await choose(user, language, 'cashbook-filter-type', m('cashbook-group-expense'));
    await waitFor(() => {
      expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
        expect.objectContaining({
          kinds: ['shop_expense', 'personal_expense', 'personal_return'],
        }),
      );
    });
    expect(
      await screen.findByText((found) => visible(found) === m('cashbook-running-all-types-only')),
    ).toBeInTheDocument();
    const text = await listText(pick('دکان خرچہ 1، بجلی', 'Shop Expense 1, Electricity'));
    expect(text).not.toContain(pick('وصولی 1، اکرم', 'Receipt 1, Akram'));
    // A type with nothing in these days says so, with a button back to every type.
    await choose(user, language, 'cashbook-filter-type', m('cashbook-group-garden'));
    expect(await screen.findByText(m('cashbook-empty-type'))).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: named(m('cashbook-empty-type-action')) }));
    await listText(pick('وصولی 1، اکرم', 'Receipt 1, Akram'));
    const last = vi.mocked(cashbookApi.cashbookGet).mock.calls.at(-1)?.[0];
    expect(last).not.toHaveProperty('kinds');
  });

  it("shows what came in and went out by type from its button, only the types that have entries, with the backend's figures", async () => {
    const { user } = await open();
    await listText('4,316');
    // Shut at first, so the book stays in view.
    expect(screen.queryByRole('table', { name: named(m('cashbook-summary-title')) })).toBeNull();
    const toggle = screen.getByRole('button', { name: named(m('cashbook-summary-title')) });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await user.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    const table = await screen.findByRole('table', { name: named(m('cashbook-summary-title')) });
    const text = visible(table.textContent);
    expect(text).toContain(m('cashbook-group-bikri'));
    expect(text).toContain(m('cashbook-group-expense'));
    expect(text).not.toContain(m('cashbook-group-garden'));
    for (const figure of ['12,800', '5,000', '10,000', '4,316']) expect(text).toContain(figure);
    // A type in the table filters the book.
    await user.click(
      within(table).getByRole('button', { name: named(m('cashbook-group-payment')) }),
    );
    await waitFor(() => {
      expect(cashbookApi.cashbookGet).toHaveBeenLastCalledWith(
        expect.objectContaining({ kinds: ['payment'] }),
      );
    });
    // The summary does not narrow with the filter.
    expect(
      visible(
        (await screen.findByRole('table', { name: named(m('cashbook-summary-title')) }))
          .textContent,
      ),
    ).toContain(m('cashbook-group-bikri'));
  });

  describe('the one-time fix of old opening balances (PRD FR-10.9)', () => {
    /** Plan 99 Section 4, two of the openings: out 30,000, in 10,000. */
    const preview = (needsCash: boolean): OpeningsToCashPreview => ({
      rows: [
        {
          account: MANI,
          kind: 'buyer',
          date: '2026-10-29',
          dr_paisa: money(30000),
          cr_paisa: money(0),
          cash_change_paisa: money(-30000),
          day_closed: true,
        },
        {
          account: HAROON,
          kind: 'seller',
          date: '2026-10-29',
          dr_paisa: money(0),
          cr_paisa: money(10000),
          cash_change_paisa: money(10000),
          day_closed: true,
        },
      ],
      total_dr_paisa: money(30000),
      total_cr_paisa: money(10000),
      cash_change_paisa: money(-20000),
      cash_before_paisa: money(1057840),
      cash_after_paisa: needsCash ? money(-2160) : money(1037840),
      needs_cash_confirm: needsCash,
      opening_difference_before: { amount_paisa: money(20000), side: 'cr' },
      opening_difference_after: { amount_paisa: money(0), side: 'zero' },
      reopens_from: '2026-10-29',
    });

    async function openFix(shown: OpeningsToCashPreview) {
      vi.mocked(cashbookApi.openingsToCashPreview).mockResolvedValue(shown);
      const app = await open();
      await app.user.click(
        await screen.findByRole('button', { name: named(m('cashbook-openings-button')) }),
      );
      await screen.findByRole('table', { name: named(m('cashbook-openings-table')) });
      return app;
    }

    const fill = async (user: ReturnType<typeof userEvent.setup>) => {
      await user.click(screen.getByRole('button', { name: named(m('cashbook-openings-button')) }));
      const box = await screen.findByRole('alertdialog');
      await user.type(
        within(box).getByRole('combobox', { name: named(m('cashbook-openings-reason')) }),
        'old books{Enter}',
      );
      await user.keyboard('adnan{Enter}');
    };

    it('is not offered when there is nothing to move', async () => {
      vi.mocked(cashbookApi.openingsToCashPreview).mockResolvedValue(nothingToMove());
      await open();
      await listText('4,316');
      expect(
        screen.queryByRole('button', { name: named(m('cashbook-openings-button')) }),
      ).toBeNull();
    });

    it('lists each opening with its effect, cash and the opening difference before and after, and saves with the reason and sign', async () => {
      vi.mocked(cashbookApi.openingsToCash).mockResolvedValue({
        rows: [],
        total_dr_paisa: money(30000),
        total_cr_paisa: money(10000),
        cash_after_paisa: money(1037840),
        opening_difference_after: { amount_paisa: money(0), side: 'zero' },
        reopened_from: '2026-10-29',
        closed_up_to: '2026-11-19',
      });
      const { user } = await openFix(preview(false));
      const text = visible(pageText());
      for (const figure of ['30,000', '10,000', '1,057,840', '1,037,840', '29-10-2026']) {
        expect(text).toContain(figure);
      }
      expect(text).toContain(m('cashbook-openings-kind-buyer'));
      await fill(user);
      await waitFor(() => {
        expect(cashbookApi.openingsToCash).toHaveBeenCalledWith({
          reason: 'old books',
          sign: 'adnan',
          confirm_cash_below_zero: false,
        });
      });
      // Back on the book after the save.
      expect(await screen.findByRole('group', { name: named(m('cashbook-list-label')) }));
    });

    it('asks the cash question when cash ends below zero, and sends nothing on a no', async () => {
      vi.mocked(cashbookApi.openingsToCash).mockResolvedValue({
        rows: [],
        total_dr_paisa: money(30000),
        total_cr_paisa: money(10000),
        cash_after_paisa: money(-2160),
        opening_difference_after: { amount_paisa: money(0), side: 'zero' },
        reopened_from: null,
        closed_up_to: null,
      });
      const { user } = await openFix(preview(true));
      await fill(user);
      const ask = await screen.findByRole('alertdialog', {
        name: named(m('common-cash-confirm-title')),
      });
      expect(visible(ask.textContent)).toContain('2,160');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(cashbookApi.openingsToCash).not.toHaveBeenCalled();
      await fill(user);
      await user.click(
        within(
          await screen.findByRole('alertdialog', { name: named(m('common-cash-confirm-title')) }),
        ).getByRole('button', { name: startsWith(m('common-cash-confirm-yes')) }),
      );
      await waitFor(() => {
        expect(cashbookApi.openingsToCash).toHaveBeenCalledWith({
          reason: 'old books',
          sign: 'adnan',
          confirm_cash_below_zero: true,
        });
      });
    });

    it('tells a wrong sign in the box', async () => {
      vi.mocked(cashbookApi.openingsToCash).mockRejectedValue(
        new CommandError({
          ok: false,
          errors: [{ code: 'ERR_SIGN_WRONG', field: 'sign', values: {} }],
          ref: 'E-1',
        }),
      );
      const { user } = await openFix(preview(false));
      await fill(user);
      const box = await screen.findByRole('alertdialog');
      expect(
        await within(box).findByText((text) => visible(text) === m('err-sign-wrong')),
      ).toBeInTheDocument();
    });
  });
});
