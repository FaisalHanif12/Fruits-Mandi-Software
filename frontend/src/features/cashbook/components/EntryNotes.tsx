// What the backend says about the entry on the form, in words under it (PRD FR-6.7, FR-7.68): the
// warnings that still let the user save, any note the screen adds (the right-type note), and the
// amber note when the entry takes cash in hand below zero, with the cash after it. Every figure is
// the backend's (CLAUDE.md rule 1).

import type { ReactNode } from 'react';
import type { CashCheck } from '../../../bindings/CashCheck';
import type { FieldError } from '../../../bindings/FieldError';
import { CashNote, ErrorMessage, Notice } from '../../../shared/ui';

interface EntryNotesProps {
  warnings: readonly FieldError[];
  cash: CashCheck | undefined;
  /** Notes of the screen itself, such as the "is this a partner's own spending?" note. */
  children?: ReactNode;
}

export function EntryNotes({ warnings, cash, children }: EntryNotesProps) {
  return (
    <div className="flex flex-col gap-3">
      {warnings.map((warning, index) => (
        <Notice key={`${warning.code}-${String(index)}`} tone="warn">
          <ErrorMessage item={warning}>
            <span />
          </ErrorMessage>
        </Notice>
      ))}
      {children}
      {cash === undefined ? null : <CashNote cash={cash} />}
    </div>
  );
}
