// A white card on the page ground (design system 6): 1 px border, 12 px radius, 24 px padding.
// No coloured side borders.

import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import { IconChip } from './IconChip';

interface CardProps {
  /** Optional card title, step 3 of the type scale. */
  title?: ReactNode;
  /** A small picture in a soft chip before the title (plan 75 §7.6). It decorates and says nothing. */
  icon?: LucideIcon;
  /** Less room inside, for a card that holds the fields of an entry screen (the بکری header). */
  compact?: boolean;
  /**
   * The card takes the height that is left in its column, for a card that holds a list (plan 38 G2).
   * It never shrinks below its content.
   */
  fill?: boolean;
  children: ReactNode;
}

export function Card({ title, icon, compact = false, fill = false, children }: CardProps) {
  return (
    <section
      className={`flex min-w-0 flex-col rounded-3 border border-border bg-surface ${compact ? 'gap-3 p-3' : 'gap-4 p-5'} ${fill ? 'flex-1' : ''}`}
    >
      {title === undefined ? null : icon === undefined ? (
        <h2 className="type-section text-ink">{title}</h2>
      ) : (
        <div className="flex items-center gap-2">
          <IconChip icon={icon} />
          <h2 className="type-section min-w-0 text-ink">{title}</h2>
        </div>
      )}
      {children}
    </section>
  );
}
