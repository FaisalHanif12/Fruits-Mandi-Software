// A saved garden goods arrived (مال آیا) open for edit (plan 44, PRD FR-7.70, FR-7.71, FR-12.19): the
// same fields as a new arrival line, filled as it was saved, under the amber band. The day never
// changes. An arrival posts nothing, so there is no cash question; it can be changed only while
// nothing of it was sold or written off. Under the form the backend's review shows what changed.
// Save asks why when the arrival is not of today.
//
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money (CLAUDE.md rule 1): the changes and the need for a reason are
//   the backend's, from the review (the same function as the save).

import { Localized, useLocalization } from '@fluent/react';
import { useMemo, useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { pickedOf } from '../../../shared/accounts';
import {
  finishFixSession,
  useEditReview,
  useEditSave,
  useFixStore,
  useReasonText,
} from '../../../shared/edit';
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
  Picker,
  QtyInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { gardensArrivalEditPreview } from '../api';
import {
  ARRIVAL_EDIT_FIELD_PATHS,
  arrivalEditDraftOf,
  buildArrivalEdit,
  type ArrivalEditDraft,
} from '../edit-seed';
import { useSaveArrivalEdit } from '../hooks';
import { arrivalNumberText } from '../list-columns';
import type { ArrivalRow, GardenRow } from '../types';

interface GardenArrivalEditProps {
  garden: GardenRow;
  /** The goods arrived that are open for edit. */
  arrival: ArrivalRow;
  /** The edit is over (saved or left): the page goes back to the entry form. */
  onDone: () => void;
}

export function GardenArrivalEdit({ garden, arrival, onDone }: GardenArrivalEditProps) {
  const { l10n } = useLocalization();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const reopened = useFixStore((state) => state.session);
  const [draft, setDraft] = useState<ArrivalEditDraft>(() => arrivalEditDraftOf(arrival));
  const reason = useReasonText();
  const number = arrivalNumberText(arrival.number, (id, vars) => l10n.getString(id, vars));
  // The weight of an arrival is asked for an item sold by weight: the item chosen says so, and a
  // weight already there stays (PRD FR-12.38).
  const byWeight = draft.item?.soldByWeight === true || draft.weight !== '';

  const change = (patch: Partial<ArrivalEditDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
  };

  const editInput = useMemo(
    () => ({
      id: arrival.id,
      version: arrival.version,
      arrival: buildArrivalEdit(draft, garden.id),
      reason: '',
    }),
    [draft, arrival.id, arrival.version, garden.id],
  );
  const review = useEditReview(
    `garden-arrival:${String(arrival.id)}`,
    editInput,
    gardensArrivalEditPreview,
  );

  const saveArrival = useSaveArrivalEdit();
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const flow = useEditSave({
    review: review.review,
    reason,
    draftText,
    send: (why) =>
      saveArrival.mutateAsync({
        id: arrival.id,
        version: arrival.version,
        arrival: buildArrivalEdit(draft, garden.id),
        reason: why,
      }),
    onSaved: () => {
      showToast('success', l10n.getString('edit-saved', { number }));
      void finishFixSession(l10n);
      onDone();
    },
  });

  // What the review found, in the shape of a refused save, so the fields can show it. A reason that
  // is still to be asked is not a mistake of the form.
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
        number={number}
        date={arrival.entry_date}
        reopenedDays={reopened?.days ?? []}
        edited={arrival.edited}
        history={{ entity: 'garden_arrival', id: arrival.id }}
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
          <ErrorNotice error={error} fieldsShown={ARRIVAL_EDIT_FIELD_PATHS} />
        )}
        <FormAside
          form={
            <div className="grid grid-cols-2 gap-4 rounded-3 border border-border bg-surface p-3">
              <Field
                label={<Localized id="gardens-field-date" />}
                error={fieldError(error, 'date')}
              >
                {(props) => (
                  <DateInput {...props} value={draft.date} readOnly onChange={() => undefined} />
                )}
              </Field>
              <Field
                label={<Localized id="glossary-sell-date" />}
                info={
                  <Localized id="gardens-arrival-sell-date-hint">
                    <p />
                  </Localized>
                }
                error={fieldError(error, 'sell_date')}
              >
                {(props) => (
                  <DateInput
                    {...props}
                    value={draft.sellDate}
                    onChange={(sellDate) => {
                      change({ sellDate });
                    }}
                  />
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
                label={<Localized id="glossary-packing" />}
                error={fieldError(error, 'packing_id')}
              >
                {(props) => (
                  <Picker
                    {...props}
                    scope="packing"
                    value={draft.packing}
                    onChange={(hit) => {
                      change({ packing: hit === null ? null : pickedOf(hit) });
                    }}
                  />
                )}
              </Field>
              <Field label={<Localized id="goods-line-col-qty" />} error={fieldError(error, 'qty')}>
                {(props) => (
                  <QtyInput
                    {...props}
                    value={draft.qty}
                    onChange={(qty) => {
                      change({ qty });
                    }}
                  />
                )}
              </Field>
              {byWeight ? (
                <Field
                  label={<Localized id="goods-line-col-weight" />}
                  error={fieldError(error, 'weight')}
                >
                  {(props) => (
                    <FreeTextInput
                      {...props}
                      value={draft.weight}
                      onChange={(weight) => {
                        change({ weight });
                      }}
                    />
                  )}
                </Field>
              ) : (
                <div />
              )}
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
      <EditSaveBoxes flow={flow} date={arrival.entry_date} initialReason={reopened?.reason ?? ''} />
    </div>
  );
}
