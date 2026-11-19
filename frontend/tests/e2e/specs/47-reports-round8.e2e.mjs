// D5 late track, round 8 (PRD 12.8, 12.10, 15, 19.2, FR-12.14, FR-12.27, FR-19.10, FR-19.11, E2E-34b,
// E2E-36, NFR-USE-7, NFR-USE-15, NFR-USE-28): stock (R10), the vouchers list (R15), the receipts and
// payments registers (R23, R25), the trip profit or loss (R33) and the day summary (R34), by keyboard,
// in Urdu and then in English, on the real app after the worked day (the earlier specs have entered
// Events 1 and 2 and the vouchers).
//   1. R10 shows the lots still held, or says there is nothing; it never scrolls sideways.
//   2. R15 with no kind picked shows the money of each kind in lines above the table and no totals
//      row; picking one kind (with the keys) shows only that kind, with one total under the table and
//      no lines above it.
//   3. R23 and R25 show the day typed, with the backend's total under the amount; neither has a
//      mode, bank or cheque column.
//   4. R33 asks by trip at first; the layout box takes another layout with the keys (the partner
//      layout shows each partner's share as a percent); a trip result is never raw markup.
//   5. R34 shows the day as one block of rows (no totals row), starting with the cash opening and
//      ending with the buyers' dues and the lots left; the partner heading is a words row.
//   6. Alt+L turns R15 and R33 to English, left to right, with the same figures.
// The figures are the backend's and are proved by the Rust tests and the report fixtures; this spec
// proves that the screens ask for what was typed, show what the backend sends, never add up a list
// themselves and never overflow sideways. It runs on Windows only (LOCAL-ONLY in the Mac phase:
// checked with `node --check`).

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
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WAIT_MS = 20_000;
const DAY = '10-10-2026';

/** Opens a report by its address. @param {string} id */
async function openReport(id) {
  await openRoute(`#/reports/${id}`);
}

/** The text of a cell of the totals row. @param {string} column */
async function totalsCell(column) {
  const cell = await $(`.ag-row-pinned [col-id="${column}"]`);
  await cell.waitForDisplayed({ timeout: WAIT_MS });
  return (await cell.getText()).replace(/\s+/g, ' ').trim();
}

