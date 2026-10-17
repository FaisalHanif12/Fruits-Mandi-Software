// Plan 15 (D4-A9, D4-A10, D4-A11, D4-A12, D4-A22 voucher screen, D4-A30 voucher part, PRD 15.2 to
// 15.4, FR-7.58, FR-7.59, FR-7.67, FR-7.68, FR-6.7, E2E-6, E2E-7, E2E-27): the ONE voucher screen
// worked with the keyboard only, in Urdu, on the day of Events 1 and 2 (made by specs 12 and 13).
//   1. وصولی (Alt+V): اکرم, short code 1. His card shows 12,880 and the panel his open bill (بکری 1).
//      10,000 typed: the bill is marked "part"; F10 saves; the message names the receipt and the list
//      under the form has it.
//   2. Changing the type with the type list keeps the amount and clears the account (FR-7.67).
//   3. ادائیگی (Alt+A) to رحیم: his payable is 4,316 (24,316 less 5,000 and the 15,000 advance, already
//      adjusted); "full balance" fills 4,316; F10 saves (E2E-7).
//   4. ادائیگی to کریم: the payable is 10,160 (بکری 2) and "full balance" fills it (E2E-27). It is NOT
//      saved: the trial balance of the whole day is checked later.
//   5. پیشگی (Alt+P): an amount that takes cash below zero shows the amber note, and F10 opens the
//      box "is this entry correct?" with the focus on "No". "No" saves nothing (FR-6.7).
// The numbers themselves are proved by the Rust test `golden_day` (E2E-6, E2E-7, E2E-27); this spec
// proves the screen sends what was typed, shows what the backend says and works by keyboard. It runs
// on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  pageText,
  pick,
  press,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const NUMBER_WAIT_MS = 15_000;

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
    { timeout: NUMBER_WAIT_MS, timeoutMsg: `The page never showed: ${texts.join(', ')}` },
  );
}

/**
 * The value of the amount field now.
 * @returns {Promise<string>}
 */
async function amountValue() {
  return (await focusField('ur', 'vouchers-field-amount')).getValue();
}

