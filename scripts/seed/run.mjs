#!/usr/bin/env node
// pnpm seed: makes dev-data/mandi.db with the full schema and the system accounts of
// PRD FR-10.4, and prints the counts (plan 03 Section 4.4). dev-data/ is ignored by git.
//
// Usage: node scripts/seed/run.mjs [--reset]
//   --reset   delete the old dev database first
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

const run = spawnSync(
  'cargo',
  ['run', '--quiet', '-p', 'mandi-db', '--example', 'seed', '--', database],
  { cwd: path.join(root, 'backend'), stdio: 'inherit' },
);
if (run.error) {
  console.error(`seed: could not run cargo: ${run.error.message}`);
  process.exit(1);
}
process.exit(run.status ?? 1);
