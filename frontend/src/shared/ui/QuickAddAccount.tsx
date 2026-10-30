// Quick add of an account without leaving the screen (PRD FR-7.69, FR-10.8, FR-7.42): the add row
// of an account picker, `F4`, or the "add" button of the unknown-code message opens this small form
// over the screen, with the typed text filled in. The fields are the ones that tell one party from
// another: type (when the field takes any account), code, name in two scripts, tag, phone and city.
// Everything else is the account form's, later, reached by the link "Full form". A saved account
// comes back to the field.
//
// - Accounts that look like the name being typed show under it (PRD FR-7.63, FR-10.7, E75). One of
//   them can be taken instead of making a new one.
// - Father or firm is not asked here (the full form has it), except when a one-time person is made
//   regular.
// - The names go through NamePair, and a word made by the spelling rules alone is checked before
//   the save (useSpellingFlow, PRD FR-7.66), once for each pair.
// - Keys: F10 saves, Esc closes (the screen under it keeps what was typed there).
// - The same form makes a one-time person a regular account (PRD FR-13.8, plan 15): the screen
//   gives the names and the phone to start with (`initial`) and does the saving itself
//   (`saveWith`), because that save also moves his balance to the new account.

import { Localized, useLocalization } from '@fluent/react';
import { useState, type ReactNode } from 'react';
import {
  pickedOf,
  useAccountNextCode,
  useSaveAccount,
  useSimilarAccounts,
  type AccountDraft,
  type AccountMatch,
  type AccountRow,
  type AccountScope,
  type AccountType,
} from '../accounts';
import type { UiError } from '../../bindings/UiError';
import { uiErrorOf } from '../api';
import { todayText } from '../format';
import { useSingleFlight, useSpellingFlow } from '../hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../keyboard';
import { Button } from './Button';
import { CodeInput } from './CodeInput';
import { ErrorNotice } from './ErrorNotice';
import { Field } from './Field';
import { FormDialog } from './FormDialog';
import { LinkButton } from './LinkButton';
import { NamePair, type NamePairValue } from './NamePair';
import { Picker } from './Picker';
import { Select } from './Select';
import { SimilarAccounts } from './SimilarAccounts';
import { TagInput } from './TagInput';
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
  /**
   * Opens the full Index form instead, with the name typed so far. Leave out where there is none
   * (the one-time person made regular, a field with its own form). The link shows only with it.
   */
  onFullForm?: ((text: string) => void) | undefined;
  /** Takes a look-alike that is already there, instead of making a new account. */
  onPickExisting?: ((match: AccountMatch) => void) | undefined;
  /** Names and phone to start with, for example those of a one-time person. */
  initial?: { names: NamePairValue; phone: string } | undefined;
  /**
   * Does the saving instead of the plain save of an account, and says when it is done. The screen
   * then shows its own message and uses its own result; `onSaved` is not called.
   */
  saveWith?:
    ((draft: AccountDraft, confirmedWords: readonly string[]) => Promise<void>) | undefined;
  /** Another title and help line than the quick add's. */
  title?: ReactNode;
  help?: ReactNode;
}

// A new account is a seller or a buyer. The people who can borrow and the khatas are added from
// their own screens (plan 32, R6).
const TYPES: readonly AccountType[] = ['seller', 'buyer'];
const FIELD_PATHS = [
  'account_type',
  'control_id',
  'code',
  'name_ur',
  'name_en',
  'tag',
  'father_or_firm_ur',
  'father_or_firm_en',
  'phone1',
  'city_id',
];
const URDU_LETTER = /[؀-ۿ]/;

function startingType(scope: AccountScope): AccountType {
  return scope === 'seller' ? 'seller' : 'buyer';
}

/** Can this look-alike be taken in this field? A seller field takes sellers, a buyer field buyers. */
function fitsField(scope: AccountScope, match: AccountMatch): boolean {
  if (scope === 'control') return false;
  if (scope === 'any') return match.account_type === 'seller' || match.account_type === 'buyer';
  return match.account_type === startingType(scope);
}

