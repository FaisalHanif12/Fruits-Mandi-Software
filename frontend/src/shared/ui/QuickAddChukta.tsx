// The short form of a new چکتا buyer (PRD FR-7.69, FR-13.3): the second add row of the buyer box of a
// بکری line (the first adds an ordinary buyer, QuickAddAccount) opens this small form over the
// screen with what was typed already in the name. A چکتا buyer is a walk-in who pays the whole bill
// on the spot, so the form asks only what tells him apart: his name in both scripts (NamePair, one
// typed and the other filled, FR-7.24) and his phone, which is optional.
//
// - The buyer is saved at once through `chukta_add` and comes back to the field, which takes him
//   (the screen underneath keeps every value it had). A person with the same names and phone is
//   returned by the backend, not doubled (FR-13.2).
// - A word made by the spelling rules alone is checked before the save, once for each pair
//   (useSpellingFlow, PRD FR-7.66). The phone is checked by the backend, with both names.
// - Keys: F10 saves, Esc closes.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { ChuktaBuyer } from '../../bindings/ChuktaBuyer';
import type { UiError } from '../../bindings/UiError';
import { useChuktaAdd } from '../accounts';
import { uiErrorOf } from '../api';
import { useSingleFlight, useSpellingFlow } from '../hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../keyboard';
import { Button } from './Button';
import { ErrorNotice } from './ErrorNotice';
import { Field } from './Field';
import { FormDialog } from './FormDialog';
import { NamePair, type NamePairValue } from './NamePair';
import { TextInput } from './TextInput';
import { fieldError } from './field-error';
import { showToast } from './toast-store';

interface QuickAddChuktaProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** What was typed in the buyer box. Letters start the name, digits start the phone. */
  typed: string;
  /** The saved buyer (or the one who was already there), to put in the field. */
  onSaved: (buyer: ChuktaBuyer) => void;
}

/** The fields of the form, so a refusal about one of them is shown at it and not twice. */
const FIELD_PATHS = ['name_ur', 'name_en', 'phone'];
const URDU_LETTER = /[؀-ۿ]/;

/** The names a typed text starts: digits are the phone, Urdu letters the Urdu name, else the English. */
function startOf(typed: string): { names: NamePairValue; phone: string } {
  const text = typed.trim();
  if (/^[\d-]+$/.test(text)) return { names: { urdu: '', english: '' }, phone: text };
  return URDU_LETTER.test(text)
    ? { names: { urdu: text, english: '' }, phone: '' }
    : { names: { urdu: '', english: text }, phone: '' };
}

function QuickAddChuktaForm({
  typed,
  onSaved,
  onClose,
}: Omit<QuickAddChuktaProps, 'open' | 'onOpenChange'> & { onClose: () => void }) {
  const { l10n } = useLocalization();
  const start = startOf(typed);
  const [names, setNames] = useState<NamePairValue>(start.names);
  const [phone, setPhone] = useState(start.phone);
  const add = useChuktaAdd();
  const spelling = useSpellingFlow();
  // What the backend refused in the last try, to show at the fields. The next try clears it.
  const [error, setError] = useState<UiError | null>(null);

  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = async () => {
    setError(null);
    try {
      const saved = await spelling.guard(names, setNames, () =>
        add.mutateAsync({
          name_ur: names.urdu.trim(),
          name_en: names.english.trim(),
          phone: phone.trim() === '' ? null : phone.trim(),
        }),
      );
      if (saved === undefined) return;
      showToast('success', l10n.getString('chukta-saved'));
      onSaved(saved);
      onClose();
    } catch (failure: unknown) {
      // The refusal is shown in the form by its fields, both errors together (FR-7.28, FR-13.2).
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
          urduLabel={<Localized id="accounts-field-name-ur" />}
          englishLabel={<Localized id="accounts-field-name-en" />}
          urduError={fieldError(error, 'name_ur')}
          englishError={fieldError(error, 'name_en')}
        />
        <Field label={<Localized id="accounts-field-phone1" />} error={fieldError(error, 'phone')}>
          {(props) => <TextInput {...props} value={phone} onChange={setPhone} lang="en" />}
        </Field>
      </div>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || add.isPending}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
      {spelling.dialog}
    </form>
  );
}

export function QuickAddChukta({ open, onOpenChange, ...form }: QuickAddChuktaProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      focusFirstEmpty
      title={<Localized id="chukta-add-title" />}
      description={<Localized id="chukta-add-help" />}
    >
      {open ? (
        <QuickAddChuktaForm
          {...form}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
