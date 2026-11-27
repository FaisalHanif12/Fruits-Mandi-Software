// Test helper (plan 100): the bell's panel and the کل مالیت page as the backend would answer them after the
// PRD worked day (Events 1 to 5, 10-10-26), in the generated shapes. The numbers are the plan's own worked
// check: money to have 58,484 + 21,564 = 80,048, money to give 10,160 + 300 + 4,000 + 100 = 14,560, net
// 65,488. Hand written for screen tests and screenshots only; the backend's golden tests hold the figures.

import type { AttentionPanel } from '../../src/bindings/AttentionPanel';
import type { ReportParty } from '../../src/bindings/ReportParty';
import type { TotalWorthPage } from '../../src/bindings/TotalWorthPage';
import type { TotalWorthRow } from '../../src/bindings/TotalWorthRow';
import type { TotalWorthSide } from '../../src/bindings/TotalWorthSide';
import type { TotalWorthSource } from '../../src/bindings/TotalWorthSource';
import type { TotalWorthTile } from '../../src/bindings/TotalWorthTile';
import { paisaForTest } from './paisa';
import { ATTENTION_18_10 } from './dashboard-real';

const rupees = (value: number) => paisaForTest(value * 100);

function party(id: number, code: string, ur: string, en: string): ReportParty {
  return {
    account_id: id,
    code,
    short_code: null,
    name_ur: ur,
    name_en: en,
    tag: null,
    person_key: null,
  };
}

const KARIM = party(11, '0011', 'کریم', 'Karim');
const AKRAM = party(20, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const ARSHAD = party(21, '0021', 'ارشد ملتانی', 'Arshad Multani');

export const PANEL_WORKED_DAY: AttentionPanel = {
  sellers: {
    rows: [{ item_key: 'seller:11', party: KARIM, amount_paisa: rupees(10_160), days: 0 }],
    count: 1,
    total_paisa: rupees(10_160),
  },
  buyers: {
    rows: [
      { item_key: 'buyer:21', party: ARSHAD, amount_paisa: rupees(18_684), days: 0 },
      { item_key: 'buyer:20', party: AKRAM, amount_paisa: rupees(2_880), days: 0 },
    ],
    count: 2,
    total_paisa: rupees(21_564),
  },
  payables: [
    {
      item_key: 'payable:2',
      account_id: 2,
      code: '02010002',
      name_ur: 'کرایہ کھاتہ',
      name_en: 'Freight Payable',
      amount_paisa: rupees(4_000),
    },
    {
      item_key: 'payable:1',
      account_id: 1,
      code: '02010001',
      name_ur: 'مزدوری کھاتہ',
      name_en: 'Labour Payable',
      amount_paisa: rupees(300),
    },
    {
      item_key: 'payable:4',
      account_id: 4,
      code: '02010004',
      name_ur: 'منشیانہ کھاتہ',
      name_en: 'Munshiana Payable',
      amount_paisa: rupees(100),
    },
  ],
  general: ATTENTION_18_10.items
    .filter((item) => item.kind === 'open_days' || item.kind === 'backup_old')
    .map((item) => ({ ...item, item_key: `kind:${item.kind}` })),
  bell_count: 8,
  hidden_count: 0,
  partners_locked: false,
};

/** A panel with nothing to do. */
export const PANEL_EMPTY: AttentionPanel = {
  sellers: { rows: [], count: 0, total_paisa: rupees(0) },
  buyers: { rows: [], count: 0, total_paisa: rupees(0) },
  payables: [],
  general: [],
  bell_count: 0,
  hidden_count: 0,
  partners_locked: false,
};

export const WORTH_TOTALS: TotalWorthTile = {
  have_paisa: rupees(80_048),
  give_paisa: rupees(14_560),
  net_paisa: rupees(65_488),
};

const row = (source: TotalWorthSource, who: ReportParty, amount: number): TotalWorthRow => ({
  source,
  party: who,
  amount_paisa: rupees(amount),
});

const HAVE_ROWS: TotalWorthRow[] = [
  row('cash', party(1, '01010001', 'روکڑ', 'Cash'), 58_484),
  row('buyers_dues', ARSHAD, 18_684),
  row('buyers_dues', AKRAM, 2_880),
];

const GIVE_ROWS: TotalWorthRow[] = [
  row('sellers_payable', KARIM, 10_160),
  row('payables', party(2, '02010002', 'کرایہ کھاتہ', 'Freight Payable'), 4_000),
  row('payables', party(1, '02010001', 'مزدوری کھاتہ', 'Labour Payable'), 300),
  row('payables', party(4, '02010004', 'منشیانہ کھاتہ', 'Munshiana Payable'), 100),
];

/** One side of the کل مالیت page after Events 1 to 5. */
export function totalWorthPage(side: TotalWorthSide): TotalWorthPage {
  const rows = side === 'have' ? HAVE_ROWS : GIVE_ROWS;
  return {
    today: '2026-10-10',
    side,
    rows,
    total_count: rows.length,
    offset: 0,
    limit: 25,
    parts: [
      { source: 'cash', side: 'have', total_paisa: rupees(58_484), count: 1 },
      { source: 'buyers_dues', side: 'have', total_paisa: rupees(21_564), count: 2 },
      { source: 'sellers_payable', side: 'give', total_paisa: rupees(10_160), count: 1 },
      { source: 'payables', side: 'give', total_paisa: rupees(4_400), count: 3 },
    ],
    totals: WORTH_TOTALS,
  };
}
