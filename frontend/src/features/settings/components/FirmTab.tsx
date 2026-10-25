// The firm's names, mandi, address and phone (PRD Section 21 "Firm", plan 04), up to three contact
// persons and the trade mark that every print shows in its header (plan 17e, FR-19.13). Both names of
// the firm, the mandi and each contact person are kept, one per script (PRD FR-7.24). A word of the
// firm's, the mandi's and each contact person's names made only by the spelling rules is checked
// before the save (PRD FR-7.66, as the backend checks it too). A contact row left empty is not sent. The trade mark is text as typed, in one script,
// never turned into Urdu (FR-7.25). Leaving with unsaved changes asks "Save changes?" (PRD FR-7.9).

import { Localized } from '@fluent/react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useLocalization } from '@fluent/react';
import { Plus, X } from 'lucide-react';
import type { FirmContact } from '../../../bindings/FirmContact';
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

/** The most contact persons a print header holds (PRD Section 21). */
const MAX_CONTACTS = 3;

/** The fields of the firm that are one text each; the contacts are a list of their own. */
type FirmFields = Pick<
  FirmSettings,
  'name_ur' | 'name_en' | 'mandi_ur' | 'mandi_en' | 'address' | 'phone' | 'trade_mark'
>;

const FIELDS = [
  'name_ur',
  'name_en',
  'mandi_ur',
  'mandi_en',
  'address',
  'phone',
  'trade_mark',
] as const satisfies readonly (keyof FirmFields)[];

/** The field paths a contact row shows next to its inputs (the backend names them like this). */
const CONTACT_PATHS = Array.from({ length: MAX_CONTACTS }, (_, index) =>
  (['name_ur', 'name_en', 'phone'] as const).map(
    (part) => `firm_contacts[${String(index)}].${part}`,
  ),
).flat();

/** A contact row with nothing in it is not a contact: it is left out of the save. */
function filled(contact: FirmContact): boolean {
  return [contact.name_ur, contact.name_en, contact.phone].some((text) => text.trim() !== '');
}

function fieldsOf(firm: FirmSettings): FirmFields {
  return {
    name_ur: firm.name_ur,
    name_en: firm.name_en,
    mandi_ur: firm.mandi_ur,
    mandi_en: firm.mandi_en,
    address: firm.address,
    phone: firm.phone,
    trade_mark: firm.trade_mark,
  };
}

const NEW_CONTACT: FirmContact = { name_ur: '', name_en: '', phone: '' };

interface FirmTabProps {
  firm: FirmSettings;
}

