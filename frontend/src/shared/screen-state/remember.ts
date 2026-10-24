// The text a screen keeps, and how it is read back (PRD FR-7.45). A remembered choice is a
// convenience, never a source of truth: a value the screen cannot read any more (an old shape, a
// record that is gone) is simply ignored, and the screen starts from its normal first state.

/** The most the backend keeps of one screen, in characters (plan 19). */
export const MAX_REMEMBERED_CHARS = 4_000;

/**
 * The JSON text of a value, or an empty text when there is nothing to keep (null) or the text would
 * be too long for the backend. An empty text makes the backend forget the screen.
 */
export function rememberedText(value: unknown): string {
  if (value === null || value === undefined) return '';
  const text = JSON.stringify(value);
  return text.length > MAX_REMEMBERED_CHARS ? '' : text;
}

/** The value a text holds, or null when it is empty or is not JSON. */
export function readRemembered(text: string | null): unknown {
  if (text === null || text === '') return null;
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return null;
  }
}

/**
 * The text of a value with the keys of every object in order, so two texts of the same value compare
 * equal whoever wrote them (the backend writes its keys in another order than a screen does). An
 * empty or unreadable text is the empty text.
 */
export function canonicalText(text: string | null): string {
  const value = readRemembered(text);
  if (value === null) return '';
  const sort = (item: unknown): unknown => {
    if (Array.isArray(item)) return item.map(sort);
    if (isRecord(item)) {
      return Object.fromEntries(
        Object.keys(item)
          .sort()
          .map((key) => [key, sort(item[key])]),
      );
    }
    return item;
  };
  return JSON.stringify(sort(value));
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** What a list or report screen is given when it opens: what it kept, and how to keep more. */
export interface MemoryProps {
  /** What the screen kept last time, as it wrote it (unread). */
  saved: unknown;
  /** Keeps a value after a short pause. Null forgets it. */
  remember: (value: unknown) => void;
}

/** A string, or the fallback for anything else. */
export function textOr(value: unknown, fallback: string): string {
  return typeof value === 'string' ? value : fallback;
}

/** One of the allowed texts, or the fallback for anything else. */
export function choiceOr<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return typeof value === 'string' && (allowed as readonly string[]).includes(value)
    ? (value as T)
    : fallback;
}

/** A whole number from 1 up, or null for anything else (an id of a record). */
export function idOrNull(value: unknown): number | null {
  return typeof value === 'number' && Number.isInteger(value) && value > 0 ? value : null;
}

/** True when a value is `true`. */
export function flagOf(value: unknown): boolean {
  return value === true;
}

/** A record a picker holds, as the text kept for it: its id, both names and its code, if any. */
export function pickedText(
  value: { id: number; name_ur: string; name_en: string; code?: string | undefined } | null,
): Record<string, unknown> | null {
  return value === null
    ? null
    : {
        id: value.id,
        name_ur: value.name_ur,
        name_en: value.name_en,
        ...(value.code === undefined ? {} : { code: value.code }),
      };
}

/** A record the pickers keep: its id, both names and its code. Null for anything else. */
export function pickedOrNull(value: unknown): {
  id: number;
  name_ur: string;
  name_en: string;
  code?: string;
} | null {
  if (!isRecord(value)) return null;
  const id = idOrNull(value['id']);
  if (id === null) return null;
  const code = value['code'];
  return {
    id,
    name_ur: textOr(value['name_ur'], ''),
    name_en: textOr(value['name_en'], ''),
    ...(typeof code === 'string' ? { code } : {}),
  };
}
