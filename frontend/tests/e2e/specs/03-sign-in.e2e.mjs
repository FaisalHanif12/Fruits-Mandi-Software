// D1 story, step 2: the sign-in screen by keyboard only. Typing and arrow moves are timed here
// (NFR-SPD-4 key to character, NFR-SPD-5 arrow move), a wrong password shows under the field with
// the tries left (FR-8.2), and a right one opens the dashboard, timed (NFR-SPD-2). The
// sign-in/sign-out loop runs 20 times for the p95.

import { browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  pageText,
  signOut,
  type,
  waitForHeading,
  waitForShell,
} from '../support/page.mjs';
import { RUNS, armTimer, readTimer, recordTiming } from '../support/timing.mjs';

/** @param {string} name @param {number[]} samples @param {string} action */
function check(name, action, samples) {
  const result = recordTiming(name, action, samples);
  console.log(
    `${name}: p95 ${result.p95_ms.toFixed(1)} ms, threshold ${result.threshold_ms.toFixed(0)} ms`,
  );
  expect(result.pass).toBe(true);
}

describe('the sign-in screen (Urdu)', () => {
  before(async () => {
    await waitForHeading('ur', 'auth-login-title');
  });

  it('key press to character on screen is under 50 ms (NFR-SPD-4)', async () => {
    await focusField('ur', 'auth-login-username');
    /** @type {number[]} */
    const samples = [];
    for (let run = 1; run <= RUNS; run += 1) {
      await armTimer({ kind: 'value', selector: 'form input[lang="en"]', text: 'k'.repeat(run) });
      await type('k');
      samples.push(await readTimer());
    }
    check('NFR-SPD-4', 'Key press to the character on screen, in a field', samples);
  });

  it('an arrow move between fields is under 30 ms (NFR-SPD-5)', async () => {
    /** @type {number[]} */
    const samples = [];
    for (let run = 0; run < RUNS; run += 1) {
      const down = run % 2 === 0;
      await armTimer({
        kind: 'focus',
        selector: down ? 'form input[type="password"]' : 'form input[lang="en"]',
      });
      await type(down ? Key.ArrowDown : Key.ArrowUp);
      samples.push(await readTimer());
    }
    check('NFR-SPD-5', 'Arrow key move between fields', samples);
  });

  it('a wrong password shows under the password field with the tries left', async () => {
    const username = await focusField('ur', 'auth-login-username');
    await username.setValue(OWNER.username);
    await type(`${Key.Enter}wrong-password${Key.Enter}`);
    const message = t('ur', 'err-wrong-password', { tries_left: 4 });
    await browser.waitUntil(async () => (await pageText()).includes(message), {
      timeoutMsg: `The page never showed: ${message}`,
    });
    expect(await pageText()).not.toContain('ERR_');
  });

  it('the right password opens the dashboard with the Owner by name', async () => {
    const password = await focusField('ur', 'auth-login-password');
    await password.setValue(OWNER.password);
    await type(Key.Enter);
    await waitForShell('ur');
    const greeting = t('ur', 'dashboard-greeting', { name: OWNER.displayName });
    await browser.waitUntil(async () => (await pageText()).includes(greeting), {
      timeoutMsg: `The dashboard never said: ${greeting}`,
    });
  });
});

describe('sign out and in, timed (NFR-SPD-2)', () => {
  it('from the Enter on the password to the dashboard is under 1 second, 20 times, p95', async () => {
    /** @type {number[]} */
    const samples = [];
    for (let run = 0; run < RUNS; run += 1) {
      await signOut('ur');
      await focusField('ur', 'auth-login-username');
      await type(`${OWNER.username}${Key.Enter}${OWNER.password}`);
      await armTimer({
        kind: 'text',
        selector: 'h1',
        text: t('ur', 'dashboard-greeting', { name: OWNER.displayName }),
      });
      await type(Key.Enter);
      samples.push(await readTimer());
      await waitForShell('ur');
    }
    check('NFR-SPD-2', 'Sign in to the dashboard fully drawn', samples);
  });
});
