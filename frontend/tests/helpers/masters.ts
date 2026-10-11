// Test helper: an in-memory fake of the masters and search commands (plan 06 contract). The
// command wrappers are mocked in the test file with vi.mock; this sets their answers from a small
// store, the way a healthy backend answers: a list, a next code, a save that returns the saved row
// with a new version, a change of active state, and a search over code and both names. Nothing is
// invented for the user to see.

import { vi } from 'vitest';
import * as mastersApi from '../../src/features/masters/api';
import type { MasterKind, MasterRecord } from '../../src/features/masters/types';
import * as searchApi from '../../src/shared/search/api';
import type { SearchHit } from '../../src/shared/search';

export interface FakeMasters {
  /** The rows of every kind now, as the backend would list them. */
  rows: Record<MasterKind, MasterRecord[]>;
}

const KINDS: readonly MasterKind[] = [
  'city',
  'item',
  'packing',
  'adda',
  'marka',
  'store',
  'narration',
  'expense_head',
  'staff',
  'bank_account',
];

export function record(
  kind: MasterKind,
  id: number,
  code: string,
  ur: string,
  en: string,
  extra: Partial<MasterRecord> = {},
): MasterRecord {
  return {
    kind,
    id,
    version: 1,
    code,
    is_active: true,
    names: { name_ur: ur, name_en: en },
    text: {},
    refs: {},
    shown: {},
    lists: {},
    ...extra,
  };
}

function hitOf(row: MasterRecord): SearchHit {
  return {
    id: row.id,
    kind: row.kind,
    code: row.code,
    name_ur: row.names?.name_ur ?? row.text['mark_text'] ?? '',
    name_en: row.names?.name_en ?? row.text['mark_text'] ?? '',
    extra_ur: '',
    extra_en: '',
    is_active: row.is_active,
  };
}

function matches(row: MasterRecord, text: string): boolean {
  const wanted = text.trim().toLowerCase();
  return [row.code, row.names?.name_ur, row.names?.name_en, row.text['mark_text']]
    .filter((part): part is string => part !== undefined)
    .some((part) => part.toLowerCase().includes(wanted));
}

/**
 * Sets the answers of the masters and search commands from `initial` and returns the store, so a
 * test can read what was saved. The wrappers must have been mocked with vi.mock in the test file.
 */
export function fakeMasters(
  initial: Partial<Record<MasterKind, MasterRecord[]>> = {},
): FakeMasters {
  const store: FakeMasters = {
    rows: Object.fromEntries(KINDS.map((kind) => [kind, [...(initial[kind] ?? [])]])) as Record<
      MasterKind,
      MasterRecord[]
    >,
  };
  let nextId = 1000;

  vi.mocked(mastersApi.mastersList).mockImplementation((kind, includeInactive) =>
    Promise.resolve(store.rows[kind].filter((row) => includeInactive || row.is_active)),
  );
  vi.mocked(mastersApi.mastersNextCode).mockImplementation((kind) => {
    const highest = Math.max(0, ...store.rows[kind].map((row) => Number(row.code) || 0));
    return Promise.resolve(String(highest + 1).padStart(4, '0'));
  });
  vi.mocked(mastersApi.mastersSave).mockImplementation((kind, draft) => {
    const names = { name_ur: draft.names.urdu, name_en: draft.names.english };
    const saved: MasterRecord = {
      kind,
      id: draft.id ?? (nextId += 1),
      version: (draft.version ?? 0) + 1,
      code: draft.code,
      is_active: true,
      names: kind === 'marka' ? null : names,
      text: draft.text,
      refs: draft.refs,
      shown: {},
      lists: draft.lists,
    };
    const rows = store.rows[kind];
    const index = rows.findIndex((row) => row.id === saved.id);
    const existing = rows[index];
    if (index >= 0 && existing !== undefined) {
      rows[index] = { ...saved, is_active: existing.is_active, shown: existing.shown };
    } else {
      rows.push(saved);
    }
    return Promise.resolve(rows.find((row) => row.id === saved.id) ?? saved);
  });
  vi.mocked(mastersApi.mastersSetActive).mockImplementation((kind, id, isActive) => {
    const rows = store.rows[kind];
    const index = rows.findIndex((row) => row.id === id);
    const found = rows[index];
    if (found === undefined) return Promise.reject(new Error('not found'));
    const changed = { ...found, is_active: isActive, version: found.version + 1 };
    rows[index] = changed;
    return Promise.resolve(changed);
  });
  vi.mocked(searchApi.searchMasters).mockImplementation(({ scope, text }) =>
    Promise.resolve(store.rows[scope].filter((row) => matches(row, text)).map(hitOf)),
  );
  vi.mocked(searchApi.searchRecent).mockImplementation(({ scope }) =>
    Promise.resolve(store.rows[scope].map(hitOf)),
  );
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
  return store;
}
