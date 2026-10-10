// A number inside text or a table cell (Code Standards F17, F25, NFR-URD-13, NFR-USE-19).
// Isolated, left to right, Inter with equal-width digits, so "(16 @ 900)" never flips inside
// Urdu and units line up under units.
//
// Money: pass `paisa`, and it is shown through formatPaisa with the app's grouping. The
// frontend never works out an amount itself (CLAUDE.md rule 1).

import type { ReactNode } from 'react';
import { formatPaisa, useGrouping, type Paisa } from '../format';

interface NumProps {
  /** An amount from the backend. */
  paisa?: Paisa;
  /** A number already made into text, for example a quantity. Fluent may fill it. */
  children?: ReactNode;
}

function Money({ paisa }: { paisa: Paisa }) {
  return formatPaisa(paisa, { grouping: useGrouping() });
}

export function Num({ paisa, children }: NumProps) {
  return (
    <bdi dir="ltr" className="type-number">
      {paisa === undefined ? children : <Money paisa={paisa} />}
    </bdi>
  );
}
