// Asks before anything that cannot be undone easily: cancel, day close, restore (PRD FR-7.8,
// design system 6). Says exactly what will happen. Two buttons: back (Esc) and the action.
// Built on Radix AlertDialog: focus stays inside, Esc and back close it, focus returns after.
//
// The action button has focus when the dialog opens, except for a danger action (for example
// cancelling a بکری), where back has focus, so a stray Enter never destroys work. When it
// closes, focus goes back to where it was (the button that opened it), never to the page.

import { Localized } from '@fluent/react';
import { AlertDialog } from 'radix-ui';
import { useRef, type ReactNode } from 'react';
import { useUiLanguage } from '../i18n';
import { SHORTCUTS, type Shortcut } from '../keyboard';
import { Button } from './Button';

export type ConfirmTone = 'primary' | 'danger';

interface ConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  /** What will happen, in plain words. */
  description: ReactNode;
  confirmLabel: ReactNode;
  onConfirm: () => void;
  tone?: ConfirmTone;
  /**
   * Which button has focus when the dialog opens. By default the action, except for a danger
   * action. An action that is not destructive but should not happen by a stray Enter (an app
   * update) asks for `back`.
   */
  initialFocus?: 'action' | 'back';
  confirmShortcut?: Shortcut;
  /**
   * Called when the dialog has closed and focus is about to go back to where it was. A screen that
   * wants focus somewhere else (for example the field to fix) moves it there and calls
   * `event.preventDefault()`, so the default return does not take it back.
   */
  onCloseFocus?: ((event: Event) => void) | undefined;
}

export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel,
  onConfirm,
  tone = 'primary',
  initialFocus,
  confirmShortcut,
  onCloseFocus,
}: ConfirmDialogProps) {
  const { language, direction } = useUiLanguage();
  const backRef = useRef<HTMLButtonElement>(null);
  const confirmRef = useRef<HTMLButtonElement>(null);
  const returnFocusRef = useRef<HTMLElement | null>(null);
  return (
    <AlertDialog.Root open={open} onOpenChange={onOpenChange}>
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="fixed inset-0 z-40 bg-scrim" />
        <AlertDialog.Content
          lang={language}
          dir={direction}
          onOpenAutoFocus={(event) => {
            event.preventDefault();
            const before = document.activeElement;
            returnFocusRef.current = before instanceof HTMLElement ? before : null;
            const toBack = initialFocus === undefined ? tone === 'danger' : initialFocus === 'back';
            (toBack ? backRef : confirmRef).current?.focus();
          }}
          onCloseAutoFocus={(event) => {
            onCloseFocus?.(event);
            if (event.defaultPrevented) return;
            const target = returnFocusRef.current;
            if (!target?.isConnected) return;
            event.preventDefault();
            target.focus();
          }}
          className="fixed inset-0 z-50 m-auto flex h-fit w-full max-w-dialog flex-col gap-3 rounded-3 bg-surface p-6 text-ink shadow-overlay"
        >
          <AlertDialog.Title className="type-section">{title}</AlertDialog.Title>
          <AlertDialog.Description className="type-body text-ink-muted">
            {description}
          </AlertDialog.Description>
          <div className="mt-2 flex flex-wrap justify-end gap-2">
            <Button
              ref={backRef}
              shortcut={SHORTCUTS.back}
              onClick={() => {
                onOpenChange(false);
              }}
            >
              <Localized id="common-back" />
            </Button>
            <Button
              ref={confirmRef}
              variant={tone}
              {...(confirmShortcut === undefined ? {} : { shortcut: confirmShortcut })}
              onClick={() => {
                onConfirm();
                onOpenChange(false);
              }}
            >
              {confirmLabel}
            </Button>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
