// An account code or short code, typed as digits (design system 6, PRD FR-7.39). Never grouped
// and never converted to Urdu. The backend resolves short codes on Enter or on leaving the
// field (CLAUDE.md, Gotchas). AccountPicker (plan 05) builds on this.

import { NumberTextInput, type NumberTextInputProps } from './NumberTextInput';

export function CodeInput(props: NumberTextInputProps) {
  return <NumberTextInput {...props} grouping={undefined} inputMode="numeric" />;
}
