// The columns of the two lists of کل مالیت (PRD FR-9.10, plan 100 B): where the money is (the source), the
// account's code and name with its tag, and the amount. Every figure is the backend's; the lists add up
// nothing. The code comes with the wider layout steps so a list never scrolls sideways (NFR-USE-15).

import type { TotalWorthRow } from '../../bindings/TotalWorthRow';
import type { TotalWorthSource } from '../../bindings/TotalWorthSource';
import type { ListColumn } from '../../shared/grid';
import { nameWithTag } from '../reports';

/** The words of each source (a new source from the backend must be added here to build). */
export const WORTH_SOURCE_IDS: Record<TotalWorthSource, string> = {
  cash: 'total-worth-source-cash',
  buyers_dues: 'total-worth-source-buyers-dues',
  sellers_dues: 'total-worth-source-sellers-dues',
  dead_amount: 'total-worth-source-dead-amount',
  money_lent: 'total-worth-source-money-lent',
  open_advances: 'total-worth-source-open-advances',
  khata_minus: 'total-worth-source-khata-minus',
  garden_minus: 'total-worth-source-garden-minus',
  other_have: 'total-worth-source-other-have',
  sellers_payable: 'total-worth-source-sellers-payable',
  payables: 'total-worth-source-payables',
  buyers_credit: 'total-worth-source-buyers-credit',
  khata_plus: 'total-worth-source-khata-plus',
  garden_plus: 'total-worth-source-garden-plus',
  other_give: 'total-worth-source-other-give',
};

export const TOTAL_WORTH_COLUMNS: readonly ListColumn<TotalWorthRow>[] = [
  {
    field: 'source',
    type: 'text',
    headerId: 'total-worth-col-source',
    grow: true,
    wrap: true,
    value: (row, { text }) => text(WORTH_SOURCE_IDS[row.source]),
  },
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    from: 'normal',
    value: (row) => row.party.code,
  },
  {
    field: 'party',
    type: 'name',
    headerId: 'total-worth-col-party',
    grow: true,
    value: (row, { language }) => nameWithTag(row.party, row.party.tag, language),
  },
  {
    field: 'amount',
    type: 'money',
    headerId: 'total-worth-col-amount',
    value: (row, { money }) => money(row.amount_paisa),
  },
];
