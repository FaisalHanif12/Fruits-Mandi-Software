// Test helper: an in-memory fake of the commands of the روکڑ area (plan 17 contract as built in T1).
// The wrappers are mocked in the test file with vi.mock; this sets their answers from the books of
// cashbook-books.ts, the way a healthy backend answers (the same books the development page
// /dev/rokar uses). The sums there are a test double's: the screens only show what comes back and
// never work out an amount (CLAUDE.md rule 1).

import { vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import type { ChartHead } from '../../src/shared/accounts';
import {
  ASLAM,
  FAMILY_KHATA,
  createCashbookBooks,
  type CashbookBooksOptions,
  type CashbookFakeBooks,
} from './cashbook-books';
import { account, balance, sampleTree, ZERO } from './accounts';

export { ASLAM, FAMILY_KHATA };
export type FakeCashbook = CashbookFakeBooks;

/** The person who can borrow, and the khata, as accounts of the chart. */
export const ASLAM_ROW = account(ASLAM.id, 'money_lent', ASLAM.code, ASLAM.name_ur, ASLAM.name_en, {
  balance: balance(0, 'dr'),
});
export const KHATA_ROW = account(
  FAMILY_KHATA.id,
  'other',
  FAMILY_KHATA.code,
  FAMILY_KHATA.name_ur,
  FAMILY_KHATA.name_en,
  { balance: ZERO },
);

/** The chart of the sample tree with the controls 0103 (money lent) and 0305 (separate khatas). */
export function cashbookTree(): ChartHead[] {
  const tree = sampleTree();
  tree[0]?.controls.push({
    id: 103,
    code: '0103',
    name_ur: 'ادھار رقم',
    name_en: 'Money Lent',
    party_type: 'other',
    balance: ZERO,
    account_count: 0,
  });
  tree[1]?.controls.push({
    id: 305,
    code: '0305',
    name_ur: 'الگ کھاتے',
    name_en: 'Separate Khatas',
    party_type: 'separate',
    balance: ZERO,
    account_count: 0,
  });
  return tree;
}

/** A rejection the way `invoke` rejects: with the refusal itself. */
const answer = <T>(make: () => T): Promise<T> =>
  new Promise<T>((resolve) => {
    resolve(make());
  });

/** Sets the answers of the cash book commands from fresh books, and returns the books. */
export function fakeCashbook(options: CashbookBooksOptions = {}): FakeCashbook {
  const books = createCashbookBooks(options);
  vi.mocked(cashbookApi.cashbookGet).mockImplementation((input) =>
    answer(() => books.cashbook(input)),
  );
  vi.mocked(cashbookApi.expensePreview).mockImplementation((entry) =>
    answer(() => books.expensePreview(entry)),
  );
  vi.mocked(cashbookApi.saveShopExpense).mockImplementation((input) =>
    answer(() => books.saveExpense({ type: 'shop_expense', ...input })),
  );
  vi.mocked(cashbookApi.savePersonalExpense).mockImplementation((input) =>
    answer(() => books.saveExpense({ type: 'personal_expense', ...input })),
  );
  vi.mocked(cashbookApi.savePersonalReturn).mockImplementation((input) =>
    answer(() => books.saveExpense({ type: 'personal_return', ...input })),
  );
  vi.mocked(cashbookApi.partnerTakes).mockImplementation((input) =>
    answer(() => books.partnerTakes(input)),
  );
  vi.mocked(cashbookApi.repeatLastMonth).mockImplementation((input) =>
    answer(() => books.repeat(input.month)),
  );
  vi.mocked(cashbookApi.saveMoneyLent).mockImplementation((input) =>
    answer(() => books.saveLentOrKhata({ type: 'money_lent', ...input })),
  );
  vi.mocked(cashbookApi.saveMoneyLentReturn).mockImplementation((input) =>
    answer(() => books.saveLentOrKhata({ type: 'money_lent_return', ...input })),
  );
  vi.mocked(cashbookApi.lentPersonCreate).mockImplementation((input) =>
    answer(() => books.createPerson(input)),
  );
  vi.mocked(cashbookApi.lentList).mockImplementation((input) =>
    answer(() => books.lentList(input)),
  );
  vi.mocked(cashbookApi.khataCreate).mockImplementation((input) =>
    answer(() => books.createKhata(input)),
  );
  vi.mocked(cashbookApi.saveKhataOut).mockImplementation((input) =>
    answer(() => books.saveLentOrKhata({ type: 'khata_out', ...input })),
  );
  vi.mocked(cashbookApi.saveKhataIn).mockImplementation((input) =>
    answer(() => books.saveLentOrKhata({ type: 'khata_in', ...input })),
  );
  vi.mocked(cashbookApi.khataStatement).mockImplementation((input) =>
    answer(() => books.khataStatement(input)),
  );
  vi.mocked(vouchersApi.vouchersPreview).mockImplementation((entry) =>
    answer(() => books.entryPreview(entry)),
  );
  vi.mocked(vouchersApi.vouchersList).mockImplementation((input) =>
    answer(() => books.vouchersList(input)),
  );
  return books;
}
