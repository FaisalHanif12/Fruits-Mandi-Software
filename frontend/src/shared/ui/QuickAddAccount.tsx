// Quick add of an account without leaving the screen (PRD FR-10.8, FR-7.42): `F4` in an account
// picker, or the "add" button of the unknown-code message, opens this small form over the screen,
// with the typed code filled in. The fields are the ones that tell one party from another: type
// (when the field takes any account), code, name in two scripts, father or firm, phone and city.
// Everything else is the account form's, later. A saved account comes back to the field.
//
// - The names go through NamePair, and a word made by the spelling rules alone is checked before
//   the save (useSpellingFlow, PRD FR-7.66), once for each pair.
// - Keys: F10 saves, Esc closes (the screen under it keeps what was typed there).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import {
  useAccountNextCode,
  useSaveAccount,
  type AccountDraft,
  type AccountRow,
  type AccountScope,
  type AccountType,
} from '../accounts';
import { uiErrorOf } from '../api';
import { todayText } from '../format';
import { useSpellingFlow } from '../hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../keyboard';
import { Button } from './Button';
import { CodeInput } from './CodeInput';
import { ErrorNotice } from './ErrorNotice';
import { Field } from './Field';
import { FormDialog } from './FormDialog';
import { NamePair, type NamePairValue } from './NamePair';
import { Picker } from './Picker';
import { Select } from './Select';
import { TextInput } from './TextInput';
import { fieldError } from './field-error';
import { showToast } from './toast-store';

interface QuickAddAccountProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Which accounts the field takes. `any` lets the user choose the type. */
  scope: AccountScope;
  /** What was typed in the field. Digits are a code; letters start the name. */
  typed: string;
  /** The code a new account would get, when the backend already said (unknown-code path). */
  suggestedCode?: string | null;
  /** The saved account, to put in the field. */
  onSaved: (account: AccountRow) => void;
}

const TYPES: readonly AccountType[] = ['seller', 'buyer', 'money_lent', 'other'];
const FIELD_PATHS = [
  'account_type',
  'control_id',
  'code',
  'name_ur',
  'name_en',
  'father_or_firm_ur',
  'father_or_firm_en',
  'phone1',
  'city_id',
];
const URDU_LETTER = /[؀-ۿ]/;
/** A full code has at least this many digits; a shorter one is a serial (PRD FR-7.39). */
const FULL_CODE_DIGITS = 8;

function startingType(scope: AccountScope): AccountType {
  return scope === 'seller' ? 'seller' : 'buyer';
}

function QuickAddForm({
  scope,
  typed,
  suggestedCode,
  onSaved,
  onClose,
}: Omit<QuickAddAccountProps, 'open' | 'onOpenChange'> & { onClose: () => void }) {
  const { l10n } = useLocalization();
  const text = typed.trim();
  const digits = /^\d+$/.test(text);
  const [type, setType] = useState<AccountType>(startingType(scope));
  const [code, setCode] = useState<string | null>(
    digits && text.length >= FULL_CODE_DIGITS ? text : null,
  );
  const [names, setNames] = useState<NamePairValue>(() =>
    digits || text === ''
      ? { urdu: '', english: '' }
      : URDU_LETTER.test(text)
        ? { urdu: text, english: '' }
        : { urdu: '', english: text },
  );
  const [father, setFather] = useState<NamePairValue>({ urdu: '', english: '' });
  const [phone, setPhone] = useState('');
  const [city, setCity] = useState<AccountDraft['city']>(null);
  const save = useSaveAccount();
  const nextCode = useAccountNextCode(type, null, code === null && type !== 'other');
  const spelling = useSpellingFlow();
  const error = save.error === null ? null : uiErrorOf(save.error);

  // The code shows what the backend suggested, or the next free one, until the user types his own.
  const codeShown =
    code ??
    (type === startingType(scope) && suggestedCode != null ? suggestedCode : (nextCode.data ?? ''));

  const draft: AccountDraft = {
    id: null,
    version: null,
    account_type: type,
    control_id: null,
    code: codeShown,
    names,
    father,
    city,
    adda: null,
    phone1: phone,
    phone2: '',
    address: '',
    linked_account: null,
    opening_date: todayText(new Date()),
    opening_dr: '',
    opening_cr: '',
    opening_is_advance: false,
    credit_allowed: true,
    credit_limit: '',
    default_marka: null,
    collector: null,
    collection_weekday: '',
    is_active: true,
    note: '',
  };

  const submit = async () => {
    try {
      const saved = await spelling.guard(names, setNames, (confirmedNames) =>
        spelling.guard(father, setFather, (confirmedFather) =>
          save.mutateAsync({
            draft,
            confirmedWords: [...new Set([...confirmedNames, ...confirmedFather])],
          }),
        ),
      );
      if (saved === undefined) return;
      showToast('success', l10n.getString('accounts-saved'));
      onSaved(saved);
      onClose();
    } catch {
      // The refusal is shown in the form by its fields.
    }
  };
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
      className="flex flex-col gap-4"
    >
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        {scope === 'any' ? (
          <Field
            label={<Localized id="accounts-field-type" />}
            error={fieldError(error, 'account_type')}
          >
            {(props) => (
              <Select
                {...props}
                value={type}
                onChange={(next) => {
                  setType(next);
                  setCode(null);
                }}
                options={TYPES.filter((option) => option !== 'other').map((option) => ({
                  value: option,
                  label: <Localized id={`accounts-type-${option}`} />,
                }))}
              />
            )}
          </Field>
        ) : null}
        <Field label={<Localized id="accounts-field-code" />} error={fieldError(error, 'code')}>
          {(props) => <CodeInput {...props} value={codeShown} onChange={setCode} />}
        </Field>
        <NamePair
          value={names}
          onChange={setNames}
          fillOnOpen
          urduLabel={<Localized id="accounts-field-name-ur" />}
          englishLabel={<Localized id="accounts-field-name-en" />}
          urduError={fieldError(error, 'name_ur')}
          englishError={fieldError(error, 'name_en')}
        />
        <NamePair
          value={father}
          onChange={setFather}
          urduLabel={<Localized id="accounts-field-father-ur" />}
          englishLabel={<Localized id="accounts-field-father-en" />}
          urduError={fieldError(error, 'father_or_firm_ur')}
          englishError={fieldError(error, 'father_or_firm_en')}
        />
        <Field label={<Localized id="accounts-field-phone1" />} error={fieldError(error, 'phone1')}>
          {(props) => <TextInput {...props} value={phone} onChange={setPhone} lang="en" />}
        </Field>
        <Field label={<Localized id="accounts-field-city" />} error={fieldError(error, 'city_id')}>
          {(props) => (
            <Picker
              {...props}
              scope="city"
              value={city}
              onChange={(hit) => {
                setCity(
                  hit === null
                    ? null
                    : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                );
              }}
            />
          )}
        </Field>
      </div>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={save.isPending}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
      {spelling.dialog}
    </form>
  );
}

export function QuickAddAccount({ open, onOpenChange, ...form }: QuickAddAccountProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="accountpicker-add-title" />}
      description={<Localized id="accountpicker-add-help" />}
    >
      {open ? (
        <QuickAddForm
          {...form}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
