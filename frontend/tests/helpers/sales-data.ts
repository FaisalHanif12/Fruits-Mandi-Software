// The answers of the sales page (plan 83, PRD FR-9.9) for the worked day, 10-10-26, after Events 1 to 5: two بکری, 60 نگ,
// خام بکری 41,400, the buyers' bill 44,364 (12,880 + 18,684 + 12,800), 12,800 paid on the spot (شاہد, a چکتا buyer) and
// 31,564 left on account. بکری 1 is رحیم، کیلا-1 (29,400; its buyers owe 31,564 and paid nothing) and بکری 2 is کریم،
// سیب-1 (12,000; شاہد paid all 12,800). Built as the types of the contract; money is paisa through `paisaForTest`.

import type { SalesDay } from '../../src/bindings/SalesDay';
import type { SalesDayRow } from '../../src/bindings/SalesDayRow';
import type { SalesPeriod } from '../../src/bindings/SalesPeriod';
import type { SalesPeriodRow } from '../../src/bindings/SalesPeriodRow';
import { paisaForTest } from './paisa';

const rupees = (value: number) => paisaForTest(value * 100);

const ref = (id: number, code: string, ur: string, en: string) => ({
  id,
  code,
  name_ur: ur,
  name_en: en,
});

export const RAHIM = ref(19, '0010', 'رحیم', 'Rahim');
export const KARIM = ref(20, '0011', 'کریم', 'Karim');
export const AKRAM = ref(21, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
export const BANANA = ref(4, '0040', 'کیلا', 'Banana');
export const APPLE = ref(5, '0041', 'سیب', 'Apple');

/** The day 10-10-26 of the plan's table. */
export const DAY_FIGURES: SalesPeriodRow = {
  date: '2026-10-10',
  bikri_count: 2,
  qty: 60,
  gross_paisa: rupees(41_400),
  buyer_total_paisa: rupees(44_364),
  cash_paisa: rupees(12_800),
  credit_paisa: rupees(31_564),
};

export const DAY_ROW: SalesPeriodRow = DAY_FIGURES;

export function period(more: Partial<SalesPeriod> = {}): SalesPeriod {
  return {
    period: { kind: 'month', from: '2026-10-01', to: '2026-10-31' },
    month: '2026-10',
    first_month: '2026-09',
    current_month: '2026-10',
    party: null,
    item: null,
    goods: 'all',
    year: 2026,
    rows: [DAY_ROW],
    // The totals row of a month with one day has the day's figures and no date.
    totals: { ...DAY_FIGURES, date: '' },
    total_count: 1,
    offset: 0,
    limit: 100,
    today: '2026-10-10',
    ...more,
  };
}

/** The totals row of a period with no sales at all (its date is empty). */
export const EMPTY_FIGURES: SalesPeriodRow = {
  date: '',
  bikri_count: 0,
  qty: 0,
  gross_paisa: rupees(0),
  buyer_total_paisa: rupees(0),
  cash_paisa: rupees(0),
  credit_paisa: rupees(0),
};

export const BIKRI_1: SalesDayRow = {
  bikri_id: 41,
  bikri_number: 1,
  sellers: [RAHIM],
  lots: [{ lot_id: 7, seller: RAHIM, item: BANANA, counter: 1 }],
  qty: 50,
  gross_paisa: rupees(29_400),
  buyer_total_paisa: rupees(31_564),
  cash_paisa: rupees(0),
  credit_paisa: rupees(31_564),
};

export const BIKRI_2: SalesDayRow = {
  bikri_id: 42,
  bikri_number: 2,
  sellers: [KARIM],
  lots: [{ lot_id: 8, seller: KARIM, item: APPLE, counter: 1 }],
  qty: 10,
  gross_paisa: rupees(12_000),
  buyer_total_paisa: rupees(12_800),
  cash_paisa: rupees(12_800),
  credit_paisa: rupees(0),
};

export const DAY: SalesDay = {
  date: '2026-10-10',
  party: null,
  item: null,
  goods: 'all',
  rows: [BIKRI_1, BIKRI_2],
  total_count: 2,
  totals: DAY_FIGURES,
};
