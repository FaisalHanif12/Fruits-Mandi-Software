// The Owner's sign on an edit (PRD FR-7.71 and FR-21.4, v1.36, plan 73). An edit made through a reopened day
// carries the sign that was given when the day was opened, and every place that writes "when and by whom"
// writes it after the name: "Edited: 13-11-2026 12:40 am, Owner, Sign: adnan". An edit of an open day has no
// sign, and its words stay as they were. `EditStamp.sign` and `EditVersion.sign` are `string | null`; the
// screens read them here only, so a missing or empty sign is one rule (no sign) in one place. Nothing is
// worked out: the sign is the backend's, shown as it was kept.

/** The sign kept with an edit, or null when it was made on an open day (or the backend sent none). */
export function signOf(item: object): string | null {
  // Read as unknown on purpose: the backend's fixtures of earlier plans have no `sign` at all, and a missing
  // sign is the same as none.
  const sign: unknown = 'sign' in item ? item.sign : null;
  return typeof sign === 'string' && sign.trim() !== '' ? sign : null;
}

/**
 * The Fluent message and its variables for a line of "when and by whom": `edit-stamp`, `edit-mark-tip` or
 * `edit-history-version`, and the same with `-signed` when the edit carries a sign.
 */
export function whenWhoMessage(
  id: 'edit-stamp' | 'edit-mark-tip' | 'edit-history-version',
  item: object,
  vars: Record<string, string>,
): { id: string; vars: Record<string, string> } {
  const sign = signOf(item);
  return sign === null ? { id, vars } : { id: `${id}-signed`, vars: { ...vars, sign } };
}
