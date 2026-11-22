// The backend's REAL answers of "see the backup key again" (PRD FR-20.10 v1.54): the key shown after a right password
// and sign, and the two refusals that show no key. The backend wrote them in its own tests
// (backend/crates/backup/tests/fixtures); they are copied here byte for byte and never edited. The key in the first is
// the backend's stand-in `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`, as the request's password and sign are stand-ins too.

import { CommandError } from '../../src/shared/api';
import type { UiError } from '../../src/bindings/UiError';
import type { KeyRevealInput } from '../../src/features/backup/types';
import type { KeyShown } from '../../src/features/backup/types';
import reveal from '../fixtures/backup_key_reveal.json';
import revealRequest from '../fixtures/backup_key_reveal_request.json';
import wrongPassword from '../fixtures/backup_key_reveal_wrong_password.json';
import wrongSign from '../fixtures/backup_key_reveal_wrong_sign.json';

export const KEY_REVEAL: KeyShown = reveal;
export const KEY_REVEAL_REQUEST: KeyRevealInput = revealRequest;

/** The refusals as the backend sends them on the wire, for the development page's stand-in. */
export const WRONG_PASSWORD_WIRE: unknown = wrongPassword;
export const WRONG_SIGN_WIRE: unknown = wrongSign;

/** The refusal of a wrong password: ERR_WRONG_PASSWORD at `password`, with the tries left. */
export function wrongPasswordRefusal(): CommandError {
  return new CommandError(wrongPassword as unknown as UiError);
}

/** The refusal of a wrong sign: ERR_SIGN_WRONG at `sign`. */
export function wrongSignRefusal(): CommandError {
  return new CommandError(wrongSign as unknown as UiError);
}

export const WRONG_PASSWORD_TRIES_LEFT = 9;
