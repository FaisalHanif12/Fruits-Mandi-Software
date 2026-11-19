// The small "Before 600" next to a field that an edit changed (plan 44, PRD FR-7.71). It is words,
// not only a colour (NFR-USE-12), and its tip has the whole sentence with the time and the name of
// the edit: "Before: 600 → now: 610, 31-10-2026 4:35 pm, Owner". It is drawn from the history of the
// entry (the audit log) and cannot be typed or removed. A field that was never changed has none.

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import type { EntryMarks, MarkText } from '../edit/useEntryMarks';

interface FieldMarkProps {
  mark: MarkText | undefined;
}

export function FieldMark({ mark }: FieldMarkProps) {
  if (mark === undefined) return null;
  return (
    <Localized id="edit-mark-before" vars={{ before: mark.before }}>
      <span
        title={mark.tip}
        data-field-mark=""
        className="inline-flex max-w-full items-center rounded-1 bg-primary-soft px-2 type-chip text-primary"
      />
    </Localized>
  );
}

/**
 * The mark of the field at a stored path, or nothing. A form gives it to its `Field` as `mark`, so a
 * field that was never changed is drawn exactly as before.
 */
export function fieldMark(marks: EntryMarks | undefined, path: string): ReactNode {
  const mark = marks?.of(path);
  return mark === undefined ? undefined : <FieldMark mark={mark} />;
}
