// The account form with every field of PRD 10.2 (plan 07): type (seller, buyer, ادھار رقم, other),
// code with the next one suggested, the name and the father or firm name in two scripts, city and
// اڈا, phones, address, opening balance, credit, collection, linked account, active and note.
//
// - Everything is text in this screen's state, so a language switch loses nothing (F39). Money
//   and the date go to the backend as typed; it parses them and says what is wrong (M8).
// - The names go through NamePair, and so does father or firm; a word made by the spelling rules
//   alone is checked before the save (useSpellingFlow, FR-7.66), once for each pair.
// - Accounts that look like this one show below the name while it is typed (FR-7.63).
// - Keys: F10 or Ctrl+S saves, Enter on the last field saves, Esc goes back (asking first),
//   arrows move between fields (shared/keyboard). An account is never deleted (FR-10.6).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useRef, useState, type ReactNode } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { useGrouping } from '../../../shared/format';
import { useFocusFirstField, useSpellingFlow } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import type { SearchHit } from '../../../shared/search';
import {
  AccountPicker,
  Button,
  Card,
  Checkbox,
  CodeInput,
  ConfirmDialog,
  DateInput,
  ErrorNotice,
  Field,
  fieldError,
  FreeTextInput,
  MoneyInput,
  NamePair,
  type NamePairValue,
  Picker,
  Select,
  showToast,
  TextInput,
} from '../../../shared/ui';
import {
  accountValuesOf,
  newAccountValues,
  sameAccountValues,
  type AccountFormValues,
} from '../account-draft';
import {
  useAccountNextCode,
  useCoaControls,
  useSaveAccount,
  useSetAccountActive,
  useSimilarAccounts,
} from '../../../shared/accounts';
import type { AccountRef, AccountRow, AccountType } from '../../../shared/accounts';
import { QuickAddMaster, type QuickMasterKind } from './QuickAddMaster';
import { SimilarAccounts } from './SimilarAccounts';

interface AccountFormProps {
  /** The account being edited, or null for a new one. */
  account: AccountRow | null;
  /** Today as `YYYY-MM-DD`, the opening date of a new account. */
  today: string;
}

const ACCOUNT_TYPES: readonly AccountType[] = ['seller', 'buyer', 'money_lent', 'other'];
/** The name of each day by the backend's number: 0 is Monday, 6 is Sunday. */
const WEEKDAY_NAMES = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;
/** The order the list shows them in: the mandi week starts on Saturday. */
const WEEKDAY_ORDER = [5, 6, 0, 1, 2, 3, 4] as const;

function toRef(hit: SearchHit): AccountRef {
  return { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en };
}

/** Every field the form draws, by the path the backend refuses it with. */
const FIELD_PATHS = [
  'account_type',
  'control_id',
  'code',
  'name_ur',
  'name_en',
  'father_or_firm_ur',
  'father_or_firm_en',
  'city_id',
  'adda_id',
  'phone1',
  'phone2',
  'address',
  'linked_account_id',
  'opening_date',
  'opening_dr',
  'opening_cr',
  'opening_is_advance',
  'credit_allowed',
  'credit_limit',
  'default_marka_id',
  'collector_staff_id',
  'collection_weekday',
  'note',
];

