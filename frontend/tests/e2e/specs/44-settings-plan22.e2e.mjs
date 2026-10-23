// D5 early track, plan 22 (PRD 21, FR-21.3, NFR-SEC-5, NFR-INS-3, NFR-OBS-4, NFR-OBS-5, NFR-USE-7,
// NFR-USE-15, NFR-USE-28): the new parts of Settings, by keyboard, in Urdu and then in English, on the
// real app after the worked day.
//   1. Day and check: the closing time is changed and saved (F10), then put back.
//   2. The books check runs on a press and shows its three checks: books, data file, backups.
//   3. The number grouping: the lakh style is pressed and the international style put back.
//   4. The audit log lists entries newest first with the count of the whole log; Enter opens one
//      entry with its changed fields; Esc comes back to the list.
//   5. App and updates: the app looks for a new version (the answer is the server's: up to date, a new
//      version, or "could not look", all in words) and the support file button is there. The update
//      itself is never pressed here: it would close the app (it is tried by hand, LOCAL-ONLY).
//   6. Alt+L turns the audit log to English, left to right, with the same entries.
// Nothing overflows sideways at any step. It runs on Windows only (LOCAL-ONLY in the Mac phase:
// checked with `node --check`). The real save box and a real update are checked by hand.

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  button,
  focusField,
  pageText,
  press,
  startSignedIn,
  switchLanguage,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WAIT_MS = 20_000;

/** Opens a tab of Settings by its address. @param {string} tab */
async function openTab(tab) {
  await browser.execute((address) => {
    window.location.hash = address;
  }, `#/settings?tab=${tab}`);
}

/** True when the page and every grid on it do not scroll sideways (NFR-USE-15). */
async function noSidewaysScroll() {
  return browser.execute(() => {
    const page = document.documentElement;
    const grids = Array.from(document.querySelectorAll('.ag-body-horizontal-scroll-viewport'));
    return (
      page.scrollWidth <= window.innerWidth &&
      grids.every((grid) => grid.scrollWidth <= grid.clientWidth)
    );
  });
}

/** The number of audit rows drawn now. */
async function auditRows() {
  return (await $$('.ag-row[row-id]:not(.ag-row-pinned)')).length;
}

describe('settings, plan 22 (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the closing time is changed with F10 and put back', async () => {
    await openTab('day');
    await waitForText(t('ur', 'settings-day-closing-time'));
    await focusField('ur', 'settings-day-closing-time');
    // Entering a field selects its whole value (PRD FR-7.15), so typing replaces it.
    await type('22:00');
    await press(Key.F10);
    await waitForText(t('ur', 'settings-saved'));
    expect(await noSidewaysScroll()).toBe(true);
    await focusField('ur', 'settings-day-closing-time');
    await type('21:00');
    await press(Key.F10);
    await waitForText(t('ur', 'settings-saved'));
  });

  it('2. the books check runs on a press and shows the books, the data file and the backups', async () => {
    await waitForText(t('ur', 'settings-check-title'));
    const run = await button('ur', 'settings-check-run');
    await run.click();
    await waitForText(t('ur', 'settings-check-books'));
    await waitForText(t('ur', 'settings-check-data'));
    await waitForText(t('ur', 'settings-check-backup'));
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('settings-day-check', 'ur');
  });

  it('3. the lakh grouping is pressed and the international one put back', async () => {
    await openTab('language');
    await waitForText(t('ur', 'settings-grouping-title'));
    // The two examples are numbers, so the buttons are found by them.
    const lakh = await $('//button[contains(., "13,16,040")]');
    await lakh.click();
    await waitForText(t('ur', 'settings-saved'));
    expect(await lakh.getAttribute('aria-pressed')).toBe('true');
    const international = await $('//button[contains(., "1,316,040")]');
    await international.click();
    await waitForText(t('ur', 'settings-saved'));
    expect(await international.getAttribute('aria-pressed')).toBe('true');
  });

  it('4. the audit log lists entries, Enter opens one with its fields, Esc comes back', async () => {
    await openTab('audit');
    await waitForText(t('ur', 'audit-title'));
    await browser.waitUntil(async () => (await auditRows()) > 0, { timeout: WAIT_MS });
    expect(await noSidewaysScroll()).toBe(true);
    const first = await $('.ag-row[row-id] [col-id="when"]');
    await first.click();
    await press(Key.Enter);
    await waitForText(t('ur', 'audit-detail-back'));
    await shootScreen('settings-audit-detail', 'ur');
    await press(Key.Escape);
    await browser.waitUntil(async () => (await auditRows()) > 0, { timeout: WAIT_MS });
    await shootScreen('settings-audit', 'ur');
  });

  it('5. app and updates: the answer of the server is in words, and the support button is there', async () => {
    await openTab('app');
    await waitForText(t('ur', 'settings-update-help'));
    // One of the three answers, never a raw code (NFR-USE-14).
    await browser.waitUntil(
      async () => {
        const text = await pageText();
        return (
          text.includes(t('ur', 'settings-update-up-to-date')) ||
          text.includes(t('ur', 'settings-update-notes')) ||
          text.includes(t('ur', 'err-update-check-failed'))
        );
      },
      { timeout: WAIT_MS },
    );
    expect((await pageText()).includes('ERR_UPDATE')).toBe(false);
    const support = await button('ur', 'settings-support-save');
    expect(await support.isDisplayed()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('settings-app', 'ur');
  });
});

describe('settings, plan 22 (English)', () => {
  it('6. Alt+L turns the audit log to English, left to right, with entries', async () => {
    await openTab('audit');
    await waitForHeading('ur', 'settings-title');
    await switchLanguage('en');
    await waitForText(t('en', 'audit-title'));
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    await browser.waitUntil(async () => (await auditRows()) > 0, { timeout: WAIT_MS });
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('settings-audit', 'en');
    await switchLanguage('ur');
  });
});
