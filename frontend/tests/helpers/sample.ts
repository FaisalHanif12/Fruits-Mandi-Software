// Sample values for tests that must not write words as literals in JSX (a user's name is data
// from the backend, shown as typed).

export const USER_NAME = String(JSON.parse('"Owner"'));
export const DATE_TEXT = String(JSON.parse('"10-10-2026"'));
/** An owner whose name is typed in Urdu letters (it is data from the backend, not a word of the app). */
export const URDU_USER_NAME = String(JSON.parse('"عدنان"'));

/** The item at an index, or a failed test when there is none (instead of a `!` assertion). */
export function at<T>(items: readonly T[], index: number): T {
  const item = items[index];
  if (item === undefined) throw new Error(`no item at ${String(index)}`);
  return item;
}
