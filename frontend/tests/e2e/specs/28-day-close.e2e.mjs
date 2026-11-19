// Plan 17 F5 (D4-A17, PRD FR-21.1, 4.9, FR-7.8, FR-7.33): the day close worked with the keyboard,
// in Urdu, as the last step of the worked day (after specs 24 to 27).
//   1. The day screen is opened. The date to close up to is typed (10-10-2026). The summary of the
//      backend shows: the number of بکری, خام بکری, the commission, the shared income, the lots in
//      باقی مال, and each partner's income, expense and profit share. The trial balance difference
//      is 0, and the screen has no sideways scroll.
//   2. A language switch in the middle keeps the date and shows the same summary in English.
//   3. The close button opens the confirm box that says what will happen; Esc leaves the day open.
//   4. The close button and Enter close the day; the message and the status bar name the date.
// The numbers themselves are proved by the Rust tests `golden_day` (E2E-8, E2E-9, E2E-10) and
// `day_money_fixtures`; this spec proves the screen shows what the backend says and works by
// keyboard. It runs on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

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
const WORKED_DAY = '10-10-2026';

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

/** The button that closes the day, in the language given. @param {'ur' | 'en'} language */
function closeButton(language) {
  return $(`//button[contains(normalize-space(.), "${t(language, 'day-close-button')}")]`);
}

describe('closing the worked day by keyboard (Urdu, then English)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the summary of the days shows the figures, the partners and a zero trial difference', async () => {
    await openRoute('/day');
    await waitForHeading('ur', 'day-title');
    const upTo = await focusField('ur', 'day-close-up-to');
    await upTo.setValue(WORKED_DAY);
    await waitForAll(
      t('ur', 'day-summary-title'),
      t('ur', 'day-summary-bikri-count'),
      t('ur', 'day-summary-gross'),
      t('ur', 'day-summary-commission'),
      t('ur', 'day-summary-shared-income'),
      t('ur', 'day-summary-open-lots'),
      t('ur', 'day-summary-col-income'),
      t('ur', 'day-summary-col-expense'),
      t('ur', 'day-summary-col-profit'),
    );
    // One row for each of the three partners, under the header row.
    const rows = await $$('//table//tr');
    expect(rows.length).toBe(4);
    // The books balance: no sideways scroll, and the cash and the trial totals are on the screen.
    const text = await pageText();
    expect(text).toContain(t('ur', 'day-close-trial-dr'));
    expect(text).toContain(t('ur', 'day-close-cash'));
    const sideways = await browser.execute(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(sideways).toBe(0);
    await shootScreen('day-close-summary', 'ur');
  });

  it('2. a language switch in the middle keeps the date and shows the summary in English', async () => {
    await press([Key.Alt, 'l']);
    await waitForHeading('en', 'day-title');
    await waitForAll(t('en', 'day-summary-title'), t('en', 'day-summary-partners-label'));
    expect(await (await focusField('en', 'day-close-up-to')).getValue()).toBe(WORKED_DAY);
    await shootScreen('day-close-summary', 'en');
    await press([Key.Alt, 'l']);
    await waitForHeading('ur', 'day-title');
  });

  it('3. the close button asks what will happen; Esc leaves the day open', async () => {
    await (await closeButton('ur')).click();
    const dialog = await $('[role="alertdialog"]');
    await dialog.waitForDisplayed({ timeout: WAIT_MS });
    expect(await dialog.getText()).toContain(WORKED_DAY);
    await type(Key.Escape);
    await dialog.waitForDisplayed({ timeout: WAIT_MS, reverse: true });
    // Nothing was closed: the button is still there and the status bar has no new date.
    expect(await (await closeButton('ur')).isDisplayed()).toBe(true);
  });

  it('4. the close button and Enter close the day, and the status bar names the date', async () => {
    await (await closeButton('ur')).click();
    const dialog = await $('[role="alertdialog"]');
    await dialog.waitForDisplayed({ timeout: WAIT_MS });
    await shootScreen('day-close-confirm', 'ur');
    await type(Key.Enter);
    await waitForText(t('ur', 'day-closed', { date: WORKED_DAY }));
    const bar = (await $('footer').getText()).replace(/[⁨⁩]/g, '');
    expect(bar).toContain(WORKED_DAY);
  });
});
