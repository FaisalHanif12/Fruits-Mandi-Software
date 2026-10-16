// The key of a shortcut, shown after a button's label: "محفوظ کریں F10" (PRD FR-7.4,
// design system 6). 22 px tall, Inter semibold, always left to right.

import type { Shortcut } from '../keyboard';

export type KeyChipTone = 'on-primary' | 'on-light';

interface KeyChipProps {
  shortcut: Shortcut;
  tone: KeyChipTone;
}

const TONE_CLASSES: Record<KeyChipTone, string> = {
  'on-primary': 'bg-primary-chip text-on-primary',
  'on-light': 'bg-surface-sunken text-ink',
};

export function KeyChip({ shortcut, tone }: KeyChipProps) {
  return (
    <kbd
      lang="en"
      dir="ltr"
      className={`inline-flex h-chip items-center rounded-1 px-2 type-key ${TONE_CLASSES[tone]}`}
    >
      {shortcut.label}
    </kbd>
  );
}
