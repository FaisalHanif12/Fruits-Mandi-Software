// Test data for the saved بکری, the list and the پختہ کھاتہ register (plan 13): the answers of
// `bikri_get`, `bikri_list`, `bikri_find` and `pukhta_register` for Event 1 of PRD 4.4, in the
// exact shapes of the generated types. The numbers are the PRD's worked numbers (E2E-1 to E2E-5):
// they are canned answers of a fake backend, and the screens only show them.

import type { BikriDetail } from '../../src/bindings/BikriDetail';
import type { BikriListRow } from '../../src/bindings/BikriListRow';
import type { PukhtaPerson } from '../../src/bindings/PukhtaPerson';
import type { PukhtaRegisterRow } from '../../src/bindings/PukhtaRegisterRow';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { SavedCharge } from '../../src/bindings/SavedCharge';
import type { SavedLotCharge } from '../../src/bindings/SavedLotCharge';
import type { SavedSale } from '../../src/bindings/SavedSale';
import type { SavedTotals } from '../../src/bindings/SavedTotals';
import { IDS } from './bikri-data';
import { paisaForTest } from './paisa';

// Written here and not imported from ./accounts, so the development page can use this file
// without pulling Vitest into the browser.
const money = (rupees: number) => paisaForTest(rupees * 100);
const ref = (id: number, code: string, ur: string, en: string): RecordRef => ({
  id,
  code,
  name_ur: ur,
  name_en: en,
});

