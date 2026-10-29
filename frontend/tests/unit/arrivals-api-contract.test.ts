// The screens and the backend agree on plan 33 version 1.1 (the date to sell, the goods arrived of a
// garden, the goods ready for a new بکری): each wrapper sends the command name the backend registers
// and the input in the shape `{ input }`. The names are also read from the backend's own command
// files, so a command renamed there fails here first. The requests are the backend's real files;
// the answers of the screens' fakes are checked against the real files in the screen tests.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import * as gardensApi from '../../src/features/gardens/api';
import * as purchasesApi from '../../src/features/purchases/api';
import * as invoke from '../../src/shared/api/invoke';
import * as arrivals from '../helpers/arrivals-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const COMMANDS_DIR = path.resolve(__dirname, '../../../backend/app/src/commands');

/** The names of the commands the backend files declare (`pub async fn name`). */
function declared(file: string): string[] {
  const text = readFileSync(path.join(COMMANDS_DIR, file), 'utf8');
  return [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
}

describe('the commands the screens call are the ones the backend declares', () => {
  it('every new command used in an api.ts exists in the backend files', () => {
    const gardens = declared('gardens.rs');
    for (const name of [
      'gardens_arrival_save',
      'gardens_arrivals_list',
      'gardens_arrival_get',
      'gardens_arrival_cancel',
      'gardens_arrival_set_sell_date',
      'gardens_arrival_write_off',
      'gardens_arrival_write_off_undo',
    ]) {
      expect(gardens).toContain(name);
    }
    expect(declared('bikri.rs')).toContain('bikri_ready_goods');
    expect(declared('purchases.rs')).toContain('purchases_set_sell_date');
  });
});

describe('every wrapper sends its command name and its input', () => {
  it('the goods ready for a new بکری: the day of the بکری as DD-MM-YYYY', async () => {
    await bikriApi.bikriReadyGoods('12-10-2026');
    expect(call).toHaveBeenLastCalledWith('bikri_ready_goods', { input: { date: '12-10-2026' } });
  });

  it('the date to sell of a purchase', async () => {
    const input = arrivals.PURCHASE_SET_SELL_DATE_REQUEST;
    await purchasesApi.purchasesSetSellDate(input);
    expect(call).toHaveBeenLastCalledWith('purchases_set_sell_date', { input });
  });

  it('the goods arrived of a garden: save, list, get, cancel, date to sell, write-off and its undo', async () => {
    await gardensApi.gardensArrivalSave(arrivals.ARRIVAL_SAVE_REQUEST);
    expect(call).toHaveBeenLastCalledWith('gardens_arrival_save', {
      input: arrivals.ARRIVAL_SAVE_REQUEST,
    });
    await gardensApi.gardensArrivalsList(arrivals.ARRIVALS_IN_STOCK_REQUEST);
    expect(call).toHaveBeenLastCalledWith('gardens_arrivals_list', {
      input: arrivals.ARRIVALS_IN_STOCK_REQUEST,
    });
    await gardensApi.gardensArrivalGet(1);
    expect(call).toHaveBeenLastCalledWith('gardens_arrival_get', { input: { id: 1 } });
    await gardensApi.gardensArrivalCancel(arrivals.ARRIVAL_CANCEL_REQUEST);
    expect(call).toHaveBeenLastCalledWith('gardens_arrival_cancel', {
      input: arrivals.ARRIVAL_CANCEL_REQUEST,
    });
    await gardensApi.gardensArrivalSetSellDate(arrivals.ARRIVAL_SET_SELL_DATE_REQUEST);
    expect(call).toHaveBeenLastCalledWith('gardens_arrival_set_sell_date', {
      input: arrivals.ARRIVAL_SET_SELL_DATE_REQUEST,
    });
    await gardensApi.gardensArrivalWriteOff(arrivals.ARRIVAL_WRITE_OFF_REQUEST);
    expect(call).toHaveBeenLastCalledWith('gardens_arrival_write_off', {
      input: arrivals.ARRIVAL_WRITE_OFF_REQUEST,
    });
    await gardensApi.gardensArrivalWriteOffUndo(arrivals.ARRIVAL_WRITE_OFF_UNDO_REQUEST);
    expect(call).toHaveBeenLastCalledWith('gardens_arrival_write_off_undo', {
      input: arrivals.ARRIVAL_WRITE_OFF_UNDO_REQUEST,
    });
  });
});

describe('the real answers have the shapes the screens read', () => {
  it('the ready goods: a trip row with its trip and a garden row with its garden and its account', () => {
    const [trip, garden] = arrivals.READY_GOODS.rows;
    expect(trip?.kind).toBe('trip');
    expect(trip?.trip?.code).toBe('0070');
    expect(trip?.garden).toBeNull();
    expect(garden?.kind).toBe('garden');
    expect(garden?.garden?.code).toBe('0080');
    expect(garden?.garden?.account.code).toBe('0080');
    expect(garden?.trip).toBeNull();
    expect(arrivals.READY_GOODS_NONE.rows).toEqual([]);
  });

  it('after a part sale the same goods come with what is left', () => {
    const garden = arrivals.READY_GOODS_AFTER_A_PART_SALE.rows.find((row) => row.kind === 'garden');
    expect(garden?.qty).toBe(100);
    expect(garden?.qty_left).toBe(40);
  });

  it('a garden lot with its arrival: the preview carries it, the saved lot names it, the oversale is refused', () => {
    expect(arrivals.GARDEN_LOT_PREVIEW.lots[0]?.arrival).toMatchObject({
      arrival_id: 1,
      qty_left: 100,
    });
    expect(arrivals.GARDEN_LOT_SAVED.lots[0]?.arrival).toMatchObject({
      id: 1,
      garden_code: '0080',
    });
    expect(arrivals.GARDEN_LOT_SAVE_OVERSOLD.errors[0]).toMatchObject({
      code: 'ERR_QTY_EXCEEDS_LOT',
      field: 'lots[0].qty_received',
      values: { available: 40 },
    });
    expect(arrivals.STOCK_WITH_GARDEN_ROWS.garden_rows.length).toBeGreaterThan(0);
  });

  it('every refusal of the new commands has an error code with words (checked by the i18n tests)', () => {
    const codes = [
      arrivals.ARRIVAL_SAVE_SELL_DATE_BEFORE,
      arrivals.ARRIVAL_CANCEL_HAS_SALES,
      arrivals.ARRIVAL_WRITE_OFF_EXCEEDS,
      arrivals.GARDEN_CLOSE_HAS_STOCK,
    ].flatMap((refusal) => refusal.errors.map((item) => item.code));
    expect(codes).toEqual([
      'ERR_SELL_DATE_BEFORE_ENTRY',
      'ERR_ARRIVAL_HAS_SALES',
      'ERR_WRITE_OFF_EXCEEDS_BALANCE',
      'ERR_GARDEN_HAS_STOCK',
    ]);
  });
});
