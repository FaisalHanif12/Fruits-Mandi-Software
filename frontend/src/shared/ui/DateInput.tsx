// A date as DD-MM-YYYY text (design system 6, Tech Stack 4.5). The backend reads it with
// parse_ddmmyyyy, including the shortcuts of PRD FR-7.19 ("10" and "10-9"), and checks the
// open-day rule (FR-6.6). The + and - keys move the date by one day (FR-7.19); arrows still move
// between fields.
//
// The two keys are shortcuts only while the field is empty or all of its text is selected, which is how
// it stands when the user arrives on it (FR-7.15). Once a digit has been typed, "-" is the dash of
// "5-9" or "05-10-2026" and is typed like any character, and "+" is typed too; otherwise the dash could
// never be typed and a date could only be pasted (robustness finding, plan 72). After a day is moved,
// the whole text is selected again, so the key can be pressed again to move on by one more day.

import { useLayoutEffect, useRef } from 'react';
import { shiftDateText } from '../format';
import { SHORTCUTS, matchesShortcut } from '../keyboard';
import { NumberTextInput, type NumberTextInputProps } from './NumberTextInput';

/** True when the key would replace the whole text (nothing typed yet, or everything selected). */
function wholeTextSelected(input: HTMLInputElement): boolean {
  if (input.value === '') return true;
  return input.selectionStart === 0 && input.selectionEnd === input.value.length;
}

export function DateInput(props: NumberTextInputProps) {
  const { value, onChange, readOnly } = props;
  // The input that just moved a day: its whole text is selected again once the new text is drawn.
  const reselect = useRef<HTMLInputElement | null>(null);
  useLayoutEffect(() => {
    const input = reselect.current;
    if (input === null) return;
    reselect.current = null;
    input.select();
  });
  return (
    <NumberTextInput
      {...props}
      grouping={undefined}
      inputMode="text"
      onKeyDown={(event) => {
        if (readOnly === true) return;
        const native = event.nativeEvent;
        const days = matchesShortcut(native, SHORTCUTS.dateNextDay)
          ? 1
          : matchesShortcut(native, SHORTCUTS.datePreviousDay)
            ? -1
            : null;
        if (days === null) return;
        // Typed into a field that has text: the key is a character of the date, not a shortcut.
        if (!wholeTextSelected(event.currentTarget)) return;
        event.preventDefault();
        reselect.current = event.currentTarget;
        onChange(shiftDateText(value, days, new Date()));
      }}
    />
  );
}
