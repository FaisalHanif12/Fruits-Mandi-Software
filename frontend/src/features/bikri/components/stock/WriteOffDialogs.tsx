// Writing goods off a lot, and undoing a write-off (PRD FR-12.16, FR-7.8). Both ask for a reason,
// both say exactly what will happen, and neither changes the books: a write-off of goods a seller
// sent is a stock move only. The quantity goes to the backend as typed (a simple sum is allowed,
// FR-7.48); the backend checks it against what is left and answers with the new numbers. A refusal
// stays inside the box and keeps what was typed.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { StockRow } from '../../../../bindings/StockRow';
import type { UiError } from '../../../../bindings/UiError';
import { uiErrorOf } from '../../../../shared/api';
import { isoToDisplayDate } from '../../../../shared/format';
import { Field, QtyInput, ReasonDialog, Select, showToast } from '../../../../shared/ui';
import { useWriteOff, useWriteOffUndo } from '../../hooks/useStockActions';
import { stockSellerName } from '../../stock-columns';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { fieldError } from '../../../../shared/ui/field-error';

/** The fields whose problems the boxes show under the field itself. */
const WRITE_OFF_FIELDS = ['qty'];
const UNDO_FIELDS = ['move_id'];

interface DialogProps {
  /** The lot, or null when the box is closed. */
  row: StockRow | null;
  onClose: () => void;
}

/** "Write off" (PRD FR-12.16): how many نگ, and why. */
export function WriteOffDialog({ row, onClose }: DialogProps) {
  const language = useLanguage();
  const writeOff = useWriteOff();
  const [qty, setQty] = useState('');
  const [error, setError] = useState<UiError | null>(null);
  const close = () => {
    setQty('');
    setError(null);
    onClose();
  };
  return (
    <ReasonDialog
      open={row !== null}
      onOpenChange={(open) => {
        if (!open) close();
      }}
      title={<Localized id="bikri-write-off-title" />}
      description={
        row === null ? null : (
          <Localized
            id="bikri-write-off-text"
            vars={{
              number: String(row.bikri_number),
              seller: stockSellerName(row, language),
              item: displayName(row.item, language),
              balance: String(row.balance),
            }}
          >
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="bikri-write-off-reason" />}
      confirmLabel={<Localized id="bikri-write-off-confirm" />}
      error={error}
      busy={writeOff.isPending}
      extraFields={WRITE_OFF_FIELDS}
      extra={
        <Field label={<Localized id="bikri-write-off-qty" />} error={fieldError(error, 'qty')}>
          {(control) => <QtyInput {...control} value={qty} onChange={setQty} />}
        </Field>
      }
      onConfirm={(reason) => {
        if (row === null) return;
        setError(null);
        writeOff
          .mutateAsync({ lotId: row.lot_id, qty: qty.trim(), reason })
          .then((done) => {
            showToast(
              'success',
              <Localized id="bikri-write-off-done" vars={{ qty: String(done.qty) }}>
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

/** "Undo write-off" (PRD FR-12.16): which write-off, and why. */
export function UndoWriteOffDialog({ row, onClose }: DialogProps) {
  const { l10n } = useLocalization();
  const undo = useWriteOffUndo();
  const [chosen, setChosen] = useState<string | undefined>(undefined);
  const [error, setError] = useState<UiError | null>(null);
  const close = () => {
    setChosen(undefined);
    setError(null);
    onClose();
  };
  // Only one write-off to undo: it is the choice already.
  const writeOffs = row?.write_offs ?? [];
  const moveId = chosen ?? (writeOffs.length === 1 ? String(writeOffs[0]?.move_id) : undefined);
  return (
    <ReasonDialog
      open={row !== null}
      onOpenChange={(open) => {
        if (!open) close();
      }}
      title={<Localized id="bikri-write-off-undo-title" />}
      description={<Localized id="bikri-write-off-undo-text" />}
      reasonLabel={<Localized id="bikri-write-off-reason" />}
      confirmLabel={<Localized id="bikri-write-off-undo-confirm" />}
      error={error}
      busy={undo.isPending}
      extraFields={UNDO_FIELDS}
      extra={
        <Field
          label={<Localized id="bikri-write-off-undo-which" />}
          error={fieldError(error, 'move_id')}
        >
          {(control) => (
            <Select
              {...control}
              value={moveId}
              onChange={setChosen}
              options={writeOffs.map((item) => ({
                value: String(item.move_id),
                label: l10n.getString('bikri-write-off-undo-option', {
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
          .then(() => {
            showToast(
              'success',
              <Localized id="bikri-write-off-undone">
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
