// Auto lock: how many minutes of no key or click before the app locks (PRD FR-8.4, plan 04
// `settings_set_security`). The number is sent as typed; the backend checks the range
// (ERR_OUT_OF_RANGE) and is the final word. Owner only.

import { Localized, useLocalization } from '@fluent/react';
import { useEffect } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { useFormState } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import { Button, ErrorNotice, Field, QtyInput, fieldError, showToast } from '../../../shared/ui';
import { useSetSecurity } from '../hooks';

interface LockTabProps {
  minutes: number;
}

export function LockTab({ minutes }: LockTabProps) {
  const { l10n } = useLocalization();
  const saved = String(minutes);
  const { values, setField, setValues } = useFormState({ idle_lock_minutes: saved });
  const save = useSetSecurity();
  const error = save.error === null ? null : uiErrorOf(save.error);
  const dirty = values.idle_lock_minutes !== saved;

  useEffect(() => {
    setValues({ idle_lock_minutes: saved });
  }, [saved, setValues]);

  const submit = (): Promise<boolean> =>
    save
      .mutateAsync({ idle_lock_minutes: values.idle_lock_minutes })
      .then(() => {
        showToast('success', l10n.getString('settings-saved'));
        return true;
      })
      .catch(() => false);
  const submitNow = () => {
    void submit();
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);

  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex max-w-md flex-col gap-4"
    >
      <LeaveGuard dirty={dirty} onSave={submit} />
      {error === null ? null : <ErrorNotice error={error} fieldsShown={['idle_lock_minutes']} />}
      <Field
        label={<Localized id="settings-lock-minutes" />}
        hint={<Localized id="settings-lock-hint" />}
        error={fieldError(error, 'idle_lock_minutes')}
      >
        {(control) => (
          <QtyInput
            {...control}
            value={values.idle_lock_minutes}
            onChange={setField('idle_lock_minutes')}
          />
        )}
      </Field>
      <div>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={save.isPending}
          disabled={!dirty}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
    </form>
  );
}
