// The screens and the backend agree on plan 22 (D5 early track, B5): the audit viewer, the Day
// settings, the number grouping, the books check, updates and the support file. Each wrapper sends
// the command name the backend registers and the input in the shape `{ input }` (the commands without
// input send only their name). The names are also read from the backend's own command files, so a
// command renamed there fails here first. The fixtures in tests/fixtures are the backend's own,
// copied; here they must still be the same bytes as the backend's, must carry exactly the fields of
// the generated types, and every error code the update commands can send must have words in Urdu and
// in English, as must every action and area the audit viewer names.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as auditApi from '../../src/features/audit/api';
import { KNOWN_ACTIONS, KNOWN_ENTITIES } from '../../src/features/audit/words';
import * as api from '../../src/features/settings/maintenance-api';
import { bundles, LANGUAGES } from '../../src/i18n';
import { errorMessageId } from '../../src/shared/api';
import * as invoke from '../../src/shared/api/invoke';
import {
  AUDIT_FIRST_PAGE,
  AUDIT_OPTIONS,
  AUDIT_SETTINGS,
  AUDIT_SETTINGS_REQUEST,
  DAY_GET,
  DAY_SET_REQUEST,
  GROUPING_REQUEST,
  HEALTH_RUN,
  INSTALL_OK,
  INSTALL_REQUEST,
  SUPPORT_REQUEST,
  SUPPORT_SAVED,
  UPDATE_AVAILABLE,
} from '../helpers/b5-real';
import type { WireCode } from '../../src/bindings/WireCode';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const COMMANDS = path.resolve(__dirname, '../../../backend/app/src/commands');
const BACKEND_FIXTURES = path.resolve(__dirname, '../../../backend/app/tests/fixtures');
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

const DECLARED: Readonly<Record<string, readonly string[]>> = {
  'audit.rs': ['audit_list', 'audit_options'],
  'settings_day.rs': ['settings_day_get', 'settings_day_set', 'settings_set_number_grouping'],
  'health.rs': ['health_run', 'health_last'],
  'support.rs': ['support_info_save'],
  'updates.rs': ['updates_check', 'updates_install'],
};

