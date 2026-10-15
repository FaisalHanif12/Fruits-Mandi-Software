// D1 story, step 5 (FR-8.4): after the set minutes with no key or click the app locks, and what
// was typed and not saved is still there when the Owner unlocks with the password. The time is
// set to one minute in Settings and put back to fifteen at the end, so the specs after this one
// are not locked while they work. About two minutes in all.

import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  openEndOfMenu,
  pageText,
  startSignedIn,
  type,
  waitForHeading,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const SLOW_TEST_MS = 240_000;
const IDLE_MINUTES = '1';
const NORMAL_MINUTES = '15';
const LOCK_WAIT_MS = 90_000;

/** Sets the auto lock minutes in Settings, on the "Auto lock" tab, and saves with F10. @param {string} minutes */
async function setIdleMinutes(minutes) {
  await openEndOfMenu('last');
  await waitForHeading('ur', 'settings-title');
  await $(
    `//*[@role="tab"][contains(normalize-space(.), "${t('ur', 'settings-tab-lock')}")]`,
  ).click();
  const field = await focusField('ur', 'settings-lock-minutes');
  await field.setValue(minutes);
  await type(Key.F10);
  await browser.waitUntil(async () => (await pageText()).includes(t('ur', 'settings-saved')));
}

describe('idle lock (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  after(async () => {
    // Put the time back, whatever happened, so nothing later is locked while it works.
    await browser.keys(Key.Escape);
    await setIdleMinutes(NORMAL_MINUTES).catch(() => undefined);
  });

  it('locks after the set minutes, and the unsaved text is still there after unlocking', async function () {
    this.timeout(SLOW_TEST_MS);
    await setIdleMinutes(IDLE_MINUTES);
    // Unsaved text in the Firm tab.
    await $(
      `//*[@role="tab"][contains(normalize-space(.), "${t('ur', 'settings-tab-firm')}")]`,
    ).click();
    const address = await focusField('ur', 'settings-firm-address');
    await address.setValue('Typed and not saved');
    const dialog = await $('[role="dialog"]');
    await dialog.waitForDisplayed({ timeout: LOCK_WAIT_MS });
    expect(await pageText()).toContain(t('ur', 'auth-lock-title'));
    await shootScreen('lock', 'ur');
    // Esc does not close it; only the password does.
    await browser.keys(Key.Escape);
    await expect(dialog).toBeDisplayed();
    await dialog.$('input[type="password"]').setValue(OWNER.password);
    await type(Key.Enter);
    await dialog.waitForDisplayed({ reverse: true });
    expect(await (await focusField('ur', 'settings-firm-address')).getValue()).toBe(
      'Typed and not saved',
    );
  });
});
