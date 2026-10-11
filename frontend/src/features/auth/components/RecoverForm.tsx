// Open with the recovery code (plan 04, PRD NFR-SEC-12): user name, the recovery code, and a new
// password twice. Only the Owner can do this. A used code is spent, so the backend sends a new
// one, and the user must print it before he goes on.

import { Localized } from '@fluent/react';
import { uiErrorOf } from '../../../shared/api';
import { useFocusFirstField, useFormState } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Field,
  PasswordInput,
  TextInput,
  fieldError,
} from '../../../shared/ui';
import { useRecover } from '../hooks';
import { useLockCountdown } from '../use-lock-countdown';
import { RecoveryCodeCard } from './RecoveryCodeCard';

interface RecoverFormProps {
  onBack: () => void;
}

const FIELDS = ['username', 'recovery_code', 'new_password', 'new_password_repeat'] as const;

export function RecoverForm({ onBack }: RecoverFormProps) {
  const { values, setField, reset } = useFormState({
    username: '',
    recovery_code: '',
    new_password: '',
    new_password_repeat: '',
  });
  const recover = useRecover();
  const firstField = useFocusFirstField<HTMLFormElement>();
  const rawError = recover.error === null ? null : uiErrorOf(recover.error);
  const { shown: error, waiting } = useLockCountdown(rawError, recover.submittedAt);
  const submit = () => {
    if (recover.isPending || waiting) return;
    recover.mutate({
      username: values.username,
      recovery_code: values.recovery_code,
      new_password: values.new_password,
      new_password_repeat: values.new_password_repeat,
    });
  };
  const navigation = useFormNavigation(submit);

  if (recover.data !== undefined) {
    return (
      <RecoveryCodeCard
        code={recover.data.new_recovery_code}
        titleId="auth-recover-done-title"
        helpId="auth-recover-done-help"
        finishId="auth-recover-done-continue"
        onFinish={() => {
          // The passwords are not kept after they have been used.
          reset();
          onBack();
        }}
      />
    );
  }

  return (
    <form
      ref={firstField}
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      <Localized id="auth-recover-title">
        <h1 className="type-title" />
      </Localized>
      <Localized id="auth-recover-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
      <Field label={<Localized id="auth-login-username" />} error={fieldError(error, 'username')}>
        {(control) => (
          <TextInput
            {...control}
            value={values.username}
            onChange={setField('username')}
            lang="en"
          />
        )}
      </Field>
      <Field
        label={<Localized id="auth-recover-code" />}
        hint={<Localized id="auth-recover-code-hint" />}
        error={fieldError(error, 'recovery_code')}
      >
        {(control) => (
          <TextInput
            {...control}
            value={values.recovery_code}
            onChange={setField('recovery_code')}
            lang="en"
          />
        )}
      </Field>
      <Field
        label={<Localized id="auth-recover-new-password" />}
        error={fieldError(error, 'new_password')}
      >
        {(control) => (
          <PasswordInput
            {...control}
            value={values.new_password}
            onChange={setField('new_password')}
            autoComplete="new-password"
          />
        )}
      </Field>
      <Field
        label={<Localized id="auth-recover-new-password-repeat" />}
        error={fieldError(error, 'new_password_repeat')}
      >
        {(control) => (
          <PasswordInput
            {...control}
            value={values.new_password_repeat}
            onChange={setField('new_password_repeat')}
            autoComplete="new-password"
          />
        )}
      </Field>
      <div className="flex flex-wrap gap-2">
        <Button
          variant="primary"
          shortcut={SHORTCUTS.confirm}
          disabled={waiting}
          busy={recover.isPending}
          onClick={submit}
        >
          <Localized id="auth-recover-submit" />
        </Button>
        <Button shortcut={SHORTCUTS.back} onClick={onBack}>
          <Localized id="common-back" />
        </Button>
      </div>
    </form>
  );
}
