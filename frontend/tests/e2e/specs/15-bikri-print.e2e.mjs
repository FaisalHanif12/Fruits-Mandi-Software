// Plan 14 (D3-A15, D3-A16, PRD FR-12.12, FR-7.36, FR-19.3 to FR-19.7, E2E-22): the seller bill and
// the buyer slips of بکری 1 (PRD 4.4) printed to PDF in Urdu AND in English, through the real
// print window of WebView2, and looked at in the in-app preview.
//   1. Opens the saved بکری 1 (made by spec 13) from the global search, and presses F9.
//   2. The print box names بکری 1, ticks the three documents and starts in the app language.
//   3. For each print language, اردو then English: picks the language and "PDF", presses
//      "Save PDF". The box lists the files; each one is on disk, is a real PDF, and has the pages
//      it should (the bill 1 page, the slips 2: one for اکرم and one for ارشد). The app itself
//      stays in Urdu (PRD FR-7.36).
//   4. The preview shows the same documents in each print language: its sheets have the language
//      and direction of the print, the bill says 29,400, 10,084 and 19,316, and the amount in
//      words (انیس ہزار تین سو سولہ روپے, "Nineteen thousand three hundred sixteen rupees only").
// The numbers themselves are proved by the Rust snapshots of `mandi-reports`; this spec proves that
// WebView2 really makes the files. It runs on Windows only (LOCAL-ONLY in the Mac phase: checked
// with `node --check`). Printing to a real printer needs a printer, so the CI run stops at PDF.

import { $, $$, browser, expect } from '@wdio/globals';
import { existsSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  currentLanguage,
  openRoute,
  pageText,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const LIST_WAIT_MS = 10_000;
const PRINT_WAIT_MS = 60_000;
const MIN_PDF_BYTES = 1_000;

/** The print languages under test, and the words that name each in its own script. */
const PRINT_LANGUAGES = /** @type {const} */ ([
  { language: 'ur', name: 'اردو', words: 'انیس ہزار تین سو سولہ روپے' },
  { language: 'en', name: 'English', words: 'Nineteen thousand three hundred sixteen rupees only' },
]);

/** The print box. */
const BOX = '//div[@role="dialog"]';

/**
 * A button of the print box by the words on it.
 * @param {string} words
 */
function boxButton(words) {
  return $(`${BOX}//button[normalize-space(.)="${words}"]`);
}

/**
 * The files the box lists after "Save PDF".
 * @returns {Promise<string[]>}
 */
async function listedFiles() {
  return browser.execute(() =>
    [...document.querySelectorAll('[role="dialog"] bdi')]
      .map((item) => item.textContent.trim())
      .filter((text) => text.toLowerCase().endsWith('.pdf')),
  );
}

/**
 * How many pages a PDF file has, counted from its page objects.
 * @param {string} file
 */
function pagesOf(file) {
  const text = readFileSync(file).toString('latin1');
  return (text.match(/\/Type\s*\/Page(?![s\w])/g) ?? []).length;
}

describe('printing بکری 1 to PDF in Urdu and English (E2E-22)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. F9 on the saved بکری 1 opens the print box with the three documents ticked', async () => {
    await openRoute('/bikri/today');
    await waitForHeading('ur', 'layout-module-today');
    await browser.keys([Key.Ctrl, 'k']);
    await type('LES');
    const found = await $('[role="option"]');
    await found.waitForDisplayed({ timeout: LIST_WAIT_MS });
    await type(Key.Enter);
    await waitForText(t('ur', 'bikri-saved-readonly'));
    await type(Key.F9);
    await waitForText(t('ur', 'print-box-title', { number: 1 }));
    const ticked = await $$(`${BOX}//input[@type="checkbox"]`);
    expect(ticked.length).toBeGreaterThanOrEqual(3);
    for (const box of ticked) expect(await box.isSelected()).toBe(true);
    await shootScreen('bikri-print-box', 'ur');
  });

  for (const { language, name } of PRINT_LANGUAGES) {
    it(`2. saves the bill and the slips as PDF in ${name}, and the app stays in Urdu`, async () => {
      await (await boxButton(name)).click();
      await (await boxButton(t('ur', 'print-box-to-pdf'))).click();
      await (await boxButton(t('ur', 'print-box-save-pdf'))).click();
      await browser.waitUntil(
        async () => (await pageText()).includes(t('ur', 'print-box-done-pdf')),
        {
          timeout: PRINT_WAIT_MS,
          timeoutMsg: 'The box never said the PDF files were saved',
        },
      );
      const files = await listedFiles();
      // The file names of plan 14: <document>-<number>-<language>.pdf.
      const bill = files.find((file) => path.basename(file) === `seller-bill-1-${language}.pdf`);
      const slips = files.find((file) => path.basename(file) === `buyer-slips-1-${language}.pdf`);
      if (bill === undefined || slips === undefined) {
        throw new Error(`The box did not list both files. It listed: ${files.join(', ')}`);
      }
      for (const file of [bill, slips]) {
        if (!existsSync(file))
          throw new Error(`The box listed a file that is not on disk: ${file}`);
        expect(statSync(file).size).toBeGreaterThan(MIN_PDF_BYTES);
        expect(readFileSync(file).subarray(0, 4).toString('latin1')).toBe('%PDF');
      }
      // The bill is one sheet (one seller); the slips are two (اکرم and ارشد), each on its own.
      expect(pagesOf(bill)).toBe(1);
      expect(pagesOf(slips)).toBe(2);
      // The print language never changes the app's own (PRD FR-7.36).
      expect(await currentLanguage()).toBe('ur');
    });
  }

  for (const { language, name, words } of PRINT_LANGUAGES) {
    it(`3. the preview shows the documents in ${name}, with the PRD 4.4 numbers`, async () => {
      // Back to the form (the box may show the answer of the last print), pick the language.
      await (await boxButton(name)).click();
      await (await boxButton(t('ur', 'print-box-preview'))).click();
      await browser.waitUntil(
        async () => (await $$(`${BOX}//div[contains(@class, "print-doc")]`)).length >= 2,
        { timeout: PRINT_WAIT_MS, timeoutMsg: 'The preview never showed the documents' },
      );
      const sheet = await $(`${BOX}//div[contains(@class, "print-doc")]`);
      expect(await sheet.getAttribute('lang')).toBe(language);
      expect(await sheet.getAttribute('dir')).toBe(language === 'ur' ? 'rtl' : 'ltr');
      const text = await pageText();
      for (const number of ['29,400', '10,084', '19,316']) expect(text).toContain(number);
      expect(text).toContain(words);
      expect(text).toContain(t(language, 'print-seller-bill-title'));
      await shootScreen(`bikri-print-preview-${language}`, 'ur');
      await (await boxButton(t('ur', 'common-back'))).click();
    });
  }

  it('4. Esc closes the box', async () => {
    await type(Key.Escape);
    const box = await $(BOX);
    await box.waitForExist({ reverse: true, timeout: LIST_WAIT_MS });
  });
});
