// Typed wrappers around the backup commands (plan 18 contract, PRD 20). Components never call
// invoke() (Code Standards F4). This is the only file that knows the wire shapes. Every command
// needs the signed-in Owner. A failure of one place of a run is part of the answer, not an error.

import { callCommand } from '../../shared/api';
import type {
  BackupList,
  BackupListInput,
  BackupNowInput,
  BackupSettingsInput,
  BackupSettingsView,
  BackupStatus,
  KeyConfirmInput,
  KeyCreated,
  RestoreDone,
  RestoreInput,
  RestorePreview,
  RestorePreviewInput,
  RunReport,
  TestRestoreInput,
  TestRestoreResult,
} from './types';

/**
 * The state of the backups for the status bar and the backup screen (PRD FR-20.6, NFR-BAK-10).
 * Reads state and settings only: no network.
 */
export function backupStatus(): Promise<BackupStatus> {
  return callCommand<BackupStatus>('backup_status');
}

/** One page of the copies, newest first, with the number of all rows (Code Standards B31). */
export function backupList(input: BackupListInput): Promise<BackupList> {
  return callCommand<BackupList>('backup_list', { input });
}

/**
 * Makes a copy now (PRD FR-20.8). Runs on a worker thread and answers when it is done, so the
 * screen shows a busy state. One result for each place; a cloud copy with no internet is queued.
 */
export function backupNow(input: BackupNowInput): Promise<RunReport> {
  return callCommand<RunReport>('backup_now', { input });
}

/** What a restore would do: the date, the totals of the copy and of now, what would be lost. Changes nothing. */
export function backupRestorePreview(input: RestorePreviewInput): Promise<RestorePreview> {
  return callCommand<RestorePreview>('backup_restore_preview', { input });
}

/**
 * Restores a copy (PRD FR-20.9). The backend makes a safety copy first. `confirmed` must be true.
 * After it the Owner is signed out, so the screen goes to the sign-in.
 */
export function backupRestore(input: RestoreInput): Promise<RestoreDone> {
  return callCommand<RestoreDone>('backup_restore', { input });
}

/** Restores a copy into a separate place and compares the totals (NFR-BAK-11). The live data is not touched. */
export function backupTestRestore(input: TestRestoreInput): Promise<TestRestoreResult> {
  return callCommand<TestRestoreResult>('backup_test_restore', { input });
}

export function backupSettingsGet(): Promise<BackupSettingsView> {
  return callCommand<BackupSettingsView>('backup_settings_get');
}

/** Saves the settings. A cloud key left null stays; an empty text removes it. Keys are never returned. */
export function backupSettingsSet(input: BackupSettingsInput): Promise<BackupSettingsView> {
  return callCommand<BackupSettingsView>('backup_settings_set', { input });
}

/** Makes the backup key. It is shown once (PRD FR-20.10). A key never confirmed is replaced by a new one. */
export function backupKeySetup(): Promise<KeyCreated> {
  return callCommand<KeyCreated>('backup_key_setup');
}

/** Compares the key typed from the printed sheet with the stored one (NFR-BAK-7). */
export function backupKeyConfirm(input: KeyConfirmInput): Promise<null> {
  return callCommand<null>('backup_key_confirm', { input });
}
