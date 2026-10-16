// NFR-SPD-1 (D1-A15): the app opens to the sign-in screen in under 2 seconds. The app starts with
// the session, so each start is a new session: the time is taken from just before the session is
// asked for to the moment the sign-in heading is on screen. 20 starts, p95 against the runner
// threshold (scripts/perf/README.md Sections 4 and 5). It includes the WebDriver round trips,
// so it is a little more than the user sees: if it passes, the real figure passes.

import { browser, expect } from '@wdio/globals';
import { waitForHeading } from '../support/page.mjs';
import { RUNS, recordTiming, sinceSessionStart } from '../support/timing.mjs';

const SLOW_TEST_MS = 600_000;

describe('cold start (NFR-SPD-1)', () => {
  it('opens to the sign-in screen in time, 20 starts, p95', async function () {
    this.timeout(SLOW_TEST_MS);
    /** @type {number[]} */
    const samples = [];
    for (let run = 0; run < RUNS; run += 1) {
      process.env.E2E_SESSION_STARTED = String(Date.now());
      await browser.reloadSession();
      await waitForHeading('ur', 'auth-login-title');
      samples.push(sinceSessionStart());
    }
    const result = recordTiming('NFR-SPD-1', 'Start of the app to the sign-in screen', samples);
    console.log(
      `NFR-SPD-1: p95 ${result.p95_ms.toFixed(0)} ms, threshold ${result.threshold_ms.toFixed(0)} ms`,
    );
    expect(result.pass).toBe(true);
  });
});
