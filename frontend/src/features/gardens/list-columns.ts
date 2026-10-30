// The columns of the lists of the gardens module (PRD FR-12.37, FR-19.10, FR-19.11): the gardens,
// and the history of one garden with its running balance. Amounts and counts are the backend's and
// shown with `money` and `String`, never worked out. The lists sit under the main menu only, so the
// room is the content area minus the 64 px rail; less used columns come with the wider layout steps
// and a list never scrolls sideways (NFR-USE-15, Code Standards F22). A cancelled entry says so in
// words. Names go through `displayName` (F38).

import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import type { GardenHistoryKind, GardenHistoryRow, GardenRow } from './types';

type Say = (id: string, vars?: Record<string, string>) => string;

/** The number of a goods arrived entry in its own series: "GA-3" or "مآ-3" (letters from Fluent). */
export function arrivalNumberText(number: number, text: Say): string {
  return text('gardens-arrival-number', { number: String(number) });
}

/** The word for what a history row is (the type list of the garden screen, FR-7.67). */
export function historyKindText(kind: GardenHistoryKind, text: Say): string {
  return text(`gardens-kind-${kind.replaceAll('_', '-')}`);
}

/** The detail of a history row: the reason, the shop, or the بکری it comes from. */
export function historyDetailText(row: GardenHistoryRow, text: Say): string {
  switch (row.kind) {
    case 'installment':
    case 'cost':
      // The reason is free text, shown as typed (PRD FR-7.25).
      return row.reason ?? '';
    case 'other_sale':
      return text('gardens-history-detail-sale-elsewhere', {
        shop: row.shop_name ?? '',
        qty: String(row.qty ?? 0),
      });
    case 'sale':
      return row.bikri_number === null
        ? ''
        : text('gardens-history-detail-bikri', { number: String(row.bikri_number) });
    case 'fruit':
    case 'close':
      return '';
  }
}

export const GARDEN_COLUMNS: readonly ListColumn<GardenRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'gardens-col-number',
    sortKey: 'code',
    // The code of a garden, type 008 (plan 34). Its account has the same code.
    value: (row) => row.code,
  },
  {
    field: 'name',
    type: 'name',
    headerId: 'gardens-col-name',
    sortKey: (language) => (language === 'ur' ? 'name_ur' : 'name_en'),
    grow: true,
    value: (row, { language }) => displayName(row, language),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'gardens-col-date',
    sortKey: 'entry_date',
    // The commission takes the room of the date at the narrowest window (plan 36); the date is on
    // the garden's own screen.
    from: 'normal',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'status',
    type: 'text',
    headerId: 'gardens-col-status',
    sortKey: 'status',
    value: (row, { text }) => {
      if (row.status === 'open') return text('gardens-status-open');
      return row.closed_date === null
        ? text('gardens-status-closed')
        : text('gardens-status-closed-on', { date: isoToDisplayDate(row.closed_date) });
    },
  },
  {
    field: 'balance',
    type: 'money',
    headerId: 'gardens-col-balance',
    sortKey: 'balance_paisa',
    value: (row, { money }) => money(row.balance_paisa),
  },
  // The shop's commission on the garden's goods sold in our shop (PRD FR-12.37): the backend's figure.
  {
    field: 'commission',
    type: 'money',
    headerId: 'glossary-commission',
    sortKey: 'commission_paisa',
    value: (row, { money }) => money(row.commission_paisa),
  },
  {
    field: 'rent_due',
    type: 'money',
    headerId: 'gardens-col-rent-due',
    sortKey: 'rent_due_paisa',
    from: 'normal',
    value: (row, { money }) => money(row.rent_due_paisa),
  },
  {
    field: 'rent',
    type: 'money',
    headerId: 'gardens-col-rent',
    sortKey: 'rent_paisa',
    from: 'wide',
    value: (row, { money }) => money(row.rent_paisa),
  },
  {
    field: 'paid',
    type: 'money',
    headerId: 'gardens-col-paid',
    sortKey: 'paid_paisa',
    from: 'wide',
    value: (row, { money }) => money(row.paid_paisa),
  },
  {
    field: 'money_in',
    type: 'money',
    headerId: 'gardens-col-money-in',
    sortKey: 'money_in_paisa',
    from: 'wide',
    value: (row, { money }) => money(row.money_in_paisa),
  },
];

export const HISTORY_COLUMNS: readonly ListColumn<GardenHistoryRow>[] = [
  {
    field: 'date',
    type: 'date',
    headerId: 'gardens-col-date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'entry',
    type: 'text',
    headerId: 'gardens-col-entry',
    grow: true,
    value: (row, { text }) => {
      const kind = historyKindText(row.kind, text);
      const detail = historyDetailText(row, text);
      return detail === '' ? kind : text('gardens-history-entry', { kind, detail });
    },
  },
  {
    field: 'effect',
    type: 'money',
    headerId: 'gardens-col-effect',
    value: (row, { money }) => money(row.effect_paisa),
  },
  {
    field: 'balance',
    type: 'money',
    headerId: 'gardens-col-running-balance',
    value: (row, { money }) => money(row.running_balance_paisa),
  },
  {
    field: 'state',
    type: 'text',
    headerId: 'gardens-col-state',
    from: 'normal',
    value: (row, { text }) => (row.cancelled ? text('gardens-entry-cancelled') : ''),
  },
];
