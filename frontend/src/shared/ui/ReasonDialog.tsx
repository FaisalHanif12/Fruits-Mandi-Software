// Asks for a reason before something that cannot be undone easily: cancelling a voucher or a
// بکری (PRD FR-7.8, FR-15.3). It says exactly what will happen, takes a reason in a text field, and
// has two buttons: back (Esc) and the action. Built on Radix AlertDialog like ConfirmDialog: focus
// stays inside, Esc and back close it, focus returns to where it was.
//
// - Focus starts in the reason field. A stray Enter cannot destroy anything, because an empty
//   reason is told at the field and nothing is sent.
// - The dialog does not close itself on the action: the screen sends the cancel and closes the
//   dialog when the backend has said yes. A refusal is shown inside the dialog (`error`), and
//   whatever the user typed stays.
// - `note` is for a line the backend asked the screen to show, such as the cash after a cancel
//   that takes cash below zero (PRD FR-6.7): the next press of the action then confirms it.

import { Localized } from '@fluent/react';
import { AlertDialog } from 'radix-ui';
import { useRef, useState, type ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { useUiLanguage } from '../i18n';
import { SHORTCUTS, runEscape, useFormNavigation } from '../keyboard';
import { Button } from './Button';
import { ErrorNotice } from './ErrorNotice';
import { Field } from './Field';
import { FreeTextInput } from './FreeTextInput';
import { fieldError } from './field-error';
import { Notice } from './Notice';

interface ReasonDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  /** What will happen, in plain words. */
  description: ReactNode;
  /** The label of the reason field. */
  reasonLabel: ReactNode;
  confirmLabel: ReactNode;
  /** What the backend refused, to show inside the dialog. */
  error: UiError | null;
  /** The action is running. */
  busy: boolean;
  /** Something the user must read before the action, for example the cash after it. */
  note?: ReactNode;
  /** The action was asked for, with the reason typed (trimmed, never empty). */
  onConfirm: (reason: string) => void;
}

function ReasonForm({
  description,
  reasonLabel,
  confirmLabel,
  error,
  busy,
  note,
  onConfirm,
  onClose,
}: Omit<ReasonDialogProps, 'open' | 'onOpenChange' | 'title'> & { onClose: () => void }) {
  const [reason, setReason] = useState('');
  const [tried, setTried] = useState(false);
  const submit = () => {
    const trimmed = reason.trim();
    if (trimmed === '') {
      setTried(true);
      return;
    }
    onConfirm(trimmed);
  };
  const navigation = useFormNavigation(submit);
  const missing = tried && reason.trim() === '';
  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-3"
    >
      <AlertDialog.Description className="type-body text-ink-muted">
        {description}
      </AlertDialog.Description>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={['reason']} />}
      {note === undefined ? null : <Notice tone="warn">{note}</Notice>}
      <Field
        label={reasonLabel}
        error={
          missing ? (
            <Localized id="err-required">
              <span className="block" />
            </Localized>
          ) : (
            fieldError(error, 'reason')
          )
        }
      >
        {(control) => (
          <FreeTextInput
            {...control}
            value={reason}
            onChange={(text) => {
              setReason(text);
              setTried(false);
            }}
          />
        )}
      </Field>
      <div className="mt-2 flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button variant="danger" busy={busy} onClick={submit}>
          {confirmLabel}
        </Button>
      </div>
    </form>
  );
}

export function ReasonDialog({ open, onOpenChange, title, ...form }: ReasonDialogProps) {
  const { language, direction } = useUiLanguage();
  const returnFocusRef = useRef<HTMLElement | null>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  return (
    <AlertDialog.Root open={open} onOpenChange={onOpenChange}>
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="fixed inset-0 z-40 bg-scrim" />
        <AlertDialog.Content
          ref={contentRef}
          lang={language}
          dir={direction}
          onEscapeKeyDown={(event) => {
            // An open suggestion list in the reason field closes first (PRD FR-7.18).
            if (runEscape('overlay')) event.preventDefault();
          }}
          onOpenAutoFocus={(event) => {
            const before = document.activeElement;
            returnFocusRef.current = before instanceof HTMLElement ? before : null;
            // Focus goes to the reason field, the first thing to fill in.
            const field = contentRef.current?.querySelector('input');
            if (field instanceof HTMLElement) {
              event.preventDefault();
              field.focus();
            }
          }}
          onCloseAutoFocus={(event) => {
            const target = returnFocusRef.current;
            if (!target?.isConnected) return;
            event.preventDefault();
            target.focus();
          }}
          className="fixed inset-0 z-50 m-auto flex h-fit max-h-screen w-full max-w-dialog flex-col gap-3 overflow-y-auto rounded-3 bg-surface p-6 text-ink shadow-overlay"
        >
          <AlertDialog.Title className="type-section">{title}</AlertDialog.Title>
          {open ? (
            <ReasonForm
              {...form}
              onClose={() => {
                onOpenChange(false);
              }}
            />
          ) : null}
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
