// A stand-in for the Tauri bridge for the development page /dev/party-ledgers (plan 66): the party
// ledgers screen (the list, a party's statement, the day slip and its print) with the backend's own
// answers of the worked day (party-ledgers-real.ts, the R35 slips of the reports stand-in), so it
// can be opened in a plain browser tab and photographed in both languages and at every window size.
// Every other command is answered by the reports stand-in or refused with ERR_NOT_IMPLEMENTED.
// Never part of a build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/party-ledgers?party=20&date=2026-10-10`:
//   side=seller  the sellers       empty  no parties          many  230 parties (page by page)
//   dead         ارشد is in the dead amount        down  the backend fails
// The screen's own address (`side`, `party`, `date`) is read from the same hash.

import type { PartyLedgerRow } from '../../src/bindings/PartyLedgerRow';
import type { PartyLedgersList } from '../../src/bindings/PartyLedgersList';
import { devFlag } from './dev-bridge';
import { installReportsDevBridge } from './dev-bridge-reports';
import { paisaForTest } from './paisa';
import {
  LEDGER_BUYERS,
  LEDGER_SELLERS,
  STATEMENT_AKRAM,
  STATEMENT_RAHIM,
} from './party-ledgers-real';

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
      row.account_id === 21 ? { ...row, is_dead: true, is_active: false } : row,
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

const handlers: Record<string, (input: Input) => unknown> = {
  party_ledgers_list: listFor,
  party_ledgers_statement: (input) => {
    const found = input['account_id'] === 18 ? STATEMENT_RAHIM : STATEMENT_AKRAM;
    const offset = Number(input['offset'] ?? 0);
    const limit = Number(input['limit'] ?? 100) || 100;
    return { ...found, rows: found.rows.slice(offset, offset + limit) };
  },
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installPartyLedgersDevBridge(): void {
  installReportsDevBridge(handlers);
}
