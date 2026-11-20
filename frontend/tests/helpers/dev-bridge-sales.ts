// A stand-in for the Tauri bridge for the development page /dev/sales (plan 83) only. In a plain browser tab there is
// no backend, so the sales page cannot be looked at. This answers its commands with the numbers of the plan (after
// Events 1 to 5, 10-10-26: 2 بکری, 60 نگ, خام بکری 41,400, the buyers' bill 44,364, نقدی 12,800, ادھاری 31,564, and two
// earlier days that are made up), for a month, a date, a year and a range, and the account pickers of the party box.
// Any other command is refused with ERR_NOT_IMPLEMENTED. Never part of a build: only the development route imports
// it.
//
// Flags in the hash, for example `#/dev/sales?empty`:
//   empty   no sales at all   down   the sales call fails

import type { LotName } from '../../src/bindings/LotName';
import type { SalesDay } from '../../src/bindings/SalesDay';
import type { SalesPeriod } from '../../src/bindings/SalesPeriod';
import type { SalesPeriodRow } from '../../src/bindings/SalesPeriodRow';
import { dashboardHandlers, inputOf, refusal } from './dev-bridge-dashboard';
import { devFlag } from './dev-bridge';
import { rokarHandlers } from './dev-bridge-rokar';
import { paisaForTest } from './paisa';

type Input = Record<string, unknown>;

const rupees = (value: number) => paisaForTest(value * 100);

const ref = (id: number, code: string, ur: string, en: string) => ({
  id,
  code,
  name_ur: ur,
  name_en: en,
});
const RAHIM = ref(19, '0010', 'رحیم', 'Rahim');
const KARIM = ref(20, '0011', 'کریم', 'Karim');
const BANANA = ref(4, '0040', 'کیلا', 'Banana');
const APPLE = ref(5, '0041', 'سیب', 'Apple');

const row = (
  date: string,
  count: number,
  qty: number,
  gross: number,
  buyerTotal: number,
  cash: number,
  credit: number,
): SalesPeriodRow => ({
  date,
  bikri_count: count,
  qty,
  gross_paisa: rupees(gross),
  buyer_total_paisa: rupees(buyerTotal),
  cash_paisa: rupees(cash),
  credit_paisa: rupees(credit),
});

/** Three days of October: the 10th is the day of the plan; the 8th and 9th are made up. */
const DAYS = [
  row('2026-10-08', 1, 30, 18_000, 19_080, 4_000, 15_080),
  row('2026-10-09', 1, 25, 15_000, 15_900, 0, 15_900),
  row('2026-10-10', 2, 60, 41_400, 44_364, 12_800, 31_564),
];
/** Their totals, written out by hand (the stand-in adds nothing up, like the page). */
const TOTALS = row('', 4, 115, 74_400, 79_344, 16_800, 62_544);
const NONE = row('', 0, 0, 0, 0, 0, 0);

const lot = (id: number, seller: typeof RAHIM, item: typeof BANANA, counter: number): LotName => ({
  lot_id: id,
  seller,
  item,
  counter,
});

function period(input: Input): SalesPeriod {
  if (devFlag('down')) throw refusal('ERR_INTERNAL');
  const asked = input['period'] as { kind?: string; date?: string; month?: string } | undefined;
  const kind = asked?.kind ?? 'month';
  const empty = devFlag('empty');
  let rows: SalesPeriodRow[];
  if (kind === 'day') rows = [{ ...DAYS[2], date: asked?.date ?? '2026-10-10' } as SalesPeriodRow];
  else if (kind === 'year')
    rows = DAYS.map((day, index) => ({
      ...day,
      date: ['2026-08', '2026-09', '2026-10'][index] ?? '',
    }));
  else rows = DAYS;
  if (empty) rows = [];
  const offset = typeof input['offset'] === 'number' ? input['offset'] : 0;
  return {
    month: asked?.month ?? '2026-10',
    year: 2026,
    period: { kind: kind as 'month', from: '2026-10-01', to: '2026-10-31' },
    party: null,
    item: null,
    goods: 'all',
    first_month: '2026-08',
    current_month: '2026-10',
    today: '2026-10-10',
    rows,
    total_count: rows.length,
    offset,
    limit: 100,
    totals: empty ? NONE : kind === 'day' ? ({ ...DAYS[2], date: '' } as SalesPeriodRow) : TOTALS,
  };
}

function day(input: Input): SalesDay {
  if (devFlag('down')) throw refusal('ERR_INTERNAL');
  const date = typeof input['date'] === 'string' ? input['date'] : '2026-10-10';
  return {
    date,
    party: null,
    item: null,
    goods: 'all',
    rows: [
      {
        bikri_id: 41,
        bikri_number: 1,
        sellers: [RAHIM],
        lots: [lot(7, RAHIM, BANANA, 1)],
        qty: 50,
        gross_paisa: rupees(29_400),
        buyer_total_paisa: rupees(31_564),
        cash_paisa: rupees(0),
        credit_paisa: rupees(31_564),
      },
      {
        bikri_id: 42,
        bikri_number: 2,
        sellers: [KARIM],
        lots: [lot(8, KARIM, APPLE, 1)],
        qty: 10,
        gross_paisa: rupees(12_000),
        buyer_total_paisa: rupees(12_800),
        cash_paisa: rupees(12_800),
        credit_paisa: rupees(0),
      },
    ],
    total_count: 2,
    totals: { ...DAYS[2], date } as SalesPeriodRow,
  };
}

const handlers: Record<string, (input: Input) => unknown> = {
  ...dashboardHandlers,
  dashboard_sales_period: period,
  dashboard_sales_day: day,
  // The party box searches the accounts of the Rokar stand-in; the item box finds nothing here.
  search_accounts: (input) => rokarHandlers['search_accounts']?.(input),
  search_accounts_recent: (input) => rokarHandlers['search_accounts_recent']?.(input),
  search_accounts_touch: () => null,
  search_masters: () => [],
  search_recent: () => [],
  search_touch: () => null,
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installSalesDevBridge(): void {
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
