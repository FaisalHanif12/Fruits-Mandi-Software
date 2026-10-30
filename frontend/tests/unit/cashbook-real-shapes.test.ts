// The canned data of the cash book tests has the shape of the backend's real answers (plan 17, T4).
// The backend wrote each real answer in its own test (tests/fixtures/*.json, copied, never edited).
// This compares the keys and kinds at every level, so a test that passes on canned data cannot hide
// a field the real backend does not send, or one it sends and the canned data forgot. Then it checks
// that the screen builds the request the backend's own test sent for the same entry.

import { describe, expect, it } from 'vitest';
import { buildExpense, newExpenseDraft } from '../../src/features/cashbook/expense-draft';
import { inputOf } from '../../src/features/cashbook/entry-support';
import { buildLent, newLentDraft } from '../../src/features/cashbook/lent-draft';
import { CommandError } from '../../src/shared/api';
import { ASLAM, HEADS, createCashbookBooks } from '../helpers/cashbook-books';
import * as real from '../helpers/cashbook-real';
import { same, sendsTheSame } from '../helpers/shapes';

describe('the comparison itself', () => {
  it('sees a missing field, an extra field and a different kind', () => {
    expect(same({ a: 1, b: 2 }, { a: 1 })).toContain('.b');
    expect(same({ a: 1 }, { a: 1, c: 2 })).toContain('.c');
    expect(same({ a: 1 }, { a: 'x' })).toContain('.a');
    expect(same({ a: null }, { a: 1 })).toBeNull();
  });

  it('sees a sent value that differs, and an extra value that is not a default', () => {
    expect(sendsTheSame({ a: 1, b: null }, { a: 1 })).toBeNull();
    expect(sendsTheSame({ a: 2 }, { a: 1 })).toContain('a');
    expect(sendsTheSame({ a: 1, b: 'x' }, { a: 1 })).toContain('b');
  });
});

/** The refusal a fake backend throws, as the plain `{ ok, errors, ref }` of the wire. */
function refusalOf(make: () => unknown) {
  try {
    make();
  } catch (error) {
    if (error instanceof CommandError) return error.uiError;
    throw error;
  }
  throw new Error('nothing was refused');
}

const shopDraft = () => ({
  ...newExpenseDraft('shop_expense', '2026-10-10'),
  headId: '1',
  amount: '3,000',
  paidTo: 'Partner A',
});
const personalDraft = () => ({
  ...newExpenseDraft('personal_expense', '2026-10-10'),
  partnerId: '1',
  amount: '2,000',
  reason: 'گھر کا خرچ',
  returnBy: '20-10-2026',
});

