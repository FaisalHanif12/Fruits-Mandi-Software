// Cancelling a saved voucher (PRD FR-15.3, FR-7.8): a box that says exactly what will happen, asks
// for a reason, and cancels by reversal. Nothing is deleted: the posting is reversed on its own day,
// the matching it made is switched off and run again, and the voucher stays in the list marked
// cancelled with its reason.
//
// If taking the voucher back would take cash in hand below zero (a receipt taken back lowers it,
// PRD FR-6.7), the backend refuses the first try. The box then shows the cash after the cancel in
// its own words and the next press of the button confirms it, so the user always sees it first.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { ErrorMessage, Num, ReasonDialog, showToast } from '../../../shared/ui';
import { useCancelVoucher } from '../hooks';
import { screenTypeOf } from '../screen-type';
import { voucherNumberText } from '../list-columns';
import type { VoucherCancelled, VoucherRow } from '../types';

interface CancelVoucherDialogProps {
  /** The voucher to cancel, or null when the box is closed. */
  row: VoucherRow | null;
  onClose: () => void;
  /** The voucher was cancelled. */
  onCancelled: (cancelled: VoucherCancelled) => void;
}

const CASH_CODE = 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED';

export function CancelVoucherDialog({ row, onClose, onCancelled }: CancelVoucherDialogProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const cancel = useCancelVoucher();
  const [error, setError] = useState<UiError | null>(null);
  // The backend said the cancel takes cash below zero: the next press confirms it.
  const [cashConfirmed, setCashConfirmed] = useState(false);

  const type = row === null ? null : screenTypeOf(row.voucher_type);
  const cashItem = error?.errors.find((item) => item.code === CASH_CODE);
  const shownError =
    error === null
      ? null
      : { ...error, errors: error.errors.filter((item) => item.code !== CASH_CODE) };

  const confirm = (reason: string) => {
    if (row === null || type === null) return;
    setError(null);
    cancel
      .mutateAsync({ id: row.id, version: row.version, reason, confirm: cashConfirmed })
      .then((cancelled) => {
        showToast(
          'success',
          l10n.getString('vouchers-cancelled', {
            kind: type,
            number: voucherNumberText(row, (id, vars) => l10n.getString(id, vars)),
          }),
        );
        setCashConfirmed(false);
        onCancelled(cancelled);
      })
      .catch((failure: unknown) => {
        const refused = uiErrorOf(failure);
        setCashConfirmed(refused.errors.some((item) => item.code === CASH_CODE));
        setError(refused);
      });
  };

  return (
    <ReasonDialog
      open={row !== null}
      onOpenChange={(open) => {
        if (open) return;
        setError(null);
        setCashConfirmed(false);
        onClose();
      }}
      title={<Localized id="vouchers-cancel-title" />}
      description={
        row === null || type === null ? null : (
          <Localized
            id={`vouchers-cancel-text-${type}`}
            vars={{
              number: voucherNumberText(row, (id, vars) => l10n.getString(id, vars)),
              amount: formatPaisa(row.amount_paisa, { grouping }),
              name:
                row.person !== null
                  ? displayName(row.person, language)
                  : row.account === null
                    ? ''
                    : displayName(row.account, language),
            }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="vouchers-cancel-reason" />}
      confirmLabel={
        <Localized
          id={cashConfirmed ? 'vouchers-cancel-confirm-cash' : 'vouchers-cancel-confirm'}
        />
      }
      error={shownError}
      busy={cancel.isPending}
      note={
        cashItem === undefined ? undefined : (
          <ErrorMessage item={cashItem}>
            <span />
          </ErrorMessage>
        )
      }
      onConfirm={confirm}
    />
  );
}
