// A label with a value that is shown, not typed (plan 13: a saved بکری opens read-only). The look is
// a Field without the box: the label above in the label style, the value below in the field text
// size. The value can be an amount (`Num`), a code, a name, or a few words. Several facts sit in a
// grid of the screen's own; nothing here works out anything (CLAUDE.md rule 1).
//
// `stat` (plan 75, item 7) draws it as a small card, like a tile of the dashboard: the label above in
// the quiet ink and the figure under it in the figure style (step 4, Inter, equal-width digits). It is
// for a figure that heads a screen (the cash at the start and at the end), not for a line of a form.

import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import { IconChip } from './IconChip';

interface FactProps {
  label: ReactNode;
  /** A small card with a bigger figure, for a figure that heads a screen. */
  stat?: boolean;
  /** A small picture in a soft chip at the end of the label row, like a tile of the dashboard. Only with `stat`. */
  icon?: LucideIcon;
  children: ReactNode;
}

export function Fact({ label, stat = false, icon, children }: FactProps) {
  return (
    <div
      className={`flex min-w-0 flex-col gap-1 ${stat ? 'rounded-2 border border-border bg-surface px-3 py-2' : ''}`}
    >
      {stat && icon !== undefined ? (
        <dt className="type-label flex items-start justify-between gap-2 text-ink-muted">
          <span className="min-w-0">{label}</span>
          <IconChip icon={icon} />
        </dt>
      ) : (
        <dt className="type-label text-ink-muted">{label}</dt>
      )}
      <dd className={`${stat ? 'type-figure' : 'type-field'} min-w-0 break-words text-ink`}>
        {children}
      </dd>
    </div>
  );
}
