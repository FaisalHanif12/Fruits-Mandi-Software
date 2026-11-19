// Typed wrappers around the search commands (plan 06 contract; module `search`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). These commands only read, except
// `search_touch`, which writes the user's "recent" list and nothing else (Code Standards B29).

import { callCommand } from '../api';
import type {
  SearchHit,
  SearchInput,
  SearchScope,
  SearchScopeInput,
  SearchTouchInput,
} from './types';

/**
 * Records whose code or name (either script, small mistakes forgiven) matches the text. Empty text
 * is every record of the list, the last chosen first, then by code (plan 32, R7). `offset` and
 * `limit` cut out one page; a page shorter than `limit` is the last.
 */
export function searchMasters(
  input: Omit<SearchInput, 'offset'> & { offset?: number },
): Promise<SearchHit[]> {
  return callCommand<SearchHit[]>('search_masters', {
    input: { ...input, offset: input.offset ?? 0 },
  });
}

/** The last 10 records used on this screen, then the rest (PRD FR-7.41). */
export function searchRecent(input: SearchScopeInput): Promise<SearchHit[]> {
  return callCommand<SearchHit[]>('search_recent', { input });
}

/** Remembers that a record was chosen, so it comes first next time. */
export function searchTouch(input: SearchTouchInput): Promise<null> {
  return callCommand<null>('search_touch', { input });
}

/**
 * One master record by its id, found in the full list of its kind (the list of Alt+↓). A screen
 * that gets only an id from the backend, such as the item a seller's memory points at, shows the
 * record in the field with this. Null when the record is not active (inactive records are never
 * offered, so a new entry never uses one).
 */
export async function findMaster(scope: SearchScope, id: number): Promise<SearchHit | null> {
  const list = await searchRecent({ scope });
  return list.find((hit) => hit.id === id) ?? null;
}