describe('the canned data of the cash book tests has the real shape', () => {
  it('the cash book: a page with its opening, closing and totals', () => {
    const books = createCashbookBooks({
      cash: 5000,
      cashRows: Array.from({ length: 6 }, (_, index) => ({
        date: '2026-10-10',
        type: 'receipt' as const,
        number: index + 1,
        narration: '',
        inRupees: 100,
        outRupees: 0,
        running: 4100 + index * 100,
      })),
    });
    const input = {
      from: '10-10-2026',
      to: '10-10-2026',
      offset: 0,
      limit: 5,
      sort: null,
      group_by_month: false,
    };
    expect(same(real.CASHBOOK_DAY, books.cashbook(input))).toBeNull();
    expect(same(real.CASHBOOK_SECOND_PAGE, books.cashbook({ ...input, offset: 5 }))).toBeNull();
    // The rows have the account the money came from or went to, or nothing.
    expect(real.CASHBOOK_DAY.rows[0]?.account?.code).toBe('06019999');
  });

  it('the heads, the previews, the saves and the refusals of the خرچہ screen', () => {
    const books = createCashbookBooks({ cash: 50_000 });
    expect(same(real.EXPENSE_HEADS, books.heads())).toBeNull();
    expect(
      same(real.PREVIEW_SHOP, books.expensePreview(buildExpense(shopDraft(), false))),
    ).toBeNull();
    expect(
      same(real.PREVIEW_PERSONAL, books.expensePreview(buildExpense(personalDraft(), false))),
    ).toBeNull();
    // The real previews list shares only for a shop expense, and a partner effect only for the rest.
    expect(real.PREVIEW_SHOP.shares).toHaveLength(3);
    expect(real.PREVIEW_SHOP.partner_effect).toBeNull();
    expect(real.PREVIEW_PERSONAL.shares).toEqual([]);
    expect(real.PREVIEW_PERSONAL.others_not_affected).toBe(true);
    expect(same(real.SAVED_SHOP, books.saveExpense(buildExpense(shopDraft(), false)))).toBeNull();
    expect(
      same(real.SAVED_PERSONAL, books.saveExpense(buildExpense(personalDraft(), false))),
    ).toBeNull();
    const refusedMistakes = refusalOf(() =>
      books.saveExpense(
        buildExpense({ ...newExpenseDraft('shop_expense', '2026-10-10'), amount: '100.50' }, false),
      ),
    );
    expect(same(real.REFUSED_SHOP_MISTAKES, refusedMistakes)).toBeNull();
  });

  it('a partner’s money taken, the proposals of last month', () => {
    const books = createCashbookBooks({ cash: 50_000 });
    books.saveExpense(buildExpense(personalDraft(), false));
    expect(
      same(
        real.PARTNER_TAKES,
        books.partnerTakes({
          partner_id: 1,
          offset: 0,
          limit: 0,
          sort: null,
          group_by_month: false,
          show_finished: false,
        }),
      ),
    ).toBeNull();
    const electricity = HEADS[0];
    if (electricity === undefined) throw new Error('no head');
    books.store.repeat.push({
      head: electricity,
      amount: 2800,
      paid_to: 'WAPDA',
      narration: 'bill',
      last_date: '2026-09-05',
    });
    expect(same(real.REPEAT_LAST_MONTH, books.repeat('2026-10'))).toBeNull();
  });

  it('a person who borrows: the loans, the saves and the new person', () => {
    const books = createCashbookBooks({ cash: 50_000 });
    const lent = buildLent(
      {
        ...newLentDraft('money_lent', '2026-10-10'),
        person: { ...ASLAM },
        amount: '5,000',
        reason: 'x',
      },
      false,
    );
    expect(same(real.SAVED_LENT, books.saveLentOrKhata(lent))).toBeNull();
    expect(
      same(
        real.LENT_LIST,
        books.lentList({
          person_id: ASLAM.id,
          offset: 0,
          limit: 0,
          sort: null,
          group_by_month: false,
          show_finished: false,
        }),
      ),
    ).toBeNull();
    expect(
      same(
        real.LENT_PERSON,
        books.createPerson({
          name_ur: 'نیا',
          name_en: 'New',
          phone: null,
          staff_id: null,
          confirmed_rules_words: [],
        }),
      ),
    ).toBeNull();
  });
});

