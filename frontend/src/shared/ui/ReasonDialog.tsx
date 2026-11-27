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
// - `signLabel` adds a second required field below the reason: the Owner's sign (PRD FR-21.4, plan 73).
//   It is raw text, never converted from Roman letters to Urdu (the sign is compared as typed), and Enter
//   in the reason goes on to it. A sign the backend refuses is told under it and nothing opens.
// - `codeLabel` adds a third required field: the 4-digit partner code, masked as it is typed (every cancel
//   asks it, PRD FR-7.93, plan 100). Only the digits are kept. It is a secret: it stays in this box's state
//   and goes only to the command; nothing logs it or keeps it. A refusal of it is told under it.
// - `saveKey` makes F10 act as the action button while the box is open (the box of a saved voucher's change, plan 92):
//   the box takes the entry keys, so the screen under it does not answer F10 as well.

import { Localized } from '@fluent/react';
import { AlertDialog } from 'radix-ui';
import { useRef, useState, type ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { useUiLanguage } from '../i18n';
import {
  ModalLayerContext,
  SHORTCUTS,
  runEscape,
  useFormNavigation,
  useModalLayer,
  useShortcut,
} from '../keyboard';
import { Button, type ButtonVariant } from './Button';
import { ErrorNotice } from './ErrorNotice';
import { Field } from './Field';
import { FreeTextInput } from './FreeTextInput';
import { fieldError } from './field-error';
import { Notice } from './Notice';
import { PasswordInput } from './PasswordInput';
import { TextInput } from './TextInput';

/** The partner code is four digits (PRD FR-17.11). The backend checks it again. */
const CODE_LENGTH = 4;

/** Only the digits of what was typed, at most four (a hint on the screen, not the rule). */
function digitsOf(text: string): string {
  return text.replace(/\D/g, '').slice(0, CODE_LENGTH);
}

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
  /**
   * The action was asked for, with the reason typed (trimmed, never empty) and, when the box has a sign
   * field, the sign as typed (trimmed, never empty) as the second argument, and when it has a code field the
   * four digits as the third. A box without a sign field calls it with the reason alone.
   */
  onConfirm: (reason: string, sign?: string, code?: string) => void;
  /** The label of the sign field. Only a box that asks for the Owner's sign has one. */
  signLabel?: ReactNode;
  /** A line under the sign field: who can sign. */
  signHint?: ReactNode;
  /** The label of the partner code field. Only a box that asks for the code has one. */
  codeLabel?: ReactNode;
  /** A line under the code field. */
  codeHint?: ReactNode;
  /** What the sign and code fields start with, for a box that is opened again after a step between. */
  initialSign?: string;
  initialCode?: string;
  /** Fields the action needs besides the reason, shown above it (for example the quantity). */
  extra?: ReactNode;
  /** The field paths `extra` shows its own problems for, so they are not told twice. */
  extraFields?: readonly string[];
  /** The look of the action button. Default: danger (a cancel). An edit asks with primary. */
  confirmVariant?: ButtonVariant;
  /** The action cannot be taken now (the day is closed): the button stays, dimmed. */
  confirmDisabled?: boolean;
  /** What the reason field starts with, for a box that is opened again after a step between. */
  initialReason?: string;
  /** F10 does what the action button does. Default: no. */
  saveKey?: boolean;
}

function ReasonForm({
  description,
  reasonLabel,
  signLabel,
  signHint,
  codeLabel,
  codeHint,
  initialSign = '',
  initialCode = '',
  confirmLabel,
  error,
  busy,
  note,
  onConfirm,
  extra,
  extraFields = [],
  confirmVariant = 'danger',
  confirmDisabled = false,
  initialReason = '',
  saveKey = false,
  onClose,
}: Omit<ReasonDialogProps, 'open' | 'onOpenChange' | 'title'> & { onClose: () => void }) {
  const [reason, setReason] = useState(initialReason);
  const [sign, setSign] = useState(initialSign);
  const [code, setCode] = useState(initialCode);
  const [tried, setTried] = useState(false);
  const asksSign = signLabel !== undefined;
  const asksCode = codeLabel !== undefined;
  const submit = () => {
    const trimmed = reason.trim();
    const signed = sign.trim();
    if (trimmed === '' || (asksSign && signed === '') || (asksCode && code.length < CODE_LENGTH)) {
      setTried(true);
      return;
    }
    if (asksCode) onConfirm(trimmed, signed, code);
    else if (asksSign) onConfirm(trimmed, signed);
    else onConfirm(trimmed);
  };
  const navigation = useFormNavigation(submit);
  useShortcut(SHORTCUTS.save, submit, { enabled: saveKey });
  const missing = tried && reason.trim() === '';
  const signMissing = tried && asksSign && sign.trim() === '';
  const codeMissing = tried && asksCode && code.length < CODE_LENGTH;
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
      {error === null ? null : (
        <ErrorNotice
          error={error}
          fieldsShown={[
            'reason',
            ...(asksSign ? ['sign'] : []),
            ...(asksCode ? ['partner_code'] : []),
            ...extraFields,
          ]}
        />
      )}
      {note === undefined ? null : <Notice tone="warn">{note}</Notice>}
      {extra}
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
      {asksSign ? (
        <Field
          label={signLabel}
          hint={signHint}
          error={
            signMissing ? (
              <Localized id="err-required">
                <span className="block" />
              </Localized>
            ) : (
              fieldError(error, 'sign')
            )
          }
        >
          {(control) => (
            <TextInput
              {...control}
              value={sign}
              onChange={(text) => {
                setSign(text);
                setTried(false);
              }}
            />
          )}
        </Field>
      ) : null}
      {asksCode ? (
        <Field
          label={codeLabel}
          hint={codeHint}
          error={
            codeMissing ? (
              <Localized id="err-partner-code-invalid">
                <span className="block" />
              </Localized>
            ) : (
              fieldError(error, 'partner_code')
            )
          }
        >
          {(control) => (
            <PasswordInput
              {...control}
              value={code}
              onChange={(text) => {
                setCode(digitsOf(text));
                setTried(false);
              }}
              autoComplete="new-password"
            />
          )}
        </Field>
      ) : null}
      <div className="mt-2 flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant={confirmVariant}
          busy={busy}
          disabled={confirmDisabled}
          {...(saveKey ? { shortcut: SHORTCUTS.save } : {})}
          onClick={submit}
        >
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
  // A box that takes F10 holds the screen's entry keys back while it is open (modalLayer.ts).
  const layer = useModalLayer(open && form.saveKey === true);
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
            <ModalLayerContext.Provider value={form.saveKey === true ? layer : null}>
              <ReasonForm
                {...form}
                onClose={() => {
                  onOpenChange(false);
                }}
              />
            </ModalLayerContext.Provider>
          ) : null}
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
