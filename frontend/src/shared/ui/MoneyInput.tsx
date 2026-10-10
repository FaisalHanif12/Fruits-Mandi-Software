// An amount in whole rupees, typed as text (design system 6, Code Standards M8 and 4.2).
// The frontend never reads it as a number: the backend parses it with parse_rupees and answers
// with paisa or an error such as ERR_DECIMALS_NOT_ALLOWED (CLAUDE.md rule 1).

import { useGrouping } from '../format';
import { NumberTextInput, type NumberTextInputProps } from './NumberTextInput';

export function MoneyInput(props: NumberTextInputProps) {
  return <NumberTextInput {...props} grouping={useGrouping()} inputMode="numeric" />;
}
