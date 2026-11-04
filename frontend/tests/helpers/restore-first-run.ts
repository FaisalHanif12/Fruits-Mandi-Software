// Test helper for the restore pages of a new computer (plan 56). The answers and requests are the
// backend's own: it wrote them in its own test (backend/app/tests/backup_first_run_commands.rs, into
// backend/crates/backup/tests/fixtures) and they are copied here, never edited. The UI tests start from
// them and change only what they are about (a second copy, bigger totals). A refusal is a rejected
// promise (`CommandError`), the way `invoke` fails. The command wrappers are mocked in the test file
// with vi.mock.

import { vi } from 'vitest';
import type { BackupRow } from '../../src/bindings/BackupRow';
import type { BackupTotals } from '../../src/bindings/BackupTotals';
import type { FirstRunCopies } from '../../src/bindings/FirstRunCopies';
import type { FirstRunListInput } from '../../src/bindings/FirstRunListInput';
import type { FirstRunPreview } from '../../src/bindings/FirstRunPreview';
import type { FirstRunPreviewInput } from '../../src/bindings/FirstRunPreviewInput';
import type { FirstRunRestoreDone } from '../../src/bindings/FirstRunRestoreDone';
import type { FirstRunRestoreInput } from '../../src/bindings/FirstRunRestoreInput';
import * as restoreApi from '../../src/features/restore/api';
import alreadySetUp from '../fixtures/backup_first_run_already_set_up.json';
import list from '../fixtures/backup_first_run_list.json';
import listOffline from '../fixtures/backup_first_run_list_offline.json';
import listRefused from '../fixtures/backup_first_run_list_refused.json';
import listRequest from '../fixtures/backup_first_run_list_request.json';
import preview from '../fixtures/backup_first_run_preview.json';
import previewRequest from '../fixtures/backup_first_run_preview_request.json';
import previewWrongKey from '../fixtures/backup_first_run_preview_wrong_key.json';
import previewWrongKeyRequest from '../fixtures/backup_first_run_preview_wrong_key_request.json';
import restore from '../fixtures/backup_first_run_restore.json';
import restoreRequest from '../fixtures/backup_first_run_restore_request.json';
import { paisaForTest } from './paisa';
import { CommandError } from '../../src/shared/api';
import { refusal } from './ui-error';

// A reader takes `unknown`, because the money in a binding is a branded type a plain number is not.
function readCopies(value: unknown): FirstRunCopies {
  return value as FirstRunCopies;
}
function readListInput(value: unknown): FirstRunListInput {
  return value as FirstRunListInput;
}
function readPreview(value: unknown): FirstRunPreview {
  return value as FirstRunPreview;
}
function readPreviewInput(value: unknown): FirstRunPreviewInput {
  return value as FirstRunPreviewInput;
}
function readRestoreDone(value: unknown): FirstRunRestoreDone {
  return value as FirstRunRestoreDone;
}
function readRestoreInput(value: unknown): FirstRunRestoreInput {
  return value as FirstRunRestoreInput;
}

export interface WireError {
  errors: { code: string; field: string | null; values: Record<string, string | number> }[];
  ok: false;
  ref: string;
}

/** A backend refusal from one of its fixtures, as a rejected promise. */
export function refusedWith(wire: WireError): CommandError {
  return new CommandError(refusal(wire.errors));
}

// The backend's own answers.
export const LIST_REQUEST = readListInput(listRequest);
export const LIST = readCopies(list);
export const LIST_REFUSED = listRefused;
export const LIST_OFFLINE = listOffline;
export const PREVIEW_REQUEST = readPreviewInput(previewRequest);
export const PREVIEW = readPreview(preview);
export const PREVIEW_WRONG_KEY = previewWrongKey;
export const PREVIEW_WRONG_KEY_REQUEST = readPreviewInput(previewWrongKeyRequest);
export const RESTORE_REQUEST = readRestoreInput(restoreRequest);
export const RESTORE_DONE = readRestoreDone(restore);
export const ALREADY_SET_UP = alreadySetUp;

// What the UI tests type, and the answers they need around it.
export const CLOUD_TYPED = {
  endpoint: 'https://shop.r2.cloudflarestorage.com',
  bucket: 'mandi-bucket',
  region: 'auto',
  prefix: 'mandi',
  key_id: 'AKIAEXAMPLEKEYID',
  secret: 'secret-access-key-value',
};

/** The key from the printed sheet (the format of backup_key_setup). */
export const BACKUP_KEY = 'ABCD-EFGH-JKLM-NPQR-STUV-WXYZ';

const TEMPLATE: BackupRow = LIST.rows[0] ?? {
  kind: 'cloud',
  name: 'mandi-20261010T090000000Z.mbk',
  created_at: '2026-10-10T09:00:00.000Z',
  size_bytes: 0,
  encrypted: true,
  status: 'good',
};

export const NEWEST: BackupRow = {
  ...TEMPLATE,
  kind: 'cloud',
  name: 'mandi-20280314T160000000Z.mbk',
  created_at: '2028-03-14T16:00:00.000Z',
  size_bytes: 3_400_000,
};
export const OLDER: BackupRow = {
  ...NEWEST,
  name: 'mandi-20280313T160000000Z.mbk',
  created_at: '2028-03-13T16:00:00.000Z',
  size_bytes: 3_300_000,
};

export const COPIES: FirstRunCopies = { rows: [NEWEST, OLDER], total_count: 2 };

export const TOTALS: BackupTotals = {
  accounts: 2413,
  bikri_entries: 1204,
  vouchers: 3100,
  journal_entries: 9050,
  audit_rows: 20_000,
  last_entry_date: '2028-03-14',
  debit_paisa: paisaForTest(4_500_000_000),
  credit_paisa: paisaForTest(4_500_000_000),
  cash_paisa: paisaForTest(5_848_400),
};

export const GOOD_PREVIEW: FirstRunPreview = {
  ...PREVIEW,
  copy: { kind: 'cloud', name: NEWEST.name },
  created_at: NEWEST.created_at,
  size_bytes: NEWEST.size_bytes,
  totals: TOTALS,
};

export const DONE: FirstRunRestoreDone = {
  ...RESTORE_DONE,
  restored_from: NEWEST.created_at,
  totals: TOTALS,
  assistant_was_on: true,
  cloud_needs_keys: false,
};

/** Sets every restore command to answer as a healthy backend would. */
export function fakeRestore() {
  // A test that queued one-time answers and failed half way must not hand them to the next test.
  vi.mocked(restoreApi.backupFirstRunList).mockReset();
  vi.mocked(restoreApi.backupFirstRunPreview).mockReset();
  vi.mocked(restoreApi.backupFirstRunRestore).mockReset();
  vi.mocked(restoreApi.backupFirstRunList).mockResolvedValue(COPIES);
  vi.mocked(restoreApi.backupFirstRunPreview).mockResolvedValue(GOOD_PREVIEW);
  vi.mocked(restoreApi.backupFirstRunRestore).mockResolvedValue(DONE);
}
