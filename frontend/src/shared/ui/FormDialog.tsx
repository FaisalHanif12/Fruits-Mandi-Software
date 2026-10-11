// A dialog that holds a small form (design system 6): the quick add of an account, for example.
// Built on Radix Dialog: focus stays inside, Esc closes it, and focus goes back to where it was
// (the field that opened it). Larger than ConfirmDialog, same look: 12 px radius, the scrim of the
// other dialogs. The words and the buttons are the form's own.

import { Dialog } from 'radix-ui';
import { useRef, type ReactNode } from 'react';
import { useUiLanguage } from '../i18n';

interface FormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  description?: ReactNode;
  children: ReactNode;
}

export function FormDialog({ open, onOpenChange, title, description, children }: FormDialogProps) {
  const { language, direction } = useUiLanguage();
  // Radix gives focus back only to a trigger button. This dialog is opened by a key in a field, so
  // the field it was opened from is remembered and focused again when the dialog closes.
  const returnFocusRef = useRef<HTMLElement | null>(null);
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-scrim" />
        <Dialog.Content
          lang={language}
          dir={direction}
          aria-describedby={undefined}
          onOpenAutoFocus={() => {
            const before = document.activeElement;
            returnFocusRef.current = before instanceof HTMLElement ? before : null;
          }}
          onCloseAutoFocus={(event) => {
            const target = returnFocusRef.current;
            if (!target?.isConnected) return;
            event.preventDefault();
            target.focus();
          }}
          className="fixed inset-0 z-50 m-auto flex h-fit max-h-screen w-full max-w-form flex-col gap-3 overflow-y-auto rounded-3 bg-surface p-6 text-ink shadow-overlay"
        >
          <Dialog.Title className="type-section">{title}</Dialog.Title>
          {description === undefined ? null : (
            <Dialog.Description className="type-body text-ink-muted">
              {description}
            </Dialog.Description>
          )}
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
