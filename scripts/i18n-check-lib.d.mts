// Types for scripts/i18n-check-lib.mjs, so the frontend tests can import it with full type checking.
export declare const IDENTICAL_ALLOWED: Set<string>;
export declare function checkI18n(
  i18nDir: string,
  options?: { identicalAllowed?: Set<string> },
): string[];
