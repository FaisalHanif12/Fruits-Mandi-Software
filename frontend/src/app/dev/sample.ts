// Samples for the gallery, read the way a backend reply is read: as JSON, then checked.
// App code never makes Paisa itself (Code Standards F3, M12); this decodes a fixed sample reply.

import type { UiError } from '../../bindings/UiError';
import { isUiError } from '../../shared/api';
import type { Paisa } from '../../shared/format';

/** A sample reply: 12,880, 18,684, 1,316,040 and 600 rupees, in paisa. */
const SAMPLE_REPLY = '[1288000, 1868400, 131604000, 60000]';

function isPaisaList(value: unknown): value is Paisa[] {
  return (
    Array.isArray(value) &&
    value.every((item) => typeof item === 'number' && Number.isSafeInteger(item))
  );
}

function decode(): readonly Paisa[] {
  const value: unknown = JSON.parse(SAMPLE_REPLY);
  if (!isPaisaList(value)) throw new Error('gallery sample reply is not a list of paisa');
  return value;
}

export const SAMPLE_AMOUNTS: readonly Paisa[] = decode();

/** A sample truck number, as a backend reply would carry it (a code, shown as typed). */
export const SAMPLE_TRUCK = String(JSON.parse('"LES-1234"'));

/** A sample reference number, in the form the backend sends (plan 03 contract 4.6). */
export const SAMPLE_REFERENCE = 'E-20261010-0007';

/** The field a sample backend error points at (a grid cell path, Code Standards 4.3). */
export const SAMPLE_ERROR_FIELD = 'lots[0].sales[2].qty';

/**
 * A sample refusal with two problems: one on a grid cell, one on the whole form. Read like a
 * backend reply, so it passes through the same check as a real one.
 */
const SAMPLE_REFUSAL = `{"ok":false,"errors":[{"code":"ERR_QTY_TOO_LARGE","field":"${SAMPLE_ERROR_FIELD}","values":{"max":1000000}},{"code":"ERR_DATE_CLOSED","field":null,"values":{"closed_up_to":"09-10-2026"}}],"ref":"${SAMPLE_REFERENCE}"}`;

function decodeRefusal(): UiError {
  const value: unknown = JSON.parse(SAMPLE_REFUSAL);
  if (!isUiError(value)) throw new Error('gallery sample refusal is not a UiError');
  return value;
}

export const SAMPLE_BACKEND_ERROR: UiError = decodeRefusal();

/** A sample signed-in user's name, as the backend would send it (a name is shown as typed). */
export const SAMPLE_USER = String(JSON.parse('"Owner"'));