const RAHIM = ref(IDS.rahim, '030110001', 'رحیم', 'Rahim');
const AKRAM = ref(IDS.akram, '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const ARSHAD = ref(IDS.arshad, '03040002', 'ارشد ملتانی', 'Arshad Multani');
const BANANA = ref(IDS.banana, '0001', 'کیلا', 'Banana');
const BOX = ref(IDS.box, '0001', 'پیٹی', 'Box');

function percent(bps: number, rupees: number): SavedCharge {
  return {
    method: 'percent',
    value_bps: bps,
    value_paisa: null,
    amount: money(rupees),
    rule_id: null,
  };
}

function perUnit(rupees: number, amount: number): SavedCharge {
  return {
    method: 'per_unit',
    value_bps: null,
    value_paisa: money(rupees),
    amount: money(amount),
    rule_id: null,
  };
}

function lotCharge(amount: number, method: SavedLotCharge['method'] = 'manual'): SavedLotCharge {
  return { method, value_paisa: money(amount), amount: money(amount), rule_id: null };
}

/** Akram 20 at 600: 12,000 + 720 + 160 = 12,880 (E2E-2). */
export const AKRAM_LINE: SavedSale = {
  id: 101,
  line_no: 1,
  journal_entry_id: 7,
  entry_date: '2026-10-10',
  buyer: AKRAM,
  pukhta: null,
  qty: 20,
  weight_grams: null,
  seller_rate: money(600),
  seller_amount: money(12_000),
  seller_comm: percent(600, 720),
  buyer_rate: money(600),
  buyer_value: money(12_000),
  buyer_comm: percent(600, 720),
  laga: perUnit(8, 160),
  buyer_total: money(12_880),
  rate_diff: money(0),
  payment_type: 'credit',
  is_active: true,
  note: null,
};

/** Arshad 30 at 580: 17,400 + 1,044 + 240 = 18,684 (E2E-2). */
export const ARSHAD_LINE: SavedSale = {
  ...AKRAM_LINE,
  id: 102,
  line_no: 2,
  buyer: ARSHAD,
  qty: 30,
  seller_rate: money(580),
  seller_amount: money(17_400),
  seller_comm: percent(600, 1_044),
  buyer_rate: money(580),
  buyer_value: money(17_400),
  buyer_comm: percent(600, 1_044),
  laga: perUnit(8, 240),
  buyer_total: money(18_684),
};

/** The stored sums of Event 1: one lot, so the lot and the whole بکری have the same ones. */
export const EVENT1_TOTALS: SavedTotals = {
  gross: money(29_400),
  freight: money(3_000),
  seller_comm: money(1_764),
  labour: money(250),
  munshiana: money(50),
  cash_paid: money(5_000),
  store_rent: money(0),
  phone: money(20),
  deductions: money(10_084),
  net: money(19_316),
  seller_credit: money(24_316),
  buyer_comm: money(1_764),
  laga: money(400),
  rate_diff: money(0),
  buyer_credit: money(31_564),
  buyer_cash: money(0),
  buyer_total: money(31_564),
  shared_income: money(3_948),
};

/** Event 1 as saved (E2E-1 to E2E-4): gross 29,400, deductions 10,084, net 19,316. */
export function savedEvent1(overrides: Partial<BikriDetail> = {}): BikriDetail {
  return {
    id: 41,
    number: 1,
    book: null,
    entry_date: '2026-10-10',
    bill_date: '2026-10-10',
    truck_no: 'LES-1234',
    zamindar: RAHIM,
    adda_id: IDS.swat,
    driver_name: 'Aslam',
    driver_phone: '03001234567',
    total_freight: null,
    store_id: null,
    note: null,
    status: 'saved',
    cancel_reason: null,
    print_count: 0,
    created_at: '2026-10-10T07:30:00Z',
    lots: [
      {
        id: 51,
        lot_no: 1,
        seller: RAHIM,
        pukhta: null,
        item: BANANA,
        packing: BOX,
        marka_id: null,
        marka_text: null,
        store_id: null,
        qty_received: 50,
        qty_sold: 50,
        qty_written_off: 0,
        weight_received_grams: null,
        freight: money(3_000),
        freight_paid_by: 'firm',
        labour: lotCharge(250, 'per_unit'),
        munshiana: lotCharge(50),
        phone: lotCharge(20),
        cash_paid: money(5_000),
        store_rent: money(0),
        adjust_advance: true,
        first_entry_id: 7,
        gross: money(29_400),
        seller_comm: money(1_764),
        deductions: money(10_084),
        net: money(19_316),
        totals: EVENT1_TOTALS,
        sales: [AKRAM_LINE, ARSHAD_LINE],
      },
    ],
    totals: EVENT1_TOTALS,
    postings: [
      { entry_id: 7, posting_type: 'bikri_first', entry_date: '2026-10-10', total: money(36_564) },
    ],
    split: [
      { partner_id: 1, name_ur: 'الف', name_en: 'Partner A', income_paisa: money(1_580) },
      { partner_id: 2, name_ur: 'ب', name_en: 'Partner B', income_paisa: money(1_184) },
      { partner_id: 3, name_ur: 'ج', name_en: 'Partner C', income_paisa: money(1_184) },
    ],
    ...overrides,
  };
}

/** The row of Event 1 in the list (PRD FR-12.7). */
export function listRow(overrides: Partial<BikriListRow> = {}): BikriListRow {
  return {
    id: 41,
    number: 1,
    entry_date: '2026-10-10',
    created_at: '2026-10-10T07:30:00Z',
    truck_no: 'LES-1234',
    sellers: [RAHIM],
    pukhta_sellers: [],
    items: [BANANA],
    qty_received: 50,
    qty_sold: 50,
    gross: money(29_400),
    net: money(19_316),
    buyer_total: money(31_564),
    status: 'saved',
    ...overrides,
  };
}

/** Shahid, the one-time buyer of Event 2 (PRD 4.5). */
export const SHAHID: PukhtaPerson = {
  name_ur: 'شاہد',
  name_en: 'Shahid',
  phone: '03000000000',
  last_date: '2026-10-10',
  person_key: 'shahid|03000000000',
};

export const NASIR: PukhtaPerson = {
  name_ur: 'ناصر',
  name_en: 'Nasir',
  phone: '03111111111',
  last_date: '2026-10-08',
  person_key: 'nasir|03111111111',
};

/** The register line of Event 2: Shahid bought 10 boxes of apples at 1,200 and paid (E2E-5). */
export function registerRow(overrides: Partial<PukhtaRegisterRow> = {}): PukhtaRegisterRow {
  return {
    entry_date: '2026-10-10',
    bikri_id: 42,
    bikri_number: 2,
    side: 'buyer',
    name_ur: SHAHID.name_ur,
    name_en: SHAHID.name_en,
    phone: SHAHID.phone,
    person_key: SHAHID.person_key,
    item: ref(IDS.apple, '0002', 'سیب', 'Apple'),
    qty: 10,
    amount: money(12_800),
    commission: money(720),
    paid: true,
    open: money(0),
    cancelled: false,
    ...overrides,
  };
}

/** Event 2's lot (PRD 4.5): 10 boxes of apples from Karim, sold to Shahid for 12,800. */
export const EVENT2_TOTALS: SavedTotals = {
  gross: money(12_000),
  freight: money(1_000),
  seller_comm: money(720),
  labour: money(50),
  munshiana: money(50),
  cash_paid: money(0),
  store_rent: money(0),
  phone: money(20),
  deductions: money(1_840),
  net: money(10_160),
  seller_credit: money(10_160),
  buyer_comm: money(720),
  laga: money(80),
  rate_diff: money(0),
  buyer_credit: money(0),
  buyer_cash: money(12_800),
  buyer_total: money(12_800),
  shared_income: money(1_540),
};

/** Events 1 and 2 on one truck, as two lots: the whole's sums are the two lots' sums (E2E-10). */
export const TWO_LOT_TOTALS: SavedTotals = {
  gross: money(41_400),
  freight: money(4_000),
  seller_comm: money(2_484),
  labour: money(300),
  munshiana: money(100),
  cash_paid: money(5_000),
  store_rent: money(0),
  phone: money(40),
  deductions: money(11_924),
  net: money(29_476),
  seller_credit: money(34_476),
  buyer_comm: money(2_484),
  laga: money(480),
  rate_diff: money(0),
  buyer_credit: money(31_564),
  buyer_cash: money(12_800),
  buyer_total: money(44_364),
  shared_income: money(5_488),
};

/** A saved بکری with two lots: Rahim's bananas (Event 1) and Karim's apples (Event 2). */
export function savedTwoLots(): BikriDetail {
  const one = savedEvent1();
  const first = one.lots[0];
  if (first === undefined) throw new Error('no lot');
  const karim = ref(IDS.karim, '030110002', 'کریم', 'Karim');
  return {
    ...one,
    lots: [
      first,
      {
        ...first,
        id: 52,
        lot_no: 2,
        seller: karim,
        item: ref(IDS.apple, '0002', 'سیب', 'Apple'),
        qty_received: 10,
        qty_sold: 10,
        gross: money(12_000),
        net: money(10_160),
        totals: EVENT2_TOTALS,
        sales: [{ ...AKRAM_LINE, id: 103, line_no: 1, qty: 10, seller_rate: money(1_200) }],
      },
    ],
    totals: TWO_LOT_TOTALS,
  };
}
