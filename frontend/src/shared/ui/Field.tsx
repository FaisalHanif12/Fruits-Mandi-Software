// A form field: label above, the control, then a hint or an error below (design system 6,
// Code Standards F16, PRD FR-7.1 and FR-7.6). The label sits at the start edge (the right edge
// in Urdu). An error replaces the hint, in red, with an icon and plain words, so colour is
// never the only signal (NFR-USE-12).
//
// The control is given as a function, so the field can pass it the id and the aria links:
//   <Field label={...} hint={...}>{(control) => <MoneyInput {...control} value={v} onChange={set} />}</Field>

import { CircleAlert } from 'lucide-react';
import { useId, type ReactNode } from 'react';

/** What a field passes to its control. Every input of shared/ui accepts these. */
export interface FieldControlProps {
  id?: string | undefined;
  'aria-describedby'?: string | undefined;
  invalid?: boolean | undefined;
  readOnly?: boolean | undefined;
}

interface FieldProps {
  label: ReactNode;
  /** A short grey line under the field saying what to type (PRD FR-7.1). */
  hint?: ReactNode;
  /** What is wrong and what to do (PRD FR-7.6). Shown instead of the hint. */
  error?: ReactNode;
  /** A value the app works out: shown on the page ground, cannot be typed into. */
  readOnly?: boolean;
  children: (control: FieldControlProps) => ReactNode;
}

export function Field({ label, hint, error, readOnly = false, children }: FieldProps) {
  const id = useId();
  const noteId = `${id}-note`;
  const hasError = error !== undefined && error !== null && error !== false;
  const hasHint = hint !== undefined && hint !== null && hint !== false;
  const note = hasError || hasHint;
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <label htmlFor={id} className="type-label text-ink">
        {label}
      </label>
      {children({
        id,
        'aria-describedby': note ? noteId : undefined,
        invalid: hasError,
        readOnly,
      })}
      {hasError ? (
        <p id={noteId} className="flex items-center gap-2 type-body text-due">
          <CircleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
          <span>{error}</span>
        </p>
      ) : hasHint ? (
        <p id={noteId} className="type-body text-ink-muted">
          {hint}
        </p>
      ) : null}
    </div>
  );
}
