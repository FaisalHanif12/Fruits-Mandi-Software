// Reads a JSON file the suite wrote itself (timings, the screenshot index).

import { existsSync, readFileSync } from 'node:fs';

/**
 * @template T
 * @param {string} file
 * @param {T} fallback what to return when the file is not there yet
 * @returns {T}
 */
export function readJson(file, fallback) {
  if (!existsSync(file)) return fallback;
  const parsed = /** @type {unknown} */ (JSON.parse(readFileSync(file, 'utf8')));
  return /** @type {T} */ (parsed);
}
