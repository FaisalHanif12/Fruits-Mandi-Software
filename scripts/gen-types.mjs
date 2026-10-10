// pnpm gen:types: regenerate frontend/src/bindings from the Rust types (ts-rs), then format them.
// Tech Stack 2 (ts-rs) and Code Standards F2. Never edit the bindings by hand.
//
// Same effect as:
//   cd backend && TS_RS_EXPORT_DIR="$PWD/../frontend/src/bindings" cargo test -p mandi-app export_bindings
//   prettier --write frontend/src/bindings
// It is written in Node so it also runs on Windows, where pnpm uses cmd.exe for scripts
// and the `VAR=value command` form does not work.
//
// Old .ts files are removed first, so a type deleted in Rust also disappears here and the
// CI "no diff" check sees it. Files that use the money type `Paisa` get its import
// (scripts/bindings-paisa.mjs). The output is stable: running it twice gives no git diff.

import { spawnSync } from 'node:child_process';
import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import * as prettier from 'prettier';
import { addPaisaImport } from './bindings-paisa.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const backendDir = path.join(root, 'backend');
const bindingsDir = path.join(root, 'frontend', 'src', 'bindings');

mkdirSync(bindingsDir, { recursive: true });
for (const name of readdirSync(bindingsDir)) {
  if (name.endsWith('.ts')) {
    rmSync(path.join(bindingsDir, name));
  }
}

const cargo = spawnSync('cargo', ['test', '-p', 'mandi-app', 'export_bindings'], {
  cwd: backendDir,
  env: { ...process.env, TS_RS_EXPORT_DIR: bindingsDir },
  stdio: 'inherit',
});
if (cargo.error) {
  console.error(`gen:types: could not run cargo: ${cargo.error.message}`);
  process.exit(1);
}
if (cargo.status !== 0) {
  console.error('gen:types: the ts-rs export test failed.');
  process.exit(cargo.status ?? 1);
}

const files = readdirSync(bindingsDir)
  .filter((name) => name.endsWith('.ts'))
  .sort();
if (files.length === 0) {
  console.error(
    'gen:types: cargo ran but wrote no bindings. Check TS_RS_EXPORT_DIR and the export test.',
  );
  process.exit(1);
}

for (const name of files) {
  const file = path.join(bindingsDir, name);
  const options = (await prettier.resolveConfig(file)) ?? {};
  // Money fields are typed Paisa; add its import where a file uses it (plan 02).
  const source = addPaisaImport(readFileSync(file, 'utf8'));
  const formatted = await prettier.format(source, { ...options, filepath: file });
  writeFileSync(file, formatted);
}

console.log(
  `gen:types: wrote ${files.length} file(s) to frontend/src/bindings: ${files.join(', ')}`,
);
