// pnpm i18n:check (NFR-URD-22). Always runs the check and always prints one result line.
// The check itself is in i18n-check-lib.mjs, so tests can import it without running this file.
// Exit 0: ur and en match. Exit 1: problems found, or the check itself could not run.

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkI18n } from './i18n-check-lib.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const i18nDir = path.join(root, 'frontend', 'src', 'i18n');

let errors;
try {
  errors = checkI18n(i18nDir);
} catch (error) {
  console.error(
    `i18n:check could not run: ${error instanceof Error ? error.message : String(error)}`,
  );
  process.exit(1);
}

if (errors.length > 0) {
  console.error(`i18n:check FAILED with ${errors.length} problem(s):`);
  for (const error of errors) console.error(`  - ${error}`);
  process.exit(1);
}
console.log('i18n:check passed: ur and en match. No missing, empty or untranslated messages.');
