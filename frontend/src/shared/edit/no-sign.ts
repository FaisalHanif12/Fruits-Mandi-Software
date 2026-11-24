// The Owner's sign is a secret, like a password (PRD FR-7.71 and FR-21.4, v1.58, plan 96 T5). It is never
// shown: not in the stamp of an entry, not in the tip of a list row, not in a field mark, not in the edit
// history and not in the audit viewer. The backend no longer sends it (plan 96 T4), and an old audit row that
// still holds the text is stripped by the backend's readers. This file is the second lock: a changed field
// that is a sign (its last key is `sign`, `owner_sign`, `current_sign` or `new_sign`) is dropped here, so
// even a reader that forgot to strip can never put the text on a screen. Pure; nothing is worked out.

const SIGN_KEYS: ReadonlySet<string> = new Set(['sign', 'owner_sign', 'current_sign', 'new_sign']);

/** True when a stored path names the Owner's sign, for example `sign` or `reopen.sign`. */
export function isSignPath(path: string): boolean {
  return SIGN_KEYS.has(path.slice(path.lastIndexOf('.') + 1).toLowerCase());
}

/** The changed fields of an entry without the Owner's sign. The order is kept. */
export function withoutSignChanges<T extends { path: string }>(changes: readonly T[]): T[] {
  return changes.filter((change) => !isSignPath(change.path));
}
