#!/usr/bin/env node
// pnpm seed: makes dev-data/mandi.db with the full schema, the system accounts of PRD FR-10.4,
// the masters and parties of the worked day, and the PRD 4.3 opening position, all through the
// real services (plan 10), and prints the E2E-19 numbers. dev-data/ is ignored by git.
//
// Usage: pnpm seed [-- --reset]
//   --reset   delete the old dev database first
//
// The audit key: by default the seed reads the audit key from the real key store (Windows
// Credential Manager, or the macOS Keychain on a Mac), like the app does. On a Mac a Keychain
// prompt may appear: the user must enter the Mac password and choose Always Allow. Or run
// `MANDI_SEED_TEST_KEY=1 pnpm seed` for a throwaway dev key that is never used for real data.
// That key is a fixed one held in memory (any value of the variable turns it on, even 0). A dev
// database made with it cannot be verified by the real app on the same keychain: the audit
// chain check reports a break. Use a database made with the real key for anything the app opens.
//
// The project folder has spaces in its name. Paths are built with node:path and cargo is
// started without a shell, so nothing is split.

import { spawnSync } from 'node:child_process';
import { mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const devData = path.join(root, 'dev-data');
const database = path.join(devData, 'mandi.db');

mkdirSync(devData, { recursive: true });
if (process.argv.includes('--reset')) {
  for (const suffix of ['', '-wal', '-shm']) {
    rmSync(database + suffix, { force: true });
  }
  rmSync(path.join(devData, 'backups'), { recursive: true, force: true });
}

// The example turns the test key on when the variable is set to anything, so this checks the same.
if (!('MANDI_SEED_TEST_KEY' in process.env)) {
  console.error(
    'seed: a Mac Keychain prompt may appear. Enter the Mac password and choose Always Allow, ' +
      'or run `MANDI_SEED_TEST_KEY=1 pnpm seed` for a throwaway dev key that is never used for ' +
      'real data (a database made with it cannot be verified by the real app).',
  );
}

const run = spawnSync(
  'cargo',
  ['run', '--quiet', '-p', 'mandi-services', '--example', 'seed_dev', '--', database],
  { cwd: path.join(root, 'backend'), stdio: 'inherit' },
);
if (run.error) {
  console.error(`seed: could not run cargo: ${run.error.message}`);
  process.exit(1);
}
process.exit(run.status ?? 1);
