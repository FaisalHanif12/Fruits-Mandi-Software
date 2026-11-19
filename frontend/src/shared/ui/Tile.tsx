// A figure with its name that opens the screen behind it (PRD 9.1, FR-9.1, design system 6): a
// dashboard tile. A label above, the figure in step 4 below, and an optional line under it (a count,
// an overdue amount). The whole tile is one button, so the keyboard and a screen reader treat it as
// one. The tone is always written as well as coloured: the note or the label says it (NFR-USE-12).
// A tile can carry a soft icon chip at the end of its label row (plan 75, item 1): it only decorates,
// so a screen reader skips it, and it takes the tone of the figure.
// `data-nav-field` lets the arrow keys of a form move between tiles in the four directions the user
// sees (PRD FR-7.13). It shows what it is given and works out nothing (CLAUDE.md rule 1).

import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import { IconChip, type IconChipTone } from './IconChip';

export type TileTone = IconChipTone;

const FIGURE_TONE: Record<TileTone, string> = {
  neutral: 'text-ink',
  good: 'text-money-in',
  bad: 'text-due',
  warn: 'text-warn',
};

interface TileProps {
  /** What the figure is: "Cash in hand". */
  label: ReactNode;
  /** The figure: a `Num`, or a short sentence of words. */
  children: ReactNode;
  /** A line under the figure. */
  note?: ReactNode;
  /** A small picture of what the tile is about, in a soft chip. It decorates and says nothing. */
  icon?: LucideIcon;
  tone?: TileTone;
  /** The figure is a few words (a state), not a number: written at the size of a field. */
  words?: boolean;
  /** Opens the screen or report behind the tile. */
  onOpen: () => void;
  /** The full name for a screen reader, when the label alone is not enough. */
  accessibleName?: string;
}

export function Tile({
  label,
  children,
  note,
  icon: Icon,
  tone = 'neutral',
  words = false,
  onOpen,
  accessibleName,
}: TileProps) {
  return (
    <button
      type="button"
      data-nav-field
      onClick={onOpen}
      aria-label={accessibleName}
      className="flex min-w-0 flex-col items-stretch gap-1 rounded-3 border border-border bg-surface p-3 text-start transition-calm hover:border-primary-chip hover:bg-row-current"
    >
      <span className="flex items-start justify-between gap-2">
        <span className="type-label min-w-0 text-ink-muted">{label}</span>
        {Icon === undefined ? null : <IconChip icon={Icon} tone={tone} />}
      </span>
      <span
        className={`${words ? 'type-field' : 'type-figure'} min-w-0 break-words ${FIGURE_TONE[tone]}`}
      >
        {children}
      </span>
      {note === undefined ? null : <span className="type-body text-ink-muted">{note}</span>}
    </button>
  );
}
