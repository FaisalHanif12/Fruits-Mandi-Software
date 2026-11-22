// "The Owner's sign" in Settings, Security (PRD FR-21.4 and Section 21, v1.36 and v1.55, plans 73 and 92): the word that
// opens a closed day to fix an entry and saves an edit of a وصولی, ادائیگی or پیشگی. The Owner types his password, his
// current sign, and the new sign twice. The sign is plain text, never converted from Roman letters to Urdu, and is
// never shown again: the backend only compares it. The two new signs must be the same: that is told on screen and
// nothing is sent. The backend is the final word: a wrong password, a wrong current sign, and a new sign that is
// empty or too long are told under their fields and nothing changes.

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

const FIELDS = ['password', 'current_sign', 'sign'];

export function OwnerSignCard() {
  const { l10n } = useLocalization();
  const change = useSetOwnerSign();
  const [password, setPassword] = useState('');
  const [currentSign, setCurrentSign] = useState('');
  const [sign, setSign] = useState('');
  const [again, setAgain] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  // The two new signs differ: told under the second one, and nothing is sent.
  const [mismatch, setMismatch] = useState(false);
  const submit = (event?: SyntheticEvent) => {
    event?.preventDefault();
    if (change.isPending) return;
    setError(null);
    if (sign.trim() !== again.trim()) {
      setMismatch(true);
      return;
    }
    change.mutate(
      { password, current_sign: currentSign.trim(), sign: sign.trim() },
      {
        onSuccess: () => {
          // The password and the signs are gone from the screen as soon as they are saved.
          setPassword('');
          setCurrentSign('');
          setSign('');
          setAgain('');
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
    <Card compact icon={PenLine} title={<Localized id="settings-sign-title" />}>
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
        <Field
          label={<Localized id="settings-sign-current" />}
          error={fieldError(error, 'current_sign')}
        >
          {(props) => <TextInput {...props} value={currentSign} onChange={setCurrentSign} />}
        </Field>
        <Field label={<Localized id="settings-sign-new" />} error={fieldError(error, 'sign')}>
          {(props) => (
            <TextInput
              {...props}
              value={sign}
              onChange={(text) => {
                setSign(text);
                setMismatch(false);
              }}
            />
          )}
        </Field>
        <Field
          label={<Localized id="settings-sign-new-again" />}
          error={
            mismatch ? (
              <Localized id="settings-sign-mismatch">
                <span className="block" />
              </Localized>
            ) : undefined
          }
        >
          {(props) => (
            <TextInput
              {...props}
              value={again}
              onChange={(text) => {
                setAgain(text);
                setMismatch(false);
              }}
            />
          )}
        </Field>
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
        <div>
          <Button
            variant="primary"
            busy={change.isPending}
            disabled={
              password === '' ||
              currentSign.trim() === '' ||
              sign.trim() === '' ||
              again.trim() === ''
            }
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
