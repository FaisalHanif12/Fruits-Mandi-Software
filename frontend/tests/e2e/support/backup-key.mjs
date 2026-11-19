// The backup key made in the first-run wizard (spec 01), kept for the later specs of the same
// suite run. The suite starts on a computer with no app data, so this key exists for this run only
// and protects nothing. A later spec that restores a locked copy types it, as the Owner would from
// his printed sheet.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { OUTPUT_DIR } from '../paths.mjs';

const KEY_FILE = path.join(OUTPUT_DIR, 'backup-key.txt');

/** @param {string} key the key as the wizard showed it, XXXX-XXXX-XXXX-XXXX-XXXX-XXXX */
export function rememberKey(key) {
  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(KEY_FILE, key, 'utf8');
}

/** @returns {string} the key spec 01 saw */
export function recalledKey() {
  if (!existsSync(KEY_FILE)) throw new Error('Spec 01 has not made the backup key yet.');
  return readFileSync(KEY_FILE, 'utf8').trim();
}
