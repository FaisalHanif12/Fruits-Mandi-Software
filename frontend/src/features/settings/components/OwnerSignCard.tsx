// "The Owner's sign" in Settings, Security (PRD FR-21.4 and Section 21, v1.36, plan 73): the word that opens a
// closed day to fix an entry. The Owner types his password and the new sign. The sign is plain text, never
// converted from Roman letters to Urdu, and is never shown again: the backend only compares it when a day is
// reopened. The backend is the final word: a wrong password and a sign that is empty or too long are told
// under their fields and nothing changes.

import { Localized, useLocalization } from '@fluent/react';
import { PenLine } from 'lucide-react';
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
  TextInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useSetOwnerSign } from '../hooks';

const FIELDS = ['password', 'sign'];

export function OwnerSignCard() {
  const { l10n } = useLocalization();
  const change = useSetOwnerSign();
  const [password, setPassword] = useState('');
  const [sign, setSign] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  const submit = (event?: SyntheticEvent) => {
    event?.preventDefault();
    if (change.isPending) return;
    setError(null);
    change.mutate(
      { password, sign: sign.trim() },
      {
        onSuccess: () => {
          // The password and the sign are gone from the screen as soon as they are saved.
          setPassword('');
          setSign('');
          showToast('success', l10n.getString('settings-sign-saved'));
        },
        onError: (failure) => {
          setError(uiErrorOf(failure));
        },
      },
    );
  };
  const navigation = useFormNavigation(submit);

  return (
    <Card compact title={<Localized id="settings-sign-title" />}>
      <form {...navigation} onSubmit={submit} className="flex max-w-field flex-col gap-3">
        <Localized id="settings-sign-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
        <Field
          label={<Localized id="settings-sign-password" />}
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
        <Field label={<Localized id="settings-sign-new" />} error={fieldError(error, 'sign')}>
          {(props) => <TextInput {...props} value={sign} onChange={setSign} />}
        </Field>
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
        <div>
          <Button
            variant="primary"
            busy={change.isPending}
            disabled={password === '' || sign.trim() === ''}
            onClick={submit}
          >
            <PenLine aria-hidden="true" className="size-icon-md" />
            <Localized id="settings-sign-button" />
          </Button>
        </div>
      </form>
    </Card>
  );
}
