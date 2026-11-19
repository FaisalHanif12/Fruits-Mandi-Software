// An entry of the روکڑ area open for edit (plan 44, PRD FR-7.70): the خرچہ, ادھار رقم, الگ کھاتہ, سرمایہ
// and جنرل screens open a saved voucher with `?id=`, fill the same form as a new entry, and save with
// `vouchers_edit`. The pieces every one of them needs are here, so no screen builds its own: the
// state of the edit (the row, the review from the backend, the reason), the band over the form with
// the stamp and the cancel button, and the boxes (the reason, the reopen of a closed day, the cancel).
// Nothing here works out money: the changes and their effect are the backend's review.

import { Localized } from '@fluent/react';
import { useQueryClient } from '@tanstack/react-query';
import { Ban } from 'lucide-react';
import { useState, type ReactNode } from 'react';
import type { EditStamp } from '../../bindings/EditStamp';
import type { VoucherSaved } from '../../bindings/VoucherSaved';
import {
  useEditReview,
  useEntryMarks,
  useFixDayFlow,
  useFixStore,
  useReasonText,
  useVoucherHref,
  type EntryMarks,
  type ReasonText,
} from '../../shared/edit';
import {
  Button,
  EditBand,
  EditReasonDialog,
  EditReviewCard,
  EditStampLine,
  EntryActions,
  type EditBandKind,
} from '../../shared/ui';
import { CancelVoucherDialog } from '../vouchers/components/CancelVoucherDialog';
import { SavedVoucherView, VOUCHERS_KEY, useEditVoucher, vouchersEditPreview } from '../vouchers';
import type { VoucherEditable, VoucherEntry, VoucherRow } from './types';

/** The voucher that is open for edit: its row (with the version and the flags) and the entry as saved. */
export interface VoucherEditing {
  row: VoucherRow;
  edited: EditStamp | null;
}

/** The kind of band for a voucher type. */
export function bandKindOf(type: VoucherRow['voucher_type']): EditBandKind {
  switch (type) {
    case 'shop_expense':
    case 'personal_expense':
    case 'personal_return':
      return 'expense';
    case 'money_lent':
    case 'money_lent_return':
      return 'lent';
    case 'khata_out':
    case 'khata_in':
      return 'khata';
    case 'capital_in':
    case 'capital_out':
      return 'capital';
    case 'general':
      return 'journal';
    case 'receipt':
      return 'receipt';
    case 'payment':
      return 'payment';
    case 'advance':
      return 'advance';
  }
}

/** The state of one edit: the review, the reason, the save and the boxes. */
export interface VoucherEditState {
  editing: VoucherEditing;
  /** What earlier edits changed on this voucher: the "Before ..." marks of its fields (FR-7.71). */
  marks: EntryMarks;
  review: ReturnType<typeof useEditReview>;
  reason: ReasonText;
  /** The reason box is open: it was asked for because the entry is not of today. */
  askReason: boolean;
  setAskReason: (open: boolean) => void;
  /** The reason is needed and has not been typed yet: the screen opens the box instead of saving. */
  needsReason: () => boolean;
  /** Sends the edit. The answer is the saved voucher, as for a new one. */
  send: (entry: VoucherEntry) => Promise<VoucherSaved>;
  /** The days reopened for this fix, if any. */
  reopenedDays: readonly string[];
  fix: ReturnType<typeof useFixDayFlow>;
  cancelRow: VoucherRow | null;
  setCancelRow: (row: VoucherRow | null) => void;
}

/** The cash question of an entry, as it was asked (the types that never ask have none). */
function confirmOf(entry: VoucherEntry): boolean {
  return 'confirm_cash_below_zero' in entry && entry.confirm_cash_below_zero;
}

/**
 * `editable` is the voucher read for the edit (null while a new entry is typed). `entry` is the
 * entry as the form holds it now, for the backend's review.
 */
export function useVoucherEdit(
  editable: VoucherEditable | null,
  entry: VoucherEntry | null,
): VoucherEditState | null {
  const edit = useEditVoucher();
  const reason = useReasonText();
  const [askReason, setAskReason] = useState(false);
  const [cancelRow, setCancelRow] = useState<VoucherRow | null>(null);
  const reopened = useFixStore((state) => state.session);
  const fix = useFixDayFlow();
  const row = editable?.row ?? null;
  const marks = useEntryMarks('voucher', row?.id ?? null, row?.edited != null);
  const review = useEditReview(
    `voucher:${String(row?.id ?? 0)}`,
    row === null || entry === null
      ? null
      : { id: row.id, version: row.version, entry, reason: reason.get() },
    vouchersEditPreview,
  );
  if (row === null) return null;
  return {
    editing: { row, edited: row.edited },
    marks,
    review,
    reason,
    askReason,
    setAskReason,
    // The backend says whether the entry needs a reason; until it has said, ask (never guess no).
    needsReason: () => (review.review?.reason_required ?? true) && reason.get().trim() === '',
    send: (built) =>
      edit.mutateAsync({
        id: row.id,
        version: row.version,
        entry: built,
        reason: reason.get().trim(),
        confirm: confirmOf(built),
      }),
    reopenedDays: reopened?.days ?? [],
    fix,
    cancelRow,
    setCancelRow,
  };
}

