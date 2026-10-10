// A date as DD-MM-YYYY text (design system 6, Tech Stack 4.5). The backend reads it with
// parse_ddmmyyyy, including the shortcuts of PRD FR-7.19 ("10" and "10-9"), and checks the
// open-day rule (FR-6.6). The + and - keys for next and previous day come with the keyboard
// engine (plan 04).

import { NumberTextInput, type NumberTextInputProps } from './NumberTextInput';

export function DateInput(props: NumberTextInputProps) {
  return <NumberTextInput {...props} grouping={undefined} inputMode="text" />;
}
