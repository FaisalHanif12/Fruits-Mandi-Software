// Scripted check of the run options --bank and --threads (plan 55 gate 2 and the speed of the run).
// No model, no key, no cargo, no network. It starts run.mjs as a child process (no shell) and looks at
// the exit code and what was written.
//
//   node scripts/ai-eval/check-modes.mjs
//
// What it proves:
//  1. --bank FILE --check passes for a small good bank, and writes nothing into backend/target/ai-eval/.
//  2. A bank with a repeated id, an unknown @key, an unknown outcome, a missing column, or a follow-up
//     to an id that is not there is refused with exit code 2 and a message that names the row.
//  3. A bank that is not the team bank skips the team rules (size, the frozen 110, the facets).
//  4. A bank file that is not there is refused with exit code 2.
//  5. The results of another bank are scored into backend/target/ai-eval-<name>/, and the team's
//     folder is left alone.
//  6. --threads is passed to the live test as MANDI_EVAL_THREADS, and the live test clamps it to 1..8.
//
// Everything is made in a temporary folder. The hidden bank is never read.

import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const RUN = join(HERE, 'run.mjs');
const ROOT = resolve(HERE, '..', '..');
const TEAM_OUT = join(ROOT, 'backend', 'target', 'ai-eval');
const TEAM_BANK = join(HERE, 'questions.tsv');

const problems = [];
const expect = (ok, what) => {
  if (!ok) problems.push(what);
};

const run = (...args) => spawnSync('node', [RUN, ...args], { encoding: 'utf8' });
const mtime = (path) => (existsSync(path) ? statSync(path).mtimeMs : null);

// ---- a small good bank, made from the first rows of the team bank (a copy; the team bank is not changed)
const lines = readFileSync(TEAM_BANK, 'utf8').split('\n');
const head = lines[0].split('\t');
const col = (name) => head.indexOf(name);
const rows = lines
  .slice(1)
  .filter(Boolean)
  .map((l) => l.split('\t'));
// A mix: a figure question, a help question, and a follow-up with its first question.
const pick = (pred) => rows.find(pred);
const first = pick(
  (r) => r[col('cat')] === 'A' && r[col('after')] === '' && r[col('world')] === 'day',
);
const help = pick((r) => r[col('cat')] === 'D' && r[col('after')] === '');
const follow = pick(
  (r) => r[col('after')] !== '' && rows.some((p) => p[col('id')] === r[col('after')]),
);
const parent = rows.find((p) => p[col('id')] === follow[col('after')]);
const good = [first, help, parent, follow].filter(
  (r, i, all) => all.findIndex((x) => x[col('id')] === r[col('id')]) === i,
);
const dir = mkdtempSync(join(tmpdir(), 'ai-eval-modes-'));
const write = (name, rowsOf, header = head) => {
  const path = join(dir, name);
  writeFileSync(path, [header.join('\t'), ...rowsOf.map((r) => r.join('\t'))].join('\n') + '\n');
  return path;
};

