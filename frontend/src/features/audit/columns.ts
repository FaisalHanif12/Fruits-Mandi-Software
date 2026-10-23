// The columns of the audit list and of the fields of one entry (PRD FR-21.3, NFR-USE-15, Code
// Standards F18, F22). A list never scrolls sideways, so the less used columns come with the wider
// layout steps. Everything shown is what the backend sent; the screen works out nothing.

import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { actionWord, entityWord, valueText } from './words';
import type { AuditChange, AuditRow } from './types';

/** The time of an entry in full: the Pakistan day and time the backend worked out (the detail). */
export function whenText(row: AuditRow): string {
  return `${isoToDisplayDate(row.date)} ${row.time}`;
}

/** The same without the seconds, so it fits the width of a date and time column in the list. */
export function whenListText(row: AuditRow): string {
  return `${isoToDisplayDate(row.date)} ${row.time.slice(0, 5)}`;
}

/** How many fields changed; a trailing plus says the backend listed only the first ones. */
export function fieldsText(row: AuditRow): string {
  return `${String(row.changes.length)}${row.changes_truncated ? '+' : ''}`;
}

export const AUDIT_COLUMNS: readonly ListColumn<AuditRow>[] = [
  {
    field: 'when',
    type: 'datetime',
    headerId: 'audit-col-when',
    value: (row) => whenListText(row),
  },
  {
    field: 'user',
    type: 'text',
    headerId: 'audit-col-user',
    grow: true,
    value: (row, context) => row.user?.display_name ?? context.text('audit-system-user'),
  },
  {
    field: 'action',
    type: 'choice',
    headerId: 'audit-col-action',
    value: (row, context) => actionWord(row.action, context.text),
  },
  {
    field: 'entity',
    type: 'text',
    headerId: 'audit-col-entity',
    grow: true,
    value: (row, context) => entityWord(row.entity, context.text),
  },
  {
    field: 'record',
    type: 'number',
    headerId: 'audit-col-record',
    from: 'normal',
    value: (row) => (row.entity_id === null ? '' : String(row.entity_id)),
  },
  {
    field: 'fields',
    type: 'count',
    headerId: 'audit-col-fields',
    value: (row) => fieldsText(row),
  },
  {
    field: 'reason',
    type: 'text',
    headerId: 'audit-col-reason',
    from: 'wide',
    grow: true,
    wrap: true,
    value: (row) => row.reason ?? '',
  },
];

/** One changed field in the detail grid: its place in the entry is its key. */
export interface ChangeRow {
  key: string;
  change: AuditChange;
}

export const CHANGE_COLUMNS: readonly ListColumn<ChangeRow>[] = [
  {
    field: 'path',
    type: 'text',
    headerId: 'audit-detail-field',
    grow: true,
    wrap: true,
    value: (item) => item.change.path,
  },
  {
    field: 'old',
    type: 'text',
    headerId: 'audit-detail-old',
    grow: true,
    wrap: true,
    value: (item, context) => valueText(item.change.old, context),
  },
  {
    field: 'new',
    type: 'text',
    headerId: 'audit-detail-new',
    grow: true,
    wrap: true,
    value: (item, context) => valueText(item.change.new, context),
  },
];
