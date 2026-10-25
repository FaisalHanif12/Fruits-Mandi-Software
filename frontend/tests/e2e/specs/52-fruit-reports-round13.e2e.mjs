// D5 late track, round 13, the fruit slice (PRD 16.7, 18.2 K and F, 19.2 R37 and R10, FR-12.37, FR-7.46,
// NFR-USE-7, NFR-USE-15, NFR-USE-28): the fruit register (R37), the analytics view K (fruit taken) and
// the fruit column of view F, by keyboard, in Urdu and then in English, on the real app (fruit may or
// may not have been taken by the earlier specs: both are right).
//   1. R37 shows the fruit taken (or says there is none) with its title and "Partner or guests" box,
//      never a raw ID, and does not scroll sideways.
//   2. The "Partner or guests" box takes "guests" with the keys (Enter opens the list, ↓ moves, Enter
//      takes it) and the register asks again, still without a sideways scroll.
//   3. View K opens as the last tab with its title, a chart or the empty message, no comparison box and
//      no sideways scroll; view F still opens.
//   4. The stock (R10) and the day summary (R34) still open with the fruit column and row.
//   5. Alt+L turns R37 and view K to English, left to right, with the same rows.
// The figures are the backend's (E2E-42) and are proved by the Rust tests and the fixtures; this spec
// proves that the real app shows them and opens the screens behind them. It runs on Windows only
// (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  openRoute,
  pageText,
  press,
  startSignedIn,
  switchLanguage,
  waitForHeading,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WAIT_MS = 20_000;

/** True when the page, the main area and every grid do not scroll sideways (NFR-USE-15). */
async function noSidewaysScroll() {
  return browser.execute(() => {
    const page = document.documentElement;
    const main = document.querySelector('main');
    const grids = Array.from(document.querySelectorAll('.ag-body-horizontal-scroll-viewport'));
    return (
      page.scrollWidth <= window.innerWidth &&
      (main === null || main.scrollWidth <= main.clientWidth) &&
      grids.every((grid) => grid.scrollWidth <= grid.clientWidth)
    );
  });
}

/** True when the page shows no raw message ID, markup or error code (NFR-USE-14). */
async function showsNoRawText() {
  const text = await pageText();
  return !/ERR_|<num>|<code>|<name>|report-col-|report-r\d+-|analytics-col-|analytics-view-|fruit-/.test(
    text,
  );
}

/** The number of rows drawn now, without the totals row. */
async function rowCount() {
  return (await $$('.ag-row[row-id]:not(.ag-row-pinned)')).length;
}

/** Waits until the table has a row, or the page says there is nothing. @param {'ur' | 'en'} language */
async function waitForAnswer(language) {
  await browser.waitUntil(
    async () =>
      (await rowCount()) > 0 ||
      (await pageText()).includes(t(language, 'report-empty')) ||
      (await pageText()).includes(t(language, 'analytics-empty')),
    { timeout: WAIT_MS, timeoutMsg: 'The screen never answered.' },
  );
}

/** True when a box with this label is on the page now. @param {string} label */
async function hasBox(label) {
  return (await $$(`//label[normalize-space()="${label}"]`)).length > 0;
}

/**
 * Picks an option of a drop-down with the keys: the box has focus, Enter opens the list, the arrows
 * move from the option now chosen, Enter takes the one under the cursor.
 * @param {import('../support/messages.mjs').Language} language
 * @param {string} fieldId the label of the drop-down
 * @param {number} downs how many options to move down from the one now chosen
 */
async function pickWithKeys(language, fieldId, downs) {
  await focusField(language, fieldId);
  await press(Key.Enter);
  const option = await $('[role="option"]');
  await option.waitForDisplayed({ timeout: WAIT_MS });
  for (let step = 0; step < downs; step += 1) await press(Key.ArrowDown);
  await press(Key.Enter);
}

describe('the fruit register, view K and the fruit column (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. R37 shows the fruit taken or says there is none, with its box, and does not scroll sideways', async () => {
    await openRoute('#/reports/R37');
    await waitForHeading('ur', 'report-r37-title');
    await waitForAnswer('ur');
    expect(await hasBox(t('ur', 'fruit-filter-for'))).toBe(true);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r37-fruit', 'ur');
  });

  it('2. the guests are picked with the keys and the register asks again', async () => {
    // "All" is the first choice; the guests are one down.
    await pickWithKeys('ur', 'fruit-filter-for', 1);
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r37-guests', 'ur');
  });

  it('3. view K is the last tab with its title and no comparison box, and view F still opens', async () => {
    await openRoute('#/analytics/K');
    await waitForHeading('ur', 'analytics-view-k-title');
    await waitForAnswer('ur');
    expect(await hasBox(t('ur', 'analytics-filter-compare'))).toBe(false);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('analytics-k', 'ur');
    await openRoute('#/analytics/F');
    await waitForHeading('ur', 'analytics-view-f-title');
    await waitForAnswer('ur');
    expect(await noSidewaysScroll()).toBe(true);
  });

  it('4. the stock and the day summary open with their fruit column and row', async () => {
    await openRoute('#/reports/R10');
    await waitForHeading('ur', 'report-r10-title');
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await openRoute('#/reports/R34');
    await waitForHeading('ur', 'report-r34-title');
    expect(await noSidewaysScroll()).toBe(true);
  });
});

describe('the fruit register, view K and the fruit column (English)', () => {
  it('5. Alt+L turns R37 and view K to English, left to right, with the same rows', async () => {
    await openRoute('#/reports/R37');
    await waitForHeading('ur', 'report-r37-title');
    await waitForAnswer('ur');
    const rows = await rowCount();
    await switchLanguage('en');
    await waitForHeading('en', 'report-r37-title');
    await waitForAnswer('en');
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    expect(await rowCount()).toBe(rows);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r37-fruit', 'en');
    await openRoute('#/analytics/K');
    await waitForHeading('en', 'analytics-view-k-title');
    await waitForAnswer('en');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('analytics-k', 'en');
    await switchLanguage('ur');
  });
});
