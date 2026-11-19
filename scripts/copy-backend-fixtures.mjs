// Copies the backend's contract fixtures into frontend/tests/fixtures, byte for byte. The backend wrote them in
// its own tests (backend/crates/services/tests/fixtures); the frontend never edits them, it only reads
// them as the generated types (tests/helpers/*-real.ts) and the contract tests compare each copy with
// its source. When backend changes a type or a fixture after a copy, run this again and the tests say
// what moved.
//
//   node scripts/copy-backend-fixtures.mjs                 # reports_ and month_
//   node scripts/copy-backend-fixtures.mjs analytics_      # another family
//
// It prints one line for each file: new, changed or same. It lists a copy here that the backend no
// longer has (stale); it never deletes anything.

import { copyFileSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SOURCE = path.join(ROOT, 'backend', 'crates', 'services', 'tests', 'fixtures');
const TARGET = path.join(ROOT, 'frontend', 'tests', 'fixtures');
const DEFAULT_PREFIXES = ['reports_', 'month_'];

const given = process.argv.slice(2);
const prefixes = given.length > 0 ? given : DEFAULT_PREFIXES;

/** The names of the .json files that start with one of the prefixes. @param {string} folder */
function namesIn(folder) {
  return readdirSync(folder).filter(
    (name) => name.endsWith('.json') && prefixes.some((prefix) => name.startsWith(prefix)),
  );
}

/** True when both files hold the same bytes. @param {string} a @param {string} b */
function sameBytes(a, b) {
  return readFileSync(a).equals(readFileSync(b));
}

const counts = { new: 0, changed: 0, same: 0 };
const sourceNames = namesIn(SOURCE);
for (const name of sourceNames) {
  const from = path.join(SOURCE, name);
  const to = path.join(TARGET, name);
  let state = 'new';
  try {
    state = sameBytes(from, to) ? 'same' : 'changed';
  } catch {
    state = 'new';
  }
  if (state !== 'same') copyFileSync(from, to);
  counts[/** @type {'new' | 'changed' | 'same'} */ (state)] += 1;
  if (state !== 'same') console.log(`${state.padEnd(8)}${name}`);
}

const stale = namesIn(TARGET).filter((name) => !sourceNames.includes(name));
for (const name of stale) console.log(`stale   ${name}`);
console.log(
  `${String(sourceNames.length)} fixtures read: ${String(counts.new)} new, ` +
    `${String(counts.changed)} changed, ${String(counts.same)} same, ${String(stale.length)} stale.`,
);
