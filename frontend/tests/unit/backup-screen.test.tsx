// بیک اپ (plan 18, PRD Section 20, FR-20.8, FR-20.9, NFR-BAK-7, NFR-BAK-11), in the Urdu and in the
// English app, against a fake backend that answers with the backend's own fixtures: the state of
// the backups, the list of copies (page by page, four states), "back up now" and what it did, the
// restore with its preview and its confirm box, a key asked for on a new computer, the test restore,
// the backup key, the settings, a language switch in the middle of an entry, and the keyboard-only
// path. Every time, size and total on the screen is a figure the backend sent.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as authApi from '../../src/features/auth/api';
import * as backupApi from '../../src/features/backup/api';
import { useBackupKeyStore } from '../../src/features/backup/key-store';
import { LANGUAGES, type Language } from '../../src/i18n';
import { utcToLocalDisplay } from '../../src/shared/format';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  KEY_CONFIRM_WRONG,
  KEY_SETUP,
  LIST_ALL,
  NOW_QUEUED,
  PREVIEW,
  SETTINGS_GET,
  SETTINGS_REFUSED,
  STATUS_NEW,
  TEST_RESULT,
  fakeBackup,
  refused,
} from '../helpers/backup';
import {
  authState,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, paragraph, visible } from '../helpers/render';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/backup/api');

const LOCAL = 'local:mandi-local-20261010T090100000Z.db';
const USB = 'usb:mandi-20261010T090100000Z.mbk';
const CLOUD = 'cloud:mandi-20261010T090100000Z.mbk';
const MADE = utcToLocalDisplay('2026-10-10T09:01:00.000Z');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  useBackupKeyStore.getState().clear();
});

afterEach(() => {
  leaveAppWindow();
});

/** The keys a button shows after its name (PRD FR-7.4); the name is the words before them. */
const KEY_CHIPS = new Set(['F2', 'F9', 'F10', 'Enter', 'Esc']);

/** Matches a name that is exactly `text`, or `text` followed by the key chip of its button. */
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  if (seen === text) return true;
  return seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim());
};

/** Waits until the text of `container` holds `text` (a message with markup is several elements). */
async function shows(container: HTMLElement, text: string) {
  await waitFor(() => {
    expect(visible(container.textContent)).toContain(text);
  });
}

