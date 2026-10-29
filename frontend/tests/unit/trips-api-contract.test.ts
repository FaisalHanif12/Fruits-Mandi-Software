// The screens and the backend agree on plan 16b: each wrapper in features/trips/api.ts and
// features/purchases/api.ts sends the command name the backend registers and the input in the shape
// `{ input }` (the two commands without input send nothing). The names are also read from the
// backend's own command files, so a command renamed there fails here first. The shapes of the
// answers are the generated bindings, which the canned data of the tests must fit (tsc checks it).

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as purchasesApi from '../../src/features/purchases/api';
import * as tripsApi from '../../src/features/trips/api';
import * as invoke from '../../src/shared/api/invoke';
import { e34Purchase, tripPartners } from '../helpers/trips-data';

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
  it('every trips_ and purchases_ command used in api.ts exists in the backend files', () => {
    const trips = declared('trips.rs');
    const purchases = declared('purchases.rs');
    for (const name of [
      'trips_defaults',
      'trips_create',
      'trips_set_partners',
      'trips_list',
      'trips_get',
      'trips_summary',
      'trips_cost_preview',
      'trips_cost_save',
      'trips_cost_cancel',
      'trips_other_sale_preview',
      'trips_other_sale_save',
      'trips_other_sale_cancel',
    ]) {
      expect(trips).toContain(name);
    }
    for (const name of [
      'purchases_preview',
      'purchases_save',
      'purchases_cancel',
      'purchases_list',
      'purchases_get',
      'purchases_write_off',
      'purchases_write_off_undo',
      // Many item lines in one entry, and the split between our shop and another shop (plan 35).
      'purchases_entry_preview',
      'purchases_entry_save',
      'purchases_change_split',
    ]) {
      expect(purchases).toContain(name);
    }
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  const tripInput = {
    date: '10-10-2026',
    market: null,
    sent_person: null,
    note: null,
    partners: tripPartners().map((row) => ({
      partner_id: row.partner.id,
      share: '40',
      source: row.source,
    })),
  };
  const entryParts = [{ partner_id: 1, source: 'shop' as const }];

  it('the two commands without input send only their name', async () => {
    await tripsApi.tripsDefaults();
    await tripsApi.tripsSummary();
    expect(call).toHaveBeenNthCalledWith(1, 'trips_defaults');
    expect(call).toHaveBeenNthCalledWith(2, 'trips_summary');
  });

  it('trips', async () => {
    await tripsApi.tripsCreate(tripInput);
    expect(call).toHaveBeenLastCalledWith('trips_create', { input: tripInput });
    await tripsApi.tripsSetPartners({ id: 1, version: 2, partners: tripInput.partners });
    expect(call).toHaveBeenLastCalledWith('trips_set_partners', {
      input: { id: 1, version: 2, partners: tripInput.partners },
    });
    const list = {
      from: null,
      to: null,
      status: null,
      partner_id: null,
      search: null,
      offset: 0,
      limit: 100,
    };
    await tripsApi.tripsList(list);
    expect(call).toHaveBeenLastCalledWith('trips_list', { input: list });
    await tripsApi.tripsGet({ id: 1, entries_offset: 0, entries_limit: 100 });
    expect(call).toHaveBeenLastCalledWith('trips_get', {
      input: { id: 1, entries_offset: 0, entries_limit: 100 },
    });
  });

  it('trip costs and sales in other shops', async () => {
    const cost = {
      trip_id: 1,
      date: '10-10-2026',
      amount: '4000',
      reason: 'Truck',
      parts: entryParts,
      confirm_cash_below_zero: false,
    };
    await tripsApi.tripsCostPreview(cost);
    expect(call).toHaveBeenLastCalledWith('trips_cost_preview', { input: cost });
    await tripsApi.tripsCostSave(cost);
    expect(call).toHaveBeenLastCalledWith('trips_cost_save', { input: cost });
    const cancel = { id: 12, version: 1, reason: 'Wrong', confirm_cash_below_zero: false };
    await tripsApi.tripsCostCancel(cancel);
    expect(call).toHaveBeenLastCalledWith('trips_cost_cancel', { input: cancel });
    const sale = {
      trip_id: 1,
      date: '10-10-2026',
      shop: 'Rehmat Fruit',
      purchase_id: 7,
      qty: '15',
      amount: '8460',
      note: null,
    };
    await tripsApi.tripsOtherSalePreview(sale);
    expect(call).toHaveBeenLastCalledWith('trips_other_sale_preview', { input: sale });
    await tripsApi.tripsOtherSaleSave(sale);
    expect(call).toHaveBeenLastCalledWith('trips_other_sale_save', { input: sale });
    await tripsApi.tripsOtherSaleCancel(cancel);
    expect(call).toHaveBeenLastCalledWith('trips_other_sale_cancel', { input: cancel });
  });

  it('purchases', async () => {
    const input = {
      trip_id: 1,
      date: '10-10-2026',
      sell_date: null,
      supplier: null,
      item_id: 61,
      packing_id: 71,
      qty: '20',
      weight: null,
      rate: '500',
      freight: '400',
      labour: null,
      other_cost: null,
      note: null,
      parts: entryParts,
      confirm_cash_below_zero: false,
    };
    await purchasesApi.purchasesPreview(input);
    expect(call).toHaveBeenLastCalledWith('purchases_preview', { input });
    await purchasesApi.purchasesSave(input);
    expect(call).toHaveBeenLastCalledWith('purchases_save', { input });
    const entry = {
      trip_id: 1,
      date: '10-10-2026',
      supplier: null,
      sell_date: null,
      parts: entryParts,
      confirm_cash_below_zero: false,
      lines: [
        {
          item_id: 61,
          packing_id: 71,
          qty: '20',
          weight: null,
          rate: '500',
          freight: null,
          labour: null,
          other_cost: null,
          note: null,
          other_qty: '5',
          shop_name: 'Haji Shop',
        },
      ],
    };
    await purchasesApi.purchasesEntryPreview(entry);
    expect(call).toHaveBeenLastCalledWith('purchases_entry_preview', { input: entry });
    await purchasesApi.purchasesEntrySave(entry);
    expect(call).toHaveBeenLastCalledWith('purchases_entry_save', { input: entry });
    const split = { id: 7, version: 1, other_qty: '5', shop_name: 'Haji Shop' };
    await purchasesApi.purchasesChangeSplit(split);
    expect(call).toHaveBeenLastCalledWith('purchases_change_split', { input: split });
    await purchasesApi.purchasesGet(e34Purchase().id);
    expect(call).toHaveBeenLastCalledWith('purchases_get', { input: { id: 7 } });
    await purchasesApi.purchasesWriteOff(7, '5', 'Spoiled');
    expect(call).toHaveBeenLastCalledWith('purchases_write_off', {
      input: { purchase_id: 7, qty: '5', reason: 'Spoiled' },
    });
    await purchasesApi.purchasesWriteOffUndo(90, 'Mistake');
    expect(call).toHaveBeenLastCalledWith('purchases_write_off_undo', {
      input: { move_id: 90, reason: 'Mistake' },
    });
    const cancel = { id: 7, version: 1, reason: 'Wrong', confirm_cash_below_zero: false };
    await purchasesApi.purchasesCancel(cancel);
    expect(call).toHaveBeenLastCalledWith('purchases_cancel', { input: cancel });
    const list = {
      trip_id: null,
      from: null,
      to: null,
      item_id: null,
      state: 'in_stock' as const,
      offset: 0,
      limit: 0,
    };
    await purchasesApi.purchasesList(list);
    expect(call).toHaveBeenLastCalledWith('purchases_list', { input: list });
  });
});
