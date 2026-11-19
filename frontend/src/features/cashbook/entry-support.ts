// Small pieces the three entry screens share (Code Standards G6): what the preview found as a
// refusal the fields can show, and a draft that follows the type in the address.

import { useState } from 'react';
import type { FieldError } from '../../bindings/FieldError';
import type { UiError } from '../../bindings/UiError';

/** Codes that only say a field is empty. Held back until the user tries to save. */
const INCOMPLETE: ReadonlySet<string> = new Set([
  'ERR_REQUIRED',
  'ERR_REASON_REQUIRED',
  'ERR_PARTNER_REQUIRED',
  'ERR_EXPENSE_HEAD_REQUIRED',
  'ERR_LENT_PERSON_REQUIRED',
  'ERR_KHATA_REQUIRED',
]);

/**
 * What the preview found, in the shape of a refused save, so the fields can show it. Problems that
 * only say "this is empty" wait until the user tries to save, so a field does not turn red before
 * he has reached it. Null when there is nothing to show.
 */
export function previewRefusal(
  errors: readonly FieldError[] | undefined,
  tried: boolean,
): UiError | null {
  const shown = (errors ?? []).filter((item) => tried || !INCOMPLETE.has(item.code));
  return shown.length === 0
    ? null
    : {
        ok: false,
        errors: shown.map((item) => ({ code: item.code, field: item.field, values: item.values })),
        ref: '',
      };
}

/**
 * The type in the address (Alt+K, the type list, a search result) is the type of the form. When it
 * changes, `apply` is called once, while drawing, the way React advises for a value that follows a
 * prop. `apply` changes the screen's draft.
 */
export function useFollowType<T extends string>(urlType: T, apply: (type: T) => void): void {
  const [seen, setSeen] = useState(urlType);
  if (seen !== urlType) {
    setSeen(urlType);
    apply(urlType);
  }
}

/**
 * The input of a save command is the entry without its type tag: the command name already says
 * the type, and the backend's own request has no tag (plan 17 request files).
 * Example: `{ type: 'shop_expense', amount: '3,000' }` is sent as `{ amount: '3,000' }`.
 */
export function inputOf<T extends { type: string }>(entry: T): Omit<T, 'type'> {
  return Object.fromEntries(Object.entries(entry).filter(([key]) => key !== 'type')) as Omit<
    T,
    'type'
  >;
}
