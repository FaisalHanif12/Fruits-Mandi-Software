// A dialog that holds a small form (design system 6): the quick add of an account, for example.
// Built on Radix Dialog: focus stays inside, Esc closes it (after an open suggestion list inside it,
// which closes first, PRD FR-7.18), and focus goes back to where it was
// (the field that opened it). Larger than ConfirmDialog, same look: 12 px radius, the scrim of the
// other dialogs. The words and the buttons are the form's own.

import { Dialog } from 'radix-ui';
import { useRef, type ReactNode, type RefObject } from 'react';
import { focusFirstEmptyField } from '../hooks';
import { useUiLanguage } from '../i18n';
import { ModalLayerContext, runEscape, useModalLayer } from '../keyboard';

interface FormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  description?: ReactNode;
  children: ReactNode;
  /**
   * Where focus goes when the dialog closes, when it is not the element that had focus as it
   * opened. A dialog opened from a menu item needs this: the item is gone by then, and the
   * button that opened the menu is where he expects to be.
   */
  returnFocusTo?: RefObject<HTMLElement | null> | undefined;
  /**
   * The form inside answers Esc itself, through the Esc order of PRD FR-7.18 (it asks "save
   * changes?" when something was typed). Esc and a click on the scrim then never close the dialog
   * by themselves; the form closes it. Used by the Index form a picker opens (plan 32, R8).
   */
  escapeToApp?: boolean | undefined;
  /**
   * `screen`: the dialog is drawn like a screen of its own over the work area, not as a small box
   * with a scrim: opaque, filling `container` (or the window when there is none), with its title as
   * the heading of the page. Used by the Index form a picker opens (plan 32, R8).
   */
  layout?: 'dialog' | 'screen' | undefined;
  /** The element a `screen` dialog fills (the work area of the shell). */
  container?: HTMLElement | null | undefined;
  /**
   * The cursor goes to the first box that is still empty when the dialog opens, because the form
   * starts with what was typed in the picker (the short form of "add it here", plan 41). Without
   * it the first control that can take focus has it.
   */
  focusFirstEmpty?: boolean | undefined;
}

export function FormDialog({
  open,
  onOpenChange,
  title,
  description,
  children,
  returnFocusTo,
  escapeToApp = false,
  layout = 'dialog',
  container,
  focusFirstEmpty = false,
}: FormDialogProps) {
  const asScreen = layout === 'screen';
  const { language, direction } = useUiLanguage();
  // Radix gives focus back only to a trigger button. This dialog is opened by a key in a field, so
  // the field it was opened from is remembered and focused again when the dialog closes.
  const returnFocusRef = useRef<HTMLElement | null>(null);
  // While open, this dialog owns the entry keys (F10, Ctrl+Enter), so the screen under it does
  // not save too (shared/keyboard/modalLayer.ts).
  const layer = useModalLayer(open);
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal {...(asScreen && container != null ? { container } : {})}>
        {asScreen ? null : <Dialog.Overlay className="fixed inset-0 z-40 bg-scrim" />}
        <Dialog.Content
          lang={language}
          dir={direction}
          aria-describedby={undefined}
          onEscapeKeyDown={(event) => {
            if (escapeToApp) {
              event.preventDefault();
              runEscape();
              return;
            }
            // PRD FR-7.18: a list open inside the dialog closes first; the dialog closes on the next Esc.
            if (runEscape('overlay')) event.preventDefault();
          }}
          onInteractOutside={(event) => {
            // A click on the scrim must not drop an Index form with a half-typed record.
            if (escapeToApp) event.preventDefault();
          }}
          onOpenAutoFocus={(event) => {
            const before = document.activeElement;
            returnFocusRef.current = before instanceof HTMLElement ? before : null;
            if (focusFirstEmpty && event.target instanceof HTMLElement) {
              event.preventDefault();
              focusFirstEmptyField(event.target);
            }
          }}
          onCloseAutoFocus={(event) => {
            const target = returnFocusTo?.current ?? returnFocusRef.current;
            if (!target?.isConnected) return;
            event.preventDefault();
            // Another form dialog took this one's place (the link "Full form" of the short form of
            // "add it here", plan 41): the field is under it, and the new dialog keeps the focus.
            const top = Array.from(document.querySelectorAll('[role="dialog"]')).at(-1);
            if (top !== undefined && !top.contains(target)) return;
            target.focus();
          }}
          className={
            asScreen
              ? `pointer-events-auto inset-0 z-40 flex flex-col gap-5 overflow-auto bg-bg p-6 text-ink ${container == null ? 'fixed' : 'absolute'}`
              : 'fixed inset-0 z-50 m-auto flex h-fit max-h-screen w-full max-w-form flex-col gap-3 overflow-y-auto rounded-3 bg-surface p-6 text-ink shadow-overlay'
          }
        >
          {asScreen ? (
            <Dialog.Title asChild>
              <h1 className="type-title">{title}</h1>
            </Dialog.Title>
          ) : (
            <Dialog.Title className="type-section">{title}</Dialog.Title>
          )}
          {description === undefined ? null : (
            <Dialog.Description className="type-body text-ink-muted">
              {description}
            </Dialog.Description>
          )}
          <ModalLayerContext.Provider value={layer}>{children}</ModalLayerContext.Provider>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
