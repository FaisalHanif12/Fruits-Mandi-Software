// "Change the partner code" in Settings (plan 44, PRD FR-17.11): the old code or the Owner's
// password, then the new code twice. When no code is made yet, the card makes it instead (the
// Owner's password, the code twice). The code is a password: masked while typed, never kept here.

import { Localized } from '@fluent/react';
import { KeyRound } from 'lucide-react';
import { useState, type SyntheticEvent } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  Card,
  ErrorNotice,
  Field,
  PasswordInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useChangePartnerCode, usePartnerCodeStatus } from '../code-hooks';
import { SetCodeCard } from './PartnerCodeCards';

const CODE_LENGTH = 4;
const FIELDS = ['old_code_or_password', 'code', 'code_again'];

function digitsOf(text: string): string {
  return text.replace(/\D/g, '').slice(0, CODE_LENGTH);
}

export function ChangeCodeCard() {
  const status = usePartnerCodeStatus();
  const change = useChangePartnerCode();
  const [old, setOld] = useState('');
  const [code, setCode] = useState('');
  const [again, setAgain] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  const navigation = useFormNavigation(() => undefined);

  // No code yet: the same card as the section's first open makes it.
  if (status.data?.is_set === false) return <SetCodeCard />;
  if (status.data === undefined) return null;

  const submit = (event?: SyntheticEvent) => {
    event?.preventDefault();
    if (change.isPending) return;
    setError(null);
    change.mutate(
      { old_code_or_password: old, code, code_again: again },
      {
        onSuccess: () => {
          setOld('');
          setCode('');
          setAgain('');
          showToast('success', <Localized id="partners-code-change-done" />);
        },
        onError: (failure) => {
          setError(uiErrorOf(failure));
        },
      },
    );
  };

  return (
    <Card compact icon={KeyRound} title={<Localized id="partners-code-change-title" />}>
      <form {...navigation} onSubmit={submit} className="flex max-w-field flex-col gap-3">
        <Field
          label={<Localized id="partners-code-change-old" />}
          error={fieldError(error, 'old_code_or_password')}
        >
          {(props) => (
            <PasswordInput
              {...props}
              value={old}
              onChange={setOld}
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
                setCode(digitsOf(text));
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
            busy={change.isPending}
            disabled={old === '' || code.length !== CODE_LENGTH || again.length !== CODE_LENGTH}
            onClick={submit}
          >
            <KeyRound aria-hidden="true" className="size-icon-md" />
            <Localized id="partners-code-change-button" />
          </Button>
        </div>
      </form>
    </Card>
  );
}
