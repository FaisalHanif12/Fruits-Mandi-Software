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
import {
  focusFirstEmptyField,
  useFocusFirstField,
  useSingleFlight,
  useSpellingFlow,
} from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import type { SearchHit } from '../../../shared/search';
import {
  AccountPicker,
  Button,
  Card,
  Checkbox,
  Code,
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
  SaveChangesPrompt,
  Select,
  showToast,
  SimilarAccounts,
  TagInput,
  type AddedRecord,
  TextInput,
  ActionBar,
  DeadInfoLine,
  FoldSection,
  FormGrid,
  useDeadAmountActions,
} from '../../../shared/ui';
import {
  accountValuesOf,
  newAccountValues,
  sameAccountValues,
  type AccountFormValues,
} from '../account-draft';
import {
  pickedOf,
  useAccountNextCode,
  useCoaControls,
  usePartyCard,
  useSaveAccount,
  useSetAccountActive,
  useSimilarAccounts,
} from '../../../shared/accounts';
import type { AccountRef, AccountRow, AccountType } from '../../../shared/accounts';

/**
 * The form opened above another screen by the "add" button of a picker (plan 32, R8): a new seller
 * or buyer, with what was typed in the picker already in the name box. It is the same form, with
 * the same fields and checks; only the way in and out is different. Nothing leaves the screen
 * underneath, so what is typed there stays.
 */
export interface EmbeddedAccount {
  /** The type to start with. */
  type: 'seller' | 'buyer';
  /** What was typed in the picker. */
  text: string;
  /** The form closed: with the account that was saved, or null if it was left. */
  onDone: (saved: AddedRecord | null) => void;
}

interface AccountFormProps {
  /** The account being edited, or null for a new one. */
  account: AccountRow | null;
  /** Today as `YYYY-MM-DD`, the opening date of a new account. */
  today: string;
  /** Set when the form is opened above another screen instead of being a screen of its own. */
  embedded?: EmbeddedAccount | undefined;
}

/** The types a new account can be: sellers and buyers. Others have their own screens (plan 32, R6). */
const NEW_ACCOUNT_TYPES: readonly AccountType[] = ['seller', 'buyer'];
const URDU_LETTER = /[؀-ۿ]/;

/** The two names a typed text starts: Urdu letters are the Urdu name, anything else the English. */
function namesOfText(text: string): NamePairValue {
  const typed = text.trim();
  if (typed === '' || /^\d+$/.test(typed)) return { urdu: '', english: '' };
  return URDU_LETTER.test(typed) ? { urdu: typed, english: '' } : { urdu: '', english: typed };
}
/** The name of each day by the backend's number: 0 is Monday, 6 is Sunday. */
const WEEKDAY_NAMES = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;
/** The order the list shows them in: the mandi week starts on Saturday. */
const WEEKDAY_ORDER = [5, 6, 0, 1, 2, 3, 4] as const;

function toRef(hit: SearchHit): AccountRef {
  return pickedOf(hit);
}

