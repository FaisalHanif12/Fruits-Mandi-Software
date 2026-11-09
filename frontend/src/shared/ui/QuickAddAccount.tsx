// Quick add of an account without leaving the screen (PRD FR-7.69, FR-10.8, FR-7.42): the add row
// of an account picker, `F4`, or the "add" button of the unknown-code message opens this small form
// over the screen, with the typed text filled in. The fields are the ones that tell one party from
// another: type, code, name in two scripts, tag, phone and city. The type list offers the field's
// side; a field that takes any account offers seller and buyer. A walk-in چکتا buyer is not made
// here: the buyer picker of a بکری line has its own add row and form for him (QuickAddChukta).
// Everything else is the account form's, later, reached by the link "Full form". A saved account
// comes back to the field.
//
// - Accounts that look like the name being typed show under it (PRD FR-7.63, FR-10.7, E75). One of
//   them can be taken instead of making a new one.
// - Father or firm is not asked here (the full form has it).
// - The names go through NamePair, and a word made by the spelling rules alone is checked before
//   the save (useSpellingFlow, PRD FR-7.66), once for each pair.
// - Keys: F10 saves, Esc closes (the screen under it keeps what was typed there).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import {
  kindMessageId,
  pickedOf,
  useAccountNextCode,
  useSaveAccount,
  useSimilarAccounts,
  type AccountDraft,
  type AccountKind,
  type AccountMatch,
  type AccountRow,
  type AccountScope,
  type AccountType,
} from '../accounts';
import type { UiError } from '../../bindings/UiError';
import { uiErrorOf } from '../api';
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
   * Opens the full Index form instead, with the name typed so far and the kind chosen here. Leave
   * out where there is none (a field with its own form). The link shows only with it.
   */
  onFullForm?: ((text: string, kind: AccountKind) => void) | undefined;
  /** Takes a look-alike that is already there, instead of making a new account. */
  onPickExisting?: ((match: AccountMatch) => void) | undefined;
}

// A new account is a seller or a buyer. The people who can borrow and the khatas are added from
// their own screens (plan 32, R6).
const FIELD_PATHS = [
  'account_type',
  'control_id',
  'code',
  'name_ur',
  'name_en',
  'tag',
  'phone1',
  'city_id',
];
const URDU_LETTER = /[؀-ۿ]/;

function startingType(scope: AccountScope): AccountType {
  return scope === 'seller' || scope === 'lot_seller' ? 'seller' : 'buyer';
}

/** The kinds a field offers: a seller or buyer field its own side only (so no list), any account both. */
function kindsOfScope(scope: AccountScope): readonly AccountKind[] {
  return scope === 'any' ? ['seller', 'buyer'] : [startingType(scope)];
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
  onClose,
}: Omit<QuickAddAccountProps, 'open' | 'onOpenChange'> & {
  onClose: () => void;
}) {
  const { l10n } = useLocalization();
  const text = typed.trim();
  const digits = /^\d+$/.test(text);
  const kinds = kindsOfScope(scope);
  const [kind, setKind] = useState<AccountKind>(startingType(scope));
  const type = kind;
  // A typed number is a code that did not resolve, and its length says nothing (plan 32, R2): the
  // code to offer is the one the backend suggested, else the next free one of the type.
  const [code, setCode] = useState<string | null>(null);
  const [names, setNames] = useState<NamePairValue>(() =>
    digits || text === ''
      ? { urdu: '', english: '' }
      : URDU_LETTER.test(text)
        ? { urdu: text, english: '' }
        : { urdu: '', english: text },
  );
  const [tag, setTag] = useState('');
  // The short form asks the names, the tag, the phone and the city (PRD FR-7.69). Father or firm is
  // the full form's.
  const [phone, setPhone] = useState('');
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
    (kind === startingType(scope) && suggestedCode != null ? suggestedCode : (nextCode.data ?? ''));

  const draft: AccountDraft = {
    id: null,
    version: null,
    account_type: type,
    control_id: null,
    code: codeShown,
    names,
    tag,
    father: { urdu: '', english: '' },
    city,
    adda: null,
    phone1: phone,
    phone2: '',
    address: '',
    linked_account: null,
    // The quick add makes the account only: it has no opening, so it saves on any day (PRD FR-10.11).
    opening_date: '',
    opening_dr: '',
    opening_cr: '',
    opening_is_advance: false,
    credit_allowed: true,
    credit_limit: '',
    default_marka: null,
    collector: null,
    is_active: true,
    note: '',
  };

  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = async () => {
    setError(null);
    try {
      const saved = await spelling.guard(names, setNames, (confirmedWords) =>
        save.mutateAsync({ draft, confirmedWords }),
      );
      if (saved === undefined) return;
      showToast('success', l10n.getString('accounts-saved'));
      onSaved(saved);
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
        {kinds.length > 1 ? (
          <Field
            label={<Localized id="accounts-field-type" />}
            error={fieldError(error, 'account_type')}
          >
            {(props) => (
              <Select
                {...props}
                value={kind}
                onChange={(next: AccountKind) => {
                  setKind(next);
                  setCode(null);
                }}
                options={kinds.map((option) => ({
                  value: option,
                  label: <Localized id={kindMessageId(option)} />,
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
              onFullForm(names.urdu.trim() !== '' ? names.urdu.trim() : names.english.trim(), kind);
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

export function QuickAddAccount({ open, onOpenChange, ...form }: QuickAddAccountProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      focusFirstEmpty
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
