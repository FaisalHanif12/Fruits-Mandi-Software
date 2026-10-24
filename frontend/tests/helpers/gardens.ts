// Test helper: sets the answers of the gardens command wrappers from the in-memory books of
// gardens-data.ts (plan 17c contract, PRD v1.7). The wrappers are mocked in the test file with
// vi.mock. The books throw the backend's refusal as a `CommandError`; here it becomes a rejected
// promise, the way `invoke` fails.

import { vi } from 'vitest';
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

/** The item picker of the new garden form: one banana. */
export function fakeGardenItemPicker(): void {
  const hitsOf = (scope: string): SearchHit[] => (scope === 'item' ? [hitOf(BANANA, 'item')] : []);
  vi.mocked(searchApi.searchMasters).mockImplementation(({ scope }) =>
    Promise.resolve(hitsOf(scope)),
  );
  vi.mocked(searchApi.searchRecent).mockImplementation(({ scope }) =>
    Promise.resolve(hitsOf(scope)),
  );
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}
