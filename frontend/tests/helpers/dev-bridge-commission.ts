// A stand-in for the Tauri bridge for the development page /dev/commission (plan 77) only. In a plain browser
// tab there is no backend, so the commission page cannot be looked at. This answers its commands with the numbers of
// the plan (after Events 1 to 5, 10-10-26: the whole shop 5,448, seller رحیم 29,400 / 1,764 / 3,928, buyer
// اکرم ٹول ٹیکس 12,000 / 720 / 160 / 880), for a month, a date, a year and a range, with a party or without, and
// the account pickers of the party boxes. Any other command is refused with ERR_NOT_IMPLEMENTED. Never part of a
// build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/commission?empty`:
//   empty   no commission at all   down   the commission call fails   locked   the partner section is locked

import type { CommissionDay } from '../../src/bindings/CommissionDay';
import type { CommissionDayRow } from '../../src/bindings/CommissionDayRow';
import type { CommissionMonth } from '../../src/bindings/CommissionMonth';
import type { CommissionMonthInput } from '../../src/bindings/CommissionMonthInput';
import type { CommissionPartyShown } from '../../src/bindings/CommissionPartyShown';
import { dashboardHandlers, inputOf, refusal } from './dev-bridge-dashboard';
import { devFlag } from './dev-bridge';
import { rokarHandlers } from './dev-bridge-rokar';
import { paisaForTest } from './paisa';

type Input = Record<string, unknown>;

const rupees = (value: number) => paisaForTest(value * 100);

const PARTNERS = [
  { id: 1, name_ur: 'الف', name_en: 'Partner A' },
  { id: 2, name_ur: 'ب', name_en: 'Partner B' },
  { id: 3, name_ur: 'ج', name_en: 'Partner C' },
];

const SHOP: CommissionDayRow = {
  date: '2026-10-10',
  seller_comm_paisa: rupees(2_484),
  buyer_comm_paisa: rupees(2_484),
  laga_paisa: rupees(480),
  total_comm_paisa: rupees(5_448),
  shared_income_paisa: rupees(5_488),
  partner_shares: [
    { partner_id: 1, share_paisa: rupees(2_196) },
    { partner_id: 2, share_paisa: rupees(1_646) },
    { partner_id: 3, share_paisa: rupees(1_646) },
  ],
  sale_paisa: null,
  goods_comm_paisa: null,
};

const SELLER: CommissionDayRow = {
  date: '2026-10-10',
  seller_comm_paisa: rupees(1_764),
  buyer_comm_paisa: rupees(1_764),
  laga_paisa: rupees(400),
  total_comm_paisa: rupees(3_928),
  shared_income_paisa: rupees(0),
  partner_shares: null,
  sale_paisa: rupees(29_400),
  goods_comm_paisa: rupees(3_928),
};

const BUYER: CommissionDayRow = {
  date: '2026-10-10',
  seller_comm_paisa: rupees(0),
  buyer_comm_paisa: rupees(720),
  laga_paisa: rupees(160),
  total_comm_paisa: rupees(880),
  shared_income_paisa: rupees(0),
  partner_shares: null,
  sale_paisa: rupees(12_000),
  goods_comm_paisa: null,
};

function partyShown(side: 'buyer' | 'seller'): CommissionPartyShown {
  const seller = side === 'seller';
  return {
    side,
    party: {
      account_id: seller ? 19 : 21,
      code: seller ? '0010' : '0020',
      short_code: null,
      name_ur: seller ? 'رحیم' : 'اکرم ٹول ٹیکس',
      name_en: seller ? 'Rahim' : 'Akram Toll Tax',
      tag: null,
      person_key: null,
    },
  };
}

/** The figures of a row of the page for a side (none is the whole shop), on the given date. */
function rowOf(side: 'buyer' | 'seller' | undefined, date: string): CommissionDayRow {
  const base = side === undefined ? SHOP : side === 'seller' ? SELLER : BUYER;
  return { ...base, date };
}

function month(input: CommissionMonthInput): CommissionMonth {
  if (devFlag('down')) throw refusal('ERR_INTERNAL');
  const period = input.period;
  const kind = period?.kind ?? 'month';
  const party = input.party === undefined ? null : partyShown(input.party.side);
  const empty = devFlag('empty');
  // A month is the 8th, 9th and 10th of October; a year is August, September and October; a range is
  // 40 days, 25 to a page; a date is one row.
  let dates: string[];
  if (kind === 'day') dates = [period?.date ?? '2026-10-10'];
  else if (kind === 'year') dates = ['2026-08', '2026-09', '2026-10'];
  else if (kind === 'range') {
    dates = Array.from({ length: 40 }, (_, index) => {
      const day = 10 - (index % 10);
      const monthNumber = 10 - Math.floor(index / 10);
      return `2026-${String(monthNumber).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
    });
  } else dates = ['2026-10-08', '2026-10-09', '2026-10-10'];
  const offset = input.offset ?? 0;
  const limit = (input.limit ?? 0) || 100;
  const all = empty ? [] : dates.map((date) => rowOf(input.party?.side, date));
  const rows = kind === 'range' ? all.slice(offset, offset + limit) : all;
  const first = all[0];
  const totals: CommissionDayRow = { ...(first ?? rowOf(input.party?.side, '')), date: '' };
  const targetMonth = input.month ?? '2026-10';
  return {
    month: kind === 'month' ? targetMonth : '2026-10',
    year: 2026,
    period: { kind, from: '2026-10-01', to: '2026-10-31' },
    party,
    first_month: '2026-08',
    current_month: '2026-10',
    partners: party === null ? PARTNERS : [],
    rows,
    total_count: all.length,
    offset,
    limit,
    totals,
    partners_locked: devFlag('locked') && party === null,
  };
}

function day(input: Input): CommissionDay {
  const party = input['party'] as { side: 'buyer' | 'seller' } | undefined;
  const dateInput = input['date'];
  const row = rowOf(party?.side, typeof dateInput === 'string' ? dateInput : '2026-10-10');
  return {
    date: row.date,
    party: party === undefined ? null : partyShown(party.side),
    partners: party === undefined ? PARTNERS : [],
    entries: [
      {
        entry_id: 5,
        posting_type: 'bikri_first',
        source_type: 'bikri',
        source_id: 41,
        bikri_id: 41,
        bikri_number: 1,
        sellers: [],
        buyers: [],
        figures: row,
      },
    ],
    total_count: 1,
    totals: row,
    partners_locked: false,
  };
}

const handlers: Record<string, (input: Input) => unknown> = {
  ...dashboardHandlers,
  dashboard_commission_month: (input) => month(input as unknown as CommissionMonthInput),
  dashboard_commission_day: (input) => day(input),
  // The party boxes search the accounts of the Rokar stand-in (sellers and buyers).
  search_accounts: (input) => rokarHandlers['search_accounts']?.(input),
  search_accounts_recent: (input) => rokarHandlers['search_accounts_recent']?.(input),
  search_accounts_touch: () => null,
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installCommissionDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      transformCallback: () => 0,
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const handler = handlers[command];
            if (handler === undefined) {
              reject(refusal('ERR_NOT_IMPLEMENTED'));
              return;
            }
            try {
              resolve(handler(inputOf(args)));
            } catch (error) {
              reject(error instanceof Error ? error : refusal('ERR_INTERNAL'));
            }
          }, 20);
        }),
    },
  });
}
