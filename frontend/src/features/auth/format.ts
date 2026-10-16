// Display helpers for the sign-in screens. Text only, no money.

/** The shortest password (NFR-SEC-1). The backend enforces it and is the final word. */
export const PASSWORD_MIN_LENGTH = 6;

const GROUP = 4;
const CODE_CHARACTERS = /[^A-Za-z0-9]/g;

/**
 * The recovery code as the user prints and types it: XXXX-XXXX-XXXX-XXXX (plan 04 contract 4).
 * The backend may send it with or without dashes, so the dashes are put back the same way.
 */
export function groupRecoveryCode(code: string): string {
  const characters = code.replace(CODE_CHARACTERS, '').toUpperCase();
  const groups: string[] = [];
  for (let start = 0; start < characters.length; start += GROUP) {
    groups.push(characters.slice(start, start + GROUP));
  }
  return groups.join('-');
}

/** The seconds an ERR_LOCKED_WAIT error asks the user to wait, or null for any other error. */
export function lockSeconds(
  items: readonly { code: string; values: Record<string, string | number> }[],
): number | null {
  const item = items.find((candidate) => candidate.code === 'ERR_LOCKED_WAIT');
  const seconds = item?.values['seconds'];
  return typeof seconds === 'number' ? seconds : null;
}
