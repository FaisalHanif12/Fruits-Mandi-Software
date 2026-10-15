// A percent typed as text, for example a partner's share "40" or "33.33" (design system 6, PRD
// FR-17.1). It looks like the other number fields: Inter with equal-width digits, written left to
// right and set against the physical right edge of the field in both languages. The percent sign
// sits after the field and is not part of the text.
//
// The value is always TEXT. The frontend never reads it as a number: the backend parses it with
// parse_bps (up to 2 decimals, 0 to 100) and says what is wrong (CLAUDE.md rule 1, Code Standards
// M8). Nothing is added up here.

import { Localized } from '@fluent/react';
import { NumberTextInput, type NumberTextInputProps } from './NumberTextInput';

export function PercentInput(props: NumberTextInputProps) {
  return (
    // dir=ltr keeps "40 %" in this order in both languages (F25).
    <div dir="ltr" className="flex min-w-0 items-center gap-2">
      <NumberTextInput {...props} grouping={undefined} inputMode="text" />
      <Localized id="common-percent-sign">
        <span aria-hidden="true" className="type-number text-ink-muted" />
      </Localized>
    </div>
  );
}
