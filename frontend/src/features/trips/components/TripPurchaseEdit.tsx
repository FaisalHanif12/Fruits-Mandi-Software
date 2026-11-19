// A saved trip purchase open for edit (plan 44, PRD FR-7.70, FR-7.71, FR-12.19): the same fields as a
// new purchase, filled as it was saved, under the amber band. The day never changes. A purchase can
// be changed only while nothing of it was sold or written off; after that the Edit button says so.
// Under the form the backend's review shows what changed and what it does to the cash and the
// partners. Save asks why when the purchase is not of today and asks "is this entry correct?" when
// it takes cash below zero (PRD FR-6.7). One save writes the reversal and the new posting.
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
  EditFormBar,
  EditReviewCard,
  EditSaveBoxes,
  EntrySources,
  ErrorNotice,
  FormAside,
  InfoHint,
  showToast,
} from '../../../shared/ui';
import {
  PURCHASE_EDIT_FIELD_PATHS,
  PurchaseEditFields,
  buildPurchaseEdit,
  purchaseEditDraftOf,
  purchaseNumberText,
  purchasesEditPreview,
  useSavePurchaseEdit,
  type PurchaseEditDraft,
  type PurchaseRow,
} from '../../purchases';
import type { TripEntryRow, TripRow } from '../types';

interface TripPurchaseEditProps {
  trip: TripRow;
  /** The purchase that is open for edit. */
  purchase: PurchaseRow;
  /** Its row in the trip's history: the sources each partner's part had on it. */
  entry: TripEntryRow;
  /** The edit is over (saved or left): the page goes back to the entry form. */
  onDone: () => void;
}

export function TripPurchaseEdit({ trip, purchase, entry, onDone }: TripPurchaseEditProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const reopened = useFixStore((state) => state.session);
  const [draft, setDraft] = useState<PurchaseEditDraft>(() =>
    purchaseEditDraftOf(purchase, entry.parts, grouping),
  );
  const reason = useReasonText();
  const number = purchaseNumberText(purchase.number, (id, vars) => l10n.getString(id, vars));

  const change = (patch: Partial<PurchaseEditDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
  };

  // What the review is asked: the purchase as typed now, with the version read.
  const editInput = useMemo(
    () => ({
      id: purchase.id,
      version: purchase.version,
      purchase: buildPurchaseEdit(draft, trip.id, trip.partners, false),
      reason: '',
    }),
    [draft, purchase.id, purchase.version, trip.id, trip.partners],
  );
  const review = useEditReview(`purchase:${String(purchase.id)}`, editInput, purchasesEditPreview);

  const savePurchase = useSavePurchaseEdit();
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const flow = useEditSave({
    review: review.review,
    reason,
    draftText,
    send: (why, confirm) =>
      savePurchase.mutateAsync({
        id: purchase.id,
        version: purchase.version,
        purchase: buildPurchaseEdit(draft, trip.id, trip.partners, confirm),
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
        kind="purchase"
        number={number}
        date={purchase.entry_date}
        reopenedDays={reopened?.days ?? []}
        edited={purchase.edited}
        history={{ entity: 'purchase', id: purchase.id }}
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
          <ErrorNotice error={error} fieldsShown={PURCHASE_EDIT_FIELD_PATHS} />
        )}
        <FormAside
          form={
            <div className="grid grid-cols-2 gap-4 rounded-3 border border-border bg-surface p-3">
              <PurchaseEditFields draft={draft} onChange={change} error={error} />
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
            </div>
          }
          aside={
            <EditReviewCard review={review.review} pending={review.pending} error={review.error} />
          }
        />
      </form>
      <EditSaveBoxes
        flow={flow}
        date={purchase.entry_date}
        initialReason={reopened?.reason ?? ''}
      />
    </div>
  );
}
