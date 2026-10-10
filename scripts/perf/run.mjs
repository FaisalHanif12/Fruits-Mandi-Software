#!/usr/bin/env node
// Speed harness: the entry of `pnpm perf` (plan 01 T5, Code Standards B25).
//
// 1. Runs the criterion benchmarks of the Rust workspace (`cargo bench`).
// 2. Reads each result from backend/target/criterion/ and compares it with its
//    runner threshold from scripts/perf/thresholds.json
//    (threshold = NFR target x runner factor, scripts/perf/README.md).
// 3. On Windows only: if the NSIS installer exists, installs it silently, times
//    it against NFR-INS-1 (under 2 minutes), checks the app exe exists, and
//    uninstalls it.
// 4. Prints one table, writes backend/target/perf/report.json, and exits 1 if
//    any value is over its threshold or a registered benchmark did not run.
//
// Usage: node scripts/perf/run.mjs [--skip-bench] [--skip-install] [--require-install]
//   --skip-bench       do not run cargo; read the results of the last run
//   --skip-install     do not run the installer timing (Windows)
//   --require-install  on Windows, fail if the installer is missing (use in CI)
//
// The project folder has spaces in its name. Every path here is built with
// node:path and every process is started without a shell, so nothing is split.

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { cpus, totalmem } from 'node:os';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, '..', '..');
const BACKEND = join(ROOT, 'backend');
const TARGET = process.env.CARGO_TARGET_DIR
  ? resolve(BACKEND, process.env.CARGO_TARGET_DIR)
  : join(BACKEND, 'target');
const CRITERION_DIR = join(TARGET, 'criterion');
const REPORT_DIR = join(TARGET, 'perf');
const NSIS_DIR = join(TARGET, 'release', 'bundle', 'nsis');
const TAURI_CONF = join(BACKEND, 'app', 'tauri.conf.json');

const NS_PER_MS = 1_000_000;
const PERCENTILE = 0.95;
const INSTALL_TIMEOUT_MS = 10 * 60 * 1000;
const UNINSTALL_WAIT_MS = 2 * 60 * 1000;
const POLL_MS = 500;

const args = new Set(process.argv.slice(2));
const IS_WINDOWS = process.platform === 'win32';

/** Reads and parses a JSON file. Throws with the file name on failure. */
function readJson(file) {
  try {
    return JSON.parse(readFileSync(file, 'utf8'));
  } catch (err) {
    throw new Error(`cannot read ${file}: ${err.message}`);
  }
}

/** Nearest-rank percentile of a list of numbers. */
function percentile(values, p) {
  const sorted = [...values].sort((a, b) => a - b);
  const rank = Math.max(1, Math.ceil(p * sorted.length));
  return sorted[rank - 1];
}

/** Formats a time in ms with a sensible unit. */
function fmtMs(ms) {
  if (ms === null || ms === undefined || Number.isNaN(ms)) return '-';
  if (ms < 0.001) return `${(ms * 1_000_000).toFixed(1)} ns`;
  if (ms < 1) return `${(ms * 1000).toFixed(1)} us`;
  if (ms < 1000) return `${ms.toFixed(2)} ms`;
  return `${(ms / 1000).toFixed(2)} s`;
}

/** Runs `cargo bench` on every bench target of the workspace. */
function runBenches() {
  // `--bench '*'` builds and runs only [[bench]] targets. Plain `cargo bench`
  // would also build the Tauri app and every lib in the release profile
  // (LTO, one codegen unit), which takes many minutes and measures nothing.
  // No shell is used, so '*' reaches cargo as a pattern, not a file glob.
  const cmd = ['bench', '--workspace', '--bench', '*'];
  console.log(`perf: running cargo ${cmd.join(' ')} in ${BACKEND}`);
  const res = spawnSync('cargo', cmd, { cwd: BACKEND, stdio: 'inherit' });
  if (res.error) throw new Error(`cannot start cargo: ${res.error.message}`);
  if (res.status !== 0) throw new Error(`cargo bench failed with exit code ${res.status}`);
}

