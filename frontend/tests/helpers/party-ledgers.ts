// Test helper: sets the answers of the party ledgers commands (plan 66) from the backend's own answers
// in party-ledgers-real.ts. The wrappers are mocked in the test file with
// vi.mock('../../src/features/party_ledgers/api'). A list is answered the way the real command does
// it: the parties of the side asked for, narrowed by the text, sliced into the page asked for, with
// the same count and totals on every page. The totals of a narrowed list are the backend's, so the
// double only keeps the fixture's totals when nothing narrows it. A refusal is a rejected promise.

import { vi } from 'vitest';
import type { PartyLedgerRow } from '../../src/bindings/PartyLedgerRow';
import type { PartyLedgersList } from '../../src/bindings/PartyLedgersList';
import type { PartyLedgerStatement } from '../../src/bindings/PartyLedgerStatement';
import type { UiError } from '../../src/bindings/UiError';
import * as ledgersApi from '../../src/features/party_ledgers/api';
import { CommandError } from '../../src/shared/api';
import { paisaForTest } from './paisa';
import { sliceOf } from './paged';
import { fakeReports, type ReportsOptions } from './reports';
import { refusal } from './ui-error';
import {
  LEDGER_BUYERS,
  LEDGER_SELLERS,
  STATEMENT_AKRAM,
  STATEMENT_RAHIM,
} from './party-ledgers-real';

export * from './party-ledgers-real';

export interface FakeLedgers {
  buyers?: PartyLedgersList;
  sellers?: PartyLedgersList;
  /** The statement by account id; the default is the backend's own answer for 20 and 18. */
  statements?: Readonly<Record<number, PartyLedgerStatement>>;
  /** Refuses the statement of this call with this error. */
  refuseStatement?: (
    input: Parameters<typeof ledgersApi.partyLedgersStatement>[0],
  ) => UiError | null;
  /** Fails the list call with this error. */
  failList?: UiError;
  /** What the report commands answer (level 3 is the R35 slip). */
  reports?: ReportsOptions;
}

/** The text a party is found by: the code and both names, in lower case. */
function haystack(row: PartyLedgerRow): string {
  return `${row.code} ${row.name_ur} ${row.name_en} ${row.tag ?? ''}`.toLowerCase();
}

/**
 * A list narrowed by text. A test double adds no money (F3): with one party left, his balance is the
 * total of his side; with none, both totals are 0; with several, the totals stay the fixture's.
 */
function narrowed(list: PartyLedgersList, text: string | null): PartyLedgersList {
  if (text === null || text.trim() === '') return list;
  const wanted = text.trim().toLowerCase();
  const rows = list.rows.filter((row) => haystack(row).includes(wanted));
  const zero = paisaForTest(0);
  const only = rows.length === 1 ? rows[0] : undefined;
  if (rows.length > 1) return { ...list, rows, count: rows.length };
  return {
    rows,
    count: rows.length,
    total_dr_paisa: only?.side === 'dr' ? only.balance_paisa : zero,
    total_cr_paisa: only?.side === 'cr' ? only.balance_paisa : zero,
  };
}

/** Sets the answers of the party ledgers commands and of the report commands behind level 3. */
export function fakePartyLedgers(options: FakeLedgers = {}): void {
  const buyers = options.buyers ?? LEDGER_BUYERS;
  const sellers = options.sellers ?? LEDGER_SELLERS;
  const statements = options.statements ?? { 20: STATEMENT_AKRAM, 18: STATEMENT_RAHIM };
  vi.mocked(ledgersApi.partyLedgersList).mockImplementation((input) => {
    if (options.failList !== undefined) return Promise.reject(new CommandError(options.failList));
    const all = narrowed(input.side === 'buyer' ? buyers : sellers, input.text);
    return Promise.resolve({ ...all, rows: sliceOf(all.rows, input.offset, input.limit) });
  });
  vi.mocked(ledgersApi.partyLedgersStatement).mockImplementation((input) => {
    const refused = options.refuseStatement?.(input) ?? null;
    if (refused !== null) return Promise.reject(new CommandError(refused));
    const found = statements[input.account_id];
    if (found === undefined) {
      return Promise.reject(
        new CommandError(refusal([{ code: 'ERR_NOT_FOUND', field: 'account_id' }])),
      );
    }
    return Promise.resolve({
      ...found,
      rows: sliceOf(found.rows, input.offset, input.limit),
    });
  });
  fakeReports(options.reports);
}
