// Compares the keys and kinds of two values at every level (plan 17 T4). A test with canned data
// can pass while the real backend sends other fields; this finds that difference. Used by the
// `*-real-shapes` tests, which compare the canned data with the backend's own answers.

type Shape = string | null | Shape[] | { [key: string]: Shape };

/** The keys and kinds of a value at every level; a list is shown by its first element. */
export function shapeOf(value: unknown): Shape {
  if (value === null || value === undefined) return null;
  if (Array.isArray(value)) return value.length === 0 ? [] : [shapeOf(value[0])];
  if (typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, item]) => [key, shapeOf(item)]),
    );
  }
  return typeof value;
}

/** The first difference between two shapes, or null. A null or an empty list can hold anything. */
export function firstDifference(a: Shape, b: Shape, path = ''): string | null {
  if (a === null || b === null) return null;
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b)) return `${path}: list against not a list`;
    const [x, y] = [a[0], b[0]];
    return x === undefined || y === undefined ? null : firstDifference(x, y, `${path}[]`);
  }
  if (typeof a === 'string' || typeof b === 'string') {
    return a === b ? null : `${path}: ${JSON.stringify(a)} against ${JSON.stringify(b)}`;
  }
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const key of [...keys].sort()) {
    const [x, y] = [a[key], b[key]];
    if (x === undefined) return `${path}.${key}: the real answer has no such field`;
    if (y === undefined) return `${path}.${key}: the canned data has no such field`;
    const inner = firstDifference(x, y, `${path}.${key}`);
    if (inner !== null) return inner;
  }
  return null;
}

/** Null when the real answer and the canned data have the same keys and kinds. */
export function same(realValue: unknown, canned: unknown): string | null {
  return firstDifference(shapeOf(realValue), shapeOf(canned));
}

/** JSON with the keys of every object in order, so two objects with the same content match. */
function canonical(value: unknown): string {
  return JSON.stringify(value, (_key, item: unknown) =>
    item !== null && typeof item === 'object' && !Array.isArray(item)
      ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a.localeCompare(b)))
      : item,
  );
}

/**
 * True when `built` (what a screen sends) holds every key of `sent` (what the backend's own test
 * sent) with the same value, and every other key of `built` is an empty default (null, false or
 * empty text). The backend's test files leave defaults out; the screen sends them.
 */
export function sendsTheSame(built: object, sent: object): string | null {
  const a = built as Record<string, unknown>;
  const b = sent as Record<string, unknown>;
  for (const [key, value] of Object.entries(b)) {
    if (canonical(a[key]) !== canonical(value)) {
      return `${key}: the screen sends ${JSON.stringify(a[key])}, the backend’s test sent ${JSON.stringify(value)}`;
    }
  }
  for (const [key, value] of Object.entries(a)) {
    if (!(key in b) && value !== null && value !== false && value !== '') {
      return `${key}: the screen sends ${JSON.stringify(value)} and the backend’s test did not`;
    }
  }
  return null;
}
