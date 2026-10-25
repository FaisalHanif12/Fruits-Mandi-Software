// Plan 17e F2 (PRD 10.2, FR-7.40, FR-15.6, FR-19.13, Section 21, NFR-URD-14): the tag (نشان), the firm's
// contacts and trade mark, and the collection list printed to PDF in Urdu AND in English through the
// real print window of WebView2. It runs after the worked day (specs 24 to 30).
//   1. A new buyer is made with the tag K, a collection day of today and an opening debit of 5,000.
//      The accounts list shows his name with the tag after it.
//   2. The global search (Ctrl+K) finds him by "ashiq k", and shows the tag after his name.
//   3. The firm settings take a contact person, his phone and the trade mark RH, and say saved.
//   4. The collection screen: F9 opens the print box. For each print language, اردو then English: the
//      language and "PDF" are chosen and "Save PDF" is pressed. The box lists `collection-list-<day>-
//      <lang>.pdf`; it is on disk, a real PDF, one page. The app stays in Urdu.
//   5. The preview in each print language shows the list as it prints: its language, direction and
//      paper, the buyer with his tag, the dues 5,000, the total of this list, the trade mark and the
//      contact in the header, and EVERY digit of the sheet is 0-9 (no Urdu or Arabic-Indic digit).
// The numbers and the words themselves are proved by the Rust snapshots of `mandi-reports` and the
// Vitest tests; this spec proves WebView2 really makes the file, shapes the Urdu and shows the digits as
// Latin. It runs on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { existsSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  choose,
  currentLanguage,
  focusField,
  openRoute,
  pageText,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const LIST_WAIT_MS = 15_000;
const PRINT_WAIT_MS = 60_000;
const MIN_PDF_BYTES = 1_000;

/** The buyer of the story: Ashiq Zafar with the tag K, in both scripts. */
const BUYER = { urdu: 'عاشق ظفر', english: 'Ashiq Zafar', tag: 'k', capital: 'K' };
const OPENING_DEBIT = '5000';

/** The names of the days by the backend's number: 0 is Monday, 6 is Sunday (AccountForm). */
const WEEKDAY_NAMES = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];

