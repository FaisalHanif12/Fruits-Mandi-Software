// D5 late track, round 12 (PRD 19.2 R1 and R5, FR-12.12, FR-12.13, FR-7.4, NFR-USE-7, NFR-USE-15,
// NFR-USE-28): the sale book report (R1, one بکری) and the credit buyers' details (R5), by keyboard, in
// Urdu and then in English, on the real app after the worked day (the earlier specs have entered
// Events 1 and 2, so بکری 1 exists).
//   1. R5 for the day typed shows one row per credit buyer with the numbered column, the goods written
//      "(item qty . rate)" and the footer, no list the screen added up, and never scrolls sideways.
//   2. R1 does not run without a بکری: it asks for one with its own words and draws no rows.
//   3. The بکری box takes the number typed and Enter; the lots of that بکری show with their
//      columns and a totals row, and never scroll sideways.
//   4. F9 on R1 opens the print box of that بکری (the bill), and Esc closes it.
//   5. Alt+L turns R1 to English, left to right, with the same بکری still picked and the same rows.
//   6. R5 is in English, left to right, with the same buyers.
//   7. R5 "Print all slips" opens one sheet for each credit buyer (as many as the table has rows), with
//      no sideways scroll, and Esc goes back to the table.
// The figures are the backend's and are proved by the Rust tests (E2E-1, E2E-36) and the report
// fixtures; this spec proves that the screens ask for what was typed, show what the backend sends,
// never add up a list themselves and never overflow sideways. It runs on Windows only (LOCAL-ONLY in
// the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  button,
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

/** True when the page shows no raw error code, message ID or markup (NFR-USE-14). */
async function showsNoRawText() {
  const text = await pageText();
  return !/ERR_|<num>|<code>|<name>|report-col-|report-r\d+-|report-pick-|report-detail-/.test(
    text,
  );
}

/** Waits until the table has a row, or the page says there is nothing. @param {'ur' | 'en'} language */
async function waitForAnswer(language) {
  await browser.waitUntil(
    async () => (await rowCount()) > 0 || (await pageText()).includes(t(language, 'report-empty')),
    { timeout: WAIT_MS, timeoutMsg: 'The screen never answered.' },
  );
}

/** True when a box with this label is on the page now. @param {string} label */
async function hasBox(label) {
  return (await $$(`//label[normalize-space()="${label}"]`)).length > 0;
}

/** Types the بکری number into its box and takes the first match with Enter. @param {'ur' | 'en'} language */
async function pickBikri(language) {
  const box = await focusField(language, 'report-filter-bikri');
  await box.setValue('1');
  const first = await $('[role="option"]');
  await first.waitForDisplayed({ timeout: WAIT_MS });
  await press(Key.Enter);
  await browser.waitUntil(async () => (await box.getValue()).includes('1'), {
    timeout: WAIT_MS,
    timeoutMsg: 'The box never showed the بکری picked.',
  });
}

describe('credit buyers details and the sale book report (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. R5 shows one row per credit buyer, the goods as "(item qty . rate)", and no sideways scroll', async () => {
    await openRoute('#/reports/R5');
    await waitForHeading('ur', 'report-r5-title');
    await focusField('ur', 'report-filter-from');
    await type(DAY);
    await focusField('ur', 'report-filter-to');
    await type(DAY);
    await waitForAnswer('ur');
    expect(await rowCount()).toBeGreaterThan(0);
    // The numbered column is the first one of the table.
    expect(await $$('.ag-header-cell[col-id="serial"]')).toHaveLength(1);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r5', 'ur');
  });

  it('2. R1 asks for a بکری first and draws no rows', async () => {
    await openRoute('#/reports/R1');
    await waitForHeading('ur', 'report-r1-title');
    await waitForText(t('ur', 'report-pick-bikri'));
    expect(await hasBox(t('ur', 'report-filter-bikri'))).toBe(true);
    expect(await rowCount()).toBe(0);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r1-pick', 'ur');
  });

  it('3. the number typed and Enter shows the lots of that بکری with a totals row', async () => {
    await pickBikri('ur');
    await waitForAnswer('ur');
    expect(await rowCount()).toBeGreaterThan(0);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r1-bikri', 'ur');
  });

  it('4. F9 opens the print box of that بکری, and Esc closes it', async () => {
    // The focus is in the box; F9 is a key of the whole screen.
    await press(Key.F9);
    await waitForText(t('ur', 'print-box-title', { number: 1 }));
    expect(await showsNoRawText()).toBe(true);
    await press(Key.Escape);
    await browser.waitUntil(
      async () => !(await pageText()).includes(t('ur', 'print-box-title', { number: 1 })),
      { timeout: WAIT_MS, timeoutMsg: 'The print box never closed.' },
    );
  });
});

describe('credit buyers details and the sale book report (English)', () => {
  it('5. Alt+L turns R1 to English, left to right, with the same بکری and rows', async () => {
    const rows = await rowCount();
    await switchLanguage('en');
    await waitForText(t('en', 'report-r1-title'));
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    // The بکری picked is still in its box and the lots are still drawn.
    const box = await focusField('en', 'report-filter-bikri');
    expect(await box.getValue()).toContain('1');
    await browser.waitUntil(async () => (await rowCount()) === rows, { timeout: WAIT_MS });
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r1-bikri', 'en');
  });

  it('6. R5 is in English, left to right, with the same buyers', async () => {
    await openRoute('#/reports/R5');
    await waitForHeading('en', 'report-r5-title');
    await focusField('en', 'report-filter-from');
    await type(DAY);
    await focusField('en', 'report-filter-to');
    await type(DAY);
    await waitForAnswer('en');
    expect(await rowCount()).toBeGreaterThan(0);
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r5', 'en');
    await switchLanguage('ur');
  });
});

describe('credit buyers slips (Urdu)', () => {
  it('7. Print all slips shows one sheet for each credit buyer, and Esc goes back to the table', async () => {
    await openRoute('#/reports/R5');
    await waitForHeading('ur', 'report-r5-title');
    await focusField('ur', 'report-filter-from');
    await type(DAY);
    await focusField('ur', 'report-filter-to');
    await type(DAY);
    await waitForAnswer('ur');
    const buyers = await rowCount();
    expect(buyers).toBeGreaterThan(0);
    await (await button('ur', 'report-print-all')).click();
    await browser.waitUntil(async () => (await $$('.print-sheet')).length > 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'The slips never opened.',
    });
    // One sheet for each buyer of the table, each with his own totals row.
    expect(await $$('.print-sheet')).toHaveLength(buyers);
    expect(await $$('.print-sheet tfoot')).toHaveLength(buyers);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r5-slips', 'ur');
    await press(Key.Escape);
    await browser.waitUntil(async () => (await $$('.print-sheet')).length === 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'Esc never went back to the table.',
    });
    await waitForAnswer('ur');
    expect(await rowCount()).toBe(buyers);
  });
});
