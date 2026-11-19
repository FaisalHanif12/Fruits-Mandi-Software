// The facts of a garden open for edit (plan 44, PRD FR-7.70, FR-7.71): the names, the main fruit, the
// rent, the end of the season and the note. The day and the shares never change (PRD FR-12.29): the
// band says so and the day box is read only. A header can be changed at any time; it posts nothing,
// so there is no cash question. Under the form the backend's review shows what changed. Save asks
// why when the garden is not of today.
//
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money (CLAUDE.md rule 1): the changes and the need for a reason are
//   the backend's, from the review (the same function as the save).
// - A change of the rent is how a relief from the garden's owner is written (plan 78 D3, PRD FR-12.51). It always
//   needs a reason and the Owner's sign, also for a garden of today, so while the rent box holds another rent
//   two fields appear under it: why, and the sign. They are sent with the review and the save, and a wrong
//   sign is told under its field and changes nothing. Whether the rent differs is read from the text (the digits
//   of the box against the digits of the saved rent); the backend decides, and ignores a rent that is the same.

import { Localized, useLocalization } from '@fluent/react';
import { useMemo, useState } from 'react';
import type { GardenHeaderEditInput } from '../../../bindings/GardenHeaderEditInput';
import type { UiError } from '../../../bindings/UiError';
import { pickedOf } from '../../../shared/accounts';
import {
  finishFixSession,
  useEditReview,
  useEditSave,
  useEntryStamp,
  useReasonText,
} from '../../../shared/edit';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  DateInput,
  EditFormBar,
  EditReviewCard,
  EditSaveBoxes,
  ErrorNotice,
  Field,
  FormAside,
  FreeTextInput,
  MoneyInput,
  NamePair,
  MultiPicker,
  Notice,
  TextInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { gardensHeaderEditPreview } from '../api';
import { GARDEN_FIELD_PATHS, buildGarden } from '../draft';
import { gardenHeaderDraftOf } from '../edit-seed';
import { useSaveGardenHeaderEdit } from '../hooks';
import type { GardenDraft, GardenRow } from '../types';

interface GardenHeaderEditProps {
  garden: GardenRow;
  /** The edit is over (saved or left): the page goes back to the entry form. */
  onDone: () => void;
}

/** The digits of a money text: "50,00,000" and "5000000" are the same rent. */
function digitsOf(text: string): string {
  return text.replaceAll(/\D/g, '');
}

