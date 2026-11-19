// Reading the problems a backup command reports. A backup command may send a code that core does
// not have yet (`BackupCode`, plan 18), while the shared error type lists only core's codes. Both
// are the text `ERR_...` on the wire, so one list of codes covers them (`WireCode`).

import type { UiError } from '../../bindings/UiError';
import type { WireCode } from '../../bindings/WireCode';

/** True when the error carries this code, whichever list it comes from. */
export function hasWireCode(error: UiError, code: WireCode): boolean {
  return error.errors.some((item) => item.code === code);
}
