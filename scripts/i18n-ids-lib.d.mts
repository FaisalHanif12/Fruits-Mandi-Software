// Types for scripts/i18n-ids-lib.mjs, so the frontend tests can import it with full type checking.
export declare function definedIds(i18nDir: string): Record<string, Set<string>>;
export declare function checkFrontendIds(
  root: string,
  defined: Record<string, Set<string>>,
  options?: { typescriptFrom?: string },
): string[];
export declare function checkBackendIds(
  root: string,
  defined: Record<string, Set<string>>,
): string[];
export declare function checkBackendTemplates(
  root: string,
  defined: Record<string, Set<string>>,
): string[];
export declare function checkMessageIds(root: string): string[];
