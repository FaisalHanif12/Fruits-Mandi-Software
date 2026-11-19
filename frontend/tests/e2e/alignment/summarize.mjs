// Plan 39: turns the result files of run.mjs (alignment-1366.json, alignment-1024.json) into a table
// for FINDINGS.md: one line per screen with the rules that fail and where, both languages and both
// sizes folded together.
//
//   node tests/e2e/alignment/summarize.mjs before          prints Markdown
//   node tests/e2e/alignment/summarize.mjs after --short   one line per screen, no details
//
// Folder names can contain spaces, so every path is built with path.join.

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT = path.resolve(HERE, '..', 'output', 'screenshots', 'plan39');
const phase = process.argv[2] ?? 'after';
const short = process.argv.includes('--short');

const runs = [];
for (const size of ['1366', '1024']) {
  const file = path.join(OUTPUT, phase, `alignment-${size}.json`);
  if (existsSync(file)) runs.push(...JSON.parse(readFileSync(file, 'utf8')));
}

/** Screen id -> { rule -> Map(where -> Set(what)) }. */
const byScreen = new Map();
const errors = [];
for (const run of runs) {
  if (run.error) errors.push(`${run.id} ${run.lang} ${run.width}: ${run.error}`);
  const screen = byScreen.get(run.id) ?? { rules: new Map(), runs: 0, tables: 0 };
  screen.runs += 1;
  screen.tables = Math.max(screen.tables, run.tables?.length ?? 0);
  for (const v of run.violations ?? []) {
    const rule = screen.rules.get(v.rule) ?? new Map();
    const where = v.where.replace(/⁨|⁩/g, '');
    const entry = rule.get(where) ?? new Set();
    entry.add(
      `${v.what}${v.text ? ` [${String(v.text).slice(0, 24)}]` : ''} (${run.lang} ${run.width})`,
    );
    rule.set(where, entry);
    screen.rules.set(v.rule, rule);
  }
  byScreen.set(run.id, screen);
}

const failing = [...byScreen.entries()].filter(([, s]) => s.rules.size > 0);
console.log(
  `${byScreen.size} screens and prints, ${failing.length} with a finding, ${runs.length} runs.\n`,
);
console.log('| Screen | Rules | Where |');
console.log('|---|---|---|');
for (const [id, screen] of [...byScreen.entries()].sort(([a], [b]) => a.localeCompare(b))) {
  if (screen.rules.size === 0) continue;
  const rules = [...screen.rules.keys()].sort().join(', ');
  const where = [...screen.rules.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([rule, places]) => {
      const names = [...places.keys()].map((w) => w.replace(/^(grid|table|card|rows)#\d+ /, ''));
      const unique = [...new Set(names)];
      const first = unique.slice(0, short ? 2 : 4).join('; ');
      return `${rule} (${places.size}): ${first}${unique.length > (short ? 2 : 4) ? '; ...' : ''}`;
    })
    .join('<br>');
  console.log(`| ${id} | ${rules} | ${where.replace(/\|/g, '/')} |`);
}
if (errors.length > 0) {
  console.log('\nScreens that did not load:');
  for (const line of errors) console.log(`- ${line}`);
}