/** The print languages under test, and the words that name each in its own script. */
const PRINT_LANGUAGES = /** @type {const} */ ([
  { language: 'ur', name: 'اردو' },
  { language: 'en', name: 'English' },
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

/** The files the box lists after "Save PDF". @returns {Promise<string[]>} */
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

/**
 * Every character of a text that is a decimal digit but not 0 to 9: Urdu, Arabic-Indic and any other
 * numeral style (PRD FR-7.32, NFR-URD-14).
 * @param {string} text
 */
function oddDigits(text) {
  return (text.match(/\p{Nd}/gu) ?? []).filter((char) => !/[0-9]/.test(char));
}

/**
 * The first input that follows a label with exactly these words (the contact person's fields: the
 * firm's own phone has the same word, so the contact is the one inside the contacts block).
 * @param {string} words
 */
async function firstInput(words) {
  const inputs = await $$(
    `//section[.//h2]//label[normalize-space(translate(., "⁨⁩", ""))="${words}"]/following::input[1]`,
  );
  const [first] = inputs;
  if (first === undefined) throw new Error(`No field with the label "${words}"`);
  return first;
}

/** The backend's number of today's weekday: 0 is Monday. */
function todayWeekday() {
  return (new Date().getDay() + 6) % 7;
}

describe('the tag, the firm contacts and the collection list on paper (Urdu app)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. a buyer is made with the tag K, today as his collection day and 5,000 owing; the list shows the tag after the name', async () => {
    await openRoute('/index/accounts/new');
    await waitForHeading('ur', 'accounts-new-title');
    const urdu = await focusField('ur', 'accounts-field-name-ur');
    await urdu.setValue(BUYER.urdu);
    const english = await focusField('ur', 'accounts-field-name-en');
    await english.setValue(BUYER.english);
    const tag = await focusField('ur', 'accounts-field-tag');
    await tag.setValue(BUYER.tag);
    await browser.keys(Key.Tab);
    // The tag is made capital when the field is left, and is never turned into Urdu.
    expect(await tag.getValue()).toBe(BUYER.capital);
    const debit = await focusField('ur', 'accounts-field-opening-dr');
    await debit.setValue(OPENING_DEBIT);
    const today = WEEKDAY_NAMES[todayWeekday()] ?? 'mon';
    await choose('ur', 'accounts-field-weekday', `accounts-weekday-${today}`);
    await shootScreen('account-new-with-tag', 'ur');
    await type(Key.F10);
    await waitForText(t('ur', 'accounts-saved'));
    await openRoute('/index/accounts');
    await waitForText(`${BUYER.urdu} ${BUYER.capital}`);
    await shootScreen('accounts-list-with-tag', 'ur');
  });

  it('2. the global search finds him by "ashiq k" and shows the tag after his name', async () => {
    await browser.keys([Key.Ctrl, 'k']);
    await type('ashiq k');
    const found = await $('[role="option"]');
    await found.waitForDisplayed({ timeout: LIST_WAIT_MS });
    const text = (await found.getText()).replace(/[⁨⁩]/g, '');
    expect(text).toContain(`${BUYER.urdu} ${BUYER.capital}`);
    await shootScreen('global-search-with-tag', 'ur');
    await type(Key.Escape);
  });

  it('3. the firm settings take a contact person with his phone and the trade mark RH', async () => {
    await openRoute('/settings');
    await waitForHeading('ur', 'settings-title');
    const add = await $(
      `//button[contains(normalize-space(.), "${t('ur', 'settings-firm-contact-add')}")]`,
    );
    await add.waitForDisplayed({ timeout: LIST_WAIT_MS });
    await add.click();
    const contactEnglish = await firstInput(t('ur', 'settings-firm-contact-name-en'));
    await contactEnglish.setValue('Asif Bhatti');
    const contactPhone = await firstInput(t('ur', 'settings-firm-contact-phone'));
    await contactPhone.setValue('0300-4058860');
    const mark = await focusField('ur', 'settings-firm-trade-mark');
    await mark.setValue('RH');
    await shootScreen('settings-firm-contacts', 'ur');
    await type(Key.F10);
    await waitForText(t('ur', 'settings-saved'));
  });

  it('4. F9 on the collection screen opens the print box for the day on the screen', async () => {
    await openRoute('/bikri/collection');
    await waitForHeading('ur', 'vouchers-collection-title');
    const row = await $('//div[@role="row"][@row-index="0"]');
    await row.waitForDisplayed({ timeout: LIST_WAIT_MS });
    await type(Key.F9);
    await waitForText(t('ur', 'vouchers-collection-print-title'));
    await shootScreen('collection-print-box', 'ur');
  });

  for (const { language, name } of PRINT_LANGUAGES) {
    it(`5. saves the list as PDF in ${name}, one page, and the app stays in Urdu`, async () => {
      await (await boxButton(name)).click();
      await (await boxButton(t('ur', 'print-box-to-pdf'))).click();
      await (await boxButton(t('ur', 'print-box-save-pdf'))).click();
      await browser.waitUntil(
        async () => (await pageText()).includes(t('ur', 'print-box-done-pdf')),
        { timeout: PRINT_WAIT_MS, timeoutMsg: 'The box never said the PDF file was saved' },
      );
      const files = await listedFiles();
      const list = files.find(
        (file) =>
          path.basename(file).startsWith('collection-list-') &&
          path.basename(file).endsWith(`-${language}.pdf`),
      );
      if (list === undefined) {
        throw new Error(
          `The box did not list the list in ${language}. It listed: ${files.join(', ')}`,
        );
      }
      if (!existsSync(list)) throw new Error(`The box listed a file that is not on disk: ${list}`);
      expect(statSync(list).size).toBeGreaterThan(MIN_PDF_BYTES);
      expect(readFileSync(list).subarray(0, 4).toString('latin1')).toBe('%PDF');
      expect(pagesOf(list)).toBe(1);
      expect(await currentLanguage()).toBe('ur');
    });
  }

  for (const { language, name } of PRINT_LANGUAGES) {
    it(`6. the preview shows the list in ${name}: the tag, the dues, the total, the header and 0-9 digits only`, async () => {
      await (await boxButton(name)).click();
      await (await boxButton(t('ur', 'print-box-preview'))).click();
      await browser.waitUntil(
        async () => (await $$(`${BOX}//div[contains(@class, "print-doc")]`)).length >= 1,
        { timeout: PRINT_WAIT_MS, timeoutMsg: 'The preview never showed the list' },
      );
      const sheet = await $(`${BOX}//div[contains(@class, "print-doc")]`);
      expect(await sheet.getAttribute('lang')).toBe(language);
      expect(await sheet.getAttribute('dir')).toBe(language === 'ur' ? 'rtl' : 'ltr');
      expect(await sheet.getAttribute('data-digits')).toBe('latin');
      const text = (await sheet.getText()).replace(/[⁨⁩]/g, '');
      const buyer = language === 'ur' ? BUYER.urdu : BUYER.english;
      expect(text).toContain(`${buyer} ${BUYER.capital}`);
      expect(text).toContain('5,000');
      expect(text).toContain(t(language, 'print-collection-total'));
      expect(text).toContain(t(language, 'print-collection-list-title'));
      // The firm's trade mark and contact person are in the header (FR-19.13, Section 21).
      expect(text).toContain('RH');
      expect(text).toContain('0300-4058860');
      // Every digit of the sheet is 0-9, in Urdu as well as in English (PRD FR-7.32, NFR-URD-14).
      expect(oddDigits(text)).toEqual([]);
      await shootScreen(`collection-print-preview-${language}`, 'ur');
      await (await boxButton(t('ur', 'common-back'))).click();
    });
  }

  it('7. Esc closes the box', async () => {
    await type(Key.Escape);
    const box = await $(BOX);
    await box.waitForExist({ reverse: true, timeout: LIST_WAIT_MS });
  });
});
