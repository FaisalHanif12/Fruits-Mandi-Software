// The Edit and Cancel buttons under a list of saved vouchers of the روکڑ area (plan 44, PRD FR-7.70,
// FR-21.4): they work on the row that has focus. Edit opens the entry on its own screen; when its
// day is closed it first asks to reopen the day (the same box as everywhere). Cancel asks for the
// reason. What the buttons may do comes from the flags on the row; the backend decides.

import { useState } from 'react';
import {
  NO_ENTRY_FLAGS,
  useFixDayFlow,
  useOpenVoucher,
  useVoucherHref,
} from '../../../shared/edit';
import { EntryActions } from '../../../shared/ui';
import { CancelVoucherDialog } from '../../vouchers/components/CancelVoucherDialog';
import type { VoucherRow } from '../types';

export function VoucherRowActions({ row }: { row: VoucherRow | null }) {
  const open = useOpenVoucher();
  const hrefOf = useVoucherHref();
  const fix = useFixDayFlow();
  const [cancelRow, setCancelRow] = useState<VoucherRow | null>(null);
  return (
    <>
      <EntryActions
        flags={row ?? NO_ENTRY_FLAGS}
        busy={fix.busy}
        onEdit={() => {
          if (row === null) return;
          if (row.needs_reopen) {
            fix.start({
              date: row.entry_date,
              ownerHref: hrefOf(row),
              then: () => {
                open(row);
              },
            });
            return;
          }
          open(row);
        }}
        onCancel={() => {
          if (row !== null) setCancelRow(row);
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
        }}
      />
    </>
  );
}
