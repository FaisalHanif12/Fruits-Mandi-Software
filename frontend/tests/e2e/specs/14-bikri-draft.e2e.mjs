// E2E-17 (D3-A13, D3-A17, PRD FR-12.10, NFR-REL-2): the app is closed while a بکری is being typed,
// and when it is opened again the draft is offered and restored.
//   1. Types a truck number and the start of a lot, then waits for the draft to be kept (5 seconds).
//   2. Closes the app without saving (the session ends) and opens it again.
//   3. After sign-in the app asks "پچھلی بکری مکمل کریں؟". Restore opens the sale book with the
//      truck number and the 12 نگ still in it.
//   4. The app is closed again: the draft is offered again, because it was never saved or thrown
//      away. This time it is discarded, and the next start offers nothing.
// The save removes its draft in one transaction (proved by the Rust tests). This spec runs on
// Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  openRoute,
  startSignedIn,
  type,
  waitForHeading,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

/** The draft is kept every 5 seconds (DRAFT_SAVE_INTERVAL_MS); a little more makes sure. */
const DRAFT_KEPT_MS = 7_000;
const OFFER_WAIT_MS = 15_000;
const NO_OFFER_WAIT_MS = 3_000;

/**
 * Types the text of a picker, waits for its list to answer, and takes the first match.
 * @param {string} text
 */
async function typeAndPick(text) {
  await type(text);
  const first = await $('[role="option"]');
  await first.waitForDisplayed({ timeout: OFFER_WAIT_MS });
  await type(Key.Enter);
}

/** The offer of the unfinished بکری. */
function offer() {
  return $(
    `//*[@role="alertdialog"][.//*[contains(normalize-space(.), "${t('ur', 'bikri-draft-title')}")]]`,
  );
}

/** A button of the offer by its words. @param {string} id */
function offerButton(id) {
  return $(`//*[@role="alertdialog"]//button[contains(normalize-space(.), "${t('ur', id)}")]`);
}

/** Closes the app and opens it again, and signs in (the session ends with the app). */
async function closeAndReopen() {
  await browser.reloadSession();
  await startSignedIn('ur', OWNER);
}

describe('E2E-17: the app is closed while a بکری is typed (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. types a truck and the start of a lot, and the draft is kept', async () => {
    await openRoute('/bikri/book');
    await waitForHeading('ur', 'glossary-bikri-book');
    await focusField('ur', 'bikri-header-truck');
    await type(`KHI-0099${Key.Enter}`);
    await typeAndPick('rahim');
    await type(`${Key.Enter}${Key.Enter}`);
    // The seller (filled), the item and the packing are Enter stops; then the 12 نگ.
    await type(Key.Enter);
    await typeAndPick('banana');
    await typeAndPick('box');
    await type('12');
    await browser.pause(DRAFT_KEPT_MS);
  });

  it('2. after the app is closed and opened, the offer asks to finish the last بکری', async () => {
    await closeAndReopen();
    const box = await offer();
    await box.waitForDisplayed({ timeout: OFFER_WAIT_MS });
    await shootScreen('bikri-draft-offer', 'ur');
  });

  it('3. restore opens the sale book with the truck number and the 12 نگ', async () => {
    const restore = await offerButton('bikri-draft-restore');
    await restore.click();
    await waitForHeading('ur', 'glossary-bikri-book');
    const truck = await focusField('ur', 'bikri-header-truck');
    await browser.waitUntil(async () => (await truck.getValue()) === 'KHI-0099', {
      timeout: OFFER_WAIT_MS,
      timeoutMsg: 'The truck number of the draft did not come back.',
    });
    const qty = await $(
      `//*[@role="group"][@aria-label="${t('ur', 'bikri-lots-label')}"]//div[@role="row"][@row-index="0"]//div[@col-id="qty"]`,
    );
    await browser.waitUntil(async () => (await qty.getText()).trim() === '12', {
      timeout: OFFER_WAIT_MS,
      timeoutMsg: 'The 12 نگ of the draft did not come back.',
    });
  });

  it('4. the draft is offered again after another close; discard throws it away', async () => {
    await closeAndReopen();
    const box = await offer();
    await box.waitForDisplayed({ timeout: OFFER_WAIT_MS });
    const discard = await offerButton('bikri-draft-discard');
    await discard.click();
    await box.waitForDisplayed({ timeout: OFFER_WAIT_MS, reverse: true });
  });

  it('5. after discard the next start offers nothing', async () => {
    await closeAndReopen();
    const box = await offer();
    const shown = await box
      .waitForDisplayed({ timeout: NO_OFFER_WAIT_MS })
      .then(() => true)
      .catch(() => false);
    expect(shown).toBe(false);
  });
});