try {
  const teamBefore = mtime(join(TEAM_OUT, 'summary.json'));
  const teamResultsBefore = mtime(join(TEAM_OUT, 'results.json'));

  // 1 + 3. A good small bank passes with the per-row rules only.
  const goodPath = write('smallbank.tsv', good);
  let r = run('--bank', goodPath, '--check');
  expect(
    r.status === 0,
    `a small good bank must pass --check; exit ${r.status}: ${r.stderr || r.stdout}`,
  );
  expect(/another bank/.test(r.stdout), 'the output must say it is another bank');
  expect(/per-row rules/.test(r.stdout), 'the output must say only the per-row rules apply');
  expect(
    mtime(join(TEAM_OUT, 'summary.json')) === teamBefore,
    'the team summary was touched by a --bank check',
  );
  expect(
    mtime(join(TEAM_OUT, 'results.json')) === teamResultsBefore,
    'the team results were touched by a --bank check',
  );

  // 2. Each broken bank is refused with exit 2.
  const id = col('id');
  const dup = [...good, [...good[0]]];
  r = run('--bank', write('dup.tsv', dup), '--check');
  expect(
    r.status === 2 && /dup|twice|unique|repeat/i.test(r.stderr + r.stdout),
    `a repeated id must be refused with exit 2; exit ${r.status}: ${r.stderr}`,
  );

  const badKey = good.map((x) => [...x]);
  badKey[0][col('figures')] = 'total=@no.such.key.at.all';
  r = run('--bank', write('badkey.tsv', badKey), '--check');
  expect(
    r.status === 2 && /no\.such\.key\.at\.all/.test(r.stderr + r.stdout),
    `an unknown @key must be refused and named; exit ${r.status}: ${r.stderr}`,
  );

  const badOutcome = good.map((x) => [...x]);
  badOutcome[1][col('outcome')] = 'wonderful';
  r = run('--bank', write('badoutcome.tsv', badOutcome), '--check');
  expect(r.status === 2, `an unknown outcome must be refused with exit 2; exit ${r.status}`);

  const noCol = head.filter((c) => c !== 'figures');
  const noColRows = good.map((x) => x.filter((_, i) => i !== col('figures')));
  r = run('--bank', write('nocol.tsv', noColRows, noCol), '--check');
  expect(
    r.status === 2 && /figures/.test(r.stderr),
    `a missing column must be refused and named; exit ${r.status}: ${r.stderr}`,
  );

  const orphan = good.map((x) => [...x]);
  orphan[orphan.findIndex((x) => x[id] === follow[id])][col('after')] = 'QZZZ9';
  r = run('--bank', write('orphan.tsv', orphan), '--check');
  expect(
    r.status === 2,
    `a follow-up to an id that is not there must be refused with exit 2; exit ${r.status}`,
  );

  // 4. A file that is not there.
  r = run('--bank', join(dir, 'nothing-here.tsv'), '--check');
  expect(
    r.status === 2 && /no bank file/.test(r.stderr),
    `a missing bank file must be refused; exit ${r.status}: ${r.stderr}`,
  );

  // 5. Scoring a results file of another bank: the files go to ai-eval-<name>/, not to the team folder.
  //    The results are made up here (an empty run of the bank): the score is not a result of the app.
  const name = `check-modes-${process.pid}`;
  const scored = write(`${name}.tsv`, good);
  const emptyResults = join(dir, 'results.json');
  writeFileSync(
    emptyResults,
    JSON.stringify({ meta: { model: 'none', questions: 0, threads: 3 }, oracle: {}, results: [] }),
  );
  r = run('--bank', scored, '--results', emptyResults, '--review-out', join(dir, 'review.tsv'));
  const bankOut = join(ROOT, 'backend', 'target', `ai-eval-${name}`);
  expect(
    [0, 1].includes(r.status),
    `scoring another bank must give exit 0 or 1; exit ${r.status}: ${r.stderr}`,
  );
  expect(
    existsSync(join(bankOut, 'summary.json')),
    `the summary of another bank must be in ${bankOut}`,
  );
  expect(/threads 3/.test(r.stdout), 'the report must show the threads of the run (3)');
  expect(/bank /.test(r.stdout), 'the report must show the name of the bank');
  expect(
    mtime(join(TEAM_OUT, 'summary.json')) === teamBefore,
    'the team summary was touched when another bank was scored',
  );
  rmSync(bankOut, { recursive: true, force: true });

  // 6. --threads reaches the live test, which clamps it. Read the source, not a run: no key, no cargo here.
  const live = readFileSync(
    join(ROOT, 'backend', 'crates', 'assistant', 'tests', 'robustness_live.rs'),
    'utf8',
  );
  const runner = readFileSync(RUN, 'utf8');
  expect(
    /MANDI_EVAL_THREADS/.test(runner) && /MANDI_EVAL_THREADS/.test(live),
    '--threads must be passed as MANDI_EVAL_THREADS and read by the live test',
  );
  expect(/\.clamp\(1, 8\)/.test(live), 'the live test must clamp the threads to 1..8');
  expect(/THREADS_DEFAULT = '3'/.test(runner), 'the default must be 3 threads');
  expect(
    /MANDI_EVAL_QUESTIONS: QUESTIONS/.test(runner),
    '--bank must reach the live test as MANDI_EVAL_QUESTIONS',
  );
} finally {
  rmSync(dir, { recursive: true, force: true });
}

if (problems.length) {
  console.error(`check-modes FAILED:\n  ${problems.join('\n  ')}`);
  process.exit(2);
}
console.log(
  'check-modes ok: --bank (check, refusals, own output folder, team folder untouched) and --threads (default 3, clamp 1 to 8, passed to the live test)',
);
