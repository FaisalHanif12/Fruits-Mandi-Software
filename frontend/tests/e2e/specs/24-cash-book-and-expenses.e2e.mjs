// Plan 17 F1 (D4-A22 for the خرچہ screen, D4-A28 type switching, PRD 16.1, 16.2, 16.3, FR-7.67,
// FR-16.8, FR-6.7, E2E-39 screen part): the روکڑ area worked with the keyboard only, in Urdu, on the
// day of Event 5 (after the specs 12 to 23).
//   1. Alt+K opens the خرچہ screen as دکان خرچہ. The head بجلی and 3,000 are typed; the split of the
//      backend shows (1,200 / 900 / 900) before the save; F10 saves; the message names the entry.
//   2. 2,000 is typed as دکان خرچہ by mistake, then the type list is changed to ذاتی خرچہ: the amount
//      stays, the head is gone, the screen asks for the partner and the reason (FR-7.67).
//   3. الف and the reason are chosen; his total now and after shows (FR-16.8); F10 saves, and his money
//      taken is listed under the form.
//   4. The روکڑ بک (side menu): it has the movements with their kinds, the opening and closing cash,
//      and no box to type counted cash (FR-16.1).
//   5. The language switch in the middle of an entry loses nothing (F39).
// The numbers themselves are proved by the Rust test `golden_day` (E2E-9, E2E-20, E2E-39); this spec
// proves the screen sends what was typed, shows what the backend says and works by keyboard. It runs
// on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { t } from '../support/messages.mjs';
import {
  Key,
  button,
  focusField,
  pageText,
  press,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { OWNER } from '../support/credentials.mjs';
import { shootScreen } from '../support/screens.mjs';

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

/**
 * Opens a drop-down by its label and takes the option with these words (a head or a partner: their
 * names are data, not Fluent messages).
 * @param {string} fieldId
 * @param {string} words
 */
async function chooseNamed(fieldId, words) {
  const box = await focusField('ur', fieldId);
  await box.click();
  const option = await $(`//*[@role="option"][contains(normalize-space(.), "${words}")]`);
  await option.waitForDisplayed({ timeout: WAIT_MS });
  await option.click();
}

describe('the روکڑ area by keyboard: خرچہ, type switching and the cash book (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. Alt+K opens the خرچہ screen as دکان خرچہ; the split shows before the save and F10 saves', async () => {
    await press([Key.Alt, 'k']);
    await waitForHeading('ur', 'cashbook-expense-title');
    await chooseNamed('cashbook-field-head', 'بجلی');
    const amount = await focusField('ur', 'cashbook-field-amount');
    await amount.setValue('3000');
    await waitForAll(t('ur', 'cashbook-shares-title'), '1,200', '900');
    await shootScreen('expense-shop', 'ur');
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'shop_expense',
        number: t('ur', 'vouchers-number-shop-expense', { number: 1 }),
      }),
    );
    // The list of the day under the form has it.
    await waitForAll(t('ur', 'vouchers-number-shop-expense', { number: 1 }), '3,000');
  });

  it('2. changing the type keeps the amount and clears the head (FR-7.67)', async () => {
    const amount = await focusField('ur', 'cashbook-field-amount');
    await amount.setValue('2000');
    const types = await $$('//div[@role="group"]//button[@aria-pressed]');
    const personal = types[1];
    if (personal === undefined) throw new Error('the type list has no second choice');
    await personal.click();
    await waitForAll(t('ur', 'cashbook-field-partner'), t('ur', 'cashbook-field-reason'));
    expect(await (await focusField('ur', 'cashbook-field-amount')).getValue()).toBe('2,000');
  });

  it('3. ذاتی خرچہ for الف shows his total now and after, and F10 saves; his money taken is listed', async () => {
    await chooseNamed('cashbook-field-partner', 'الف');
    const reason = await focusField('ur', 'cashbook-field-reason');
    await reason.setValue('گھر کا خرچ');
    await waitForAll(t('ur', 'cashbook-effect-title'), t('ur', 'cashbook-others-not-affected'));
    await shootScreen('expense-personal', 'ur');
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'personal_expense',
        number: t('ur', 'vouchers-number-personal-expense', { number: 1 }),
      }),
    );
    await waitForAll(t('ur', 'cashbook-takes-title'), '2,000');
  });

  it('4. the روکڑ بک has the movements, the opening and closing cash and no box for counted cash', async () => {
    const menu = await $(
      `//nav[@aria-label="${t('ur', 'layout-rokar-label').replace(/[⁨⁩]/g, '')}"]`,
    );
    const first = await menu.$('button');
    await first.click();
    await waitForHeading('ur', 'cashbook-title');
    await waitForAll(t('ur', 'cashbook-opening'), t('ur', 'cashbook-closing'));
    const text = await pageText();
    expect(text).toContain(t('ur', 'cashbook-kind-shop-expense'));
    // Only the two date fields can be typed in: there is no cash count box (PRD FR-16.1).
    const inputs = await $$('main input[type="text"]');
    expect(inputs.length).toBe(2);
    await shootScreen('cashbook', 'ur');
  });

  it('5. a language switch in the middle of an entry loses nothing', async () => {
    await press([Key.Alt, 'k']);
    await waitForHeading('ur', 'cashbook-expense-title');
    const amount = await focusField('ur', 'cashbook-field-amount');
    await amount.setValue('777');
    await press([Key.Alt, 'l']);
    await waitForHeading('en', 'cashbook-expense-title');
    expect(await (await focusField('en', 'cashbook-field-amount')).getValue()).toBe('777');
    await shootScreen('expense-shop', 'en');
    // The new-entry button is there in English too; leave the screen clean for the specs after it.
    await (await button('en', 'cashbook-new')).click();
    await press([Key.Alt, 'l']);
    await waitForHeading('ur', 'cashbook-expense-title');
  });
});