describe('the screen builds the request the backend’s own test sent', () => {
  it('a دکان خرچہ: save and preview', () => {
    const built = buildExpense(shopDraft(), false);
    expect(sendsTheSame(inputOf(built), real.REQUEST_SAVE_SHOP)).toBeNull();
    expect(sendsTheSame(built, real.REQUEST_PREVIEW_SHOP.entry)).toBeNull();
    expect(real.REQUEST_PREVIEW_SHOP.replacing_voucher_id).toBeNull();
  });

  it('a mistake is sent as typed: the decimals and the missing head reach the backend', () => {
    const built = buildExpense(
      { ...newExpenseDraft('shop_expense', '2026-10-10'), amount: '100.50' },
      false,
    );
    expect(sendsTheSame(inputOf(built), real.REQUEST_SAVE_SHOP_MISTAKES)).toBeNull();
  });

  it('a ذاتی خرچہ with a return-by day, and the answer of the cash question', () => {
    const built = buildExpense(personalDraft(), false);
    expect(sendsTheSame(inputOf(built), real.REQUEST_SAVE_PERSONAL)).toBeNull();
    // The backend's preview request leaves the return-by day out.
    const previewed = buildExpense({ ...personalDraft(), returnBy: '' }, false);
    expect(sendsTheSame(previewed, real.REQUEST_PREVIEW_PERSONAL.entry)).toBeNull();
    const big = buildExpense(
      {
        ...newExpenseDraft('personal_expense', '2026-10-10'),
        partnerId: '2',
        amount: '60,000',
        reason: 'plot',
      },
      false,
    );
    expect(sendsTheSame(inputOf(big), real.REQUEST_SAVE_PERSONAL_CASH_CONFIRM)).toBeNull();
    expect(
      inputOf(
        buildExpense(
          {
            ...newExpenseDraft('personal_expense', '2026-10-10'),
            partnerId: '2',
            amount: '60,000',
            reason: 'plot',
          },
          true,
        ),
      ),
    ).toMatchObject({ confirm_cash_below_zero: true });
  });

  it('a ذاتی خرچہ واپسی', () => {
    const built = buildExpense(
      {
        ...newExpenseDraft('personal_return', '2026-10-10'),
        partnerId: '1',
        amount: '500',
        narration: 'first part',
      },
      false,
    );
    expect(sendsTheSame(inputOf(built), real.REQUEST_SAVE_PERSONAL_RETURN)).toBeNull();
    const exceeds = buildExpense(
      { ...newExpenseDraft('personal_return', '2026-10-10'), partnerId: '1', amount: '2,500' },
      false,
    );
    expect(sendsTheSame(inputOf(exceeds), real.REQUEST_SAVE_PERSONAL_RETURN_EXCEEDS)).toBeNull();
  });

  it('money lent, and its return in cash or from the salary', () => {
    const person = { ...ASLAM, id: 33 };
    const lent = buildLent(
      {
        ...newLentDraft('money_lent', '2026-10-10'),
        person,
        amount: '5,000',
        reason: 'گھریلو ضرورت',
        returnBy: '15-10-2026',
      },
      false,
    );
    expect(sendsTheSame(inputOf(lent), real.REQUEST_SAVE_LENT)).toBeNull();
    const early = buildLent(
      {
        ...newLentDraft('money_lent', '2026-10-10'),
        person,
        amount: '5,000',
        reason: 'گھریلو ضرورت',
        returnBy: '05-10-2026',
      },
      false,
    );
    expect(sendsTheSame(inputOf(early), real.REQUEST_SAVE_LENT_BEFORE_ENTRY)).toBeNull();
    const cash = buildLent(
      { ...newLentDraft('money_lent_return', '2026-10-11'), person, amount: '3,000' },
      false,
    );
    expect(sendsTheSame(inputOf(cash), real.REQUEST_SAVE_LENT_RETURN_CASH)).toBeNull();
    const salary = buildLent(
      {
        ...newLentDraft('money_lent_return', '2026-10-12'),
        person,
        amount: '2,000',
        mode: 'salary',
        salaryHeadId: '2',
      },
      false,
    );
    expect(sendsTheSame(inputOf(salary), real.REQUEST_SAVE_LENT_RETURN_SALARY)).toBeNull();
  });

  it('the lists ask for a person or partner and a page, nothing else', () => {
    expect(Object.keys(real.REQUEST_PARTNER_TAKES).sort()).toEqual([
      'limit',
      'offset',
      'partner_id',
    ]);
    expect(Object.keys(real.REQUEST_LENT_LIST).sort()).toEqual(['limit', 'offset', 'person_id']);
    expect(Object.keys(real.REQUEST_CASHBOOK_DAY).sort()).toEqual([
      'from',
      'limit',
      'offset',
      'to',
    ]);
    expect(real.REQUEST_REPEAT_LAST_MONTH).toEqual({ month: '2026-10' });
  });
});
