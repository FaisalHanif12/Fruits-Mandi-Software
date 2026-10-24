// A value a test needs to be there: the element it is about, the first of a list. It says what was
// missing, so a failing test explains itself, and it keeps the test free of `as` and `!` (which the
// lint rules forbid in turn).

export function must<T>(value: T | null | undefined, what = 'the element'): T {
  if (value === null || value === undefined) throw new Error(`${what} is not there`);
  return value;
}
