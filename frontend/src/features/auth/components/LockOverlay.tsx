// The idle lock (PRD FR-8.4, NFR-SEC-3): a dialog over the app that asks for the password. The
// screen underneath stays as it was, with every unsaved value, because it is not unmounted. The
// dialog cannot be closed with Esc or a click outside. Unlocking uses the same wrong-try counter
// as sign-in (plan 04 contract 4).

import { Localized } from '@fluent/react';
import { Dialog } from 'radix-ui';
import { uiErrorOf } from '../../../shared/api';
import { useFormState, useSingleFlight } from '../../../shared/hooks';
import { useUiLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Field,
  PasswordInput,
  ScriptText,
  fieldError,
} from '../../../shared/ui';
import { useLogout, useUnlock } from '../hooks';
import { useLockCountdown } from '../use-lock-countdown';

interface LockOverlayProps {
  /** The signed-in user's name, shown as typed. */
  userName: string;
}

const FIELDS = ['password'] as const;

export function LockOverlay({ userName }: LockOverlayProps) {
  const { language, direction } = useUiLanguage();
  const { values, setField, reset } = useFormState({ password: '' });
  const unlock = useUnlock();
  // One try at a time: three Enter keys in the same instant would count three wrong tries.
  const flight = useSingleFlight();
  const logout = useLogout();
  const rawError = unlock.error === null ? null : uiErrorOf(unlock.error);
  const { shown: error, waiting } = useLockCountdown(rawError, unlock.submittedAt);
  const submit = () => {
    if (values.password === '' || unlock.isPending || waiting) return;
    void flight.run(
      () =>
        unlock
          .mutateAsync({ password: values.password })
          .then(() => {
            reset();
          })
          // The refusal is shown on the form by its fields.
          .catch(() => undefined),
      undefined,
    );
  };
  const navigation = useFormNavigation(submit);
  return (
    <Dialog.Root open modal>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-scrim" />
        <Dialog.Content
          lang={language}
          dir={direction}
          aria-describedby={undefined}
          onEscapeKeyDown={(event) => {
            event.preventDefault();
          }}
          onInteractOutside={(event) => {
            event.preventDefault();
          }}
          className="fixed inset-0 z-50 m-auto flex h-fit w-full max-w-dialog flex-col gap-3 rounded-3 bg-surface p-6 text-ink shadow-overlay"
        >
          <Dialog.Title className="type-section">
            <Localized id="auth-lock-title" />
          </Dialog.Title>
          <Localized id="auth-lock-help">
            <p className="type-body text-ink-muted" />
          </Localized>
          <Localized id="auth-lock-user" vars={{ name: userName }} elems={{ name: <ScriptText /> }}>
            <p className="type-body" />
          </Localized>
          <form
            {...navigation}
            onSubmit={(event) => {
              event.preventDefault();
            }}
            className="flex flex-col gap-3"
          >
            {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
            <Field
              label={<Localized id="auth-login-password" />}
              error={fieldError(error, 'password')}
            >
              {(control) => (
                <PasswordInput
                  {...control}
                  value={values.password}
                  onChange={setField('password')}
                />
              )}
            </Field>
            <div className="flex flex-wrap justify-end gap-2">
              <Button
                onClick={() => {
                  logout.mutate();
                }}
              >
                <Localized id="layout-user-signout" />
              </Button>
              <Button
                variant="primary"
                shortcut={SHORTCUTS.confirm}
                disabled={values.password === '' || waiting}
                busy={flight.busy || unlock.isPending}
                onClick={submit}
              >
                <Localized id="auth-lock-unlock" />
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
