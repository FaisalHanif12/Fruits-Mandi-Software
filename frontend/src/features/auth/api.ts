// Typed wrappers around the sign-in commands (plan 04 contract 4). Components never call
// invoke() (Code Standards F4). Every input is text as typed; the backend checks it.

import type { AuthStatus } from '../../bindings/AuthStatus';
import type { ChangePasswordInput } from '../../bindings/ChangePasswordInput';
import type { FirstRunDone } from '../../bindings/FirstRunDone';
import type { FirstRunInput } from '../../bindings/FirstRunInput';
import type { LoginInput } from '../../bindings/LoginInput';
import type { RecoverDone } from '../../bindings/RecoverDone';
import type { RecoverInput } from '../../bindings/RecoverInput';
import type { SessionUser } from '../../bindings/SessionUser';
import type { UnlockInput } from '../../bindings/UnlockInput';
import { callCommand } from '../../shared/api';

/** First run, the signed-in user, the idle lock, the firm names and the app version. No login. */
export function authStatus(): Promise<AuthStatus> {
  return callCommand<AuthStatus>('auth_status');
}

/** Creates the firm and the Owner. Only while no user exists. Returns the recovery code. */
export function setupFirstRun(input: FirstRunInput): Promise<FirstRunDone> {
  return callCommand<FirstRunDone>('setup_first_run', { input });
}

export function authLogin(input: LoginInput): Promise<SessionUser> {
  return callCommand<SessionUser>('auth_login', { input });
}

export function authLogout(): Promise<null> {
  return callCommand<null>('auth_logout');
}

export function authLock(): Promise<null> {
  return callCommand<null>('auth_lock');
}

export function authUnlock(input: UnlockInput): Promise<SessionUser> {
  return callCommand<SessionUser>('auth_unlock', { input });
}

/** Resets the Owner's password with the recovery code. Returns a new code to print. */
export function authRecover(input: RecoverInput): Promise<RecoverDone> {
  return callCommand<RecoverDone>('auth_recover', { input });
}

/** Changes the Owner's password from the user menu (PRD FR-5.6). A wrong current password counts towards the lockout. */
export function authChangePassword(input: ChangePasswordInput): Promise<null> {
  return callCommand<null>('auth_change_password', { input });
}