describe('the voucher screen: وصولی, ادائیگی and پیشگی by keyboard (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. وصولی from اکرم: his card, his open bill, the bill marked while 10,000 is typed, F10 saves', async () => {
    await press([Key.Alt, 'v']);
    await waitForHeading('ur', 'vouchers-receipt-title');
    await pick('ur', 'vouchers-field-buyer', '1', 'اکرم');
    await waitForAll('12,880', t('ur', 'vouchers-bills-title'));
    const amount = await focusField('ur', 'vouchers-field-amount');
    await amount.setValue('10000');
    await waitForAll(t('ur', 'vouchers-col-settles'), t('ur', 'vouchers-effect-part'), '2,880');
    await shootScreen('voucher-receipt', 'ur');
    await type(Key.F10);
    await waitForText(
      t('ur', 'vouchers-saved', {
        kind: 'receipt',
        number: t('ur', 'vouchers-number-receipt', { number: 1 }),
      }),
    );
    // The list under the form has the receipt, and the form is ready for the next.
    await waitForAll(t('ur', 'vouchers-number-receipt', { number: 1 }), '10,000');
    expect(await amountValue()).toBe('');
  });

  it('2. changing the type keeps the amount and clears the account', async () => {
    const amount = await focusField('ur', 'vouchers-field-amount');
    await amount.setValue('3000');
    const types = await $$('//div[@role="group"]//button[@aria-pressed]');
    // The type list: وصولی, ادائیگی, پیشگی. Pick ادائیگی.
    const payment = types[1];
    if (payment === undefined) throw new Error('the type list has no second choice');
    await payment.click();
    await waitForHeading('ur', 'vouchers-payment-title');
    expect(await amountValue()).toBe('3,000');
    const account = await focusField('ur', 'vouchers-field-paid-to');
    expect(await account.getValue()).toBe('');
  });

  it('3. ادائیگی to رحیم: his payable is 4,316 and "full balance" fills it; F10 saves (E2E-7)', async () => {
    await pick('ur', 'vouchers-field-paid-to', '030110001', 'رحیم');
    await waitForAll(t('ur', 'vouchers-payables-title'), '24,316', '15,000', '4,316');
    const full = await $(
      `//button[starts-with(normalize-space(.), "${t('ur', 'vouchers-fill-full')}")]`,
    );
    await full.click();
    expect(await amountValue()).toBe('4,316');
    await shootScreen('voucher-payment', 'ur');
    await type(Key.F10);
    await waitForText(
      t('ur', 'vouchers-saved', {
        kind: 'payment',
        number: t('ur', 'vouchers-number-payment', { number: 1 }),
      }),
    );
  });

  it('4. ادائیگی to کریم: the payable is 10,160 and "full balance" fills it, not saved (E2E-27)', async () => {
    await pick('ur', 'vouchers-field-paid-to', '030110002', 'کریم');
    await waitForAll(t('ur', 'vouchers-payables-title'), '10,160');
    const full = await $(
      `//button[starts-with(normalize-space(.), "${t('ur', 'vouchers-fill-full')}")]`,
    );
    await full.click();
    expect(await amountValue()).toBe('10,160');
  });

  it('5. a پیشگی that takes cash below zero shows the note, and the box asks with "No" in focus', async () => {
    // Alt+P is the same screen with another type: the typed amount stays, no question is asked.
    await press([Key.Alt, 'p']);
    await waitForHeading('ur', 'vouchers-advance-title');
    await pick('ur', 'vouchers-field-seller', '030110002', 'کریم');
    const amount = await focusField('ur', 'vouchers-field-amount');
    await amount.setValue('9999999');
    await waitForText(t('ur', 'common-cash-confirm-title'));
    await shootScreen('voucher-advance-cash', 'ur');
    await type(Key.F10);
    const no = await $(
      `//div[@role="alertdialog"]//button[contains(., "${t('ur', 'common-cash-confirm-no')}")]`,
    );
    await no.waitForDisplayed({ timeout: NUMBER_WAIT_MS });
    expect(await no.isFocused()).toBe(true);
    await type(Key.Enter);
    await no.waitForExist({ reverse: true, timeout: NUMBER_WAIT_MS });
    // Nothing was saved: the form still holds the amount.
    expect(await amountValue()).toBe('9,999,999');
  });

  it('6. two receipts in the waiting list: F2 puts the first in it, F10 saves both (PRD FR-15.1)', async () => {
    await press([Key.Alt, 'v']);
    await waitForHeading('ur', 'vouchers-receipt-title');
    // The cash amount of step 5 is still typed: the type changed, the amount stayed. Replace it.
    await pick('ur', 'vouchers-field-buyer', '2', 'ارشد');
    const first = await focusField('ur', 'vouchers-field-amount');
    await first.setValue('1000');
    await type(Key.F2);
    // The voucher is in the list, waiting, with the backend's amount; nothing is saved yet.
    await waitForAll(t('ur', 'vouchers-waiting-title'), t('ur', 'vouchers-state-waiting'), '1,000');
    await pick('ur', 'vouchers-field-buyer', '1', 'اکرم');
    const second = await focusField('ur', 'vouchers-field-amount');
    await second.setValue('2000');
    await shootScreen('voucher-waiting-list', 'ur');
    await type(Key.F10);
    await waitForText(t('ur', 'vouchers-saved-many', { count: 2 }));
    // Both are in the list of the day, and nothing is left waiting.
    await waitForAll(t('ur', 'vouchers-number-receipt', { number: 3 }), '1,000', '2,000');
    await browser.waitUntil(
      async () => !(await pageText()).includes(t('ur', 'vouchers-waiting-title')),
      { timeout: NUMBER_WAIT_MS, timeoutMsg: 'The waiting list never went away.' },
    );
  });
});
