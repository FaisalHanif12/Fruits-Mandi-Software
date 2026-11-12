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
import type { ReportPage } from '../../src/bindings/ReportPage';
import * as reportsApi from '../../src/features/reports/api';
import type { UiError } from '../../src/bindings/UiError';
import * as ledgersApi from '../../src/features/party_ledgers/api';
import type { PartyLedgerGoTo } from '../../src/bindings/PartyLedgerGoTo';
import type { PartyLedgerGoToInput } from '../../src/bindings/PartyLedgerGoToInput';
import { CommandError } from '../../src/shared/api';
import { addDays } from '../../src/shared/format';
import { DAY } from './app';
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
  /**
   * The backend's totals of one window or range (plan 74 W3): a double adds no money (F3), so a test
   * that shows a window other than the whole statement gives the opening, sums and closing it wants
   * to see. Without it the totals of the fixture stay, whatever rows the dates leave.
   */
  totalsOf?: (
    input: Parameters<typeof ledgersApi.partyLedgersStatement>[0],
  ) => Partial<PartyLedgerStatement> | undefined;
  /**
   * "Go to a date" (plan 74 W5). The default double answers like the backend: the 30 days that end on
   * the earlier of the date plus 15 days and today (10-10-2026), with an entry when the party's rows
   * hold that date.
   */
  goTo?: (input: PartyLedgerGoToInput) => PartyLedgerGoTo | UiError;
  /**
   * The R35 slip of one party on one date (`YYYY-MM-DD`), when a test opens several days and each must
   * show its own figures (plan 74). Without it the report double answers one slip for every date.
   */
  slipFor?: (accountId: number, date: string) => ReportPage | undefined;
  /** What the report commands answer (level 3 is the R35 slip). */
  reports?: ReportsOptions;
}

/** Today in the tests, `YYYY-MM-DD`. */
const TODAY = DAY.today;

/** A typed date `DD-MM-YYYY` as `YYYY-MM-DD`, or null for a box left empty. */
function isoOfTyped(text: string | null): string | null {
  if (text === null) return null;
  const [day = '', month = '', year = ''] = text.split('-');
  return `${year}-${month}-${day}`;
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
  /** The rows of an account inside the dates asked for, as the backend reads a range (a window is one). */
  const statementFor = (input: Parameters<typeof ledgersApi.partyLedgersStatement>[0]) => {
    const found = statements[input.account_id];
    if (found === undefined) return undefined;
    const first = isoOfTyped(input.from);
    const last = isoOfTyped(input.to);
    const inside = found.rows.filter(
      (row) => (first === null || row.date >= first) && (last === null || row.date <= last),
    );
    return {
      ...found,
      ...options.totalsOf?.(input),
      rows: sliceOf(inside, input.offset, input.limit),
      count: inside.length,
    };
  };
  vi.mocked(ledgersApi.partyLedgersStatement).mockImplementation((input) => {
    const refused = options.refuseStatement?.(input) ?? null;
    if (refused !== null) return Promise.reject(new CommandError(refused));
    const answer = statementFor(input);
    if (answer === undefined) {
      return Promise.reject(
        new CommandError(refusal([{ code: 'ERR_NOT_FOUND', field: 'account_id' }])),
      );
    }
    return Promise.resolve(answer);
  });
  vi.mocked(ledgersApi.partyLedgersGoToDate).mockImplementation((input) => {
    const custom = options.goTo?.(input);
    if (custom !== undefined) {
      return 'errors' in custom
        ? Promise.reject(new CommandError(custom))
        : Promise.resolve(custom);
    }
    const date = isoOfTyped(input.date) ?? '';
    const to = addDays(date, 15) < TODAY ? addDays(date, 15) : TODAY;
    const from = addDays(to, -29);
    const typed = (iso: string) => `${iso.slice(8)}-${iso.slice(5, 7)}-${iso.slice(0, 4)}`;
    const statement = statementFor({
      account_id: input.account_id,
      from: typed(from),
      to: typed(to),
      offset: 0,
      limit: 100,
    });
    if (statement === undefined) {
      return Promise.reject(
        new CommandError(refusal([{ code: 'ERR_NOT_FOUND', field: 'account_id' }])),
      );
    }
    const place = statement.rows.findIndex((row) => row.date === date);
    return Promise.resolve({
      from,
      to,
      statement,
      has_entry: place >= 0,
      row: place >= 0 ? place : null,
    });
  });
  fakeReports(options.reports);
  const slipFor = options.slipFor;
  if (slipFor !== undefined) {
    const answerOf = vi.mocked(reportsApi.reportsRun).getMockImplementation();
    vi.mocked(reportsApi.reportsRun).mockImplementation((input) => {
      const party = input.filters.party;
      const typed = input.filters.as_of;
      if (input.id === 'R35' && party !== null && typed !== '') {
        const slip = slipFor(party.account_id, isoOfTyped(typed) ?? '');
        if (slip !== undefined) return Promise.resolve(slip);
      }
      if (answerOf === undefined) throw new Error('the report double is not set');
      return answerOf(input);
    });
  }
}
