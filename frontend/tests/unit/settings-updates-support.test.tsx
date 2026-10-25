// Settings, the app area (plan 22, NFR-INS-3, NFR-INS-4, NFR-SEC-11, NFR-OBS-4), in Urdu and in
// English, against a fake backend that answers with the backend's own fixtures: the app looks for a
// new version when the area opens, shows what is new in the app language, and installs only after the
// Owner presses yes in a box that starts on "Back"; never while there are unsaved entries; each of the
// seven update refusals is told in words. The support file goes where the Owner chooses in the save
// box; a path the backend refuses gets its own words.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { UpdateCheck } from '../../src/bindings/UpdateCheck';
import * as settingsApi from '../../src/features/settings/api';
import * as maintenance from '../../src/features/settings/maintenance-api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as dialog from '../../src/shared/api/dialog';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  INSTALL_CHANGED,
  INSTALL_NOT_CONFIRMED,
  INSTALL_NOT_SIGNED,
  INSTALL_OK,
  INSTALL_REQUEST,
  SUPPORT_REFUSED,
  SUPPORT_REQUEST,
  SUPPORT_SAVED,
  UPDATE_AVAILABLE,
  UPDATE_FAILED,
  UPDATE_NONE,
} from '../helpers/b5-real';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/settings/maintenance-api');
vi.mock('../../src/shared/api/dialog');
vi.mock('../../src/shared/urdu/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

const named = (language: Language, id: string) => (name: string) =>
  visible(name) === message(language, id);

/**
 * The app asks once for a new version when it starts (plan 19, a note on the dashboard and in the
 * bell), before the Owner opens the area of Settings that asks for itself. So the first answer of
 * the fake goes to the start-up check, and the count of asks has that one in it.
 */
const START_UP_CHECKS = 1;

/** A button whose name is the message, followed by its key chip when it has one. */
const keyed = (language: Language, id: string) => (name: string) =>
  visible(name).startsWith(message(language, id));

/** The element that holds exactly this text as one leaf. Fluent wraps variables in isolation marks. */
const leaf = (expected: string) => (_content: string, element: Element | null) =>
  element !== null && element.children.length === 0 && visible(element.textContent) === expected;

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(maintenance.updatesCheck).mockResolvedValue(UPDATE_AVAILABLE);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('Settings, app and updates (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const notesOf = (data: UpdateCheck) => (language === 'ur' ? data.notes_ur : data.notes_en);

  async function openApp() {
    const app = await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: m('settings-title') });
    const user = userEvent.setup();
    await user.click(screen.getByRole('tab', { name: named(language, 'settings-tab-app') }));
    return { ...app, user };
  }

  describe('looking for a new version', () => {
    it('asks when the area opens, and says the app is up to date with its version', async () => {
      vi.mocked(maintenance.updatesCheck).mockResolvedValue(UPDATE_NONE);
      const { user } = await openApp();
      expect(await screen.findByText(m('settings-update-up-to-date'))).toBeInTheDocument();
      expect(
        screen.getByText(leaf(m('settings-app-version', { version: '1.0.0' }))),
      ).toBeInTheDocument();
      expect(maintenance.updatesCheck).toHaveBeenCalledTimes(1 + START_UP_CHECKS);
      // Nothing is on offer, so there is nothing to install.
      expect(
        screen.queryByRole('button', { name: named(language, 'settings-update-install') }),
      ).toBeNull();
      await user.click(
        screen.getByRole('button', { name: named(language, 'settings-update-check-again') }),
      );
      await waitFor(() => {
        expect(maintenance.updatesCheck).toHaveBeenCalledTimes(2 + START_UP_CHECKS);
      });
    });

    it('says it is looking while the server answers', async () => {
      vi.mocked(maintenance.updatesCheck).mockReturnValue(new Promise(() => undefined));
      await openApp();
      expect(await screen.findByText(m('settings-update-checking'))).toBeInTheDocument();
    });

    it('when the server cannot be reached, says so in words and offers to try again', async () => {
      vi.mocked(maintenance.updatesCheck)
        .mockResolvedValueOnce(UPDATE_NONE)
        .mockRejectedValueOnce(new CommandError(UPDATE_FAILED))
        .mockResolvedValue(UPDATE_NONE);
      const { user } = await openApp();
      expect(await screen.findByText(leaf(m('err-update-check-failed')))).toBeInTheDocument();
      await user.click(
        screen.getByRole('button', { name: named(language, 'settings-update-check-again') }),
      );
      expect(await screen.findByText(m('settings-update-up-to-date'))).toBeInTheDocument();
      expect(screen.queryByText(leaf(m('err-update-check-failed')))).toBeNull();
    });
  });

  describe('what is new', () => {
    it('shows the version, the day and the notes of the app language as a list', async () => {
      await openApp();
      expect(
        await screen.findByText(leaf(m('settings-update-available', { version: '99.0.0' }))),
      ).toBeInTheDocument();
      expect(screen.getByText(/23-10-2026/)).toBeInTheDocument();
      const items = Array.from(document.querySelectorAll('li')).map((li) =>
        visible(li.textContent),
      );
      expect(items).toEqual(
        notesOf(UPDATE_AVAILABLE)
          .split('\n')
          .map((line) => line.slice(2)),
      );
      // The notes are release text: they carry their own language and direction.
      const block = document.querySelector('li')?.closest('div');
      expect(block).toHaveAttribute('lang', language);
      expect(block).toHaveAttribute('dir', language === 'ur' ? 'rtl' : 'ltr');
    });

    it('uses the other language when the app language has no notes, then the plain notes', async () => {
      vi.mocked(maintenance.updatesCheck).mockResolvedValue({
        ...UPDATE_AVAILABLE,
        notes_ur: language === 'ur' ? '' : UPDATE_AVAILABLE.notes_ur,
        notes_en: language === 'en' ? '' : UPDATE_AVAILABLE.notes_en,
      });
      await openApp();
      await screen.findByText(leaf(m('settings-update-available', { version: '99.0.0' })));
      const other = language === 'ur' ? UPDATE_AVAILABLE.notes_en : UPDATE_AVAILABLE.notes_ur;
      expect(Array.from(document.querySelectorAll('li')).map((li) => li.textContent)).toEqual(
        other.split('\n').map((line) => line.slice(2)),
      );
    });

    it('with no notes in either language, shows the plain notes; with none at all, says so', async () => {
      vi.mocked(maintenance.updatesCheck).mockResolvedValue({
        ...UPDATE_AVAILABLE,
        notes_ur: '',
        notes_en: '',
        notes: '## Heading\n\nOne paragraph.\n\n- first\n- second',
      });
      const first = await openApp();
      await screen.findByText(leaf(m('settings-update-available', { version: '99.0.0' })));
      expect(screen.getByRole('heading', { level: 4, name: 'Heading' })).toBeInTheDocument();
      expect(screen.getByText('One paragraph.')).toBeInTheDocument();
      expect(screen.getByText('second')).toBeInTheDocument();
      first.unmount();
      vi.mocked(maintenance.updatesCheck).mockResolvedValue({
        ...UPDATE_AVAILABLE,
        notes_ur: '',
        notes_en: '',
        notes: '',
      });
      await openApp();
      expect(await screen.findByText(m('settings-update-no-notes'))).toBeInTheDocument();
    });

    it('a language switch writes the notes in the other language and asks the server nothing new', async () => {
      await openApp();
      await screen.findByText(leaf(m('settings-update-available', { version: '99.0.0' })));
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(screen.getByText(leaf(message(other, 'settings-update-notes')))).toBeInTheDocument();
      }, SLOW_MS);
      const expected = (other === 'ur' ? UPDATE_AVAILABLE.notes_ur : UPDATE_AVAILABLE.notes_en)
        .split('\n')
        .map((line) => line.slice(2));
      expect(Array.from(document.querySelectorAll('li')).map((li) => li.textContent)).toEqual(
        expected,
      );
      expect(maintenance.updatesCheck).toHaveBeenCalledTimes(1 + START_UP_CHECKS);
    });
  });

  describe('installing', () => {
    async function askToInstall() {
      const app = await openApp();
      const install = await screen.findByRole('button', {
        name: named(language, 'settings-update-install'),
      });
      await app.user.click(install);
      const box = await screen.findByRole('alertdialog');
      return { ...app, box, install };
    }

    it('asks first, with the focus on "Back", and nothing is sent when he goes back', async () => {
      const { user, box } = await askToInstall();
      expect(
        within(box).getByText(leaf(m('settings-update-confirm-title', { version: '99.0.0' }))),
      ).toBeInTheDocument();
      expect(within(box).getByText(m('settings-update-confirm-body'))).toBeInTheDocument();
      const back = within(box).getByRole('button', { name: keyed(language, 'common-back') });
      expect(back).toHaveFocus();
      await user.click(back);
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(maintenance.updatesInstall).not.toHaveBeenCalled();
    });

    it('a stray Enter on the box does not start the update', async () => {
      const { user } = await askToInstall();
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(maintenance.updatesInstall).not.toHaveBeenCalled();
    });

    it('yes sends the version he saw and the confirmation, as the backend wrote its request', async () => {
      let finish: (value: typeof INSTALL_OK) => void = () => undefined;
      vi.mocked(maintenance.updatesInstall).mockReturnValue(
        new Promise((resolve) => {
          finish = resolve;
        }),
      );
      const { user, box } = await askToInstall();
      await user.click(
        within(box).getByRole('button', { name: named(language, 'settings-update-confirm-yes') }),
      );
      await waitFor(() => {
        expect(maintenance.updatesInstall).toHaveBeenCalledWith(INSTALL_REQUEST);
      });
      // While it works: the app says it will close, and the buttons are off.
      expect(await screen.findByText(m('settings-update-installing'))).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: named(language, 'settings-update-install') }),
      ).toBeDisabled();
      act(() => {
        finish(INSTALL_OK);
      });
      expect(
        await screen.findByText(
          leaf(
            m('settings-update-done', {
              version: '99.0.0',
              file: 'mandi-local-20261023-140000.db',
            }),
          ),
        ),
      ).toBeInTheDocument();
    });

    it('never during work: with unsaved entries the button is off and says why', async () => {
      vi.mocked(maintenance.updatesCheck).mockResolvedValue({ ...UPDATE_AVAILABLE, drafts: 2 });
      await openApp();
      const install = await screen.findByRole('button', {
        name: named(language, 'settings-update-install'),
      });
      expect(install).toBeDisabled();
      expect(
        screen.getByText(leaf(m('settings-update-has-drafts', { drafts: '2' }))),
      ).toBeInTheDocument();
      expect(maintenance.updatesInstall).not.toHaveBeenCalled();
    });

    it('keyboard only: Tab to the button, Enter asks, Esc goes back', async () => {
      const { user } = await openApp();
      const install = await screen.findByRole('button', {
        name: named(language, 'settings-update-install'),
      });
      install.focus();
      await user.keyboard('{Enter}');
      expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(install).toHaveFocus();
      expect(maintenance.updatesInstall).not.toHaveBeenCalled();
    });

    const REFUSALS: readonly [string, string, () => unknown][] = [
      [
        'ERR_UPDATE_NOT_SIGNED',
        'err-update-not-signed',
        () => new CommandError(INSTALL_NOT_SIGNED),
      ],
      [
        'ERR_UPDATE_NOT_CONFIRMED',
        'err-update-not-confirmed',
        () => new CommandError(INSTALL_NOT_CONFIRMED),
      ],
      [
        'ERR_UPDATE_BACKUP_FAILED',
        'err-update-backup-failed',
        () => fail({ code: 'ERR_UPDATE_BACKUP_FAILED' }),
      ],
      ['ERR_UPDATE_NONE', 'err-update-none', () => fail({ code: 'ERR_UPDATE_NONE' })],
    ];

    it.each(REFUSALS)(
      '%s is told in words and nothing else changes',
      async (_code, id, refusal) => {
        vi.mocked(maintenance.updatesInstall).mockRejectedValue(refusal());
        const { user, box } = await askToInstall();
        await user.click(
          within(box).getByRole('button', { name: named(language, 'settings-update-confirm-yes') }),
        );
        expect(await screen.findByText(leaf(m(id)))).toBeInTheDocument();
        // He can look again, and the install button is back.
        expect(
          screen.getByRole('button', { name: named(language, 'settings-update-install') }),
        ).toBeEnabled();
      },
    );

    it('ERR_UPDATE_DURING_WORK names how many entries are unsaved', async () => {
      vi.mocked(maintenance.updatesInstall).mockRejectedValue(
        fail({ code: 'ERR_UPDATE_DURING_WORK', values: { drafts: 3 } }),
      );
      const { user, box } = await askToInstall();
      await user.click(
        within(box).getByRole('button', { name: named(language, 'settings-update-confirm-yes') }),
      );
      expect(
        await screen.findByText(leaf(m('err-update-during-work', { drafts: 3 }))),
      ).toBeInTheDocument();
    });

    it('ERR_UPDATE_CHANGED looks for the new version again, so he reads what is new first', async () => {
      vi.mocked(maintenance.updatesInstall).mockRejectedValue(new CommandError(INSTALL_CHANGED));
      vi.mocked(maintenance.updatesCheck)
        .mockResolvedValueOnce(UPDATE_AVAILABLE)
        .mockResolvedValueOnce(UPDATE_AVAILABLE)
        .mockResolvedValue({ ...UPDATE_AVAILABLE, version: '99.1.0' });
      const { user, box } = await askToInstall();
      await user.click(
        within(box).getByRole('button', { name: named(language, 'settings-update-confirm-yes') }),
      );
      expect(await screen.findByText(leaf(m('err-update-changed')))).toBeInTheDocument();
      expect(
        await screen.findByText(leaf(m('settings-update-available', { version: '99.1.0' }))),
      ).toBeInTheDocument();
      expect(maintenance.updatesCheck).toHaveBeenCalledTimes(2 + START_UP_CHECKS);
    });
  });

  describe('the support file', () => {
    async function press() {
      const app = await openApp();
      await screen.findByText(m('settings-support-help'));
      await app.user.click(
        screen.getByRole('button', { name: named(language, 'settings-support-save') }),
      );
      return app;
    }

    it('asks where to save, with a name for the day, and sends the path he chose', async () => {
      vi.mocked(dialog.chooseSavePath).mockResolvedValue('/chosen/support.txt');
      vi.mocked(maintenance.supportInfoSave).mockResolvedValue({
        ...SUPPORT_SAVED,
        path: '/chosen/support.txt',
        log_lines: 42,
      });
      await press();
      await waitFor(() => {
        expect(maintenance.supportInfoSave).toHaveBeenCalledWith({ path: '/chosen/support.txt' });
      });
      const asked = vi.mocked(dialog.chooseSavePath).mock.calls[0]?.[0];
      expect(asked?.extension).toBe('txt');
      expect(asked?.filterName).toBe(m('settings-support-file-type'));
      expect(asked?.defaultName).toMatch(/^mandi-support-\d{8}\.txt$/);
      expect(
        await screen.findByText(
          leaf(m('settings-support-saved', { path: '/chosen/support.txt', lines: '42' })),
        ),
      ).toBeInTheDocument();
      // The request has exactly the shape of the backend's own.
      expect(Object.keys(SUPPORT_REQUEST)).toEqual(['path']);
    });

    it('closing the box without choosing sends nothing', async () => {
      vi.mocked(dialog.chooseSavePath).mockResolvedValue(null);
      await press();
      await waitFor(() => {
        expect(dialog.chooseSavePath).toHaveBeenCalledTimes(1);
      });
      expect(maintenance.supportInfoSave).not.toHaveBeenCalled();
    });

    it('a path the backend refuses gets its own words, not a range with nothing in it', async () => {
      vi.mocked(dialog.chooseSavePath).mockResolvedValue('relative.txt');
      vi.mocked(maintenance.supportInfoSave).mockRejectedValue(new CommandError(SUPPORT_REFUSED));
      await press();
      expect(await screen.findByText(m('settings-support-path-refused'))).toBeInTheDocument();
      expect(screen.queryByText(m('err-out-of-range', { min: '', max: '' }))).toBeNull();
      expect(document.body.textContent).not.toContain('{');
    });

    it('a save box that cannot open says something went wrong', async () => {
      vi.mocked(dialog.chooseSavePath).mockRejectedValue(new Error('no window'));
      await press();
      expect(await screen.findByText(m('err-internal'))).toBeInTheDocument();
      expect(maintenance.supportInfoSave).not.toHaveBeenCalled();
    });
  });

  it('writes nothing except through the three commands of this area', async () => {
    await openApp();
    await screen.findByText(leaf(m('settings-update-available', { version: '99.0.0' })));
    expect(maintenance.updatesInstall).not.toHaveBeenCalled();
    expect(maintenance.supportInfoSave).not.toHaveBeenCalled();
    expect(settingsApi.settingsSetFirm).not.toHaveBeenCalled();
  });
});
