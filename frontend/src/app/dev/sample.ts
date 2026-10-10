// Sample amounts for the gallery, read the way a backend reply is read: as JSON, then checked.
// App code never makes Paisa itself (Code Standards F3, M12); this decodes a fixed sample reply.

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
