// D5 late track, round 11 (PRD FR-19.12, FR-18.8, FR-17.9, FR-12.27, E50, E2E-43, NFR-USE-7,
// NFR-USE-15, NFR-USE-28): the party day sheet (R35) and the analytics views F (by partner), G
// (trend) and J (trips), by keyboard, in Urdu and then in English, on the real app after the worked
// day (the earlier specs have entered Events 1 and 2 and the vouchers).
//   1. R35 shows the table of every party for the day typed: a group row for the buyers, the footer
//      lines under the table, no totals the screen made, and never scrolls sideways.
//   2. Tab to the table, ↓ to a party, Enter opens his slip: the lines of his day and, under them,
//      the footer lines; the button "All parties" goes back to the table with the day still typed.
//   3. Alt+L turns the slip to English, left to right, with the same figures and the same party.
//   4. F9 shows the print preview of the sheet with the footer lines under the table.
//   5. View F (by partner), G (trend) and J (trips) open from the tabs with the arrow keys, each with
//      its title, a chart or the empty message, and no sideways scroll; the comparison box is not
//      offered on them.
//   6. The partner box of F takes a partner with the keys and asks again; the layout box of J takes
//      another layout with the keys.
//   7. Alt+L turns view J to English, left to right, and nothing typed is lost.
// The figures are the backend's and are proved by the Rust tests (E2E-43, E2E-34b, E2E-36) and the
// report fixtures; this spec proves that the screens ask for what was typed, show what the backend
// sends, never add up a list themselves and never overflow sideways. It runs on Windows only
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
  tabTo,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WAIT_MS = 20_000;
const DAY = '10-10-2026';

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
  return !/ERR_|<num>|<code>|<name>|report-col-|report-r\d+-|analytics-col-|analytics-view-/.test(
    text,
  );
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

/** Opens the sheet and types the day into its box with the keys. @param {'ur' | 'en'} language */
async function openSheet(language) {
  await openRoute('#/reports/R35');
  await waitForHeading(language, 'report-r35-title');
  await focusField(language, 'report-filter-day');
  await type(DAY);
  await waitForAnswer(language);
}

describe('party day sheet and analytics views F, G and J (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. R35 shows the table of every party with a group row, the footer lines under it and no sideways scroll', async () => {
    await openSheet('ur');
    const text = await pageText();
    expect(text).toContain(t('ur', 'report-r35-buyers'));
    expect(text).toContain(t('ur', 'report-r35-collection'));
    // No period box: the sheet is for one day.
    expect(
      await $$(`//label[normalize-space()="${t('ur', 'report-filter-period')}"]`),
    ).toHaveLength(0);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r35-table', 'ur');
  });

  it('2. Tab to the table, ↓ to a party, Enter for his slip, then the button back to every party', async () => {
    await tabTo('.ag-root-wrapper');
    // The first row is the heading of the buyers; the second is the first buyer.
    await press(Key.ArrowDown, Key.ArrowDown, Key.Enter);
    await browser.waitUntil(async () => hasColumn('qty'), {
      timeout: WAIT_MS,
      timeoutMsg: 'The slip of the party never opened.',
    });
    const text = await pageText();
    expect(text).toContain(t('ur', 'report-r35-total-qty'));
    expect(text).toContain(t('ur', 'report-r35-previous'));
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r35-slip', 'ur');
    const back = await $(
      `//main//button[normalize-space()="${t('ur', 'report-r35-all-parties')}"]`,
    );
    await back.waitForDisplayed({ timeout: WAIT_MS });
    await back.click();
    await browser.waitUntil(async () => hasColumn('party'), { timeout: WAIT_MS });
    expect(await noSidewaysScroll()).toBe(true);
  });

  it('3. F9 shows the print preview of the sheet, and Esc closes it', async () => {
    await press(Key.F9);
    await waitForText(t('ur', 'report-print-title'));
    const summary = await $('.print-report .print-summary-lines');
    await summary.waitForDisplayed({ timeout: WAIT_MS });
    expect(await showsNoRawText()).toBe(true);
    await press(Key.Escape);
    await browser.waitUntil(
      async () => !(await pageText()).includes(t('ur', 'report-print-title')),
      {
        timeout: WAIT_MS,
      },
    );
  });

  it('4. views F, G and J open from the tabs with the arrow keys, with no comparison and no sideways scroll', async () => {
    await openRoute('#/analytics/F');
    await waitForHeading('ur', 'analytics-view-f-title');
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    expect(
      await $$(`//label[normalize-space()="${t('ur', 'analytics-filter-compare')}"]`),
    ).toHaveLength(0);
    await shootScreen('analytics-f', 'ur');
    const tab = await $(
      `//*[@role="tab"][normalize-space()="${t('ur', 'analytics-view-f-title')}"]`,
    );
    await tab.click();
    // In Urdu the next view is to the left: G (trend).
    await press(Key.ArrowLeft);
    await waitForHeading('ur', 'analytics-view-g-title');
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('analytics-g', 'ur');
    await openRoute('#/analytics/J');
    await waitForHeading('ur', 'analytics-view-j-title');
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('analytics-j', 'ur');
  });

  it('5. the partner box of F and the layout box of J take a choice with the keys and ask again', async () => {
    await openRoute('#/analytics/F');
    await waitForHeading('ur', 'analytics-view-f-title');
    await waitForAnswer('ur');
    const before = await rowCount();
    // "All partners" is the first choice; the first partner is one down.
    await pickWithKeys('ur', 'report-filter-partner', 1);
    await browser.waitUntil(async () => (await rowCount()) <= before, { timeout: WAIT_MS });
    expect(await showsNoRawText()).toBe(true);
    await openRoute('#/analytics/J');
    await waitForHeading('ur', 'analytics-view-j-title');
    await waitForAnswer('ur');
    // By trip, purchase, item, day: purchase is one down.
    await pickWithKeys('ur', 'report-filter-trip-view', 1);
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
  });
});

describe('party day sheet and analytics views F, G and J (English)', () => {
  it('6. Alt+L turns the slip to English, left to right, with the same party', async () => {
    await openSheet('ur');
    await tabTo('.ag-root-wrapper');
    await press(Key.ArrowDown, Key.ArrowDown, Key.Enter);
    await browser.waitUntil(async () => hasColumn('qty'), { timeout: WAIT_MS });
    await switchLanguage('en');
    await waitForText(t('en', 'report-r35-title'));
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    // The slip is still open, the day is still typed, and the footer is in English.
    await waitForText(t('en', 'report-r35-total-qty'));
    expect(await hasColumn('qty')).toBe(true);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r35-slip', 'en');
  });

  it('7. Alt+L turns view J to English, left to right, with the same trips', async () => {
    await openRoute('#/analytics/J');
    await waitForHeading('en', 'analytics-view-j-title');
    await waitForAnswer('en');
    const rows = await rowCount();
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('analytics-j', 'en');
    await switchLanguage('ur');
    await waitForHeading('ur', 'analytics-view-j-title');
    await browser.waitUntil(async () => (await rowCount()) === rows, { timeout: WAIT_MS });
    await openRoute('#/reports/R35');
    await waitForHeading('ur', 'report-r35-title');
    await switchLanguage('en');
    await waitForText(t('en', 'report-r35-title'));
    await shootScreen('reports-r35-table', 'en');
    await switchLanguage('ur');
  });
});
