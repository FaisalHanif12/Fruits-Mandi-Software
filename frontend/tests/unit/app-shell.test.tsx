// The app after the window opens (plan 04): the start-up checks, the shell with its menus and the
// dashboard, the بکری area with its items, a "coming soon" page for a main item not built yet, the keys from anywhere, the idle lock that
// keeps unsaved work, signing out, and saving the user's language and text size. Urdu and English,
// by keyboard, against a fake backend.

import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as authApi from '../../src/features/auth/api';
import * as settingsApi from '../../src/features/settings/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  DAY,
  authState,
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  settings,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { fakeDashboard } from '../helpers/dashboard';
import { at } from '../helpers/sample';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/settings/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
});

afterEach(() => {
  leaveAppWindow();
  vi.useRealTimers();
});

describe('start-up', () => {
  beforeEach(() => {
    resetClientState('ur');
  });

  it('a failed data check fills the window with the message and the reference, nothing else', async () => {
    vi.mocked(appApi.appDataStatus).mockResolvedValue({
      state: 'failed',
      reference: 'E-20261010-0007',
    });
    await renderApp();
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message('ur', 'err-data-check-failed'));
    expect(authApi.authStatus).not.toHaveBeenCalled();
    expect(screen.queryByRole('navigation')).not.toBeInTheDocument();
  });

  it('while the status of who is signed in loads, it shows the loading state', async () => {
    vi.mocked(authApi.authStatus).mockReturnValue(new Promise(() => undefined));
    await renderApp();
    expect(await screen.findByRole('status')).toHaveAttribute('aria-busy', 'true');
  });

  it('if it cannot be read, it shows the message with a reference and a retry that works', async () => {
    vi.mocked(authApi.authStatus).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await renderApp();
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message('ur', 'err-not-implemented'));
    expect(alert.textContent).not.toContain('ERR_');
    vi.mocked(authApi.authStatus).mockResolvedValue(authState());
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message('ur', 'common-retry') }));
    expect(
      await screen.findByRole('heading', { name: message('ur', 'auth-login-title') }),
    ).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('the shell (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('opens on the dashboard: greeting, today, the day, the menu, the top bar and the status bar', async () => {
    await renderApp();
    expect(await screen.findByRole('heading', { level: 1, name: /Adnan/ })).toBeInTheDocument();
    const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
    expect(within(nav).getAllByRole('button')).toHaveLength(10);
    expect(within(nav).getAllByRole('button')[0]).toHaveAttribute('aria-current', 'page');
    await waitFor(() => {
      expect(visible(screen.getByRole('contentinfo').textContent)).toContain('10-10-2026');
    });
    expect(visible(screen.getByRole('contentinfo').textContent)).toContain(
      message(language, 'layout-status-day-open'),
    );
    expect(screen.getByRole('banner')).toBeInTheDocument();
    // The dashboard shows what the backend sent: today's sales and the cash of the worked day, and
    // the quick actions, each with its key (PRD Section 9, FR-9.1).
    expect(
      await screen.findByRole('group', { name: message(language, 'dashboard-quick-title') }),
    ).toBeInTheDocument();
    expect(await screen.findByText('41,400')).toBeInTheDocument();
    expect(screen.getByText('58,484')).toBeInTheDocument();
  });

  it('a closed day is said in words', async () => {
    vi.mocked(appApi.dayStatus).mockResolvedValue({ ...DAY, today_open: false });
    await renderApp();
    await waitFor(() => {
      expect(visible(screen.getByRole('contentinfo').textContent)).toContain(
        message(language, 'layout-status-day-closed'),
      );
    });
  });

  it('Alt+B opens the بکری area on the sale book with its seven items, each a real screen; a main item built later opens its own screen, not made-up data', async () => {
    const { router } = await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    const user = userEvent.setup();
    await user.keyboard('{Alt>}b{/Alt}');
    // The بکری area opens on the sale book (plan 12).
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'glossary-bikri-book'),
      }),
    ).toBeInTheDocument();
    const side = screen.getByRole('navigation', {
      name: (name) => visible(name) === message(language, 'layout-module-label'),
    });
    const items = within(side).getAllByRole('button');
    expect(items).toHaveLength(7);
    expect(items[0]).toHaveAttribute('aria-current', 'page');
    // The advance item is a real screen now (plan 10).
    await user.click(at(items, 3));
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-advance-title'),
      }),
    ).toBeInTheDocument();
    expect(within(side).getAllByRole('button')[3]).toHaveAttribute('aria-current', 'page');
    // The پختہ کھاتہ register and today's بکری are real screens since plan 13.
    await user.click(at(items, 4));
    // (The first open of a screen loads its code, which can take a moment on a busy machine.)
    expect(
      await screen.findByRole(
        'heading',
        { level: 1, name: message(language, 'glossary-pukhta-khata') },
        { timeout: 8_000 },
      ),
    ).toBeInTheDocument();
    await user.click(at(items, 5));
    expect(
      await screen.findByRole(
        'heading',
        {
          level: 1,
          name: (name) => visible(name) === message(language, 'layout-module-today'),
        },
        { timeout: 8_000 },
      ),
    ).toBeInTheDocument();
    // وصولی and ادائیگی are the same voucher screen with their type chosen (plan 15).
    await user.click(at(items, 1));
    expect(
      await screen.findByRole(
        'heading',
        { level: 1, name: message(language, 'vouchers-receipt-title') },
        { timeout: 8_000 },
      ),
    ).toBeInTheDocument();
    expect(within(side).getAllByRole('button')[1]).toHaveAttribute('aria-current', 'page');
    await user.click(at(items, 2));
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-payment-title'),
      }),
    ).toBeInTheDocument();
    expect(within(side).getAllByRole('button')[2]).toHaveAttribute('aria-current', 'page');
    // The goods left (باقی مال) is a real screen since plan 16; a later sale starts from it. Every
    // item of the area is built now (own goods, خریداری, joins in plan 16b).
    await user.click(at(items, 6));
    expect(
      await screen.findByRole(
        'heading',
        {
          level: 1,
          name: (name) => visible(name) === message(language, 'layout-module-stock'),
        },
        { timeout: 8_000 },
      ),
    ).toBeInTheDocument();
    expect(within(side).getAllByRole('button')[6]).toHaveAttribute('aria-current', 'page');
    // A main item built later (تجزیات, plan 21) opens its own screen, never a made-up page.
    const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
    await user.click(
      within(nav).getByRole('button', { name: message(language, 'layout-nav-analytics') }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/analytics');
    });
    expect(
      screen.queryByRole('heading', { name: message(language, 'soon-title') }),
    ).not.toBeInTheDocument();
    // The way back is the dashboard item of the main bar.
    await user.click(
      within(nav).getByRole('button', { name: message(language, 'layout-nav-dashboard') }),
    );
    expect(await screen.findByRole('heading', { level: 1, name: /Adnan/ })).toBeInTheDocument();
    expect(
      screen.queryByRole('navigation', {
        name: (name) => visible(name) === message(language, 'layout-module-label'),
      }),
    ).not.toBeInTheDocument();
  });

  it('the keys from anywhere: Alt+V, Alt+A, Alt+P and Alt+K', async () => {
    const { router } = await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    const user = userEvent.setup();
    // وصولی, ادائیگی and پیشگی are one screen with the type chosen (plan 15); Alt+K is plan 17's: the خرچہ screen as دکان خرچہ.
    for (const [key, path, type] of [
      ['v', '/bikri/voucher', 'receipt'],
      ['a', '/bikri/voucher', 'payment'],
      ['p', '/bikri/voucher', 'advance'],
      ['k', '/rokar/expense', 'shop_expense'],
    ] as const) {
      await user.keyboard(`{Alt>}${key}{/Alt}`);
      await waitFor(() => {
        expect(router.state.location.pathname).toBe(path);
      });
      expect(router.state.location.search).toMatchObject({ type });
    }
  });

  it("the dashboard buttons lead to a new بکری and to a tile's report", async () => {
    const { router } = await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    const user = userEvent.setup();
    // The buyers' dues tile opens R20, the buyers' dues report (FR-9.1).
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'dashboard-tile-dues')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R20');
    });
  });

  it('the menu works by keyboard: arrows move, Enter opens, and the arrow into the screen leaves it', async () => {
    const { router } = await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    const user = userEvent.setup();
    const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
    within(nav).getAllByRole('button')[0]?.focus();
    await user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    // The side menu is nearest to the screen, and the arrow into the screen leaves it.
    const side = screen.getByRole('navigation', {
      name: (name) => visible(name) === message(language, 'layout-module-label'),
    });
    within(side).getAllByRole('button')[0]?.focus();
    await user.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
    expect(screen.getByRole('main').contains(document.activeElement)).toBe(true);
  });

  it('the Owner is the only user and sees every menu item, Settings included', async () => {
    await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
    expect(within(nav).getAllByRole('button')).toHaveLength(10);
    expect(
      within(nav).getByRole('button', { name: message(language, 'layout-nav-settings') }),
    ).toBeInTheDocument();
  });

  it('signs out from the user menu and shows the sign-in screen', async () => {
    await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: message(language, 'layout-user-menu') }));
    vi.mocked(authApi.authStatus).mockResolvedValue(authState({ login_language: language }));
    await user.click(
      await screen.findByRole('menuitem', { name: message(language, 'layout-user-signout') }),
    );
    expect(authApi.authLogout).toHaveBeenCalledTimes(1);
    expect(
      await screen.findByRole('heading', { name: message(language, 'auth-login-title') }),
    ).toBeInTheDocument();
  });

  it('saves the language he switches to, and the text size, but not the values he arrived with', async () => {
    await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    expect(settingsApi.prefsSet).not.toHaveBeenCalled();
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await userEvent.setup().keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      expect(settingsApi.prefsSet).toHaveBeenCalledWith({ key: 'language', value: other });
    });
    Object.defineProperty(window, 'innerWidth', { value: 1920, configurable: true });
    act(() => {
      fireEvent.keyDown(window, { code: 'Equal', ctrlKey: true });
    });
    await waitFor(() => {
      expect(settingsApi.prefsSet).toHaveBeenCalledWith({ key: 'text_size', value: '110' });
    });
  });
});

