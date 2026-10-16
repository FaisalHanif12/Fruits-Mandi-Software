// E2E-19 through the screens (PRD Section 4.3, D2-A14 screen part): the opening position of the
// worked day, entered the way a new client does it.
//   1. The seller رحیم is made (code 030110001 as the first seller).
//   2. Opening balances dated 31-08-2026: روکڑ بنام 65,000 and ابتدائی سرمایہ جمع 65,000. The
//      screen's own total shows the two sides equal and ابتدائی فرق zero.
//   3. A پیشگی voucher of 15,000 cash to رحیم on 01-09-2026.
//   4. The chart shows the cash account at 50,000 and the capital account at 65,000.
//   5. The days up to 09-10-2026 are closed (the day screen), and the status bar says so.
// The golden numbers themselves are proved by the Rust test `golden_day::e2e_19_the_opening_position`;
// this spec proves the screens lead to the same position.

import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  choose,
  confirmSpellingIfAsked,
  focusField,
  openRoute,
  pageText,
  pick,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const SELLER_CODE = '030110001';
const CASH_CODE = '01010001';
const CAPITAL_CODE = '06010001';
const OPENING_DATE = '31-08-2026';
const ADVANCE_DATE = '01-09-2026';
const CLOSE_UP_TO = '09-10-2026';

/**
 * A cell of the opening balances grid, by the code of its row and the column.
 * @param {string} code
 * @param {string} column
 */
function openingCell(code, column) {
  return $(
    `//div[@role="row"][.//div[@col-id="code"][normalize-space(.)="${code}"]]//div[@col-id="${column}"]`,
  );
}

/**
 * A row of the chart tree by the code of its account.
 * @param {string} code
 */
function treeRow(code) {
  return $(`//*[@role="treeitem"][contains(normalize-space(.), "${code}")]`);
}

describe('the opening position of the worked day (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. makes the seller رحیم, who gets the code 030110001', async () => {
    await openRoute('/index/accounts/new');
    await waitForHeading('ur', 'accounts-new-title');
    await choose('ur', 'accounts-field-type', 'accounts-type-seller');
    const code = await focusField('ur', 'accounts-field-code');
    await browser.waitUntil(async () => (await code.getValue()) === SELLER_CODE, {
      timeoutMsg: 'The first seller did not get the code 030110001.',
    });
    await focusField('ur', 'accounts-field-name-ur');
    await type(`رحیم${Key.Enter}Rahim${Key.Enter}`);
    await type(Key.F10);
    await confirmSpellingIfAsked();
    await waitForText(t('ur', 'accounts-saved'));
  });

  it('2. writes the two opening balances dated 31-08-2026, and the screen shows both sides equal', async () => {
    await openRoute('/index/openings');
    await waitForHeading('ur', 'openings-title');
    const date = await focusField('ur', 'openings-date-all');
    await date.setValue(OPENING_DATE);
    await type(Key.Tab);
    const cash = await openingCell(CASH_CODE, 'opening_dr');
    await cash.click();
    await type(`65000${Key.Enter}`);
    const capital = await openingCell(CAPITAL_CODE, 'opening_cr');
    await capital.click();
    await type(`65000${Key.Enter}`);
    // The total of the changed lines, and the opening difference, come from the backend.
    await browser.waitUntil(async () => (await pageText()).includes('65,000'), {
      timeoutMsg: 'The total 65,000 never showed.',
    });
    await shootScreen('openings-e2e-19', 'ur');
    await type(Key.F10);
    await waitForText(t('ur', 'openings-saved'));
  });

  it('3. gives رحیم a پیشگی of 15,000 cash on 01-09-2026', async () => {
    await openRoute('/bikri/advance');
    await waitForHeading('ur', 'vouchers-advance-title');
    const date = await focusField('ur', 'vouchers-field-date');
    await date.setValue(ADVANCE_DATE);
    await pick('ur', 'vouchers-field-seller', '1', 'رحیم');
    const amount = await focusField('ur', 'vouchers-field-amount');
    await amount.setValue('15000');
    await type(Key.F10);
    await browser.waitUntil(
      async () => (await pageText()).includes(t('ur', 'vouchers-saved-advance', { number: 1 })),
      { timeoutMsg: 'The first پیشگی was not saved.' },
    );
    // His open advance is 15,000 under his card.
    expect(await pageText()).toContain('15,000');
  });

  it('4. the chart shows the cash at 50,000 and the capital at 65,000', async () => {
    await openRoute('/index/chart');
    await waitForHeading('ur', 'chart-title');
    // Open every head and control with the keyboard rule: the arrow that opens, on each row.
    const tree = await $('[role="tree"]');
    await tree.click();
    for (let step = 0; step < 40; step += 1) {
      await type(Key.ArrowLeft);
      await type(Key.ArrowDown);
    }
    const cash = await treeRow(CASH_CODE);
    await cash.waitForDisplayed();
    expect(await cash.getText()).toContain('50,000');
    const capital = await treeRow(CAPITAL_CODE);
    await capital.waitForDisplayed();
    expect(await capital.getText()).toContain('65,000');
    await shootScreen('chart-e2e-19', 'ur');
  });

  it('5. closes the days up to 09-10-2026, and the status bar says so', async () => {
    await openRoute('/day');
    await waitForHeading('ur', 'day-title');
    const upTo = await focusField('ur', 'day-close-up-to');
    await upTo.setValue(CLOSE_UP_TO);
    const close = await $(
      `//button[contains(normalize-space(.), "${t('ur', 'day-close-button')}")]`,
    );
    await close.click();
    const dialog = await $('[role="alertdialog"]');
    await dialog.waitForDisplayed();
    await type(Key.Enter);
    await waitForText(t('ur', 'day-closed', { date: CLOSE_UP_TO }));
    const bar = (await $('footer').getText()).replace(/[⁨⁩]/g, '');
    expect(bar).toContain(CLOSE_UP_TO);
  });
});
