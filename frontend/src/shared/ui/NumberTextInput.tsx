// The base of MoneyInput, QtyInput, DateInput and CodeInput (design system 6): Inter with
// equal-width digits, written left to right and set against the physical right edge in both
// languages (NFR-USE-19, Code Standards F17).
//
// The value is always TEXT: the backend parses it and is the only judge of it (Code Standards
// M8, 4.2, CLAUDE.md rule 1). Two kinds of comma, kept apart (plan 02 4.7, finding R02-6):
// - A comma the field puts in by itself for display (PRD FR-7.20) is never sent. The user types
//   12000, sees 12,000, and the value is "12000".
// - A comma the user types or pastes is sent exactly as typed, and the screen shows the user's
//   own text. "1,200" is sent as "1,200" (the backend accepts it). "850,50" is sent as "850,50"
//   and the backend refuses it with ERR_NOT_A_NUMBER. The field never drops it, because that
//   would turn 850 rupees 50 paisa into 85,050 rupees.
// Only plain digits are shown grouped. Anything else ("20+25", Urdu digits, "15000.50", text with
// a typed comma) is shown and sent as it is, so the backend can read it or say what is wrong
// (FR-7.48, E31). The text cursor stays after the same character when a separator appears or goes.

import { useLayoutEffect, useRef } from 'react';
import { groupDigits, type Grouping } from '../format';
import { controlClasses } from './control-classes';
import type { FieldControlProps } from './Field';

const SEPARATOR = ',';
const PLAIN_DIGITS = /^[0-9]+$/;

export interface NumberTextInputProps extends FieldControlProps {
  value: string;
  onChange: (text: string) => void;
  placeholder?: string;
}

interface BaseProps extends NumberTextInputProps {
  /** Grouping to show while typing; undefined shows the text as typed. */
  grouping?: Grouping | undefined;
  inputMode: 'numeric' | 'text';
}

/** The text on screen for a value: grouped when it is plain digits, else the value itself. */
export function displayText(value: string, grouping: Grouping | undefined): string {
  if (grouping === undefined || !PLAIN_DIGITS.test(value)) return value;
  return groupDigits(value, grouping);
}

/** The part of `after` that is new compared with `before`: the text the user typed or pasted. */
function insertedText(before: string, after: string): string {
  const shortest = Math.min(before.length, after.length);
  let prefix = 0;
  while (prefix < shortest && before[prefix] === after[prefix]) prefix += 1;
  let suffix = 0;
  while (
    suffix < shortest - prefix &&
    before[before.length - 1 - suffix] === after[after.length - 1 - suffix]
  ) {
    suffix += 1;
  }
  return after.slice(prefix, after.length - suffix);
}

export interface Typing {
  /** The value to send. */
  value: string;
  /** How many characters of the value are before the cursor. */
  kept: number;
}

/**
 * What an edit means. `shown` is the text on screen before the edit, `value` the value it came
 * from, `text` the input's text after the edit and `caret` the cursor in it.
 *
 * If the field grouped the digits for display, the commas that were already there are its own and
 * are taken out. If the edit added a comma, the user typed it: the text is kept exactly as it is
 * on screen, including the field's own commas around it, and the backend checks it. Example:
 * shown "12,000" and the user adds "5" gives "120005"; the user adds "," at the end gives "12,000,".
 */
export function readTyping(
  shown: string,
  value: string,
  text: string,
  caret: number,
  grouping: Grouping | undefined,
): Typing {
  const asShown = { value: text, kept: caret };
  if (grouping === undefined || shown === value) return asShown;
  if (insertedText(shown, text).includes(SEPARATOR)) return asShown;
  return {
    value: text.replaceAll(SEPARATOR, ''),
    kept: text.slice(0, caret).replaceAll(SEPARATOR, '').length,
  };
}

/** Where the cursor goes in `text` so that `kept` characters other than separators are before it. */
function caretAfter(text: string, kept: number): number {
  let seen = 0;
  for (let index = 0; index < text.length; index += 1) {
    if (seen === kept) return index;
    if (text[index] !== SEPARATOR) seen += 1;
  }
  return text.length;
}

interface PendingCaret {
  kept: number;
  /** True when the field's own commas are in the text, so they must not be counted. */
  grouped: boolean;
}

export function NumberTextInput({
  id,
  'aria-describedby': describedBy,
  invalid,
  readOnly,
  value,
  onChange,
  placeholder,
  grouping,
  inputMode,
}: BaseProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const pendingCaret = useRef<PendingCaret | null>(null);
  const shown = displayText(value, grouping);

  useLayoutEffect(() => {
    const input = inputRef.current;
    const pending = pendingCaret.current;
    if (input === null || pending === null) return;
    pendingCaret.current = null;
    const position = pending.grouped
      ? caretAfter(input.value, pending.kept)
      : Math.min(pending.kept, input.value.length);
    input.setSelectionRange(position, position);
  });

  return (
    <input
      ref={inputRef}
      id={id}
      type="text"
      dir="ltr"
      lang="en"
      inputMode={inputMode}
      value={shown}
      placeholder={placeholder}
      readOnly={readOnly}
      aria-invalid={invalid}
      aria-describedby={describedBy}
      autoComplete="off"
      spellCheck={false}
      onChange={(event) => {
        const text = event.target.value;
        const caret = event.target.selectionStart ?? text.length;
        const typing = readTyping(shown, value, text, caret, grouping);
        pendingCaret.current = {
          kept: typing.kept,
          grouped: displayText(typing.value, grouping) !== typing.value,
        };
        onChange(typing.value);
      }}
      className={`${controlClasses({ invalid, readOnly })} type-number-field`}
    />
  );
}