export function FirmTab({ firm }: FirmTabProps) {
  const { l10n } = useLocalization();
  const { values, setField, setValues } = useFormState<FirmFields>(fieldsOf(firm));
  // The contact persons being edited. They belong to the saved firm they were made against: when a
  // save brings a new `firm`, they are no longer read and the saved ones show (no effect needed).
  const [edited, setEdited] = useState<{ forFirm: FirmSettings; list: FirmContact[] } | null>(null);
  const contacts = edited !== null && edited.forFirm === firm ? edited.list : firm.firm_contacts;
  const setContacts = (change: (current: FirmContact[]) => FirmContact[]) => {
    setEdited({ forFirm: firm, list: change(contacts) });
  };
  // The words of each name to check, and the ones he confirmed (PRD FR-7.66). They sit here, with
  // the values, so a language switch loses nothing. They belong to the saved firm they were made
  // against: when a save brings a new `firm`, they are no longer read.
  const [marksOf, setMarksOf] = useState<{
    forFirm: FirmSettings;
    firm?: SpellingMarks | undefined;
    mandi?: SpellingMarks | undefined;
    /** The marks of each contact person's names, by his place in the list. */
    contacts?: (SpellingMarks | undefined)[] | undefined;
  }>({ forFirm: firm });
  const marks = marksOf.forFirm === firm ? marksOf : { forFirm: firm };
  const rememberMarks = (name: 'firm' | 'mandi') => (next: NamePairValue) => {
    setMarksOf({ ...marks, [name]: next.spelling });
  };
  const rememberContactMarks = (index: number) => (next: NamePairValue) => {
    const list = [...(marks.contacts ?? [])];
    list[index] = next.spelling;
    setMarksOf({ ...marks, contacts: list });
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
    () =>
      FIELDS.some((field) => values[field] !== firm[field]) ||
      JSON.stringify(contacts.filter(filled)) !== JSON.stringify(firm.firm_contacts),
    [values, contacts, firm],
  );

  // The saved values arrived again (after a save): the form shows exactly them.
  useEffect(() => {
    setValues(fieldsOf(firm));
  }, [firm, setValues]);

  const changeContact = (index: number, next: Partial<FirmContact>) => {
    setContacts((current) =>
      current.map((contact, place) => (place === index ? { ...contact, ...next } : contact)),
    );
  };

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

  /** Checks the names of each contact person in turn (the spelling check), then saves. */
  const saveAfterContacts = (
    index: number,
    words: readonly string[],
  ): Promise<FirmSettings | undefined> => {
    const contact = contacts[index];
    if (index >= contacts.length) {
      return save.mutateAsync({
        ...values,
        firm_contacts: contacts.filter(filled),
        confirmed_rules_words: [...new Set(words)],
      });
    }
    if (contact === undefined || !filled(contact)) return saveAfterContacts(index + 1, words);
    return spelling.guard(
      { urdu: contact.name_ur, english: contact.name_en, spelling: marks.contacts?.[index] },
      rememberContactMarks(index),
      (more) => saveAfterContacts(index + 1, [...words, ...more]),
    );
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
            (mandiWords) => saveAfterContacts(0, [...firmWords, ...mandiWords]),
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
        {error === null ? null : (
          <ErrorNotice error={error} fieldsShown={[...FIELDS, 'firm_contacts', ...CONTACT_PATHS]} />
        )}
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
        <Field
          label={<Localized id="settings-firm-trade-mark" />}
          hint={<Localized id="settings-firm-trade-mark-hint" />}
          error={fieldError(error, 'trade_mark')}
        >
          {(control) => (
            <TextInput {...control} value={values.trade_mark} onChange={setField('trade_mark')} />
          )}
        </Field>
        <section className="flex flex-col gap-3 rounded-3 border border-border p-3">
          <Localized id="settings-firm-contacts-title">
            <h2 className="type-section" />
          </Localized>
          <Localized id="settings-firm-contacts-help">
            <p className="type-body text-ink-muted" />
          </Localized>
          {fieldError(error, 'firm_contacts') === undefined ? null : (
            <p role="alert" className="type-body text-due">
              {fieldError(error, 'firm_contacts')}
            </p>
          )}
          {contacts.length === 0 ? (
            <Localized id="settings-firm-contacts-empty">
              <p className="type-body text-ink-muted" />
            </Localized>
          ) : null}
          {contacts.map((contact, index) => (
            <div key={index} className="flex flex-col gap-3 border-t border-border pt-3">
              <div className="flex items-center justify-between gap-3">
                <Localized id="settings-firm-contact-title" vars={{ number: String(index + 1) }}>
                  <h3 className="type-label" />
                </Localized>
                <Button
                  onClick={() => {
                    setContacts((current) => current.filter((_, place) => place !== index));
                    setMarksOf({
                      ...marks,
                      contacts: (marks.contacts ?? []).filter((_, place) => place !== index),
                    });
                  }}
                  accessibleName={l10n.getString('settings-firm-contact-remove-label', {
                    number: String(index + 1),
                  })}
                >
                  <X aria-hidden="true" className="size-icon-md" />
                  <Localized id="settings-firm-contact-remove" />
                </Button>
              </div>
              <div className="grid grid-cols-1 gap-3 normal:grid-cols-2">
                <NamePair
                  value={{
                    urdu: contact.name_ur,
                    english: contact.name_en,
                    spelling: marks.contacts?.[index],
                  }}
                  onChange={(next) => {
                    changeContact(index, { name_ur: next.urdu, name_en: next.english });
                    rememberContactMarks(index)(next);
                  }}
                  urduLabel={<Localized id="settings-firm-contact-name-ur" />}
                  englishLabel={<Localized id="settings-firm-contact-name-en" />}
                  urduError={fieldError(error, `firm_contacts[${String(index)}].name_ur`)}
                  englishError={fieldError(error, `firm_contacts[${String(index)}].name_en`)}
                />
                <Field
                  label={<Localized id="settings-firm-contact-phone" />}
                  error={fieldError(error, `firm_contacts[${String(index)}].phone`)}
                >
                  {(control) => (
                    <TextInput
                      {...control}
                      value={contact.phone}
                      onChange={(phone) => {
                        changeContact(index, { phone });
                      }}
                      lang="en"
                    />
                  )}
                </Field>
              </div>
            </div>
          ))}
          <div className="flex flex-wrap items-center gap-3">
            <Button
              disabled={contacts.length >= MAX_CONTACTS}
              onClick={() => {
                setContacts((current) => [...current, NEW_CONTACT]);
              }}
            >
              <Plus aria-hidden="true" className="size-icon-md" />
              <Localized id="settings-firm-contact-add" />
            </Button>
            {contacts.length >= MAX_CONTACTS ? (
              <Localized id="settings-firm-contacts-full">
                <p className="type-body text-ink-muted" />
              </Localized>
            ) : null}
          </div>
        </section>
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
