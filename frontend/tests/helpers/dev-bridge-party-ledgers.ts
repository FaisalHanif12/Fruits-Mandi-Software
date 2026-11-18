// A stand-in for the Tauri bridge for the development page /dev/party-ledgers (plan 66): the party
// ledgers screen (the list, a party's statement, the day slip and its print) with the backend's own
// answers of the worked day (party-ledgers-real.ts, the R35 slips of the reports stand-in), so it
// can be opened in a plain browser tab and photographed in both languages and at every window size.
// Every other command is answered by the reports stand-in or refused with ERR_NOT_IMPLEMENTED.
// Never part of a build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/party-ledgers?party=21&date=2026-10-10`:
//   side=seller  the sellers       empty  no parties          many  230 parties (page by page)
//   dead         ارشد is in the dead amount        down  the backend fails
//   real         the statement of احسان (seller, party=35) as the backend writes it for the worked day: the opening
//                balance first, then his بکری and his ادائیگی (plan 78 A7); today is 31-10-2026
//   later        with `real`: today is 09-11-2026, so the window starts on 11-10-2026 and opens with the balance brought forward
//   four         the statement of اکرم with five entries on four dates in two windows of 30 days (plan 74, plan 78); today is 10-10-2026
// The screen's own address (`side`, `party`, `date`) is read from the same hash.

import type { PartyLedgerRow } from '../../src/bindings/PartyLedgerRow';
import type { PartyLedgersList } from '../../src/bindings/PartyLedgersList';
import { devFlag } from './dev-bridge';
import { installReportsDevBridge, page as reportPage } from './dev-bridge-reports';
import { addDays } from '../../src/shared/format';
import { STATEMENT_ROWS_FROM_11_10, STATEMENT_ROWS_WHOLE } from './plan78-real';
import { paisaForTest } from './paisa';
import {
  AKRAM_STATEMENT,
  BUYER_STATEMENT,
  HEAD_AKRAM,
  SELLER_STATEMENT,
  windowTotals,
} from './party-statement-rows';
import { LEDGER_BUYERS, LEDGER_SELLERS, STATEMENT_AKRAM } from './party-ledgers-real';

type Input = Record<string, unknown>;

/** The list as the flags choose it: the real answer, a dead person, nobody, or a long list. */
function listFor(input: Input): PartyLedgersList {
  const real = input['side'] === 'seller' ? LEDGER_SELLERS : LEDGER_BUYERS;
  if (devFlag('empty')) {
    return { rows: [], count: 0, total_dr_paisa: paisaForTest(0), total_cr_paisa: paisaForTest(0) };
  }
  let rows: readonly PartyLedgerRow[] = real.rows;
  let count = real.count;
  if (devFlag('dead')) {
    rows = rows.map((row) =>
      row.account_id === 22 ? { ...row, is_dead: true, is_active: false } : row,
    );
  }
  const firstRow = rows[0];
  if (devFlag('many') && firstRow !== undefined) {
    // A long list to page through: the same row again with its own id and code. The footer stays the
    // real totals; this only shows the pager, no money is worked out.
    rows = Array.from({ length: 230 }, (_, index) => ({
      ...firstRow,
      account_id: 1000 + index,
      code: `00${String(2000 + index)}`,
    }));
    count = 230;
  }
  const offset = Number(input['offset'] ?? 0);
  const limit = Number(input['limit'] ?? 100) || 100;
  return { ...real, rows: rows.slice(offset, offset + limit), count };
}

// The flag is read once, when the stand-in is installed: moving the window changes the address, and
// the flags in the hash go with it.
let fourDates = false;

const handlers: Record<string, (input: Input) => unknown> = {
  // Today is 10-10-2026: the first window of a statement ends on it (plan 74).
  day_status: () => ({
    // `real`: the 30 days that end on 31-10-2026 hold the whole of احسان's statement (the opening, the بکری and the payment).
    today: devFlag('later') ? '2026-11-09' : devFlag('real') ? '2026-10-31' : '2026-10-10',
    closed_up_to: null,
    today_open: true,
  }),
  party_ledgers_list: listFor,
  party_ledgers_statement_rows: (input) => {
    if (devFlag('real') && input['account_id'] === STATEMENT_ROWS_WHOLE.head.account_id) {
      // The backend's own answers: the whole period (its opening balance is the first row), or a window that
      // starts on 11-10-2026 (the balance of the day before is brought forward).
      const from = typeof input['from'] === 'string' ? input['from'] : '';
      const [day = '', month = '', year = ''] = from.split('-');
      return `${year}-${month}-${day}` > '2026-10-10'
        ? STATEMENT_ROWS_FROM_11_10
        : STATEMENT_ROWS_WHOLE;
    }
    const four = fourDates && input['account_id'] === 21;
    const found = four
      ? { ...BUYER_STATEMENT, head: HEAD_AKRAM }
      : input['account_id'] === 19
        ? SELLER_STATEMENT
        : AKRAM_STATEMENT;
    // The dates asked for as typed, `DD-MM-YYYY`: the rows inside them, as the backend reads a range.
    const iso = (text: unknown) => {
      if (typeof text !== 'string') return null;
      const [day = '', month = '', year = ''] = text.split('-');
      return `${year}-${month}-${day}`;
    };
    const first = iso(input['from']);
    const last = iso(input['to']);
    const history = input['include_history'] === true;
    const inside = found.rows.filter(
      (row) =>
        (first === null || row.date >= first) &&
        (last === null || row.date <= last) &&
        (history || !row.is_replaced),
    );
    const offset = Number(input['offset'] ?? 0);
    const limit = Number(input['limit'] ?? 100) || 100;
    const totals = four
      ? windowTotals({ from: typeof input['from'] === 'string' ? input['from'] : undefined })
      : undefined;
    return {
      ...found,
      ...totals,
      rows: inside.slice(offset, offset + limit),
      count: inside.length,
    };
  },
  // The 30 days that end on the earlier of the date plus 15 days and today (10-10-2026), with an entry
  // when the party has one that day, as the backend's command says.
  reports_run: (input) => reportPage(input),
  party_ledgers_go_to_date: (input) => {
    const [day = '', month = '', year = ''] = String(input['date']).split('-');
    const date = `${year}-${month}-${day}`;
    const plus = addDays(date, 15);
    const to = plus < '2026-10-10' ? plus : '2026-10-10';
    const from = addDays(to, -29);
    const rows = BUYER_STATEMENT.rows.filter((row) => row.date >= from && row.date <= to);
    const place = rows.findIndex((row) => row.date === date);
    return {
      from,
      to,
      // The answer still carries the old window page (the shape of plan 74); the screen ignores it.
      statement: STATEMENT_AKRAM,
      has_entry: place >= 0,
      row: place >= 0 ? place : null,
    };
  },
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installPartyLedgersDevBridge(): void {
  fourDates = devFlag('four');
  installReportsDevBridge(handlers);
}
