// A stand-in for the Tauri bridge for the development page /dev/settings (plan 22) only. In a plain
// browser tab there is no backend, so the new Settings tabs (Day and check, Audit log, App and updates)
// cannot be looked at. This answers their commands with the backend's own fixtures (b5-real.ts), so
// the tabs can be opened in a browser and photographed in both languages and at every window size.
// It answers nothing else: any other command is refused with ERR_NOT_IMPLEMENTED. Never part of a
// build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/settings?tab=audit&many`:
//   tab=day|audit|app|language   the tab that opens
//   none     no check yet, no update on offer, an empty log   bad     the books check finds problems
//   failed   the update server cannot be reached              drafts  unsaved entries (no install)
//   many     230 entries in the audit log                     down    every command fails
//   installing  the install never answers (the app is closing)

import type { AuditRow } from '../../src/bindings/AuditRow';
import type { DailyHealth } from '../../src/bindings/DailyHealth';
import type { Settings } from '../../src/bindings/Settings';
import { devFlag } from './dev-bridge';
import {
  AUDIT_FIRST_PAGE,
  AUDIT_OPTIONS,
  AUDIT_SETTINGS,
  DAY_GET,
  HEALTH_LAST,
  INSTALL_OK,
  SUPPORT_SAVED,
  UPDATE_AVAILABLE,
  UPDATE_NONE,
} from './b5-real';
import { paisaForTest } from './paisa';

type Input = Record<string, unknown>;

const SETTINGS: Settings = {
  firm: {
    name_ur: 'ماہر عدنان اینڈ برادرز',
    name_en: 'Maher Adnan and Brothers',
    mandi_ur: 'نئی سبزی منڈی کوٹ رادھا کشن',
    mandi_en: 'New Sabzi Mandi Kot Radha Kishan',
    address: 'Kot Radha Kishan',
    phone: '0300-1234567',
    firm_contacts: [],
    trade_mark: '',
  },
  idle_lock_minutes: 15,
  default_language: 'ur',
  number_grouping: 'international',
};

const CHECKED_AT = '2026-10-23T09:05:00Z';

/** The check of the worked day, or one that finds problems (the `bad` flag). */
function health(): DailyHealth | null {
  if (devFlag('none')) return null;
  if (!devFlag('bad')) return { ...HEALTH_LAST, checked_at: CHECKED_AT };
  return {
    ...HEALTH_LAST,
    checked_at: CHECKED_AT,
    ok: false,
    books: {
      ...HEALTH_LAST.books,
      balanced: false,
      difference_paisa: paisaForTest(200_000),
      journal_chain: { ok: false, at: '12' },
      audit_chain: { ok: false, at: null },
    },
    data: { state: 'failed', reference: 'E-20261023-0007' },
    backup: {
      last_local_at: null,
      last_usb_at: null,
      last_cloud_at: '2026-10-20T09:00:00Z',
      cloud_age_hours: 60,
      cloud_level: 'alert',
      queue_len: 2,
      errors: 1,
    },
  };
}

/** The rows of the log: the backend's, with real days and numbers (its fixture holds stand-ins). */
function auditRows(): AuditRow[] {
  const base = [...AUDIT_FIRST_PAGE.rows, ...AUDIT_SETTINGS.rows];
  const rows: AuditRow[] = [];
  const count = devFlag('many') ? 230 : base.length;
  for (let index = 0; index < count; index += 1) {
    const row = base[index % base.length];
    if (row === undefined) continue;
    rows.push({
      ...row,
      id: 1000 - index,
      date: '2026-10-10',
      time: `14:05:${String(index % 60).padStart(2, '0')}`,
      at: '2026-10-10T09:05:00Z',
    });
  }
  return rows;
}

const handlers: Record<string, (input: Input) => unknown> = {
  settings_get: () => SETTINGS,
  // The learned words, as a page: none yet (the shape of a real answer, so the tab draws its empty state).
  translit_pairs_list: () => ({ rows: [], total_count: 0, totals: { use_count: 0 } }),
  settings_day_get: () => DAY_GET,
  settings_day_set: (input) => input,
  settings_set_number_grouping: (input) => input['grouping'],
  health_last: () => health(),
  health_run: () => health(),
  updates_check: () => {
    if (devFlag('failed')) throw refusal('ERR_UPDATE_CHECK_FAILED');
    if (devFlag('none')) return UPDATE_NONE;
    return { ...UPDATE_AVAILABLE, drafts: devFlag('drafts') ? 2 : 0 };
  },
  updates_install: () => INSTALL_OK,
  support_info_save: (input) => ({
    ...SUPPORT_SAVED,
    path: typeof input['path'] === 'string' ? input['path'] : '',
    log_lines: 120,
  }),
  audit_options: () => AUDIT_OPTIONS,
  audit_list: (input) => {
    const all = devFlag('none') ? [] : auditRows();
    const offset = Number(input['offset'] ?? 0);
    const limit = Number(input['limit'] ?? 100) || 100;
    return {
      ...AUDIT_FIRST_PAGE,
      rows: all.slice(offset, offset + limit),
      offset,
      limit,
      total_count: all.length,
    };
  },
};

/** What Tauri rejects with for a command this page does not answer, or for the `down` flag. */
function refusal(code: string): Error {
  return Object.assign(new Error(code), {
    ok: false,
    errors: [{ code, field: null, values: {} }],
    ref: 'E-20261010-0001',
  });
}

function inputOf(args: unknown): Input {
  if (typeof args !== 'object' || args === null) return {};
  const inner = (args as Input)['input'];
  return typeof inner === 'object' && inner !== null ? (inner as Input) : {};
}

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installSettingsDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const handler = handlers[command];
            if (handler === undefined) {
              reject(refusal('ERR_NOT_IMPLEMENTED'));
              return;
            }
            if (devFlag('down')) {
              reject(refusal('ERR_INTERNAL'));
              return;
            }
            if (command === 'updates_install' && devFlag('installing')) return;
            try {
              resolve(handler(inputOf(args)));
            } catch (error) {
              reject(error instanceof Error ? error : refusal('ERR_INTERNAL'));
            }
          }, 20);
        }),
    },
  });
}
