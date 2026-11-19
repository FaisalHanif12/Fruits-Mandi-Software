// Plan 15 (D4-A13 slip part, PRD FR-15.2, FR-7.36, FR-19.7): the receipt slip of the receipt saved by
// spec 16 (R-1, 10,000 from اکرم) printed to PDF in Urdu AND in English, through the real print window
// of WebView2.
//   1. Opens the voucher screen on وصولی, chooses the receipt in the list and presses F9.
//   2. The print box names the receipt.
//   3. For each print language, اردو then English: picks the language and "PDF", presses "Save PDF".
//      The box lists a file; it is on disk, is a real PDF, and is one page. The app stays in Urdu.
//   4. The preview shows the slip in each print language, with the amount in words (دس ہزار روپے,
//      "Ten thousand rupees only").
// The numbers and the words themselves are proved by the Rust snapshots of `mandi-reports`; this spec
// proves WebView2 really makes the file. It runs on Windows only (LOCAL-ONLY in the Mac phase:
// checked with `node --check`).

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
  { language: 'ur', name: 'اردو', words: 'دس ہزار روپے' },
  { language: 'en', name: 'English', words: 'Ten thousand rupees only' },
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

describe('printing the receipt slip to PDF in Urdu and English', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. F9 on the receipt in the list opens the print box for it', async () => {
    await openRoute('/bikri/receipt');
    await waitForHeading('ur', 'vouchers-receipt-title');
    const row = await $('//div[@role="row"][@row-index="0"]//div[@col-id="number"]');
    await row.waitForDisplayed({ timeout: LIST_WAIT_MS });
    await row.click();
    await type(Key.F9);
    await waitForText(
      t('ur', 'vouchers-print-title', {
        number: t('ur', 'vouchers-number-receipt', { number: 1 }),
      }),
    );
    await shootScreen('voucher-print-box', 'ur');
  });

  for (const { language, name } of PRINT_LANGUAGES) {
    it(`2. saves the slip as PDF in ${name}, one page, and the app stays in Urdu`, async () => {
      await (await boxButton(name)).click();
      await (await boxButton(t('ur', 'print-box-to-pdf'))).click();
      await (await boxButton(t('ur', 'print-box-save-pdf'))).click();
      await browser.waitUntil(
        async () => (await pageText()).includes(t('ur', 'print-box-done-pdf')),
        { timeout: PRINT_WAIT_MS, timeoutMsg: 'The box never said the PDF file was saved' },
      );
      const files = await listedFiles();
      const slip = files.find((file) => path.basename(file).endsWith(`-${language}.pdf`));
      if (slip === undefined) {
        throw new Error(
          `The box did not list the slip in ${language}. It listed: ${files.join(', ')}`,
        );
      }
      if (!existsSync(slip)) throw new Error(`The box listed a file that is not on disk: ${slip}`);
      expect(statSync(slip).size).toBeGreaterThan(MIN_PDF_BYTES);
      expect(readFileSync(slip).subarray(0, 4).toString('latin1')).toBe('%PDF');
      expect(pagesOf(slip)).toBe(1);
      expect(await currentLanguage()).toBe('ur');
    });
  }

  for (const { language, name, words } of PRINT_LANGUAGES) {
    it(`3. the preview shows the slip in ${name}, with the amount in words`, async () => {
      await (await boxButton(name)).click();
      await (await boxButton(t('ur', 'print-box-preview'))).click();
      await browser.waitUntil(
        async () => (await $$(`${BOX}//div[contains(@class, "print-doc")]`)).length >= 1,
        { timeout: PRINT_WAIT_MS, timeoutMsg: 'The preview never showed the slip' },
      );
      const sheet = await $(`${BOX}//div[contains(@class, "print-doc")]`);
      expect(await sheet.getAttribute('lang')).toBe(language);
      expect(await sheet.getAttribute('dir')).toBe(language === 'ur' ? 'rtl' : 'ltr');
      const text = await pageText();
      expect(text).toContain('10,000');
      expect(text).toContain(words);
      expect(text).toContain(t(language, 'print-receipt-slip-title'));
      await shootScreen(`voucher-print-preview-${language}`, 'ur');
      await (await boxButton(t('ur', 'common-back'))).click();
    });
  }

  it('4. Esc closes the box', async () => {
    await type(Key.Escape);
    const box = await $(BOX);
    await box.waitForExist({ reverse: true, timeout: LIST_WAIT_MS });
  });
});
