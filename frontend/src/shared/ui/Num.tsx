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
  /**
   * Written at the size of the text around it, not at the 16 px of a number: for a figure that is
   * already in a bigger role (a dashboard tile). The digits are still equal-width and isolated.
   */
  inherit?: boolean;
  /** A number already made into text, for example a quantity. Fluent may fill it. */
  children?: ReactNode;
}

function Money({ paisa }: { paisa: Paisa }) {
  return formatPaisa(paisa, { grouping: useGrouping() });
}

/**
 * The elements for `<Localized elems={RUNNING_NUMBER}>`: they turn each `<num>…</num>` of a Fluent
 * message into a number at the size of the words around it (Inter, equal-width digits, isolated), so a
 * count or a date inside a line of words never depends on the font of the words (Code Standards F25).
 */
export const RUNNING_NUMBER = { num: <Num inherit /> };

export function Num({ paisa, inherit = false, children }: NumProps) {
  return (
    <bdi dir="ltr" className={inherit ? 'number-run' : 'type-number'}>
      {paisa === undefined ? children : <Money paisa={paisa} />}
    </bdi>
  );
}
