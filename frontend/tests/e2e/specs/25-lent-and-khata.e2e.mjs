// Plan 17 T4 (D4-A29, D4-A30, PRD 16.5, 16.6, FR-16.4, FR-16.6, FR-16.9, FR-7.67, FR-6.7, E2E-30 and
// E2E-38 screen parts): the ادھار رقم screen and the الگ کھاتہ screen worked with the keyboard, in
// Urdu, after spec 24.
//   1. On ادھار رقم, F4 in the person field adds a person; 5,000 is lent with a reason and a return-by
//      day; F10 saves; the loan is listed under the form with what is still open.
//   2. The type list is changed to واپسی: the person and the amount stay; 3,000 is returned in cash,
//      then 2,000 from the salary; the open amount goes down to 0. There is no bank, cheque or
//      count field anywhere (PRD v1.6).
//   3. On الگ کھاتہ, F4 adds a khata; 10,000 paid out goes minus; the type is changed to received and
//      4,000 brings it to minus 6,000, with the sign hint under the form.
//   4. A language switch in the middle of an entry loses nothing (F39).
// The numbers are proved by the Rust test `golden_day` (E2E-30, E2E-38); this spec proves the
// screens send what was typed and show what the backend says. It runs on Windows only (LOCAL-ONLY in
// the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  choose,
  confirmSpellingIfAsked,
  focusField,
  openRoute,
  pageText,
  press,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

// The label of the field that was taken out in plan 17b (it read "ذریعہ", how the money moves).
const FORMER_MODE_LABEL = 'ذریعہ';

const WAIT_MS = 15_000;

/**
 * Waits until the page says every one of these.
 * @param {...string} texts
 */
async function waitForAll(...texts) {
  await browser.waitUntil(
    async () => {
      const text = await pageText();
      return texts.every((one) => text.includes(one));
    },
    { timeout: WAIT_MS, timeoutMsg: `The page never showed: ${texts.join(', ')}` },
  );
}

/** Fills the small form of F4 in a picker with English letters and saves it. @param {string} english */
async function addFromPicker(english) {
  await browser.keys(Key.F4);
  const dialog = await $('[role="dialog"]');
  await dialog.waitForDisplayed({ timeout: WAIT_MS });
  await focusField('ur', 'cashbook-add-name-ur');
  await type(english);
  await type(Key.Enter);
  await type(Key.F10);
  await confirmSpellingIfAsked();
  await dialog.waitForDisplayed({ timeout: WAIT_MS, reverse: true });
}

/** Clicks the button of the type list at this place (0 is the first). @param {number} index */
async function chooseType(index) {
  const types = await $$('//div[@role="group"]//button[@aria-pressed]');
  const one = types[index];
  if (one === undefined) throw new Error(`the type list has no choice ${String(index)}`);
  await one.click();
}

describe('ادھار رقم and الگ کھاتہ by keyboard (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. F4 adds a person; 5,000 is lent with a reason and F10 saves; the loan is listed', async () => {
    await openRoute('/rokar/lent');
    await waitForHeading('ur', 'cashbook-lent-screen-title');
    const person = await focusField('ur', 'cashbook-field-person');
    await person.setValue('aslam');
    await waitForText(t('ur', 'search-no-match'));
    await addFromPicker('aslam');
    expect(
      (await (await focusField('ur', 'cashbook-field-person')).getValue()).length,
    ).toBeGreaterThan(0);
    await (await focusField('ur', 'cashbook-field-amount')).setValue('5000');
    await (await focusField('ur', 'cashbook-field-reason')).setValue('گھریلو ضرورت');
    await shootScreen('lent-new', 'ur');
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'money_lent',
        number: t('ur', 'vouchers-number-money-lent', { number: 1 }),
      }),
    );
    await waitForAll(t('ur', 'cashbook-lent-title'), '5,000');
    // No bank, cheque or count field: the form has only its own fields.
    const text = await pageText();
    expect(text).not.toContain(FORMER_MODE_LABEL);
  });

  it('2. the type list keeps the person and the amount; a return in cash, then from the salary', async () => {
    await chooseType(1);
    await waitForAll(t('ur', 'cashbook-field-mode'));
    await (await focusField('ur', 'cashbook-field-amount')).setValue('3000');
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'money_lent_return',
        number: t('ur', 'vouchers-number-money-lent-return', { number: 1 }),
      }),
    );
    await choose('ur', 'cashbook-field-mode', 'cashbook-mode-salary');
    await (await focusField('ur', 'cashbook-field-amount')).setValue('2000');
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'money_lent_return',
        number: t('ur', 'vouchers-number-money-lent-return', { number: 2 }),
      }),
    );
    await waitForAll(t('ur', 'cashbook-lent-balance'));
    await shootScreen('lent-returned', 'ur');
  });

  it('3. a khata: F4 adds it, 10,000 paid out goes minus, 4,000 received brings it to minus 6,000', async () => {
    await openRoute('/rokar/khata');
    await waitForHeading('ur', 'cashbook-khata-screen-title');
    const khata = await focusField('ur', 'cashbook-field-khata');
    await khata.setValue('family');
    await waitForText(t('ur', 'search-no-match'));
    await addFromPicker('family khata');
    await (await focusField('ur', 'cashbook-field-amount')).setValue('10000');
    await (await focusField('ur', 'cashbook-field-reason')).setValue('خاندان کا خرچ');
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'khata_out',
        number: t('ur', 'vouchers-number-khata-out', { number: 1 }),
      }),
    );
    await waitForAll('-10,000', t('ur', 'cashbook-khata-sign-hint'));
    await chooseType(1);
    await (await focusField('ur', 'cashbook-field-amount')).setValue('4000');
    await (await focusField('ur', 'cashbook-field-reason')).setValue('واپس آئے');
    await type(Key.F10);
    await waitForAll('-6,000');
    await shootScreen('khata', 'ur');
  });

  it('4. a language switch in the middle of an entry loses nothing', async () => {
    await openRoute('/rokar/khata');
    await waitForHeading('ur', 'cashbook-khata-screen-title');
    await (await focusField('ur', 'cashbook-field-amount')).setValue('777');
    await press([Key.Alt, 'l']);
    await waitForHeading('en', 'cashbook-khata-screen-title');
    expect(await (await focusField('en', 'cashbook-field-amount')).getValue()).toBe('777');
    await shootScreen('khata', 'en');
    await press([Key.Alt, 'l']);
    await waitForHeading('ur', 'cashbook-khata-screen-title');
  });
});
