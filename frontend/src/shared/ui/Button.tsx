// The one button of the app (design system 6, Code Standards F7, F12). 44 px tall in both
// languages, 8 px radius. A button with a shortcut shows its key in a chip after the label, for
// example "محفوظ کریں F10" (PRD FR-7.4). One primary button per screen, at the end of the
// action row.

import type { ReactNode, Ref } from 'react';
import type { Shortcut } from '../keyboard';
import { KeyChip } from './KeyChip';

export type ButtonVariant = 'primary' | 'secondary' | 'danger';

interface ButtonProps {
  children: ReactNode;
  onClick: () => void;
  variant?: ButtonVariant;
  shortcut?: Shortcut;
  /** The action cannot run now. The button stays visible, dimmed. */
  disabled?: boolean;
  /** The action is running (for example a save). The button cannot be pressed again. */
  busy?: boolean;
  /**
   * The full name for a screen reader when several buttons with the same words are on one screen,
   * for example "Statement of Partner A". The words on the button stay the short ones.
   */
  accessibleName?: string;
  ref?: Ref<HTMLButtonElement>;
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  primary: 'border-primary bg-primary text-on-primary',
  secondary: 'border-border-control bg-surface text-ink',
  danger: 'border-due bg-surface text-due',
};

export function Button({
  children,
  onClick,
  variant = 'secondary',
  shortcut,
  disabled = false,
  busy = false,
  accessibleName,
  ref,
}: ButtonProps) {
  return (
    <button
      ref={ref}
      type="button"
      onClick={onClick}
      disabled={disabled || busy}
      aria-busy={busy}
      aria-label={accessibleName}
      className={`inline-flex h-control shrink-0 items-center gap-2 rounded-2 border px-4 type-control transition-calm disabled:cursor-not-allowed disabled:opacity-60 ${VARIANT_CLASSES[variant]}`}
    >
      {children}
      {shortcut === undefined ? null : (
        <KeyChip shortcut={shortcut} tone={variant === 'primary' ? 'on-primary' : 'on-light'} />
      )}
    </button>
  );
}
