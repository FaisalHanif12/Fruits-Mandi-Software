// A saved trip cost or sale in another shop open for edit (plan 44, PRD FR-7.70, FR-7.71, FR-12.19):
// the same fields as a new entry, filled as it was saved, under the amber band. The day never
// changes (the band says so and the day box is read only); the type is the entry's own. Under the
// form the backend's review shows what changed and what it does to the balances, the cash and the
// partners. Save asks why when the entry is not of today and asks "is this entry correct?" when it
// takes cash below zero (PRD FR-6.7). One save writes the reversal and the new posting.
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
  useFixStore,
  useReasonText,
  useEditSave,
} from '../../../shared/edit';
import { useGrouping } from '../../../shared/format';
import { useFocusFirstField } from '../../../shared/hooks';
import { useEscape, useFormNavigation, useShortcut, SHORTCUTS } from '../../../shared/keyboard';
import {
  DateInput,
  EditFormBar,
  EditReviewCard,
  EditSaveBoxes,
  EntrySources,
  ErrorNotice,
  Field,
  FormAside,
  FreeTextInput,
  InfoHint,
  fieldError,
  showToast,
} from '../../../shared/ui';
import type { PurchaseRow } from '../../purchases';
import { tripsCostEditPreview, tripsOtherSaleEditPreview } from '../api';
import { COST_FIELD_PATHS, SALE_FIELD_PATHS, buildEntryCost, buildEntrySale } from '../draft';
import { draftOfEntry } from '../edit-seed';
import { useSaveCostEdit, useSaveOtherSaleEdit } from '../hooks';
import type { EntryDraft, TripEntryRow, TripRow } from '../types';
import { CostFieldsForm } from './CostFieldsForm';
import { OtherSaleFieldsForm } from './OtherSaleFieldsForm';

interface TripEntryEditProps {
  trip: TripRow;
  /** The cost or the sale in another shop that is open for edit. */
  entry: TripEntryRow;
  /** The trip's purchases, to name the one the goods of a sale come from. */
  purchases: readonly PurchaseRow[];
  /** The edit is over (saved or left): the page goes back to the entry form. */
  onDone: () => void;
}

export function TripEntryEdit({ trip, entry, purchases, onDone }: TripEntryEditProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const reopened = useFixStore((state) => state.session);
  const [draft, setDraft] = useState<EntryDraft>(() => draftOfEntry(entry, grouping));
  const reason = useReasonText();
  const isCost = entry.kind === 'cost';

  const change = (patch: Partial<EntryDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
  };

  // What the review and the save are asked: the entry as typed now, with the version read.
  const costEdit = useMemo(
    () =>
      isCost
        ? {
            id: entry.id,
            version: entry.version,
            cost: buildEntryCost(draft, trip.id, trip.partners, false),
            reason: '',
          }
        : null,
    [isCost, draft, entry.id, entry.version, trip.id, trip.partners],
  );
  const saleEdit = useMemo(
    () =>
      isCost
        ? null
        : {
            id: entry.id,
            version: entry.version,
            sale: buildEntrySale(draft, trip.id),
            reason: '',
            confirm_cash_below_zero: false,
          },
    [isCost, draft, entry.id, entry.version, trip.id],
  );
  const costReview = useEditReview(`trip-cost:${String(entry.id)}`, costEdit, tripsCostEditPreview);
  const saleReview = useEditReview(
    `trip-sale:${String(entry.id)}`,
    saleEdit,
    tripsOtherSaleEditPreview,
  );
  const review = isCost ? costReview : saleReview;

  const saveCost = useSaveCostEdit();
  const saveSale = useSaveOtherSaleEdit();
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const flow = useEditSave({
    review: review.review,
    reason,
    draftText,
    send: (why, confirm) =>
      isCost
        ? saveCost.mutateAsync({
            id: entry.id,
            version: entry.version,
            cost: buildEntryCost(draft, trip.id, trip.partners, confirm),
            reason: why,
          })
        : saveSale.mutateAsync({
            id: entry.id,
            version: entry.version,
            sale: buildEntrySale(draft, trip.id),
            reason: why,
            confirm_cash_below_zero: confirm,
          }),
    onSaved: () => {
      showToast('success', l10n.getString('edit-saved', { number: String(entry.number) }));
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

  const withStock = purchases.filter(
    (purchase) => purchase.qty_left > 0 || purchase.id === entry.purchase_id,
  );
  const fieldsShown = isCost ? COST_FIELD_PATHS : SALE_FIELD_PATHS;

  return (
    <div className="flex flex-col gap-3">
      <EditFormBar
        kind="trip"
        number={String(entry.number)}
        date={entry.entry_date}
        reopenedDays={reopened?.days ?? []}
        edited={entry.edited}
        history={{ entity: isCost ? 'trip_cost' : 'trip_other_sale', id: entry.id }}
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
        {error === null ? null : <ErrorNotice error={error} fieldsShown={fieldsShown} />}
        <FormAside
          form={
            <div className="grid grid-cols-2 gap-4 rounded-3 border border-border bg-surface p-3">
              <Field label={<Localized id="trips-field-date" />} error={fieldError(error, 'date')}>
                {(props) => (
                  <DateInput {...props} value={draft.date} readOnly onChange={() => undefined} />
                )}
              </Field>
              <div />
              {isCost ? (
                <CostFieldsForm
                  fields={draft.cost}
                  amount={draft.amount}
                  error={error}
                  onChange={({ reason: why, amount }) => {
                    change({
                      ...(why === undefined ? {} : { cost: { reason: why } }),
                      ...(amount === undefined ? {} : { amount }),
                    });
                  }}
                />
              ) : (
                <OtherSaleFieldsForm
                  fields={draft.sale}
                  amount={draft.amount}
                  purchases={withStock}
                  error={error}
                  onChange={({ shop, purchaseId, qty, amount }) => {
                    change({
                      sale: {
                        shop: shop ?? draft.sale.shop,
                        purchaseId: purchaseId ?? draft.sale.purchaseId,
                        qty: qty ?? draft.sale.qty,
                      },
                      ...(amount === undefined ? {} : { amount }),
                    });
                  }}
                />
              )}
              {isCost ? null : (
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
                          change({ note });
                        }}
                      />
                    )}
                  </Field>
                </div>
              )}
              {isCost ? (
                <div className="col-span-2 flex flex-col gap-1">
                  <div className="flex items-center gap-1">
                    <Localized id="trips-sources-title">
                      <h3 className="type-label text-ink-muted" />
                    </Localized>
                    <InfoHint>
                      <Localized id="trips-sources-hint">
                        <p />
                      </Localized>
                    </InfoHint>
                  </div>
                  <EntrySources
                    partners={trip.partners}
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
      <EditSaveBoxes flow={flow} date={entry.entry_date} initialReason={reopened?.reason ?? ''} />
    </div>
  );
}
