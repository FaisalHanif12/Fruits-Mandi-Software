// Test helper of plan 87: the capital card as the backend sends it (PRD FR-16.14, `CapitalWhere` in
// `backend/crates/core/src/cashbook.rs`), for four states of the books. Every amount of the two blocks is unsigned:
// the side of an amount is the block it stands in. The numbers agree inside each card (have − owe = net = whose
// total), because the screen adds nothing up and a test must read exactly what the backend sent.
//
//   WORKED_DAY    the worked day after Events 1 to 5 (PRD FR-16.14): 58,484 + 21,564 = 80,048 had, 10,160 + 4,400 =
//                 14,560 owed, net 65,488 = −1,004 + 746 + 746 + 65,000.
//   UMAR_BEFORE   Umar's data of 20-11-2026, before the fix: two partner opening capitals of 500,000 sit on the opening
//                 difference, cash is below zero (on the paying side), and the net is short: 105,420 − 201,950 =
//                 96,530 owed more than held. The partners' totals are 419,235 each (838,470), less the difference of
//                 935,000 that is not placed: 838,470 − 935,000 = −96,530.
//   UMAR_AFTER    the same books after the opening capital was put into cash: cash 838,260, partner totals 419,235
//                 each, and the old books' difference of 65,000 جمع as a warning without a button.
//   EMPTY_CARD    nothing recorded yet.

import type { RecordRef } from '../../src/bindings/RecordRef';
import type {
  CapitalDifferencePart,
  CapitalWhere,
  CapitalWhereKind,
  CapitalWhereLine,
  CapitalWhoseLine,
  CapitalLink,
  CapitalSide,
} from '../../src/features/cashbook/capital-types';
import { paisaForTest } from './paisa';

const rupees = (value: number) => paisaForTest(value * 100);

export const PARTNER_A: RecordRef = { id: 1, code: '', name_ur: 'الف', name_en: 'Partner A' };
export const PARTNER_B: RecordRef = { id: 2, code: '', name_ur: 'ب', name_en: 'Partner B' };
export const PARTNER_C: RecordRef = { id: 3, code: '', name_ur: 'ج', name_en: 'Partner C' };
export const ADNAN: RecordRef = { id: 1, code: '', name_ur: 'عدنان', name_en: 'Adnan' };
export const USMAN: RecordRef = { id: 2, code: '', name_ur: 'عثمان', name_en: 'Usman' };

/** The accounts of the charges owed: labour, freight and munshiana (head 02), one line each. */
export const LABOUR: RecordRef = {
  id: 77,
  code: '02010001',
  name_ur: 'مزدوری کھاتہ',
  name_en: 'Labour Payable',
};
export const FREIGHT: RecordRef = {
  id: 78,
  code: '02010002',
  name_ur: 'کرایہ کھاتہ',
  name_en: 'Freight Payable',
};
export const MUNSHIANA: RecordRef = {
  id: 79,
  code: '02010004',
  name_ur: 'منشیانہ کھاتہ',
  name_en: 'Munshiana Payable',
};
/** ابتدائی سرمایہ, the account of the opening capital line. */
export const OPENING_CAPITAL_ACCOUNT: RecordRef = {
  id: 90,
  code: '06010001',
  name_ur: 'ابتدائی سرمایہ',
  name_en: 'Opening Capital',
};

const LINK_OF: Record<CapitalWhereKind, CapitalLink> = {
  cash: 'cashbook',
  buyers_owe: 'buyer_dues',
  buyer_credits: 'buyer_ledgers',
  dead_amount: 'dead_amount',
  seller_advances: 'advances',
  sellers_owe_beyond: 'seller_dues',
  sellers_payable: 'seller_dues',
  money_lent: 'money_lent',
  owed_for_charges: 'account_ledger',
  separate_khatas: 'separate_khatas',
  gardens_held: 'gardens',
  linked_in_partner_totals: 'partners',
  other: 'trial_balance',
};

export function line(
  kind: CapitalWhereKind,
  side: CapitalSide,
  amount: number,
  account: RecordRef | null = null,
): CapitalWhereLine {
  return { kind, side, amount_paisa: rupees(amount), link: LINK_OF[kind], account };
}

export function whose(
  kind: CapitalWhoseLine['kind'],
  partner: RecordRef | null,
  amount: number,
  shareBps: number | null = null,
  account: RecordRef | null = null,
): CapitalWhoseLine {
  const link: CapitalLink =
    kind === 'partner_total'
      ? 'partner_statement'
      : kind === 'advances_open'
        ? 'advances'
        : 'account_ledger';
  return { kind, partner, account, amount_paisa: rupees(amount), share_bps: shareBps, link };
}

