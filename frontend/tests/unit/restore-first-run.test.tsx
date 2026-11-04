// The first screen of a new install and the restore pages (plan 56, PRD FR-20.11, NFR-INS-2,
// NFR-INS-6), in Urdu and in English, against a fake backend. The answers of the restore commands are
// PROVISIONAL until backend3's contract is approved (tests/helpers/restore-first-run.ts).
//
// What is proved: the two choices and the language switch; a new shop still opens the wizard; the
// whole flow with the keyboard only (where, copy and key, check, restore, done, sign in); each
// refusal shown next to its field; an empty list; a copy that failed a check; a failed restore going
// back to the first screen with the reason and no key kept; no key ever drawn in plain form; nothing
// lost on a language switch; and the reminders after the first sign-in.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as authApi from '../../src/features/auth/api';
import * as backupApi from '../../src/features/backup/api';
import { useAfterRestoreStore } from '../../src/features/restore/after-restore-store';
import * as restoreApi from '../../src/features/restore/api';
import { useRestoreStore } from '../../src/features/restore/store';
import * as settingsApi from '../../src/features/settings/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as dialog from '../../src/shared/api/dialog';
import * as events from '../../src/shared/api/events';
import { utcToLocalDisplay } from '../../src/shared/format';
import {
  authState,
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
} from '../helpers/app';
import { fakeBackup } from '../helpers/backup';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import { fakeMasters } from '../helpers/masters';
import { message, paragraph, visible } from '../helpers/render';
import {
  BACKUP_KEY,
  CLOUD_TYPED,
  COPIES,
  DONE,
  GOOD_PREVIEW,
  NEWEST,
  OLDER,
  fakeRestore,
} from '../helpers/restore-first-run';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/restore/api');
vi.mock('../../src/features/backup/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/api/events');
vi.mock('../../src/shared/api/dialog');

const KEY_CHIPS = new Set(['F2', 'F9', 'F10', 'Enter', 'Esc']);

