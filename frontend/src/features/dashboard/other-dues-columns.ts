// The columns of the lists behind the tiles that count what people owe or hold (PRD FR-9.1, FR-9.7): the people
// who borrowed and still owe (ادھار رقم باقی), the sellers with an open advance (پیشگی باقی) and the dead
// amount section of کل اگراہی. Every figure is the backend's; the lists add up nothing. Less used columns come
// with the wider layout steps so a list never scrolls sideways (NFR-USE-15, F22). Money and dates are centred
// (NFR-USE-19).

import type { AdvancesReportRow } from '../../bindings/AdvancesReportRow';
import type { DeadRow } from '../../bindings/DeadRow';
import type { LentOpenRow } from '../../bindings/LentOpenRow';
import { isPositivePaisa, isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { nameWithTag } from '../reports';

/** The kind of person as words: staff, garden owner, market trip person, other. */
function personTypeText(type: LentOpenRow['person_type'], text: (id: string) => string): string {
  return text(`cashbook-person-type-${type.replaceAll('_', '-')}`);
}

export const LENT_OPEN_COLUMNS: readonly ListColumn<LentOpenRow>[] = [
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    from: 'normal',
    value: (row) => row.party.code,
  },
  {
    field: 'person',
    type: 'name',
    headerId: 'lent-open-col-person',
    sortKey: 'party',
    grow: true,
    value: (row, { language }) => nameWithTag(row.party, row.party.tag, language),
  },
  {
    field: 'type',
    type: 'status',
    headerId: 'lent-open-col-type',
    from: 'wide',
    value: (row, { text }) => personTypeText(row.person_type, text),
  },
  {
    field: 'open',
    type: 'money',
    headerId: 'lent-open-col-open',
    sortKey: 'open',
    value: (row, { money }) => money(row.open_paisa),
  },
  {
    field: 'oldest',
    type: 'date',
    headerId: 'lent-open-col-oldest',
    sortKey: 'oldest_open_date',
    from: 'normal',
    value: (row) => (row.oldest_open_date === null ? '' : isoToDisplayDate(row.oldest_open_date)),
  },
  {
    field: 'overdue',
    type: 'money',
    headerId: 'lent-open-col-overdue',
    from: 'wide',
    value: (row, { money }) => money(row.overdue_paisa),
    tone: (row) => (isPositivePaisa(row.overdue_paisa) ? 'due' : undefined),
  },
];

export const ADVANCES_OPEN_COLUMNS: readonly ListColumn<AdvancesReportRow>[] = [
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    from: 'normal',
    value: (row) => row.account.code,
  },
  {
    field: 'seller',
    type: 'name',
    headerId: 'seller-dues-col-seller',
    grow: true,
    value: (row, { language }) => displayName(row.account, language),
  },
  {
    field: 'given',
    type: 'money',
    headerId: 'advances-open-col-given',
    from: 'normal',
    value: (row, { money }) => money(row.given_paisa),
  },
  {
    field: 'adjusted',
    type: 'money',
    headerId: 'advances-open-col-adjusted',
    from: 'normal',
    value: (row, { money }) => money(row.adjusted_paisa),
  },
  {
    field: 'returned',
    type: 'money',
    headerId: 'advances-open-col-returned',
    from: 'wide',
    value: (row, { money }) => money(row.returned_paisa),
  },
  {
    field: 'open',
    type: 'money',
    headerId: 'advances-open-col-open',
    value: (row, { money }) => money(row.open_paisa),
  },
  {
    field: 'last_date',
    type: 'date',
    headerId: 'advances-open-col-last',
    from: 'normal',
    value: (row) => (row.last_date === null ? '' : isoToDisplayDate(row.last_date)),
  },
];

/** The dead amount section of کل اگراہی: who, what kind, the day he was moved and what he still owes. */
export const DEAD_OWED_COLUMNS: readonly ListColumn<DeadRow>[] = [
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    from: 'normal',
    value: (row) => row.code,
  },
  {
    field: 'party',
    type: 'name',
    headerId: 'dead-col-party',
    grow: true,
    value: (row, { language }) => displayName(row, language),
  },
  {
    field: 'type',
    type: 'status',
    headerId: 'dead-col-type',
    from: 'wide',
    value: (row, { text }) => text(`dead-type-${row.party_type}`),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'dead-col-date',
    from: 'normal',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'still_owed',
    type: 'money',
    headerId: 'dead-col-still-owed',
    value: (row, { money }) => money(row.still_owed_paisa),
  },
];
