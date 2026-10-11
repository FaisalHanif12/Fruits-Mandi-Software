// Typed wrappers around the search commands (plan 06 contract; module `search`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). These commands only read, except
// `search_touch`, which writes the user's "recent" list and nothing else (Code Standards B29).

import { callCommand } from '../api';
import type { SearchHit, SearchInput, SearchScopeInput, SearchTouchInput } from './types';

/** Records whose code or name (either script, small mistakes forgiven) matches the text. */
export function searchMasters(input: SearchInput): Promise<SearchHit[]> {
  return callCommand<SearchHit[]>('search_masters', { input });
}

/** The last 10 records used on this screen, then the rest (PRD FR-7.41). */
export function searchRecent(input: SearchScopeInput): Promise<SearchHit[]> {
  return callCommand<SearchHit[]>('search_recent', { input });
}

/** Remembers that a record was chosen, so it comes first next time. */
export function searchTouch(input: SearchTouchInput): Promise<null> {
  return callCommand<null>('search_touch', { input });
}
