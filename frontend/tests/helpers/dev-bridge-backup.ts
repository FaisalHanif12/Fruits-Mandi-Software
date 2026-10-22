// A stand-in for the Tauri bridge for the development page /dev/backup only (plan 18). In a plain
// browser tab there is no backend, so the backup screen cannot be looked at. This answers the ten
// backup commands with the backend's own fixtures (backup-real.ts), so the screen can be opened in a
// browser and photographed in both languages. It answers nothing else: any other command is refused
// with ERR_NOT_IMPLEMENTED. Never part of a build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/backup?old&many`:
//   new     no key and no copy yet        old     the last cloud copy is over a day old
//   key     a key made but not confirmed  cloud   the cloud copy is on with its keys stored
//   empty   no copies at all              many    120 copies (page by page)
//   bad     one copy failed its check     down    the cloud could not be reached
//   lost    the restore would lose records

import type { BackupList } from '../../src/bindings/BackupList';
import type { BackupRow } from '../../src/bindings/BackupRow';
import type { BackupStatus } from '../../src/bindings/BackupStatus';
import { devFlag } from './dev-bridge';
import {
  KEY_SETUP,
  LIST_ALL,
  NOW_ALL_OK,
  NOW_QUEUED,
  PREVIEW,
  RESTORE_DONE,
  SETTINGS_GET,
  STATUS_AFTER,
  STATUS_NEW,
  TEST_RESULT,
} from './backup-real';

type Input = Record<string, unknown>;

function status(): BackupStatus {
  if (devFlag('new')) return STATUS_NEW;
  const base = { ...STATUS_AFTER };
  if (devFlag('old')) {
    return {
      ...base,
      cloud_level: 'red',
      cloud_age_hours: 30,
      queue_len: 2,
      errors: [
        {
          kind: 'cloud',
          code: 'ERR_BACKUP_CLOUD_FAILED',
          at: '2026-10-10T09:02:00.000Z',
          check_failed: false,
          file: null,
        },
      ],
    };
  }
  if (devFlag('key')) return { ...base, key_confirmed: false, key_confirm_due: true };
  return base;
}

function rows(): BackupRow[] {
  if (devFlag('empty')) return [];
  const template = LIST_ALL.rows[0];
  if (devFlag('many') && template !== undefined) {
    return Array.from({ length: 120 }, (_, index) => ({
      ...template,
      name: `mandi-local-${String(index).padStart(4, '0')}.db`,
      size_bytes: 2_500_000 + index * 1_000,
    }));
  }
  const sized = LIST_ALL.rows.map((row, index) => ({
    ...row,
    size_bytes: 3_400_000 * (index + 1),
  }));
  return devFlag('bad')
    ? sized.map((row) => (row.kind === 'usb' ? { ...row, status: 'bad' as const } : row))
    : sized;
}

const handlers: Record<string, (input: Input) => unknown> = {
  backup_status: () => status(),
  backup_list: (input) => {
    const kind = typeof input['kind'] === 'string' ? input['kind'] : null;
    const offset = Number(input['offset'] ?? 0);
    const limit = Number(input['limit'] ?? 100) || 100;
    const wanted = rows().filter((row) => kind === null || row.kind === kind);
    const page: BackupList = {
      rows: wanted.slice(offset, offset + limit),
      total_count: wanted.length,
      cloud_reachable: !devFlag('down'),
    };
    return page;
  },
  backup_now: () => (devFlag('old') ? NOW_QUEUED : NOW_ALL_OK),
  backup_restore_preview: (input) => ({
    ...PREVIEW,
    backup: input['backup'],
    lost: devFlag('lost') ? { journal_entries: 12, bikri_entries: 2, vouchers: 3 } : PREVIEW.lost,
  }),
  backup_restore: () => RESTORE_DONE,
  backup_test_restore: () => TEST_RESULT,
  backup_settings_get: () =>
    devFlag('cloud')
      ? {
          ...SETTINGS_GET,
          cloud_access_key_set: true,
          cloud_secret_set: true,
          settings: {
            ...SETTINGS_GET.settings,
            usb_enabled: true,
            usb_folder: 'E:\\MandiBackup',
            cloud_enabled: true,
            cloud_endpoint: 'https://example.r2.cloudflarestorage.com',
            cloud_bucket: 'mandi-backup',
          },
        }
      : SETTINGS_GET,
  backup_settings_set: (input) => ({ ...SETTINGS_GET, settings: input['settings'] }),
  backup_key_setup: () => KEY_SETUP,
  backup_key_confirm: () => null,
};

/** What Tauri rejects with for a command this page does not answer. */
function refusal(): Error {
  return Object.assign(new Error('ERR_NOT_IMPLEMENTED'), {
    ok: false,
    errors: [{ code: 'ERR_NOT_IMPLEMENTED', field: null, values: {} }],
    ref: '',
  });
}

function inputOf(args: unknown): Input {
  if (typeof args !== 'object' || args === null) return {};
  const inner = (args as Input)['input'];
  return typeof inner === 'object' && inner !== null ? (inner as Input) : {};
}

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installBackupDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const handler = handlers[command];
            if (handler === undefined) {
              reject(refusal());
              return;
            }
            resolve(handler(inputOf(args)));
          }, 20);
        }),
    },
  });
}
