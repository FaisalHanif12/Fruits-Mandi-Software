// D5 early track, plan 20 and 21 (D5-Q1, FR-19.3, FR-18.4, NFR-USE-7, NFR-USE-15, NFR-USE-28): the Excel
// export of a report and of an analytics view, by keyboard, in Urdu and then in English, on the real app
// after the worked day.
//   1. R18 in Urdu: Tab goes from Print preview to Excel; Enter writes the file; the notice says where
//      it is and how many rows it holds; Esc closes the notice. The file is a real .xlsx on disk.
//   2. Analytics view A in English: the same, and the file name starts with the view.
//   3. A path the backend refuses (no folder) is told in words and nothing is written.
//   4. The print preview has its own Excel button, beside PDF and Print.
// The system's save box cannot be driven by WebDriver, so the spec answers that ONE plugin call (the
// save box) with a path in the temp folder, in the real webview; everything else is the real app, and
// the file is checked on disk (it starts with "PK", the mark of an .xlsx, which is a zip). The real save
// box is checked by hand (LOCAL-ONLY). It runs on Windows only (LOCAL-ONLY in the Mac phase: checked
// with `node --check`).

import { existsSync, readFileSync, rmSync, statSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  button,
  press,
  startSignedIn,
  switchLanguage,
  waitForHeading,
  waitForText,
  pageText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WAIT_MS = 30_000;
const FILE = path.join(tmpdir(), 'mandi-e2e-excel.xlsx');

/** Opens a screen by its address. @param {string} address */
async function open(address) {
  await browser.execute((hash) => {
    window.location.hash = hash;
  }, address);
}

/**
 * Answers the save box with `chosen` from now on (the one plugin call WebDriver cannot drive). It is a
 * script text, because the window of the app has properties no type knows.
 */
async function answerSaveBox(chosen) {
  await browser.execute(
    `const internals = window.__TAURI_INTERNALS__;
     if (internals.__realInvoke === undefined) internals.__realInvoke = internals.invoke;
     const chosen = arguments[0];
     internals.invoke = (command, args, options) =>
       command === 'plugin:dialog|save'
         ? Promise.resolve(chosen)
         : internals.__realInvoke(command, args, options);`,
    chosen,
  );
}

/** True when the page does not scroll sideways (NFR-USE-15). */
async function noSidewaysScroll() {
  return browser.execute(() => document.documentElement.scrollWidth <= window.innerWidth);
}

/** The file the export wrote: it exists, is not empty and starts with "PK". */
function expectRealWorkbook() {
  expect(existsSync(FILE)).toBe(true);
  expect(statSync(FILE).size).toBeGreaterThan(0);
  expect(readFileSync(FILE).subarray(0, 2).toString('latin1')).toBe('PK');
}

describe('Excel export (Urdu)', () => {
  before(async () => {
    rmSync(FILE, { force: true });
    await startSignedIn('ur', OWNER);
    await answerSaveBox(FILE);
  });

  it('1. R18: Tab reaches Excel after Print preview, Enter writes a real file, Esc closes the notice', async () => {
    await open('#/reports/R18');
    await waitForHeading('ur', 'report-r18-title');
    const rows = await $('.ag-row[row-id]');
    await rows.waitForDisplayed({ timeout: WAIT_MS });
    const preview = await button('ur', 'report-preview');
    await preview.click();
    await press(Key.Escape);
    await preview.waitForDisplayed({ timeout: WAIT_MS });
    // Focus is on the print button; the next stop is the Excel button (the order of the header).
    await browser.execute((element) => {
      element.focus();
    }, preview);
    await press(Key.Tab);
    const focused = await browser.execute(() => document.activeElement?.textContent ?? '');
    expect(focused).toContain(t('ur', 'export-excel'));
    await press(Key.Enter);
    await browser.waitUntil(
      async () =>
        (await pageText()).includes(FILE) || (await pageText()).includes(t('ur', 'export-close')),
      { timeout: WAIT_MS },
    );
    const close = await button('ur', 'export-close');
    await close.waitForDisplayed({ timeout: WAIT_MS });
    expect(await pageText()).toContain(FILE);
    expectRealWorkbook();
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('excel-report', 'ur');
    await press(Key.Escape);
    await browser.waitUntil(async () => !(await pageText()).includes(FILE), { timeout: WAIT_MS });
    rmSync(FILE, { force: true });
  });

  it('3. a path the backend refuses is told in words and nothing is written', async () => {
    await answerSaveBox(path.join(tmpdir(), 'no-such-folder-for-mandi', 'x.xlsx'));
    const excel = await button('ur', 'export-excel');
    await excel.click();
    await waitForText(t('ur', 'export-path-refused'));
    expect(existsSync(path.join(tmpdir(), 'no-such-folder-for-mandi'))).toBe(false);
    const close = await button('ur', 'export-close');
    await close.click();
    await answerSaveBox(FILE);
  });

  it('4. the print preview has its own Excel button beside PDF and Print', async () => {
    const preview = await button('ur', 'report-preview');
    await preview.click();
    await waitForText(t('ur', 'report-print-pdf'));
    const pdf = await button('ur', 'report-print-pdf');
    const inFrame = await pdf
      .parentElement()
      .$(`.//button[contains(., "${t('ur', 'export-excel')}")]`);
    expect(await inFrame.isDisplayed()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('excel-preview', 'ur');
    await press(Key.Escape);
  });
});

describe('Excel export (English)', () => {
  it('2. analytics view A in English: the file is written and the notice says where', async () => {
    await switchLanguage('en');
    await open('#/analytics/A');
    await waitForHeading('en', 'analytics-view-a-title');
    const rows = await $('.ag-row[row-id]');
    await rows.waitForDisplayed({ timeout: WAIT_MS });
    const excel = await button('en', 'export-excel');
    await excel.click();
    const close = await button('en', 'export-close');
    await close.waitForDisplayed({ timeout: WAIT_MS });
    expect(await pageText()).toContain(FILE);
    expectRealWorkbook();
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('excel-analytics', 'en');
    await close.click();
    rmSync(FILE, { force: true });
    await switchLanguage('ur');
  });
});