export function GardenHeaderEdit({ garden, onDone }: GardenHeaderEditProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const stamp = useEntryStamp('garden', garden.id);
  const [draft, setDraft] = useState<GardenDraft>(() => gardenHeaderDraftOf(garden, grouping));
  const reason = useReasonText();
  // The reason and the sign of a change of the rent (plan 78 D3). Text on the screen's own state (F39).
  const [rentReason, setRentReason] = useState('');
  const [sign, setSign] = useState('');
  const [tried, setTried] = useState(false);
  const savedRent = digitsOf(formatPaisa(garden.rent_paisa, { grouping }));
  const rentChanged = digitsOf(draft.rent) !== savedRent;

  const change = (patch: Partial<GardenDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
    setTried(false);
  };

  const editInput = useMemo<GardenHeaderEditInput>(
    () => ({
      id: garden.id,
      version: garden.version,
      garden: buildGarden(draft, false),
      reason: rentChanged ? rentReason.trim() : '',
      ...(rentChanged && sign.trim() !== '' ? { sign: sign.trim() } : {}),
    }),
    [draft, garden.id, garden.version, rentChanged, rentReason, sign],
  );
  const review = useEditReview(
    `garden-header:${String(garden.id)}`,
    editInput,
    gardensHeaderEditPreview,
  );

  const saveHeader = useSaveGardenHeaderEdit();
  const draftText = useMemo(
    () => JSON.stringify({ draft, rentChanged, rentReason, sign }),
    [draft, rentChanged, rentReason, sign],
  );
  const flow = useEditSave({
    review: review.review,
    reason,
    draftText,
    send: (why) =>
      saveHeader.mutateAsync({
        id: garden.id,
        version: garden.version,
        garden: buildGarden(draft, false),
        reason: why,
        ...(rentChanged ? { sign: sign.trim() } : {}),
      }),
    onSaved: () => {
      showToast('success', l10n.getString('edit-saved', { number: garden.code }));
      void finishFixSession(l10n);
      onDone();
    },
  });

  const previewError: UiError | null = useMemo(() => {
    // What only says "this is empty" waits for the save; a wrong sign is told under its field when the save
    // is refused, not at every key of the sign (plan 78 D3).
    const held = new Set([
      'ERR_REASON_REQUIRED',
      'ERR_REQUIRED',
      'ERR_SIGN_REQUIRED',
      'ERR_SIGN_WRONG',
    ]);
    const shown = review.errors.filter((item) => !held.has(item.code));
    return shown.length === 0
      ? null
      : { ok: false, errors: shown.map((item) => ({ ...item })), ref: '' };
  }, [review.errors]);
  const error = flow.refusal ?? previewError;

  const back = () => {
    void finishFixSession(l10n);
    onDone();
  };
  const saveNow = () => {
    // A change of the rent always needs both: they are told at their fields before anything is sent.
    if (rentChanged && (rentReason.trim() === '' || sign.trim() === '')) {
      setTried(true);
      return;
    }
    void flow.save();
  };
  const navigation = useFormNavigation(saveNow);
  useShortcut(SHORTCUTS.save, saveNow);
  useEscape('leave', back);

  return (
    <div className="flex flex-col gap-3">
      <EditFormBar
        kind="garden"
        number={garden.code}
        date={garden.entry_date}
        edited={stamp}
        history={{ entity: 'garden', id: garden.id }}
        busy={flow.busy}
        onSave={saveNow}
        onBack={back}
      />
      <form
        ref={formRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        {error === null ? null : (
          <ErrorNotice error={error} fieldsShown={[...GARDEN_FIELD_PATHS, 'reason', 'sign']} />
        )}
        <FormAside
          form={
            <div className="grid grid-cols-2 gap-4 rounded-3 border border-border bg-surface p-3">
              <div className="col-span-2">
                <NamePair
                  value={draft.names}
                  onChange={(names) => {
                    change({ names });
                  }}
                  urduLabel={<Localized id="gardens-field-name-ur" />}
                  englishLabel={<Localized id="gardens-field-name-en" />}
                  urduError={fieldError(error, 'name_ur')}
                  englishError={fieldError(error, 'name_en')}
                />
              </div>
              <Field
                label={<Localized id="gardens-field-date" />}
                error={fieldError(error, 'date')}
              >
                {(props) => (
                  <DateInput {...props} value={draft.date} readOnly onChange={() => undefined} />
                )}
              </Field>
              <Field
                label={<Localized id="glossary-qism-maal" />}
                error={fieldError(error, 'item_ids') ?? fieldError(error, 'item_id')}
              >
                {(props) => (
                  <MultiPicker
                    {...props}
                    scope="item"
                    values={draft.items}
                    valueOf={pickedOf}
                    onChange={(items) => {
                      change({ items });
                    }}
                  />
                )}
              </Field>
              <Field
                label={<Localized id="gardens-field-rent" />}
                error={fieldError(error, 'rent')}
              >
                {(props) => (
                  <MoneyInput
                    {...props}
                    value={draft.rent}
                    onChange={(rent) => {
                      change({ rent });
                    }}
                  />
                )}
              </Field>
              {rentChanged ? (
                <div className="col-span-2 flex flex-col gap-3">
                  <Notice tone="info">
                    <Localized id="gardens-rent-change-help" />
                  </Notice>
                  <div className="grid grid-cols-2 gap-4">
                    <Field
                      label={<Localized id="gardens-rent-change-reason" />}
                      hint={<Localized id="gardens-rent-change-reason-hint" />}
                      error={
                        tried && rentReason.trim() === '' ? (
                          <Localized id="err-required">
                            <span className="block" />
                          </Localized>
                        ) : (
                          fieldError(error, 'reason')
                        )
                      }
                    >
                      {(props) => (
                        <FreeTextInput
                          {...props}
                          value={rentReason}
                          onChange={(text) => {
                            setRentReason(text);
                            reason.set(text);
                            setTried(false);
                          }}
                        />
                      )}
                    </Field>
                    <Field
                      label={<Localized id="edit-fix-sign-label" />}
                      hint={<Localized id="gardens-rent-sign-hint" />}
                      error={
                        tried && sign.trim() === '' ? (
                          <Localized id="err-required">
                            <span className="block" />
                          </Localized>
                        ) : (
                          fieldError(error, 'sign')
                        )
                      }
                    >
                      {(props) => (
                        // The sign is compared as typed: raw text, never converted from Roman letters.
                        <TextInput
                          {...props}
                          value={sign}
                          onChange={(text) => {
                            setSign(text);
                            setTried(false);
                          }}
                        />
                      )}
                    </Field>
                  </div>
                </div>
              ) : null}
              <Field
                label={<Localized id="gardens-field-season-end" />}
                error={fieldError(error, 'season_end')}
              >
                {(props) => (
                  <DateInput
                    {...props}
                    value={draft.seasonEnd}
                    onChange={(seasonEnd) => {
                      change({ seasonEnd });
                    }}
                  />
                )}
              </Field>
              <div className="col-span-2">
                <Field
                  label={<Localized id="gardens-field-note" />}
                  error={fieldError(error, 'note')}
                >
                  {(props) => (
                    <FreeTextInput
                      {...props}
                      value={draft.note}
                      onChange={(note) => {
                        change({ note });
                      }}
                    />
                  )}
                </Field>
              </div>
            </div>
          }
          aside={
            <EditReviewCard review={review.review} pending={review.pending} error={review.error} />
          }
        />
      </form>
      <EditSaveBoxes flow={flow} date={garden.entry_date} />
    </div>
  );
}
