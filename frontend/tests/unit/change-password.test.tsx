// Change password from the user menu (PRD FR-5.6, NFR-SEC-1, NFR-SEC-2), in the Urdu and in the
// English app, by keyboard only, against a fake backend. The backend checks all three fields; the
// dialog shows what it says at the field it belongs to.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as authApi from '../../src/features/auth/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');

const named = (language: Language, id: string, args: Record<string, string | number> = {}) => {
  const wanted = message(language, id, args);
  return (name: string) => visible(name) === wanted;
};

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('change password (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** Opens the dialog with the keyboard: Enter on the user button, Enter on the first item. */
  async function openDialog() {
    await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    const user = userEvent.setup();
    const button = screen.getByRole('button', { name: message(language, 'layout-user-menu') });
    button.focus();
    await user.keyboard('{Enter}');
    await screen.findByRole('menuitem', { name: message(language, 'layout-user-change-password') });
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog');
    return { user, dialog, button };
  }

  const fieldOf = (dialog: HTMLElement, id: string) =>
    within(dialog).getByLabelText<HTMLInputElement>(named(language, id));

  it('is the first item of the user menu, and all three fields are there, the first with focus', async () => {
    const { dialog } = await openDialog();
    expect(visible(dialog.textContent)).toContain(message(language, 'auth-change-title'));
    expect(fieldOf(dialog, 'auth-change-current')).toHaveFocus();
    expect(fieldOf(dialog, 'auth-change-new')).toBeInTheDocument();
    expect(fieldOf(dialog, 'auth-change-repeat')).toBeInTheDocument();
    // The shortest length is told next to the new password.
    expect(visible(dialog.textContent)).toContain(
      message(language, 'auth-wizard-password-hint', { min: '6' }),
    );
  });

  it('by keyboard only: Enter after each field, saves on the last, says so and closes', async () => {
    vi.mocked(authApi.authChangePassword).mockResolvedValue(null);
    const { user, button } = await openDialog();
    await user.keyboard('oldsecret{Enter}newsecret{Enter}newsecret{Enter}');
    await waitFor(() => {
      expect(authApi.authChangePassword).toHaveBeenCalledWith({
        current: 'oldsecret',
        new: 'newsecret',
        repeat: 'newsecret',
      });
    });
    expect(await screen.findByText(message(language, 'auth-change-saved'))).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    // He is still signed in, and focus is back on the user button.
    expect(authApi.authLogout).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(button).toHaveFocus();
    });
  });

  it('F10 saves from any field', async () => {
    vi.mocked(authApi.authChangePassword).mockResolvedValue(null);
    const { user } = await openDialog();
    await user.keyboard('oldsecret{Enter}newsecret{Enter}newsecret');
    press('F10');
    await waitFor(() => {
      expect(authApi.authChangePassword).toHaveBeenCalledTimes(1);
    });
  });

  it('a wrong current password is told at its field in its own words, with the tries left', async () => {
    vi.mocked(authApi.authChangePassword).mockRejectedValue(
      fail({ code: 'ERR_WRONG_PASSWORD', field: 'current', values: { tries_left: 4 } }),
    );
    const { user, dialog } = await openDialog();
    await user.keyboard('wrong{Enter}newsecret{Enter}newsecret{Enter}');
    const current = fieldOf(dialog, 'auth-change-current');
    await waitFor(() => {
      expect(current).toHaveAttribute('aria-invalid', 'true');
    });
    const note = document.getElementById(current.getAttribute('aria-describedby') ?? '');
    expect(visible(note?.textContent)).toBe(
      message(language, 'auth-change-wrong-current', { tries_left: 4 }),
    );
    expect(document.body.textContent).not.toContain('ERR_');
    // The dialog stays, with what he typed.
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(fieldOf(dialog, 'auth-change-new').value).toBe('newsecret');
  });

  it('a short new password and two different ones are told at their own fields', async () => {
    vi.mocked(authApi.authChangePassword).mockRejectedValue(
      fail(
        { code: 'ERR_PASSWORD_TOO_SHORT', field: 'new', values: { min: 6 } },
        { code: 'ERR_PASSWORDS_DIFFER', field: 'repeat' },
      ),
    );
    const { user, dialog } = await openDialog();
    await user.keyboard('oldsecret{Enter}abc{Enter}abd{Enter}');
    const next = fieldOf(dialog, 'auth-change-new');
    const repeat = fieldOf(dialog, 'auth-change-repeat');
    await waitFor(() => {
      expect(next).toHaveAttribute('aria-invalid', 'true');
    });
    expect(repeat).toHaveAttribute('aria-invalid', 'true');
    const text = (field: HTMLElement) =>
      visible(document.getElementById(field.getAttribute('aria-describedby') ?? '')?.textContent);
    expect(text(next)).toContain(message(language, 'err-password-too-short', { min: 6 }));
    expect(text(repeat)).toContain(message(language, 'err-passwords-differ'));
  });

  it('a lockout is shown in the dialog with the wait, not at a field', async () => {
    vi.mocked(authApi.authChangePassword).mockRejectedValue(
      fail({ code: 'ERR_LOCKED_WAIT', values: { seconds: 60 } }),
    );
    const { user, dialog } = await openDialog();
    await user.keyboard('x{Enter}newsecret{Enter}newsecret{Enter}');
    const alert = await within(dialog).findByRole('alert');
    expect(visible(alert.textContent)).toContain(
      message(language, 'err-locked-wait', { seconds: 60 }),
    );
  });

  it('Esc closes it, sends nothing, and gives focus back to the user button', async () => {
    const { user, button } = await openDialog();
    await user.keyboard('abc');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(authApi.authChangePassword).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(button).toHaveFocus();
    });
    // Opened again, the fields are empty: a password is never kept.
    button.focus();
    await user.keyboard('{Enter}');
    await screen.findByRole('menuitem', { name: message(language, 'layout-user-change-password') });
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog');
    expect(fieldOf(dialog, 'auth-change-current').value).toBe('');
  });
});
