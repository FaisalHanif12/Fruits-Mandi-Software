// The columns of R38, the dead amount (PRD FR-10.10, R38): the person, his code, what kind of
// person, why, the day he was moved, what he owed then, what was recovered since, the corrections
// (edits and cancels of earlier entries since the move, signed), what he still owes and the day of
// the last recovery. Every figure is the backend's. Less used columns come with the wider layout
// steps so the list never scrolls sideways (NFR-USE-15, F22). Money columns are centred (plan 39).

import type { DeadRow } from '../../bindings/DeadRow';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';

/** The name of the person: a one-time person by the name saved on his lines, any other by his account. */
function personName(row: DeadRow, language: 'ur' | 'en'): string {
  if (row.pukhta_person_key === null) return displayName(row, language);
  return displayName(
    { name_ur: row.pukhta_name_ur ?? row.name_ur, name_en: row.pukhta_name_en ?? row.name_en },
    language,
  );
}

export const DEAD_COLUMNS: readonly ListColumn<DeadRow>[] = [
  {
    field: 'party',
    type: 'name',
    headerId: 'dead-col-party',
    grow: true,
    value: (row, { language }) => personName(row, language),
  },
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    from: 'normal',
    value: (row) => row.code,
  },
  {
    field: 'type',
    type: 'status',
    headerId: 'dead-col-type',
    from: 'wide',
    value: (row, { text }) => text(`dead-type-${row.party_type}`),
  },
  {
    field: 'reason',
    // "Left the business" and "کاروبار چھوڑ دیا" are longer than a status column.
    type: 'text',
    headerId: 'dead-col-reason',
    from: 'wide',
    value: (row, { text }) => text(`dead-reason-${row.reason}`),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'dead-col-date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'owed_then',
    type: 'money',
    headerId: 'dead-col-owed-then',
    value: (row, { money }) => money(row.owed_then_paisa),
  },
  {
    field: 'recovered',
    type: 'money',
    headerId: 'dead-col-recovered',
    value: (row, { money }) => money(row.recovered_paisa),
  },
  {
    field: 'corrections',
    type: 'money',
    headerId: 'dead-col-corrections',
    from: 'wide',
    value: (row, { signedMoney }) => signedMoney(row.corrections_paisa),
  },
  {
    field: 'still_owed',
    type: 'money',
    headerId: 'dead-col-still-owed',
    value: (row, { money }) => money(row.still_owed_paisa),
  },
  {
    field: 'last_recovery',
    type: 'date',
    headerId: 'dead-col-last-recovery',
    from: 'wide',
    value: (row) =>
      row.last_recovery_date === null ? '' : isoToDisplayDate(row.last_recovery_date),
  },
  {
    field: 'status',
    // The words are longer than a status column ("پوری وصول ہو گئی"), so it is as wide as a text column.
    type: 'text',
    headerId: 'dead-col-status',
    from: 'normal',
    value: (row, { text }) => text(`dead-status-${row.status}`),
  },
];
