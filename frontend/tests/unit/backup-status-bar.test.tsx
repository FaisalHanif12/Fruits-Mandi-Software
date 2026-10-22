// The last good cloud backup in the status bar (plan 18, PRD FR-5.5, FR-20.6, FR-20.7, NFR-BAK-10),
// in the Urdu and in the English app, against a fake backend: green with a tick under 24 hours, red
// with a warning sign and words when the copy is old or missing or failed its check, grey when the
// cloud copy is off, the waiting copies, the state when it cannot be read, a click that opens the
// backup screen. The words and times are the backend's; colour always has an icon and words with it
// (NFR-USE-12). The overlay while the copy at exit runs is in backup-exit-overlay.test.tsx.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as backupApi from '../../src/features/backup/api';
import { statusBarWords } from '../../src/features/backup/components/BackupStatusItem';
import {
  STATUS_POLL_MS,
  STATUS_POLL_RUNNING_MS,
  statusPollMs,
} from '../../src/features/backup/hooks';
import { LANGUAGES } from '../../src/i18n';
import { utcToLocalDisplay } from '../../src/shared/format';
import { STATUS_AFTER, STATUS_NEW, fakeBackup, refused } from '../helpers/backup';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/backup/api');

const WHEN = utcToLocalDisplay('2026-10-10T09:01:00.000Z');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

/** The words of the status bar, without the number markup. */
const bar = () => visible(document.querySelector('footer')?.textContent);

describe('which words the status bar shows', () => {
  it('green under a day, with the time; the waiting copies are added', () => {
    expect(statusBarWords(STATUS_AFTER)).toEqual({
      tone: 'good',
      messageId: 'backup-bar-ok',
      vars: { date: WHEN, queued: '0' },
    });
    expect(statusBarWords({ ...STATUS_AFTER, queue_len: 2 }).messageId).toBe('backup-bar-ok-queue');
  });

  it('red when the copy is old (over 24 hours) or over 48 hours, with the time', () => {
    for (const level of ['red', 'alert'] as const) {
      expect(statusBarWords({ ...STATUS_AFTER, cloud_level: level, cloud_age_hours: 30 })).toEqual({
        tone: 'bad',
        messageId: 'backup-bar-old',
        vars: { date: WHEN },
      });
    }
  });

  it('red when there is no cloud copy yet, grey when the cloud copy is off', () => {
    expect(statusBarWords(STATUS_NEW).messageId).toBe('backup-bar-never');
    expect(statusBarWords({ ...STATUS_NEW, cloud_level: 'off' })).toEqual({
      tone: 'quiet',
      messageId: 'layout-status-backup-none',
      vars: {},
    });
  });

  it('a copy that failed its check is the loudest thing, even when the cloud copy is fresh', () => {
    const failed = {
      kind: 'local' as const,
      code: 'ERR_BACKUP_CHECK_FAILED' as const,
      at: '2026-10-10T09:02:00.000Z',
      check_failed: true,
      file: 'x',
    };
    expect(statusBarWords({ ...STATUS_AFTER, errors: [failed] })).toEqual({
      tone: 'bad',
      messageId: 'backup-bar-check-failed',
      vars: {},
    });
    // A problem that is not a failed check does not change the colour.
    expect(
      statusBarWords({ ...STATUS_AFTER, errors: [{ ...failed, check_failed: false }] }).tone,
    ).toBe('good');
  });
});

describe('how often the state is read', () => {
  it('about once a minute, and every two seconds while a copy or a restore runs', () => {
    expect(statusPollMs(undefined)).toBe(STATUS_POLL_MS);
    expect(statusPollMs(STATUS_AFTER)).toBe(STATUS_POLL_MS);
    expect(statusPollMs({ ...STATUS_AFTER, running: true })).toBe(STATUS_POLL_RUNNING_MS);
    expect(STATUS_POLL_MS).toBe(60_000);
    expect(STATUS_POLL_RUNNING_MS).toBe(2_000);
  });
});

describe.each(LANGUAGES)('the status bar item (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the time of the last good cloud copy in green with a tick', async () => {
    fakeBackup();
    await renderApp('/');
    await waitFor(() => {
      expect(bar()).toContain(message(language, 'backup-bar-ok', { date: WHEN }));
    });
    const item = document.querySelector('footer [data-backup-level]');
    expect(item).toHaveAttribute('data-backup-level', 'green');
    expect(item?.className).toContain('text-money-in');
    expect(item?.querySelector('svg')).not.toBeNull();
  });

  it('shows red with words when the last copy is older than a day', async () => {
    fakeBackup({ status: { cloud_level: 'red', cloud_age_hours: 30 } });
    await renderApp('/');
    await waitFor(() => {
      expect(bar()).toContain(message(language, 'backup-bar-old', { date: WHEN }));
    });
    const item = document.querySelector('footer [data-backup-level]');
    expect(item).toHaveAttribute('data-backup-level', 'red');
    expect(item?.className).toContain('text-due');
  });

  it('says so when no cloud copy was ever made, and when the cloud copy is off', async () => {
    fakeBackup({ status: STATUS_NEW });
    const view = await renderApp('/');
    await waitFor(() => {
      expect(bar()).toContain(message(language, 'backup-bar-never'));
    });
    view.unmount();
    fakeBackup({ status: { ...STATUS_NEW, cloud_level: 'off', cloud_enabled: false } });
    await renderApp('/');
    await waitFor(() => {
      expect(bar()).toContain(message(language, 'layout-status-backup-none'));
    });
  });

  it('shows how many cloud copies wait for the internet', async () => {
    fakeBackup({ status: { queue_len: 3 } });
    await renderApp('/');
    await waitFor(() => {
      expect(bar()).toContain(message(language, 'backup-bar-ok-queue', { date: WHEN, queued: 3 }));
    });
  });

  it('a state that cannot be read is written as unknown, not as good', async () => {
    fakeBackup();
    vi.mocked(backupApi.backupStatus).mockRejectedValue(refused({ code: 'ERR_INTERNAL' }));
    await renderApp('/');
    await waitFor(() => {
      expect(bar()).toContain(message(language, 'backup-bar-unknown'));
    });
  });

  it('a click opens the backup screen', async () => {
    fakeBackup();
    const { router } = await renderApp('/');
    const footer = await screen.findByRole('contentinfo');
    await waitFor(() => {
      expect(
        within(footer).queryByText(/./, { selector: '[data-backup-level] span' }),
      ).not.toBeNull();
    });
    const item = footer.querySelector<HTMLElement>('[data-backup-level]');
    if (item === null) throw new Error('no backup item');
    await userEvent.setup().click(item);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/backup');
    });
  });
});
