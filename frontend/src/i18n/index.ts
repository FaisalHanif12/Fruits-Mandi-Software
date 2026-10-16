// Both Fluent bundles are built once at start, so a language switch needs no loading
// (Tech Stack 3 and 4.2, PRD FR-7.33). The files are bundled into the app: nothing is fetched.

import { FluentBundle, FluentResource } from '@fluent/bundle';

export type Language = 'ur' | 'en';

export const LANGUAGES: readonly Language[] = ['ur', 'en'];

/** Urdu is the default for a new user (PRD FR-7.30). */
export const DEFAULT_LANGUAGE: Language = 'ur';

const sources: Record<Language, Record<string, string>> = {
  ur: import.meta.glob<string>('./ur/*.ftl', { query: '?raw', import: 'default', eager: true }),
  en: import.meta.glob<string>('./en/*.ftl', { query: '?raw', import: 'default', eager: true }),
};

/**
 * Builds the bundle for one language from all of its .ftl files, plus `extra` files (the
 * development gallery adds its own words this way, so they never ship). A file that cannot be
 * read is a programming error (pnpm i18n:check catches it first), so it stops the app loudly.
 */
export function createBundle(language: Language, extra: Record<string, string> = {}): FluentBundle {
  // Keep Fluent's bidi isolation on, so numbers and names never flip inside Urdu (NFR-URD-13).
  const bundle = new FluentBundle(language, { useIsolating: true });
  for (const [path, source] of Object.entries({ ...sources[language], ...extra }).sort(([a], [b]) =>
    a.localeCompare(b),
  )) {
    const errors = bundle.addResource(new FluentResource(source));
    const first = errors[0];
    if (first !== undefined) {
      throw new Error(`Fluent file ${path} cannot be read: ${first.message}`);
    }
  }
  return bundle;
}

export const bundles: Record<Language, FluentBundle> = {
  ur: createBundle('ur'),
  en: createBundle('en'),
};