/** Finds every criterion result directory (one per benchmark) under dir. */
function findResults(dir, since) {
  const found = [];
  if (!existsSync(dir)) return found;
  const walk = (d) => {
    for (const entry of readdirSync(d, { withFileTypes: true })) {
      const full = join(d, entry.name);
      if (!entry.isDirectory()) continue;
      if (entry.name === 'new') {
        const bench = join(full, 'benchmark.json');
        const sample = join(full, 'sample.json');
        const estimates = join(full, 'estimates.json');
        if (existsSync(bench) && existsSync(sample) && existsSync(estimates)) {
          // Only results written by this run count, so a stale result of a
          // deleted benchmark can never pass for a fresh one.
          if (since === null || statSync(sample).mtimeMs >= since) {
            found.push({ bench, sample, estimates });
          }
        }
      } else if (entry.name !== 'report' && entry.name !== 'base') {
        walk(full);
      }
    }
  };
  walk(dir);
  return found;
}

/** Turns one criterion result into { id, p95Ms, meanMs, samples }. */
function readResult(files) {
  const bench = readJson(files.bench);
  const sample = readJson(files.sample);
  const estimates = readJson(files.estimates);
  // Each sample is a batch of iterations; its time per iteration is times/iters.
  // p95 is taken over those per-sample times (README Section 4).
  const perIter = sample.iters.map((n, i) => sample.times[i] / n);
  return {
    id: bench.full_id,
    p95Ms: percentile(perIter, PERCENTILE) / NS_PER_MS,
    meanMs: estimates.mean.point_estimate / NS_PER_MS,
    samples: perIter.length,
  };
}

/** Compares measured benchmarks with thresholds.json. Returns table rows. */
function checkBenches(config, results) {
  const rows = [];
  const byId = new Map(results.map((r) => [r.id, r]));
  const factor = config.runner_factor;
  for (const entry of config.benches) {
    const r = byId.get(entry.id);
    byId.delete(entry.id);
    const measured = r ? (entry.stat === 'mean' ? r.meanMs : r.p95Ms) : null;
    if (entry.kind === 'calibration') {
      // Not an NFR target. Checks the runner factor: the runner must not be
      // slower than the reference CPU on this fixed workload (README Section 3).
      const limit = entry.expected_reference_ms;
      let result = 'MISSING';
      if (r) result = measured <= limit ? 'OK' : 'CHECK FACTOR';
      rows.push({
        action: entry.action,
        nfr: 'factor check',
        stat: entry.stat,
        measured,
        mean: r?.meanMs ?? null,
        threshold: limit,
        target: null,
        result,
        fails: result === 'MISSING',
      });
      continue;
    }
    const threshold = entry.target_ms * factor;
    let result = 'MISSING';
    if (r) result = measured <= threshold ? 'OK' : 'OVER';
    rows.push({
      action: entry.action,
      nfr: entry.nfr,
      stat: entry.stat,
      measured,
      mean: r?.meanMs ?? null,
      threshold,
      target: entry.target_ms,
      result,
      fails: result !== 'OK',
    });
  }
  // A benchmark with no entry in thresholds.json is a gap: it measures
  // something nobody compares. It fails until it is registered.
  for (const r of byId.values()) {
    rows.push({
      action: r.id,
      nfr: '-',
      stat: 'p95',
      measured: r.p95Ms,
      mean: r.meanMs,
      threshold: null,
      target: null,
      result: 'NO THRESHOLD',
      fails: true,
    });
  }
  return rows;
}

/** Lists the .exe files directly inside dir, if it exists. */
function exesIn(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).filter(
    (f) => f.toLowerCase().endsWith('.exe') && f.toLowerCase() !== 'uninstall.exe',
  );
}

/** Waits until check() is true or the time runs out. Returns true if it became true. */
function waitFor(check, timeoutMs) {
  const end = Date.now() + timeoutMs;
  while (Date.now() < end) {
    if (check()) return true;
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, POLL_MS);
  }
  return check();
}

