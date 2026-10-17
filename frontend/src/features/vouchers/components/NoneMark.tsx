// "Nothing" in a table cell: a dash that is also written for a screen reader, so the cell is never an
// empty sound (NFR-USE-12). Used where a row of a panel is not touched by the amount typed.

import { Localized } from '@fluent/react';
import { Minus } from 'lucide-react';

export function NoneMark() {
  return (
    <span className="text-ink-muted">
      <Minus aria-hidden="true" className="size-icon-sm" />
      <Localized id="vouchers-effect-none">
        <span className="sr-only" />
      </Localized>
    </span>
  );
}