function QuickAddForm({
  scope,
  typed,
  suggestedCode,
  onSaved,
  onFullForm,
  onPickExisting,
  initial,
  saveWith,
  onClose,
}: Omit<QuickAddAccountProps, 'open' | 'onOpenChange' | 'title' | 'help'> & {
  onClose: () => void;
}) {
  const { l10n } = useLocalization();
  const text = typed.trim();
  const digits = /^\d+$/.test(text);
  const [type, setType] = useState<AccountType>(startingType(scope));
  // A typed number is a code that did not resolve, and its length says nothing (plan 32, R2): the
  // code to offer is the one the backend suggested, else the next free one of the type.
  const [code, setCode] = useState<string | null>(null);
  const [names, setNames] = useState<NamePairValue>(() =>
    initial !== undefined
      ? initial.names
      : digits || text === ''
        ? { urdu: '', english: '' }
        : URDU_LETTER.test(text)
          ? { urdu: text, english: '' }
          : { urdu: '', english: text },
  );
  const [tag, setTag] = useState('');
  const [father, setFather] = useState<NamePairValue>({ urdu: '', english: '' });
  // The short form asks the names, the tag, the phone and the city (PRD FR-7.69). Father or firm is
  // the full form's, except when a one-time person is made regular (PRD FR-13.8).
  const askFather = initial !== undefined;
  const [phone, setPhone] = useState(initial?.phone ?? '');
  const [city, setCity] = useState<AccountDraft['city']>(null);
  const save = useSaveAccount();
  const nextCode = useAccountNextCode(type, null, code === null);
  const similar = useSimilarAccounts(names.urdu, names.english, phone, tag, null);
  const spelling = useSpellingFlow();
  // What the backend refused in the last try, to show at the fields. The next try clears it.
  const [error, setError] = useState<UiError | null>(null);

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
    tag,
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
    collection_weekday: null,
    is_active: true,
    note: '',
  };

  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = async () => {
    setError(null);
    try {
      const saved = await spelling.guard(names, setNames, (confirmedNames) =>
        spelling.guard(father, setFather, (confirmedFather) => {
          const confirmedWords = [...new Set([...confirmedNames, ...confirmedFather])];
          return saveWith === undefined
            ? save.mutateAsync({ draft, confirmedWords })
            : saveWith(draft, confirmedWords).then(() => null);
        }),
      );
      if (saved === undefined) return;
      if (saved !== null) {
        showToast('success', l10n.getString('accounts-saved'));
        onSaved(saved);
      }
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
                options={TYPES.map((option) => ({
                  value: option,
                  label: <Localized id={`accounts-type-${option}`} />,
                }))}
              />
            )}
          </Field>
        ) : null}
        <NamePair
          value={names}
          onChange={setNames}
          fillOnOpen
          urduLabel={<Localized id="accounts-field-name-ur" />}
          englishLabel={<Localized id="accounts-field-name-en" />}
          urduError={fieldError(error, 'name_ur')}
          englishError={fieldError(error, 'name_en')}
        />
        <Field
          label={<Localized id="accounts-field-tag" />}
          hint={<Localized id="accounts-field-tag-hint" />}
          error={fieldError(error, 'tag')}
        >
          {(props) => <TagInput {...props} value={tag} onChange={setTag} />}
        </Field>
        {askFather ? (
          <NamePair
            value={father}
            onChange={setFather}
            urduLabel={<Localized id="accounts-field-father-ur" />}
            englishLabel={<Localized id="accounts-field-father-en" />}
            urduError={fieldError(error, 'father_or_firm_ur')}
            englishError={fieldError(error, 'father_or_firm_en')}
          />
        ) : null}
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
                setCity(hit === null ? null : pickedOf(hit));
              }}
            />
          )}
        </Field>
        {/* The code comes last: it is given by itself, so the cursor starts at the names. */}
        <Field label={<Localized id="accounts-field-code" />} error={fieldError(error, 'code')}>
          {(props) => <CodeInput {...props} value={codeShown} onChange={setCode} />}
        </Field>
      </div>
      <SimilarAccounts
        matches={similar.data ?? []}
        onPick={
          onPickExisting === undefined
            ? undefined
            : (match) => {
                onPickExisting(match);
              }
        }
        canPick={(match) => fitsField(scope, match)}
      />
      <div className="flex flex-wrap items-center justify-between gap-2">
        {onFullForm === undefined ? (
          <span />
        ) : (
          <LinkButton
            onClick={() => {
              onFullForm(names.urdu.trim() !== '' ? names.urdu.trim() : names.english.trim());
            }}
          >
            <Localized id="search-quick-full-form" />
          </LinkButton>
        )}
        <div className="flex flex-wrap justify-end gap-2">
          <Button shortcut={SHORTCUTS.back} onClick={onClose}>
            <Localized id="common-back" />
          </Button>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={flight.busy || save.isPending}
            onClick={submitNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </div>
      {spelling.dialog}
    </form>
  );
}

export function QuickAddAccount({
  open,
  onOpenChange,
  title,
  help,
  ...form
}: QuickAddAccountProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      focusFirstEmpty
      title={title ?? <Localized id="accountpicker-add-title" />}
      description={help ?? <Localized id="accountpicker-add-help" />}
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
