// Test helper for the plan 04 screens: a fake backend (the command wrappers are mocked in each test
// file with vi.mock), and the whole app rendered in a real router with a memory history. Nothing is
// invented for the user to see: every screen shows what these fakes return, or the error they fail
// with, the same as it would with the real commands.

import { createMemoryHistory, RouterProvider } from '@tanstack/react-router';
import { act, fireEvent, render, waitFor } from '@testing-library/react';
import { vi } from 'vitest';
import type { AuthStatus } from '../../src/bindings/AuthStatus';
import type { DayStatus } from '../../src/bindings/DayStatus';
import type { FirmDisplay } from '../../src/bindings/FirmDisplay';
import type { SessionUser } from '../../src/bindings/SessionUser';
import type { Settings } from '../../src/bindings/Settings';
import * as appApi from '../../src/app/api';
import { AppProviders } from '../../src/app/providers';
import { createAppRouter } from '../../src/app/router';
import * as authApi from '../../src/features/auth/api';
import * as settingsApi from '../../src/features/settings/api';
import type { Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { useToastStore } from '../../src/shared/ui';
import { useTextSizeStore } from '../../src/app/layout/text-size';
import { useWizardStore } from '../../src/features/auth/wizard-store';
import { refusal, type RefusalItem } from './ui-error';
import { CommandError } from '../../src/shared/api';

export const FIRM: FirmDisplay = {
  name_ur: 'مہر عدنان اینڈ برادرز',
  name_en: 'Maher Adnan and Brothers',
  mandi_ur: 'نئی سبزی منڈی، کوٹ رادھا کشن',
  mandi_en: 'New Sabzi Mandi, Kot Radha Kishan',
};

export function sessionUser(overrides: Partial<SessionUser> = {}): SessionUser {
  return {
    id: 1,
    username: 'owner',
    display_name: 'Adnan',
    language: 'ur',
    text_size: 100,
    ...overrides,
  };
}

export function authState(overrides: Partial<AuthStatus> = {}): AuthStatus {
  return {
    first_run: false,
    session: null,
    locked: false,
    login_language: 'ur',
    firm: FIRM,
    app_version: '0.1.12',
    ...overrides,
  };
}

export function settings(overrides: Partial<Settings> = {}): Settings {
  return {
    firm: {
      name_ur: FIRM.name_ur,
      name_en: FIRM.name_en,
      mandi_ur: FIRM.mandi_ur,
      mandi_en: FIRM.mandi_en,
      address: 'Shop 1',
      phone: '0300-0000000',
      firm_contacts: [],
      trade_mark: '',
    },
    idle_lock_minutes: 15,
    default_language: 'ur',
    number_grouping: 'international',
    ...overrides,
  };
}

export const DAY: DayStatus = { today: '2026-10-10', closed_up_to: null, today_open: true };

/** A backend refusal, the way a command fails. */
export function fail(...items: RefusalItem[]): CommandError {
  return new CommandError(refusal(items));
}

/**
 * Sets every command to answer like a healthy backend with nobody signed in. A test changes only
 * what it is about. The wrappers must have been mocked with vi.mock in the test file.
 */
export function healthyBackend() {
  vi.mocked(appApi.appDataStatus).mockResolvedValue({ state: 'ok', reference: null });
  vi.mocked(appApi.dayStatus).mockResolvedValue(DAY);
  vi.mocked(appApi.exitApp).mockResolvedValue(undefined);
  vi.mocked(authApi.authStatus).mockResolvedValue(authState());
  vi.mocked(authApi.authLock).mockResolvedValue(null);
  vi.mocked(authApi.authLogout).mockResolvedValue(null);
  vi.mocked(settingsApi.settingsGet).mockResolvedValue(settings());
  vi.mocked(settingsApi.prefsSet).mockResolvedValue(null);
}

/** Who is signed in, for the next answers of auth_status. */
export function signedIn(user: SessionUser = sessionUser(), locked = false) {
  vi.mocked(authApi.authStatus).mockResolvedValue(authState({ session: user, locked }));
}

/** Resets everything the screens keep outside React, so one test never leaks into the next. */
export function resetClientState(language: Language = 'ur') {
  act(() => {
    useLanguageStore.setState({ language });
    useTextSizeStore.setState({ chosen: 100 });
    useWizardStore.getState().reset();
    // A message of an earlier test must not be on the screen of the next one.
    useToastStore.setState({ toasts: [] });
  });
  document.documentElement.style.zoom = '';
  try {
    // What a screen keeps on this computer ("More filters" open, "More columns" ticked) starts fresh.
    window.localStorage.clear();
  } catch {
    // Storage can be blocked; nothing is kept then either.
  }
}

/** Renders the whole app on a path, as the window does after the start-up check. */
export async function renderApp(path = '/') {
  Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true });
  const router = createAppRouter(createMemoryHistory({ initialEntries: [path] }));
  await router.load();
  const result = render(
    <AppProviders>
      <RouterProvider router={router} />
    </AppProviders>,
  );
  await waitFor(() => {
    expectSomethingShown();
  });
  return { router, ...result };
}

function expectSomethingShown() {
  if (document.body.textContent === '') throw new Error('nothing on screen yet');
}

export function leaveAppWindow() {
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__');
}

/**
 * Presses a key by its physical code on the element that has focus, the way the window gets it.
 * user-event does not know the F keys, so the shortcuts of PRD 7.3 (F2, F9, F10) use this.
 */
export function press(code: string, init: KeyboardEventInit = {}) {
  act(() => {
    fireEvent.keyDown(document.activeElement ?? document.body, { code, key: code, ...init });
  });
}