export function AccountForm({ account, today }: AccountFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const navigate = useNavigate();
  const isNew = account === null;
  const controls = useCoaControls();
  const [type, setType] = useState<AccountType>(account?.account_type ?? 'buyer');
  const [controlId, setControlId] = useState<number | null>(account?.control.id ?? null);
  const nextCode = useAccountNextCode(
    type,
    controlId,
    isNew && (type !== 'other' || controlId !== null),
  );

  // The form starts from the account that was read, once.
  const [initial] = useState(() =>
    account === null
      ? newAccountValues('buyer', null, '', today)
      : accountValuesOf(account, grouping),
  );
  const [values, setValues] = useState<AccountFormValues>(initial);
  const [baseline, setBaseline] = useState<AccountFormValues>(initial);
  const [codeTyped, setCodeTyped] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const [confirmInactive, setConfirmInactive] = useState(false);
  // A city or an اڈا being added on the spot (FR-7.64): what was typed in its picker starts the name.
  const [quickAdd, setQuickAdd] = useState<{ kind: QuickMasterKind; typed: string } | null>(null);
  const save = useSaveAccount();
  const setActive = useSetAccountActive();
  const containerRef = useFocusFirstField<HTMLFormElement>();
  const formRef = useRef<HTMLFormElement>(null);
  const similar = useSimilarAccounts(
    values.names.urdu,
    values.names.english,
    values.phone1,
    account?.id ?? null,
  );

  // A new account shows the suggested code until the user types his own.
  const suggested = nextCode.data;
  const codeShown = codeTyped || !isNew ? values.code : (suggested ?? '');

  const error =
    save.error !== null
      ? uiErrorOf(save.error)
      : setActive.error !== null
        ? uiErrorOf(setActive.error)
        : null;
  const dirty = !leaving && !sameAccountValues(values, baseline);
  const focusNames = () => {
    formRef.current?.querySelector<HTMLElement>('[role="combobox"], input')?.focus();
  };
  const spelling = useSpellingFlow({ onFix: focusNames });

  useEffect(() => {
    if (leaving) void navigate({ to: '/index/accounts' });
  }, [leaving, navigate]);

  const setNames = (names: NamePairValue) => {
    setValues((current) => ({ ...current, names }));
  };
  const setFather = (father: NamePairValue) => {
    setValues((current) => ({ ...current, father }));
  };

  const persist = async (): Promise<boolean> => {
    try {
      // The names first, then father or firm: each pair asks about its own marked words.
      const saved = await spelling.guard(values.names, setNames, (confirmedNames) =>
        spelling.guard(values.father, setFather, (confirmedFather) =>
          save.mutateAsync({
            draft: { ...values, code: codeShown, account_type: type, control_id: controlId },
            confirmedWords: [...new Set([...confirmedNames, ...confirmedFather])],
          }),
        ),
      );
      if (saved === undefined) return false;
      setBaseline(accountValuesOf(saved, grouping));
      showToast('success', l10n.getString('accounts-saved'));
      return true;
    } catch {
      return false;
    }
  };
  const saveAndClose = () => {
    void persist().then((ok) => {
      if (ok) setLeaving(true);
    });
  };
  const navigation = useFormNavigation(saveAndClose);
  useShortcut(SHORTCUTS.save, saveAndClose);
  const goBack = () => {
    void navigate({ to: '/index/accounts' });
  };
  useEscape('leave', goBack);

  const changeActive = (isActive: boolean) => {
    if (account === null) return;
    setActive.mutate(
      { id: account.id, isActive },
      {
        onSuccess: () => {
          showToast(
            'success',
            l10n.getString(isActive ? 'accounts-made-active' : 'accounts-made-inactive'),
          );
          setLeaving(true);
        },
      },
    );
  };

  const text =
    (
      field:
        | 'phone1'
        | 'phone2'
        | 'address'
        | 'note'
        | 'opening_dr'
        | 'opening_cr'
        | 'credit_limit'
        | 'opening_date',
    ) =>
    (value: string) => {
      setValues((current) => ({ ...current, [field]: value }));
    };
  const ref =
    (field: 'city' | 'adda' | 'linked_account' | 'default_marka' | 'collector') =>
    (hit: SearchHit | null) => {
      setValues((current) => ({ ...current, [field]: hit === null ? null : toRef(hit) }));
    };

  const hasOpeningDebit = values.opening_dr.trim() !== '';
  const forSeller = type === 'seller';
  const forBuyer = type === 'buyer';
  const heading = isNew ? (
    <Localized id="accounts-new-title">
      <h1 className="type-title" />
    </Localized>
  ) : (
    <Localized
      id="accounts-edit-title"
      vars={{ name: displayName({ name_ur: account.name_ur, name_en: account.name_en }, language) }}
    >
      <h1 className="type-title" />
    </Localized>
  );

  const pair = (
    label: { urdu: string; english: string },
    names: NamePairValue,
    onChange: (next: NamePairValue) => void,
    fields: { ur: string; en: string },
  ): ReactNode => (
    <NamePair
      value={names}
      onChange={onChange}
      urduLabel={<Localized id={label.urdu} />}
      englishLabel={<Localized id={label.english} />}
      urduError={fieldError(error, fields.ur)}
      englishError={fieldError(error, fields.en)}
    />
  );

  return (
    <div className="flex max-w-form flex-col gap-5">
      <div className="flex flex-wrap items-center gap-3">
        {heading}
        {account !== null && !account.is_active ? (
          <Localized id="accounts-status-inactive">
            <span className="type-chip rounded-1 bg-surface-sunken px-2 text-ink-muted" />
          </Localized>
        ) : null}
      </div>
      {account !== null && !account.is_active ? (
        <Localized id="accounts-inactive-note">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
      <form
        ref={(element) => {
          containerRef.current = element;
          formRef.current = element;
        }}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        <LeaveGuard dirty={dirty} onSave={persist} />
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}

        <Card title={<Localized id="accounts-section-account" />}>
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-3">
            <Field
              label={<Localized id="accounts-field-type" />}
              error={fieldError(error, 'account_type')}
              readOnly={!isNew}
            >
              {(props) => (
                <Select
                  {...props}
                  value={type}
                  onChange={(next) => {
                    setType(next);
                    setCodeTyped(false);
                    setValues((current) => ({ ...current, account_type: next, code: '' }));
                  }}
                  options={ACCOUNT_TYPES.map((option) => ({
                    value: option,
                    label: <Localized id={`accounts-type-${option}`} />,
                  }))}
                />
              )}
            </Field>
            {type === 'other' ? (
              <Field
                label={<Localized id="accounts-field-control" />}
                error={fieldError(error, 'control_id')}
                readOnly={!isNew}
              >
                {(props) => (
                  <Select
                    {...props}
                    value={controlId === null ? undefined : String(controlId)}
                    onChange={(next) => {
                      setControlId(Number(next));
                      setCodeTyped(false);
                      setValues((current) => ({ ...current, code: '' }));
                    }}
                    options={(controls.data ?? []).map((control) => ({
                      value: String(control.id),
                      label: (
                        <span className="flex gap-2">
                          <span>{displayName(control, language)}</span>
                          <span dir="ltr">{control.code}</span>
                        </span>
                      ),
                    }))}
                  />
                )}
              </Field>
            ) : null}
            <Field
              label={<Localized id="accounts-field-code" />}
              hint={isNew ? <Localized id="accounts-field-code-hint" /> : undefined}
              error={fieldError(error, 'code')}
            >
              {(props) => (
                <CodeInput
                  {...props}
                  value={codeShown}
                  onChange={(code) => {
                    setCodeTyped(true);
                    setValues((current) => ({ ...current, code }));
                  }}
                />
              )}
            </Field>
          </div>
        </Card>

        <Card title={<Localized id="accounts-section-names" />}>
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            {pair(
              { urdu: 'accounts-field-name-ur', english: 'accounts-field-name-en' },
              values.names,
              setNames,
              { ur: 'name_ur', en: 'name_en' },
            )}
          </div>
          <SimilarAccounts
            matches={(similar.data ?? []).filter((match) => match.id !== account?.id)}
          />
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            {pair(
              { urdu: 'accounts-field-father-ur', english: 'accounts-field-father-en' },
              values.father,
              setFather,
              { ur: 'father_or_firm_ur', en: 'father_or_firm_en' },
            )}
          </div>
        </Card>

        <Card title={<Localized id="accounts-section-contact" />}>
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            <Field
              label={<Localized id="accounts-field-city" />}
              error={fieldError(error, 'city_id')}
            >
              {(props) => (
                <Picker
                  {...props}
                  scope="city"
                  value={values.city}
                  onChange={ref('city')}
                  onAddNew={(typed) => {
                    setQuickAdd({ kind: 'city', typed });
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="accounts-field-adda" />}
              error={fieldError(error, 'adda_id')}
            >
              {(props) => (
                <Picker
                  {...props}
                  scope="adda"
                  value={values.adda}
                  onChange={ref('adda')}
                  onAddNew={(typed) => {
                    setQuickAdd({ kind: 'adda', typed });
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="accounts-field-phone1" />}
              error={fieldError(error, 'phone1')}
            >
              {(props) => (
                <TextInput {...props} value={values.phone1} onChange={text('phone1')} lang="en" />
              )}
            </Field>
            <Field
              label={<Localized id="accounts-field-phone2" />}
              error={fieldError(error, 'phone2')}
            >
              {(props) => (
                <TextInput {...props} value={values.phone2} onChange={text('phone2')} lang="en" />
              )}
            </Field>
          </div>
          <Field
            label={<Localized id="accounts-field-address" />}
            error={fieldError(error, 'address')}
          >
            {(props) => (
              <FreeTextInput {...props} value={values.address} onChange={text('address')} />
            )}
          </Field>
        </Card>

        <Card title={<Localized id="accounts-section-opening" />}>
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-3">
            <Field
              label={<Localized id="accounts-field-opening-date" />}
              error={fieldError(error, 'opening_date')}
            >
              {(props) => (
                <DateInput {...props} value={values.opening_date} onChange={text('opening_date')} />
              )}
            </Field>
            <Field
              label={<Localized id="accounts-field-opening-dr" />}
              error={fieldError(error, 'opening_dr')}
            >
              {(props) => (
                <MoneyInput {...props} value={values.opening_dr} onChange={text('opening_dr')} />
              )}
            </Field>
            <Field
              label={<Localized id="accounts-field-opening-cr" />}
              error={fieldError(error, 'opening_cr')}
            >
              {(props) => (
                <MoneyInput {...props} value={values.opening_cr} onChange={text('opening_cr')} />
              )}
            </Field>
          </div>
          {forSeller && hasOpeningDebit ? (
            <Checkbox
              checked={values.opening_is_advance}
              onChange={(checked) => {
                setValues((current) => ({ ...current, opening_is_advance: checked }));
              }}
              label={<Localized id="accounts-field-opening-advance" />}
            />
          ) : null}
        </Card>

        {forBuyer ? (
          <Card title={<Localized id="accounts-section-credit" />}>
            <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
              <Checkbox
                checked={values.credit_allowed}
                onChange={(checked) => {
                  setValues((current) => ({ ...current, credit_allowed: checked }));
                }}
                label={<Localized id="accounts-field-credit-allowed" />}
              />
              <Field
                label={<Localized id="accounts-field-credit-limit" />}
                error={fieldError(error, 'credit_limit')}
              >
                {(props) => (
                  <MoneyInput
                    {...props}
                    value={values.credit_limit}
                    onChange={text('credit_limit')}
                  />
                )}
              </Field>
              <Field
                label={<Localized id="accounts-field-collector" />}
                error={fieldError(error, 'collector_staff_id')}
              >
                {(props) => (
                  <Picker
                    {...props}
                    scope="staff"
                    value={values.collector}
                    onChange={ref('collector')}
                  />
                )}
              </Field>
              <Field
                label={<Localized id="accounts-field-weekday" />}
                error={fieldError(error, 'collection_weekday')}
              >
                {(props) => (
                  <Select
                    {...props}
                    value={
                      values.collection_weekday === null
                        ? undefined
                        : String(values.collection_weekday)
                    }
                    onChange={(next) => {
                      setValues((current) => ({ ...current, collection_weekday: Number(next) }));
                    }}
                    options={WEEKDAY_ORDER.map((day) => ({
                      value: String(day),
                      label: <Localized id={`accounts-weekday-${WEEKDAY_NAMES[day]}`} />,
                    }))}
                  />
                )}
              </Field>
            </div>
          </Card>
        ) : null}

        <Card title={<Localized id="accounts-section-more" />}>
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            {forSeller || forBuyer ? (
              <Field
                label={<Localized id="accounts-field-linked" />}
                hint={<Localized id="accounts-field-linked-hint" />}
                error={fieldError(error, 'linked_account_id')}
              >
                {(props) => (
                  // A seller is linked to his buyer account and a buyer to his seller account.
                  <AccountPicker
                    {...props}
                    scope={forSeller ? 'buyer' : 'seller'}
                    value={values.linked_account}
                    onChange={(hit) => {
                      setValues((current) => ({
                        ...current,
                        linked_account:
                          hit === null
                            ? null
                            : {
                                id: hit.id,
                                code: hit.code,
                                name_ur: hit.name_ur,
                                name_en: hit.name_en,
                              },
                      }));
                    }}
                  />
                )}
              </Field>
            ) : null}
            {forSeller ? (
              <Field
                label={<Localized id="accounts-field-marka" />}
                error={fieldError(error, 'default_marka_id')}
              >
                {(props) => (
                  <Picker
                    {...props}
                    scope="marka"
                    value={values.default_marka}
                    onChange={ref('default_marka')}
                  />
                )}
              </Field>
            ) : null}
          </div>
          <Field label={<Localized id="accounts-field-note" />} error={fieldError(error, 'note')}>
            {(props) => <FreeTextInput {...props} value={values.note} onChange={text('note')} />}
          </Field>
        </Card>

        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap gap-2">
            {account === null || account.is_system ? null : account.is_active ? (
              <Button
                variant="danger"
                busy={setActive.isPending}
                onClick={() => {
                  setConfirmInactive(true);
                }}
              >
                <Localized id="accounts-make-inactive" />
              </Button>
            ) : (
              <Button
                busy={setActive.isPending}
                onClick={() => {
                  changeActive(true);
                }}
              >
                <Localized id="accounts-make-active" />
              </Button>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            <Button shortcut={SHORTCUTS.back} onClick={goBack}>
              <Localized id="common-back" />
            </Button>
            <Button
              variant="primary"
              shortcut={SHORTCUTS.save}
              busy={save.isPending}
              onClick={saveAndClose}
            >
              <Localized id="layout-save-save" />
            </Button>
          </div>
        </div>
      </form>
      {spelling.dialog}
      <QuickAddMaster
        open={quickAdd !== null}
        onOpenChange={(open) => {
          if (!open) setQuickAdd(null);
        }}
        kind={quickAdd?.kind ?? 'city'}
        typed={quickAdd?.typed ?? ''}
        onSaved={(saved) => {
          const kind = quickAdd?.kind ?? 'city';
          setValues((current) => ({ ...current, [kind]: saved }));
        }}
      />
      {account === null ? null : (
        <ConfirmDialog
          open={confirmInactive}
          onOpenChange={setConfirmInactive}
          title={<Localized id="accounts-inactive-title" />}
          description={
            <Localized
              id="accounts-inactive-description"
              vars={{
                name: displayName({ name_ur: account.name_ur, name_en: account.name_en }, language),
              }}
            />
          }
          confirmLabel={<Localized id="accounts-make-inactive" />}
          tone="danger"
          onConfirm={() => {
            changeActive(false);
          }}
        />
      )}
    </div>
  );
}
