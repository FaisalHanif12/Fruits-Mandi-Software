// The short form of a new commission agent's shop (PRD FR-7.69, FR-16.11 v1.59, plan 97 item 1): the add row of the آڑھتی
// box of the پھل screen ("+ نئی آڑھتی دکان شامل کریں"), `F4`, or Enter on a name that matches nothing opens this small form
// over the screen with what was typed already in the name. The shop is a seller account marked آڑھتی دکان, so the form asks
// only what tells it apart: its name in both scripts (NamePair, one typed and the other filled, FR-7.24) and its phone,
// which is optional. The code is given by itself.
//
// - The shop is saved at once through `fruit_agent_shop_add` and comes back to the box, which takes it (the screen
//   underneath keeps every value it had).
// - A word made by the spelling rules alone is checked before the save, once for each pair
//   (useSpellingFlow, PRD FR-7.66). The phone is checked by the backend.
// - Keys: Enter or ↓ go to the next field, Enter on the last one and F10 save, Esc closes.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { AccountRow } from '../../bindings/AccountRow';
import type { UiError } from '../../bindings/UiError';
import { useAgentShopAdd } from '../accounts';
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

interface QuickAddAgentShopProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** What was typed in the آڑھتی box. Letters start the name, digits start the phone. */
  typed: string;
  /** The saved shop, to put in the field. */
  onSaved: (shop: AccountRow) => void;
}

/** The fields of the form, so a refusal about one of them is shown at it and not twice. */
const FIELD_PATHS = ['name_ur', 'name_en', 'phone1'];
const URDU_LETTER = /[؀-ۿ]/;

/** The names a typed text starts: digits are the phone, Urdu letters the Urdu name, else the English. */
function startOf(typed: string): { names: NamePairValue; phone: string } {
  const text = typed.trim();
  if (/^[\d-]+$/.test(text)) return { names: { urdu: '', english: '' }, phone: text };
  return URDU_LETTER.test(text)
    ? { names: { urdu: text, english: '' }, phone: '' }
    : { names: { urdu: '', english: text }, phone: '' };
}

function QuickAddAgentShopForm({
  typed,
  onSaved,
  onClose,
}: Omit<QuickAddAgentShopProps, 'open' | 'onOpenChange'> & { onClose: () => void }) {
  const { l10n } = useLocalization();
  const start = startOf(typed);
  const [names, setNames] = useState<NamePairValue>(start.names);
  const [phone, setPhone] = useState(start.phone);
  const add = useAgentShopAdd();
  const spelling = useSpellingFlow();
  // What the backend refused in the last try, to show at the fields. The next try clears it.
  const [error, setError] = useState<UiError | null>(null);

  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = async () => {
    setError(null);
    try {
      // The words he confirmed go with the save: the backend refuses a word that only the rules made and that is
      // not among them, with ERR_SPELLING_UNCONFIRMED, and the guard then asks him once more (FR-7.66).
      const saved = await spelling.guard(names, setNames, (confirmedWords) =>
        add.mutateAsync({
          name_ur: names.urdu.trim(),
          name_en: names.english.trim(),
          phone: phone.trim(),
          confirmed_rules_words: confirmedWords,
        }),
      );
      if (saved === undefined) return;
      showToast('success', l10n.getString('agent-shop-saved'));
      onSaved(saved);
      onClose();
    } catch (failure: unknown) {
      // The refusal is shown in the form by its fields, both errors together (FR-7.28).
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
        <Field label={<Localized id="accounts-field-phone1" />} error={fieldError(error, 'phone1')}>
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

export function QuickAddAgentShop({ open, onOpenChange, ...form }: QuickAddAgentShopProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      focusFirstEmpty
      title={<Localized id="agent-shop-add-title" />}
      description={<Localized id="agent-shop-add-help" />}
    >
      {open ? (
        <QuickAddAgentShopForm
          {...form}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
