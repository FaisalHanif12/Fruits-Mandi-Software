// A form field: label above, the control, then a hint or an error below (design system 6,
// Code Standards F16, PRD FR-7.1 and FR-7.6). The label sits at the start edge (the right edge
// in Urdu). An error replaces the hint, in red, with an icon and plain words, so colour is
// never the only signal (NFR-USE-12). An `inline` field has its label before the control, on one
// line, for a field that sits in a row of buttons (the date of the sale book, plan 71 H4).
//
// The control is given as a function, so the field can pass it the id and the aria links:
//   <Field label={...} hint={...}>{(control) => <MoneyInput {...control} value={v} onChange={set} />}</Field>

import { CircleAlert } from 'lucide-react';
import { useId, type ReactNode } from 'react';
import { InfoHint } from './InfoHint';

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
  /**
   * The same kind of words, behind a small ⓘ next to the label instead of a line under the field,
   * so the form stays short (plan 38 G5). Use it for a hint that is not needed every time.
   */
  info?: ReactNode;
  /** What is wrong and what to do (PRD FR-7.6). Shown instead of the hint. */
  error?: ReactNode;
  /**
   * The mark of an edit on this field ("Before 600", plan 44, PRD FR-7.71), beside the label. Always
   * words, so colour is never the only signal.
   */
  mark?: ReactNode;
  /** A value the app works out: shown on the page ground, cannot be typed into. */
  readOnly?: boolean;
  /**
   * The label is read by a screen reader and not drawn, for a field in a list whose column
   * heading is its label (the share of each partner). Every field still has a label (F28).
   */
  hideLabel?: boolean;
  /** The label stands before the control on one line, so the field is as high as a button. */
  inline?: boolean;
  children: (control: FieldControlProps) => ReactNode;
}

export function Field({
  label,
  hint,
  info,
  error,
  mark,
  readOnly = false,
  hideLabel = false,
  inline = false,
  children,
}: FieldProps) {
  const id = useId();
  const noteId = `${id}-note`;
  const hasError = error !== undefined && error !== null && error !== false;
  const hasHint = hint !== undefined && hint !== null && hint !== false;
  const note = hasError || hasHint;
  const hasMark = mark !== undefined && mark !== null && mark !== false;
  const head =
    info === undefined && !hasMark ? (
      <label htmlFor={id} className={hideLabel ? 'sr-only' : 'type-label text-ink'}>
        {label}
      </label>
    ) : (
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
        <label htmlFor={id} className={hideLabel ? 'sr-only' : 'type-label text-ink'}>
          {label}
        </label>
        {info === undefined ? null : <InfoHint>{info}</InfoHint>}
        {mark}
      </div>
    );
  const control = children({
    id,
    'aria-describedby': note ? noteId : undefined,
    invalid: hasError,
    readOnly,
  });
  return (
    <div className={inline ? 'flex min-w-0 flex-col gap-1' : 'flex min-w-0 flex-col gap-2'}>
      {inline ? (
        <div className="flex min-w-0 items-center gap-2">
          {head}
          {control}
        </div>
      ) : (
        <>
          {head}
          {control}
        </>
      )}
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