/** Windows only: silent install, timing, exe check and silent uninstall (NFR-INS-1). */
function timeInstall(config) {
  const row = {
    action: 'Install (NSIS, silent)',
    nfr: config.install.nfr,
    stat: 'one run',
    measured: null,
    mean: null,
    threshold: config.install.target_ms * config.runner_factor,
    target: config.install.target_ms,
    result: 'SKIPPED',
    fails: false,
    note: '',
  };
  if (!IS_WINDOWS) {
    row.note = 'install timing runs on Windows only';
    return row;
  }
  if (args.has('--skip-install')) {
    row.note = '--skip-install given';
    return row;
  }
  const setups = existsSync(NSIS_DIR)
    ? readdirSync(NSIS_DIR).filter((f) => f.endsWith('-setup.exe'))
    : [];
  if (setups.length === 0) {
    row.note = `no *-setup.exe in ${NSIS_DIR}; run pnpm build first`;
    if (args.has('--require-install')) {
      row.result = 'MISSING';
      row.fails = true;
    }
    return row;
  }
  if (setups.length > 1) {
    row.result = 'ERROR';
    row.fails = true;
    row.note = `more than one installer in ${NSIS_DIR}: ${setups.join(', ')}`;
    return row;
  }
  const setup = join(NSIS_DIR, setups[0]);
  const conf = readJson(TAURI_CONF);
  const product = conf.productName;
  const installMode = conf.bundle?.windows?.nsis?.installMode ?? 'currentUser';
  const base =
    installMode === 'perMachine'
      ? (process.env.ProgramW6432 ?? process.env.ProgramFiles)
      : process.env.LOCALAPPDATA;
  if (!base) {
    row.result = 'ERROR';
    row.fails = true;
    row.note = 'cannot find the install base folder (LOCALAPPDATA or ProgramFiles)';
    return row;
  }
  const installDir = join(base, product);
  if (exesIn(installDir).length > 0) {
    row.result = 'ERROR';
    row.fails = true;
    row.note = `the app is already installed in ${installDir}; uninstall it first`;
    return row;
  }

  console.log(`perf: installing ${setup} silently`);
  const start = process.hrtime.bigint();
  const res = spawnSync(setup, ['/S'], { stdio: 'inherit', timeout: INSTALL_TIMEOUT_MS });
  const ms = Number(process.hrtime.bigint() - start) / NS_PER_MS;
  row.measured = ms;
  if (res.error || res.status !== 0) {
    row.result = 'ERROR';
    row.fails = true;
    row.note = `installer failed: ${res.error?.message ?? `exit code ${res.status}`}`;
    return row;
  }
  const exes = exesIn(installDir);
  if (exes.length === 0) {
    row.result = 'ERROR';
    row.fails = true;
    row.note = `installer exit 0 but no app exe in ${installDir}`;
    return row;
  }
  row.result = ms <= row.threshold ? 'OK' : 'OVER';
  row.fails = row.result !== 'OK';
  row.note = `installed ${join(installDir, exes[0])}`;

  // Close the app in case the installer started it, then uninstall.
  for (const exe of exes) {
    spawnSync('taskkill', ['/F', '/IM', exe], { stdio: 'ignore' });
  }
  const uninstaller = join(installDir, 'uninstall.exe');
  if (!existsSync(uninstaller)) {
    row.result = 'ERROR';
    row.fails = true;
    row.note += `; no uninstall.exe in ${installDir}`;
    return row;
  }
  // The NSIS uninstaller copies itself to a temp folder and returns at once,
  // so wait until the app exe is gone.
  spawnSync(uninstaller, ['/S'], { stdio: 'inherit' });
  const gone = waitFor(() => exesIn(installDir).length === 0, UNINSTALL_WAIT_MS);
  if (!gone) {
    row.result = 'ERROR';
    row.fails = true;
    row.note += `; uninstall did not remove ${installDir} within ${UNINSTALL_WAIT_MS / 1000} s`;
    return row;
  }
  row.note += '; uninstalled';
  return row;
}

