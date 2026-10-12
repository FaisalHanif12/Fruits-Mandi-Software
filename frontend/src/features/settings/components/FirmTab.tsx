// The firm's names, mandi, address and phone (PRD Section 21 "Firm", plan 04). Both names of
// the firm and the mandi are kept, one per script (PRD FR-7.24). Leaving with unsaved changes asks
// "Save changes?" (PRD FR-7.9).

import { Localized } from '@fluent/react';
import { useEffect, useMemo } from 'react';
import { useLocalization } from '@fluent/react';
import type { FirmSettings } from '../../../bindings/FirmSettings';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { useFormState } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Field,
  fieldError,
  FreeTextInput,
  NamePair,
  showToast,
  TextInput,
} from '../../../shared/ui';
import { useSetFirm } from '../hooks';

const FIELDS = ['name_ur', 'name_en', 'mandi_ur', 'mandi_en', 'address', 'phone'] as const;

interface FirmTabProps {
  firm: FirmSettings;
}

export function FirmTab({ firm }: FirmTabProps) {
  const { l10n } = useLocalization();
  const { values, setField, setValues } = useFormState<FirmSettings>(firm);
  const save = useSetFirm();
  const error = save.error === null ? null : uiErrorOf(save.error);
  const dirty = useMemo(
    () => FIELDS.some((field) => values[field] !== firm[field]),
    [values, firm],
  );

  // The saved values arrived again (after a save): the form shows exactly them.
  useEffect(() => {
    setValues(firm);
  }, [firm, setValues]);

  const submit = (): Promise<boolean> =>
    save
      .mutateAsync(values)
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
      className="flex max-w-2xl flex-col gap-4"
    >
      <LeaveGuard dirty={dirty} onSave={submit} />
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
      <NamePair
        value={{ urdu: values.name_ur, english: values.name_en }}
        onChange={(next) => {
          setValues({ ...values, name_ur: next.urdu, name_en: next.english });
        }}
        urduLabel={<Localized id="settings-firm-name-ur" />}
        englishLabel={<Localized id="settings-firm-name-en" />}
        urduError={fieldError(error, 'name_ur')}
        englishError={fieldError(error, 'name_en')}
      />
      <NamePair
        value={{ urdu: values.mandi_ur, english: values.mandi_en }}
        onChange={(next) => {
          setValues({ ...values, mandi_ur: next.urdu, mandi_en: next.english });
        }}
        urduLabel={<Localized id="settings-firm-mandi-ur" />}
        englishLabel={<Localized id="settings-firm-mandi-en" />}
        urduError={fieldError(error, 'mandi_ur')}
        englishError={fieldError(error, 'mandi_en')}
      />
      <Field label={<Localized id="settings-firm-phone" />} error={fieldError(error, 'phone')}>
        {(control) => (
          <TextInput {...control} value={values.phone} onChange={setField('phone')} lang="en" />
        )}
      </Field>
      <Field label={<Localized id="settings-firm-address" />} error={fieldError(error, 'address')}>
        {(control) => (
          <FreeTextInput {...control} value={values.address} onChange={setField('address')} />
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
