// The cards of the partner code (plan 44, PRD FR-17.11): make the code the first time (it needs the
// Owner's password), open the section with it, and make a new one when it is forgotten (the Owner's
// password or the recovery code). Four digits, typed twice when made. The backend counts wrong tries
// and the wait after five; a wrong code says how many tries are left. The code is a password: it is
// masked while typed, and nothing here keeps it.

import { Localized } from '@fluent/react';
import { KeyRound, LockKeyhole } from 'lucide-react';
import { useState, type SyntheticEvent } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useCountdown } from '../../../shared/hooks';
import { useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  Card,
  ErrorNotice,
  Field,
  LinkButton,
  PasswordInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useResetPartnerCode, useSetPartnerCode, useUnlockPartners } from '../code-hooks';

/** A code is four digits. The backend checks it again (ERR_PARTNER_CODE_INVALID). */
const CODE_LENGTH = 4;

/** Only the digits of what was typed, at most four (a hint on the screen, not the rule). */
function digitsOf(text: string): string {
  return text.replace(/\D/g, '').slice(0, CODE_LENGTH);
}

const FIELDS = ['code', 'code_again', 'password', 'old_code_or_password', 'password_or_recovery'];

interface UnlockCardProps {
  /** Seconds to wait before another try, from the status; 0 when none. */
  waitSeconds: number;
  onForgot: () => void;
  /** The section opened. */
  onOpened?: () => void;
}

/** The box that opens the section: the code, and "forgot the code". */
export function UnlockCard({ waitSeconds, onForgot, onOpened }: UnlockCardProps) {
  const unlock = useUnlockPartners();
  const [code, setCode] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  const navigation = useFormNavigation(() => undefined);
  // The wait counts down by itself after the backend has said how long it is.
  const left = useCountdown(waitSeconds, waitSeconds);
  const waiting = waitSeconds > 0 && left > 0;

  const submit = (event?: SyntheticEvent) => {
    event?.preventDefault();
    if (waiting || unlock.isPending) return;
    setError(null);
    unlock.mutate(code, {
      onSuccess: () => {
        setCode('');
        onOpened?.();
      },
      onError: (failure) => {
        setCode('');
        setError(uiErrorOf(failure));
      },
    });
  };

  return (
    <Card compact title={<Localized id="partners-code-locked-title" />}>
      <form {...navigation} onSubmit={submit} className="flex max-w-field flex-col gap-3">
        <Localized id="partners-code-locked-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        <Field label={<Localized id="partners-code-field" />} error={fieldError(error, 'code')}>
          {(props) => (
            <PasswordInput
              {...props}
              value={code}
              onChange={(text) => {
                setCode(digitsOf(text));
              }}
              autoComplete="current-password"
            />
          )}
        </Field>
        {error === null ? null : <ErrorNotice error={error} fieldsShown={['code']} />}
        {waiting ? (
          <Localized id="partners-code-wait" vars={{ seconds: String(left) }}>
            <p className="type-body text-due" role="status" />
          </Localized>
        ) : null}
        <div className="flex flex-wrap items-center gap-3">
          <Button
            variant="primary"
            busy={unlock.isPending}
            disabled={waiting || code.length !== CODE_LENGTH}
            onClick={submit}
          >
            <LockKeyhole aria-hidden="true" className="size-icon-md" />
            <Localized id="partners-code-open" />
          </Button>
          <LinkButton onClick={onForgot}>
            <Localized id="partners-code-forgot" />
          </LinkButton>
        </div>
      </form>
    </Card>
  );
}

interface SetCodeCardProps {
  onDone?: () => void;
}

