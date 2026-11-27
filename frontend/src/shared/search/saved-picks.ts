// The records picked since the last save, to be recorded as used once more when an entry is saved
// (plan 100 I, PRD FR-7.41: pickers show the most used first, and the counts are updated on save). A pick
// is already recorded when it is made; recording it again after the save makes a saved record count more
// than one picked and thrown away. The order itself is the backend's.

/** Records one picked record as used (`search_touch` or `search_accounts_touch`). */
type Touch = () => Promise<unknown>;

const waiting = new Map<string, Touch>();

/** The mutation meta of a call that records a pick: it is not a save. */
export const NOT_A_SAVE = { notASave: true } as const;

/** A record was picked in a field. Picking the same one again before a save keeps one entry. */
export function notePick(key: string, touch: Touch): void {
  waiting.set(key, touch);
}

/** A save went through: each record picked since the last one is recorded as used. A failure is dropped:
    the list order is a help, never a reason to tell the user anything. */
export function recordSavedPicks(): void {
  const list = [...waiting.values()];
  waiting.clear();
  for (const touch of list) void touch().catch(() => undefined);
}

/** Forgets the picks that wait, for a new session of the app. */
export function forgetPicks(): void {
  waiting.clear();
}
