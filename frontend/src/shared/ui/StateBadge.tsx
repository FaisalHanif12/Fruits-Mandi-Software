// The round icon at the top of the empty and error states (StatesUr board).

import type { LucideIcon } from 'lucide-react';

export type StateBadgeTone = 'primary' | 'due';

interface StateBadgeProps {
  icon: LucideIcon;
  tone: StateBadgeTone;
}

const TONE_CLASSES: Record<StateBadgeTone, string> = {
  primary: 'bg-primary-soft text-primary',
  due: 'bg-due-soft text-due',
};

export function StateBadge({ icon: Icon, tone }: StateBadgeProps) {
  return (
    <div
      className={`flex size-badge shrink-0 items-center justify-center rounded-full ${TONE_CLASSES[tone]}`}
    >
      <Icon aria-hidden="true" className="size-icon-lg" />
    </div>
  );
}
