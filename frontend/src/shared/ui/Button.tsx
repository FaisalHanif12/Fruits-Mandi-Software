// The one button of the app (design system 6, Code Standards F7, F12). 44 px tall in both
// languages, 8 px radius. A button with a shortcut shows its key in a chip after the label, for
// example "محفوظ کریں F10" (PRD FR-7.4). One primary button per screen, at the end of the
// action row.

import type { ReactNode } from 'react';
import type { Shortcut } from '../keyboard';
import { KeyChip } from './KeyChip';

export type ButtonVariant = 'primary' | 'secondary' | 'danger';

interface ButtonProps {
  children: ReactNode;
  onClick: () => void;
  variant?: ButtonVariant;
  shortcut?: Shortcut;
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  primary: 'border-primary bg-primary text-on-primary',
  secondary: 'border-border-control bg-surface text-ink',
  danger: 'border-due bg-surface text-due',
};

export function Button({ children, onClick, variant = 'secondary', shortcut }: ButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`inline-flex h-control items-center gap-2 rounded-2 border px-4 type-control ${VARIANT_CLASSES[variant]}`}
    >
      {children}
      {shortcut === undefined ? null : (
        <KeyChip shortcut={shortcut} tone={variant === 'primary' ? 'on-primary' : 'on-light'} />
      )}
    </button>
  );
}