/** Prints the result table. */
function printTable(rows, factor) {
  const header = [
    'Action',
    'NFR',
    'Measured',
    'Stat',
    'Mean',
    'Runner threshold',
    'NFR target',
    'Result',
  ];
  const body = rows.map((r) => [
    r.action,
    r.nfr,
    fmtMs(r.measured),
    r.stat,
    fmtMs(r.mean),
    fmtMs(r.threshold),
    fmtMs(r.target),
    r.result,
  ]);
  const widths = header.map((h, i) => Math.max(h.length, ...body.map((b) => String(b[i]).length)));
  const line = (cells) => cells.map((c, i) => String(c).padEnd(widths[i])).join(' | ');
  console.log('');
  console.log(`Speed results (runner factor ${factor.toFixed(2)}, scripts/perf/README.md)`);
  console.log(line(header));
  console.log(widths.map((w) => '-'.repeat(w)).join('-|-'));
  for (const b of body) console.log(line(b));
  for (const r of rows) if (r.note) console.log(`note: ${r.action}: ${r.note}`);
  for (const r of rows) {
    if (r.result === 'CHECK FACTOR') {
      console.log(
        `warning: ${r.action} is slower than the reference CPU estimate. ` +
          'The runner may be slower than the reference machine; measure the runner factor again (README Section 3).',
      );
    }
  }
}

function main() {
  const config = readJson(join(HERE, 'thresholds.json'));
  if (!(typeof config.runner_factor === 'number' && config.runner_factor >= 1)) {
    throw new Error('thresholds.json: runner_factor must be a number of at least 1.0');
  }
  for (const e of config.benches) {
    const okStat = e.stat === 'p95' || e.stat === 'mean';
    const okKind =
      (e.kind === 'nfr' && typeof e.target_ms === 'number' && e.target_ms > 0 && e.nfr) ||
      (e.kind === 'calibration' && typeof e.expected_reference_ms === 'number');
    if (!e.id || !e.action || !okStat || !okKind) {
      throw new Error(`thresholds.json: bad entry ${JSON.stringify(e)}`);
    }
  }
  let since = null;
  if (!args.has('--skip-bench')) {
    // Allow 1 s of clock difference between Node and the file system.
    since = Date.now() - 1000;
    runBenches();
  }
  const results = findResults(CRITERION_DIR, since).map(readResult);
  const rows = checkBenches(config, results);
  rows.push(timeInstall(config));
  printTable(rows, config.runner_factor);
  // The CPU is printed so the runner factor can be checked on every run
  // (README Section 2: the runner CPU must be one of the models listed there).
  const cpu = cpus()[0]?.model?.trim() ?? 'unknown';
  const memGb = (totalmem() / 1024 ** 3).toFixed(1);
  console.log(`perf: machine ${cpu}, ${cpus().length} logical CPUs, ${memGb} GB memory`);

  mkdirSync(REPORT_DIR, { recursive: true });
  const report = {
    at: new Date().toISOString(),
    platform: process.platform,
    arch: process.arch,
    cpu,
    logical_cpus: cpus().length,
    memory_gb: Number(memGb),
    runner_factor: config.runner_factor,
    rows,
  };
  writeFileSync(join(REPORT_DIR, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
  console.log(`perf: report written to ${join(REPORT_DIR, 'report.json')}`);

  const failed = rows.filter((r) => r.fails);
  if (failed.length > 0) {
    console.error(
      `perf: FAIL, ${failed.length} item(s): ${failed.map((r) => `${r.action} ${r.result}`).join('; ')}`,
    );
    process.exit(1);
  }
  console.log('perf: PASS');
}

try {
  main();
} catch (err) {
  console.error(`perf: ERROR ${err.message}`);
  process.exit(2);
}
