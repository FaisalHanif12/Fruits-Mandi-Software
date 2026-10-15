// Adds a partner (PRD 17.1, plan 11): his name in both scripts (NamePair, so the Urdu name can be
// typed in English letters, PRD FR-7.24), and optionally his opening capital in whole rupees. He
// has no share until the shares are saved. The spelling check runs before the save (FR-7.66).
//
// "Save and add another" keeps the dialog open for the next partner, so the first-run wizard
// does not need one dialog per partner. Keys: F10 saves, Ctrl+Enter saves and adds another, Esc
// closes.

import { Localized, useLocalization } from '@fluent/react';
import { useState, type RefObject } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useFocusFirstField, useSingleFlight, useSpellingFlow } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Field,
  FormDialog,
  MoneyInput,
  NamePair,
  fieldError,
  showToast,
  type NamePairValue,
} from '../../../shared/ui';
import { useCreatePartner } from '../hooks';

const EMPTY_NAMES: NamePairValue = { urdu: '', english: '' };
const FIELD_PATHS = ['name_ur', 'name_en', 'opening_capital'];

interface AddPartnerFormProps {
  onClose: () => void;
}

function AddPartnerForm({ onClose }: AddPartnerFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const [names, setNames] = useState<NamePairValue>(EMPTY_NAMES);
  const [capital, setCapital] = useState('');
  // A new number each time the form is emptied for the next partner: the name pair starts fresh.
  const [round, setRound] = useState(0);
  const create = useCreatePartner();
  const spelling = useSpellingFlow();
  const first = useFocusFirstField<HTMLFormElement>(round);
  const error = create.error === null ? null : uiErrorOf(create.error);

  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = async (addAnother: boolean) => {
    try {
      const saved = await spelling.guard(names, setNames, (confirmedWords) =>
        create.mutateAsync({
          name_ur: names.urdu,
          name_en: names.english,
          opening_capital: capital.trim() === '' ? null : capital,
          confirmed_rules_words: confirmedWords,
        }),
      );
      if (saved === undefined) return;
      showToast(
        'success',
        l10n.getString('partners-added', { name: displayName(saved, language) }),
      );
      if (addAnother) {
        setNames(EMPTY_NAMES);
        setCapital('');
        setRound((current) => current + 1);
        create.reset();
      } else {
        onClose();
      }
    } catch {
      // The refusal is shown in the form, at the fields.
    }
  };
  const run = (addAnother: boolean) => () => {
    void flight.run(() => submit(addAnother), undefined);
  };
  const navigation = useFormNavigation(run(false));
  useShortcut(SHORTCUTS.save, run(false));
  useShortcut(SHORTCUTS.saveAndNext, run(true));

  return (
    <form
      ref={first}
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      <Localized id="partners-add-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      <div key={round} className="flex flex-col gap-4">
        <NamePair
          value={names}
          onChange={setNames}
          urduLabel={<Localized id="partners-field-name-ur" />}
          englishLabel={<Localized id="partners-field-name-en" />}
          urduError={fieldError(error, 'name_ur')}
          englishError={fieldError(error, 'name_en')}
        />
      </div>
      <Field
        label={<Localized id="partners-field-capital" />}
        hint={<Localized id="partners-field-capital-hint" />}
        error={fieldError(error, 'opening_capital')}
      >
        {(control) => <MoneyInput {...control} value={capital} onChange={setCapital} />}
      </Field>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          shortcut={SHORTCUTS.saveAndNext}
          busy={flight.busy || create.isPending}
          onClick={run(true)}
        >
          <Localized id="partners-add-save-more" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || create.isPending}
          onClick={run(false)}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
      {spelling.dialog}
    </form>
  );
}

interface AddPartnerDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Where focus goes when the dialog closes: the share boxes, to type the shares next. */
  returnFocusTo?: RefObject<HTMLElement | null>;
}

export function AddPartnerDialog({ open, onOpenChange, returnFocusTo }: AddPartnerDialogProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      returnFocusTo={returnFocusTo}
      title={<Localized id="partners-add-title" />}
    >
      {open ? (
        <AddPartnerForm
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
