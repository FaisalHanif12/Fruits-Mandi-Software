// The columns of the list of vouchers under the form (PRD 15.1): the number in the series of its
// type, the day, who it is with (the one-time person's name for a پختہ کھاتہ account), the amount,
// how the money moved and whether it stands or was cancelled. Amounts are the stored sums the
// backend sends, shown with `money` and never worked out. Less used columns come with the wider
// layout steps, so the list never scrolls sideways (NFR-USE-15, Code Standards F22). The room is
// the content area minus the 200 px side menu: 692 px at the narrowest compact window.

import type { VoucherRow } from '../../bindings/VoucherRow';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';

/**
 * The number of a voucher in its series: "R-12" or "و-12". Every kind has its own letters in
 * Fluent (`vouchers-number-<kind>`, the kind written with hyphens), also the kinds of the روکڑ area.
 */
export function voucherNumberText(
  row: Pick<VoucherRow, 'voucher_type' | 'number'>,
  text: (id: string, vars?: Record<string, string>) => string,
): string {
  return text(`vouchers-number-${row.voucher_type.replaceAll('_', '-')}`, {
    number: String(row.number),
  });
}

export const VOUCHER_COLUMNS: readonly ListColumn<VoucherRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'vouchers-col-number',
    sortKey: 'number',
    value: (row, { text }) => voucherNumberText(row, text),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'vouchers-col-date',
    sortKey: 'entry_date',
    from: 'normal',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'account',
    type: 'name',
    headerId: 'vouchers-col-account',
    sortKey: 'account',
    grow: true,
    value: (row, { language }) =>
      row.person !== null
        ? displayName(row.person, language)
        : row.account === null
          ? ''
          : displayName(row.account, language),
  },
  {
    field: 'amount',
    type: 'money',
    headerId: 'vouchers-col-amount',
    sortKey: 'amount_paisa',
    value: (row, { money }) => money(row.amount_paisa),
  },
  {
    field: 'status',
    type: 'status',
    headerId: 'vouchers-col-state',
    sortKey: 'status',
    value: (row, { text }) =>
      text(row.status === 'cancelled' ? 'vouchers-state-cancelled' : 'vouchers-state-saved'),
  },
];
