// Change password, from the user menu (PRD FR-5.6, NFR-SEC-1, NFR-SEC-2). The Owner types the
// password he uses now, then the new one twice. The backend checks all three and is the final
// word: a wrong current password counts towards the same lockout as sign-in, a short new password
// and two different ones are refused with the field they belong to.
//
// Keys: Enter moves to the next field and saves on the last, F10 or Ctrl+S saves, Esc closes. The
// passwords are text in this form's state and are gone when the dialog closes.

import { Localized, useLocalization } from '@fluent/react';
import { useState, type RefObject } from 'react';
import { errorsForField, uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Field,
  FormDialog,
  PasswordInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { PASSWORD_MIN_LENGTH } from '../format';
import { useChangePassword } from '../hooks';

const FIELDS = ['current', 'new', 'repeat'];

interface ChangePasswordDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The user button that opened the menu this came from: focus goes back to it. */
  returnFocusTo?: RefObject<HTMLElement | null> | undefined;
}

function ChangePasswordForm({ onClose }: { onClose: () => void }) {
  const { l10n } = useLocalization();
  const [current, setCurrent] = useState('');
  const [next, setNext] = useState('');
  const [repeat, setRepeat] = useState('');
  const change = useChangePassword();
  const error = change.error === null ? null : uiErrorOf(change.error);

  // One change at a time: a second Enter while one is on its way would count a second wrong
  // try towards the lockout (NFR-REL-1, NFR-SEC-2).
  const flight = useSingleFlight();
  const submit = () => {
    void flight.run(
      () =>
        change
          .mutateAsync({ current, new: next, repeat })
          .then(() => {
            showToast('success', l10n.getString('auth-change-saved'));
            onClose();
          })
          // The refusal is shown in the dialog by its fields.
          .catch(() => undefined),
      undefined,
    );
  };
  const navigation = useFormNavigation(submit);
  useShortcut(SHORTCUTS.save, submit);

  // A wrong current password is told in its own words: "the user name or password" would be
  // wrong here, he is already signed in. The tries left come from the backend.
  const wrong = errorsForField(error, 'current').find((item) => item.code === 'ERR_WRONG_PASSWORD');
  const currentError =
    wrong === undefined ? (
      fieldError(error, 'current')
    ) : (
      <Localized
        id="auth-change-wrong-current"
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
      <Field label={<Localized id="auth-change-current" />} error={currentError}>
        {(control) => (
          <PasswordInput
            {...control}
            value={current}
            onChange={setCurrent}
            autoComplete="current-password"
          />
        )}
      </Field>
      <Field
        label={<Localized id="auth-change-new" />}
        hint={
          <Localized id="auth-wizard-password-hint" vars={{ min: String(PASSWORD_MIN_LENGTH) }} />
        }
        error={fieldError(error, 'new')}
      >
        {(control) => (
          <PasswordInput {...control} value={next} onChange={setNext} autoComplete="new-password" />
        )}
      </Field>
      <Field label={<Localized id="auth-change-repeat" />} error={fieldError(error, 'repeat')}>
        {(control) => (
          <PasswordInput
            {...control}
            value={repeat}
            onChange={setRepeat}
            autoComplete="new-password"
          />
        )}
      </Field>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || change.isPending}
          onClick={submit}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
    </form>
  );
}

export function ChangePasswordDialog({
  open,
  onOpenChange,
  returnFocusTo,
}: ChangePasswordDialogProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      returnFocusTo={returnFocusTo}
      title={<Localized id="auth-change-title" />}
      description={<Localized id="auth-change-help" />}
    >
      {open ? (
        <ChangePasswordForm
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
