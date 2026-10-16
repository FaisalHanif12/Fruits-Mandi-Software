// A quantity in نگ, typed as text (design system 6, Code Standards 4.2). A simple sum such as
// 20+25 is allowed and worked out by the backend (PRD FR-7.48).

import { useGrouping } from '../format';
import { NumberTextInput, type NumberTextInputProps } from './NumberTextInput';

export function QtyInput(props: NumberTextInputProps) {
  return <NumberTextInput {...props} grouping={useGrouping()} inputMode="numeric" />;
}
