// The screen and the backend agree on the party ledgers (plan 66, PRD FR-12.43): each wrapper sends
// the command name the backend registers, with the input in the shape `{ input }` the backend's own
// requests have, and the copies of the backend's fixtures are its bytes. The names are also read
// from the backend's own command file, so a command renamed there fails here first. Level 3 has no
// command of its own: it is the R35 report for one party and one date, so its filters are checked
// against the request the report screen's own fixture holds.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { partyLedgersList, partyLedgersStatement } from '../../src/features/party_ledgers/api';
import { parseLedgerSearch } from '../../src/features/party_ledgers/search';
import { partyDayFilters } from '../../src/features/reports/party-day';
import * as invoke from '../../src/shared/api/invoke';
import {
  LEDGER_BUYERS_REQUEST,
  LEDGER_SELLERS_REQUEST,
  STATEMENT_AKRAM_REQUEST,
  STATEMENT_RAHIM_REQUEST,
} from '../helpers/party-ledgers-real';
import { R35_SLIP_AKRAM_REQUEST, R35_SLIP_RAHIM_REQUEST } from '../helpers/reports-party-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue({});
});

const COMMANDS = path.resolve(__dirname, '../../../backend/app/src/commands');

describe('the commands of the party ledgers are the ones the backend declares', () => {
  it('both commands are in commands/party_ledgers.rs', () => {
    const text = readFileSync(path.join(COMMANDS, 'party_ledgers.rs'), 'utf8');
    const names = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    expect(names).toEqual(['party_ledgers_list', 'party_ledgers_statement']);
  });
});

describe('the copies of the backend fixtures are the bytes the backend wrote', () => {
  const SOURCE = path.resolve(__dirname, '../../../backend/crates/services/tests/fixtures');
  const COPIES = path.resolve(__dirname, '../fixtures');
  const names = readdirSync(SOURCE).filter(
    (name) => name.endsWith('.json') && name.startsWith('party_ledgers_'),
  );

  it('there are fixtures for both commands', () => {
    expect(names.length).toBeGreaterThanOrEqual(8);
  });

  it.each(names)('%s is the same file', (name) => {
    expect(readFileSync(path.join(COPIES, name), 'utf8')).toBe(
      readFileSync(path.join(SOURCE, name), 'utf8'),
    );
  });
});

describe('the wrappers send what the backend reads', () => {
  it('the list sends the side, the text and the page', async () => {
    await partyLedgersList(LEDGER_BUYERS_REQUEST);
    expect(call).toHaveBeenCalledWith('party_ledgers_list', { input: LEDGER_BUYERS_REQUEST });
    await partyLedgersList(LEDGER_SELLERS_REQUEST);
    expect(call).toHaveBeenLastCalledWith('party_ledgers_list', { input: LEDGER_SELLERS_REQUEST });
  });

  it('the statement sends the account, the two dates and the page', async () => {
    await partyLedgersStatement(STATEMENT_AKRAM_REQUEST);
    expect(call).toHaveBeenCalledWith('party_ledgers_statement', {
      input: STATEMENT_AKRAM_REQUEST,
    });
    await partyLedgersStatement(STATEMENT_RAHIM_REQUEST);
    expect(call).toHaveBeenLastCalledWith('party_ledgers_statement', {
      input: STATEMENT_RAHIM_REQUEST,
    });
  });
});

describe('level 3 is the R35 slip of one party and one date', () => {
  it('the filters are the report screen’s own request for the buyer’s and the seller’s slip', () => {
    expect(partyDayFilters(20, '2026-10-10')).toEqual(R35_SLIP_AKRAM_REQUEST.filters);
    expect(partyDayFilters(18, '2026-10-10')).toEqual(R35_SLIP_RAHIM_REQUEST.filters);
  });

  it('the slip on the screen adds the language that sorts a seller’s lots, and nothing else (plan 71 L7)', () => {
    expect(partyDayFilters(18, '2026-10-10', 'ur')).toEqual({
      ...R35_SLIP_RAHIM_REQUEST.filters,
      order_language: 'ur',
    });
    expect(partyDayFilters(18, '2026-10-10', 'en')).toMatchObject({ order_language: 'en' });
  });
});

describe('the address of the screen', () => {
  it('reads the side, the party and the date, and drops what does not read', () => {
    expect(parseLedgerSearch({ side: 'seller', party: '18', date: '2026-10-10' })).toEqual({
      side: 'seller',
      party: 18,
      date: '2026-10-10',
    });
    expect(parseLedgerSearch({ side: 'dealer', party: 'x', date: '2026-10-10' })).toEqual({});
    // A date alone, or a date that is not a date, shows the level above.
    expect(parseLedgerSearch({ party: 20, date: '10-10-2026' })).toEqual({ party: 20 });
  });
});
