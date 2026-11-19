// The columns of the list of backups (PRD FR-20.8): when, where, how big, whether it needs the key,
// and the result of its check. Times are shown in the computer's own time, which is Pakistan time
// on the client's machine (Code Standards G9). The status is written in words, because colour is
// never the only signal (NFR-USE-12). Less used columns come with the wider layout steps, so the
// list never scrolls sideways (NFR-USE-15, Code Standards F22).

import { utcToLocalDisplay } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { kindId, sizeText } from './format';
import type { BackupRow } from './types';

/** The ID of a row: a file name can be the same in two places (USB and cloud). */
export function backupRowId(row: Pick<BackupRow, 'kind' | 'name'>): string {
  return `${row.kind}:${row.name}`;
}

export const BACKUP_COLUMNS: readonly ListColumn<BackupRow>[] = [
  {
    field: 'created',
    type: 'datetime',
    headerId: 'backup-col-created',
    value: (row) => utcToLocalDisplay(row.created_at),
  },
  {
    field: 'kind',
    type: 'text',
    headerId: 'backup-col-place',
    value: (row, { text }) => text(kindId(row.kind)),
  },
  {
    field: 'size',
    type: 'number',
    headerId: 'backup-col-size',
    value: (row, { text }) => sizeText(row.size_bytes, text),
  },
  {
    field: 'status',
    type: 'choice',
    headerId: 'backup-col-status',
    value: (row, { text }) => text(`backup-status-${row.status}`),
  },
  {
    field: 'locked',
    type: 'text',
    headerId: 'backup-col-locked',
    from: 'normal',
    value: (row, { text }) => text(row.encrypted ? 'backup-locked-yes' : 'backup-locked-no'),
  },
  {
    field: 'name',
    type: 'code',
    headerId: 'backup-col-name',
    from: 'wide',
    grow: true,
    value: (row) => row.name,
  },
];
