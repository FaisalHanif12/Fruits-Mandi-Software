// D1 story, step 4 (FR-8.2, NFR-SEC-2, plan 04 Section 3): five wrong passwords lock the sign-in
// for one minute. The screen counts the wait down by itself and keeps the button off until it is
// over; then the right password signs in and the count starts again. The wait is real time, so
// this spec takes about a minute and a half.

import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  button,
  focusField,
  pageText,
  type,
  waitForHeading,
  waitForShell,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WRONG_TRIES = 5;
const LOCK_SECONDS = 60;
const SLOW_TEST_MS = 180_000;
const SECONDS_MARK = 1_234_567;

describe('lockout (Urdu)', () => {
  before(async () => {
    await waitForHeading('ur', 'auth-login-title');
  });

  it('shows the tries left after each wrong password, then locks on the fifth', async function () {
    this.timeout(SLOW_TEST_MS);
    await focusField('ur', 'auth-login-username');
    await type(`${OWNER.username}${Key.Enter}`);
    for (let attempt = 1; attempt <= WRONG_TRIES; attempt += 1) {
      const password = await focusField('ur', 'auth-login-password');
      await password.setValue(`wrong-${String(attempt)}`);
      await type(Key.Enter);
      const expected =
        attempt < WRONG_TRIES
          ? t('ur', 'err-wrong-password', { tries_left: WRONG_TRIES - attempt })
          : t('ur', 'err-locked-wait', { seconds: SECONDS_MARK }).split(String(SECONDS_MARK))[0];
      await browser.waitUntil(async () => (await pageText()).includes(expected ?? ''), {
        timeoutMsg: `Try ${String(attempt)}: the page never showed "${expected ?? ''}"`,
      });
    }
    await shootScreen('login-locked', 'ur');
  });

  it('keeps the Sign in button off while the wait runs', async () => {
    const submit = await button('ur', 'auth-login-submit');
    await expect(submit).toBeDisabled();
  });

  it('opens again when the wait is over, and the right password signs in', async function () {
    this.timeout(SLOW_TEST_MS);
    const prefix =
      t('ur', 'err-locked-wait', { seconds: SECONDS_MARK }).split(String(SECONDS_MARK))[0] ?? '';
    // The message goes away when the countdown reaches zero.
    await browser.waitUntil(async () => !(await pageText()).includes(prefix), {
      timeout: (LOCK_SECONDS + 30) * 1000,
      interval: 1000,
      timeoutMsg: 'The lock never ended.',
    });
    await $('form').waitForDisplayed();
    const password = await focusField('ur', 'auth-login-password');
    await password.setValue(OWNER.password);
    await type(Key.Enter);
    await waitForShell('ur');
    // A right password starts the count again: nothing is left of the wrong tries.
    expect(await pageText()).not.toContain(t('ur', 'err-wrong-password', { tries_left: 4 }));
  });
});