interface VoucherEditTopProps {
  edit: VoucherEditState;
}

/** The amber band, the stamp and the cancel button over the form of an entry open for edit. */
export function VoucherEditTop({ edit }: VoucherEditTopProps) {
  const { row } = edit.editing;
  return (
    <>
      <EditBand
        kind={bandKindOf(row.voucher_type)}
        number={String(row.number)}
        date={row.entry_date}
        reopenedDays={edit.reopenedDays}
      />
      {edit.editing.edited === null ? null : (
        <EditStampLine stamp={edit.editing.edited} history={{ entity: 'voucher', id: row.id }} />
      )}
      <div className="flex flex-wrap items-center gap-2">
        <Button
          variant="danger"
          onClick={() => {
            edit.setCancelRow(row);
          }}
        >
          <Ban aria-hidden="true" className="size-icon-md" />
          <Localized id="edit-action-cancel" />
        </Button>
      </div>
    </>
  );
}

interface VoucherEditBoxesProps {
  edit: VoucherEditState;
  /** The edit was cancelled: the screen goes back to a new entry. */
  onCancelled: () => void;
  /** The reason was typed: the screen saves. */
  onReason: () => void;
  busy: boolean;
}

/** The change list and its effect, and the boxes of an edit (the reason, reopen, cancel). */
export function VoucherEditBoxes({
  edit,
  onCancelled,
  onReason,
  busy,
}: VoucherEditBoxesProps): ReactNode {
  return (
    <>
      <EditReviewCard
        review={edit.review.review}
        pending={edit.review.pending}
        error={edit.review.error}
      />
      <EditReasonDialog
        open={edit.askReason}
        onOpenChange={edit.setAskReason}
        date={edit.editing.row.entry_date}
        error={null}
        busy={busy}
        initialReason={edit.reopenedDays.length === 0 ? '' : edit.reason.get()}
        onConfirm={(text) => {
          edit.reason.set(text);
          edit.setAskReason(false);
          onReason();
        }}
      />
      {edit.fix.dialog}
      <CancelVoucherDialog
        row={edit.cancelRow}
        onClose={() => {
          edit.setCancelRow(null);
        }}
        onCancelled={() => {
          edit.setCancelRow(null);
          onCancelled();
        }}
      />
    </>
  );
}

interface ClosedCashEntryProps {
  /** The Fluent message of the screen's title. */
  titleId: string;
  row: VoucherRow;
}

/**
 * A saved entry that cannot be edited yet (plan 44, PRD FR-7.70, FR-21.4): its day is closed, so
 * Edit reopens the day and then opens the form; or it is locked, and the buttons say why in words.
 * The entry is shown as stored; nothing is worked out again.
 */
export function ClosedCashEntry({ titleId, row }: ClosedCashEntryProps) {
  const queryClient = useQueryClient();
  const hrefOf = useVoucherHref();
  const fix = useFixDayFlow();
  const [cancelRow, setCancelRow] = useState<VoucherRow | null>(null);
  return (
    <div className="flex max-w-page flex-col gap-4">
      <Localized id={titleId}>
        <h1 className="type-title" />
      </Localized>
      <SavedVoucherView row={row} />
      <EntryActions
        flags={row}
        busy={fix.busy}
        onEdit={() => {
          fix.start({
            date: row.entry_date,
            ownerHref: hrefOf(row),
            // The entry is read again; with its day open it comes back ready to edit.
            then: () => {
              void queryClient.invalidateQueries({ queryKey: VOUCHERS_KEY });
            },
          });
        }}
        onCancel={() => {
          setCancelRow(row);
        }}
      />
      {fix.dialog}
      <CancelVoucherDialog
        row={cancelRow}
        onClose={() => {
          setCancelRow(null);
        }}
        onCancelled={() => {
          setCancelRow(null);
          void queryClient.invalidateQueries({ queryKey: VOUCHERS_KEY });
        }}
      />
    </div>
  );
}

/** An entry is ready for a form only while its day is open and it is not locked. */
export function isReadyToEdit(row: Pick<VoucherRow, 'can_edit' | 'needs_reopen'>): boolean {
  return row.can_edit && !row.needs_reopen;
}
