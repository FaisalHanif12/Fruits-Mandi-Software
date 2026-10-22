// Test helper: sets the answers of the trips and purchases command wrappers from the in-memory
// books of trips-data.ts (plan 16b contract, PRD v1.7). The wrappers are mocked in the test file
// with vi.mock. The books throw the backend's refusal as a `CommandError`; here it becomes a
// rejected promise, the way `invoke` fails.

import { vi } from 'vitest';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { SearchHit } from '../../src/bindings/SearchHit';
import * as purchasesApi from '../../src/features/purchases/api';
import * as tripsApi from '../../src/features/trips/api';
import * as searchApi from '../../src/shared/search/api';
import { BOX, ORANGE, createTripBooks, type TripStore, type TripsOptions } from './trips-data';

export * from './trips-data';
export type FakeTrips = TripStore;

/** Answers at once, or rejects with the refusal the books threw. */
function answer<T>(make: () => T): Promise<T> {
  return new Promise<T>((resolve) => {
    resolve(make());
  });
}

/** Sets the answers of the trips and purchases commands, and returns the store behind them. */
export function fakeTrips(options: TripsOptions = {}): FakeTrips {
  const books = createTripBooks(options);
  vi.mocked(tripsApi.tripsDefaults).mockImplementation(() => answer(() => books.defaults()));
  vi.mocked(tripsApi.tripsList).mockImplementation((input) => answer(() => books.list(input)));
  vi.mocked(tripsApi.tripsGet).mockImplementation((input) => answer(() => books.get(input)));
  vi.mocked(tripsApi.tripsCreate).mockImplementation((input) => answer(() => books.create(input)));
  vi.mocked(purchasesApi.purchasesPreview).mockImplementation((input) =>
    answer(() => books.purchasePreview(input)),
  );
  vi.mocked(purchasesApi.purchasesSave).mockImplementation((input) =>
    answer(() => books.purchaseSave(input)),
  );
  vi.mocked(tripsApi.tripsCostPreview).mockImplementation((input) =>
    answer(() => books.costPreview(input)),
  );
  vi.mocked(tripsApi.tripsCostSave).mockImplementation((input) =>
    answer(() => books.costSave(input)),
  );
  vi.mocked(tripsApi.tripsOtherSalePreview).mockImplementation((input) =>
    answer(() => books.salePreview(input)),
  );
  vi.mocked(tripsApi.tripsOtherSaleSave).mockImplementation((input) =>
    answer(() => books.saleSave(input)),
  );
  vi.mocked(tripsApi.tripsCostCancel).mockImplementation(() => answer(() => books.costCancel()));
  vi.mocked(tripsApi.tripsOtherSaleCancel).mockImplementation(() =>
    answer(() => books.saleCancel()),
  );
  vi.mocked(purchasesApi.purchasesGet).mockImplementation((id) =>
    answer(() => books.purchaseGet(id)),
  );
  vi.mocked(purchasesApi.purchasesList).mockImplementation((input) =>
    answer(() => books.purchaseList(input)),
  );
  vi.mocked(purchasesApi.purchasesCancel).mockImplementation(() =>
    answer(() => books.purchaseCancel()),
  );
  vi.mocked(purchasesApi.purchasesWriteOff).mockImplementation(() =>
    answer(() => books.writeOff()),
  );
  vi.mocked(purchasesApi.purchasesWriteOffUndo).mockImplementation(() =>
    answer(() => books.writeOffUndo()),
  );
  return books.store;
}

const hitOf = (record: RecordRef, kind: SearchHit['kind']): SearchHit => ({
  id: record.id,
  kind,
  code: record.code,
  name_ur: record.name_ur,
  name_en: record.name_en,
  extra_ur: '',
  extra_en: '',
  is_active: true,
});

/** The item and packing pickers of the purchase form: one orange and one box. */
export function fakeItemPickers(): void {
  const hitsOf = (scope: string): SearchHit[] =>
    scope === 'item' ? [hitOf(ORANGE, 'item')] : scope === 'packing' ? [hitOf(BOX, 'packing')] : [];
  vi.mocked(searchApi.searchMasters).mockImplementation(({ scope }) =>
    Promise.resolve(hitsOf(scope)),
  );
  vi.mocked(searchApi.searchRecent).mockImplementation(({ scope }) =>
    Promise.resolve(hitsOf(scope)),
  );
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}
