// A figure in a cell of a table or a grid with a mark or an ⓘ beside it (plan 39 R1, NFR-USE-19): the
// figure stands in the middle of the column, under the middle of its heading, and the mark before it
// (a loss) and the ⓘ after it hang on its sides in the room that is left. Left to right in both
// languages. Nothing here works out anything (CLAUDE.md rule 1).

import type { ReactNode } from 'react';

interface FigureCellProps {
  /** The figure: a `Num`, or a few words of it. It is what the heading stands over. */
  children: ReactNode;
  /** A mark before the figure (a loss triangle). */
  lead?: ReactNode;
  /** What goes after the figure: the ⓘ with its working. */
  extra?: ReactNode;
}

export function FigureCell({ children, lead, extra }: FigureCellProps) {
  return (
    <span className="figure-cell">
      {lead === undefined || lead === null ? null : <span data-slot="lead">{lead}</span>}
      <span data-slot="figure">{children}</span>
      {extra === undefined || extra === null ? null : <span data-slot="extra">{extra}</span>}
    </span>
  );
}
