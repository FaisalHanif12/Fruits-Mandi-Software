// The box that asks "is this entry correct?" before an entry that needs the user's yes (PRD FR-6.7, FR-7.8; plan 91 for
// a seller's money paid above today's sale). One shell for every such question, so each looks and behaves the same:
// the cash question (`CashConfirmDialog`) and the seller question (`SellerOverConfirmDialog`) only bring their words.
//
// - Two buttons: "ہاں، محفوظ کریں" (Yes, save) and "نہیں، واپس جائیں" (No, go back).
// - Focus starts on No, so pressing Enter out of habit never saves a mistake. Esc is No.
// - The screen decides when to open it (the backend says `needs_confirm`) and what saving means. Every figure in the
//   words is the backend's, shown with formatPaisa; nothing is worked out here (CLAUDE.md rule 1).
// - `onCloseFocus` lets the screen put focus on the field to fix after No (PRD FR-6.7).

import { Localized } from '@fluent/react';
import { AlertDialog } from 'radix-ui';
import { useRef, type ReactNode } from 'react';
import { useUiLanguage } from '../i18n';
import { SHORTCUTS } from '../keyboard';
import { Button } from './Button';

interface ConfirmEntryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The short title, a Fluent message. */
  title: ReactNode;
  /** What will happen and the question, a Fluent message (its child element is needed for `<num>` markup). */
  body: ReactNode;
  /** The user said yes: save with the confirmation. */
  onConfirm: () => void;
  /** Called when the box has closed and focus is about to return; the screen may take it. */
  onCloseFocus?: ((event: Event) => void) | undefined;
}

export function ConfirmEntryDialog({
  open,
  onOpenChange,
  title,
  body,
  onConfirm,
  onCloseFocus,
}: ConfirmEntryDialogProps) {
  const { language, direction } = useUiLanguage();
  const noRef = useRef<HTMLButtonElement>(null);
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
            noRef.current?.focus();
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
          <AlertDialog.Description className="type-body text-ink">{body}</AlertDialog.Description>
          <div className="mt-2 flex flex-wrap justify-end gap-2">
            <Button
              ref={noRef}
              shortcut={SHORTCUTS.back}
              onClick={() => {
                onOpenChange(false);
              }}
            >
              <Localized id="common-cash-confirm-no" />
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                onConfirm();
                onOpenChange(false);
              }}
            >
              <Localized id="common-cash-confirm-yes" />
            </Button>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
