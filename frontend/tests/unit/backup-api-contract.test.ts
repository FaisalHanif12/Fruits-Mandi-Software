// The screens and the backend agree on plan 18 (backup and restore): each wrapper in
// features/backup/api.ts sends the command name the backend registers and the input in the shape
// `{ input }` (the three commands without input send only their name). The names are also read
// from the backend's own command file, so a command renamed there fails here first. The fixtures
// in tests/fixtures are the backend's own, copied; here they must still be the same bytes as the
// backend's, must carry exactly the fields of the generated types, and every error code the
// backup commands can send must have words in Urdu and in English.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import backupCodeSource from '../../src/bindings/BackupCode.ts?raw';
import * as api from '../../src/features/backup/api';
import { bundles, LANGUAGES } from '../../src/i18n';
import * as invoke from '../../src/shared/api/invoke';
import { errorMessageId } from '../../src/shared/api';
import type { WireCode } from '../../src/bindings/WireCode';
import {
  KEY_CONFIRM_REQUEST,
  LIST_ALL,
  LIST_REQUEST,
  NOW_ALL_OK,
  NOW_QUEUED,
  NOW_REQUEST,
  PREVIEW,
  PREVIEW_REQUEST,
  RESTORE_DONE,
  RESTORE_REQUEST,
  SETTINGS_GET,
  SETTINGS_SET_REQUEST,
  STATUS_AFTER,
  STATUS_NEW,
  TEST_REQUEST,
  TEST_RESULT,
} from '../helpers/backup-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const BACKEND_FILE = path.resolve(__dirname, '../../../backend/app/src/commands/backup.rs');
const BACKEND_FIXTURES = path.resolve(__dirname, '../../../backend/crates/backup/tests/fixtures');
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

