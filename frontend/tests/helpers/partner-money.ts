// Test helper: an in-memory fake of the commands of the second half of plan 17 (F2): the جنرل entry,
// partner capital and the partner statement, on top of the cash book fakes (cashbook.ts). The
// wrappers are mocked in the test file with vi.mock; this sets their answers from the books of
// partner-money-books.ts, the way a healthy backend answers. The sums there are a test double's:
// the screens only show what comes back and never work out an amount (CLAUDE.md rule 1).
//
// The test file needs the `vi.mock` lines of cashbook-screen.ts, and also
//   vi.mock('../../src/features/partners/api');  (already listed there)

import { vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as partnersApi from '../../src/features/partners/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import type { Language } from '../../src/i18n';
import { account, balance, type account as AccountFn } from './accounts';
import type { CashbookBooksOptions } from './cashbook-books';
import { prepareCashbook } from './cashbook-screen';
import type { FakeCashbook } from './cashbook';
import {
  GENERAL_ACCOUNTS,
  SHAHID,
  createPartnerMoneyBooks,
  type PartnerMoneyBooks,
} from './partner-money-books';
import { fakePartners, overviewRow, THREE_PARTNERS } from './partners';
import { AKRAM, PUKHTA_BUYER, RAHIM } from './vouchers';
import { money } from './accounts';

type AccountRow = ReturnType<typeof AccountFn>;

/** The accounts the جنرل screen can pick, as rows of the chart (the picker reads these). */
export const GENERAL_ROWS: AccountRow[] = [
  account(1, 'other', '01010001', 'روکڑ', 'Cash', { balance: balance(53484, 'dr') }),
  AKRAM,
  RAHIM,
  PUKHTA_BUYER,
  account(501, 'other', '05010001', 'بجلی', 'Electricity'),
  account(401, 'other', '04010001', 'کمیشن آمدن', 'Commission Income'),
];

export interface PartnerMoney {
  cash: FakeCashbook;
  money: PartnerMoneyBooks;
  partners: ReturnType<typeof fakePartners>;
}

/** Sets every fake an app test of the second half of plan 17 needs, and returns the books. */
export function preparePartnerMoney(
  language: Language,
  options: CashbookBooksOptions = {},
): PartnerMoney {
  const cash = prepareCashbook(language, GENERAL_ROWS, options);
  const books = createPartnerMoneyBooks(cash, { accounts: GENERAL_ACCOUNTS });
  const answer = <T>(make: () => T): Promise<T> =>
    new Promise<T>((resolve) => {
      resolve(make());
    });
  vi.mocked(cashbookApi.generalPreview).mockImplementation((input) =>
    answer(() => books.generalPreview(input)),
  );
  vi.mocked(cashbookApi.generalSave).mockImplementation((input) =>
    answer(() => books.generalSave(input)),
  );
  vi.mocked(partnersApi.capitalHistory).mockImplementation((input) =>
    answer(() => books.capitalHistory(input)),
  );
  vi.mocked(partnersApi.saveCapitalIn).mockImplementation((input) =>
    answer(() => books.saveCapital('capital_in', input)),
  );
  vi.mocked(partnersApi.saveCapitalOut).mockImplementation((input) =>
    answer(() => books.saveCapital('capital_out', input)),
  );
  vi.mocked(partnersApi.partnersStatement).mockImplementation((input) =>
    answer(() => books.statement(input)),
  );
  // The generic voucher preview also answers for the capital kinds.
  const previous = vi.mocked(vouchersApi.vouchersPreview).getMockImplementation();
  vi.mocked(vouchersApi.vouchersPreview).mockImplementation((entry, replacing) => {
    if (entry.type === 'capital_in' || entry.type === 'capital_out') {
      return answer(() => books.capitalPreview(entry));
    }
    if (previous === undefined) throw new Error('no voucher preview fake');
    return previous(entry, replacing);
  });
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([SHAHID]);
  const partners = fakePartners(THREE_PARTNERS);
  fakeOverview(partners);
  return { cash, money: books, partners };
}

/** The partners with the numbers of PRD 4.9 and E2E-37 on their overview cards. */
export function fakeOverview(partners: ReturnType<typeof fakePartners>): void {
  const [a, b, c] = THREE_PARTNERS;
  if (a === undefined || b === undefined || c === undefined) throw new Error('no partners');
  partners.overview = [
    overviewRow(a, {
      month_profit_paisa: money(996),
      month_taken_paisa: money(2000),
      month_returned_paisa: money(0),
      month_net_paisa: money(-1004),
      total_paisa: money(-1004),
    }),
    overviewRow(b, {
      month_profit_paisa: money(746),
      capital_balance_paisa: money(50000),
      month_capital_in_paisa: money(50000),
      total_paisa: money(50746),
    }),
    overviewRow(c, {
      month_profit_paisa: money(746),
      month_net_paisa: money(746),
      total_paisa: money(746),
    }),
  ];
}