export const WORKED_DAY: CapitalWhere = {
  as_of: '2026-10-10',
  have_lines: [
    line('cash', 'have', 58_484),
    line('buyers_owe', 'have', 21_564),
    line('seller_advances', 'have', 0),
    line('money_lent', 'have', 0),
  ],
  have_total_paisa: rupees(80_048),
  owe_lines: [
    line('sellers_payable', 'owe', 10_160),
    line('owed_for_charges', 'owe', 300, LABOUR),
    line('owed_for_charges', 'owe', 4_000, FREIGHT),
    line('owed_for_charges', 'owe', 100, MUNSHIANA),
  ],
  owe_total_paisa: rupees(14_560),
  net_paisa: rupees(65_488),
  net_side: 'have',
  whose_lines: [
    whose('partner_total', PARTNER_A, -1_004),
    whose('partner_total', PARTNER_B, 746, 114),
    whose('partner_total', PARTNER_C, 746, 114),
    whose('opening_capital', null, 65_000, null, OPENING_CAPITAL_ACCOUNT),
  ],
  whose_lines_total_paisa: rupees(65_488),
  whose_lines_side: 'have',
  difference: null,
  whose_total_paisa: rupees(65_488),
  sides_equal: true,
};

const UMAR_WHOSE: CapitalWhoseLine[] = [
  whose('partner_total', ADNAN, 419_235, 5_000),
  whose('partner_total', USMAN, 419_235, 5_000),
];

/** The six party openings of Umar's data: the opening side of the account, then its side on the difference. */
const PARTY_PARTS: CapitalDifferencePart[] = (
  [
    ['Mani', 'مانی', 30_000, 'cr'],
    ['Ans', 'انس', 15_000, 'cr'],
    ['adnan', 'عدنان', 60_000, 'cr'],
    ['Faisal', 'فیصل', 10_000, 'dr'],
    ['Haroon', 'ہارون', 10_000, 'dr'],
    ['Ehsan', 'احسان', 20_000, 'dr'],
  ] as const
).map(([en, ur, amount, side], index) => ({
  account: { id: 10 + index, code: `002${String(index)}`, name_ur: ur, name_en: en },
  partner: null,
  date: '2026-10-29',
  amount_paisa: rupees(amount),
  side,
  can_place: false,
  link: 'account_ledger',
}));

const PARTNER_PARTS: CapitalDifferencePart[] = [ADNAN, USMAN].map((partner) => ({
  account: { ...partner, id: partner.id + 100 },
  partner,
  date: '2026-10-29',
  amount_paisa: rupees(500_000),
  side: 'dr',
  can_place: true,
  link: 'account_ledger',
}));

export const UMAR_BEFORE: CapitalWhere = {
  as_of: '2026-11-20',
  have_lines: [line('buyers_owe', 'have', 105_420), line('seller_advances', 'have', 0)],
  have_total_paisa: rupees(105_420),
  owe_lines: [
    line('cash', 'owe', 161_740),
    line('sellers_payable', 'owe', 40_000),
    line('owed_for_charges', 'owe', 210, LABOUR),
  ],
  owe_total_paisa: rupees(201_950),
  net_paisa: rupees(96_530),
  net_side: 'owe',
  whose_lines: UMAR_WHOSE,
  whose_lines_total_paisa: rupees(838_470),
  whose_lines_side: 'have',
  difference: {
    amount_paisa: rupees(935_000),
    side: 'dr',
    groups: [
      {
        kind: 'partner_capital',
        amount_paisa: rupees(1_000_000),
        side: 'dr',
        parts: PARTNER_PARTS,
      },
      { kind: 'account_opening', amount_paisa: rupees(65_000), side: 'cr', parts: PARTY_PARTS },
    ],
    placeable_partners: [ADNAN, USMAN],
  },
  whose_total_paisa: rupees(96_530),
  sides_equal: true,
};

export const UMAR_AFTER: CapitalWhere = {
  as_of: '2026-11-20',
  have_lines: [
    line('cash', 'have', 838_260),
    line('buyers_owe', 'have', 105_420),
    line('seller_advances', 'have', 0),
  ],
  have_total_paisa: rupees(943_680),
  owe_lines: [line('sellers_payable', 'owe', 40_000), line('owed_for_charges', 'owe', 210, LABOUR)],
  owe_total_paisa: rupees(40_210),
  net_paisa: rupees(903_470),
  net_side: 'have',
  whose_lines: UMAR_WHOSE,
  whose_lines_total_paisa: rupees(838_470),
  whose_lines_side: 'have',
  difference: {
    amount_paisa: rupees(65_000),
    side: 'cr',
    groups: [
      { kind: 'account_opening', amount_paisa: rupees(65_000), side: 'cr', parts: PARTY_PARTS },
    ],
    placeable_partners: [],
  },
  whose_total_paisa: rupees(903_470),
  sides_equal: true,
};

/** A card with nothing recorded yet: every line is 0. */
export const EMPTY_CARD: CapitalWhere = {
  as_of: '2026-10-10',
  have_lines: [
    line('cash', 'have', 0),
    line('buyers_owe', 'have', 0),
    line('seller_advances', 'have', 0),
  ],
  have_total_paisa: rupees(0),
  owe_lines: [line('sellers_payable', 'owe', 0)],
  owe_total_paisa: rupees(0),
  net_paisa: rupees(0),
  net_side: 'have',
  whose_lines: [whose('opening_capital', null, 0, null, OPENING_CAPITAL_ACCOUNT)],
  whose_lines_total_paisa: rupees(0),
  whose_lines_side: 'have',
  difference: null,
  whose_total_paisa: rupees(0),
  sides_equal: true,
};