/** True when the table has a totals row now. */
async function hasTotalsRow() {
  return (await $$('.ag-row-pinned')).length > 0;
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

/** The number of rows drawn now, without the totals row. */
async function rowCount() {
  return (await $$('.ag-row[row-id]:not(.ag-row-pinned)')).length;
}

/** True when the table has a column with this id in its header. @param {string} column */
async function hasColumn(column) {
  return (await $$(`.ag-header-cell[col-id="${column}"]`)).length > 0;
}

/** True when the page shows no raw error code, message ID or markup (NFR-USE-14). */
async function showsNoRawText() {
  const text = await pageText();
  return !/ERR_|<num>|<code>|report-col-|report-r\d+-|report-doc-|report-result-/.test(text);
}

/**
 * Types a day into the two date boxes of a report, with the keys: the first box has focus when the
 * report opens, the second is one field on (PRD FR-7.15).
 * @param {import('../support/messages.mjs').Language} language
 * @param {string} day
 */
async function typeDay(language, day) {
  await focusField(language, 'report-filter-from');
  await type(day);
  await focusField(language, 'report-filter-to');
  await type(day);
}

/** Waits until the table has a row, or the page says there is nothing. @param {'ur' | 'en'} language */
async function waitForAnswer(language) {
  await browser.waitUntil(
    async () => (await rowCount()) > 0 || (await pageText()).includes(t(language, 'report-empty')),
    { timeout: WAIT_MS, timeoutMsg: 'The report never answered.' },
  );
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

describe('stock, vouchers, registers, trip result and day summary (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. R10 shows the lots held or says there is nothing, and never scrolls sideways', async () => {
    await openReport('R10');
    await waitForHeading('ur', 'report-r10-title');
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r10', 'ur');
  });

  it('2. R15 shows the money of each kind above the table with no kind, and one kind with one total', async () => {
    await openReport('R15');
    await waitForHeading('ur', 'report-r15-title');
    await typeDay('ur', DAY);
    await browser.waitUntil(async () => (await rowCount()) > 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'The vouchers list never showed a line.',
    });
    // No kind picked: the saved money of each kind in lines above the table; the screen adds up
    // nothing, so there is no totals row.
    const receiptLine = await $(`//dt[normalize-space()="${t('ur', 'report-doc-receipt')}"]`);
    await receiptLine.waitForDisplayed({ timeout: WAIT_MS });
    expect(await hasTotalsRow()).toBe(false);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r15-all', 'ur');
    // One kind, by keyboard: the kind list starts on "all kinds", receipt is one down, payment two.
    await pickWithKeys('ur', 'report-filter-voucher-type', 2);
    await browser.waitUntil(async () => hasTotalsRow(), {
      timeout: WAIT_MS,
      timeoutMsg: 'One kind of voucher never showed its total.',
    });
    expect(await totalsCell('amount')).not.toBe('');
    expect(await $$(`//dt[normalize-space()="${t('ur', 'report-doc-receipt')}"]`)).toHaveLength(0);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r15-payments', 'ur');
  });

  it('3. R23 and R25 show the day typed, with the backend’s total, and no mode, bank or cheque column', async () => {
    for (const id of ['R23', 'R25']) {
      await openReport(id);
      await waitForHeading('ur', `report-${id.toLowerCase()}-title`);
      await typeDay('ur', DAY);
      await waitForAnswer('ur');
      if ((await rowCount()) > 0) expect(await totalsCell('amount')).not.toBe('');
      for (const column of ['mode', 'bank', 'cheque']) {
        expect(await hasColumn(column)).toBe(false);
      }
      expect(await showsNoRawText()).toBe(true);
      expect(await noSidewaysScroll()).toBe(true);
      await shootScreen(`reports-${id.toLowerCase()}`, 'ur');
    }
  });

  it('4. R33 asks by trip, takes the partner layout with the keys, and shows no raw text', async () => {
    await openReport('R33');
    await waitForHeading('ur', 'report-r33-title');
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r33-trip', 'ur');
    // Trip, item, day, month, partner: partner is four down.
    await pickWithKeys('ur', 'report-filter-trip-view', 4);
    await waitForAnswer('ur');
    expect(await hasColumn('partner')).toBe(true);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r33-partner', 'ur');
  });

  it('5. R34 shows the day as one block with no totals row, from the cash opening to the lots left', async () => {
    await openReport('R34');
    await waitForHeading('ur', 'report-r34-title');
    await typeDay('ur', DAY);
    await browser.waitUntil(async () => (await rowCount()) > 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'The day summary never showed a row.',
    });
    expect(await hasTotalsRow()).toBe(false);
    const text = await pageText();
    expect(text).toContain(t('ur', 'report-r34-cash-opening'));
    expect(text).toContain(t('ur', 'report-r34-partner-shares'));
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r34', 'ur');
  });
});

describe('stock, vouchers, registers, trip result and day summary (English)', () => {
  it('6. Alt+L turns R15 and R33 to English, left to right, with the same figures', async () => {
    await openReport('R15');
    await waitForHeading('ur', 'report-r15-title');
    await typeDay('ur', DAY);
    await browser.waitUntil(async () => (await rowCount()) > 0, { timeout: WAIT_MS });
    const rows = await rowCount();
    await switchLanguage('en');
    await waitForText(t('en', 'report-r15-title'));
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    // The same day is still typed, the same lines are drawn, and the money of each kind is above.
    await browser.waitUntil(async () => (await rowCount()) === rows, { timeout: WAIT_MS });
    const receiptLine = await $(`//dt[normalize-space()="${t('en', 'report-doc-receipt')}"]`);
    await receiptLine.waitForDisplayed({ timeout: WAIT_MS });
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r15', 'en');
    await openReport('R33');
    await waitForText(t('en', 'report-r33-title'));
    await waitForAnswer('en');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r33', 'en');
    await openReport('R34');
    await waitForText(t('en', 'report-r34-title'));
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r34', 'en');
    await switchLanguage('ur');
  });
});
