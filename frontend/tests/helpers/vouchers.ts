// Test helper: an in-memory fake of the voucher and the day commands (plan 10 contract). The
// wrappers are mocked in the test file with vi.mock; this sets their answers the way a healthy
// backend answers. The sums here are the fake backend's (a test double): the screen only shows what
// comes back and never works out an amount (CLAUDE.md rule 1).

import type { DayStatus } from '../../src/bindings/DayStatus';
import { vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as dayApi from '../../src/features/day/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import type { AdvancePreviewRow } from '../../src/features/vouchers/types';
import { ZERO, balance, money } from './accounts';

const rupeesOf = (text: string): number => Number(text.replaceAll(',', '') || '0');

/** `DD-MM-YYYY` as `YYYY-MM-DD`. */
const isoOf = (text: string): string => text.split('-').reverse().join('-');

export interface FakeAdvances {
  /** The vouchers saved so far, in order. */
  saved: { seller: number; amount: string }[];
}

/**
 * Sets the answers of the advance commands. A seller starts at zero; each saved advance adds to his
 * debit. The preview chains vouchers of one seller in a list, like the real one.
 */
export function fakeAdvances(): FakeAdvances {
  const store: FakeAdvances = { saved: [] };
  const debit = (seller: number) =>
    store.saved
      .filter((one) => one.seller === seller)
      .reduce((t, one) => t + rupeesOf(one.amount), 0);

  vi.mocked(vouchersApi.vouchersPreviewAdvances).mockImplementation((drafts) => {
    const running = new Map<number, number>();
    let total = 0;
    const rows = drafts.map((draft, index): AdvancePreviewRow => {
      const seller = draft.seller.id;
      const before = running.get(seller) ?? debit(seller);
      const amount = rupeesOf(draft.amount);
      const decimals = draft.amount.includes('.');
      running.set(seller, before + (decimals ? 0 : amount));
      if (!decimals) total += amount;
      return {
        account: {
          id: draft.seller.id,
          code: draft.seller.code ?? '',
          name_ur: draft.seller.name_ur,
          name_en: draft.seller.name_en,
        },
        // The real answer names the line: `lines[i].<field>`.
        errors: decimals
          ? [
              {
                code: 'ERR_DECIMALS_NOT_ALLOWED',
                field: `lines[${String(index)}].amount`,
                values: {},
              },
            ]
          : [],
        amount_paisa: money(decimals ? 0 : amount),
        current_balance: before === 0 ? ZERO : balance(before, 'dr'),
        new_balance: balance(before + (decimals ? 0 : amount), 'dr'),
      };
    });
    return Promise.resolve({ rows, total_paisa: money(total) });
  });
  vi.mocked(vouchersApi.vouchersSaveAdvance).mockImplementation((draft) => {
    const seller = draft.seller.id;
    store.saved.push({ seller, amount: draft.amount });
    return Promise.resolve({
      voucher_id: 500 + store.saved.length,
      voucher_type: 'advance',
      number: store.saved.length,
      entry_id: 1000 + store.saved.length,
      new_balance: balance(debit(seller), 'dr'),
    });
  });
  vi.mocked(vouchersApi.assistAdvances).mockImplementation((sellerId) =>
    Promise.resolve(
      sellerId === 31
        ? {
            open: [
              {
                source: 'voucher',
                voucher_number: 1,
                entry_date: '2026-09-01',
                given_paisa: money(15000),
                adjusted_paisa: money(0),
                open_paisa: money(15000),
              },
            ],
            open_paisa: money(15000),
            last: { entry_date: '2026-09-01', amount_paisa: money(15000), voucher_number: 1 },
          }
        : { open: [], open_paisa: money(0), last: null },
    ),
  );
  return store;
}

export interface FakeDay {
  status: DayStatus;
  /** How many days a close would close, as the preview says. */
  days: number;
  /** The trial balance difference the preview says. */
  trial: ReturnType<typeof balance>;
}

/** Sets the answers of the day commands. A close and a reopen change the day status the app reads. */
export function fakeDay(options: Partial<FakeDay> = {}): FakeDay {
  const store: FakeDay = {
    status: { today: '2026-10-10', closed_up_to: '2026-09-30', today_open: true },
    days: 10,
    trial: ZERO,
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
