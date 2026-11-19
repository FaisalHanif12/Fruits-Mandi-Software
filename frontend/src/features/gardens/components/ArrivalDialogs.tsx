// The boxes of a goods arrived entry (PRD FR-12.40, FR-7.8): cancel it with a reason, write goods
// off with a quantity and a reason, undo a write-off. Nothing is posted by any of them: goods
// arrived carry no money. The quantity goes to the backend as typed (a simple sum is allowed,
// FR-7.48); the backend checks it against what is left. A refusal stays inside the box and keeps
// what was typed.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Field, QtyInput, ReasonDialog, Select, fieldError, showToast } from '../../../shared/ui';
import { useCancelGardenArrival, useWriteOffArrival, useWriteOffArrivalUndo } from '../hooks';
import { arrivalNumberText } from '../list-columns';
import type { ArrivalRow } from '../types';

interface DialogProps {
  /** The entry, or null when the box is closed. */
  row: ArrivalRow | null;
  onClose: () => void;
}

/** "Cancel the entry" (only while nothing of it is sold or written off): a reason. */
export function CancelArrivalDialog({ row, onClose }: DialogProps) {
  const { l10n } = useLocalization();
  const cancel = useCancelGardenArrival();
  const [error, setError] = useState<UiError | null>(null);
  const close = () => {
    setError(null);
    onClose();
  };
  return (
    <ReasonDialog
      open={row !== null}
      onOpenChange={(open) => {
        if (!open) close();
      }}
      title={<Localized id="gardens-arrival-cancel-title" />}
      description={
        row === null ? null : (
          <Localized
            id="gardens-arrival-cancel-text"
            vars={{ number: arrivalNumberText(row.number, (id, vars) => l10n.getString(id, vars)) }}
          >
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="gardens-arrival-reason" />}
      confirmLabel={<Localized id="gardens-arrival-cancel-confirm" />}
      confirmVariant="danger"
      error={error}
      busy={cancel.isPending}
      onConfirm={(reason) => {
        if (row === null) return;
        setError(null);
        cancel
          .mutateAsync({ id: row.id, version: row.version, reason })
          .then((done) => {
            showToast(
              'success',
              l10n.getString('gardens-arrival-cancelled-toast', {
                number: arrivalNumberText(done.number, (id, vars) => l10n.getString(id, vars)),
              }),
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

/** "Write off" (nothing is posted): how many نگ, and why. */
export function WriteOffArrivalDialog({ row, onClose }: DialogProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const writeOff = useWriteOffArrival();
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
      title={<Localized id="gardens-arrival-write-off-title" />}
      description={
        row === null ? null : (
          <Localized
            id="gardens-arrival-write-off-text"
            vars={{
              number: arrivalNumberText(row.number, (id, vars) => l10n.getString(id, vars)),
              item: displayName(row.item, language),
              packing: displayName(row.packing, language),
              qty_left: String(row.qty_left),
            }}
          >
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="gardens-arrival-reason" />}
      confirmLabel={<Localized id="gardens-arrival-write-off-confirm" />}
      error={error}
      busy={writeOff.isPending}
      extraFields={['qty']}
      extra={
        <Field
          label={<Localized id="gardens-arrival-write-off-qty" />}
          error={fieldError(error, 'qty')}
        >
          {(control) => <QtyInput {...control} value={qty} onChange={setQty} />}
        </Field>
      }
      onConfirm={(reason) => {
        if (row === null) return;
        setError(null);
        writeOff
          .mutateAsync({ arrival_id: row.id, qty: qty.trim(), reason })
          .then((done) => {
            showToast(
              'success',
              l10n.getString('gardens-arrival-write-off-done', { qty: String(done.qty) }),
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

/** "Undo write-off": which write-off, and why. */
export function UndoWriteOffArrivalDialog({ row, onClose }: DialogProps) {
  const { l10n } = useLocalization();
  const undo = useWriteOffArrivalUndo();
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
      title={<Localized id="gardens-arrival-write-off-undo-title" />}
      description={<Localized id="gardens-arrival-write-off-undo-text" />}
      reasonLabel={<Localized id="gardens-arrival-reason" />}
      confirmLabel={<Localized id="gardens-arrival-write-off-undo-confirm" />}
      error={error}
      busy={undo.isPending}
      extraFields={['move_id']}
      extra={
        <Field
          label={<Localized id="gardens-arrival-write-off-pick" />}
          error={fieldError(error, 'move_id')}
        >
          {(control) => (
            <Select
              {...control}
              value={moveId}
              onChange={setChosen}
              options={writeOffs.map((item) => ({
                value: String(item.move_id),
                label: l10n.getString('gardens-arrival-write-off-option', {
                  qty: String(item.qty),
                  reason: item.reason ?? '',
                }),
              }))}
            />
          )}
        </Field>
      }
      onConfirm={(reason) => {
        if (moveId === undefined) return;
        setError(null);
        undo
          .mutateAsync({ move_id: Number(moveId), reason })
          .then(() => {
            showToast('success', <Localized id="gardens-arrival-write-off-undone" />);
            close();
          })
          .catch((failure: unknown) => {
            setError(uiErrorOf(failure));
          });
      }}
    />
  );
}