describe('the commands the screens call are the ones the backend declares', () => {
  it('every backup_ command used in api.ts exists in the backend file', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    const declared = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    for (const name of [
      'backup_status',
      'backup_list',
      'backup_now',
      'backup_restore_preview',
      'backup_restore',
      'backup_test_restore',
      'backup_settings_get',
      'backup_settings_set',
      'backup_key_setup',
      'backup_key_confirm',
    ]) {
      expect(declared).toContain(name);
    }
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  it('the three commands without input send only their name', async () => {
    await api.backupStatus();
    await api.backupSettingsGet();
    await api.backupKeySetup();
    expect(call.mock.calls).toEqual([
      ['backup_status'],
      ['backup_settings_get'],
      ['backup_key_setup'],
    ]);
  });

  it('the others send the exact request the backend wrote', async () => {
    await api.backupList(LIST_REQUEST);
    await api.backupNow(NOW_REQUEST);
    await api.backupRestorePreview(PREVIEW_REQUEST);
    await api.backupRestore(RESTORE_REQUEST);
    await api.backupTestRestore(TEST_REQUEST);
    await api.backupSettingsSet(SETTINGS_SET_REQUEST);
    await api.backupKeyConfirm(KEY_CONFIRM_REQUEST);
    expect(call.mock.calls).toEqual([
      ['backup_list', { input: LIST_REQUEST }],
      ['backup_now', { input: NOW_REQUEST }],
      ['backup_restore_preview', { input: PREVIEW_REQUEST }],
      ['backup_restore', { input: RESTORE_REQUEST }],
      ['backup_test_restore', { input: TEST_REQUEST }],
      ['backup_settings_set', { input: SETTINGS_SET_REQUEST }],
      ['backup_key_confirm', { input: KEY_CONFIRM_REQUEST }],
    ]);
  });
});

describe('the fixtures are the backend’s own', () => {
  it('every backup_ fixture is the same text as the backend’s file', () => {
    const names = readdirSync(BACKEND_FIXTURES).filter((name) => name.endsWith('.json'));
    expect(names.length).toBeGreaterThan(20);
    for (const name of names) {
      expect(readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8'), name).toBe(
        readFileSync(path.join(BACKEND_FIXTURES, name), 'utf8'),
      );
    }
  });
});

const keysOf = (value: object) => Object.keys(value).sort();

describe('the answers carry exactly the fields of the generated types', () => {
  it('BackupStatus', () => {
    const fields = [
      'cloud_age_hours',
      'cloud_enabled',
      'cloud_level',
      'cloud_ready',
      'errors',
      'key_confirm_due',
      'key_confirmed',
      'key_set',
      'last_cloud_at',
      'last_local_at',
      'last_test_restore_at',
      'last_usb_at',
      'pack_pending',
      'queue_len',
      'running',
      'test_restore_due',
      'usb_enabled',
      'usb_present',
    ];
    expect(keysOf(STATUS_NEW)).toEqual(fields);
    expect(keysOf(STATUS_AFTER)).toEqual(fields);
  });

  it('BackupList and BackupRow', () => {
    expect(keysOf(LIST_ALL)).toEqual(['cloud_reachable', 'rows', 'total_count']);
    for (const row of LIST_ALL.rows) {
      expect(keysOf(row)).toEqual([
        'created_at',
        'encrypted',
        'kind',
        'name',
        'size_bytes',
        'status',
      ]);
    }
  });

  it('RunReport and TargetResult', () => {
    for (const report of [NOW_ALL_OK, NOW_QUEUED]) {
      expect(keysOf(report)).toEqual(['check_ms', 'results', 'snapshot_ms', 'started_at']);
      for (const result of report.results) {
        expect(keysOf(result)).toEqual([
          'code',
          'file',
          'kind',
          'ok',
          'queued',
          'reason',
          'size_bytes',
        ]);
      }
    }
  });

  it('RestorePreview, RestoreDone and TestRestoreResult', () => {
    expect(keysOf(PREVIEW)).toEqual([
      'app_schema_version',
      'backup',
      'created_at',
      'current',
      'lost',
      'needs_upgrade',
      'schema_version',
      'size_bytes',
      'totals',
    ]);
    expect(keysOf(PREVIEW.totals)).toEqual([
      'accounts',
      'audit_rows',
      'bikri_entries',
      'cash_paisa',
      'credit_paisa',
      'debit_paisa',
      'journal_entries',
      'last_entry_date',
      'vouchers',
    ]);
    expect(keysOf(PREVIEW.lost)).toEqual(['bikri_entries', 'journal_entries', 'vouchers']);
    expect(keysOf(RESTORE_DONE)).toEqual([
      'restored_from',
      'safety_copy',
      'totals',
      'upgraded_from',
    ]);
    expect(keysOf(TEST_RESULT)).toEqual([
      'backup',
      'checks',
      'created_at',
      'live',
      'ok',
      'restored',
    ]);
  });

  it('BackupSettingsView', () => {
    expect(keysOf(SETTINGS_GET)).toEqual([
      'cloud_access_key_set',
      'cloud_secret_set',
      'local_dir_effective',
      'settings',
    ]);
    expect(keysOf(SETTINGS_GET.settings)).toEqual([
      'closing_time',
      'cloud_bucket',
      'cloud_enabled',
      'cloud_endpoint',
      'cloud_prefix',
      'cloud_region',
      'local_dir',
      'usb_enabled',
      'usb_folder',
    ]);
  });
});

describe('every code a backup command can send has words in both languages', () => {
  it('has a message in Urdu and English for each BackupCode', () => {
    const codes = [...backupCodeSource.matchAll(/["'](ERR_[A-Z0-9_]+)["']/g)].map(
      (match) => match[1] as WireCode,
    );
    expect(codes.length).toBeGreaterThan(20);
    for (const code of codes) {
      for (const language of LANGUAGES) {
        expect(bundles[language].hasMessage(errorMessageId(code)), `${language} ${code}`).toBe(
          true,
        );
      }
    }
  });
});
