// D5 late track, round 14 (PRD FR-19.3, FR-19.13, FR-7.36, FR-7.32, NFR-URD-14, NFR-USE-15, NFR-USE-28):
// the report prints on the shared print layout (the ONE header with the firm, its contacts and trade
// mark, the title and the filters; the footer of the backend's `reports_print_header`), by keyboard, in
// Urdu and then in English, on the real app after the worked day.
//   1. R5 F9 shows the print: the firm name in the header, the table inside the sheet (no sideways
//      scroll), and every digit on it a plain 0-9 (never an Urdu or Arabic-Indic digit).
//   2. The A5 paper keeps the table inside its sheet.
//   3. "Print all slips" shows one sheet for each credit buyer, and Esc goes back to the table.
//   4. The analytics view J (up to eighteen columns) prints inside its sheet, on A4.
//   5. Alt+L turns the print of R37 to English, left to right, with the same lines.
// The figures are the backend's and are proved by the Rust tests and the fixtures; this spec proves
// that the real app draws the print with the shared header, keeps every table inside its sheet and
// writes only English digits. It runs on Windows only (LOCAL-ONLY in the Mac phase: checked with
// `node --check`). The Urdu shaping of the header and the printing itself are the Windows check.

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

/** True when every sheet holds its table: nothing runs past the sheet, and the page does not scroll. */
async function sheetsHoldTheirTables() {
  return browser.execute(() => {
    const page = document.documentElement;
    const sheets = Array.from(document.querySelectorAll('.print-sheet'));
    return (
      page.scrollWidth <= window.innerWidth &&
      sheets.every((sheet) => sheet.scrollWidth <= sheet.clientWidth)
    );
  });
}

/** True when the print has no digit outside 0-9 (Urdu and Arabic-Indic digits are never printed). */
async function printHasOnlyEnglishDigits() {
  const text = await browser.execute(() => document.querySelector('.print-doc')?.textContent ?? '');
  return !/[۰-۹٠-٩]/.test(text);
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

/** Waits until a print sheet is on the page. */
async function waitForSheet() {
  await browser.waitUntil(async () => (await $$('.print-sheet')).length > 0, {
    timeout: WAIT_MS,
    timeoutMsg: 'The print never showed a sheet.',
  });
}

/** Opens R5 for the day typed. @param {'ur' | 'en'} language */
async function openCreditBuyers(language) {
  await openRoute('#/reports/R5');
  await waitForHeading(language, 'report-r5-title');
  await focusField(language, 'report-filter-from');
  await type(DAY);
  await focusField(language, 'report-filter-to');
  await type(DAY);
  await waitForAnswer(language);
}

describe('the report prints (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. R5 F9 shows the print with the shared header, the table inside its sheet and English digits only', async () => {
    await openCreditBuyers('ur');
    await press(Key.F9);
    await waitForText(t('ur', 'report-print-title'));
    await waitForSheet();
    const firm = await $('.print-header .print-firm-name');
    await firm.waitForDisplayed({ timeout: WAIT_MS });
    expect((await firm.getText()).trim()).not.toBe('');
    expect(await sheetsHoldTheirTables()).toBe(true);
    expect(await printHasOnlyEnglishDigits()).toBe(true);
    await shootScreen('reports-print-r5', 'ur');
  });

  it('2. the A5 paper keeps the table inside its sheet', async () => {
    await (await button('ur', 'print-paper-a5')).click();
    await browser.pause(700);
    await waitForSheet();
    expect(await sheetsHoldTheirTables()).toBe(true);
    expect(await printHasOnlyEnglishDigits()).toBe(true);
    await (await button('ur', 'print-paper-a4')).click();
    await press(Key.Escape);
    await browser.waitUntil(async () => (await $$('.print-sheet')).length === 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'Esc never went back to the table.',
    });
  });

  it('3. Print all slips shows one sheet for each credit buyer, and Esc goes back to the table', async () => {
    await waitForAnswer('ur');
    const buyers = await rowCount();
    expect(buyers).toBeGreaterThan(0);
    await (await button('ur', 'report-print-all')).click();
    await waitForSheet();
    expect(await $$('.print-sheet')).toHaveLength(buyers);
    expect(await sheetsHoldTheirTables()).toBe(true);
    expect(await printHasOnlyEnglishDigits()).toBe(true);
    await shootScreen('reports-print-r5-slips', 'ur');
    await press(Key.Escape);
    await browser.waitUntil(async () => (await $$('.print-sheet')).length === 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'Esc never went back to the table.',
    });
  });

  it('4. the analytics view J prints inside its sheet', async () => {
    await openRoute('#/analytics/J');
    await waitForHeading('ur', 'analytics-view-j-title');
    await waitForAnswer('ur');
    await press(Key.F9);
    await waitForText(t('ur', 'report-print-title'));
    await waitForSheet();
    expect(await sheetsHoldTheirTables()).toBe(true);
    expect(await printHasOnlyEnglishDigits()).toBe(true);
    await shootScreen('analytics-print-j', 'ur');
  });
});

describe('the report prints (English)', () => {
  it('5. Alt+L turns the print of R37 to English, left to right, with English digits only', async () => {
    await openRoute('#/reports/R37');
    await waitForHeading('ur', 'report-r37-title');
    await waitForAnswer('ur');
    await switchLanguage('en');
    await waitForHeading('en', 'report-r37-title');
    await waitForAnswer('en');
    await press(Key.F9);
    await waitForText(t('en', 'report-print-title'));
    await waitForSheet();
    expect(
      await browser.execute(() => document.querySelector('.print-doc')?.getAttribute('dir')),
    ).toBe('ltr');
    expect(await sheetsHoldTheirTables()).toBe(true);
    expect(await printHasOnlyEnglishDigits()).toBe(true);
    await shootScreen('reports-print-r37', 'en');
    await press(Key.Escape);
    await switchLanguage('ur');
  });
});
