// The firm's names, mandi, address and phone (PRD Section 21 "Firm", plan 04). Both names of
// the firm and the mandi are kept, one per script (PRD FR-7.24). A word of a name made only by the
// spelling rules is checked before the save (PRD FR-7.66). Leaving with unsaved changes asks
// "Save changes?" (PRD FR-7.9).

import { Localized } from '@fluent/react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useLocalization } from '@fluent/react';
import type { FirmSettings } from '../../../bindings/FirmSettings';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { useFormState, useSpellingFlow, useSingleFlight } from '../../../shared/hooks';
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
  type NamePairValue,
} from '../../../shared/ui';
import type { SpellingMarks } from '../../../shared/urdu/spelling';
import { useSetFirm } from '../hooks';

const FIELDS = ['name_ur', 'name_en', 'mandi_ur', 'mandi_en', 'address', 'phone'] as const;

interface FirmTabProps {
  firm: FirmSettings;
}

export function FirmTab({ firm }: FirmTabProps) {
  const { l10n } = useLocalization();
  const { values, setField, setValues } = useFormState<FirmSettings>(firm);
  // The words of each name to check, and the ones he confirmed (PRD FR-7.66). They sit here, with
  // the values, so a language switch loses nothing. They belong to the saved firm they were made
  // against: when a save brings a new `firm`, they are no longer read.
  const [marksOf, setMarksOf] = useState<{
    forFirm: FirmSettings;
    firm?: SpellingMarks | undefined;
    mandi?: SpellingMarks | undefined;
  }>({ forFirm: firm });
  const marks = marksOf.forFirm === firm ? marksOf : { forFirm: firm };
  const rememberMarks = (name: 'firm' | 'mandi') => (next: NamePairValue) => {
    setMarksOf({ ...marks, [name]: next.spelling });
  };
  const formRef = useRef<HTMLFormElement>(null);
  const spelling = useSpellingFlow({
    onFix: () => {
      formRef.current?.querySelector<HTMLElement>('[role="combobox"], input')?.focus();
    },
  });
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

  const firmNames: NamePairValue = {
    urdu: values.name_ur,
    english: values.name_en,
    spelling: marks.firm,
  };
  const mandiNames: NamePairValue = {
    urdu: values.mandi_ur,
    english: values.mandi_en,
    spelling: marks.mandi,
  };

  const submitOnce = (): Promise<boolean> =>
    spelling
      .guard(
        firmNames,
        (next) => {
          rememberMarks('firm')(next);
        },
        (firmWords) =>
          spelling.guard(
            mandiNames,
            (next) => {
              rememberMarks('mandi')(next);
            },
            (mandiWords) =>
              save.mutateAsync({
                ...values,
                confirmed_rules_words: [...new Set([...firmWords, ...mandiWords])],
              }),
          ),
      )
      .then((saved) => {
        if (saved === undefined) return false;
        showToast('success', l10n.getString('settings-saved'));
        return true;
      })
      .catch(() => false);
  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = (): Promise<boolean> => flight.run(submitOnce, false);
  const submitNow = () => {
    void submit();
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);

  return (
    <>
      <form
        ref={formRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex max-w-reading flex-col gap-4"
      >
        <LeaveGuard dirty={dirty} onSave={submit} />
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
        <NamePair
          value={firmNames}
          onChange={(next) => {
            setValues({ ...values, name_ur: next.urdu, name_en: next.english });
            rememberMarks('firm')(next);
          }}
          urduLabel={<Localized id="settings-firm-name-ur" />}
          englishLabel={<Localized id="settings-firm-name-en" />}
          urduError={fieldError(error, 'name_ur')}
          englishError={fieldError(error, 'name_en')}
        />
        <NamePair
          value={mandiNames}
          onChange={(next) => {
            setValues({ ...values, mandi_ur: next.urdu, mandi_en: next.english });
            rememberMarks('mandi')(next);
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
        <Field
          label={<Localized id="settings-firm-address" />}
          error={fieldError(error, 'address')}
        >
          {(control) => (
            <FreeTextInput {...control} value={values.address} onChange={setField('address')} />
          )}
        </Field>
        <div>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={flight.busy || save.isPending}
            disabled={!dirty}
            onClick={submitNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </form>
      {spelling.dialog}
    </>
  );
}
