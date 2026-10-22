// Test helper: sets the answers of the backup command wrappers (plan 18) from the backend's own
// answers in backup-real.ts. The wrappers are mocked in the test file with vi.mock. Only what a
// test changes is passed in; the rest is the real answer of the backend. A refusal is a rejected
// promise (`CommandError`), the way `invoke` fails.

import { vi } from 'vitest';
import type { BackupList } from '../../src/bindings/BackupList';
import type { BackupRow } from '../../src/bindings/BackupRow';
import type { BackupSettingsView } from '../../src/bindings/BackupSettingsView';
import type { BackupStatus } from '../../src/bindings/BackupStatus';
import * as backupApi from '../../src/features/backup/api';
import { CommandError } from '../../src/shared/api';
import { refusal, type RefusalItem } from './ui-error';
import {
  KEY_SETUP,
  LIST_ALL,
  NOW_ALL_OK,
  PREVIEW,
  RESTORE_DONE,
  SETTINGS_GET,
  STATUS_AFTER,
  TEST_RESULT,
} from './backup-real';

export * from './backup-real';

/** A backend refusal, as a rejected promise. */
export function refused(...items: RefusalItem[]): CommandError {
  return new CommandError(refusal(items));
}

export interface BackupOptions {
  status?: Partial<BackupStatus>;
  rows?: readonly BackupRow[];
  cloudReachable?: boolean;
  settings?: BackupSettingsView;
}

export interface FakeBackup {
  status: BackupStatus;
  rows: BackupRow[];
  settings: BackupSettingsView;
}

/** Sets the answers of the backup commands, and returns the store behind them. */
export function fakeBackup(options: BackupOptions = {}): FakeBackup {
  const store: FakeBackup = {
    status: { ...STATUS_AFTER, ...options.status },
    rows: [...(options.rows ?? LIST_ALL.rows)],
    settings: options.settings ?? SETTINGS_GET,
  };
  vi.mocked(backupApi.backupStatus).mockImplementation(() => Promise.resolve(store.status));
  vi.mocked(backupApi.backupList).mockImplementation((input) => {
    const wanted = store.rows.filter((row) => input.kind === null || row.kind === input.kind);
    const limit = input.limit > 0 ? input.limit : 100;
    const page: BackupList = {
      rows: wanted.slice(input.offset, input.offset + limit),
      total_count: wanted.length,
      cloud_reachable: options.cloudReachable ?? true,
    };
    return Promise.resolve(page);
  });
  vi.mocked(backupApi.backupNow).mockImplementation(() => Promise.resolve(NOW_ALL_OK));
  vi.mocked(backupApi.backupRestorePreview).mockImplementation((input) =>
    Promise.resolve({ ...PREVIEW, backup: input.backup }),
  );
  vi.mocked(backupApi.backupRestore).mockImplementation(() => Promise.resolve(RESTORE_DONE));
  vi.mocked(backupApi.backupTestRestore).mockImplementation(() => Promise.resolve(TEST_RESULT));
  vi.mocked(backupApi.backupSettingsGet).mockImplementation(() => Promise.resolve(store.settings));
  vi.mocked(backupApi.backupSettingsSet).mockImplementation((input) => {
    store.settings = {
      ...store.settings,
      settings: input.settings,
      cloud_access_key_set:
        input.cloud_access_key_id === null
          ? store.settings.cloud_access_key_set
          : input.cloud_access_key_id !== '',
      cloud_secret_set:
        input.cloud_secret_access_key === null
          ? store.settings.cloud_secret_set
          : input.cloud_secret_access_key !== '',
    };
    return Promise.resolve(store.settings);
  });
  vi.mocked(backupApi.backupKeySetup).mockImplementation(() => Promise.resolve(KEY_SETUP));
  vi.mocked(backupApi.backupKeyConfirm).mockImplementation(() => Promise.resolve(null));
  return store;
}
