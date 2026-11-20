// Plan 78 and the backend (C1 load buttons, E where the capital is, D3 garden refusals): each wrapper sends the
// command name the backend registers, in the shape `{ input }`; the names are also read from the backend's own
// command files, so a command renamed there fails here first; and the values of the two new refusals of a
// garden (`max_paisa`, `open_paisa`) read as money. The inputs are the backend's real request files.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import * as cashbookApi from '../../src/features/cashbook/api';
import { messageVars } from '../../src/shared/api';
import * as invoke from '../../src/shared/api/invoke';
import * as real from '../helpers/plan78-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const COMMANDS_DIR = path.resolve(__dirname, '../../../backend/app/src/commands');

/** The names of the commands a backend file declares (`pub async fn name`). */
function declared(file: string): string[] {
  const text = readFileSync(path.join(COMMANDS_DIR, file), 'utf8');
  return [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
}

describe('the commands of plan 78 are the ones the backend declares', () => {
  it('the names exist in the backend files', () => {
    expect(declared('bikri.rs')).toEqual(
      expect.arrayContaining(['bikri_goods_sources', 'bikri_goods_to_load']),
    );
    expect(declared('cashbook.rs')).toContain('cashbook_capital_where');
    // Plan 87: the fix of the card. A command the screen calls that the backend does not have is a failed screen,
    // so the names are read from the backend's files here.
    expect(declared('partners.rs')).toContain('partners_place_opening_capital');
  });
});

describe('the print commands of the capital card (plan 87)', () => {
  it('the backend declares them, under the names the screen calls', () => {
    expect(declared('cashbook.rs')).toContain('cashbook_capital_print_data');
    expect(declared('report_print.rs')).toContain('cashbook_capital_print');
  });
});

describe('every wrapper of plan 78 sends its command and its input', () => {
  it('the counts of the load buttons take no input', async () => {
    await bikriApi.bikriGoodsSources();
    expect(call).toHaveBeenLastCalledWith('bikri_goods_sources');
    expect(real.GOODS_SOURCES).toEqual({ stock_lots: 1, trip_rows: 1, garden_rows: 1 });
  });

  it('the rows of a card ask for their kind and a page, with the kind of the backend’s own request (plan 80 R2-5)', async () => {
    await bikriApi.bikriGoodsToLoad('trip', 0, 25);
    expect(call).toHaveBeenLastCalledWith('bikri_goods_to_load', {
      input: { ...real.GOODS_TO_LOAD_TRIP_REQUEST, offset: 0, limit: 25 },
    });
    await bikriApi.bikriGoodsToLoad('garden', 50, 50);
    expect(call).toHaveBeenLastCalledWith('bikri_goods_to_load', {
      input: { ...real.GOODS_TO_LOAD_GARDEN_REQUEST, offset: 50, limit: 50 },
    });
  });

  it('the card of the cash book asks as of a day as typed, or as of today with no day (plan 87)', async () => {
    await cashbookApi.cashbookCapitalWhere({});
    expect(call).toHaveBeenLastCalledWith('cashbook_capital_where', { input: {} });
    await cashbookApi.cashbookCapitalWhere({ as_of: '10-10-2026' });
    expect(call).toHaveBeenLastCalledWith('cashbook_capital_where', {
      input: { as_of: '10-10-2026' },
    });
  });

  it('putting the opening capital into cash sends the partners, the reason and the sign as typed (plan 87)', async () => {
    const input = { partner_ids: [1, 2], reason: 'old books', sign: 'adnan' };
    await cashbookApi.partnersPlaceOpeningCapital(input);
    expect(call).toHaveBeenLastCalledWith('partners_place_opening_capital', { input });
  });

  it('the print of the card sends the day, the language and the paper, and where it goes (plan 87)', async () => {
    const sheet = { language: 'ur', paper: 'a4' } as const;
    await cashbookApi.cashbookCapitalPrintData(sheet);
    expect(call).toHaveBeenLastCalledWith('cashbook_capital_print_data', { input: sheet });
    await cashbookApi.cashbookCapitalPrint({ sheet, to: 'pdf', copies: null });
    expect(call).toHaveBeenLastCalledWith('cashbook_capital_print', {
      input: { sheet, to: 'pdf', copies: null },
    });
  });
});

describe('the value of the new refusal of a garden refund reads as money', () => {
  it('the most a refund can be is in rupees, from open_paisa', () => {
    const refund = real.REFUND_TOO_MUCH.errors[0];
    expect(refund?.code).toBe('ERR_RETURN_EXCEEDS_OPEN');
    expect(messageVars(refund?.values ?? {}, 'international')['open']).toBe('5,000');
  });
});
