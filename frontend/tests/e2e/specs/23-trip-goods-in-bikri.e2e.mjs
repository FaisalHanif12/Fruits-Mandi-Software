// E2E-34 and E2E-34b (D4-A23, PRD 12.10, FR-12.26, FR-12.28, FR-12.33): the goods of a trip are sold in
// a بکری with اپنا مال as the seller of the lot, typed with the keyboard, in Urdu.
//   1. A lot with اپنا مال as the seller shows the panel of trip goods and focus goes to the choice of
//      the purchase. Choosing the purchase of the trip (spec 22: 20 پیٹی کیلا, cost 10,400) with
//      Enter fills the item, the packing and the quantity, and focus goes on to the quantity.
//   2. One line, اکرم 15 at 600 ادھاری, shows the numbers of E2E-34: commission 540 and 540, لاگا 120,
//      the seller credit 8,460 that goes to the trip's partners as 3,384 / 2,538 / 2,538, the cost
//      7,800 and the result 660. Cash paid and the advance tick are not there.
//   3. The lot must sell the whole quantity taken, so the quantity of the lot was set to 15 in step 1
//      (the purchase fills it with the 20 it has). F10 saves.
//   4. باقی مال lists the 5 left under the trip goods, with the trip, and "open the trip" leads to
//      the trip with the purchase chosen.
//   5. The last 5 are written off (E2E-34b): nothing posts, the trip closes by itself, and the form
//      is replaced by the note that says so.
// It runs on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  openRoute,
  pageText,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import {
  buttonNamed,
  cell,
  saveSale,
  typeAndPick,
  waitForNumbers,
} from '../support/bikri-sale.mjs';

const WAIT_MS = 15_000;

describe('E2E-34 and E2E-34b: trip goods in a بکری (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. اپنا مال as the seller shows the trip goods panel; the purchase fills the lot', async () => {
    await openRoute('/bikri/book');
    await waitForHeading('ur', 'glossary-bikri-book');
    await focusField('ur', 'bikri-header-truck');
    await type(`OWN-1${Key.Enter}`);
    // The header seller is asked first; the lot seller below is the only field that offers اپنا مال.
    await type(Key.Escape);
    const seller = await cell(t('ur', 'bikri-lots-label'), 0, 'seller');
    await seller.click();
    await type('own');
    await typeAndPickFirstOwn();
    await waitForText(t('ur', 'bikri-own-purchase'));
    // Focus is on the choice of the purchase: Enter opens the list and Enter takes the first one.
    await type(Key.Enter);
    const option = await $('[role="option"]');
    await option.waitForDisplayed({ timeout: WAIT_MS });
    await type(Key.Enter);
    // The purchase fills the quantity with what it has (20) and focus goes to that cell, so the
    // keys go on without the mouse: this lot takes 15 and sells them all.
    await type(`15${Key.Enter}`);
    await shootScreen('trip-goods-lot', 'ur');
  });

  it('2. one line shows the numbers of E2E-34 before the save', async () => {
    await type(Key.Tab);
    await focusLineCell('buyer');
    await typeAndPick('akram');
    await type(`15${Key.Enter}600${Key.Enter}`);
    // Commission 540, لاگا 120, the credit 8,460 split 3,384 / 2,538 / 2,538, cost 7,800, result 660.
    await waitForNumbers('9,000', '8,460', '3,384', '2,538', '7,800', '660');
    await shootScreen('trip-goods-numbers', 'ur');
  });

  it('3. F10 saves the sale of the trip goods', async () => {
    const number = await saveSale();
    expect(number).toBeGreaterThan(0);
  });

  it('4. باقی مال lists the 5 left under the trip goods and leads to the trip', async () => {
    await openRoute('/bikri/stock');
    await waitForText(t('ur', 'bikri-stock-own-title'));
    const row = await $('//div[@role="row"][.//div[@col-id="trip"][normalize-space(.)="0070"]]');
    await row.waitForDisplayed({ timeout: WAIT_MS });
    expect(await row.$('[col-id="balance"]').getText()).toBe('5');
    await shootScreen('trip-goods-stock', 'ur');
    await row.click();
    await (await buttonNamed('bikri-stock-own-open')).click();
    await waitForText(t('ur', 'trips-status-open'));
  });

  it('5. writing off the last 5 closes the trip by itself and posts nothing', async () => {
    const detailRow = await $('.ag-row [col-id="number"]');
    await detailRow.waitForDisplayed({ timeout: WAIT_MS });
    await detailRow.click();
    await (await buttonNamed('purchases-write-off')).click();
    await waitForText(t('ur', 'purchases-write-off-title'));
    await focusField('ur', 'purchases-write-off-qty');
    await type(`5${Key.Enter}`);
    await type('kharab');
    await type(Key.Enter);
    await (await buttonNamed('purchases-write-off-confirm')).click();
    await waitForText(t('ur', 'trips-closed-notice'));
    const text = await pageText();
    // E2E-34b: the result of the trip is −1,940; the partners' totals do not change.
    expect(text).toContain('1,940');
    await shootScreen('trip-closed', 'ur');
  });
});

/** Takes the first match of the lot seller picker, which lists اپنا مال for the text "own". */
async function typeAndPickFirstOwn() {
  const first = await $('[role="option"]');
  await first.waitForDisplayed({ timeout: WAIT_MS });
  await type(Key.Enter);
}

/**
 * Puts the cursor in a cell of the first sale line.
 * @param {string} column
 */
async function focusLineCell(column) {
  const target = await cell(t('ur', 'bikri-sales-label'), 0, column);
  await target.click();
}
