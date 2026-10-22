// D5 early track, plan 18 (PRD Section 20, FR-20.8, FR-20.9, NFR-BAK-11, NFR-BAK-10, NFR-USE-7,
// NFR-USE-28): the backup screen, by keyboard, in Urdu and then in English.
//   1. The main bar opens بیک اپ. The key made in the wizard (spec 01) is confirmed, so there is no
//      key card. The status bar says the cloud copy is not set up (the cloud is off on a new
//      computer), with its icon and words.
//   2. "Back up now" with Enter makes the copy on this computer. The run is told in words, a toast
//      says it is done, and the list shows the new copy as good, newest first.
//   3. Enter on a copy opens the restore question: its date, its books next to the books now, what
//      would be lost. Esc closes it and nothing is restored.
//   4. The test restore of the newest copy runs and every check passes.
//   5. The settings: the closing time is changed and saved with F10.
//   6. Alt+L turns the screen to English, left to right, with the list still there.
//   7. A restore of the copy through the confirm box signs the Owner out; he signs in again and the
//      books are the same.
// The numbers themselves are the backend's and are proved by `round_trip.rs` (E2E-18); this spec
// proves the screen asks for what was pressed and shows what the backend sends. It runs on Windows
// only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  button,
  focusField,
  pageText,
  press,
  signIn,
  startSignedIn,
  switchLanguage,
  type,
  waitForHeading,
  waitForShell,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WAIT_MS = 20_000;
const COPY_WAIT_MS = 60_000;

/** A button of the main bar by its words (the name is its aria-label). @param {string} id */
function navItem(id) {
  return $(`//nav//button[@aria-label="${t('ur', id)}"]`);
}

describe('backup (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the main bar opens the backup screen; the key is confirmed, so no key card shows', async () => {
    await (await navItem('layout-nav-backup')).click();
    await waitForHeading('ur', 'backup-title');
    await waitForText(t('ur', 'backup-state-title'));
    expect(await pageText()).not.toContain(t('ur', 'backup-key-missing'));
    expect(await pageText()).not.toContain(t('ur', 'backup-key-unconfirmed'));
    // The cloud copy is switched off on a new computer, and the status bar says so.
    const bar = await $('footer');
    await browser.waitUntil(
      async () => (await bar.getText()).includes(t('ur', 'layout-status-backup-none')),
      { timeout: WAIT_MS },
    );
    await shootScreen('backup', 'ur');
  });

  it('2. "back up now" with Enter makes a copy; the list shows it as good', async () => {
    const now = await button('ur', 'backup-now');
    await browser.execute((element) => {
      element.focus();
    }, now);
    await type(Key.Enter);
    await waitForText(t('ur', 'backup-run-ok'));
    await waitForText(t('ur', 'backup-now-done'));
    await browser.waitUntil(
      async () => (await pageText()).includes(t('ur', 'backup-status-good')),
      { timeout: COPY_WAIT_MS },
    );
    await shootScreen('backup-after-run', 'ur');
  });

  it('3. Enter on a copy asks about the restore, shows its books and what is lost; Esc restores nothing', async () => {
    const row = await $('.ag-row [col-id="created"]');
    await row.waitForDisplayed({ timeout: WAIT_MS });
    await row.click();
    await type(Key.Enter);
    await $('[role="dialog"]').waitForDisplayed({ timeout: WAIT_MS });
    await waitForText(t('ur', 'backup-totals-in-backup'));
    await waitForText(t('ur', 'backup-restore-safety'));
    await shootScreen('backup-restore-preview', 'ur');
    await press(Key.Escape);
    await $('[role="dialog"]').waitForDisplayed({ reverse: true, timeout: WAIT_MS });
    // The screen is still here and the Owner is still signed in.
    await waitForHeading('ur', 'backup-title');
  });

  it('4. the test restore of the newest copy runs and every check passes', async () => {
    const test = await $(
      `//button[contains(normalize-space(.), "${t('ur', 'backup-test-now')}")]`,
    );
    await test.click();
    await waitForText(t('ur', 'backup-test-ok'));
    const passed = await $$(
      `//li[contains(normalize-space(.), "${t('ur', 'backup-check-passed')}")]`,
    );
    expect(passed).toHaveLength(5);
    await shootScreen('backup-test-restore', 'ur');
    await press(Key.Escape);
    await $('[role="dialog"]').waitForDisplayed({ reverse: true, timeout: WAIT_MS });
  });

  it('5. the closing time is changed and saved with F10', async () => {
    const time = await focusField('ur', 'backup-set-closing-time');
    await time.setValue('22:00');
    await press(Key.F10);
    await waitForText(t('ur', 'backup-settings-saved'));
    await shootScreen('backup-settings', 'ur');
  });
});

describe('backup (English)', () => {
  it('6. Alt+L turns the screen to English, left to right, with the list still there', async () => {
    await switchLanguage('en');
    await waitForHeading('en', 'backup-title');
    await expect($('html')).toHaveAttribute('dir', 'ltr');
    await waitForText(t('en', 'backup-status-good'));
    // The time typed in the settings is still there after the switch.
    const time = await focusField('en', 'backup-set-closing-time');
    expect(await time.getValue()).toBe('22:00');
    await shootScreen('backup', 'en');
  });

  it('7. restoring a copy signs the Owner out; signing in again shows the same books', async () => {
    const row = await $('.ag-row [col-id="created"]');
    await row.click();
    await type(Key.Enter);
    await $('[role="dialog"]').waitForDisplayed({ timeout: WAIT_MS });
    await waitForText(t('en', 'backup-totals-in-backup'));
    const restore = await $(
      `//*[@role="dialog"]//button[normalize-space(.) = "${t('en', 'backup-restore-button')}"]`,
    );
    await browser.waitUntil(async () => restore.isEnabled(), { timeout: WAIT_MS });
    await restore.click();
    // The question starts on Back, so a stray Enter restores nothing.
    const question = await $('[role="alertdialog"]');
    await question.waitForDisplayed({ timeout: WAIT_MS });
    await shootScreen('backup-restore-confirm', 'en');
    const confirm = await $(
      `//*[@role="alertdialog"]//button[normalize-space(.) = "${t('en', 'backup-restore-button')}"]`,
    );
    await confirm.click();
    await waitForHeading('en', 'auth-login-title');
    await signIn('en', OWNER);
    await waitForShell('en');
    await (await $(`//nav//button[@aria-label="${t('en', 'layout-nav-backup')}"]`)).click();
    await waitForHeading('en', 'backup-title');
    await waitForText(t('en', 'backup-status-good'));
  });
});