function cell(rowId: string, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function cellElement(rowId: string, colId: string): HTMLElement {
  const found = document.querySelector<HTMLElement>(
    `.ag-row[row-id="${rowId}"] [col-id="${colId}"]`,
  );
  if (found === null) throw new Error(`no cell ${rowId}:${colId}`);
  return found;
}

/** The words of a size, the way the list writes them. */
const sizeWords = (language: Language, value: string, unit: 'bytes' | 'kb' | 'mb' | 'gb') =>
  message(language, 'backup-size-value', {
    value,
    unit: message(language, `backup-size-${unit}`),
  });

describe.each(LANGUAGES)('the backup screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open(options: Parameters<typeof fakeBackup>[0] = {}) {
    const store = fakeBackup(options);
    const view = await renderApp('/backup');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'backup-title')),
    });
    return { ...view, store, user: userEvent.setup() };
  }

  const button = (id: string) => screen.getByRole('button', { name: named(message(language, id)) });
  const buttons = (id: string) =>
    screen
      .getAllByRole('button')
      .filter((item) => visible(item.textContent).includes(message(language, id)));

  describe('the state and the list', () => {
    it('shows the last copies and the rows the backend sent, asking for the first page of 25', async () => {
      await open();
      await waitFor(() => {
        expect(cell(LOCAL, 'kind')).toBe(message(language, 'backup-kind-local'));
      });
      expect(backupApi.backupList).toHaveBeenCalledWith({
        kind: null,
        offset: 0,
        limit: PAGE_ROWS,
      });
      expect(cell(LOCAL, 'created')).toBe(MADE);
      expect(cell(USB, 'kind')).toBe(message(language, 'backup-kind-usb'));
      expect(cell(CLOUD, 'kind')).toBe(message(language, 'backup-kind-cloud'));
      expect(cell(LOCAL, 'status')).toBe(message(language, 'backup-status-good'));
      expect(cell(LOCAL, 'size')).toBe(sizeWords(language, '0', 'bytes'));
      // Only a copy locked with the key says so.
      expect(cell(LOCAL, 'locked')).toBe(message(language, 'backup-locked-no'));
      expect(cell(CLOUD, 'locked')).toBe(message(language, 'backup-locked-yes'));
      expect(screen.getByText(paragraph(rangeText(language, 1, 3, 3)))).toBeInTheDocument();
      // The state card: the time of each last copy comes from the backend's state.
      const state = screen
        .getByText(named(message(language, 'backup-state-title')))
        .closest('section');
      expect(visible(state?.textContent)).toContain(MADE);
      expect(visible(state?.textContent)).toContain(message(language, 'backup-state-cloud-fresh'));
      // The pack and the test restore are still to do, and the screen says so.
      expect(visible(state?.textContent)).toContain(message(language, 'backup-state-pack-pending'));
      expect(visible(state?.textContent)).toContain(message(language, 'backup-state-test-due'));
    });

    it('a place can be chosen, and the choice is sent to the backend', async () => {
      const { user } = await open();
      await waitFor(() => {
        expect(cell(LOCAL, 'kind')).not.toBe('');
      });
      const filter = screen.getByRole('group', {
        name: named(message(language, 'backup-filter-label')),
      });
      await user.click(
        within(filter).getByRole('button', {
          name: named(message(language, 'backup-filter-cloud')),
        }),
      );
      await waitFor(() => {
        expect(backupApi.backupList).toHaveBeenLastCalledWith({
          kind: 'cloud',
          offset: 0,
          limit: PAGE_ROWS,
        });
      });
      await waitFor(() => {
        expect(document.querySelector(`.ag-row[row-id="${LOCAL}"]`)).toBeNull();
      });
      expect(cell(CLOUD, 'kind')).toBe(message(language, 'backup-kind-cloud'));
    });

    it('a cloud that could not be reached is said in words, not hidden', async () => {
      await open({ cloudReachable: false });
      expect(
        await screen.findByText(named(message(language, 'backup-list-cloud-unreachable'))),
      ).toBeInTheDocument();
    });

    it('a copy that failed its check is written as failed, and the state shows the problem', async () => {
      const bad = LIST_ALL.rows.map((row) =>
        row.kind === 'usb' ? { ...row, status: 'bad' as const } : row,
      );
      await open({
        rows: bad,
        status: {
          errors: [
            {
              kind: 'usb',
              code: 'ERR_BACKUP_CHECK_FAILED',
              at: '2026-10-10T09:02:00.000Z',
              check_failed: true,
              file: 'mandi-20261010T090100000Z.mbk',
              reason: null,
            },
          ],
        },
      });
      await waitFor(() => {
        expect(cell(USB, 'status')).toBe(message(language, 'backup-status-bad'));
      });
      expect(
        await screen.findByText(named(message(language, 'err-backup-check-failed'))),
      ).toBeInTheDocument();
    });

    it('an empty list says why and has a button that makes the first copy', async () => {
      const { user } = await open({ rows: [] });
      expect(
        await screen.findByText(named(message(language, 'backup-list-empty'))),
      ).toBeInTheDocument();
      const all = buttons('backup-now');
      await user.click(all[all.length - 1] ?? document.body);
      await waitFor(() => {
        expect(backupApi.backupNow).toHaveBeenCalledWith({ local: true, usb: true, cloud: true });
      });
    });

    it('a list that cannot be read says so with a retry that works', async () => {
      fakeBackup();
      vi.mocked(backupApi.backupList).mockRejectedValueOnce(refused({ code: 'ERR_INTERNAL' }));
      await renderApp('/backup');
      const alert = await screen.findByRole('alert');
      expect(visible(alert.textContent)).toContain(message(language, 'backup-list-failed'));
      expect(visible(alert.textContent)).toContain(message(language, 'err-internal'));
      await userEvent
        .setup()
        .click(within(alert).getByRole('button', { name: message(language, 'common-retry') }));
      await waitFor(() => {
        expect(cell(LOCAL, 'kind')).toBe(message(language, 'backup-kind-local'));
      });
    });

    it('a state that cannot be read says so with a retry that works', async () => {
      fakeBackup();
      vi.mocked(backupApi.backupStatus).mockRejectedValueOnce(refused({ code: 'ERR_INTERNAL' }));
      await renderApp('/backup');
      const alert = await screen.findByRole('alert');
      expect(visible(alert.textContent)).toContain(message(language, 'backup-load-failed'));
      await userEvent
        .setup()
        .click(within(alert).getByRole('button', { name: message(language, 'common-retry') }));
      expect(
        await screen.findByRole('heading', {
          level: 1,
          name: named(message(language, 'backup-title')),
        }),
      ).toBeInTheDocument();
    });

    it('turns the list page by page and says which rows are shown', async () => {
      const template = LIST_ALL.rows[0];
      if (template === undefined) throw new Error('no fixture row');
      const many = Array.from({ length: 120 }, (_, index) => ({
        ...template,
        name: `mandi-local-${String(index).padStart(4, '0')}.db`,
      }));
      const { user } = await open({ rows: many });
      await waitFor(() => {
        expect(
          screen.getByText(paragraph(rangeText(language, 1, PAGE_ROWS, 120))),
        ).toBeInTheDocument();
      });
      await user.click(await pagerButton(language, 'next'));
      await waitFor(() => {
        expect(
          screen.getByText(paragraph(rangeText(language, PAGE_ROWS + 1, 2 * PAGE_ROWS, 120))),
        ).toBeInTheDocument();
      });
      expect(backupApi.backupList).toHaveBeenLastCalledWith({
        kind: null,
        offset: PAGE_ROWS,
        limit: PAGE_ROWS,
      });
    });
  });

  describe('back up now', () => {
    it('asks for a copy in every place that is on, and tells what each place did', async () => {
      const { user } = await open();
      await user.click(button('backup-now'));
      await waitFor(() => {
        expect(backupApi.backupNow).toHaveBeenCalledWith({ local: true, usb: true, cloud: true });
      });
      expect(
        (await screen.findAllByText(named(message(language, 'backup-now-done')))).length,
      ).toBeGreaterThan(0);
      // One line for each place.
      const lines = await screen.findAllByText(named(message(language, 'backup-run-ok')));
      expect(lines.length).toBe(3);
    });

    it('does not ask for a place that is switched off', async () => {
      const { user } = await open({ status: { usb_enabled: false, cloud_enabled: false } });
      await user.click(button('backup-now'));
      await waitFor(() => {
        expect(backupApi.backupNow).toHaveBeenCalledWith({ local: true, usb: false, cloud: false });
      });
    });

    it('a cloud copy that waits for the internet says so, and is not an error', async () => {
      const { user } = await open();
      vi.mocked(backupApi.backupNow).mockResolvedValueOnce(NOW_QUEUED);
      await user.click(button('backup-now'));
      const waiting = await screen.findByText(named(message(language, 'backup-run-queued')));
      expect(waiting).toBeInTheDocument();
      // The local copy of the same run is plain.
      expect(screen.getByText(named(message(language, 'backup-run-ok')))).toBeInTheDocument();
    });

    it('a run that is refused shows the reason in the app language', async () => {
      const { user } = await open();
      vi.mocked(backupApi.backupNow).mockRejectedValueOnce(refused({ code: 'ERR_BACKUP_BUSY' }));
      await user.click(button('backup-now'));
      const alert = await screen.findByRole('alert');
      expect(visible(alert.textContent)).toContain(message(language, 'err-backup-busy'));
    });

    it('a place that failed shows why, in the app language', async () => {
      const { user } = await open();
      vi.mocked(backupApi.backupNow).mockResolvedValueOnce({
        ...NOW_QUEUED,
        results: [
          {
            kind: 'usb',
            ok: false,
            file: null,
            size_bytes: 0,
            queued: false,
            code: 'ERR_BACKUP_USB_NOT_PRESENT',
            reason: null,
          },
        ],
      });
      await user.click(button('backup-now'));
      expect(
        await screen.findByText(named(message(language, 'err-backup-usb-not-present'))),
      ).toBeInTheDocument();
    });
    it('a cloud copy that failed shows the real reason of the failure', async () => {
      const { user } = await open();
      vi.mocked(backupApi.backupNow).mockResolvedValueOnce({
        ...NOW_QUEUED,
        results: [
          {
            kind: 'cloud',
            ok: false,
            file: null,
            size_bytes: 0,
            queued: false,
            code: 'ERR_BACKUP_CLOUD_FAILED',
            reason: 'refused',
          },
        ],
      });
      await user.click(button('backup-now'));
      expect(
        await screen.findByText(
          named(message(language, 'err-backup-cloud-failed', { reason: 'refused' })),
        ),
      ).toBeInTheDocument();
    });
  });

  describe('restore', () => {
    async function startRestore(row: string) {
      const view = await open();
      await waitFor(() => {
        expect(cell(row, 'kind')).not.toBe('');
      });
      await view.user.click(cellElement(row, 'kind'));
      await view.user.keyboard('{Enter}');
      const dialog = await screen.findByRole('dialog');
      return { ...view, dialog };
    }

    it('Enter on a copy shows its date, its books next to the books now, and what is lost', async () => {
      const { dialog } = await startRestore(CLOUD);
      await shows(dialog, message(language, 'backup-restore-made-on', { date: MADE }));
      expect(backupApi.backupRestorePreview).toHaveBeenCalledWith({
        backup: { kind: 'cloud', name: 'mandi-20261010T090100000Z.mbk' },
        key: null,
      });
      const totals = within(dialog).getByRole('table', {
        name: message(language, 'backup-restore-totals-label'),
      });
      expect(visible(totals.textContent)).toContain(message(language, 'backup-totals-in-backup'));
      expect(visible(totals.textContent)).toContain(message(language, 'backup-totals-now'));
      expect(visible(totals.textContent)).toContain(message(language, 'backup-totals-cash'));
      // This copy loses nothing.
      expect(
        within(dialog).getByText(named(message(language, 'backup-restore-lost-none'))),
      ).toBeInTheDocument();
      expect(backupApi.backupRestore).not.toHaveBeenCalled();
    });

    it('newer records are listed as lost, with their numbers from the backend', async () => {
      fakeBackup();
      vi.mocked(backupApi.backupRestorePreview).mockResolvedValue({
        ...PREVIEW,
        lost: { journal_entries: 12, bikri_entries: 2, vouchers: 3 },
      });
      await renderApp('/backup');
      await waitFor(() => {
        expect(cell(CLOUD, 'kind')).not.toBe('');
      });
      const user = userEvent.setup();
      await user.click(cellElement(CLOUD, 'kind'));
      await user.keyboard('{Enter}');
      const dialog = await screen.findByRole('dialog');
      await shows(
        dialog,
        message(language, 'backup-restore-lost', { journal: 12, bikri: 2, vouchers: 3 }),
      );
    });

    it('the confirm box says what will happen and starts on Back; confirming restores', async () => {
      const { user, dialog } = await startRestore(CLOUD);
      await user.click(
        await within(dialog).findByRole('button', {
          name: named(message(language, 'backup-restore-button')),
        }),
      );
      const confirm = await screen.findByRole('alertdialog');
      expect(visible(confirm.textContent)).toContain(
        message(language, 'backup-restore-confirm-description', {
          date: MADE,
          lost: 0,
          journal: 0,
          bikri: 0,
          vouchers: 0,
        }),
      );
      // A stray Enter must not replace the books: focus starts on Back.
      expect(document.activeElement).toBe(
        within(confirm).getByRole('button', { name: new RegExp(message(language, 'common-back')) }),
      );
      expect(backupApi.backupRestore).not.toHaveBeenCalled();
      // The restore signs the Owner out: the next read of who is signed in says nobody is.
      vi.mocked(authApi.authStatus).mockResolvedValue(
        authState({ session: null, login_language: language }),
      );
      await user.click(
        within(confirm).getByRole('button', {
          name: named(message(language, 'backup-restore-button')),
        }),
      );
      await waitFor(() => {
        expect(backupApi.backupRestore).toHaveBeenCalledWith({
          backup: { kind: 'cloud', name: 'mandi-20261010T090100000Z.mbk' },
          key: null,
          confirmed: true,
        });
      });
      // Everything is read again, and the sign-in screen opens.
      await screen.findByRole('heading', { name: named(message(language, 'auth-login-title')) });
      expect(
        (await screen.findAllByText(named(message(language, 'backup-restored', { date: MADE }))))
          .length,
      ).toBeGreaterThan(0);
    });

    it('Back in the confirm box restores nothing', async () => {
      const { user, dialog } = await startRestore(CLOUD);
      await user.click(
        await within(dialog).findByRole('button', {
          name: named(message(language, 'backup-restore-button')),
        }),
      );
      const confirm = await screen.findByRole('alertdialog');
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(confirm).not.toBeInTheDocument();
      expect(backupApi.backupRestore).not.toHaveBeenCalled();
    });

    it('the restore button of the list works for the row that has focus, and is off with none', async () => {
      const { user } = await open();
      await waitFor(() => {
        expect(cell(LOCAL, 'kind')).not.toBe('');
      });
      expect(button('backup-restore-selected')).toBeDisabled();
      expect(button('backup-test-selected')).toBeDisabled();
      await user.click(cellElement(LOCAL, 'kind'));
      expect(button('backup-restore-selected')).toBeEnabled();
      await user.click(button('backup-restore-selected'));
      const dialog = await screen.findByRole('dialog');
      await shows(dialog, message(language, 'backup-restore-made-on', { date: MADE }));
      expect(backupApi.backupRestorePreview).toHaveBeenCalledWith({
        backup: { kind: 'local', name: 'mandi-local-20261010T090100000Z.db' },
        key: null,
      });
    });

    it('a restore the backend refuses shows the reason in the dialog and changes nothing', async () => {
      const { user, dialog } = await startRestore(CLOUD);
      vi.mocked(backupApi.backupRestore).mockRejectedValueOnce(
        refused({ code: 'ERR_BACKUP_CORRUPT' }),
      );
      await user.click(
        await within(dialog).findByRole('button', {
          name: named(message(language, 'backup-restore-button')),
        }),
      );
      const confirm = await screen.findByRole('alertdialog');
      await user.click(
        within(confirm).getByRole('button', {
          name: named(message(language, 'backup-restore-button')),
        }),
      );
      const alert = await within(await screen.findByRole('dialog')).findByRole('alert');
      expect(visible(alert.textContent)).toContain(message(language, 'err-backup-corrupt'));
      // Still signed in, and the screen is still here.
      expect(
        screen.getByRole('heading', {
          level: 1,
          hidden: true,
          name: named(message(language, 'backup-title')),
        }),
      ).toBeInTheDocument();
    });

    it('a copy that cannot be read says so with a retry', async () => {
      fakeBackup();
      vi.mocked(backupApi.backupRestorePreview).mockRejectedValueOnce(
        refused({ code: 'ERR_BACKUP_CORRUPT' }),
      );
      await renderApp('/backup');
      await waitFor(() => {
        expect(cell(LOCAL, 'kind')).not.toBe('');
      });
      const user = userEvent.setup();
      await user.click(cellElement(LOCAL, 'kind'));
      await user.keyboard('{Enter}');
      const dialog = await screen.findByRole('dialog');
      const alert = await within(dialog).findByRole('alert');
      expect(visible(alert.textContent)).toContain(
        message(language, 'backup-restore-preview-failed'),
      );
      expect(visible(alert.textContent)).toContain(message(language, 'err-backup-corrupt'));
      await user.click(
        within(alert).getByRole('button', { name: message(language, 'common-retry') }),
      );
      await shows(dialog, message(language, 'backup-restore-made-on', { date: MADE }));
    });

    it('on a new computer a locked copy asks for the key first; a wrong key changes nothing', async () => {
      const { user } = await open({ status: { key_set: false, key_confirmed: false } });
      await waitFor(() => {
        expect(cell(USB, 'kind')).not.toBe('');
      });
      await user.click(cellElement(USB, 'kind'));
      await user.keyboard('{Enter}');
      const dialog = await screen.findByRole('dialog');
      const field = await within(dialog).findByLabelText<HTMLInputElement>(
        named(message(language, 'backup-restore-key-label')),
      );
      // Nothing is read from the copy before the key is given.
      expect(backupApi.backupRestorePreview).not.toHaveBeenCalled();
      vi.mocked(backupApi.backupRestorePreview).mockRejectedValueOnce(
        refused({ code: 'ERR_BACKUP_KEY_WRONG', field: 'key' }),
      );
      await user.type(field, 'ABCD-EFGH-JKLM-NPQR-STUV-WXYZ');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(backupApi.backupRestorePreview).toHaveBeenCalledWith({
          backup: { kind: 'usb', name: 'mandi-20261010T090100000Z.mbk' },
          key: 'ABCD-EFGH-JKLM-NPQR-STUV-WXYZ',
        });
      });
      const again = await within(dialog).findByLabelText<HTMLInputElement>(
        named(message(language, 'backup-restore-key-label')),
      );
      await waitFor(() => {
        expect(again).toHaveAttribute('aria-invalid', 'true');
      });
      expect(visible(dialog.textContent)).toContain(message(language, 'err-backup-key-wrong'));
      expect(backupApi.backupRestore).not.toHaveBeenCalled();
      // The right key shows the copy, and the restore carries that key.
      await user.clear(again);
      await user.type(again, 'XXXX-XXXX-XXXX-XXXX-XXXX-XXXX');
      await user.keyboard('{Enter}');
      await shows(dialog, message(language, 'backup-restore-made-on', { date: MADE }));
      await user.click(
        within(dialog).getByRole('button', {
          name: named(message(language, 'backup-restore-button')),
        }),
      );
      const confirm = await screen.findByRole('alertdialog');
      vi.mocked(authApi.authStatus).mockResolvedValue(
        authState({ session: null, login_language: language }),
      );
      await user.click(
        within(confirm).getByRole('button', {
          name: named(message(language, 'backup-restore-button')),
        }),
      );
      await waitFor(() => {
        expect(backupApi.backupRestore).toHaveBeenCalledWith({
          backup: { kind: 'usb', name: 'mandi-20261010T090100000Z.mbk' },
          key: 'XXXX-XXXX-XXXX-XXXX-XXXX-XXXX',
          confirmed: true,
        });
      });
    });

    it('switching the language in the middle keeps the key typed', async () => {
      const { user } = await open({ status: { key_set: false, key_confirmed: false } });
      await waitFor(() => {
        expect(cell(USB, 'kind')).not.toBe('');
      });
      await user.click(cellElement(USB, 'kind'));
      await user.keyboard('{Enter}');
      const dialog = await screen.findByRole('dialog');
      const field = await within(dialog).findByLabelText<HTMLInputElement>(
        named(message(language, 'backup-restore-key-label')),
      );
      await user.type(field, 'ABCD-EF');
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.setState({ language: other });
      });
      const moved = await screen.findByLabelText<HTMLInputElement>(
        named(message(other, 'backup-restore-key-label')),
      );
      expect(moved.value).toBe('ABCD-EF');
    });
  });

  describe('test restore', () => {
    it('tests the newest copy, shows each check and the books next to the live books', async () => {
      const { user } = await open();
      await user.click(buttons('backup-test-now')[0] ?? document.body);
      const dialog = await screen.findByRole('dialog');
      await within(dialog).findByText(named(message(language, 'backup-test-ok')));
      expect(backupApi.backupTestRestore).toHaveBeenCalledWith({ backup: null, key: null });
      for (const check of TEST_RESULT.checks) {
        expect(visible(dialog.textContent)).toContain(
          message(language, `backup-check-${check.name}`),
        );
      }
      expect(
        within(dialog).getAllByText(named(message(language, 'backup-check-passed'))).length,
      ).toBe(TEST_RESULT.checks.length);
      const totals = within(dialog).getByRole('table', {
        name: message(language, 'backup-test-totals-label'),
      });
      expect(visible(totals.textContent)).toContain(message(language, 'backup-totals-restored'));
      expect(visible(totals.textContent)).toContain(message(language, 'backup-totals-live'));
    });

    it('a copy that fails a check is a result, with the failing check named', async () => {
      const { user } = await open();
      vi.mocked(backupApi.backupTestRestore).mockResolvedValueOnce({
        ...TEST_RESULT,
        ok: false,
        checks: TEST_RESULT.checks.map((check) =>
          check.name === 'balanced' ? { ...check, ok: false } : check,
        ),
      });
      await user.click(buttons('backup-test-now')[0] ?? document.body);
      const dialog = await screen.findByRole('dialog');
      await within(dialog).findByText(named(message(language, 'backup-test-not-ok')));
      expect(
        within(dialog).getAllByText(named(message(language, 'backup-check-failed'))).length,
      ).toBe(1);
    });

    it('the button of the list tests the copy that has focus', async () => {
      const { user } = await open();
      await waitFor(() => {
        expect(cell(USB, 'kind')).not.toBe('');
      });
      await user.click(cellElement(USB, 'kind'));
      await user.click(button('backup-test-selected'));
      await screen.findByRole('dialog');
      await waitFor(() => {
        expect(backupApi.backupTestRestore).toHaveBeenCalledWith({
          backup: { kind: 'usb', name: 'mandi-20261010T090100000Z.mbk' },
          key: null,
        });
      });
    });

    it('a locked copy on a computer with no key asks for the key and tests again with it', async () => {
      const { user } = await open();
      vi.mocked(backupApi.backupTestRestore).mockRejectedValueOnce(
        refused({ code: 'ERR_BACKUP_KEY_REQUIRED', field: 'key' }),
      );
      await user.click(buttons('backup-test-now')[0] ?? document.body);
      const dialog = await screen.findByRole('dialog');
      const field = await within(dialog).findByLabelText<HTMLInputElement>(
        named(message(language, 'backup-restore-key-label')),
      );
      await user.type(field, 'XXXX-XXXX-XXXX-XXXX-XXXX-XXXX');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(backupApi.backupTestRestore).toHaveBeenLastCalledWith({
          backup: null,
          key: 'XXXX-XXXX-XXXX-XXXX-XXXX-XXXX',
        });
      });
      await within(dialog).findByText(named(message(language, 'backup-test-ok')));
    });

    it('a test that cannot run says so with a retry', async () => {
      const { user } = await open();
      vi.mocked(backupApi.backupTestRestore).mockRejectedValueOnce(
        refused({ code: 'ERR_INTERNAL' }),
      );
      await user.click(buttons('backup-test-now')[0] ?? document.body);
      const dialog = await screen.findByRole('dialog');
      const alert = await within(dialog).findByRole('alert');
      expect(visible(alert.textContent)).toContain(message(language, 'backup-test-failed'));
      await user.click(
        within(alert).getByRole('button', { name: message(language, 'common-retry') }),
      );
      await within(dialog).findByText(named(message(language, 'backup-test-ok')));
    });
  });

  describe('the backup key', () => {
    it('with no key yet the screen says so, makes the key on request and shows it once', async () => {
      const { user } = await open({ status: STATUS_NEW });
      expect(
        await screen.findByText(named(message(language, 'backup-key-missing'))),
      ).toBeInTheDocument();
      expect(backupApi.backupKeySetup).not.toHaveBeenCalled();
      await user.click(button('backup-key-make'));
      expect(await screen.findByLabelText(KEY_SETUP.key)).toBeInTheDocument();
      expect(backupApi.backupKeySetup).toHaveBeenCalledTimes(1);
    });

    it('the key is printed, typed back from the paper, and confirmed; a wrong key is shown at its field', async () => {
      const { user } = await open({ status: STATUS_NEW });
      await user.click(button('backup-key-make'));
      await screen.findByLabelText(KEY_SETUP.key);
      const typed = screen.getByLabelText<HTMLInputElement>(
        named(message(language, 'backup-key-type-label')),
      );
      const confirm = button('backup-key-confirm');
      // Confirming waits until the Owner says he printed the key.
      await user.type(typed, 'ABCD-EFGH-JKLM-NPQR-STUV-WXYZ');
      expect(confirm).toBeDisabled();
      await user.click(
        screen.getByRole('checkbox', { name: named(message(language, 'backup-key-printed')) }),
      );
      expect(confirm).toBeEnabled();
      vi.mocked(backupApi.backupKeyConfirm).mockRejectedValueOnce(
        refused({ code: 'ERR_BACKUP_KEY_WRONG', field: 'key' }),
      );
      await user.click(confirm);
      await waitFor(() => {
        expect(typed).toHaveAttribute('aria-invalid', 'true');
      });
      expect(backupApi.backupKeyConfirm).toHaveBeenCalledWith({
        key: 'ABCD-EFGH-JKLM-NPQR-STUV-WXYZ',
      });
      expect(visible(document.body.textContent)).toContain(
        message(language, 'err-backup-key-wrong'),
      );
      await user.clear(typed);
      await user.type(typed, KEY_SETUP.key);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(backupApi.backupKeyConfirm).toHaveBeenLastCalledWith({ key: KEY_SETUP.key });
      });
      expect(
        (await screen.findAllByText(named(message(language, 'backup-key-confirmed')))).length,
      ).toBeGreaterThan(0);
      // The key is gone from the store and from the screen once the step is over.
      expect(useBackupKeyStore.getState().key).toBeNull();
    });

    it('the printed sheet is only on paper, and holds the key', async () => {
      const { user } = await open({ status: STATUS_NEW });
      await user.click(button('backup-key-make'));
      await screen.findByLabelText(KEY_SETUP.key);
      const sheet = document.querySelector('[data-key-sheet]');
      expect(sheet).not.toBeNull();
      expect(visible(sheet?.textContent)).toContain(KEY_SETUP.key);
      expect(sheet).toHaveClass('hidden', 'print:block');
    });

    it('a key that was never confirmed asks for it, and a lost sheet can be replaced', async () => {
      const { user } = await open({
        status: { key_set: true, key_confirmed: false, key_confirm_due: true },
      });
      expect(
        await screen.findByText(named(message(language, 'backup-key-confirm-due'))),
      ).toBeInTheDocument();
      await user.click(button('backup-key-lost'));
      expect(await screen.findByLabelText(KEY_SETUP.key)).toBeInTheDocument();
    });

    it('a key already in use is not shown again, and the Owner is told', async () => {
      const { user } = await open({
        status: { key_set: true, key_confirmed: false, key_confirm_due: false },
      });
      vi.mocked(backupApi.backupKeySetup).mockRejectedValueOnce(
        refused({ code: 'ERR_BACKUP_KEY_ALREADY_SET' }),
      );
      await user.click(
        await screen.findByRole('button', { name: named(message(language, 'backup-key-lost')) }),
      );
      expect(
        await screen.findByText(named(message(language, 'backup-key-already'))),
      ).toBeInTheDocument();
      expect(useBackupKeyStore.getState().key).toBeNull();
    });

    it('a confirmed key shows no key card at all', async () => {
      await open();
      await waitFor(() => {
        expect(cell(LOCAL, 'kind')).not.toBe('');
      });
      expect(screen.queryByText(named(message(language, 'backup-key-missing')))).toBeNull();
      expect(screen.queryByText(named(message(language, 'backup-key-title')))).toBeNull();
    });
  });

  describe('the settings', () => {
    const label = (id: string) =>
      screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

    it('shows what is stored, and the cloud fields only while the cloud copy is on', async () => {
      const { user } = await open();
      expect((await label('backup-set-closing-time')).value).toBe('21:00');
      // The folder in the hint is set apart as a code, never written out as raw markup.
      expect(document.body.textContent).not.toMatch(/<code>/);
      expect(screen.queryByLabelText(named(message(language, 'backup-set-bucket')))).toBeNull();
      await user.click(
        await screen.findByRole('checkbox', {
          name: named(message(language, 'backup-set-cloud-on')),
        }),
      );
      expect(await label('backup-set-bucket')).toBeInTheDocument();
      // The keys are secrets: only whether they are stored is shown.
      expect(screen.getAllByText(named(message(language, 'backup-set-key-none'))).length).toBe(2);
    });

    it('saves what was typed, as the exact request, and keys the screen never shows', async () => {
      const { user, store } = await open();
      const closing = await label('backup-set-closing-time');
      await user.clear(closing);
      await user.type(closing, '22:30');
      await user.click(
        await screen.findByRole('checkbox', {
          name: named(message(language, 'backup-set-cloud-on')),
        }),
      );
      await user.type(
        await label('backup-set-endpoint'),
        'https://example.r2.cloudflarestorage.com',
      );
      await user.type(await label('backup-set-bucket'), 'mandi-backup');
      await user.type(await label('backup-set-key-id'), 'AKIA-EXAMPLE');
      await user.type(await label('backup-set-secret'), 'example-secret');
      await user.click(
        screen.getByRole('button', { name: named(message(language, 'layout-save-save')) }),
      );
      await waitFor(() => {
        expect(backupApi.backupSettingsSet).toHaveBeenCalledWith({
          settings: {
            local_dir: '',
            usb_enabled: false,
            usb_folder: '',
            cloud_enabled: true,
            cloud_endpoint: 'https://example.r2.cloudflarestorage.com',
            cloud_bucket: 'mandi-backup',
            cloud_region: 'auto',
            cloud_prefix: 'mandi',
            closing_time: '22:30',
          },
          cloud_access_key_id: 'AKIA-EXAMPLE',
          cloud_secret_access_key: 'example-secret',
        });
      });
      expect(store.settings.cloud_secret_set).toBe(true);
      expect(
        (await screen.findAllByText(named(message(language, 'backup-settings-saved')))).length,
      ).toBeGreaterThan(0);
      // After the save the keys are stored and the typed text is gone.
      await waitFor(() => {
        expect(screen.getAllByText(named(message(language, 'backup-set-key-stored'))).length).toBe(
          2,
        );
      });
      expect((await label('backup-set-secret')).value).toBe('');
    });

    it('every problem shows at its own field and nothing is lost', async () => {
      const { user } = await open();
      vi.mocked(backupApi.backupSettingsSet).mockRejectedValueOnce(
        refused(
          { code: 'ERR_REQUIRED', field: 'usb_folder' },
          { code: 'ERR_REQUIRED', field: 'cloud_bucket' },
        ),
      );
      await user.click(
        await screen.findByRole('checkbox', {
          name: named(message(language, 'backup-set-usb-on')),
        }),
      );
      await user.click(
        await screen.findByRole('checkbox', {
          name: named(message(language, 'backup-set-cloud-on')),
        }),
      );
      await user.type(await label('backup-set-endpoint'), 'https://x.example');
      await user.click(
        screen.getByRole('button', { name: named(message(language, 'layout-save-save')) }),
      );
      const folder = await label('backup-set-usb-folder');
      await waitFor(() => {
        expect(folder).toHaveAttribute('aria-invalid', 'true');
      });
      expect(await label('backup-set-bucket')).toHaveAttribute('aria-invalid', 'true');
      expect((await label('backup-set-endpoint')).value).toBe('https://x.example');
      // The same problems the backend's own refused request carries are all drawn.
      expect(SETTINGS_REFUSED.errors.map((item) => item.field)).toContain('closing_time');
    });

    it('removing a stored key sends an empty text, and keeping it sends nothing', async () => {
      fakeBackup({
        settings: {
          ...SETTINGS_GET,
          cloud_access_key_set: true,
          cloud_secret_set: true,
          settings: {
            ...SETTINGS_GET.settings,
            cloud_enabled: true,
            cloud_bucket: 'b',
            cloud_endpoint: 'https://e.example',
          },
        },
      });
      await renderApp('/backup');
      const user = userEvent.setup();
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'backup-title')),
      });
      await user.click(
        (
          await screen.findAllByRole('button', {
            name: named(message(language, 'backup-set-key-remove')),
          })
        )[1] ?? document.body,
      );
      await user.click(
        screen.getByRole('button', { name: named(message(language, 'layout-save-save')) }),
      );
      await waitFor(() => {
        expect(backupApi.backupSettingsSet).toHaveBeenCalledWith(
          expect.objectContaining({ cloud_access_key_id: null, cloud_secret_access_key: '' }),
        );
      });
    });

    it('switching the language in the middle keeps what was typed', async () => {
      const { user } = await open();
      const closing = await label('backup-set-closing-time');
      await user.clear(closing);
      await user.type(closing, '23:15');
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.setState({ language: other });
      });
      const moved = await screen.findByLabelText<HTMLInputElement>(
        named(message(other, 'backup-set-closing-time')),
      );
      expect(moved.value).toBe('23:15');
    });
  });

  describe('by keyboard only', () => {
    it('Enter on "back up now" makes the copies, and Esc leaves to the dashboard', async () => {
      const { user, router } = await open();
      const now = button('backup-now');
      now.focus();
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(backupApi.backupNow).toHaveBeenCalledTimes(1);
      });
      await screen.findAllByText(named(message(language, 'backup-run-ok')));
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/');
      });
    });

    it('a copy is restored with the keyboard: arrows to the row, Enter, Enter on Restore, Enter on the question', async () => {
      const { user } = await open();
      await waitFor(() => {
        expect(cell(CLOUD, 'kind')).not.toBe('');
      });
      // Focus enters the list on its first cell; the arrow moves down to the cloud row.
      cellElement(LOCAL, 'created').focus();
      await user.click(cellElement(CLOUD, 'created'));
      await user.keyboard('{Enter}');
      const dialog = await screen.findByRole('dialog');
      const restore = await within(dialog).findByRole('button', {
        name: named(message(language, 'backup-restore-button')),
      });
      await waitFor(() => {
        expect(restore).toBeEnabled();
      });
      restore.focus();
      await user.keyboard('{Enter}');
      const confirm = await screen.findByRole('alertdialog');
      // Back has focus: Enter on it closes the question and restores nothing.
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(confirm).not.toBeInTheDocument();
      });
      expect(backupApi.backupRestore).not.toHaveBeenCalled();
    });
  });

  it('the status bar link and the main menu item open this screen', async () => {
    fakeBackup();
    const { router } = await renderApp('/');
    const nav = await screen.findByRole('navigation', {
      name: message(language, 'layout-nav-label'),
    });
    await userEvent
      .setup()
      .click(within(nav).getByRole('button', { name: message(language, 'layout-nav-backup') }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/backup');
    });
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'backup-title')),
      }),
    ).toBeInTheDocument();
  });
});

describe('the wire error of a refused key', () => {
  it('is the backend’s own shape', () => {
    expect(KEY_CONFIRM_WRONG.errors[0]?.field).toBe('key');
  });
});
