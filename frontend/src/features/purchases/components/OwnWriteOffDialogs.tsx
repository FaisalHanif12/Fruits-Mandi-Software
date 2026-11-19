// Writing trip goods off, and undoing a write-off (PRD FR-12.28, FR-7.8). Both ask for a reason and
// say exactly what will happen. Nothing is posted: the partners paid for the goods already and no
// money comes back, so the cost of the goods written off only shows as a loss in the trip's
// result. The quantity goes to the backend as typed (a simple sum is allowed, FR-7.48); the
// backend checks it against what is left, works out the cost (C21) and answers with the new
// numbers. A refusal stays inside the box and keeps what was typed. The same two boxes serve the
// trip screen and the list of باقی مال.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { formatPaisa, isoToDisplayDate, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  Field,
  Num,
  QtyInput,
  ReasonDialog,
  Select,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useWriteOffOwn, useWriteOffOwnUndo } from '../hooks';
import { purchaseNumberText } from '../list-columns';
import type { OwnStockTarget } from '../types';

/** The fields whose problems the boxes show under the field itself. */
const WRITE_OFF_FIELDS = ['qty'];
const UNDO_FIELDS = ['move_id'];

interface DialogProps {
  /** The purchase, or null when the box is closed. */
  target: OwnStockTarget | null;
  onClose: () => void;
}

/** "Write off" (PRD FR-12.28): how many نگ, and why. */
export function OwnWriteOffDialog({ target, onClose }: DialogProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const writeOff = useWriteOffOwn();
  const [qty, setQty] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  const close = () => {
    setQty('');
    setError(null);
    onClose();
  };
  return (
    <ReasonDialog
      open={target !== null}
      onOpenChange={(open) => {
        if (!open) close();
      }}
      title={<Localized id="purchases-write-off-title" />}
      description={
        target === null ? null : (
          <Localized
            id="purchases-write-off-text"
            vars={{
              number: purchaseNumberText(target.number, (id, vars) => l10n.getString(id, vars)),
              item: displayName(target.item, language),
              packing: displayName(target.packing, language),
              qty_left: String(target.qtyLeft),
            }}
          >
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="purchases-write-off-reason" />}
      confirmLabel={<Localized id="purchases-write-off-confirm" />}
      error={error}
      busy={writeOff.isPending}
      extraFields={WRITE_OFF_FIELDS}
      extra={
        <Field label={<Localized id="purchases-write-off-qty" />} error={fieldError(error, 'qty')}>
          {(control) => <QtyInput {...control} value={qty} onChange={setQty} />}
        </Field>
      }
      onConfirm={(reason) => {
        if (target === null) return;
        setError(null);
        writeOff
          .mutateAsync({ purchaseId: target.purchaseId, qty: qty.trim(), reason })
          .then((done) => {
            showToast(
              'success',
              <Localized
                id={
                  done.trip_closed ? 'purchases-write-off-done-closed' : 'purchases-write-off-done'
                }
                vars={{
                  qty: String(done.qty),
                  cost: formatPaisa(done.cost_paisa, { grouping }),
                }}
                elems={{ num: <Num /> }}
              >
                <span />
              </Localized>,
            );
            close();
          })
          .catch((failure: unknown) => {
            setError(uiErrorOf(failure));
          });
      }}
    />
  );
}

/** "Undo write-off" (PRD FR-12.28): which write-off, and why. */
export function UndoOwnWriteOffDialog({ target, onClose }: DialogProps) {
  const { l10n } = useLocalization();
  const undo = useWriteOffOwnUndo();
  const [chosen, setChosen] = useState<string | undefined>(undefined);
  const [error, setError] = useState<UiError | null>(null);
  const close = () => {
    setChosen(undefined);
    setError(null);
    onClose();
  };
  // Only one write-off to undo: it is the choice already.
  const writeOffs = target?.writeOffs ?? [];
  const moveId = chosen ?? (writeOffs.length === 1 ? String(writeOffs[0]?.move_id) : undefined);
  return (
    <ReasonDialog
      open={target !== null}
      onOpenChange={(open) => {
        if (!open) close();
      }}
      title={<Localized id="purchases-write-off-undo-title" />}
      description={<Localized id="purchases-write-off-undo-text" />}
      reasonLabel={<Localized id="purchases-write-off-reason" />}
      confirmLabel={<Localized id="purchases-write-off-undo-confirm" />}
      error={error}
      busy={undo.isPending}
      extraFields={UNDO_FIELDS}
      extra={
        <Field
          label={<Localized id="purchases-write-off-undo-which" />}
          error={fieldError(error, 'move_id')}
        >
          {(control) => (
            <Select
              {...control}
              value={moveId}
              onChange={setChosen}
              options={writeOffs.map((item) => ({
                value: String(item.move_id),
                label: l10n.getString('purchases-write-off-undo-option', {
                  date: isoToDisplayDate(item.move_date),
                  qty: String(item.qty),
                  reason: item.reason ?? '',
                }),
              }))}
            />
          )}
        </Field>
      }
      onConfirm={(reason) => {
        if (moveId === undefined) {
          // Several write-offs and none chosen: say so at the field, send nothing.
          setError({
            ok: false,
            errors: [{ code: 'ERR_REQUIRED', field: 'move_id', values: {} }],
            ref: '',
          });
          return;
        }
        setError(null);
        undo
          .mutateAsync({ moveId: Number(moveId), reason })
          .then((done) => {
            showToast(
              'success',
              <Localized
                id={
                  done.trip_reopened
                    ? 'purchases-write-off-undone-reopened'
                    : 'purchases-write-off-undone'
                }
              >
                <span />
              </Localized>,
            );
            close();
          })
          .catch((failure: unknown) => {
            setError(uiErrorOf(failure));
          });
      }}
    />
  );
}
