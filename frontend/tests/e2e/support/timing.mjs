// Speed measurements for the Windows suite (README of scripts/perf Sections 4 and 5). A time is
// read inside the page with performance.now(), from the key event to the change on screen, so the
// WebDriver round trip is not part of it. Each action runs at least 20 times and the p95 is
// compared with the NFR target times the runner factor (1.0 today, scripts/perf/thresholds.json).
// Results go to output/timings.json; every spec adds its own keys.

import { browser } from '@wdio/globals';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { OUTPUT_DIR, ROOT_DIR } from '../paths.mjs';
import { readJson } from './json.mjs';

export const TIMINGS_FILE = path.join(OUTPUT_DIR, 'timings.json');
const THRESHOLDS_FILE = path.join(ROOT_DIR, 'scripts', 'perf', 'thresholds.json');

/** How many times each action is measured (scripts/perf/README.md Section 4). */
export const RUNS = 20;
const P95 = 0.95;
const POLL_MS = 20;
const WAIT_MS = 10_000;

/** The NFR targets the suite measures, in milliseconds. */
/** @type {Record<string, number>} */
export const TARGET_MS = {
  'NFR-SPD-1': 2000,
  'NFR-SPD-2': 1000,
  'NFR-SPD-3': 300,
  'NFR-SPD-4': 50,
  'NFR-SPD-5': 30,
  'NFR-SPD-19': 500,
};

/** @returns {number} */
function runnerFactor() {
  const thresholds = readJson(THRESHOLDS_FILE, /** @type {{ runner_factor?: number }} */ ({}));
  return thresholds.runner_factor ?? 1;
}

/**
 * The 95th percentile, nearest rank.
 * @param {number[]} samples
 * @returns {number}
 */
export function p95(samples) {
  const sorted = [...samples].sort((a, b) => a - b);
  const rank = Math.max(Math.ceil(P95 * sorted.length) - 1, 0);
  return sorted[rank] ?? 0;
}

/**
 * Adds one measured action to output/timings.json.
 * @param {string} nfr the NFR ID, for example "NFR-SPD-19"
 * @param {string} action what was measured, in words
 * @param {number[]} samples milliseconds, one per run
 * @returns {{ p95_ms: number, threshold_ms: number, pass: boolean }}
 */
export function recordTiming(nfr, action, samples) {
  const target = TARGET_MS[nfr];
  if (target === undefined) throw new Error(`No target for ${nfr}`);
  const threshold = target * runnerFactor();
  const result = {
    action,
    runs: samples.length,
    samples,
    p95_ms: p95(samples),
    target_ms: target,
    threshold_ms: threshold,
  };
  mkdirSync(OUTPUT_DIR, { recursive: true });
  const all = readJson(TIMINGS_FILE, /** @type {Record<string, unknown>} */ ({}));
  const pass = result.p95_ms <= threshold;
  all[nfr] = { ...result, pass };
  writeFileSync(TIMINGS_FILE, JSON.stringify(all, null, 2));
  return { p95_ms: result.p95_ms, threshold_ms: threshold, pass };
}

/**
 * What ends a timer: a change on screen the page can check without running any code from text.
 * @typedef {(
 *   | { kind: 'attribute'; name: string; value: string }
 *   | { kind: 'text'; selector: string; text: string }
 *   | { kind: 'focus'; selector: string }
 *   | { kind: 'value'; selector: string; text: string }
 * )} StopWhen
 */

/**
 * Starts a timer in the page. It begins at the next key event and ends when `stop` is true on
 * screen. Press the key after calling this, then call readTimer().
 * @param {StopWhen} stop
 */
export async function armTimer(stop) {
  await browser.execute((when) => {
    /** @type {{ t0: number | null, ms: number | null }} */
    const timer = { t0: null, ms: null };
    Object.assign(window, { __e2eTimer: timer });
    const clean = (/** @type {string | null | undefined} */ text) =>
      (text ?? '').replace(/[⁨⁩]/g, '');
    const done = () => {
      switch (when.kind) {
        case 'attribute':
          return document.documentElement.getAttribute(when.name) === when.value;
        case 'text':
          return Array.from(document.querySelectorAll(when.selector)).some((element) =>
            clean(element.textContent).includes(when.text),
          );
        case 'focus':
          return document.activeElement?.matches(when.selector) === true;
        case 'value':
          return Array.from(document.querySelectorAll(when.selector)).some(
            (element) => element instanceof HTMLInputElement && element.value.includes(when.text),
          );
      }
    };
    const check = () => {
      if (timer.t0 === null || timer.ms !== null || !done()) return;
      timer.ms = performance.now() - timer.t0;
      observer.disconnect();
      window.removeEventListener('keydown', onKey, true);
      document.removeEventListener('focusin', check, true);
    };
    const observer = new MutationObserver(check);
    const onKey = () => {
      if (timer.t0 !== null) return;
      timer.t0 = performance.now();
      check();
    };
    window.addEventListener('keydown', onKey, true);
    document.addEventListener('focusin', check, true);
    observer.observe(document.documentElement, {
      subtree: true,
      childList: true,
      attributes: true,
      characterData: true,
    });
  }, stop);
}

/**
 * The time the armed timer measured, in milliseconds.
 * @returns {Promise<number>}
 */
export async function readTimer() {
  const read = () =>
    browser.execute(() => {
      const timer = /** @type {Window & { __e2eTimer?: { ms: number | null } }} */ (window)
        .__e2eTimer;
      return timer?.ms ?? null;
    });
  await browser.waitUntil(async () => (await read()) !== null, {
    timeout: WAIT_MS,
    interval: POLL_MS,
    timeoutMsg: 'The change the timer waits for never came.',
  });
  const ms = await read();
  if (ms === null) throw new Error('The timer did not stop.');
  return ms;
}

/**
 * How long the app took to start: from the moment the session was asked for (the app starts with
 * it) to now. wdio.conf.mjs sets the start time before the session.
 * @returns {number}
 */
export function sinceSessionStart() {
  const started = Number(process.env.E2E_SESSION_STARTED);
  return Number.isFinite(started) && started > 0 ? Date.now() - started : Number.NaN;
}
