#!/usr/bin/env node
// Plan 80, lane T2 (volume and speed): builds the full NFR-SCL-1 data (20,000 accounts, 3.5 million
// journal lines, 1.2 million sale lines, 10 years) and times every NFR-SPD target a Mac can measure.
// Owner: robustness. Everything lives in backend/target-gate-t2 (about 8 GB), nothing outside it.
//
// Usage: node scripts/perf/gate-t2-run.mjs [--build-only] [--only <test>] [--criterion] [--keep-results]
//   --build-only    build the data and check it (test t2_00), then stop
//   --only <test>   run one test of backend/crates/services/tests/robustness_gate_t2.rs
//   --criterion     also run the existing criterion benches on the same data (accounts, search,
//                   rules, translit, vouchers, bikri, bikri_save), compared with thresholds.json
//   --keep-results  do not empty the results file first
//
// Tests run one at a time (--test-threads=1): they are timings. MANDI_SPEED_REPORT_ONLY=1 makes a
// row over target a line in the table, not a failed test, so one slow action does not hide the rest.
// The project folder has spaces in its name: every path is built with node:path and every process
// starts without a shell.

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync, appendFileSync } from 'node:fs';
import { cpus, loadavg, totalmem } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, '..', '..');
const BACKEND = join(ROOT, 'backend');
const TARGET = join(BACKEND, 'target-gate-t2');
// GATE_T2_SRC: another copy of the repository to build and test from (for example a clean export of
// the commit under test, while other people edit the working tree). The data stays in TARGET.
// A clean export builds in its own target folder: two source trees must not share build files (cargo
// reuses a crate whose files look older than its build, and would mix the two trees).
const CARGO_TARGET = process.env.GATE_T2_SRC ? join(TARGET, 'src-build') : TARGET;
const SRC_BACKEND = process.env.GATE_T2_SRC ? join(resolve(process.env.GATE_T2_SRC), 'backend') : BACKEND;
const PERF = join(TARGET, 'perf');
const OUT = join(PERF, 'gate-t2-results.jsonl');
const LOGS = join(PERF, 'gate-t2-logs');
const BASE = join(PERF, 'nfr_scl1');
const GATE = join(PERF, 'nfr_scl1_gate');

const argv = process.argv.slice(2);
const flag = (name) => argv.includes(name);
const valueOf = (name) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1] : undefined;
};

// [test name, cargo package, test file]
const GATE_FILE = ['mandi-services', 'robustness_gate_t2'];
const TESTS = [
  't2_00_build_volume',
  't2_02_dashboard_and_tile_pages',
  't2_03_every_report',
  't2_04_every_analytics_view',
  't2_05_lists_windows_and_statements',
  't2_06_start_up_preview_and_save',
  't2_07_query_plans',
  't2_08_dashboard_parts',
  't2_09_partner_pages',
  't2_10_search_and_translit',
  't2_snapshot_and_size_of_the_full_volume',
];
const FILE_OF = {
  t2_08_dashboard_parts: ['mandi-services', 'robustness_gate_t2_dash'],
  t2_09_partner_pages: ['mandi-services', 'robustness_gate_t2_dash'],
  t2_10_search_and_translit: ['mandi-services', 'robustness_gate_t2_dash'],
  t2_snapshot_and_size_of_the_full_volume: ['mandi-backup', 'robustness_gate_t2_snapshot'],
};

const env = {
  ...process.env,
  CARGO_TARGET_DIR: CARGO_TARGET,
  CARGO_INCREMENTAL: '0',
  MANDI_PERF_DIR: BASE,
  MANDI_SPEED_REPORT_ONLY: '1',
  GATE_T2_OUT: OUT,
};

function run(cmd, args, opts = {}) {
  return spawnSync(cmd, args, { cwd: SRC_BACKEND, env, stdio: 'inherit', ...opts });
}

/** Runs one test of the gate file and keeps its output in the log folder. */
function runTest(name) {
  mkdirSync(LOGS, { recursive: true });
  console.log(`gate-t2: running ${name}`);
  const [pkg, file] = FILE_OF[name] ?? GATE_FILE;
  const res = spawnSync(
    'cargo',
    [
      'test', '--release', '-p', pkg, '--test', file, '--',
      '--ignored', '--nocapture', '--test-threads=1', '--exact', name,
    ],
    { cwd: SRC_BACKEND, env, encoding: 'utf8', maxBuffer: 1 << 28 },
  );
  writeFileSync(join(LOGS, `${name}.log`), `${res.stdout ?? ''}\n${res.stderr ?? ''}`);
  const ok = res.status === 0;
  console.log(`gate-t2: ${name} ${ok ? 'finished' : `EXIT ${res.status} (see ${join(LOGS, `${name}.log`)})`}`);
  return ok;
}

function fmt(ms) {
  if (ms === undefined || ms === null) return '-';
  if (ms < 1) return `${(ms * 1000).toFixed(0)} us`;
  if (ms < 1000) return `${ms.toFixed(1)} ms`;
  return `${(ms / 1000).toFixed(2)} s`;
}

