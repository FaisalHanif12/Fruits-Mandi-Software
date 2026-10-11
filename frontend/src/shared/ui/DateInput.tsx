// A date as DD-MM-YYYY text (design system 6, Tech Stack 4.5). The backend reads it with
// parse_ddmmyyyy, including the shortcuts of PRD FR-7.19 ("10" and "10-9"), and checks the
// open-day rule (FR-6.6). The + and - keys move the date by one day (FR-7.19); arrows still move
// between fields.

import { shiftDateText } from '../format';
import { SHORTCUTS, matchesShortcut } from '../keyboard';
import { NumberTextInput, type NumberTextInputProps } from './NumberTextInput';

export function DateInput(props: NumberTextInputProps) {
  const { value, onChange, readOnly } = props;
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
        event.preventDefault();
        onChange(shiftDateText(value, days, new Date()));
      }}
    />
  );
}
