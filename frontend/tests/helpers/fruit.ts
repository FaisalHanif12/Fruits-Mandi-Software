// Test helper: sets the answers of the fruit command wrappers from the in-memory books of
// fruit-data.ts (plan 17d contract, PRD 16.7). The wrappers are mocked in the test file with
// vi.mock. The books throw the backend's refusal as a `CommandError`; here it becomes a rejected
// promise, the way `invoke` fails.

import { vi } from 'vitest';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { SearchHit } from '../../src/bindings/SearchHit';
import * as fruitApi from '../../src/features/fruit/api';
import * as searchApi from '../../src/shared/search/api';
import {
  APPLE_REF,
  BANANA_REF,
  BOX_REF,
  createFruitBooks,
  type FruitOptions,
  type FruitStore,
} from './fruit-data';

export * from './fruit-data';
export type FakeFruit = FruitStore;

/** Answers at once, or rejects with the refusal the books threw. */
function answer<T>(make: () => T): Promise<T> {
  return new Promise<T>((resolve) => {
    resolve(make());
  });
}

/** Sets the answers of the fruit commands, and returns the store behind them. */
export function fakeFruit(options: FruitOptions = {}): FakeFruit {
  const books = createFruitBooks(options);
  vi.mocked(fruitApi.fruitPreview).mockImplementation((input) =>
    answer(() => books.preview(input)),
  );
  vi.mocked(fruitApi.fruitSave).mockImplementation((input) => answer(() => books.save(input)));
  vi.mocked(fruitApi.fruitEdit).mockImplementation((input) => answer(() => books.edit(input)));
  vi.mocked(fruitApi.fruitCancel).mockImplementation((input) => answer(() => books.cancel(input)));
  vi.mocked(fruitApi.fruitGet).mockImplementation((input) => answer(() => books.get(input)));
  vi.mocked(fruitApi.fruitList).mockImplementation((input) => answer(() => books.list(input)));
  vi.mocked(fruitApi.fruitLots).mockImplementation((input) => answer(() => books.lots(input)));
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

/** The item and packing pickers of the other-shop source: banana and apple, box. */
export function fakeFruitPickers(): void {
  const hitsOf = (scope: string): SearchHit[] =>
    scope === 'item'
      ? [hitOf(BANANA_REF, 'item'), hitOf(APPLE_REF, 'item')]
      : scope === 'packing'
        ? [hitOf(BOX_REF, 'packing')]
        : [];
  vi.mocked(searchApi.searchMasters).mockImplementation(({ scope }) =>
    Promise.resolve(hitsOf(scope)),
  );
  vi.mocked(searchApi.searchRecent).mockImplementation(({ scope }) =>
    Promise.resolve(hitsOf(scope)),
  );
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}
