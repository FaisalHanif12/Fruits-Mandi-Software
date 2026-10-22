// The real answers and requests of the backend for plan 18 (backup and restore). The backend wrote
// them in its own test (backend/app/tests/backup_commands.rs, into backend/crates/backup/tests/
// fixtures); they are copied here, never edited, and read as the generated types. Money in them is
// paisa, as on the wire. A reader takes `unknown`, because the money in a binding is a branded type
// that a plain number is not.

import type { BackupList } from '../../src/bindings/BackupList';
import type { BackupListInput } from '../../src/bindings/BackupListInput';
import type { BackupNowInput } from '../../src/bindings/BackupNowInput';
import type { BackupSettingsInput } from '../../src/bindings/BackupSettingsInput';
import type { BackupSettingsView } from '../../src/bindings/BackupSettingsView';
import type { BackupStatus } from '../../src/bindings/BackupStatus';
import type { KeyConfirmInput } from '../../src/bindings/KeyConfirmInput';
import type { KeyCreated } from '../../src/bindings/KeyCreated';
import type { RestoreDone } from '../../src/bindings/RestoreDone';
import type { RestoreInput } from '../../src/bindings/RestoreInput';
import type { RestorePreview } from '../../src/bindings/RestorePreview';
import type { RestorePreviewInput } from '../../src/bindings/RestorePreviewInput';
import type { RunReport } from '../../src/bindings/RunReport';
import type { TestRestoreInput } from '../../src/bindings/TestRestoreInput';
import type { TestRestoreResult } from '../../src/bindings/TestRestoreResult';
import type { BackupWireError } from '../../src/bindings/BackupWireError';
import keyConfirmRequest from '../fixtures/backup_key_confirm_request.json';
import keyConfirmWrong from '../fixtures/backup_key_confirm_wrong.json';
import keyConfirmWrongRequest from '../fixtures/backup_key_confirm_wrong_request.json';
import keySetup from '../fixtures/backup_key_setup.json';
import keySetupAgain from '../fixtures/backup_key_setup_again.json';
import list from '../fixtures/backup_list.json';
import listRequest from '../fixtures/backup_list_request.json';
import now from '../fixtures/backup_now.json';
import nowQueued from '../fixtures/backup_now_queued.json';
import nowQueuedRequest from '../fixtures/backup_now_queued_request.json';
import nowRequest from '../fixtures/backup_now_request.json';
import restore from '../fixtures/backup_restore.json';
import restorePreview from '../fixtures/backup_restore_preview.json';
import restorePreviewRequest from '../fixtures/backup_restore_preview_request.json';
import restoreRequest from '../fixtures/backup_restore_request.json';
import restoreUnconfirmed from '../fixtures/backup_restore_unconfirmed.json';
import restoreUnconfirmedRequest from '../fixtures/backup_restore_unconfirmed_request.json';
import settingsGet from '../fixtures/backup_settings_get.json';
import settingsSet from '../fixtures/backup_settings_set.json';
import settingsSetRefused from '../fixtures/backup_settings_set_refused.json';
import settingsSetRefusedRequest from '../fixtures/backup_settings_set_refused_request.json';
import settingsSetRequest from '../fixtures/backup_settings_set_request.json';
import statusAfter from '../fixtures/backup_status_after.json';
import statusNew from '../fixtures/backup_status_new.json';
import testRestore from '../fixtures/backup_test_restore.json';
import testRestoreRequest from '../fixtures/backup_test_restore_request.json';

function readBackupList(value: unknown): BackupList {
  return value as BackupList;
}
function readBackupListInput(value: unknown): BackupListInput {
  return value as BackupListInput;
}
function readBackupNowInput(value: unknown): BackupNowInput {
  return value as BackupNowInput;
}
function readBackupSettingsInput(value: unknown): BackupSettingsInput {
  return value as BackupSettingsInput;
}
function readBackupSettingsView(value: unknown): BackupSettingsView {
  return value as BackupSettingsView;
}
function readBackupStatus(value: unknown): BackupStatus {
  return value as BackupStatus;
}
function readKeyConfirmInput(value: unknown): KeyConfirmInput {
  return value as KeyConfirmInput;
}
function readKeyCreated(value: unknown): KeyCreated {
  return value as KeyCreated;
}
function readRestoreDone(value: unknown): RestoreDone {
  return value as RestoreDone;
}
function readRestoreInput(value: unknown): RestoreInput {
  return value as RestoreInput;
}
function readRestorePreview(value: unknown): RestorePreview {
  return value as RestorePreview;
}
function readRestorePreviewInput(value: unknown): RestorePreviewInput {
  return value as RestorePreviewInput;
}
function readRunReport(value: unknown): RunReport {
  return value as RunReport;
}
function readTestRestoreInput(value: unknown): TestRestoreInput {
  return value as TestRestoreInput;
}
function readTestRestoreResult(value: unknown): TestRestoreResult {
  return value as TestRestoreResult;
}
function readWireError(value: unknown): BackupWireError {
  return value as BackupWireError;
}

// Status
export const STATUS_NEW = readBackupStatus(statusNew);
export const STATUS_AFTER = readBackupStatus(statusAfter);

// The list
export const LIST_REQUEST = readBackupListInput(listRequest);
export const LIST_ALL = readBackupList(list);

// Back up now
export const NOW_REQUEST = readBackupNowInput(nowRequest);
export const NOW_ALL_OK = readRunReport(now);
export const NOW_QUEUED_REQUEST = readBackupNowInput(nowQueuedRequest);
export const NOW_QUEUED = readRunReport(nowQueued);

// Restore
export const PREVIEW_REQUEST = readRestorePreviewInput(restorePreviewRequest);
export const PREVIEW = readRestorePreview(restorePreview);
export const RESTORE_REQUEST = readRestoreInput(restoreRequest);
export const RESTORE_DONE = readRestoreDone(restore);
export const RESTORE_UNCONFIRMED_REQUEST = readRestoreInput(restoreUnconfirmedRequest);
export const RESTORE_UNCONFIRMED = readWireError(restoreUnconfirmed);

// Test restore
export const TEST_REQUEST = readTestRestoreInput(testRestoreRequest);
export const TEST_RESULT = readTestRestoreResult(testRestore);

// Settings
export const SETTINGS_GET = readBackupSettingsView(settingsGet);
export const SETTINGS_SET_REQUEST = readBackupSettingsInput(settingsSetRequest);
export const SETTINGS_SET = readBackupSettingsView(settingsSet);
export const SETTINGS_REFUSED_REQUEST = readBackupSettingsInput(settingsSetRefusedRequest);
export const SETTINGS_REFUSED = readWireError(settingsSetRefused);

// The key
export const KEY_SETUP = readKeyCreated(keySetup);
export const KEY_SETUP_AGAIN = readWireError(keySetupAgain);
export const KEY_CONFIRM_REQUEST = readKeyConfirmInput(keyConfirmRequest);
export const KEY_CONFIRM_WRONG_REQUEST = readKeyConfirmInput(keyConfirmWrongRequest);
export const KEY_CONFIRM_WRONG = readWireError(keyConfirmWrong);
