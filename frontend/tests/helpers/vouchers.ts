// Test helper: an in-memory fake of the voucher commands (plans 10 and 15 contract) and of the day
// commands. The wrappers are mocked in the test file with vi.mock; this sets their answers from the
// books of voucher-books.ts, the way a healthy backend answers (the same books the development page
// /dev/vouchers uses). The sums there are a test double's: the screen only shows what comes back and
// never works out an amount (CLAUDE.md rule 1).

import { vi } from 'vitest';
import type { DayCloseSummary } from '../../src/bindings/DayCloseSummary';
import type { DayStatus } from '../../src/bindings/DayStatus';
import * as appApi from '../../src/app/api';
import * as dayApi from '../../src/features/day/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { account, ZERO, balance, money } from './accounts';
import {
  TODAY_ISO,
  collectionList,
  createBooks,
  voucherRow,
  type BooksOptions,
  type VoucherBooks,
} from './voucher-books';

export { TODAY_ISO, collectionList, voucherRow };
export type FakeVouchers = VoucherBooks;

const isoOf = (text: string): string => text.split('-').reverse().join('-');

export const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  balance: balance(12880, 'dr'),
});
export const ARSHAD = account(22, 'buyer', '0021', 'ارشد ملتانی', 'Arshad Multani', {
  balance: balance(18684, 'dr'),
});
export const PUKHTA_BUYER = account(
  23,
  'buyer',
  '0030',
  'پختہ کھاتہ (خریدار)',
  'Pukhta Khata (Buyer)',
);
export const RAHIM = account(31, 'seller', '0010', 'رحیم', 'Rahim', {
  balance: balance(4316, 'cr'),
});
export const KARIM = account(32, 'seller', '0011', 'کریم', 'Karim', {
  balance: balance(10160, 'cr'),
});
export const TRANSPORT = account(41, 'other', '02010002', 'کرایہ کھاتہ', 'Freight Payable', {
  balance: balance(4000, 'cr'),
});

/** Sets the answers of the voucher commands from fresh books, and returns the books. */
export function fakeVouchers(options: BooksOptions = {}): FakeVouchers {
  const books = createBooks(options);
  // A refusal thrown by the books is a rejection, as `invoke` rejects with the refusal itself.
  const answer = <T>(make: () => T): Promise<T> =>
    new Promise<T>((resolve) => {
      resolve(make());
    });
  vi.mocked(vouchersApi.vouchersPreview).mockImplementation((entry) =>
    answer(() => books.preview(entry)),
  );
  vi.mocked(vouchersApi.vouchersSave).mockImplementation((built) =>
    answer(() => books.save(built)),
  );
  vi.mocked(vouchersApi.vouchersEdit).mockImplementation((input) =>
    answer(() => books.edit(input)),
  );
  vi.mocked(vouchersApi.vouchersCancel).mockImplementation((input) =>
    answer(() => books.cancel(input)),
  );
  vi.mocked(vouchersApi.vouchersGet).mockImplementation((id) => answer(() => books.get(id)));
  vi.mocked(vouchersApi.vouchersList).mockImplementation((input) =>
    answer(() => books.list(input)),
  );
  vi.mocked(vouchersApi.assistOpenBills).mockImplementation((input) =>
    answer(() => books.openBills(input)),
  );
  vi.mocked(vouchersApi.assistOpenPayables).mockImplementation((input) =>
    answer(() => books.openPayables(input)),
  );
  vi.mocked(vouchersApi.assistAdvances).mockImplementation((sellerId) =>
    answer(() => books.advances(sellerId)),
  );
  return books.store;
}

/** The advance screen's fake, as the first tests of the advance read it. */
export const fakeAdvances = fakeVouchers;

export interface FakeDay {
  status: DayStatus;
  /** How many days a close would close, as the preview says. */
  days: number;
  /** The trial balance difference the preview says. */
  trial: ReturnType<typeof balance>;
  /** The full summary of the days to close (PRD FR-21.1). Default: the worked day of PRD 4.9. */
  summary: DayCloseSummary;
}

const PARTNER_REFS = [
  { id: 1, code: '', name_ur: 'الف', name_en: 'Partner A' },
  { id: 2, code: '', name_ur: 'ب', name_en: 'Partner B' },
  { id: 3, code: '', name_ur: 'ج', name_en: 'Partner C' },
] as const;

/** PRD 4.9: 2 بکری, خام 41,400, commission 5,448, shared income 5,488, shares 996 / 746 / 746. */
export const WORKED_DAY_SUMMARY: DayCloseSummary = {
  bikri_count: 2,
  gross_paisa: money(41400),
  commission_paisa: money(5448),
  shared_income_paisa: money(5488),
  partners: [
    {
      partner: PARTNER_REFS[0],
      income_paisa: money(2196),
      expense_paisa: money(1200),
      profit_paisa: money(996),
    },
    {
      partner: PARTNER_REFS[1],
      income_paisa: money(1646),
      expense_paisa: money(900),
      profit_paisa: money(746),
    },
    {
      partner: PARTNER_REFS[2],
      income_paisa: money(1646),
      expense_paisa: money(900),
      profit_paisa: money(746),
    },
  ],
  open_lots: 0,
  open_qty: 0,
};

/** Sets the answers of the day commands. A close and a reopen change the day status the app reads. */
export function fakeDay(options: Partial<FakeDay> = {}): FakeDay {
  const store: FakeDay = {
    status: { today: '2026-10-10', closed_up_to: '2026-09-30', today_open: true },
    days: 10,
    trial: ZERO,
    summary: WORKED_DAY_SUMMARY,
    ...options,
  };
  const reading = () => ({
    ...store.status,
    today_open:
      store.status.closed_up_to === null || store.status.closed_up_to < store.status.today,
  });
  vi.mocked(appApi.dayStatus).mockImplementation(() => Promise.resolve(reading()));
  vi.mocked(dayApi.dayClosePreview).mockImplementation((upTo) =>
    Promise.resolve({
      from_date: '2026-10-01',
      // No date typed: the oldest open day (PRD FR-21.1).
      up_to: upTo === '' ? '2026-10-01' : isoOf(upTo),
      days: store.days,
      trial_dr_paisa: money(85048),
      trial_cr_paisa: money(85048),
      trial_difference: store.trial,
      cash_in_hand: balance(58484, 'dr'),
      entries: 12,
      summary: store.summary,
    }),
  );
  vi.mocked(dayApi.dayClose).mockImplementation((upTo) => {
    store.status = { ...store.status, closed_up_to: isoOf(upTo) };
    return Promise.resolve(reading());
  });
  vi.mocked(dayApi.dayReopen).mockImplementation((date) => {
    const before = new Date(`${isoOf(date)}T00:00:00Z`);
    before.setUTCDate(before.getUTCDate() - 1);
    store.status = { ...store.status, closed_up_to: before.toISOString().slice(0, 10) };
    return Promise.resolve(reading());
  });
  return store;
}
