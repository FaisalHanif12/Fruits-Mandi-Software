// The facts of a garden open for edit (plan 44, PRD FR-7.70, FR-7.71): the names, the main fruit, the
// rent, the end of the season and the note. The day and the shares never change (PRD FR-12.29): the
// band says so and the day box is read only. A header can be changed at any time; it posts nothing,
// so there is no cash question. Under the form the backend's review shows what changed. Save asks
// why when the garden is not of today.
//
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money (CLAUDE.md rule 1): the changes and the need for a reason are
//   the backend's, from the review (the same function as the save).

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
import { useGrouping } from '../../../shared/format';
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
  Picker,
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

export function GardenHeaderEdit({ garden, onDone }: GardenHeaderEditProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const stamp = useEntryStamp('garden', garden.id);
  const [draft, setDraft] = useState<GardenDraft>(() => gardenHeaderDraftOf(garden, grouping));
  const reason = useReasonText();

  const change = (patch: Partial<GardenDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
  };

  const editInput = useMemo<GardenHeaderEditInput>(
    () => ({
      id: garden.id,
      version: garden.version,
      garden: buildGarden(draft, false),
      reason: '',
    }),
    [draft, garden.id, garden.version],
  );
  const review = useEditReview(
    `garden-header:${String(garden.id)}`,
    editInput,
    gardensHeaderEditPreview,
  );

  const saveHeader = useSaveGardenHeaderEdit();
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
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
      }),
    onSaved: () => {
      showToast('success', l10n.getString('edit-saved', { number: garden.code }));
      void finishFixSession(l10n);
      onDone();
    },
  });

  const previewError: UiError | null = useMemo(() => {
    const shown = review.errors.filter(
      (item) => item.code !== 'ERR_REASON_REQUIRED' && item.code !== 'ERR_REQUIRED',
    );
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
        {error === null ? null : <ErrorNotice error={error} fieldsShown={GARDEN_FIELD_PATHS} />}
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
                error={fieldError(error, 'item_id')}
              >
                {(props) => (
                  <Picker
                    {...props}
                    scope="item"
                    value={draft.item}
                    onChange={(hit) => {
                      change({ item: hit === null ? null : pickedOf(hit) });
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
