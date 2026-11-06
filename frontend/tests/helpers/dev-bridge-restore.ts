// A stand-in for the Tauri bridge for the development page /dev/restore only (plan 56). In a plain
// browser tab there is no backend, so the first screen and the restore pages cannot be looked at.
// This answers the three restore commands with canned answers (tests/helpers/restore-first-run.ts,
// PROVISIONAL until backend3's contract is approved), so the pages can be photographed in both
// languages at every window size. It answers nothing else. Never part of a build: only the
// development route imports it.
//
// Flags in the hash, for example `#/dev/restore?at=copy&many`:
//   at=where|copy|check|running|done|failed   open the flow at that page (the data is filled in)
//   empty    no copy at the place            many     30 copies         bad     the copy is damaged
//   upgrade  the copy is from an older app   noai     the assistant was off   cloudkeys  a file or folder
//   was used, so the cloud keys must be typed again
//   down     the cloud refuses the two keys    wrongkey  the first key try is wrong
//   https    the backend refuses the address (no https:// or no host)    path   the bucket name is in the address
//   hold     the restore never ends (to photograph the progress)
//   fail     the restore fails (the first screen shows the reason)
// `after` shows the reminders of a restored computer. `at=wherefilled` opens the "where" page with the cloud fields typed.

import { devFlag, devValue } from './dev-bridge';
import { COPIES, DONE, GOOD_PREVIEW, NEWEST, OLDER } from './restore-first-run';
import { useRestoreStore } from '../../src/features/restore/store';
import type {
  FirstRunCopies,
  FirstRunPreview,
  RestoreStage,
} from '../../src/features/restore/types';

function copies(): FirstRunCopies {
  if (devFlag('empty')) return { rows: [], total_count: 0 };
  if (devFlag('many')) {
    return {
      rows: Array.from({ length: 30 }, (_, index) => ({
        ...NEWEST,
        name: `mandi-2028${String(index).padStart(4, '0')}.mbk`,
        created_at: new Date(Date.UTC(2028, 2, 14 - index, 16)).toISOString(),
        size_bytes: 3_400_000 - index * 10_000,
      })),
      total_count: 30,
    };
  }
  return COPIES;
}

interface RefusedItem {
  code: string;
  field: string | null;
  values?: Record<string, string | number>;
}

function refusalOf(items: RefusedItem[]): Error {
  return Object.assign(new Error(items[0]?.code ?? 'ERR_INTERNAL'), {
    ok: false,
    errors: items.map((item) => ({ ...item, values: item.values ?? {} })),
    ref: 'E-20280315-0001',
  });
}

let keyTries = 0;

function preview(): FirstRunPreview {
  if (devFlag('wrongkey') && keyTries++ === 0) {
    throw refusalOf([{ code: 'ERR_BACKUP_KEY_WRONG', field: 'key' }]);
  }
  const base: FirstRunPreview = { ...GOOD_PREVIEW, needs_upgrade: devFlag('upgrade') };
  // The backend refuses a copy that fails a check, so `bad` is the refusal of a damaged copy.
  if (devFlag('bad')) throw refusalOf([{ code: 'ERR_BACKUP_CORRUPT', field: 'copy' }]);
  return base;
}

const handlers: Record<string, () => unknown> = {
  backup_first_run_list: () => {
    // What the real backend says for wrong cloud keys (backend3, plan 56): two items, one reason.
    if (devFlag('down')) {
      throw refusalOf([
        {
          code: 'ERR_BACKUP_CLOUD_FAILED',
          field: 'source.cloud.key_id',
          values: { reason: 'refused' },
        },
        {
          code: 'ERR_BACKUP_CLOUD_FAILED',
          field: 'source.cloud.secret',
          values: { reason: 'refused' },
        },
      ]);
    }
    if (devFlag('https')) {
      throw refusalOf([
        { code: 'ERR_OUT_OF_RANGE', field: 'source.cloud.endpoint', values: { reason: 'https' } },
      ]);
    }
    // An address with a path after the host is its own refusal (ERR_BACKUP_CLOUD_ADDRESS_PATH).
    if (devFlag('path')) {
      throw refusalOf([
        {
          code: 'ERR_BACKUP_CLOUD_ADDRESS_PATH',
          field: 'source.cloud.endpoint',
          values: {
            address: 'https://shop.r2.cloudflarestorage.com',
            path: '/mandi-bucket',
            bucket: 'mandi-bucket',
          },
        },
      ]);
    }
    return copies();
  },
  backup_first_run_preview: preview,
  backup_first_run_restore: () => {
    if (devFlag('hold')) return new Promise(() => undefined);
    if (devFlag('fail')) throw refusalOf([{ code: 'ERR_INTERNAL', field: null }]);
    return {
      ...DONE,
      assistant_was_on: !devFlag('noai'),
      cloud_needs_keys: devFlag('cloudkeys'),
    };
  },
};

function refusal(): Error {
  return refusalOf([{ code: 'ERR_NOT_IMPLEMENTED', field: null }]);
}

/** Fills the store so the flow opens at the page named by `at` (for photographs). */
export function openRestoreAt(at: string | null): void {
  const store = useRestoreStore.getState();
  const rows = copies().rows;
  if (at === null || at === 'start') return;
  store.setChoice('restore');
  if (at === 'where') return;
  // The typed address matches the refusal that is being photographed, if there is one.
  const typedEndpoint = devFlag('https')
    ? 'https://'
    : devFlag('path')
      ? 'https://shop.r2.cloudflarestorage.com/mandi-bucket'
      : 'https://shop.r2.cloudflarestorage.com';
  store.setCloudField('endpoint', typedEndpoint);
  store.setCloudField('bucket', 'mandi-bucket');
  store.setCloudField('key_id', 'AKIAEXAMPLEKEYID');
  store.setCloudField('secret', 'secret');
  if (at === 'wherefilled') return;
  store.copiesFound(rows);
  if (at === 'copy') return;
  store.setBackupKey('ABCD-EFGH-JKLM-NPQR-STUV-WXYZ');
  store.previewed(preview());
  if (at === 'check') return;
  store.started();
  if (at === 'running') {
    store.progressed({ stage: 'putting_in_place', percent: 40 });
    return;
  }
  if (at === 'done') {
    store.finished({
      ...DONE,
      assistant_was_on: !devFlag('noai'),
      cloud_needs_keys: devFlag('cloudkeys'),
    });
  }
}

/** A way to move the progress bar from the browser console, for photographs. */
export function exposeProgress(): void {
  Object.assign(window, {
    restoreProgress: (stage: RestoreStage, percent: number | null) => {
      useRestoreStore.getState().progressed({ stage, percent });
    },
  });
}

/** The page named by `at`, or null. */
export function restoreAtFlag(): string | null {
  return devValue('at');
}

/** The older copy exists for tests of choosing another one. */
export { OLDER };

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installRestoreDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      invoke: (command: string): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const handler = handlers[command];
            if (handler === undefined) {
              reject(refusal());
              return;
            }
            try {
              resolve(handler());
            } catch (failure: unknown) {
              reject(failure instanceof Error ? failure : refusal());
            }
          }, 20);
        }),
    },
  });
}
