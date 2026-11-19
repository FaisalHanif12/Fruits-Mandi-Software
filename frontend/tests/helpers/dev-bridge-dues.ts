// A stand-in for the Tauri bridge for the development pages /dev/dues/* (plan 79) only. In a plain browser tab there is no
// backend, so the pages behind the dashboard tiles cannot be looked at. This answers their commands with the numbers of the
// plan: the sellers (فیصل F, احسان, عدنان: the shop owes 202,780), the four parts of کل اگراہی, the people who
// borrowed, the open advance of رحیم and the cash sales of October 2026 (شاہد, چکتا, 12,800). Any other command
// is refused with ERR_NOT_IMPLEMENTED. Never part of a build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/dues/sellers?empty`:
//   empty   nothing on any list   down   every page fails   full   every section of کل اگراہی has someone

import type { SellerDuesPage } from '../../src/bindings/SellerDuesPage';
import type { SellerDuesRow } from '../../src/bindings/SellerDuesRow';
import { paisaForTest } from './paisa';
import { dashboardHandlers, inputOf, refusal } from './dev-bridge-dashboard';
import { devFlag } from './dev-bridge';
import { plan76Handlers } from './dev-bridge-plan76';
import { rokarHandlers } from './dev-bridge-rokar';
import {
  ADVANCES_OPEN,
  CASH_DAY,
  CASH_MONTH,
  DEAD,
  LENT_OPEN,
  SELLERS_ALL,
  SELLERS_NONE,
  TOTAL_DUES,
  TOTAL_DUES_FULL,
  party,
} from './dues-pages';

type Input = Record<string, unknown>;

const ZERO = paisaForTest(0);

/** In the `full` world one seller owes the shop 1,500 beyond his advances (the sellers section of کل اگراہی). */
const KARIM_OWES: SellerDuesRow = {
  party: party(14, '0014', 'کریم', 'Karim'),
  city: null,
  balance: { amount_paisa: paisaForTest(150_000), side: 'dr' },
  payable_paisa: ZERO,
  advance_open_paisa: ZERO,
  owes_paisa: paisaForTest(150_000),
  last_change_date: '2026-10-10',
};

function failIfDown(): void {
  if (devFlag('down')) throw refusal('ERR_INTERNAL');
}

function sellers(input: Input): SellerDuesPage {
  failIfDown();
  if (devFlag('empty')) return { ...SELLERS_NONE, shop_owes_paisa: ZERO };
  const partyId = typeof input['party_id'] === 'number' ? input['party_id'] : null;
  const side = input['side'];
  // Nobody owes the shop in the example, so that side is empty (but the `full` world has one seller who does);
  // the shop owes each of the three.
  if (side === 'seller_owes') {
    const rows = devFlag('full') ? [KARIM_OWES] : [];
    return { ...SELLERS_ALL, rows, total_count: rows.length };
  }
  const rows = SELLERS_ALL.rows.filter(
    (row) => partyId === null || row.party.account_id === partyId,
  );
  return { ...SELLERS_ALL, rows, total_count: rows.length };
}

const handlers: Record<string, (input: Input) => unknown> = {
  ...dashboardHandlers,
  dashboard_seller_dues: sellers,
  dashboard_total_dues: () => {
    failIfDown();
    return devFlag('full') ? TOTAL_DUES_FULL : TOTAL_DUES;
  },
  dashboard_buyer_dues: (input) => {
    failIfDown();
    const page = plan76Handlers['dashboard_buyer_dues']?.(input);
    return devFlag('empty') ? { rows: [], total_count: 0, total_paisa: ZERO } : page;
  },
  reports_dead_amount: () => {
    failIfDown();
    return devFlag('full') ? DEAD : { ...DEAD, rows: [], row_count: 0 };
  },
  dashboard_lent_open: () => {
    failIfDown();
    return devFlag('empty') && !devFlag('full')
      ? { rows: [], total_count: 0, total_paisa: ZERO, overdue_paisa: ZERO }
      : LENT_OPEN;
  },
  advances_report: () => {
    failIfDown();
    return devFlag('empty')
      ? { ...ADVANCES_OPEN, rows: [], total_count: 0, given_paisa: ZERO, open_paisa: ZERO }
      : ADVANCES_OPEN;
  },
  dashboard_cash_sales_month: (input) => {
    failIfDown();
    const month = typeof input['month'] === 'string' ? input['month'] : '2026-10';
    return devFlag('empty') || month !== '2026-10'
      ? { ...CASH_MONTH, month, rows: [], buyer_days_total: 0, total_paisa: ZERO }
      : CASH_MONTH;
  },
  dashboard_cash_sales_day: () => {
    failIfDown();
    return CASH_DAY;
  },
  // The seller box searches the accounts of the Rokar stand-in.
  search_accounts: (input) => rokarHandlers['search_accounts']?.(input),
  search_accounts_recent: (input) => rokarHandlers['search_accounts_recent']?.(input),
  search_accounts_touch: () => null,
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installDuesDevBridge(): void {
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
