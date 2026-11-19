// A saved garden installment, running cost or sale in another shop open for edit (plan 44, PRD
// FR-7.70, FR-7.71, FR-12.19): the same fields as a new entry, filled as it was saved, under the
// amber band. The day never changes (the band says so and the day box is read only); the type is the
// entry's own. Under the form the backend's review shows what changed and what it does to the
// balances, the cash and the partners. Save asks why when the entry is not of today and asks "is this
// entry correct?" when it takes cash below zero (PRD FR-6.7). One save writes the reversal and the
// new posting.
//
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money (CLAUDE.md rule 1): the changes, the effect, the cash and the
//   need for a reason are the backend's, from the review (the same function as the save).
// - Four states: the form (always filled), the review loading or failed (a box, with the form
//   still usable), and the refusal of a save (at its fields and in words).

import { Localized, useLocalization } from '@fluent/react';
import { useMemo, useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import {
  finishFixSession,
  useEditReview,
  useEditSave,
  useFixStore,
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
  EntrySources,
  ErrorNotice,
  Field,
  FormAside,
  InfoHint,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { gardensEntryEditPreview } from '../api';
import { ENTRY_FIELD_PATHS, buildEntry } from '../draft';
import { draftOfGardenEntry } from '../edit-seed';
import { useArrivalsList, useSaveGardenEntryEdit } from '../hooks';
import { historyKindText } from '../list-columns';
import type { EntryDraft, GardenHistoryRow, GardenRow } from '../types';
import { GardenEntryFieldsForm } from './GardenEntryFieldsForm';

interface GardenEntryEditProps {
  garden: GardenRow;
  /** The installment, cost or sale in another shop that is open for edit. */
  row: GardenHistoryRow;
  /** The edit is over (saved or left): the page goes back to the entry form. */
  onDone: () => void;
}

export function GardenEntryEdit({ garden, row, onDone }: GardenEntryEditProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const reopened = useFixStore((state) => state.session);
  const [draft, setDraft] = useState<EntryDraft>(() => draftOfGardenEntry(row, grouping));
  const reason = useReasonText();
  const payment = draft.type === 'installment' || draft.type === 'cost';
  // What the entry is called in the message: "Installment", "Cost", "Sale in another shop".
  const kindText = historyKindText(row.kind, (id, vars) => l10n.getString(id, vars));
  // The entry's own number in the band and the message: "Installment 1".
  const numberText = row.number === null ? kindText : `${kindText} ${String(row.number)}`;
  // The garden's goods arrived: a sale in another shop takes its نگ from one of them (FR-12.41).
  const arrivalList = useArrivalsList(garden.id);
  const arrivals = useMemo(
    () => arrivalList.data?.pages.flatMap((page) => page.rows) ?? [],
    [arrivalList.data],
  );

  const change = (patch: Partial<EntryDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
  };

  // What the review is asked: the entry as typed now, with the version read.
  const editInput = useMemo(
    () =>
      row.entry_id === null || row.version === null
        ? null
        : {
            id: row.entry_id,
            version: row.version,
            entry: buildEntry(draft, garden.id, garden.partners, false),
            reason: '',
            confirm_cash_below_zero: false,
          },
    [row.entry_id, row.version, draft, garden.id, garden.partners],
  );
  const review = useEditReview(
    `garden-entry:${String(row.entry_id)}`,
    editInput,
    gardensEntryEditPreview,
  );

  const saveEntry = useSaveGardenEntryEdit();
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const flow = useEditSave({
    review: review.review,
    reason,
    draftText,
    send: (why, confirm) => {
      if (row.entry_id === null || row.version === null) {
        return Promise.reject(new Error('This row has no entry to change.'));
      }
      return saveEntry.mutateAsync({
        id: row.entry_id,
        version: row.version,
        entry: buildEntry(draft, garden.id, garden.partners, confirm),
        reason: why,
        confirm_cash_below_zero: confirm,
      });
    },
    onSaved: () => {
      showToast('success', l10n.getString('edit-saved', { number: numberText }));
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
        number={numberText}
        date={row.entry_date}
        reopenedDays={reopened?.days ?? []}
        edited={row.edited}
        history={{ entity: 'garden_entry', id: row.entry_id ?? 0 }}
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
          <ErrorNotice error={error} fieldsShown={ENTRY_FIELD_PATHS[draft.type]} />
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
              <div />
              <GardenEntryFieldsForm
                draft={draft}
                onChange={change}
                error={error}
                arrivals={arrivals}
              />
              {payment ? (
                <div className="col-span-2 flex flex-col gap-1">
                  <div className="flex items-center gap-1">
                    <Localized id="gardens-sources-title">
                      <h3 className="type-label text-ink-muted" />
                    </Localized>
                    <InfoHint>
                      <Localized id="gardens-sources-hint">
                        <p />
                      </Localized>
                    </InfoHint>
                  </div>
                  <EntrySources
                    partners={garden.partners}
                    value={draft.sources}
                    onChange={(partnerId, source) => {
                      change({ sources: { ...draft.sources, [partnerId]: source } });
                    }}
                  />
                </div>
              ) : null}
            </div>
          }
          aside={
            <EditReviewCard review={review.review} pending={review.pending} error={review.error} />
          }
        />
      </form>
      <EditSaveBoxes flow={flow} date={row.entry_date} initialReason={reopened?.reason ?? ''} />
    </div>
  );
}
