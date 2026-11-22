// The one button of the app (design system 6, Code Standards F7, F12). 44 px tall in both
// languages, 8 px radius. A button with a shortcut shows its key in a chip after the label, for
// example "محفوظ کریں F10" (PRD FR-7.4). One primary button per screen, at the end of the
// action row.

import type { ReactNode, Ref } from 'react';
import type { Shortcut } from '../keyboard';
import { KeyChip } from './KeyChip';

export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost';

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
  /**
   * A button that holds only an icon (the arrows that turn a page): as wide as it is tall, so a row
   * of them is narrow. Its name is a visually hidden text inside it (plan 38).
   */
  iconOnly?: boolean;
  /** Fully rounded (a pill, or a circle with `iconOnly`), the look of the chat window (plan 51). */
  round?: boolean;
  /**
   * The smallest size a target may have (32 px, NFR-USE-23) instead of the 44 px of a button, for an icon
   * button that stands beside a grid, such as the "+" under it. Its tall sibling is `iconOnly`: with
   * both it is a 32 px circle.
   */
  small?: boolean;
  /**
   * A second, smaller line under the label (the days the week button goes to, plan 72). The button
   * grows to hold two lines; it is still at least as tall as every other button.
   */
  detail?: ReactNode;
  /** The id of text that explains the button, for example why it is off (read after its name). */
  describedBy?: string;
  /** A button that opens and shuts a part of the page: whether it is open now (read by a screen reader). */
  expanded?: boolean;
  /** The id of the part of the page that an `expanded` button opens and shuts. */
  controls?: string;
  /** The tip that shows when the button is pointed at; it is the same words as its accessible name. */
  title?: string;
  /**
   * In the top row of the sale book, when the row is short of room (`BikriToolbar` marks it `data-compact`), the
   * button gives up half its side padding and its key chip, so an icon and a short word take as little as they can
   * (plan 95; the key stays in the tip and the name the screen gives it).
   */
  tightInCompactRow?: boolean;
  ref?: Ref<HTMLButtonElement>;
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  primary: 'border-primary bg-primary text-on-primary',
  secondary: 'border-border-control bg-surface text-ink',
  danger: 'border-due bg-surface text-due',
  // No box until it is pointed at: the close and size buttons of a window header.
  ghost: 'border-transparent bg-transparent text-ink hover:bg-primary-soft',
};

export function Button({
  children,
  onClick,
  variant = 'secondary',
  shortcut,
  disabled = false,
  busy = false,
  accessibleName,
  iconOnly = false,
  round = false,
  small = false,
  detail,
  describedBy,
  expanded,
  controls,
  title,
  tightInCompactRow = false,
  ref,
}: ButtonProps) {
  const chip =
    shortcut === undefined ? null : (
      <KeyChip
        shortcut={shortcut}
        tone={variant === 'primary' ? 'on-primary' : 'on-light'}
        className={tightInCompactRow ? 'group-data-[compact]/toolbar:hidden' : ''}
      />
    );
  return (
    <button
      ref={ref}
      type="button"
      onClick={onClick}
      disabled={disabled || busy}
      aria-busy={busy}
      aria-label={accessibleName}
      aria-describedby={describedBy}
      aria-expanded={expanded}
      aria-controls={controls}
      title={title}
      className={`inline-flex ${detail === undefined ? (small ? 'h-target' : 'h-control') : 'min-h-control py-2'} shrink-0 items-center gap-2 ${round ? 'rounded-full' : 'rounded-2'} border type-control transition-calm disabled:cursor-not-allowed disabled:opacity-60 ${iconOnly ? `${small ? 'w-target' : 'w-control'} justify-center px-0` : `px-4 ${tightInCompactRow ? 'group-data-[compact]/toolbar:px-2' : ''}`} ${VARIANT_CLASSES[variant]}`}
    >
      {detail === undefined ? (
        <>
          {children}
          {chip}
        </>
      ) : (
        <span className="flex flex-col items-start gap-1 text-start">
          <span className="inline-flex items-center gap-2">
            {children}
            {chip}
          </span>
          <span className="type-chip font-normal text-ink-muted">{detail}</span>
        </span>
      )}
    </button>
  );
}
