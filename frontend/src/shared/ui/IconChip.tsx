// The soft chip behind a small picture (plan 75 §7.6): a rounded square in the soft colour of a tone with a
// Lucide icon in it. A tile of the dashboard, a card, the title of a page and a figure that heads a screen
// all use this one, so the picture looks the same everywhere. It only decorates, so a screen reader skips
// it; the words next to it say the same thing (NFR-USE-12: a tone is never the only signal).

import type { LucideIcon } from 'lucide-react';

export type IconChipTone = 'neutral' | 'good' | 'bad' | 'warn';

/** The colours of the chip, one for each tone (the same as the figure beside it, NFR-USE-12). */
const CHIP_TONE: Record<IconChipTone, string> = {
  neutral: 'bg-primary-soft text-primary',
  good: 'bg-money-in-soft text-money-in',
  bad: 'bg-due-soft text-due',
  warn: 'bg-warn-soft text-warn',
};

interface IconChipProps {
  icon: LucideIcon;
  tone?: IconChipTone;
}

export function IconChip({ icon: Icon, tone = 'neutral' }: IconChipProps) {
  return (
    <span
      aria-hidden="true"
      className={`flex size-icon-lg shrink-0 items-center justify-center rounded-2 ${CHIP_TONE[tone]}`}
    >
      <Icon aria-hidden="true" className="size-icon-sm" />
    </span>
  );
}
