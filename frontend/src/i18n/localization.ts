// The one place a ReactLocalization is made (app, print window, tests). It records every message
// ID that no bundle has, so a missing word is found by the tests, not by the Owner. A raw ID such
// as `print-partner-month-col-advances` on a printed header in Urdu is English in the middle of
// Urdu, and a bug (NFR-URD-1, NFR-URD-22, plan 81).
//
// The tests read the record after each test (tests/setup.ts) and fail on any entry.

import { ReactLocalization } from '@fluent/react';
import { bundles, type Language } from './index';

/** Fluent's wording when a message is not in any bundle. */
const MISSING = /^The id "(.+)" did not match any messages/;

const missing = new Set<string>();

/** Remembers that `id` was asked for in `language` and is not there. */
export function recordMissingMessage(language: Language, id: string): void {
  missing.add(`${language}: ${id}`);
}

/** The IDs missing so far, as "ur: some-id", and forgets them. */
export function takeMissingMessages(): string[] {
  const found = [...missing];
  missing.clear();
  return found;
}

/**
 * A localization for one language that records a missing message. A caller that gives a fallback
 * (`getString(id, vars, fallback)`) expects some IDs to be missing (a code the app has no words
 * for), so that call is not recorded. Other Fluent problems (a variable not given) are not logged
 * here: the code standards allow no console output (G4), and `message()` in the tests reads them
 * as errors.
 */
class RecordingLocalization extends ReactLocalization {
  constructor(language: Language) {
    super([bundles[language]], undefined, (error) => {
      const id = MISSING.exec(error.message)?.[1];
      if (id !== undefined) recordMissingMessage(language, id);
    });
  }

  override getString(
    id: string,
    vars?: Parameters<ReactLocalization['getString']>[1],
    fallback?: string,
  ): string {
    if (fallback !== undefined && this.getBundle(id) === null) return fallback;
    return super.getString(id, vars, fallback);
  }
}

export function createLocalization(language: Language): ReactLocalization {
  return new RecordingLocalization(language);
}
