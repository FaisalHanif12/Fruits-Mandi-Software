// Sign in and the recovery path (plan 04, PRD Section 8, FR-8.2, NFR-SEC-2), in Urdu and English,
// by keyboard only, against a fake backend. The backend checks every password; the screen shows
// what it says.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as authApi from '../../src/features/auth/api';
import * as settingsApi from '../../src/features/settings/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  FIRM,
  authState,
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { TEST_REFERENCE } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');

const USERNAME = 'owner';
const PASSWORD = 'secret12';

function field(language: Language, id: string): HTMLInputElement {
  const label = message(language, id);
  return screen.getByLabelText<HTMLInputElement>((text) => visible(text) === label);
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
  vi.useRealTimers();
});

describe.each(LANGUAGES)('sign in (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    vi.mocked(authApi.authStatus).mockResolvedValue(authState({ login_language: language }));
  });

  it('shows the firm in the language of the screen, the version, and the form', async () => {
    await renderApp();
    expect(
      await screen.findByRole('heading', { name: message(language, 'auth-login-title') }),
    ).toBeInTheDocument();
    const firm = language === 'ur' ? FIRM.name_ur : FIRM.name_en;
    expect(screen.getByText(firm)).toBeInTheDocument();
    expect(
      screen.queryByText(language === 'ur' ? FIRM.name_en : FIRM.name_ur),
    ).not.toBeInTheDocument();
    expect(document.body.textContent).toContain('0.1.12');
    expect(document.documentElement.dir).toBe(language === 'ur' ? 'rtl' : 'ltr');
  });

  it('starts on the user name, keeps Sign in off until both are filled, and Enter signs in', async () => {
    vi.mocked(authApi.authLogin).mockResolvedValue(sessionUser({ language }));
    await renderApp();
    const user = userEvent.setup();
    const username = await screen.findByLabelText(
      (text) => visible(text) === message(language, 'auth-login-username'),
    );
    expect(username).toHaveFocus();
    const submit = screen.getByRole('button', {
      name: new RegExp(message(language, 'auth-login-submit')),
    });
    expect(submit).toBeDisabled();
    await user.keyboard(`${USERNAME}{Enter}`);
    expect(field(language, 'auth-login-password')).toHaveFocus();
    expect(submit).toBeDisabled();
    // Enter in the last field signs in. Nothing else is pressed.
    await user.keyboard(`${PASSWORD}{Enter}`);
    await waitFor(() => {
      expect(authApi.authLogin).toHaveBeenCalledWith({ username: USERNAME, password: PASSWORD });
    });
  });

  it('after signing in, the app opens with the dashboard and the user in his language', async () => {
    const owner = sessionUser({ language: 'en', text_size: 110 });
    vi.mocked(authApi.authLogin).mockImplementation(() => {
      vi.mocked(authApi.authStatus).mockResolvedValue(authState({ session: owner }));
      return Promise.resolve(owner);
    });
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    await user.keyboard(`${USERNAME}{Enter}${PASSWORD}{Enter}`);
    // His own language and text size now apply.
    await waitFor(() => {
      expect(useLanguageStore.getState().language).toBe('en');
    });
    expect(await screen.findByRole('heading', { level: 1, name: /Adnan/ })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe('en');
  });

  it('a language chosen on the sign-in screen stays after signing in and is saved as his choice (FR-7.30)', async () => {
    const other: Language = language === 'ur' ? 'en' : 'ur';
    // His saved choice is the language the screen opened in; he switches on the screen itself.
    const owner = sessionUser({ language });
    vi.mocked(authApi.authLogin).mockImplementation(() => {
      vi.mocked(authApi.authStatus).mockResolvedValue(authState({ session: owner }));
      return Promise.resolve(owner);
    });
    vi.mocked(settingsApi.prefsSet).mockResolvedValue(null);
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    await screen.findByRole('heading', { name: message(other, 'auth-login-title') });
    await user.keyboard(`${USERNAME}{Enter}${PASSWORD}{Enter}`);
    expect(await screen.findByRole('heading', { level: 1, name: /Adnan/ })).toBeInTheDocument();
    expect(useLanguageStore.getState().language).toBe(other);
    expect(document.documentElement.lang).toBe(other);
    await waitFor(() => {
      expect(settingsApi.prefsSet).toHaveBeenCalledWith({ key: 'language', value: other });
    });
  });

  it('a wrong password shows under the password field with the tries left, and no raw code', async () => {
    vi.mocked(authApi.authLogin).mockRejectedValue(
      fail({ code: 'ERR_WRONG_PASSWORD', field: 'password', values: { tries_left: 4 } }),
    );
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    await user.keyboard(`${USERNAME}{Enter}wrong{Enter}`);
    const password = field(language, 'auth-login-password');
    await waitFor(() => {
      expect(password).toHaveAttribute('aria-invalid', 'true');
    });
    const note = document.getElementById(password.getAttribute('aria-describedby') ?? '');
    expect(visible(note?.textContent)).toBe(
      message(language, 'err-wrong-password', { tries_left: '4' }),
    );
    expect(document.body.textContent).not.toContain('ERR_');
    // The fields still have what was typed, so he can correct it.
    expect(password.value).toBe('wrong');
  });

  it('a lock after five wrong tries counts down by itself and then lets him try again', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.mocked(authApi.authLogin).mockRejectedValue(
      fail({ code: 'ERR_LOCKED_WAIT', values: { seconds: 3 } }),
    );
    await renderApp();
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    await user.keyboard(`${USERNAME}{Enter}${PASSWORD}{Enter}`);
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(
      message(language, 'err-locked-wait', { seconds: '3' }),
    );
    const submit = screen.getByRole('button', {
      name: new RegExp(message(language, 'auth-login-submit')),
    });
    expect(submit).toBeDisabled();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_000);
    });
    expect(visible(screen.getByRole('alert').textContent)).toContain(
      message(language, 'err-locked-wait', { seconds: '2' }),
    );
    await act(async () => {
      await vi.advanceTimersByTimeAsync(2_000);
    });
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(submit).toBeEnabled();
  });

  it('an inactive user is told so, with the reference number', async () => {
    vi.mocked(authApi.authLogin).mockRejectedValue(fail({ code: 'ERR_USER_INACTIVE' }));
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    await user.keyboard(`${USERNAME}{Enter}${PASSWORD}{Enter}`);
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-user-inactive'));
    expect(alert.querySelector('bdi[dir="ltr"]')).toHaveTextContent(TEST_REFERENCE);
  });

  it('a command that is not ready shows its message as an error, never as made-up data', async () => {
    vi.mocked(authApi.authLogin).mockRejectedValue(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    await user.keyboard(`${USERNAME}{Enter}${PASSWORD}{Enter}`);
    expect(visible((await screen.findByRole('alert')).textContent)).toContain(
      message(language, 'err-not-implemented'),
    );
  });

  it('the show-password eye reveals and hides the password', async () => {
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    const password = field(language, 'auth-login-password');
    expect(password).toHaveAttribute('type', 'password');
    await user.click(
      screen.getByRole('button', { name: message(language, 'common-password-show') }),
    );
    expect(password).toHaveAttribute('type', 'text');
    await user.click(
      screen.getByRole('button', { name: message(language, 'common-password-hide') }),
    );
    expect(password).toHaveAttribute('type', 'password');
  });

  it('Exit closes the app', async () => {
    await renderApp();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'auth-login-exit') }));
    expect(appApi.exitApp).toHaveBeenCalledTimes(1);
  });

  it('switching language in the middle of typing keeps what was typed', async () => {
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    await user.keyboard(`${USERNAME}{Enter}abc`);
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      expect(useLanguageStore.getState().language).toBe(other);
    });
    expect(field(other, 'auth-login-username').value).toBe(USERNAME);
    const password = field(other, 'auth-login-password');
    expect(password.value).toBe('abc');
    expect(password).toHaveFocus();
    expect(
      screen.getByRole('heading', { name: message(other, 'auth-login-title') }),
    ).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('recovery with the code (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    vi.mocked(authApi.authStatus).mockResolvedValue(authState({ login_language: language }));
  });

  async function openRecovery() {
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', { name: message(language, 'auth-login-title') });
    await user.click(screen.getByRole('button', { name: message(language, 'auth-login-forgot') }));
    await screen.findByRole('heading', { name: message(language, 'auth-recover-title') });
    return user;
  }

  it('asks for the user name, the code and a new password twice, and Esc goes back', async () => {
    const user = await openRecovery();
    expect(field(language, 'auth-login-username')).toHaveFocus();
    await user.keyboard('{Escape}');
    expect(
      await screen.findByRole('heading', { name: message(language, 'auth-login-title') }),
    ).toBeInTheDocument();
  });

  it('by keyboard: resets, then shows the new code to print before going on', async () => {
    vi.mocked(authApi.authRecover).mockResolvedValue({ new_recovery_code: 'ABCD1234EFGH5678' });
    const user = await openRecovery();
    await user.keyboard('owner{Enter}7KQ2M9XD4TPAHW3C{Enter}newsecret{Enter}newsecret{Enter}');
    await waitFor(() => {
      expect(authApi.authRecover).toHaveBeenCalledWith({
        username: 'owner',
        recovery_code: '7KQ2M9XD4TPAHW3C',
        new_password: 'newsecret',
        new_password_repeat: 'newsecret',
      });
    });
    // The code is shown in groups of four, left to right.
    const code = await screen.findByLabelText('ABCD1234EFGH5678');
    expect(code).toHaveTextContent('ABCD-1234-EFGH-5678');
    const finish = screen.getByRole('button', {
      name: new RegExp(message(language, 'auth-recover-done-continue')),
    });
    expect(finish).toBeDisabled();
    await user.click(screen.getByRole('checkbox'));
    expect(finish).toBeEnabled();
    await user.click(finish);
    expect(
      await screen.findByRole('heading', { name: message(language, 'auth-login-title') }),
    ).toBeInTheDocument();
  });

  it('a wrong code or user name is shown at the screen, and the typing stays', async () => {
    vi.mocked(authApi.authRecover).mockRejectedValue(fail({ code: 'ERR_RECOVERY_CODE_WRONG' }));
    const user = await openRecovery();
    await user.keyboard('owner{Enter}BADCODE{Enter}newsecret{Enter}newsecret{Enter}');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-recovery-code-wrong'));
    expect(field(language, 'auth-recover-code').value).toBe('BADCODE');
  });

  it('a password problem points at its field', async () => {
    vi.mocked(authApi.authRecover).mockRejectedValue(
      fail({ code: 'ERR_PASSWORDS_DIFFER', field: 'new_password_repeat' }),
    );
    const user = await openRecovery();
    await user.keyboard('owner{Enter}CODE{Enter}one{Enter}two{Enter}');
    const repeat = field(language, 'auth-recover-new-password-repeat');
    await waitFor(() => {
      expect(repeat).toHaveAttribute('aria-invalid', 'true');
    });
    expect(within(document.body).queryByRole('alert')).not.toBeInTheDocument();
  });

  it('switching language in the middle keeps every field', async () => {
    const user = await openRecovery();
    await user.keyboard('owner{Enter}CODE9');
    await user.keyboard('{Alt>}l{/Alt}');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await screen.findByRole('heading', { name: message(other, 'auth-recover-title') });
    expect(field(other, 'auth-login-username').value).toBe('owner');
    expect(field(other, 'auth-recover-code').value).toBe('CODE9');
  });
});

describe('first language of the sign-in screen', () => {
  it('starts in the language last used on this computer (PRD FR-7.30)', async () => {
    resetClientState('ur');
    vi.mocked(authApi.authStatus).mockResolvedValue(authState({ login_language: 'en' }));
    await renderApp();
    expect(
      await screen.findByRole('heading', { name: message('en', 'auth-login-title') }),
    ).toBeInTheDocument();
    expect(document.documentElement.dir).toBe('ltr');
    expect(settingsApi.prefsSet).not.toHaveBeenCalled();
  });
});
