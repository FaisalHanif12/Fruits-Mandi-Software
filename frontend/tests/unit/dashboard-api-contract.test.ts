// The screens and the backend agree on the dashboard, the attention list and what a screen
// remembers (round 9, PRD Section 9, FR-7.44, FR-7.45): each wrapper sends the command name the
// backend registers, with the input in the shape `{ input }` (the dashboard and the attention list
// send only their name). The names are also read from the backend's own command files, so a command
// renamed there fails here first. The fixtures in tests/fixtures are the backend's own, copied (the
// same bytes), and the dashboard of the worked day carries the numbers of PRD 4.9.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { assistAttention, dashboardGet } from '../../src/features/dashboard/api';
import { prefsScreenGet, prefsScreenSet } from '../../src/shared/screen-state/api';
import * as invoke from '../../src/shared/api/invoke';
import {
  ATTENTION_18_10,
  DASHBOARD_WORKED_DAY,
  PREFS_SCREEN_SET,
  PREFS_SCREEN_SET_REQUEST,
} from '../helpers/dashboard-real';
import { paisaForTest } from '../helpers/paisa';

vi.mock('../../src/shared/api/invoke');
// The setup file stands in for the screen-state commands in every test; this test is about the real
// wrappers, so it asks for them.
vi.unmock('../../src/shared/screen-state/api');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const COMMANDS = path.resolve(__dirname, '../../../backend/app/src/commands');
const BACKEND_FIXTURES = path.resolve(__dirname, '../../../backend/crates/services/tests/fixtures');
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

function declared(file: string): string[] {
  const text = readFileSync(path.join(COMMANDS, file), 'utf8');
  return [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
}

describe('the commands the screens call are the ones the backend declares', () => {
  it('dashboard_get and assist_attention are in commands/dashboard.rs', () => {
    expect(declared('dashboard.rs')).toEqual(
      expect.arrayContaining(['dashboard_get', 'assist_attention']),
    );
  });

  it('prefs_screen_get and prefs_screen_set are in commands/screen_prefs.rs', () => {
    expect(declared('screen_prefs.rs')).toEqual(
      expect.arrayContaining(['prefs_screen_get', 'prefs_screen_set']),
    );
  });
});

describe('the wrappers send what the backend reads', () => {
  it('the dashboard and the attention list send only the command name', async () => {
    await dashboardGet();
    await assistAttention();
    expect(call).toHaveBeenNthCalledWith(1, 'dashboard_get');
    expect(call).toHaveBeenNthCalledWith(2, 'assist_attention');
  });

  it('a screen asks and saves with `{ input }`, the shape of the backend request', async () => {
    await prefsScreenGet({ screen: 'report.r20' });
    await prefsScreenSet(PREFS_SCREEN_SET_REQUEST);
    expect(call).toHaveBeenNthCalledWith(1, 'prefs_screen_get', {
      input: { screen: 'report.r20' },
    });
    expect(call).toHaveBeenNthCalledWith(2, 'prefs_screen_set', {
      input: PREFS_SCREEN_SET_REQUEST,
    });
    // The request of the backend's own test is the screen `report.r20` and its JSON text.
    expect(PREFS_SCREEN_SET_REQUEST).toEqual({ screen: 'report.r20', value: '{"city_id":3}' });
    expect(PREFS_SCREEN_SET).toEqual({ screen: 'report.r20', value: '{"city_id":3}' });
  });
});

describe('the fixtures are the backend’s own', () => {
  it('each copy has the same bytes as its source', () => {
    for (const name of [
      'dashboard_get_worked_day.json',
      'dashboard_get_worked_day_request.json',
      'assist_attention_18_10.json',
      'assist_attention_18_10_request.json',
      'prefs_screen_set.json',
      'prefs_screen_set_request.json',
    ]) {
      const source = readFileSync(path.join(BACKEND_FIXTURES, name));
      const copy = readFileSync(path.join(FRONTEND_FIXTURES, name));
      expect(copy.equals(source), name).toBe(true);
    }
  });

  it('the dashboard of the worked day carries the numbers of PRD 4.9', () => {
    const tiles = DASHBOARD_WORKED_DAY.tiles;
    // Money is paisa on the wire: 41,400 rupees is 4,140,000 paisa.
    expect(tiles.sales_gross_paisa).toBe(paisaForTest(4_140_000));
    expect(tiles.sales_count).toBe(2);
    expect(tiles.cash_in_hand_paisa).toBe(paisaForTest(5_848_400));
    expect(tiles.buyers_dues_paisa).toBe(paisaForTest(2_156_400));
    expect(tiles.sellers_payable_paisa).toBe(paisaForTest(1_016_000));
    expect(tiles.advances_open_paisa).toBe(paisaForTest(0));
    expect(DASHBOARD_WORKED_DAY.partners.map((partner) => partner.today_paisa)).toEqual([
      paisaForTest(99_600),
      paisaForTest(74_600),
      paisaForTest(74_600),
    ]);
    expect(DASHBOARD_WORKED_DAY.chart_days).toHaveLength(30);
    expect(DASHBOARD_WORKED_DAY.bikri_today_count).toBe(2);
  });

  it('the attention list of 18-10 says what the bell counts', () => {
    expect(ATTENTION_18_10.items.map((item) => item.kind)).toEqual([
      'open_days',
      'payables_old',
      'backup_old',
    ]);
    expect(ATTENTION_18_10.total_count).toBe(
      ATTENTION_18_10.items.reduce((sum, item) => sum + item.count, 0),
    );
  });
});