/** Every field the form draws, by the path the backend refuses it with. */
const FIELD_PATHS = [
  'account_type',
  'control_id',
  'code',
  'name_ur',
  'name_en',
  'tag',
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

export function AccountForm({ account, today, embedded }: AccountFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const navigate = useNavigate();
  const isNew = account === null;
  const controls = useCoaControls();
  const [type, setType] = useState<AccountType>(account?.account_type ?? embedded?.type ?? 'buyer');
  const [controlId, setControlId] = useState<number | null>(account?.control.id ?? null);
  const nextCode = useAccountNextCode(
    type,
    controlId,
    isNew && (type !== 'other' || controlId !== null),
  );

  // The form starts from the account that was read, once.
  const [initial] = useState(() => {
    if (account !== null) return accountValuesOf(account, grouping);
    const fresh = newAccountValues(embedded?.type ?? 'buyer', null, '', today);
    return embedded === undefined ? fresh : { ...fresh, names: namesOfText(embedded.text) };
  });
  const [values, setValues] = useState<AccountFormValues>(initial);
  const [baseline, setBaseline] = useState<AccountFormValues>(initial);
  const [codeTyped, setCodeTyped] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const [confirmInactive, setConfirmInactive] = useState(false);
  // The question "save changes?" of Esc above another screen (the route guard cannot ask it there).
  const [askLeave, setAskLeave] = useState(false);
  const save = useSaveAccount();
  const setActive = useSetAccountActive();
  // A person moved to the dead amount (PRD FR-10.10): the form says so, and "make active" becomes
  // "make active again", which asks for a reason and ends the move. A seller, a buyer or someone with
  // money lent whose balance is بنام can be moved (the dialog asks the backend again).
  const deadActions = useDeadAmountActions();
  const card = usePartyCard(account === null ? null : account.id);
  const dead = card.data?.dead ?? null;
  const movable =
    account !== null &&
    account.is_active &&
    !account.is_system &&
    account.account_type !== 'other' &&
    account.balance.side === 'dr';
  const containerRef = useFocusFirstField<HTMLFormElement>();
  const formRef = useRef<HTMLFormElement>(null);
  const similar = useSimilarAccounts(
    values.names.urdu,
    values.names.english,
    values.phone1,
    values.tag,
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
  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();

  // The host's way out, kept in a ref: the form leaves once, whichever object the host passes.
  // Above another screen the cursor goes to the first box still empty, once the next code has come
  // (the code box is empty until then). Opened as a page, the first box has the focus as always.
  const isEmbedded = embedded !== undefined;
  const codeReady = !nextCode.isPending;
  useEffect(() => {
    if (isEmbedded && codeReady) focusFirstEmptyField(formRef.current);
  }, [isEmbedded, codeReady]);
  // The account the last save made, handed back when the form closes.
  const savedRef = useRef<AddedRecord | null>(null);
  const doneRef = useRef(embedded?.onDone);
  useEffect(() => {
    doneRef.current = embedded?.onDone;
  });
  useEffect(() => {
    if (!leaving) return;
    if (isEmbedded) doneRef.current?.(savedRef.current);
    else void navigate({ to: '/index/accounts' });
  }, [leaving, isEmbedded, navigate]);

  const setNames = (names: NamePairValue) => {
    setValues((current) => ({ ...current, names }));
  };
  const setFather = (father: NamePairValue) => {
    setValues((current) => ({ ...current, father }));
  };

  const persistOnce = async (): Promise<boolean> => {
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
      savedRef.current = { id: saved.id, code: saved.code };
      setBaseline(accountValuesOf(saved, grouping));
      showToast('success', l10n.getString('accounts-saved'));
      return true;
    } catch {
      return false;
    }
  };
  const persist = (): Promise<boolean> => flight.run(persistOnce, false);
  const saveAndClose = () => {
    void persist().then((ok) => {
      if (ok) setLeaving(true);
    });
  };
  const navigation = useFormNavigation(saveAndClose);
  useShortcut(SHORTCUTS.save, saveAndClose);
  const goBack = () => {
    if (embedded === undefined) {
      void navigate({ to: '/index/accounts' });
    } else if (dirty) {
      setAskLeave(true);
    } else {
      embedded.onDone(null);
    }
  };
  useEscape('leave', goBack, !askLeave);

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
  // A part that is shut says how many of its fields hold something, and a part with a problem opens
  // itself (plan 38, account form).
  const has = (...paths: string[]) => paths.some((path) => fieldError(error, path) !== undefined);
  const filledCount = (...texts: string[]) => texts.filter((text) => text.trim() !== '').length;
  const heading = isNew ? (
    <Localized id="accounts-new-title">
      <h1 className="type-title" />
    </Localized>
  ) : (
    <Localized id="accounts-edit-title" vars={{ name: displayName(account, language) }}>
      <h1 className="type-title" />
    </Localized>
  );

  const pair = (
    label: { urdu: string; english: string },
    names: NamePairValue,
    onChange: (next: NamePairValue) => void,
    fields: { ur: string; en: string },
    key: 'names' | 'father',
  ): ReactNode => (
    <NamePair
      value={names}
      onChange={onChange}
      fillOnOpen={embedded !== undefined}
      // What the pair writes on open is the form's starting point, not a change of the user's.
      onFilledOnOpen={(filled) => {
        onChange(filled);
        setBaseline((current) => ({ ...current, [key]: filled }));
      }}
      urduLabel={<Localized id={label.urdu} />}
      englishLabel={<Localized id={label.english} />}
      urduError={fieldError(error, fields.ur)}
      englishError={fieldError(error, fields.en)}
    />
  );

  return (
    <div
      className={`flex max-w-form flex-col gap-3 ${embedded === undefined ? 'h-full min-h-0' : ''}`}
    >
      <div className="flex shrink-0 flex-wrap items-center gap-3">
        {embedded === undefined ? heading : null}
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
      {dead === null ? null : <DeadInfoLine info={dead} />}
      <div className={embedded === undefined ? 'min-h-0 flex-1 overflow-auto' : undefined}>
        <form
          ref={(element) => {
            containerRef.current = element;
            formRef.current = element;
          }}
          {...navigation}
          onSubmit={(event) => {
            event.preventDefault();
          }}
          className="flex flex-col gap-3"
        >
          {embedded === undefined ? <LeaveGuard dirty={dirty} onSave={persist} /> : null}
          {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}

          <Card title={<Localized id="accounts-section-account" />}>
            <FormGrid columns={3}>
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
                    options={(isNew ? NEW_ACCOUNT_TYPES : [type]).map((option) => ({
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
                            <Code>{control.code}</Code>
                          </span>
                        ),
                      }))}
                    />
                  )}
                </Field>
              ) : null}
              <Field
                label={<Localized id="accounts-field-code" />}
                info={
                  isNew ? (
                    <Localized id="accounts-field-code-hint">
                      <p />
                    </Localized>
                  ) : undefined
                }
                error={fieldError(error, 'code')}
                readOnly={!isNew}
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
            </FormGrid>
          </Card>

          <Card title={<Localized id="accounts-section-names" />}>
            <FormGrid columns={3}>
              {pair(
                { urdu: 'accounts-field-name-ur', english: 'accounts-field-name-en' },
                values.names,
                setNames,
                { ur: 'name_ur', en: 'name_en' },
                'names',
              )}
              <Field
                label={<Localized id="accounts-field-tag" />}
                info={
                  <Localized id="accounts-field-tag-hint">
                    <p />
                  </Localized>
                }
                error={fieldError(error, 'tag')}
              >
                {(props) => (
                  <TagInput
                    {...props}
                    value={values.tag}
                    onChange={(tag) => {
                      setValues((current) => ({ ...current, tag }));
                    }}
                  />
                )}
              </Field>
            </FormGrid>
            <SimilarAccounts
              matches={(similar.data ?? []).filter((match) => match.id !== account?.id)}
            />
            <FormGrid columns={2}>
              {pair(
                { urdu: 'accounts-field-father-ur', english: 'accounts-field-father-en' },
                values.father,
                setFather,
                { ur: 'father_or_firm_ur', en: 'father_or_firm_en' },
                'father',
              )}
            </FormGrid>
          </Card>

          <FoldSection
            title={<Localized id="accounts-section-contact" />}
            defaultOpen
            filled={filledCount(values.phone1, values.phone2, values.address)}
            forceOpen={has('city_id', 'adda_id', 'phone1', 'phone2', 'address')}
          >
            <FormGrid columns={4}>
              <Field
                label={<Localized id="accounts-field-city" />}
                error={fieldError(error, 'city_id')}
              >
                {(props) => (
                  <Picker {...props} scope="city" value={values.city} onChange={ref('city')} />
                )}
              </Field>
              <Field
                label={<Localized id="accounts-field-adda" />}
                error={fieldError(error, 'adda_id')}
              >
                {(props) => (
                  <Picker {...props} scope="adda" value={values.adda} onChange={ref('adda')} />
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
            </FormGrid>
            <Field
              label={<Localized id="accounts-field-address" />}
              error={fieldError(error, 'address')}
            >
              {(props) => (
                <FreeTextInput {...props} value={values.address} onChange={text('address')} />
              )}
            </Field>
          </FoldSection>

          <FoldSection
            title={<Localized id="accounts-section-opening" />}
            defaultOpen
            filled={filledCount(values.opening_dr, values.opening_cr)}
            forceOpen={has('opening_date', 'opening_dr', 'opening_cr')}
          >
            <FormGrid columns={3}>
              <Field
                label={<Localized id="accounts-field-opening-date" />}
                error={fieldError(error, 'opening_date')}
              >
                {(props) => (
                  <DateInput
                    {...props}
                    value={values.opening_date}
                    onChange={text('opening_date')}
                  />
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
            </FormGrid>
            {forSeller && hasOpeningDebit ? (
              <Checkbox
                checked={values.opening_is_advance}
                onChange={(checked) => {
                  setValues((current) => ({ ...current, opening_is_advance: checked }));
                }}
                label={<Localized id="accounts-field-opening-advance" />}
              />
            ) : null}
          </FoldSection>

          {forBuyer ? (
            <FoldSection
              title={<Localized id="accounts-section-credit" />}
              filled={filledCount(values.credit_limit)}
              forceOpen={has('credit_limit', 'collector_staff_id', 'collection_weekday')}
            >
              <FormGrid columns={4}>
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
              </FormGrid>
            </FoldSection>
          ) : null}

          <FoldSection
            title={<Localized id="accounts-section-more" />}
            filled={filledCount(values.note)}
            forceOpen={has('linked_account_id', 'default_marka_id', 'note')}
          >
            <FormGrid columns={2}>
              {forSeller || forBuyer ? (
                <Field
                  label={<Localized id="accounts-field-linked" />}
                  info={
                    <Localized id="accounts-field-linked-hint">
                      <p />
                    </Localized>
                  }
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
                          linked_account: hit === null ? null : pickedOf(hit),
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
            </FormGrid>
            <Field label={<Localized id="accounts-field-note" />} error={fieldError(error, 'note')}>
              {(props) => <FreeTextInput {...props} value={values.note} onChange={text('note')} />}
            </Field>
          </FoldSection>
        </form>
      </div>
      <ActionBar>
        <div className="me-auto flex flex-col gap-1">
          {account === null ? null : account.is_system ? (
            // A system account is used by the books, so it stays active: the button is there,
            // off, and the reason is said in words (PRD FR-10.4).
            <>
              <div>
                <Button disabled onClick={() => undefined}>
                  <Localized id="accounts-make-inactive" />
                </Button>
              </div>
              <Localized id="accounts-system-stays-active">
                <p className="type-body text-ink-muted" />
              </Localized>
            </>
          ) : account.is_active ? (
            <div className="flex flex-wrap gap-2">
              <Button
                variant="danger"
                busy={setActive.isPending}
                onClick={() => {
                  setConfirmInactive(true);
                }}
              >
                <Localized id="accounts-make-inactive" />
              </Button>
              {movable && deadActions !== null ? (
                <Button
                  onClick={() => {
                    deadActions.move({
                      accountId: account.id,
                      personKey: null,
                      names: {
                        name_ur: account.name_ur,
                        name_en: account.name_en,
                        tag: account.tag,
                      },
                    });
                  }}
                >
                  <Localized id="dead-move-button" />
                </Button>
              ) : null}
            </div>
          ) : dead !== null && deadActions !== null ? (
            <Button
              onClick={() => {
                deadActions.undo({
                  markId: dead.mark_id,
                  names: { name_ur: account.name_ur, name_en: account.name_en, tag: account.tag },
                });
              }}
            >
              <Localized id="dead-undo-button" />
            </Button>
          ) : dead !== null ? null : (
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
            busy={flight.busy || save.isPending}
            onClick={saveAndClose}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </ActionBar>
      {spelling.dialog}
      {embedded === undefined ? null : (
        <SaveChangesPrompt
          open={askLeave}
          onOpenChange={setAskLeave}
          onSave={() => {
            setAskLeave(false);
            saveAndClose();
          }}
          onLeave={() => {
            setAskLeave(false);
            embedded.onDone(null);
          }}
        />
      )}
      {account === null ? null : (
        <ConfirmDialog
          open={confirmInactive}
          onOpenChange={setConfirmInactive}
          title={<Localized id="accounts-inactive-title" />}
          description={
            <Localized
              id="accounts-inactive-description"
              vars={{ name: displayName(account, language) }}
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
