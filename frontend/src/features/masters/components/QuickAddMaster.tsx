// The short form of "add it here" for a master list (plan 41, PRD FR-7.69, FR-7.64, FR-10.8): the
// add row of a picker, or `F4` in it, opens this small form over the screen, with what was typed as
// the name. It is one form for every list, drawn from the same configuration as the Index form
// (configs.ts), with only the fields a new record needs:
//
//   city, store, expense head   the names;
//   item                        the names, the group (fruit or vegetable) and sold by (starts at نگ);
//   packing                     the names and the approximate weight (optional);
//   اڈا                         the names and the city (optional);
//   مارکہ                       the mark text; the owner is the lot's seller when it comes from a lot;
//   standard narration          the Urdu and the English text;
//   staff                       the names, the job and the phone (optional).
//
// - The code is given by itself: it is sent empty and the backend gives the next one (PRD FR-10.3).
// - The names go through NamePair, and a word made by the spelling rules alone is checked before
//   the save (useSpellingFlow, PRD FR-7.66). A narration is free text and is not checked.
// - Records that look like the name being typed show under it, and one can be taken instead of
//   making a new one (PRD FR-7.63, FR-10.7, E75).
// - The link "Full form" opens the Index form of the kind over the screen, with the name typed so
//   far, for fields this form does not ask.
// - Keys: F10 saves, Esc closes. While it is open, F10 and Esc act only inside it, and the screen
//   under it keeps everything typed there.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight, useSpellingFlow } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import type { SearchHit } from '../../../shared/search';
import {
  Button,
  ErrorNotice,
  Field,
  FormDialog,
  LinkButton,
  NamePair,
  TextInput,
  fieldError,
  showToast,
  type NamePairValue,
  type PickerValue,
} from '../../../shared/ui';
import { configOfKind } from '../configs';
import { newValues, type FormValues } from '../draft';
import { useSaveMaster, useSimilarMasters } from '../hooks';
import type { MasterKind, MasterRecord } from '../types';
import { MasterFieldInput, fieldPath } from './MasterFieldInput';
import { SimilarMasters } from './SimilarMasters';

/** The fields of the form of each kind that the short form asks (the rest are the full form's). */
const QUICK_FIELDS: Readonly<Partial<Record<MasterKind, readonly string[]>>> = {
  item: ['item_group', 'sold_by'],
  packing: ['approx_kg'],
  adda: ['city'],
  marka: ['mark_text'],
  staff: ['job', 'phone'],
};

const URDU_LETTER = /[؀-ۿ]/;
const ONLY_DIGITS = /^\d+$/;

export interface QuickAddMasterProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  kind: MasterKind;
  /** What was typed in the picker. It starts the name (the mark text, for a مارکہ). */
  typed: string;
  /** For a مارکہ added from a lot: the lot's seller, who becomes its owner. */
  owner?: PickerValue | null | undefined;
  /** The saved record. */
  onSaved: (record: MasterRecord) => void;
  /** Opens the Index form instead, with the name typed so far. The link shows only with it. */
  onFullForm?: ((text: string) => void) | undefined;
  /** Takes a look-alike that is already there, instead of making a new record. */
  onPickExisting?: ((hit: SearchHit) => void) | undefined;
}

/** The form as it starts: what was typed, in the script it is written in. */
function startingValues(kind: MasterKind, typed: string, owner: PickerValue | null): FormValues {
  const config = configOfKind(kind);
  const fresh = newValues(config, '');
  const text = typed.trim();
  const withOwner: FormValues =
    owner === null ? fresh : { ...fresh, refs: { ...fresh.refs, owner } };
  // A number is a code that matched nothing, not a name.
  if (text === '' || ONLY_DIGITS.test(text)) return withOwner;
  if (config.names === null) {
    return kind === 'marka'
      ? { ...withOwner, text: { ...withOwner.text, mark_text: text } }
      : withOwner;
  }
  return {
    ...withOwner,
    names: URDU_LETTER.test(text) ? { urdu: text, english: '' } : { urdu: '', english: text },
  };
}

function QuickAddForm({
  kind,
  typed,
  owner,
  onSaved,
  onFullForm,
  onPickExisting,
  onClose,
}: Omit<QuickAddMasterProps, 'open' | 'onOpenChange'> & { onClose: () => void }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const config = configOfKind(kind);
  const [values, setValues] = useState<FormValues>(() =>
    startingValues(kind, typed, owner ?? null),
  );
  const save = useSaveMaster(kind);
  const spelling = useSpellingFlow();
  const error = save.error === null ? null : uiErrorOf(save.error);
  const fields = config.fields.filter((spec) => QUICK_FIELDS[kind]?.includes(spec.key) === true);
  const fieldsShown = ['name_ur', 'name_en', ...fields.map(fieldPath)];

  // What the look-alike search reads: the name typed, in either script, or the mark text.
  const similarText =
    config.names === null
      ? (values.text['mark_text'] ?? '')
      : values.names.urdu.trim() !== ''
        ? values.names.urdu
        : values.names.english;
  const similar = useSimilarMasters(kind, similarText);

  const setNames = (names: NamePairValue) => {
    setValues((current) => ({ ...current, names }));
  };

  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = async () => {
    try {
      const saved = await spelling.guard(values.names, setNames, (confirmedWords) =>
        // The code is empty: the backend gives the next free one (PRD FR-10.3).
        save.mutateAsync({ draft: { ...values, code: '' }, confirmedWords }),
      );
      if (saved === undefined) return;
      showToast('success', l10n.getString('masters-saved'));
      onSaved(saved);
      onClose();
    } catch {
      // The refusal is shown in the form by its fields.
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
      {error === null ? null : <ErrorNotice error={error} fieldsShown={fieldsShown} />}
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        {config.names === null ? null : (
          <NamePair
            value={values.names}
            onChange={setNames}
            urduLabel={<Localized id={config.names.urduLabelId} />}
            englishLabel={<Localized id={config.names.englishLabelId} />}
            urduError={fieldError(error, 'name_ur')}
            englishError={fieldError(error, 'name_en')}
            checkSpelling={config.names.spelling}
            fillOnOpen
          />
        )}
        {fields.map((spec) => (
          <MasterFieldInput
            key={spec.key}
            spec={spec}
            values={values}
            error={error}
            onText={(key, text) => {
              setValues((current) => ({ ...current, text: { ...current.text, [key]: text } }));
            }}
            onRef={(key, hit) => {
              setValues((current) => ({
                ...current,
                refs: {
                  ...current.refs,
                  [key]:
                    hit === null
                      ? null
                      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                },
              }));
            }}
          />
        ))}
        {kind === 'marka' && owner != null ? (
          // The owner comes from the lot; it is shown, not asked.
          <Field label={<Localized id="masters-marka-owner" />} readOnly>
            {(props) => (
              <TextInput
                {...props}
                value={displayName(owner, language)}
                onChange={() => undefined}
              />
            )}
          </Field>
        ) : null}
      </div>
      <SimilarMasters
        matches={similar.data ?? []}
        onPick={
          onPickExisting === undefined
            ? undefined
            : (hit) => {
                onPickExisting(hit);
              }
        }
      />
      <div className="flex flex-wrap items-center justify-between gap-2">
        {onFullForm === undefined ? (
          <span />
        ) : (
          <LinkButton
            onClick={() => {
              onFullForm(similarText.trim());
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

export function QuickAddMaster({ open, onOpenChange, ...form }: QuickAddMasterProps) {
  const config = configOfKind(form.kind);
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      focusFirstEmpty
      title={<Localized id={config.newId} />}
      description={<Localized id="search-quick-help" />}
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
