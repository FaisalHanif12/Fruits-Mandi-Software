// D2-A11 (PRD FR-7.42, FR-10.8, FR-7.64): a new account is added from a picker without leaving
// the screen. On the پیشگی screen, in the seller field:
//  - an unknown full code is typed and the field is left: the screen says there is no such seller,
//    F4 opens the small form with that code, the names are typed (English letters become Urdu),
//    F10 saves, and the new seller is in the field, with his card;
//  - F4 on text that matches nobody opens the same form with the text as the name.
// The form closes and the screen under it is as it was.

import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  confirmSpellingIfAsked,
  focusField,
  openRoute,
  pageText,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const UNKNOWN_CODE = '030115555';
const DIALOG_WAIT_MS = 10_000;

describe('quick add on the advance screen (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
    await openRoute('/bikri/advance');
    await waitForHeading('ur', 'vouchers-advance-title');
  });

  it('an unknown code says so, and F4 opens the form with that code', async () => {
    const seller = await focusField('ur', 'vouchers-field-seller');
    await seller.setValue(UNKNOWN_CODE);
    await type(Key.Tab);
    await waitForText(t('ur', 'accountpicker-unknown-seller'));
    await shootScreen('advance-unknown-code', 'ur');
    await browser.keys(Key.F4);
    const dialog = await $('[role="dialog"]');
    await dialog.waitForDisplayed({ timeout: DIALOG_WAIT_MS });
    expect(await pageText()).toContain(t('ur', 'accountpicker-add-title'));
    const code = await focusField('ur', 'accounts-field-code');
    expect(await code.getValue()).toBe(UNKNOWN_CODE);
  });

  it('the names are typed in English letters, F10 saves, and the seller is in the field', async () => {
    await focusField('ur', 'accounts-field-name-ur');
    await type('zahid ');
    await type('ali');
    await type(Key.Enter);
    await type(Key.Enter);
    await type(Key.F10);
    await confirmSpellingIfAsked();
    const dialog = await $('[role="dialog"]');
    await dialog.waitForDisplayed({ timeout: DIALOG_WAIT_MS, reverse: true });
    const seller = await focusField('ur', 'vouchers-field-seller');
    expect((await seller.getValue()).length).toBeGreaterThan(0);
    // His card is under the field (PRD FR-7.38), and the screen is the same one.
    expect(await pageText()).toContain(UNKNOWN_CODE);
    await waitForHeading('ur', 'vouchers-advance-title');
    await shootScreen('advance-quick-added', 'ur');
  });

  it('F4 on text that matches nobody opens the form with the text as the name', async () => {
    const seller = await focusField('ur', 'vouchers-field-seller');
    await seller.setValue('bilal');
    await waitForText(t('ur', 'search-no-match'));
    await browser.keys(Key.F4);
    const dialog = await $('[role="dialog"]');
    await dialog.waitForDisplayed({ timeout: DIALOG_WAIT_MS });
    const english = await focusField('ur', 'accounts-field-name-en');
    expect(await english.getValue()).toBe('bilal');
    // Esc closes it without saving.
    await type(Key.Escape);
    await dialog.waitForDisplayed({ timeout: DIALOG_WAIT_MS, reverse: true });
    await waitForHeading('ur', 'vouchers-advance-title');
  });
});
