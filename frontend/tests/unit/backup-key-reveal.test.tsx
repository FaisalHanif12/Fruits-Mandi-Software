// "See the key again" on the backup screen (PRD FR-20.10 v1.54, Umar 21-11-2026), in the Urdu and in the English app,
// against a fake backend that answers with the backend's own fixtures. The Owner lost his printed sheet: a button opens a
// box that asks his password and his sign; a right pair shows the key large and set apart with a Print button and no
// copy; a wrong password or a wrong sign shows its error at its field and no key; and the key leaves the screen and the
// memory when the box closes (not in the key store, not in the query cache, not on the page).

import { createMemoryHistory, RouterProvider } from '@tanstack/react-router';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders, createQueryClient } from '../../src/app/providers';
import { createAppRouter } from '../../src/app/router';
import * as backupApi from '../../src/features/backup/api';
import { useBackupKeyStore } from '../../src/features/backup/key-store';
import { LANGUAGES } from '../../src/i18n';
import { fakeBackup } from '../helpers/backup';
import {
  KEY_REVEAL,
  KEY_REVEAL_REQUEST,
  WRONG_PASSWORD_TRIES_LEFT,
  wrongPasswordRefusal,
  wrongSignRefusal,
} from '../helpers/backup-key-reveal';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/backup/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  useBackupKeyStore.getState().clear();
});

afterEach(() => {
  leaveAppWindow();
});

const KEY_CHIPS = new Set(['F2', 'F9', 'F10', 'Enter', 'Esc']);
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  if (seen === text) return true;
  return seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim());
};

