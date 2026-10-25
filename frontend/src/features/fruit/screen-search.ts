// The address of the پھل screen (PRD 16.7, FR-19.4): `/rokar/fruit?entry=7` opens entry 7 in its saved
// view. Kept apart from the screen so the router can read the address without loading the screen
// (Code Standards F36). A value that is not a whole number above 0 is "bad": the screen says the
// entry was not found, the same words as an id the backend does not know.

/** The search of the fruit address: no entry, a fruit entry id, or text that is not an id. */
export interface FruitSearch {
  entry?: number | 'bad';
}

export function parseFruitSearch(search: Record<string, unknown>): FruitSearch {
  const raw = search['entry'];
  if (raw === undefined || raw === null || raw === '') return {};
  const id = typeof raw === 'number' ? raw : typeof raw === 'string' ? Number(raw) : Number.NaN;
  return Number.isSafeInteger(id) && id > 0 ? { entry: id } : { entry: 'bad' };
}