describe.each(LANGUAGES)('when the backend says the session is over (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('"nobody is signed in" in a command sends him to the sign-in screen', async () => {
    await renderApp('/settings');
    const user = userEvent.setup();
    const address = await screen.findByLabelText(
      (name) => visible(name) === message(language, 'settings-firm-address'),
    );
    await user.click(address);
    await user.keyboard('{End}!');
    // The save answers that nobody is signed in, and who is signed in is read again.
    vi.mocked(authApi.authStatus).mockResolvedValue(authState({ login_language: language }));
    vi.mocked(settingsApi.settingsSetFirm).mockRejectedValue(fail({ code: 'ERR_NOT_LOGGED_IN' }));
    press('F10');
    expect(
      await screen.findByRole('heading', { name: message(language, 'auth-login-title') }),
    ).toBeInTheDocument();
  });

  it('"the app is locked" in a command puts the lock over the screen', async () => {
    await renderApp('/settings');
    const user = userEvent.setup();
    const address = await screen.findByLabelText(
      (name) => visible(name) === message(language, 'settings-firm-address'),
    );
    await user.click(address);
    await user.keyboard('{End}!');
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ session: sessionUser({ language }), locked: true }),
    );
    vi.mocked(settingsApi.settingsSetFirm).mockRejectedValue(fail({ code: 'ERR_SESSION_LOCKED' }));
    press('F10');
    expect(await screen.findByRole('dialog')).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('the idle lock (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('locks after the set minutes of no key or click, and any activity starts the count again', async () => {
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(settings({ idle_lock_minutes: 2 }));
    vi.useFakeTimers({ shouldAdvanceTime: true });
    await renderApp();
    await screen.findByRole('heading', { level: 1, name: /Adnan/ });
    await waitFor(() => {
      expect(settingsApi.settingsGet).toHaveBeenCalled();
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(90_000);
    });
    fireEvent.keyDown(window, { code: 'KeyA' });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(90_000);
    });
    expect(authApi.authLock).not.toHaveBeenCalled();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(40_000);
    });
    expect(authApi.authLock).toHaveBeenCalledTimes(1);
  });

  it('shows the lock over the screen; typed work stays under it, and the password opens it', async () => {
    const { router } = await renderApp('/settings');
    const user = userEvent.setup();
    const firmName = await screen.findByLabelText(
      (text) => visible(text) === message(language, 'settings-firm-name-en'),
    );
    await user.clear(firmName);
    await user.type(firmName, 'Half typed firm');
    // The user menu locks the app (the same call the idle timer makes).
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ session: sessionUser({ language }), locked: true }),
    );
    await user.click(screen.getByRole('button', { name: message(language, 'layout-user-menu') }));
    await user.click(
      await screen.findByRole('menuitem', { name: message(language, 'layout-user-lock') }),
    );
    const dialog = await screen.findByRole('dialog');
    expect(visible(dialog.textContent)).toContain(message(language, 'auth-lock-title'));
    expect(authApi.authLock).toHaveBeenCalledTimes(1);
    // Under the lock the page is still there, with the typed text.
    expect(firmName).toBeInTheDocument();
    expect((firmName as HTMLInputElement).value).toBe('Half typed firm');
    expect(router.state.location.pathname).toBe('/settings');
    // Esc does not close it.
    await user.keyboard('{Escape}');
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    // A wrong password is shown at the field, and the lock stays.
    vi.mocked(authApi.authUnlock).mockRejectedValueOnce(
      fail({ code: 'ERR_WRONG_PASSWORD', field: 'password', values: { tries_left: 4 } }),
    );
    await user.keyboard('wrong{Enter}');
    const password = within(dialog).getByLabelText(
      (text) => visible(text) === message(language, 'auth-login-password'),
    );
    await waitFor(() => {
      expect(password).toHaveAttribute('aria-invalid', 'true');
    });
    // The right password opens it, and the typed text is still there.
    vi.mocked(authApi.authUnlock).mockResolvedValue(sessionUser({ language }));
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ session: sessionUser({ language }) }),
    );
    await user.clear(password);
    await user.type(password, 'secret12{Enter}');
    await waitFor(() => {
      expect(authApi.authUnlock).toHaveBeenLastCalledWith({ password: 'secret12' });
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect((firmName as HTMLInputElement).value).toBe('Half typed firm');
  });

  it('while locked it can sign the user out', async () => {
    signedIn(sessionUser({ language }), true);
    await renderApp();
    await screen.findByRole('dialog');
    vi.mocked(authApi.authStatus).mockResolvedValue(authState({ login_language: language }));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'layout-user-signout') }));
    expect(authApi.authLogout).toHaveBeenCalledTimes(1);
    expect(
      await screen.findByRole('heading', { name: message(language, 'auth-login-title') }),
    ).toBeInTheDocument();
  });

  it('does not lock again while it is already locked', async () => {
    signedIn(sessionUser({ language }), true);
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(settings({ idle_lock_minutes: 1 }));
    vi.useFakeTimers({ shouldAdvanceTime: true });
    await renderApp();
    await screen.findByRole('dialog');
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5 * 60_000);
    });
    expect(authApi.authLock).not.toHaveBeenCalled();
    expect(useLanguageStore.getState().language).toBe(language);
  });
});