describe.each(LANGUAGES)('see the backup key again (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  // The app is drawn with a data cache of its own here, so the test can look into it after the box closes.
  let client = createQueryClient();

  async function open(options: Parameters<typeof fakeBackup>[0] = {}) {
    fakeBackup({
      ...options,
      status: { key_set: true, key_confirmed: true, key_confirm_due: false, ...options.status },
    });
    client = createQueryClient();
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true });
    const router = createAppRouter(createMemoryHistory({ initialEntries: ['/backup'] }));
    await router.load();
    const view = render(
      <AppProviders queryClient={client}>
        <RouterProvider router={router} />
      </AppProviders>,
    );
    await screen.findByRole('heading', { level: 1, name: named(m('backup-title')) });
    return { ...view, user: userEvent.setup() };
  }

  const revealButton = () =>
    screen.findByRole('button', { name: named(m('backup-key-reveal-button')) });

  async function openBox(user: ReturnType<typeof userEvent.setup>) {
    await user.click(await revealButton());
    return screen.findByRole('dialog');
  }

  const passwordField = (box: HTMLElement) =>
    within(box).getByLabelText<HTMLInputElement>(named(m('backup-key-reveal-password')));
  const signField = (box: HTMLElement) =>
    within(box).getByLabelText<HTMLInputElement>(named(m('backup-key-reveal-sign')));
  const showButton = (box: HTMLElement) =>
    within(box).getByRole('button', { name: named(m('backup-key-reveal-show')) });

  it('a confirmed key shows only the quiet card with the button: no notice, no key, no make button', async () => {
    await open();
    expect(await revealButton()).toBeInTheDocument();
    expect(screen.queryByText(named(m('backup-key-missing')))).toBeNull();
    expect(screen.queryByText(named(m('backup-key-unconfirmed')))).toBeNull();
    expect(screen.queryByRole('button', { name: named(m('backup-key-make')) })).toBeNull();
    expect(visible(document.body.textContent)).not.toContain(KEY_REVEAL.key);
  });

  it('a key that is not confirmed yet offers the button beside "I lost the printed sheet"', async () => {
    await open({ status: { key_confirmed: false } });
    expect(await revealButton()).toBeInTheDocument();
    expect(screen.getByRole('button', { name: named(m('backup-key-lost')) })).toBeInTheDocument();
  });

  it('no key set yet: there is nothing to see again, so the button is not offered', async () => {
    await open({ status: { key_set: false, key_confirmed: false } });
    await screen.findByText(named(m('backup-key-missing')));
    expect(screen.queryByRole('button', { name: named(m('backup-key-reveal-button')) })).toBeNull();
  });

  it('the box asks the password and the sign, and Show stays off until both are typed', async () => {
    const { user } = await open();
    const box = await openBox(user);
    expect(visible(box.textContent)).toContain(m('backup-key-reveal-title'));
    expect(showButton(box)).toBeDisabled();
    await user.type(passwordField(box), KEY_REVEAL_REQUEST.password);
    expect(showButton(box)).toBeDisabled();
    await user.type(signField(box), KEY_REVEAL_REQUEST.sign);
    expect(showButton(box)).toBeEnabled();
    expect(passwordField(box).type).toBe('password');
  });

  it('a right password and sign show the key large, with Print and no way to copy it', async () => {
    vi.mocked(backupApi.backupKeyReveal).mockResolvedValue(KEY_REVEAL);
    const { user } = await open();
    const box = await openBox(user);
    await user.type(passwordField(box), KEY_REVEAL_REQUEST.password);
    await user.type(signField(box), KEY_REVEAL_REQUEST.sign);
    await user.click(showButton(box));
    // What was sent: the password and the sign as typed (the sign trimmed), nothing else.
    expect(backupApi.backupKeyReveal).toHaveBeenCalledWith({
      password: KEY_REVEAL_REQUEST.password,
      sign: KEY_REVEAL_REQUEST.sign,
    });
    const shown = await within(box).findByLabelText(KEY_REVEAL.key);
    expect(visible(shown.textContent)).toBe(KEY_REVEAL.key);
    expect(shown.className).toContain('select-none');
    // A copy, a cut or a context menu on the key does nothing.
    const copy = new Event('copy', { bubbles: true, cancelable: true });
    shown.dispatchEvent(copy);
    expect(copy.defaultPrevented).toBe(true);
    expect(within(box).getByRole('button', { name: named(m('backup-key-print')) })).toBeEnabled();
    expect(visible(box.textContent)).toContain(m('backup-key-reveal-shown'));
    // The password and the sign are gone from the form once the key is shown.
    expect(within(box).queryByLabelText(named(m('backup-key-reveal-password')))).toBeNull();
    // The key is on the printed sheet too (hidden on screen, drawn on paper only), the same sheet as at setup.
    const sheet = document.querySelector('[data-key-sheet]');
    expect(visible(sheet?.textContent)).toContain(KEY_REVEAL.key);
    expect(visible(sheet?.textContent)).toContain(m('backup-key-sheet-title'));
  });

  it('Print prints the sheet', async () => {
    vi.mocked(backupApi.backupKeyReveal).mockResolvedValue(KEY_REVEAL);
    const printed = vi.spyOn(window, 'print').mockImplementation(() => undefined);
    const { user } = await open();
    const box = await openBox(user);
    await user.type(passwordField(box), KEY_REVEAL_REQUEST.password);
    await user.type(signField(box), KEY_REVEAL_REQUEST.sign);
    await user.click(showButton(box));
    await user.click(
      await within(box).findByRole('button', { name: named(m('backup-key-print')) }),
    );
    expect(printed).toHaveBeenCalledTimes(1);
    printed.mockRestore();
  });

  it('a wrong password shows its words with the tries left at the password field, and no key', async () => {
    vi.mocked(backupApi.backupKeyReveal).mockRejectedValue(wrongPasswordRefusal());
    const { user } = await open();
    const box = await openBox(user);
    await user.type(passwordField(box), KEY_REVEAL_REQUEST.password);
    await user.type(signField(box), KEY_REVEAL_REQUEST.sign);
    await user.click(showButton(box));
    await waitFor(() => {
      expect(passwordField(box)).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(box.textContent)).toContain(
      m('backup-key-reveal-wrong-password', { tries_left: WRONG_PASSWORD_TRIES_LEFT }),
    );
    expect(signField(box)).not.toHaveAttribute('aria-invalid', 'true');
    expect(visible(document.body.textContent)).not.toContain(KEY_REVEAL.key);
    expect(document.querySelector('[data-shown-key]')).toBeNull();
    // No raw message id on the screen.
    expect(visible(box.textContent)).not.toMatch(/\b(err|backup)-[a-z-]+/);
  });

  it('a wrong sign shows its error at the sign field only, and no key', async () => {
    vi.mocked(backupApi.backupKeyReveal).mockRejectedValue(wrongSignRefusal());
    const { user } = await open();
    const box = await openBox(user);
    await user.type(passwordField(box), KEY_REVEAL_REQUEST.password);
    await user.type(signField(box), KEY_REVEAL_REQUEST.sign);
    await user.click(showButton(box));
    await waitFor(() => {
      expect(signField(box)).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(box.textContent)).toContain(m('err-sign-wrong'));
    expect(passwordField(box)).not.toHaveAttribute('aria-invalid', 'true');
    expect(visible(document.body.textContent)).not.toContain(KEY_REVEAL.key);
  });

  it('the key is not kept after the box closes: not on the page, not in the key store, not in the query memory', async () => {
    vi.mocked(backupApi.backupKeyReveal).mockResolvedValue(KEY_REVEAL);
    const { user } = await open();
    const box = await openBox(user);
    await user.type(passwordField(box), KEY_REVEAL_REQUEST.password);
    await user.type(signField(box), KEY_REVEAL_REQUEST.sign);
    await user.click(showButton(box));
    await within(box).findByLabelText(KEY_REVEAL.key);
    // While it is shown it is in no store.
    expect(useBackupKeyStore.getState().key).toBeNull();
    await user.click(
      within(box).getByRole('button', { name: named(m('backup-key-reveal-close')) }),
    );
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(visible(document.body.textContent)).not.toContain(KEY_REVEAL.key);
    expect(document.querySelector('[data-key-sheet]')).toBeNull();
    expect(useBackupKeyStore.getState().key).toBeNull();
    await waitFor(() => {
      const kept = client
        .getMutationCache()
        .getAll()
        .filter((mutation) => JSON.stringify(mutation.state.data ?? null).includes(KEY_REVEAL.key));
      expect(kept).toHaveLength(0);
    });
    // Opening the box again asks for the password and the sign again: nothing was kept.
    const again = await openBox(user);
    expect(passwordField(again).value).toBe('');
    expect(document.querySelector('[data-shown-key]')).toBeNull();
  });

  it('Esc closes the box before the key is shown, and the focus goes back to the button', async () => {
    const { user } = await open();
    const box = await openBox(user);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(box).not.toBeInTheDocument();
    });
    expect(await revealButton()).toHaveFocus();
  });
});