/** Matches a name that is exactly `text`, or `text` followed by the key chip of its button. */
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  if (seen === text) return true;
  return seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim());
};

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeRestore();
  fakeBackup();
  fakeAccounts([]);
  fakeAccountSearch([]);
  fakeMasters({});
  vi.mocked(events.listenEvent).mockResolvedValue(() => undefined);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('restore on a new computer (%s)', (language) => {
  const say = (id: string, args: Record<string, string | number> = {}) =>
    message(language, id, args);

  beforeEach(() => {
    resetClientState(language);
    act(() => {
      useRestoreStore.getState().reset();
      useAfterRestoreStore.getState().clear();
    });
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ first_run: true, login_language: language }),
    );
  });

  const button = (id: string) => screen.getByRole('button', { name: named(say(id)) });
  const field = (id: string) =>
    screen.getByLabelText<HTMLInputElement>((text) => visible(text) === say(id));

  async function start() {
    await renderApp();
    await screen.findByRole('heading', { name: say('restore-start-title') });
    return userEvent.setup();
  }

  /** From the first screen to the "where" page, by keyboard: Tab to the second card, Enter. */
  async function openRestore(user: ReturnType<typeof userEvent.setup>) {
    await user.tab();
    await user.keyboard('{Enter}');
    await screen.findByRole('heading', { name: say('restore-where-title') });
  }

  /** The cloud page filled by keyboard: Enter moves on, and keeps the usual folder and region. */
  async function typeCloud(
    user: ReturnType<typeof userEvent.setup>,
    endpoint: string = CLOUD_TYPED.endpoint,
  ) {
    await user.keyboard(`${endpoint}{Enter}`);
    await user.keyboard(`${CLOUD_TYPED.bucket}{Enter}`);
    await user.keyboard('{Enter}'); // the folder: mandi
    await user.keyboard(`${CLOUD_TYPED.key_id}{Enter}`);
    await user.keyboard(`${CLOUD_TYPED.secret}{Enter}`);
    await user.keyboard('{Enter}'); // the region: auto. Enter on the last field looks for the copies.
  }

  async function toCopyPage(user: ReturnType<typeof userEvent.setup>) {
    await openRestore(user);
    await typeCloud(user);
    await screen.findByRole('heading', { name: say('restore-copy-title') });
  }

  async function toCheckPage(user: ReturnType<typeof userEvent.setup>) {
    await toCopyPage(user);
    await user.keyboard(`${BACKUP_KEY}{Enter}`);
    await screen.findByRole('heading', { name: say('restore-check-title') });
  }

  describe('the first screen', () => {
    it('shows the two choices and the language switch, with the first choice focused', async () => {
      await start();
      expect(
        screen.getByRole('button', {
          name: new RegExp(say('restore-start-new-title')),
        }),
      ).toHaveFocus();
      expect(
        screen.getByRole('button', { name: new RegExp(say('restore-start-restore-title')) }),
      ).toBeInTheDocument();
      expect(
        screen.getByRole('group', { name: say('common-language-switch') }),
      ).toBeInTheDocument();
      expect(document.documentElement.dir).toBe(language === 'ur' ? 'rtl' : 'ltr');
    });

    it('a new shop opens the wizard as it was, and Back returns to the first screen', async () => {
      const user = await start();
      await user.keyboard('{Enter}');
      await screen.findByRole('heading', { name: say('auth-wizard-firm-title') });
      await user.click(button('common-back'));
      await screen.findByRole('heading', { name: say('restore-start-title') });
    });
  });

  describe('the whole flow, with the keyboard only', () => {
    it('where, copy and key, check, restore, done, then the sign-in screen', async () => {
      const user = await start();
      // A restore that works: after it, the app has a data file and the sign-in shows.
      vi.mocked(restoreApi.backupFirstRunRestore).mockImplementation(() => {
        vi.mocked(authApi.authStatus).mockResolvedValue(
          authState({ first_run: false, login_language: language }),
        );
        return Promise.resolve(DONE);
      });
      await toCopyPage(user);
      expect(restoreApi.backupFirstRunList).toHaveBeenCalledWith({
        source: { cloud: CLOUD_TYPED },
      });

      // The newest copy is chosen, and the key field has focus.
      expect(
        screen.getByText(
          paragraph(say('restore-copy-chosen', { date: utcToLocalDisplay(NEWEST.created_at) })),
        ),
      ).toBeInTheDocument();
      const key = field('backup-restore-key-label');
      expect(key).toHaveFocus();
      await user.keyboard(`${BACKUP_KEY}{Enter}`);
      expect(restoreApi.backupFirstRunPreview).toHaveBeenCalledWith({
        source: { cloud: CLOUD_TYPED },
        copy: NEWEST.name,
        key: BACKUP_KEY,
      });

      // The checks and the books of the copy, before anything is put on this computer.
      await screen.findByRole('heading', { name: say('restore-check-title') });
      const checks = screen.getByRole('list', { name: say('restore-check-list-label') });
      expect(within(checks).getAllByRole('listitem')).toHaveLength(6);
      expect(screen.getByText(say('restore-check-ok'))).toBeInTheDocument();
      expect(restoreApi.backupFirstRunRestore).not.toHaveBeenCalled();

      // Restore asks once more; Enter on the question restores.
      await user.keyboard('{Enter}');
      await screen.findByRole('alertdialog');
      await user.keyboard('{Enter}');
      await screen.findByRole('heading', { name: say('restore-done-title') });
      expect(restoreApi.backupFirstRunRestore).toHaveBeenCalledWith({
        source: { cloud: CLOUD_TYPED },
        copy: NEWEST.name,
        key: BACKUP_KEY,
      });
      // The keys are dropped the moment the restore is over.
      expect(useRestoreStore.getState().backupKey).toBe('');
      expect(useRestoreStore.getState().cloud.secret).toBe('');
      expect(useRestoreStore.getState().cloud.key_id).toBe('');

      // The sign-in button has focus: Enter carries on to the sign-in screen.
      expect(button('restore-done-signin')).toHaveFocus();
      await user.keyboard('{Enter}');
      await screen.findByRole('heading', { name: say('auth-login-title') });
    });
  });

  describe('the where page', () => {
    it('asks for the cloud fields with the words of Settings, and masks both keys', async () => {
      const user = await start();
      await openRestore(user);
      for (const id of [
        'backup-set-endpoint',
        'backup-set-bucket',
        'backup-set-prefix',
        'backup-set-region',
        'backup-set-key-id',
        'backup-set-secret',
      ]) {
        expect(field(id)).toBeInTheDocument();
      }
      expect(field('backup-set-prefix')).toHaveValue('mandi');
      expect(field('backup-set-region')).toHaveValue('auto');
      // Both keys are dots while typed, and there is no eye to show them.
      expect(field('backup-set-key-id')).toHaveAttribute('type', 'password');
      expect(field('backup-set-secret')).toHaveAttribute('type', 'password');
      expect(screen.queryByRole('button', { name: say('common-password-show') })).toBeNull();
    });

    it('does not ask the backend while a required field is empty, and says which', async () => {
      const user = await start();
      await openRestore(user);
      await user.click(button('restore-where-find'));
      expect(restoreApi.backupFirstRunList).not.toHaveBeenCalled();
      expect(screen.getAllByText(say('err-required'))).toHaveLength(4);
    });

    it('shows each refusal next to the field it concerns and changes nothing', async () => {
      vi.mocked(restoreApi.backupFirstRunList).mockRejectedValue(
        fail(
          { code: 'ERR_REQUIRED', field: 'source.cloud.bucket' },
          { code: 'ERR_REQUIRED', field: 'source.cloud.secret' },
        ),
      );
      const user = await start();
      await openRestore(user);
      await typeCloud(user);
      await waitFor(() => {
        expect(field('backup-set-bucket')).toHaveAttribute('aria-invalid', 'true');
      });
      expect(field('backup-set-secret')).toHaveAttribute('aria-invalid', 'true');
      expect(field('backup-set-endpoint')).not.toHaveAttribute('aria-invalid', 'true');
      // Still on the first page, with what was typed kept.
      expect(field('backup-set-endpoint')).toHaveValue(CLOUD_TYPED.endpoint);
      expect(screen.queryByRole('heading', { name: say('restore-copy-title') })).toBeNull();
    });

    it('takes a USB drive or folder, or one file: the path is typed or chosen, and sent as that', async () => {
      vi.mocked(dialog.chooseOpenPath).mockResolvedValue('E:\\MandiBackup');
      const user = await start();
      await openRestore(user);
      const place = (id: string) => screen.getByRole('button', { pressed: false, name: say(id) });
      await user.click(place('restore-where-folder'));
      await user.click(button('restore-where-choose-folder'));
      await waitFor(() => {
        expect(field('restore-where-folder-path')).toHaveValue('E:\\MandiBackup');
      });
      expect(dialog.chooseOpenPath).toHaveBeenCalledWith(
        expect.objectContaining({ directory: true }),
      );
      await user.click(button('restore-where-find'));
      await screen.findByRole('heading', { name: say('restore-copy-title') });
      expect(restoreApi.backupFirstRunList).toHaveBeenCalledWith({
        source: { folder: 'E:\\MandiBackup' },
      });
      // One file: its own path field, sent as a file. The folder typed before is not kept.
      await user.click(button('common-back'));
      await user.click(place('restore-where-file'));
      expect(field('restore-where-file-path')).toHaveValue('');
      await user.type(field('restore-where-file-path'), 'E:\\MandiBackup\\one.mbk{Enter}');
      await screen.findByRole('heading', { name: say('restore-copy-title') });
      expect(restoreApi.backupFirstRunList).toHaveBeenLastCalledWith({
        source: { file: 'E:\\MandiBackup\\one.mbk' },
      });
    });

    it('holds back a cloud address that is not https, with a hint at the field', async () => {
      const user = await start();
      await openRestore(user);
      await user.keyboard(
        'http://shop.example{Enter}bucket{Enter}{Enter}AKIA{Enter}secret{Enter}{Enter}',
      );
      expect(await screen.findByText(say('restore-where-endpoint-https'))).toBeInTheDocument();
      expect(restoreApi.backupFirstRunList).not.toHaveBeenCalled();
    });

    it('says why the backend refused the cloud address: not https, then the bucket written into it', async () => {
      vi.mocked(restoreApi.backupFirstRunList)
        .mockRejectedValueOnce(
          fail({
            code: 'ERR_OUT_OF_RANGE',
            field: 'source.cloud.endpoint',
            values: { reason: 'https' },
          }),
        )
        .mockRejectedValueOnce(
          fail({
            code: 'ERR_OUT_OF_RANGE',
            field: 'source.cloud.endpoint',
            values: { reason: 'path' },
          }),
        );
      const user = await start();
      await openRestore(user);
      // "https://" alone passes the quick check on this page; the backend says it has no host.
      await typeCloud(user, 'https://');
      expect(await screen.findByText(say('err-out-of-range', { reason: 'https' }))).toBeVisible();
      expect(field('backup-set-endpoint')).toHaveAttribute('aria-invalid', 'true');
      expect(field('backup-set-endpoint')).toHaveValue('https://');
      expect(screen.queryByRole('heading', { name: say('restore-copy-title') })).toBeNull();

      // Editing the address takes its old message away, because it was about the old text.
      await user.clear(field('backup-set-endpoint'));
      await user.type(
        field('backup-set-endpoint'),
        'https://shop.r2.cloudflarestorage.com/mandi-bucket',
      );
      expect(screen.queryByText(say('err-out-of-range', { reason: 'https' }))).toBeNull();
      expect(field('backup-set-endpoint')).not.toHaveAttribute('aria-invalid', 'true');

      // The bucket name inside the address: the message names the field it belongs in.
      await user.click(button('restore-where-find'));
      const pathMessage = say('err-out-of-range', { reason: 'path' });
      // Fluent puts invisible isolation marks around the bucket field's name; `visible` drops them.
      expect(await screen.findByText((text) => visible(text) === pathMessage)).toBeVisible();
      expect(pathMessage).toContain(say('backup-set-bucket'));
      expect(field('backup-set-endpoint')).toHaveAttribute('aria-invalid', 'true');
      // The other fields keep what was typed, and the keys stay masked.
      expect(field('backup-set-bucket')).toHaveValue(CLOUD_TYPED.bucket);
      expect(field('backup-set-secret')).toHaveAttribute('type', 'password');

      // Writing the address alone works: the copies are listed.
      await user.clear(field('backup-set-endpoint'));
      await user.type(field('backup-set-endpoint'), CLOUD_TYPED.endpoint);
      await user.click(button('restore-where-find'));
      await screen.findByRole('heading', { name: say('restore-copy-title') });
      expect(restoreApi.backupFirstRunList).toHaveBeenCalledTimes(3);
    });

    it('takes away the message of a path or a key field when it is edited, and keeps the others', async () => {
      vi.mocked(restoreApi.backupFirstRunList).mockRejectedValueOnce(
        fail(
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
        ),
      );
      const user = await start();
      await openRestore(user);
      await typeCloud(user);
      await waitFor(() => {
        expect(field('backup-set-key-id')).toHaveAttribute('aria-invalid', 'true');
      });
      expect(field('backup-set-secret')).toHaveAttribute('aria-invalid', 'true');
      await user.type(field('backup-set-key-id'), 'X');
      expect(field('backup-set-key-id')).not.toHaveAttribute('aria-invalid', 'true');
      expect(field('backup-set-secret')).toHaveAttribute('aria-invalid', 'true');
    });

    it('shows that it is looking while the backend works, and cannot be pressed twice', async () => {
      let answer: (list: typeof COPIES) => void = () => undefined;
      vi.mocked(restoreApi.backupFirstRunList).mockReturnValue(
        new Promise((resolve) => {
          answer = resolve;
        }),
      );
      const user = await start();
      await openRestore(user);
      await typeCloud(user);
      expect(await screen.findByText(say('restore-where-finding'))).toBeInTheDocument();
      await user.click(button('restore-where-find'));
      expect(restoreApi.backupFirstRunList).toHaveBeenCalledTimes(1);
      answer(COPIES);
      await screen.findByRole('heading', { name: say('restore-copy-title') });
    });
  });

  describe('the copy and key page', () => {
    it('lists the copies, newest first, and a click chooses an older one', async () => {
      const user = await start();
      await toCopyPage(user);
      const rows = document.querySelectorAll('.ag-row');
      expect(rows).toHaveLength(2);
      expect(rows[0]).toHaveAttribute('row-id', NEWEST.name);
      const older = document.querySelector(`.ag-row[row-id="${OLDER.name}"] [col-id="created"]`);
      if (older === null) throw new Error('no older row');
      await user.click(older);
      await waitFor(() => {
        expect(useRestoreStore.getState().copyName).toBe(OLDER.name);
      });
    });

    it('masks the backup key, and a wrong key is shown at its field so he can try again', async () => {
      vi.mocked(restoreApi.backupFirstRunPreview).mockRejectedValueOnce(
        fail({ code: 'ERR_BACKUP_KEY_WRONG', field: 'key' }),
      );
      const user = await start();
      await toCopyPage(user);
      const key = field('backup-restore-key-label');
      expect(key).toHaveAttribute('type', 'password');
      await user.keyboard(`${BACKUP_KEY}{Enter}`);
      expect(await screen.findByText(say('err-backup-key-wrong'))).toBeInTheDocument();
      expect(key).toHaveAttribute('aria-invalid', 'true');
      // Still here, and the second try works.
      await user.clear(key);
      await user.keyboard(`${BACKUP_KEY}{Enter}`);
      await screen.findByRole('heading', { name: say('restore-check-title') });
    });

    it('has an eye to check what was typed from the paper, off by default, and only on the backup key', async () => {
      const user = await start();
      await toCopyPage(user);
      const key = field('backup-restore-key-label');
      await user.keyboard(BACKUP_KEY);
      expect(key).toHaveAttribute('type', 'password');
      await user.click(screen.getByRole('button', { name: say('common-password-show') }));
      expect(key).toHaveAttribute('type', 'text');
      expect(key).toHaveValue(BACKUP_KEY);
      await user.click(screen.getByRole('button', { name: say('common-password-hide') }));
      expect(key).toHaveAttribute('type', 'password');
      // The cloud keys on the first page have no eye at all.
      await user.keyboard('{Escape}');
      await screen.findByRole('heading', { name: say('restore-where-title') });
      expect(screen.queryByRole('button', { name: say('common-password-show') })).toBeNull();
    });

    it('a damaged copy is refused in plain words, he stays on the page and can choose another', async () => {
      vi.mocked(restoreApi.backupFirstRunPreview).mockRejectedValueOnce(
        fail({ code: 'ERR_BACKUP_CORRUPT', field: 'copy', values: { check: 'checksum' } }),
      );
      const user = await start();
      await toCopyPage(user);
      await user.keyboard(`${BACKUP_KEY}{Enter}`);
      expect(await screen.findByText(say('err-backup-corrupt'))).toBeInTheDocument();
      expect(screen.getByRole('heading', { name: say('restore-copy-title') })).toBeInTheDocument();
      expect(useRestoreStore.getState().backupKey).toBe(BACKUP_KEY);
    });

    it('says plainly when no copy was found, and a button goes back to change the place', async () => {
      vi.mocked(restoreApi.backupFirstRunList).mockResolvedValue({ rows: [], total_count: 0 });
      const user = await start();
      await openRestore(user);
      await typeCloud(user);
      await screen.findByRole('heading', { name: say('restore-copy-empty') });
      await user.click(button('restore-copy-change-place'));
      await screen.findByRole('heading', { name: say('restore-where-title') });
    });
  });

  describe('the check page', () => {
    it('shows the date, the checks and the books of the copy, and nothing is restored yet', async () => {
      const user = await start();
      await toCheckPage(user);
      expect(
        screen.getByText(
          paragraph(
            say('backup-restore-made-on', { date: utcToLocalDisplay(GOOD_PREVIEW.created_at) }),
          ),
        ),
      ).toBeInTheDocument();
      const table = screen.getByRole('table', { name: say('restore-totals-label') });
      expect(within(table).getByText('2413')).toBeInTheDocument();
      expect(restoreApi.backupFirstRunRestore).not.toHaveBeenCalled();
    });

    it('a copy with a failed check (the backend refuses these, so this is a safety net) cannot be restored', async () => {
      vi.mocked(restoreApi.backupFirstRunPreview).mockResolvedValue({
        ...GOOD_PREVIEW,
        checks: GOOD_PREVIEW.checks.map((check) =>
          check.name === 'balanced' ? { ...check, ok: false } : check,
        ),
      });
      const user = await start();
      await toCheckPage(user);
      expect(screen.getByText(say('restore-check-bad'))).toBeInTheDocument();
      expect(screen.getByText(say('backup-check-balanced'))).toBeInTheDocument();
      expect(
        screen.queryByRole('button', { name: named(say('backup-restore-button')) }),
      ).toBeNull();
      await user.click(button('common-back'));
      await screen.findByRole('heading', { name: say('restore-copy-title') });
    });

    it('names the tamper check among the checks', async () => {
      const user = await start();
      await toCheckPage(user);
      expect(screen.getByText(say('backup-check-chains'))).toBeInTheDocument();
    });

    it('leaving the restore pages for the first screen drops the three keys', async () => {
      const user = await start();
      await toCheckPage(user);
      expect(useRestoreStore.getState().backupKey).toBe(BACKUP_KEY);
      await user.keyboard('{Escape}{Escape}{Escape}');
      await screen.findByRole('heading', { name: say('restore-start-title') });
      const state = useRestoreStore.getState();
      expect(state.backupKey).toBe('');
      expect(state.cloud.key_id).toBe('');
      expect(state.cloud.secret).toBe('');
      // What is not secret stays, so coming back costs little typing.
      expect(state.cloud.endpoint).toBe(CLOUD_TYPED.endpoint);
    });

    it('Esc goes back one page at a time', async () => {
      const user = await start();
      await toCheckPage(user);
      await user.keyboard('{Escape}');
      await screen.findByRole('heading', { name: say('restore-copy-title') });
      await user.keyboard('{Escape}');
      await screen.findByRole('heading', { name: say('restore-where-title') });
      await user.keyboard('{Escape}');
      await screen.findByRole('heading', { name: say('restore-start-title') });
    });
  });

  describe('the restore itself', () => {
    it('shows the stages and the progress while the books are put in place', async () => {
      let send: (payload: unknown) => void = () => undefined;
      vi.mocked(events.listenEvent).mockImplementation((_name, handler) => {
        send = handler;
        return Promise.resolve(() => undefined);
      });
      let finish: (value: typeof DONE) => void = () => undefined;
      vi.mocked(restoreApi.backupFirstRunRestore).mockReturnValue(
        new Promise((resolve) => {
          finish = resolve;
        }),
      );
      const user = await start();
      await toCheckPage(user);
      await user.keyboard('{Enter}');
      await screen.findByRole('alertdialog');
      await user.keyboard('{Enter}');
      await screen.findByRole('heading', { name: say('restore-running-title') });
      act(() => {
        send({ stage: 'putting_in_place', percent: 40 });
      });
      await waitFor(() => {
        expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '40');
      });
      expect(screen.getByText(say('restore-stage-putting-in-place'))).toBeInTheDocument();
      // A payload it does not know is ignored.
      act(() => {
        send({ stage: 'something_else', percent: 10 });
      });
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '40');
      await act(async () => {
        finish(DONE);
        await Promise.resolve();
      });
      await screen.findByRole('heading', { name: say('restore-done-title') });
    });

    it('a failure goes back to the first screen with the reason, and no key is kept', async () => {
      vi.mocked(restoreApi.backupFirstRunRestore).mockRejectedValue(fail({ code: 'ERR_INTERNAL' }));
      const user = await start();
      await toCheckPage(user);
      await user.keyboard('{Enter}');
      await screen.findByRole('alertdialog');
      await user.keyboard('{Enter}');
      await screen.findByRole('heading', { name: say('restore-start-title') });
      expect(screen.getByText(say('restore-start-failed-title'))).toBeInTheDocument();
      expect(screen.getByText(say('err-internal'))).toBeInTheDocument();
      const state = useRestoreStore.getState();
      expect(state.backupKey).toBe('');
      expect(state.cloud.secret).toBe('');
      expect(state.cloud.key_id).toBe('');
      expect(state.choice).toBe('start');
    });

    it('the done page tells about the cloud keys only when a file or folder was used', async () => {
      vi.mocked(restoreApi.backupFirstRunRestore).mockResolvedValue({
        ...DONE,
        cloud_needs_keys: true,
      });
      const user = await start();
      await toCheckPage(user);
      await user.keyboard('{Enter}');
      await screen.findByRole('alertdialog');
      await user.keyboard('{Enter}');
      await screen.findByRole('heading', { name: say('restore-done-title') });
      expect(screen.getByText(say('restore-done-note-cloud-keys'))).toBeInTheDocument();
      expect(useAfterRestoreStore.getState().pending).toEqual({
        assistantWasOn: true,
        cloudNeedsKeys: true,
      });
    });

    it('the done page tells about the OpenRouter key only when the assistant was on', async () => {
      vi.mocked(restoreApi.backupFirstRunRestore).mockResolvedValue({
        ...DONE,
        assistant_was_on: false,
      });
      const user = await start();
      await toCheckPage(user);
      await user.keyboard('{Enter}');
      await screen.findByRole('alertdialog');
      await user.keyboard('{Enter}');
      await screen.findByRole('heading', { name: say('restore-done-title') });
      expect(screen.getByText(say('restore-done-note-backup'))).toBeInTheDocument();
      expect(screen.queryByText(say('restore-done-note-openrouter'))).toBeNull();
    });
  });

  describe('nothing is lost on a language switch', () => {
    it('what was typed stays, in the middle of the where page', async () => {
      const user = await start();
      await openRestore(user);
      await user.keyboard(`${CLOUD_TYPED.endpoint}{Enter}${CLOUD_TYPED.bucket}`);
      const other: Language = language === 'ur' ? 'en' : 'ur';
      await user.click(
        screen.getByRole('button', { name: message(other, 'common-language-' + other) }),
      );
      await waitFor(() => {
        expect(
          screen.getByRole('heading', { name: message(other, 'restore-where-title') }),
        ).toBeInTheDocument();
      });
      expect(
        screen.getByLabelText<HTMLInputElement>(
          (text) => visible(text) === message(other, 'backup-set-endpoint'),
        ),
      ).toHaveValue(CLOUD_TYPED.endpoint);
      expect(
        screen.getByLabelText<HTMLInputElement>(
          (text) => visible(text) === message(other, 'backup-set-bucket'),
        ),
      ).toHaveValue(CLOUD_TYPED.bucket);
      expect(document.documentElement.dir).toBe(other === 'ur' ? 'rtl' : 'ltr');
    });

    it('the chosen copy and the typed key stay, in the middle of the copy page', async () => {
      const user = await start();
      await toCopyPage(user);
      await user.keyboard('ABCD-EFGH');
      const other: Language = language === 'ur' ? 'en' : 'ur';
      await user.click(
        screen.getByRole('button', { name: message(other, 'common-language-' + other) }),
      );
      await waitFor(() => {
        expect(
          screen.getByRole('heading', { name: message(other, 'restore-copy-title') }),
        ).toBeInTheDocument();
      });
      expect(useRestoreStore.getState().backupKey).toBe('ABCD-EFGH');
      expect(useRestoreStore.getState().copyName).toBe(NEWEST.name);
    });
  });
});