/** The first time: the Owner's password, then the new code twice. */
export function SetCodeCard({ onDone }: SetCodeCardProps) {
  const setCode = useSetPartnerCode();
  const [password, setPassword] = useState('');
  const [code, setNewCode] = useState('');
  const [again, setAgain] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  const navigation = useFormNavigation(() => undefined);

  const submit = (event?: SyntheticEvent) => {
    event?.preventDefault();
    if (setCode.isPending) return;
    setError(null);
    setCode.mutate(
      { password, code, code_again: again },
      {
        onSuccess: () => {
          setPassword('');
          setNewCode('');
          setAgain('');
          showToast('success', <Localized id="partners-code-set-done" />);
          onDone?.();
        },
        onError: (failure) => {
          setError(uiErrorOf(failure));
        },
      },
    );
  };

  return (
    <Card compact title={<Localized id="partners-code-set-title" />}>
      <form {...navigation} onSubmit={submit} className="flex max-w-field flex-col gap-3">
        <Localized id="partners-code-set-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        <Field
          label={<Localized id="partners-code-password" />}
          error={fieldError(error, 'password')}
        >
          {(props) => (
            <PasswordInput
              {...props}
              value={password}
              onChange={setPassword}
              autoComplete="current-password"
            />
          )}
        </Field>
        <Field label={<Localized id="partners-code-new" />} error={fieldError(error, 'code')}>
          {(props) => (
            <PasswordInput
              {...props}
              value={code}
              onChange={(text) => {
                setNewCode(digitsOf(text));
              }}
              autoComplete="new-password"
            />
          )}
        </Field>
        <Field
          label={<Localized id="partners-code-again" />}
          error={fieldError(error, 'code_again')}
        >
          {(props) => (
            <PasswordInput
              {...props}
              value={again}
              onChange={(text) => {
                setAgain(digitsOf(text));
              }}
              autoComplete="new-password"
            />
          )}
        </Field>
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
        <div>
          <Button
            variant="primary"
            busy={setCode.isPending}
            disabled={
              password === '' || code.length !== CODE_LENGTH || again.length !== CODE_LENGTH
            }
            onClick={submit}
          >
            <KeyRound aria-hidden="true" className="size-icon-md" />
            <Localized id="partners-code-set-button" />
          </Button>
        </div>
      </form>
    </Card>
  );
}

interface ResetCodeCardProps {
  onBack: () => void;
  onDone?: () => void;
}

/** A forgotten code: the Owner's password or the recovery code, then a new code twice. */
export function ResetCodeCard({ onBack, onDone }: ResetCodeCardProps) {
  const reset = useResetPartnerCode();
  const [secret, setSecret] = useState('');
  const [code, setNewCode] = useState('');
  const [again, setAgain] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  const navigation = useFormNavigation(() => undefined);

  const submit = (event?: SyntheticEvent) => {
    event?.preventDefault();
    if (reset.isPending) return;
    setError(null);
    reset.mutate(
      { password_or_recovery: secret, code, code_again: again },
      {
        onSuccess: () => {
          setSecret('');
          setNewCode('');
          setAgain('');
          showToast('success', <Localized id="partners-code-reset-done" />);
          onDone?.();
        },
        onError: (failure) => {
          setError(uiErrorOf(failure));
        },
      },
    );
  };

  return (
    <Card compact title={<Localized id="partners-code-reset-title" />}>
      <form {...navigation} onSubmit={submit} className="flex max-w-field flex-col gap-3">
        <Localized id="partners-code-reset-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        <Field
          label={<Localized id="partners-code-reset-secret" />}
          error={fieldError(error, 'password_or_recovery')}
        >
          {(props) => (
            <PasswordInput
              {...props}
              value={secret}
              onChange={setSecret}
              autoComplete="current-password"
            />
          )}
        </Field>
        <Field label={<Localized id="partners-code-new" />} error={fieldError(error, 'code')}>
          {(props) => (
            <PasswordInput
              {...props}
              value={code}
              onChange={(text) => {
                setNewCode(digitsOf(text));
              }}
              autoComplete="new-password"
            />
          )}
        </Field>
        <Field
          label={<Localized id="partners-code-again" />}
          error={fieldError(error, 'code_again')}
        >
          {(props) => (
            <PasswordInput
              {...props}
              value={again}
              onChange={(text) => {
                setAgain(digitsOf(text));
              }}
              autoComplete="new-password"
            />
          )}
        </Field>
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
        <div className="flex flex-wrap items-center gap-3">
          <Button
            variant="primary"
            busy={reset.isPending}
            disabled={secret === '' || code.length !== CODE_LENGTH || again.length !== CODE_LENGTH}
            onClick={submit}
          >
            <KeyRound aria-hidden="true" className="size-icon-md" />
            <Localized id="partners-code-reset-button" />
          </Button>
          <LinkButton onClick={onBack}>
            <Localized id="common-back" />
          </LinkButton>
        </div>
      </form>
    </Card>
  );
}
