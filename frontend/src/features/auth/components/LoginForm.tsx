// Sign in (PRD Section 8.1, FR-8.2, board LoginUr and LoginEn). User name, password with a show
// eye, Enter signs in, and Exit closes the app. A wrong password shows under the password field
// with the tries left. After the fifth wrong try the backend says how many seconds to wait, and
// this screen counts them down by itself and keeps the button off until the wait is over.
// Nothing here checks a password: the backend does (NFR-SEC-1, NFR-SEC-2).

import { Localized } from '@fluent/react';
import { exitApp } from '../../../app/api';
import { uiErrorOf } from '../../../shared/api';
import { useFocusFirstField, useFormState, useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Field,
  LinkButton,
  PasswordInput,
  TextInput,
  fieldError,
} from '../../../shared/ui';
import { useLogin } from '../hooks';
import { useLockCountdown } from '../use-lock-countdown';

interface LoginFormProps {
  onForgot: () => void;
}

const FIELDS = ['password'] as const;

export function LoginForm({ onForgot }: LoginFormProps) {
  const { values, setField } = useFormState({ username: '', password: '' });
  const login = useLogin();
  // One try at a time: three Enter keys in the same instant would count three wrong tries.
  const flight = useSingleFlight();
  const firstField = useFocusFirstField<HTMLFormElement>();
  const error = login.error === null ? null : uiErrorOf(login.error);
  const { shown: shownWithCountdown, waiting } = useLockCountdown(error, login.submittedAt);
  const canSubmit =
    values.username !== '' &&
    values.password !== '' &&
    !waiting &&
    !login.isPending &&
    !flight.busy;
  const submit = () => {
    if (!canSubmit) return;
    void flight.run(
      () =>
        login
          .mutateAsync({ username: values.username, password: values.password })
          // The refusal is shown on the form by its fields.
          .catch(() => undefined),
      undefined,
    );
  };
  const navigation = useFormNavigation(submit);

  return (
    <form
      ref={firstField}
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      <Localized id="auth-login-title">
        <h1 className="type-title" />
      </Localized>
      {shownWithCountdown === null ? null : (
        <ErrorNotice error={shownWithCountdown} fieldsShown={FIELDS} />
      )}
      <Field label={<Localized id="auth-login-username" />}>
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
        label={<Localized id="auth-login-password" />}
        error={fieldError(shownWithCountdown, 'password')}
      >
        {(control) => (
          <PasswordInput {...control} value={values.password} onChange={setField('password')} />
        )}
      </Field>
      <div className="flex flex-wrap gap-2">
        <Button
          variant="primary"
          shortcut={SHORTCUTS.confirm}
          disabled={!canSubmit}
          busy={flight.busy || login.isPending}
          onClick={submit}
        >
          <Localized id="auth-login-submit" />
        </Button>
        <Button
          onClick={() => {
            void exitApp();
          }}
        >
          <Localized id="auth-login-exit" />
        </Button>
      </div>
      <LinkButton onClick={onForgot}>
        <Localized id="auth-login-forgot" />
      </LinkButton>
    </form>
  );
}
