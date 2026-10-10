// Turns backend error codes into Fluent message IDs (Code Standards 4.3, NFR-USE-14).
// ERR_QTY_EXCEEDS_LOT becomes err-qty-exceeds-lot. The message lives in i18n/<lang>/errors.ftl.

import type { ErrorCode } from '../../bindings/ErrorCode';
import type { UiError } from '../../bindings/UiError';
import { CommandError, toUiError } from './invoke';

/** The Fluent message ID for an error code. */
export function errorMessageId(code: ErrorCode): string {
  return `err-${code.replace(/^ERR_/, '').toLowerCase().replaceAll('_', '-')}`;
}

/** The UiError inside anything a query or mutation failed with. */
export function uiErrorOf(error: unknown): UiError {
  return error instanceof CommandError ? error.uiError : toUiError(error);
}
