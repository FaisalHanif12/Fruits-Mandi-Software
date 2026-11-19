// The state of a row as a quiet chip (plan 75, item 4): a small icon and the word, in a soft tone.
// The word is always there and the icon goes with it, so a state is never told by colour alone
// (NFR-USE-12). The tone only helps the eye: primary for something in order, muted for something
// finished or switched off, amber for something that needs a look. It shows what it is given and
// works out nothing.

import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

export type StatusChipTone = 'primary' | 'muted' | 'warn' | 'due';

const TONE_CLASSES: Record<StatusChipTone, string> = {
  primary: 'bg-primary-soft text-primary',
  muted: 'bg-surface-sunken text-ink-muted',
  warn: 'bg-warn-soft text-warn',
  due: 'bg-due-soft text-due',
};

/** What a list column hands to the grid for one row: the tone and the icon that go with the word. */
export interface StatusChipLook {
  tone: StatusChipTone;
  icon: LucideIcon;
}

interface StatusChipProps extends StatusChipLook {
  /** The word of the state, already in the app language. */
  children: ReactNode;
}

export function StatusChip({ tone, icon: Icon, children }: StatusChipProps) {
  return (
    <span
      className={`type-chip-small inline-flex max-w-full items-center gap-1 rounded-full px-2 ${TONE_CLASSES[tone]}`}
    >
      <Icon aria-hidden="true" className="size-icon-sm shrink-0" />
      <span className="truncate">{children}</span>
    </span>
  );
}
