// The facts of a trip open for edit (plan 44, PRD FR-7.70, FR-7.71): the market, who was sent and the
// note. The day and the shares never change (PRD FR-12.29): the band says so and the day box is read
// only. A header can be changed at any time; it posts nothing, so there is no cash question. Under
// the form the backend's review shows what changed. Save asks why when the trip is not of today.
//
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money (CLAUDE.md rule 1): the changes and the need for a reason are
//   the backend's, from the review (the same function as the save).

import { Localized, useLocalization } from '@fluent/react';
import { useMemo, useState } from 'react';
import type { TripHeaderEditInput } from '../../../bindings/TripHeaderEditInput';
import type { UiError } from '../../../bindings/UiError';
import {
  finishFixSession,
  useEditReview,
  useEditSave,
  useEntryStamp,
  useReasonText,
} from '../../../shared/edit';
import { formatBps, isoToDisplayDate } from '../../../shared/format';
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
  fieldError,
  showToast,
} from '../../../shared/ui';
import { tripsHeaderEditPreview } from '../api';
import { useSaveTripHeaderEdit } from '../hooks';
import type { TripInput, TripRow } from '../types';

/** The fields of a trip's header, by the field paths the backend's problems use. */
const HEADER_FIELD_PATHS = ['date', 'market', 'sent_person', 'note'] as const;

interface HeaderDraft {
  market: string;
  sentPerson: string;
  note: string;
}

/** The trip as the backend takes it: the typed fields, and its own day and shares unchanged. */
function tripInputOf(trip: TripRow, draft: HeaderDraft): TripInput {
  const text = (value: string): string | null => (value.trim() === '' ? null : value.trim());
  return {
    date: isoToDisplayDate(trip.entry_date),
    market: text(draft.market),
    sent_person: text(draft.sentPerson),
    note: text(draft.note),
    partners: trip.partners.map((row) => ({
      partner_id: row.partner.id,
      share: formatBps(row.share_bps),
      source: row.source,
    })),
  };
}

interface TripHeaderEditProps {
  trip: TripRow;
  /** The edit is over (saved or left): the page goes back to the entry form. */
  onDone: () => void;
}

export function TripHeaderEdit({ trip, onDone }: TripHeaderEditProps) {
  const { l10n } = useLocalization();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const stamp = useEntryStamp('trip', trip.id);
  const [draft, setDraft] = useState<HeaderDraft>({
    market: trip.market ?? '',
    sentPerson: trip.sent_person ?? '',
    note: trip.note ?? '',
  });
  const reason = useReasonText();

  const editInput = useMemo<TripHeaderEditInput>(
    () => ({
      id: trip.id,
      version: trip.version,
      trip: tripInputOf(trip, draft),
      reason: '',
    }),
    [trip, draft],
  );
  const review = useEditReview(`trip-header:${String(trip.id)}`, editInput, tripsHeaderEditPreview);

  const saveHeader = useSaveTripHeaderEdit();
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const flow = useEditSave({
    review: review.review,
    reason,
    draftText,
    send: (why) =>
      saveHeader.mutateAsync({
        id: trip.id,
        version: trip.version,
        trip: tripInputOf(trip, draft),
        reason: why,
      }),
    onSaved: () => {
      showToast('success', l10n.getString('edit-saved', { number: trip.code }));
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
        kind="trip"
        number={trip.code}
        date={trip.entry_date}
        edited={stamp}
        history={{ entity: 'trip', id: trip.id }}
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
        {error === null ? null : <ErrorNotice error={error} fieldsShown={HEADER_FIELD_PATHS} />}
        <FormAside
          form={
            <div className="grid grid-cols-2 gap-4 rounded-3 border border-border bg-surface p-3">
              <Field label={<Localized id="trips-field-date" />} error={fieldError(error, 'date')}>
                {(props) => (
                  <DateInput
                    {...props}
                    value={isoToDisplayDate(trip.entry_date)}
                    readOnly
                    onChange={() => undefined}
                  />
                )}
              </Field>
              <div />
              <Field
                label={<Localized id="trips-field-market" />}
                error={fieldError(error, 'market')}
              >
                {(props) => (
                  <FreeTextInput
                    {...props}
                    value={draft.market}
                    onChange={(market) => {
                      setDraft((current) => ({ ...current, market }));
                    }}
                  />
                )}
              </Field>
              <Field
                label={<Localized id="trips-field-sent" />}
                error={fieldError(error, 'sent_person')}
              >
                {(props) => (
                  <FreeTextInput
                    {...props}
                    value={draft.sentPerson}
                    onChange={(sentPerson) => {
                      setDraft((current) => ({ ...current, sentPerson }));
                    }}
                  />
                )}
              </Field>
              <div className="col-span-2">
                <Field
                  label={<Localized id="trips-field-note" />}
                  error={fieldError(error, 'note')}
                >
                  {(props) => (
                    <FreeTextInput
                      {...props}
                      value={draft.note}
                      onChange={(note) => {
                        setDraft((current) => ({ ...current, note }));
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
      <EditSaveBoxes flow={flow} date={trip.entry_date} />
    </div>
  );
}
