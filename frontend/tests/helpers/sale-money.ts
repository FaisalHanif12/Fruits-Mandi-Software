// Test data: the answer of `gardens_sale_money` (plan 99, PRD FR-12.53) with the numbers of the plan's
// worked example: مانی buys 20 آم from the garden's lot at 1,000: value 20,000, his bill 21,360, nothing
// paid on the spot, 21,360 still on his account, and the garden credited 18,800 (less 1,200 commission);
// a row of the lot's charges, and a sale in another shop of 10,000. The sums are this test double's (a
// test double works on rupees); the screen only shows what comes back (CLAUDE.md rule 1). It is replaced
// by the backend's own answers where a test reads real shapes. No test runner in it.

import type { GardensSaleMoney } from '../../src/bindings/GardensSaleMoney';
import type { GardensSaleMoneyInput } from '../../src/bindings/GardensSaleMoneyInput';
import type { SaleMoneyRow } from '../../src/bindings/SaleMoneyRow';
import type { Working } from '../../src/bindings/Working';
import type { Paisa } from '../../src/shared/format/money';
import { paisaForTest, plainPaisaForTest } from './paisa';

const money = (rupees: number) => paisaForTest(rupees * 100);

const arg = (name: string, rupees: number): Working['args'][number] => ({
  name,
  value: { kind: 'money', paisa: money(rupees) },
});

const MANGO = { id: 3, code: '0042', name_ur: 'آم', name_en: 'Mango' };
const BOX = { id: 1, code: '0050', name_ur: 'پیٹی', name_en: 'Box' };
const LOT = { lot_id: 7, lot_no: 1, lot_counter: 1, item: MANGO, packing: BOX };

const EMPTY: Omit<SaleMoneyRow, 'kind' | 'date' | 'garden_credit_paisa' | 'working'> = {
  bikri_id: null,
  bikri_number: null,
  lot: null,
  item: null,
  buyer: null,
  shop_name: null,
  entry_id: null,
  qty: null,
  weight_grams: null,
  rate_paisa: null,
  value_paisa: null,
  buyer_total_paisa: null,
  paid_paisa: null,
  collected_paisa: null,
  still_due_paisa: null,
  charges: null,
};

export const MANI_ROW: SaleMoneyRow = {
  ...EMPTY,
  kind: 'bikri',
  date: '2026-10-10',
  bikri_id: 41,
  bikri_number: 5,
  lot: LOT,
  buyer: { id: 90, code: '0029', name_ur: 'مانی', name_en: 'Mani' },
  qty: 20,
  rate_paisa: money(1_000),
  value_paisa: money(20_000),
  buyer_total_paisa: money(21_360),
  paid_paisa: money(0),
  collected_paisa: money(0),
  still_due_paisa: money(21_360),
  garden_credit_paisa: money(18_800),
  working: {
    message_id: 'working-sale-money-credit',
    args: [arg('value', 20_000), arg('commission', 1_200), arg('result', 18_800)],
    rule_id: null,
  },
};

export const CHARGES_ROW: SaleMoneyRow = {
  ...EMPTY,
  kind: 'lot_charges',
  date: '2026-10-10',
  bikri_id: 41,
  bikri_number: 5,
  lot: LOT,
  charges: {
    freight_paisa: money(1_000),
    labour_paisa: money(250),
    munshiana_paisa: money(0),
    phone_paisa: money(0),
    store_rent_paisa: money(0),
  },
  garden_credit_paisa: money(-1_250),
  working: {
    message_id: 'working-sale-money-lot-charges',
    args: [
      arg('freight', 1_000),
      arg('labour', 250),
      arg('munshiana', 0),
      arg('phone', 0),
      arg('store_rent', 0),
      arg('result', 1_250),
    ],
    rule_id: null,
  },
};

export const OTHER_SHOP_ROW: SaleMoneyRow = {
  ...EMPTY,
  kind: 'other_shop',
  date: '2026-10-12',
  item: MANGO,
  shop_name: 'Haji Shop',
  entry_id: 14,
  qty: 20,
  value_paisa: money(10_000),
  garden_credit_paisa: money(10_000),
  working: {
    message_id: 'working-sale-money-other-shop',
    args: [arg('result', 10_000)],
    rule_id: null,
  },
};

export const SALE_MONEY_ROWS: readonly SaleMoneyRow[] = [MANI_ROW, CHARGES_ROW, OTHER_SHOP_ROW];

/** The plain number of an amount a row may not have. */
const plain = (paisa: Paisa | null) => (paisa === null ? 0 : plainPaisaForTest(paisa));

/** One page of `rows`, with the totals over all of them, as the backend answers. */
export function saleMoneyAnswer(
  rows: readonly SaleMoneyRow[],
  input: GardensSaleMoneyInput,
): GardensSaleMoney {
  const limit = input.limit === 0 ? 100 : input.limit;
  const sum = (pick: (row: SaleMoneyRow) => number) =>
    paisaForTest(rows.reduce((total, row) => total + pick(row), 0));
  return {
    rows: rows.slice(input.offset, input.offset + limit),
    total_count: rows.length,
    totals: {
      qty: rows.reduce((total, row) => total + (row.qty ?? 0), 0),
      value_paisa: sum((row) => plain(row.value_paisa)),
      buyer_total_paisa: sum((row) => plain(row.buyer_total_paisa)),
      paid_paisa: sum((row) => plain(row.paid_paisa)),
      collected_paisa: sum((row) => plain(row.collected_paisa)),
      still_due_paisa: sum((row) => plain(row.still_due_paisa)),
      lot_charges_paisa: sum((row) =>
        row.kind === 'lot_charges' ? -plain(row.garden_credit_paisa) : 0,
      ),
      garden_credit_paisa: sum((row) => plain(row.garden_credit_paisa)),
    },
  };
}
