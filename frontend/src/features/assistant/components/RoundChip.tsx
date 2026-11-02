// A round chip (plan 51): a quick question, a suggested next question, the button that folds the quick
// questions. White with a navy word and a fine border, fully rounded, as the chips of a chat widget.
// One look for all of them, so they cannot drift apart (Code Standards F12).

import type { ReactNode } from 'react';

interface RoundChipProps {
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  /** The chip opens and closes something: says whether it is open. */
  expanded?: boolean;
  /**
   * A round chip with only an icon, as wide as it is tall, for the box's row (plan 51). Its name is
   * given as `accessibleName`, and shows as a tooltip.
   */
  accessibleName?: string;
}

export function RoundChip({
  children,
  onClick,
  disabled = false,
  expanded,
  accessibleName,
}: RoundChipProps) {
  const iconOnly = accessibleName !== undefined;
  return (
    <button
      type="button"
      disabled={disabled}
      aria-expanded={expanded}
      aria-label={accessibleName}
      title={accessibleName}
      onClick={onClick}
      className={`inline-flex items-center gap-2 rounded-full border border-border-control type-chip text-primary transition-calm hover:bg-primary-soft disabled:cursor-not-allowed disabled:opacity-60 ${expanded === true ? 'bg-primary-soft' : 'bg-surface'} ${iconOnly ? 'size-control shrink-0 justify-center px-0' : 'min-h-control-compact px-4'}`}
    >
      {children}
    </button>
  );
}
