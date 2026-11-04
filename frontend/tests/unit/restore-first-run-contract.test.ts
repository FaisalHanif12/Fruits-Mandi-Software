// The restore pages of a new computer and the backend agree on plan 56: each wrapper in
// features/restore/api.ts sends the command name the backend registers and the input in the shape
// `{ input }`, with the very requests the backend's own test uses. The names are also read from the
// backend's own command file, so a command renamed there fails here first. The fixtures in
// tests/fixtures are the backend's own, copied; here they must still be the same bytes as the
// backend's, and every error code the three commands can send must have words in Urdu and in English.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as api from '../../src/features/restore/api';
import { bundles, LANGUAGES } from '../../src/i18n';
import { errorMessageId } from '../../src/shared/api';
import * as invoke from '../../src/shared/api/invoke';
import {
  LIST,
  LIST_REQUEST,
  PREVIEW,
  PREVIEW_REQUEST,
  RESTORE_DONE,
  RESTORE_REQUEST,
} from '../helpers/restore-first-run';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const BACKEND_FILE = path.resolve(
  __dirname,
  '../../../backend/app/src/commands/backup_first_run.rs',
);
const BACKEND_FIXTURES = path.resolve(__dirname, '../../../backend/crates/backup/tests/fixtures');
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

describe('the commands the pages call are the ones the backend declares', () => {
  it('every backup_first_run_ command used in api.ts exists in the backend file', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    const declared = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    for (const name of [
      'backup_first_run_list',
      'backup_first_run_preview',
      'backup_first_run_restore',
    ]) {
      expect(declared).toContain(name);
    }
  });

  it('the progress event is the one the backend sends', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    expect(text).toContain(`PROGRESS_EVENT: &str = "${api.RESTORE_PROGRESS_EVENT}"`);
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  it('list, preview and restore send the backend requests', async () => {
    await api.backupFirstRunList(LIST_REQUEST);
    await api.backupFirstRunPreview(PREVIEW_REQUEST);
    await api.backupFirstRunRestore(RESTORE_REQUEST);
    expect(call.mock.calls).toEqual([
      ['backup_first_run_list', { input: LIST_REQUEST }],
      ['backup_first_run_preview', { input: PREVIEW_REQUEST }],
      ['backup_first_run_restore', { input: RESTORE_REQUEST }],
    ]);
  });

  it('returns what the command sent', async () => {
    call
      .mockResolvedValueOnce(LIST)
      .mockResolvedValueOnce(PREVIEW)
      .mockResolvedValueOnce(RESTORE_DONE);
    expect(await api.backupFirstRunList(LIST_REQUEST)).toBe(LIST);
    expect(await api.backupFirstRunPreview(PREVIEW_REQUEST)).toBe(PREVIEW);
    expect(await api.backupFirstRunRestore(RESTORE_REQUEST)).toBe(RESTORE_DONE);
  });
});

describe('the fixtures are the backend ones', () => {
  it('every backup_first_run_ fixture has the same bytes as the backend file', () => {
    const names = readdirSync(BACKEND_FIXTURES).filter((name) =>
      name.startsWith('backup_first_run_'),
    );
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) {
      const mine = readFileSync(path.join(FRONTEND_FIXTURES, name));
      const theirs = readFileSync(path.join(BACKEND_FIXTURES, name));
      expect(mine.equals(theirs), name).toBe(true);
    }
  });

  it('the answers carry the fields the screens read', () => {
    expect(Object.keys(PREVIEW).sort()).toEqual(
      [
        'app_schema_version',
        'checks',
        'copy',
        'created_at',
        'needs_upgrade',
        'schema_version',
        'size_bytes',
        'totals',
      ].sort(),
    );
    expect(Object.keys(RESTORE_DONE).sort()).toEqual(
      [
        'assistant_was_on',
        'chain_key_restored',
        'cloud_needs_keys',
        'restored_from',
        'totals',
        'upgraded_from',
      ].sort(),
    );
    expect(Object.keys(LIST).sort()).toEqual(['rows', 'total_count']);
  });
});

describe('every code the commands can send has words in both languages', () => {
  // The codes in the doc comments of the backend file (plan 56 Section 5).
  const CODES = [
    'ERR_ALREADY_SET_UP',
    'ERR_REQUIRED',
    'ERR_OUT_OF_RANGE',
    'ERR_BACKUP_CLOUD_FAILED',
    'ERR_BACKUP_USB_NOT_PRESENT',
    'ERR_BACKUP_BUSY',
    'ERR_BACKUP_KEY_FORMAT',
    'ERR_BACKUP_KEY_WRONG',
    'ERR_BACKUP_NOT_FOUND',
    'ERR_BACKUP_CORRUPT',
    'ERR_BACKUP_NEWER_THAN_APP',
    'ERR_BACKUP_CHAIN_KEY_MISSING',
    'ERR_DISK_FULL',
    'ERR_MIGRATION_FAILED',
    'ERR_BACKUP_RESTORE_FAILED',
  ] as const;

  it('the backend file names each of them', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    for (const code of CODES) expect(text, code).toContain(code);
  });

  it.each(LANGUAGES)('%s has a message for each code', (language) => {
    for (const code of CODES) {
      expect(bundles[language].hasMessage(errorMessageId(code)), code).toBe(true);
    }
  });
});