describe('the commands the screens call are the ones the backend declares', () => {
  it('every command used by the wrappers exists in the backend file of its module', () => {
    for (const [file, names] of Object.entries(DECLARED)) {
      const text = readFileSync(path.join(COMMANDS, file), 'utf8');
      const declared = [...text.matchAll(/pub (?:async )?fn (\w+)/g)].map(
        (match) => match[1] ?? '',
      );
      for (const name of names) expect(declared, `${file} declares ${name}`).toContain(name);
    }
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  it('the audit viewer: the page with the filters as the backend wrote them, and the lists', async () => {
    await auditApi.auditList(AUDIT_SETTINGS_REQUEST);
    expect(call).toHaveBeenLastCalledWith('audit_list', { input: AUDIT_SETTINGS_REQUEST });
    await auditApi.auditOptions();
    expect(call).toHaveBeenLastCalledWith('audit_options');
  });

  it('the Day settings and the number grouping', async () => {
    await api.settingsDayGet();
    expect(call).toHaveBeenLastCalledWith('settings_day_get');
    await api.settingsDaySet(DAY_SET_REQUEST);
    expect(call).toHaveBeenLastCalledWith('settings_day_set', { input: DAY_SET_REQUEST });
    await api.settingsSetNumberGrouping(GROUPING_REQUEST);
    expect(call).toHaveBeenLastCalledWith('settings_set_number_grouping', {
      input: GROUPING_REQUEST,
    });
  });

  it('the books check, updates and the support file', async () => {
    await api.healthRun();
    expect(call).toHaveBeenLastCalledWith('health_run');
    await api.healthLast();
    expect(call).toHaveBeenLastCalledWith('health_last');
    await api.updatesCheck();
    expect(call).toHaveBeenLastCalledWith('updates_check');
    await api.updatesInstall(INSTALL_REQUEST);
    expect(call).toHaveBeenLastCalledWith('updates_install', { input: INSTALL_REQUEST });
    await api.supportInfoSave(SUPPORT_REQUEST);
    expect(call).toHaveBeenLastCalledWith('support_info_save', { input: SUPPORT_REQUEST });
  });
});

describe('the fixtures are the backend’s own', () => {
  it('every b5 fixture is the same text as the backend’s file, and none is missing', () => {
    const backend = readdirSync(BACKEND_FIXTURES).filter((name) => name.startsWith('b5_'));
    const frontend = readdirSync(FRONTEND_FIXTURES).filter((name) => name.startsWith('b5_'));
    expect(frontend.toSorted()).toEqual(backend.toSorted());
    expect(backend.length).toBeGreaterThanOrEqual(29);
    for (const name of backend) {
      expect(
        readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8'),
        `${name} differs from the backend's file`,
      ).toBe(readFileSync(path.join(BACKEND_FIXTURES, name), 'utf8'));
    }
  });
});

/** The keys of an object, sorted, so a field added or dropped by the backend fails here. */
const keys = (value: object) => Object.keys(value).toSorted();

describe('the answers carry exactly the fields of the generated types', () => {
  it('AuditPage, AuditRow, AuditChange, AuditValue and the resolved filters', () => {
    expect(keys(AUDIT_FIRST_PAGE)).toEqual([
      'filters',
      'limit',
      'months',
      'offset',
      'rows',
      'total_count',
    ]);
    expect(keys(AUDIT_FIRST_PAGE.filters)).toEqual([
      'action',
      'entity',
      'entity_id',
      'from',
      'text',
      'to',
      'user',
    ]);
    const row = AUDIT_FIRST_PAGE.rows[0];
    expect(row).toBeDefined();
    if (row === undefined) return;
    expect(keys(row)).toEqual([
      'action',
      'at',
      'changes',
      'changes_truncated',
      'date',
      'entity',
      'entity_id',
      'id',
      'new_json',
      'old_json',
      'reason',
      'time',
      'user',
    ]);
    const change = row.changes[0];
    expect(change === undefined ? [] : keys(change)).toEqual(['new', 'old', 'path']);
    // The settings page of the backend carries a text value and a null (a field that is new).
    const kinds = AUDIT_SETTINGS.rows.flatMap((item) =>
      item.changes.flatMap((one) => [one.old.kind, one.new.kind]),
    );
    expect(new Set(kinds)).toEqual(new Set(['text', 'null', 'bool']));
  });

  it('AuditOptions: the users, the actions and the areas', () => {
    expect(keys(AUDIT_OPTIONS)).toEqual(['actions', 'entities', 'users']);
    expect(AUDIT_OPTIONS.users[0] === undefined ? [] : keys(AUDIT_OPTIONS.users[0])).toEqual([
      'display_name',
      'id',
      'username',
    ]);
  });

  it('DaySettings, DailyHealth, UpdateCheck, UpdateInstalled and SupportInfoSaved', () => {
    expect(keys(DAY_GET)).toEqual(['auto_day_close', 'closing_time']);
    expect(keys(HEALTH_RUN)).toEqual(['backup', 'books', 'checked_at', 'data', 'ok']);
    expect(keys(HEALTH_RUN.books)).toEqual([
      'audit_chain',
      'audit_rows',
      'balanced',
      'difference_paisa',
      'journal_chain',
      'journal_entries',
      'ran',
    ]);
    expect(keys(HEALTH_RUN.data)).toEqual(['reference', 'state']);
    expect(HEALTH_RUN.backup === null ? [] : keys(HEALTH_RUN.backup)).toEqual([
      'cloud_age_hours',
      'cloud_level',
      'errors',
      'last_cloud_at',
      'last_local_at',
      'last_usb_at',
      'queue_len',
    ]);
    expect(keys(UPDATE_AVAILABLE)).toEqual([
      'available',
      'current_version',
      'date',
      'drafts',
      'notes',
      'notes_en',
      'notes_ur',
      'version',
    ]);
    expect(keys(INSTALL_OK)).toEqual(['backup_file', 'version']);
    expect(keys(SUPPORT_SAVED)).toEqual(['log_lines', 'path', 'size_bytes']);
  });
});

describe('every message the screens need has words in both languages', () => {
  it('the seven codes of the update commands', () => {
    const codes: WireCode[] = [
      'ERR_UPDATE_CHECK_FAILED',
      'ERR_UPDATE_NOT_SIGNED',
      'ERR_UPDATE_DURING_WORK',
      'ERR_UPDATE_NOT_CONFIRMED',
      'ERR_UPDATE_BACKUP_FAILED',
      'ERR_UPDATE_CHANGED',
      'ERR_UPDATE_NONE',
    ];
    for (const language of LANGUAGES) {
      for (const code of codes) {
        expect(bundles[language].hasMessage(errorMessageId(code)), `${language} ${code}`).toBe(
          true,
        );
      }
    }
  });

  it('each action and area the audit viewer knows, and the lists of the real fixture', () => {
    for (const language of LANGUAGES) {
      for (const code of KNOWN_ACTIONS) {
        const id = `audit-action-${code.replaceAll('_', '-')}`;
        expect(bundles[language].hasMessage(id), `${language} ${id}`).toBe(true);
      }
      for (const code of KNOWN_ENTITIES) {
        const id = `audit-entity-${code.replaceAll('_', '-')}`;
        expect(bundles[language].hasMessage(id), `${language} ${id}`).toBe(true);
      }
    }
    // Everything the backend's own fixture lists is known, so the real log reads in words.
    for (const code of AUDIT_OPTIONS.actions)
      expect(KNOWN_ACTIONS as readonly string[]).toContain(code);
    for (const code of AUDIT_OPTIONS.entities)
      expect(KNOWN_ENTITIES as readonly string[]).toContain(code);
  });
});