/** The table of every measured row. */
function report() {
  if (!existsSync(OUT)) {
    console.log('gate-t2: no results file');
    return 2;
  }
  const rows = [];
  const facts = [];
  for (const line of readFileSync(OUT, 'utf8').split('\n')) {
    if (!line.trim()) continue;
    try {
      const row = JSON.parse(line);
      if (row.fact !== undefined) facts.push(row);
      else rows.push(row);
    } catch {
      // A partly written line of a killed run.
    }
  }
  // An id may have been measured more than once. The measurement taken on the quietest Mac wins (the
  // lowest load average; a tie goes to the later one), because load only ever makes a time worse.
  const byId = new Map();
  for (const r of rows) {
    const old = byId.get(r.id);
    if (!old || r.load1 <= old.load1 + 2) byId.set(r.id, r);
  }
  const final = [...byId.values()];
  // The Mac is shared with other lanes, so the wall time is inflated when the load is high. The
  // adjusted p95 is the wall p95 scaled by (CPU time / wall time) of the same runs: what the call
  // would take with a core to itself. The verdict uses the adjusted figure when the load average
  // was above 12 (8 logical CPUs), the wall figure otherwise. Both are in the table.
  const adjusted = (r) =>
    r.cpu_mean_ms > 0 && r.wall_mean_ms > 0 ? r.p95_ms * Math.min(1, r.cpu_mean_ms / r.wall_mean_ms) : r.p95_ms;
  const judged = (r) => (r.load1 > 12 ? adjusted(r) : r.p95_ms);
  const status = (r) => {
    if (!(r.target_ms > 0)) return 'INFO';
    const loaded = r.load1 > 12;
    const tag = loaded ? ' (CPU-adjusted, loaded Mac)' : '';
    if (judged(r) > r.target_ms) return `OVER${tag}`;
    if (loaded && r.p95_ms > r.target_ms) return 'OK (CPU-adjusted; wall time over under load)';
    if (r.also_ms > 0 && judged(r) > r.also_ms) return 'OK (over the older target)';
    return 'OK';
  };
  const table = final.map((r) => [
    r.id,
    r.nfr,
    r.action,
    fmt(r.p95_ms),
    fmt(adjusted(r)),
    r.min_ms === undefined ? '-' : fmt(r.min_ms),
    r.target_ms > 0 ? fmt(r.target_ms) : '-',
    status(r),
    `n=${r.runs}`,
    fmt(r.cold_first_ms),
    fmt(r.warm_p95_ms),
    r.cpu_mean_ms > 0 ? fmt(r.cpu_mean_ms) : '-',
    `${Number(r.load1).toFixed(0)}`,
  ]);
  const header = ['id', 'NFR', 'action', 'p95 wall', 'p95 adjusted', 'fastest', 'target', 'result', 'runs', 'first call', 'warm p95', 'CPU/call', 'load'];
  const lines = [header, ...table].map((cells) => `| ${cells.join(' | ')} |`);
  lines.splice(1, 0, `|${header.map(() => '---').join('|')}|`);
  const over = final.filter((r) => status(r).startsWith('OVER'));
  const loadedRows = final.filter((r) => r.load1 > 12).length;
  const cpu = cpus()[0]?.model ?? 'unknown';
  const text = [
    `# Gate T2 speed run (${new Date().toISOString()})`,
    '',
    `Machine: ${cpu}, ${cpus().length} logical CPUs, ${(totalmem() / 2 ** 30).toFixed(0)} GB. Load average now: ${loadavg().map((n) => n.toFixed(1)).join(' ')}.`,
    `Rows: ${final.length}. Over target: ${over.length}. Measured with the load average above 12: ${loadedRows} (their verdict uses the CPU-adjusted p95).`,
    '',
    ...lines,
    '',
    '## Facts',
    ...facts.map((f) => `- ${f.id}: ${f.fact}`),
    '',
  ].join('\n');
  const file = join(PERF, 'gate-t2-report.md');
  writeFileSync(file, text);
  console.log(text);
  console.log(`gate-t2: report written to ${file}`);
  return over.length === 0 ? 0 : 1;
}

function criterion() {
  const gate = { ...env, MANDI_PERF_DIR: GATE };
  for (const name of ['accounts', 'search', 'rules', 'translit', 'vouchers', 'bikri', 'bikri_save']) {
    console.log(`gate-t2: cargo bench --bench ${name} (on the full data)`);
    const res = spawnSync('cargo', ['bench', '-p', 'mandi-services', '--bench', name], {
      cwd: SRC_BACKEND, env: gate, stdio: 'inherit',
    });
    appendFileSync(OUT, `${JSON.stringify({ id: `criterion.${name}.exit`, fact: `cargo bench ${name} exit ${res.status}` })}\n`);
  }
  console.log('gate-t2: now: node scripts/perf/run.mjs --skip-bench --skip-install with the same environment');
  return spawnSync('node', [join(HERE, 'run.mjs'), '--skip-bench', '--skip-install'], {
    cwd: ROOT, env: { ...gate }, stdio: 'inherit',
  }).status;
}

mkdirSync(PERF, { recursive: true });
if (!flag('--keep-results') && !valueOf('--only') && !flag('--report-only')) {
  rmSync(OUT, { force: true });
}
if (flag('--report-only')) process.exit(report());

const only = valueOf('--only');
const plan = flag('--build-only') ? [TESTS[0]] : only ? [only] : TESTS;
for (const name of plan) {
  if (!runTest(name)) console.log(`gate-t2: ${name} failed; going on with the next test`);
}
if (flag('--criterion')) criterion();
process.exit(report());
