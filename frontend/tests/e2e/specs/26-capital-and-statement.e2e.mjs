// Plan 17 T4 (D4-A31, D4-A32, D4-A33, PRD 17.3, 17.4, FR-17.7, FR-17.8, FR-17.9, FR-17.10, FR-7.67,
// FR-6.7, E2E-35, E2E-37 screen parts): the سرمایہ screen, the partner overview and the partner
// statement worked with the keyboard, in Urdu, after spec 25.
//   1. On سرمایہ, partner ب is chosen and 50,000 is typed as سرمایہ جمع; his balance after shows
//      before the save; F10 saves; the history under the form has the entry and the balance.
//   2. The type list is changed to سرمایہ واپسی: the partner and the amount stay. An amount above
//      his balance shows the warning and still saves (FR-17.8).
//   3. The partner overview has the columns of plan 17 (returned, خالص, capital put in, the trips
//      column and کل رقم), and its button opens the statement of that partner.
//   4. The statement lists a dated line for the capital with its series number and the running کل
//      رقم; a note line has no amount. A language switch keeps the partner and the lines.
// The numbers are proved by the Rust tests `golden_day` and `partner_money` (E2E-35, E2E-37,
// D4-A33); this spec proves the screens send what was typed and show what the backend says. It runs
// on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
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
 * Opens a drop-down by its label and takes the option with these words (a partner's name is data).
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

describe('سرمایہ, the partner overview and the partner statement by keyboard (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. partner ب puts in 50,000: the balance after shows before the save, F10 saves, the history has it', async () => {
    await openRoute('/partners/capital?type=capital_in');
    await waitForHeading('ur', 'partners-capital-screen-title');
    await waitForText(t('ur', 'partners-capital-pick'));
    await chooseNamed('cashbook-field-partner', 'ب');
    await (await focusField('ur', 'cashbook-field-amount')).setValue('50000');
    await waitForText(t('ur', 'cashbook-balance-after'));
    await shootScreen('capital', 'ur');
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'capital_in',
        number: t('ur', 'vouchers-number-capital-in', { number: 1 }),
      }),
    );
    await waitForAll(
      t('ur', 'partners-capital-kind-invested'),
      t('ur', 'partners-capital-balance'),
      '50,000',
    );
    // No bank, cheque or count field (PRD v1.6).
    expect(await pageText()).not.toContain(t('ur', 'vouchers-field-mode'));
  });

  it('2. the type list keeps the partner and the amount; more than the balance warns and still saves', async () => {
    const types = await $$('//div[@role="group"]//button[@aria-pressed]');
    const back = types[1];
    if (back === undefined) throw new Error('the type list has no second choice');
    await back.click();
    await (await focusField('ur', 'cashbook-field-amount')).setValue('60000');
    await waitForText(
      t('ur', 'err-capital-exceeds-balance', { balance: '50,000', over: '10,000' }),
    );
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'capital_out',
        number: t('ur', 'vouchers-number-capital-out', { number: 1 }),
      }),
    );
  });

  it('3. the overview has the columns of plan 17, and its button opens the statement', async () => {
    await openRoute('/partners');
    await waitForHeading('ur', 'partners-title');
    await waitForAll(
      t('ur', 'partners-metric-returned'),
      t('ur', 'partners-metric-net'),
      t('ur', 'partners-metric-capital-in'),
      t('ur', 'partners-metric-ventures'),
      t('ur', 'partners-metric-total'),
    );
    await shootScreen('partners-overview', 'ur');
    const open = await $(
      `//button[contains(@aria-label, "${t('ur', 'partners-open-statement')}")]`,
    );
    await open.click();
    await waitForHeading('ur', 'partners-statement-title');
  });

  it('4. the statement has a dated line for the capital with its number and the running total', async () => {
    await openRoute('/partners/statement?partner=2');
    await waitForHeading('ur', 'partners-statement-title');
    await waitForAll(t('ur', 'vouchers-number-capital-in', { number: 1 }), '50,000');
    await shootScreen('partner-statement', 'ur');
    // A language switch keeps the partner and the lines.
    await press([Key.Alt, 'l']);
    await waitForHeading('en', 'partners-statement-title');
    await waitForAll(t('en', 'vouchers-number-capital-in', { number: 1 }), '50,000');
    await shootScreen('partner-statement', 'en');
    await press([Key.Alt, 'l']);
    await waitForHeading('ur', 'partners-statement-title');
  });
});
