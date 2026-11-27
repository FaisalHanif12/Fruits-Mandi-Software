// Test helper: sets the answers of the gardens command wrappers from the in-memory books of
// gardens-data.ts (plan 17c contract, PRD v1.7). The wrappers are mocked in the test file with
// vi.mock. The books throw the backend's refusal as a `CommandError`; here it becomes a rejected
// promise, the way `invoke` fails.

import { paisaForTest } from './paisa';
import { vi } from 'vitest';
import type { ArrivalsList } from '../../src/bindings/ArrivalsList';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { SearchHit } from '../../src/bindings/SearchHit';
import * as gardensApi from '../../src/features/gardens/api';
import * as searchApi from '../../src/shared/search/api';
import { BANANA, createGardenBooks, type GardenStore, type GardensOptions } from './gardens-data';

export * from './gardens-data';
export type FakeGardens = GardenStore;

/** Answers at once, or rejects with the refusal the books threw. */
function answer<T>(make: () => T): Promise<T> {
  return new Promise<T>((resolve) => {
    resolve(make());
  });
}

/** A list of goods arrived with nothing in it, as the backend answers it. */
export function noArrivals(): ArrivalsList {
  return {
    rows: [],
    total_count: 0,
    totals: {
      qty: 0,
      qty_sold: 0,
      qty_written_off: 0,
      qty_left: 0,
      qty_our_sold: 0,
      qty_other_sold: 0,
      qty_our_left: 0,
      qty_other_left: 0,
      sale_value_paisa: paisaForTest(0),
      commission_paisa: paisaForTest(0),
    },
    months: [],
  };
}

/**
 * The goods arrived list answers the entries given when the goods not sold yet are asked for (and
 * when every entry is asked for, as the sale in another shop does), and nothing when the goods sold
 * are asked for (PRD FR-12.53: the two lists of a garden).
 */
export function fakeUnsoldArrivals(list: ArrivalsList): void {
  vi.mocked(gardensApi.gardensArrivalsList).mockImplementation((input) =>
    Promise.resolve(input.state === 'sold_out' ? noArrivals() : list),
  );
}

/** Sets the answers of the gardens commands, and returns the store behind them. */
export function fakeGardens(options: GardensOptions = {}): FakeGardens {
  const books = createGardenBooks(options);
  vi.mocked(gardensApi.gardensDefaults).mockImplementation(() => answer(() => books.defaults()));
  vi.mocked(gardensApi.gardensCreatePreview).mockImplementation((input) =>
    answer(() => books.createPreview(input)),
  );
  vi.mocked(gardensApi.gardensCreate).mockImplementation((input) =>
    answer(() => books.create(input)),
  );
  vi.mocked(gardensApi.gardensList).mockImplementation((input) => answer(() => books.list(input)));
  vi.mocked(gardensApi.gardensGet).mockImplementation((input) => answer(() => books.get(input)));
  vi.mocked(gardensApi.gardensSummary).mockImplementation(() => answer(() => books.summary()));
  vi.mocked(gardensApi.gardensEntryPreview).mockImplementation((input) =>
    answer(() => books.entryPreview(input)),
  );
  vi.mocked(gardensApi.gardensEntrySave).mockImplementation((input) =>
    answer(() => books.entrySave(input)),
  );
  vi.mocked(gardensApi.gardensEntryCancel).mockImplementation((input) =>
    answer(() => books.entryCancel(input)),
  );
  vi.mocked(gardensApi.gardensClosePreview).mockImplementation((input) =>
    answer(() => books.closePreview(input)),
  );
  vi.mocked(gardensApi.gardensClose).mockImplementation((input) =>
    answer(() => books.close(input)),
  );
  vi.mocked(gardensApi.gardensReopen).mockImplementation((input) =>
    answer(() => books.reopen(input)),
  );
  // A garden with no goods arrived yet; the tests of the goods arrived set their own answers.
  vi.mocked(gardensApi.gardensArrivalsList).mockImplementation(() => Promise.resolve(noArrivals()));
  // No sale of the garden yet; the tests of the sale money set their own answers.
  vi.mocked(gardensApi.gardensSaleMoney).mockImplementation(() =>
    Promise.resolve({
      rows: [],
      total_count: 0,
      totals: {
        qty: 0,
        value_paisa: paisaForTest(0),
        buyer_total_paisa: paisaForTest(0),
        paid_paisa: paisaForTest(0),
        collected_paisa: paisaForTest(0),
        still_due_paisa: paisaForTest(0),
        lot_charges_paisa: paisaForTest(0),
        garden_credit_paisa: paisaForTest(0),
      },
    }),
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
  sold_by_weight: false,
});

/** The item picker of the new garden form: one banana, and the other items a test adds (plan 78 D2). */
export function fakeGardenItemPicker(others: readonly RecordRef[] = []): void {
  const items = [BANANA, ...others];
  const hitsOf = (scope: string): SearchHit[] =>
    scope === 'item' ? items.map((record) => hitOf(record, 'item')) : [];
  vi.mocked(searchApi.searchMasters).mockImplementation(({ scope, text }) => {
    const wanted = text.trim().toLowerCase();
    return Promise.resolve(
      hitsOf(scope).filter(
        (hit) =>
          wanted === '' ||
          hit.name_en.toLowerCase().includes(wanted) ||
          hit.name_ur.includes(wanted) ||
          hit.code.includes(wanted),
      ),
    );
  });
  vi.mocked(searchApi.searchRecent).mockImplementation(({ scope }) =>
    Promise.resolve(hitsOf(scope)),
  );
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}
