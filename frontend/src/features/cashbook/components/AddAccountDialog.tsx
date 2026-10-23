// Adds a person who can borrow (ادھار رقم, PRD 16.5) or a new الگ کھاتہ (PRD 16.6) without leaving
// the screen: `F4` in the picker, or the button of an empty result, opens this small form with the
// typed text filled in as a name. The new account comes back into the picker.
//
// - The names go through NamePair, and a word made by the spelling rules alone is checked before
//   the save (useSpellingFlow, PRD FR-7.66).
// - A person has a phone and may be a staff member; a khata has a note.
// - Keys: F10 saves, Esc closes (the screen under it keeps what was typed there).
// - A refusal shows at its field (name_ur, name_en, phone, note, staff_id).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { RecordRef } from '../../../bindings/RecordRef';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight, useSpellingFlow } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Field,
  FormDialog,
  FreeTextInput,
  NamePair,
  Picker,
  TextInput,
  fieldError,
  showToast,
  type NamePairValue,
  type PickerValue,
} from '../../../shared/ui';
import { useCreateKhata, useCreateLentPerson } from '../hooks';

export type AddKind = 'person' | 'khata';

const FIELD_PATHS = ['name_ur', 'name_en', 'phone', 'staff_id', 'note'];
const URDU_LETTER = /[؀-ۿ]/;

interface AddAccountDialogProps {
  kind: AddKind;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** What was typed in the picker: Urdu letters start the Urdu name, others the English name. */
  typed: string;
  /** The account that was made, to put in the picker. */
  onCreated: (account: RecordRef) => void;
}

function AddForm({
  kind,
  typed,
  onCreated,
  onClose,
}: Omit<AddAccountDialogProps, 'open' | 'onOpenChange'> & { onClose: () => void }) {
  const { l10n } = useLocalization();
  const text = typed.trim();
  const [names, setNames] = useState<NamePairValue>(() =>
    text === '' || /^\d+$/.test(text)
      ? { urdu: '', english: '' }
      : URDU_LETTER.test(text)
        ? { urdu: text, english: '' }
        : { urdu: '', english: text },
  );
  const [phone, setPhone] = useState('');
  const [staff, setStaff] = useState<PickerValue | null>(null);
  const [note, setNote] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  const createPerson = useCreateLentPerson();
  const createKhata = useCreateKhata();
  const spelling = useSpellingFlow();
  const flight = useSingleFlight();

  const submit = async () => {
    setError(null);
    try {
      const made = await spelling.guard(names, setNames, (confirmed): Promise<RecordRef> =>
        kind === 'person'
          ? createPerson
              .mutateAsync({
                name_ur: names.urdu.trim(),
                name_en: names.english.trim(),
                phone: phone.trim() === '' ? null : phone.trim(),
                staff_id: staff?.id ?? null,
                confirmed_rules_words: confirmed,
              })
              .then((person) => person.account)
          : createKhata
              .mutateAsync({
                name_ur: names.urdu.trim(),
                name_en: names.english.trim(),
                note: note.trim() === '' ? null : note.trim(),
                confirmed_rules_words: confirmed,
              })
              .then((khata) => khata.account),
      );
      if (made === undefined) return;
      showToast('success', l10n.getString(`cashbook-add-${kind}-saved`));
      onCreated(made);
      onClose();
    } catch (failure: unknown) {
      // The refusal is shown in the form by its fields.
      setError(uiErrorOf(failure));
    }
  };
  const submitNow = () => {
    void flight.run(submit, undefined);
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);

  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        <NamePair
          value={names}
          onChange={setNames}
          fillOnOpen
          urduLabel={<Localized id="cashbook-add-name-ur" />}
          englishLabel={<Localized id="cashbook-add-name-en" />}
          urduError={fieldError(error, 'name_ur')}
          englishError={fieldError(error, 'name_en')}
        />
        {kind === 'person' ? (
          <>
            <Field label={<Localized id="cashbook-add-phone" />} error={fieldError(error, 'phone')}>
              {(props) => <TextInput {...props} value={phone} onChange={setPhone} lang="en" />}
            </Field>
            <Field
              label={<Localized id="cashbook-add-staff" />}
              hint={<Localized id="cashbook-add-staff-hint" />}
              error={fieldError(error, 'staff_id')}
            >
              {(props) => (
                <Picker
                  {...props}
                  scope="staff"
                  value={staff}
                  onChange={(hit) => {
                    setStaff(
                      hit === null
                        ? null
                        : {
                            id: hit.id,
                            code: hit.code,
                            name_ur: hit.name_ur,
                            name_en: hit.name_en,
                          },
                    );
                  }}
                />
              )}
            </Field>
          </>
        ) : (
          <div className="normal:col-span-2">
            <Field label={<Localized id="cashbook-add-note" />} error={fieldError(error, 'note')}>
              {(props) => <FreeTextInput {...props} value={note} onChange={setNote} />}
            </Field>
          </div>
        )}
      </div>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || createPerson.isPending || createKhata.isPending}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
      {spelling.dialog}
    </form>
  );
}

export function AddAccountDialog({ kind, open, onOpenChange, ...form }: AddAccountDialogProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id={`cashbook-add-${kind}-title`} />}
      description={<Localized id={`cashbook-add-${kind}-help`} />}
    >
      {open ? (
        <AddForm
          kind={kind}
          {...form}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
