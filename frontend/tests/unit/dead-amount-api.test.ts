// The screens and the backend agree on the dead amount (plan 40, PRD FR-10.10): each wrapper sends the
// command name the backend registers, with the input in the shape `{ input }`, and the picker search
// asks for dead people only when its field says so (the receipt and ادھار رقم واپسی fields). The names
// are also read from the backend's own command file, so a command renamed there fails here first.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { deadMark, deadPreview, deadUndo } from '../../src/features/masters/api';
import { reportsDeadAmount } from '../../src/features/reports/api';
import { searchAccounts, searchAccountsRecent } from '../../src/shared/accounts/api';
import * as invoke from '../../src/shared/api/invoke';
import { searchPukhtaPeople } from '../../src/features/pukhta_khata/api';
import {
  PEOPLE_DEAD_HIDDEN_REQUEST,
  PEOPLE_INCLUDE_DEAD_REQUEST,
  SEARCH_INCLUDE_DEAD_REQUEST,
} from '../helpers/dead-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue([]);
});

const COMMANDS = path.resolve(__dirname, '../../../backend/app/src/commands');

describe('the commands of the dead amount are the ones the backend declares', () => {
  it('the four commands are in commands/dead.rs', () => {
    const text = readFileSync(path.join(COMMANDS, 'dead.rs'), 'utf8');
    const names = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    expect(names).toEqual(
      expect.arrayContaining([
        'masters_dead_preview',
        'masters_dead_mark',
        'masters_dead_undo',
        'reports_dead_amount',
      ]),
    );
  });
});

describe('the copies of the backend fixtures are the bytes the backend wrote', () => {
  const SOURCE = path.resolve(__dirname, '../../../backend/crates/services/tests/fixtures');
  const COPIES = path.resolve(__dirname, '../fixtures');
  const names = readdirSync(SOURCE).filter(
    (name) =>
      name.endsWith('.json') &&
      [
        'dead_',
        'masters_dead_',
        'reports_dead_',
        'search_accounts_include_dead',
        'search_pukhta_people_',
      ].some((prefix) => name.startsWith(prefix)),
  );

  it('there are fixtures for the four commands', () => {
    expect(names.length).toBeGreaterThanOrEqual(20);
  });

  it.each(names)('%s is the same file', (name) => {
    expect(readFileSync(path.join(COPIES, name), 'utf8')).toBe(
      readFileSync(path.join(SOURCE, name), 'utf8'),
    );
  });
});

describe('the wrappers send what the backend reads', () => {
  it('the preview sends the account and, for a one-time person, his key', async () => {
    await deadPreview({ account_id: 22, pukhta_person_key: null });
    expect(call).toHaveBeenCalledWith('masters_dead_preview', {
      input: { account_id: 22, pukhta_person_key: null },
    });
  });

  it('the move sends the date as typed, the reason and the note', async () => {
    await deadMark({
      account_id: 22,
      pukhta_person_key: null,
      entry_date: '11-10-2026',
      reason: 'died',
      note: '',
    });
    expect(call).toHaveBeenCalledWith('masters_dead_mark', {
      input: {
        account_id: 22,
        pukhta_person_key: null,
        entry_date: '11-10-2026',
        reason: 'died',
        note: '',
      },
    });
  });

  it('the undo sends the move and the reason', async () => {
    await deadUndo({ mark_id: 7, reason: 'Paid in full' });
    expect(call).toHaveBeenCalledWith('masters_dead_undo', {
      input: { mark_id: 7, reason: 'Paid in full' },
    });
  });

  it('R38 sends the filter, the search and the page', async () => {
    await reportsDeadAmount({ filter: 'owed', search: '', offset: 0, limit: 100 });
    expect(call).toHaveBeenCalledWith('reports_dead_amount', {
      input: { filter: 'owed', search: '', offset: 0, limit: 100 },
    });
  });
});

describe('the picker search asks for dead people only when its field says so', () => {
  it('the receipt search sends exactly the request the backend sent in its own test', async () => {
    await searchAccounts({
      scope: 'buyer',
      controlId: null,
      text: 'arshad',
      limit: 10,
      includeDead: true,
    });
    const sent = call.mock.calls[0]?.[1] as { input: Record<string, unknown> };
    expect(sent.input).toEqual(SEARCH_INCLUDE_DEAD_REQUEST);
  });

  it('a plain search sends no include_dead', async () => {
    await searchAccounts({ scope: 'buyer', controlId: null, text: 'ak', limit: 20 });
    const input = call.mock.calls[0]?.[1] as { input: Record<string, unknown> };
    expect(input.input).not.toHaveProperty('include_dead');
  });

  it('the receipt field sends include_dead: true, in the search and in the recent list', async () => {
    await searchAccounts({
      scope: 'buyer',
      controlId: null,
      text: 'ar',
      limit: 20,
      includeDead: true,
    });
    await searchAccountsRecent('buyer', null, true);
    const search = call.mock.calls[0]?.[1] as { input: Record<string, unknown> };
    const recent = call.mock.calls[1]?.[1] as { input: Record<string, unknown> };
    expect(search.input['include_dead']).toBe(true);
    expect(recent.input['include_dead']).toBe(true);
  });
});

describe('the one-time people search asks for dead people only on the receipt screen', () => {
  it('the receipt sends exactly the request the backend sent in its own test', async () => {
    await searchPukhtaPeople('nasir', 'buyer', true);
    const sent = call.mock.calls[0]?.[1] as { input: Record<string, unknown> };
    expect(sent.input).toEqual(PEOPLE_INCLUDE_DEAD_REQUEST);
  });

  it('every other field sends no include_dead, like the backend test of the hidden case', async () => {
    await searchPukhtaPeople('nasir', 'buyer');
    const sent = call.mock.calls[0]?.[1] as { input: Record<string, unknown> };
    expect(sent.input).toEqual(PEOPLE_DEAD_HIDDEN_REQUEST);
  });
});
