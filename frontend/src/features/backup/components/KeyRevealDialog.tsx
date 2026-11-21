// See the backup key again (PRD FR-20.10 v1.54, Umar 21-11-2026): the Owner lost the printed sheet. The box asks his
// password and his sign (FR-21.4); the backend checks both and answers the key only when both are right. A wrong
// password or sign shows its error at its field and nothing else (the backend checks the password first, so a wrong
// password never says anything about the sign). The key is shown large and set apart, with a Print button that prints
// the same key sheet as at setup, and no copy: the text cannot be selected or copied. It is held only in this box's
// own state, so it leaves the screen, and the memory, the moment the box closes. It is never put in the key store, the
// query cache, a URL or a log.

import { Localized } from '@fluent/react';
import { KeyRound } from 'lucide-react';
import { useEffect, useState, type RefObject } from 'react';
import { errorsForField, uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Code,
  ErrorNotice,
  Field,
  FormDialog,
  PasswordInput,
  TextInput,
  fieldError,
} from '../../../shared/ui';
import '../backup.css';
import { useRevealBackupKey } from '../hooks';

const FIELDS = ['password', 'sign'];

interface KeyRevealDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The button that opened the box: focus goes back to it. */
  returnFocusTo?: RefObject<HTMLElement | null> | undefined;
}

/** The key on screen and on its sheet. It cannot be selected or copied (no clipboard, Umar 21-11-2026). */
function ShownKey({ keyText, onClose }: { keyText: string; onClose: () => void }) {
  const print = () => {
    window.print();
  };
  useShortcut(SHORTCUTS.print, print);
  return (
    <div className="flex flex-col gap-4">
      <p
        data-shown-key=""
        aria-label={keyText}
        onCopy={(event) => {
          event.preventDefault();
        }}
        onCut={(event) => {
          event.preventDefault();
        }}
        onContextMenu={(event) => {
          event.preventDefault();
        }}
        className="type-title select-none rounded-2 border border-border bg-surface-sunken px-4 py-3 text-center tracking-code"
      >
        <Code>{keyText}</Code>
      </p>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.print} onClick={print}>
          <Localized id="backup-key-print" />
        </Button>
        <Button variant="primary" shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="backup-key-reveal-close" />
        </Button>
      </div>
      {/* Drawn on paper only (backup.css): the same sheet as at setup, to be kept in a safe place. */}
      <div data-key-sheet="" className="hidden print:block">
        <Localized id="backup-key-sheet-title">
          <h1 className="type-title" />
        </Localized>
        <p className="type-title tracking-code">
          <Code>{keyText}</Code>
        </p>
        <Localized id="backup-key-sheet-note">
          <p className="type-body" />
        </Localized>
      </div>
    </div>
  );
}

function KeyRevealForm({
  onClose,
  onShown,
}: {
  onClose: () => void;
  onShown: (key: string) => void;
}) {
  const [password, setPassword] = useState('');
  const [sign, setSign] = useState('');
  const reveal = useRevealBackupKey();
  const { reset } = reveal;
  const error = reveal.error === null ? null : uiErrorOf(reveal.error);
  const flight = useSingleFlight();

  // The call is dropped from the query's memory when the box goes away.
  useEffect(() => reset, [reset]);

  const submit = () => {
    if (password === '' || sign.trim() === '') return;
    void flight.run(
      () =>
        reveal
          .mutateAsync({ password, sign: sign.trim() })
          .then((answer) => {
            // Nothing typed stays once the key is shown.
            setPassword('');
            setSign('');
            onShown(answer.key);
          })
          // The refusal is shown at its field.
          .catch(() => undefined),
      undefined,
    );
  };
  const navigation = useFormNavigation(submit);

  // A wrong password is told in its own words: the Owner is signed in already, so "user name" would be wrong.
  const wrong = errorsForField(error, 'password').find(
    (item) => item.code === 'ERR_WRONG_PASSWORD',
  );
  const passwordError =
    wrong === undefined ? (
      fieldError(error, 'password')
    ) : (
      <Localized
        id="backup-key-reveal-wrong-password"
        vars={{ tries_left: wrong.values['tries_left'] ?? 0 }}
      >
        <span className="block" />
      </Localized>
    );

  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
      <Field label={<Localized id="backup-key-reveal-password" />} error={passwordError}>
        {(control) => (
          <PasswordInput
            {...control}
            value={password}
            onChange={setPassword}
            autoComplete="current-password"
          />
        )}
      </Field>
      <Field label={<Localized id="backup-key-reveal-sign" />} error={fieldError(error, 'sign')}>
        {(control) => <TextInput {...control} value={sign} onChange={setSign} />}
      </Field>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          disabled={password === '' || sign.trim() === ''}
          busy={flight.busy || reveal.isPending}
          onClick={submit}
        >
          <KeyRound aria-hidden="true" className="size-icon-md" />
          <Localized id="backup-key-reveal-show" />
        </Button>
      </div>
    </form>
  );
}

export function KeyRevealDialog({ open, onOpenChange, returnFocusTo }: KeyRevealDialogProps) {
  // The key, held here and nowhere else (not in a store, not in the query cache). It is dropped the moment the box
  // closes, by any way: the Close button, Esc, a click outside, or the screen setting `open` to false.
  const [shown, setShown] = useState<string | null>(null);
  const [wasOpen, setWasOpen] = useState(open);
  if (wasOpen !== open) {
    setWasOpen(open);
    if (!open) setShown(null);
  }
  const close = () => {
    setShown(null);
    onOpenChange(false);
  };
  return (
    <FormDialog
      open={open}
      onOpenChange={(next) => {
        if (!next) setShown(null);
        onOpenChange(next);
      }}
      returnFocusTo={returnFocusTo}
      title={<Localized id="backup-key-reveal-title" />}
      description={
        <Localized id={shown === null ? 'backup-key-reveal-help' : 'backup-key-reveal-shown'} />
      }
    >
      {!open ? null : shown === null ? (
        <KeyRevealForm onClose={close} onShown={setShown} />
      ) : (
        <ShownKey keyText={shown} onClose={close} />
      )}
    </FormDialog>
  );
}