describe.each(LANGUAGES)('after the first sign-in on a restored computer (%s)', (language) => {
  const say = (id: string) => message(language, id);

  beforeEach(() => {
    resetClientState(language);
    act(() => {
      useRestoreStore.getState().reset();
      useAfterRestoreStore.getState().clear();
    });
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ session: sessionUser({ language }), login_language: language }),
    );
    vi.mocked(settingsApi.prefsSet).mockResolvedValue(null);
  });

  it('offers a new backup, and the OpenRouter key when the assistant was on', async () => {
    act(() => {
      useAfterRestoreStore.getState().remember({ assistantWasOn: true, cloudNeedsKeys: false });
    });
    await renderApp('/');
    expect(await screen.findByText(say('restore-after-title'))).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: say('restore-after-openrouter') }),
    ).toBeInTheDocument();
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: say('restore-after-backup') }));
    await waitFor(() => {
      expect(backupApi.backupNow).toHaveBeenCalledWith({ local: true, usb: false, cloud: true });
    });
    await user.click(screen.getByRole('button', { name: named(say('restore-after-close')) }));
    await waitFor(() => {
      expect(screen.queryByText(say('restore-after-title'))).toBeNull();
    });
  });

  it('offers the cloud keys in Backup when a file or folder was used', async () => {
    act(() => {
      useAfterRestoreStore.getState().remember({ assistantWasOn: false, cloudNeedsKeys: true });
    });
    const { router } = await renderApp('/');
    const user = userEvent.setup();
    await user.click(await screen.findByRole('button', { name: say('restore-after-cloud-keys') }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/backup');
    });
  });

  it('does not offer the OpenRouter key when the assistant was off, and shows nothing otherwise', async () => {
    act(() => {
      useAfterRestoreStore.getState().remember({ assistantWasOn: false, cloudNeedsKeys: false });
    });
    await renderApp('/');
    expect(await screen.findByText(say('restore-after-title'))).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: say('restore-after-openrouter') })).toBeNull();
  });

  it('is not shown on an ordinary sign-in', async () => {
    await renderApp('/');
    await waitFor(() => {
      expect(appApi.dayStatus).toHaveBeenCalled();
    });
    expect(screen.queryByText(say('restore-after-title'))).toBeNull();
  });
});
