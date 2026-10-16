// Plan 13 (D3-A5 to D3-A9 screen part, D3-A12, D3-A14, PRD 12.6, 13): Events 1 and 2 of the worked
// day (PRD 4.4, 4.5) typed with the keyboard only and SAVED with F10, in Urdu, then looked at again
// on the screens they change.
//   1. Makes what Event 2 needs: the seller کریم (030110002), the item سیب and لاہور اڈا. (The rest
//      was made by specs 09 and 12.)
//   2. Event 1: truck, زمیندار, اڈا, the lot, two sale lines, F10. The message names بکری 1 and a
//      new empty بکری is ready with the cursor on the truck number.
//   3. Opens the saved بکری 1 from the "last saved" link: the stored numbers of E2E-1 to E2E-4 and
//      nothing can be typed in it.
//   4. Event 2: a lot from کریم sold to شاہد, a one-time buyer with his name and phone, 10 at 1,200;
//      F10. The message names بکری 2.
//   5. آج کی بکری lists both. The پختہ کھاتہ register shows شاہد, paid. اکرم's balance shows his
//      12,880 on the accounts list (PRD 6.2).
// The numbers themselves are proved by the Rust test `golden_day` (E2E-1 to E2E-5); this spec proves
// the screens send what was typed and show what is stored. It runs on Windows only (LOCAL-ONLY in
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
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
  withText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const LIST_WAIT_MS = 10_000;
const NUMBER_WAIT_MS = 15_000;

/**
 * Types the text of a picker, waits for its list to answer, and takes the first match.
 * @param {string} text
 */
async function typeAndPick(text) {
  await type(text);
  const first = await $('[role="option"]');
  await first.waitForDisplayed({ timeout: LIST_WAIT_MS });
  await type(Key.Enter);
}

/**
 * A cell of a grid by the name of the grid, the row and the column.
 * @param {string} gridLabel
 * @param {number} rowIndex
 * @param {string} column
 */
function cell(gridLabel, rowIndex, column) {
  return $(
    `//*[@role="group"][@aria-label="${gridLabel}"]//div[@role="row"][@row-index="${String(rowIndex)}"]//div[@col-id="${column}"]`,
  );
}

/**
 * Waits until the page says every one of these numbers.
 * @param {...string} numbers
 */
async function waitForNumbers(...numbers) {
  await browser.waitUntil(
    async () => {
      const text = await pageText();
      return numbers.every((number) => text.includes(number));
    },
    { timeout: NUMBER_WAIT_MS, timeoutMsg: `The page never showed: ${numbers.join(', ')}` },
  );
}

/**
 * Waits for the message after a save, with its number (PRD FR-7.7).
 * @param {number} number
 */
async function waitForSaved(number) {
  await waitForText(t('ur', 'bikri-saved-toast', { number }));
}

/**
 * Makes a seller account: the second one gets 030110002.
 * @param {string} nameUr
 * @param {string} nameEn
 */
async function makeSeller(nameUr, nameEn) {
  await openRoute('/index/accounts/new');
  await waitForHeading('ur', 'accounts-new-title');
  await choose('ur', 'accounts-field-type', 'accounts-type-seller');
  await focusField('ur', 'accounts-field-name-ur');
  await type(`${nameUr}${Key.Enter}${nameEn}${Key.Enter}`);
  await type(Key.F10);
  await confirmSpellingIfAsked();
  await waitForText(t('ur', 'accounts-saved'));
}

/**
 * Makes an item or an adda from its form under /index/<path>/new.
 * @param {string} path
 * @param {string} nameUr
 * @param {string} nameEn
 * @param {() => Promise<void>} [afterNames]
 */
async function makeMaster(path, nameUr, nameEn, afterNames) {
  await openRoute(`/index/${path}/new`);
  await focusField('ur', 'masters-field-name-ur');
  await type(`${nameUr}${Key.Enter}${nameEn}${Key.Enter}`);
  if (afterNames !== undefined) await afterNames();
  await type(Key.F10);
  await confirmSpellingIfAsked();
  await waitForText(t('ur', 'masters-saved'));
}

/**
 * Types the phone charge of the lot in its "more" panel, then goes back to the sale lines.
 * @param {number} lot
 * @param {string} amount
 */
async function typePhoneCharge(lot, amount) {
  const toggle = await withText(t('ur', 'bikri-more-toggle', { lot }));
  await toggle.click();
  await focusField('ur', 'glossary-phone');
  await type(amount);
  const buyer = await cell(t('ur', 'bikri-sales-label'), 0, 'buyer');
  await buyer.click();
}

describe('saving the worked day, Events 1 and 2 (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. makes the seller کریم, the item سیب and لاہور اڈا that Event 2 uses', async () => {
    await makeSeller('کریم', 'Karim');
    await makeMaster('items', 'سیب', 'Apple', () =>
      choose('ur', 'masters-item-group', 'masters-item-group-fruit'),
    );
    await makeMaster('addas', 'لاہور اڈا', 'Lahore Adda');
  });

  it('2. Event 1: types the whole sale with the keyboard and saves it with F10', async () => {
    await openRoute('/bikri/book');
    await waitForHeading('ur', 'glossary-bikri-book');
    await focusField('ur', 'bikri-header-truck');
    await type(`LES-1234${Key.Enter}`);
    await typeAndPick('rahim');
    await typeAndPick('swat');
    await type(Key.Enter);
    // The seller is the زمیندار, filled by the screen and an Enter stop; then item and packing.
    await type(Key.Enter);
    await typeAndPick('banana');
    await typeAndPick('box');
    await type(`50${Key.Enter}3000${Key.Enter}250${Key.Enter}50${Key.Enter}5000${Key.Enter}`);
    await typePhoneCharge(1, '20');
    await typeAndPick('akram');
    await type(`20${Key.Enter}600${Key.Enter}`);
    await typeAndPick('arshad');
    await type(`${Key.Enter}580${Key.Enter}`);
    await waitForNumbers('29,400', '10,084', '19,316', '31,564', '3,948');
    await shootScreen('bikri-save-event1-typed', 'ur');
    await type(Key.F10);
    await waitForSaved(1);
  });

  it('3. a new empty بکری is ready with the cursor on the truck number', async () => {
    const truck = await focusField('ur', 'bikri-header-truck');
    await expect(truck).toBeFocused();
    expect(await truck.getValue()).toBe('');
    await waitForText(t('ur', 'bikri-last-saved', { number: 1 }));
  });

  it('4. the saved بکری 1 opens read-only with the stored numbers of E2E-1 to E2E-4', async () => {
    const open = await $(
      `//button[contains(normalize-space(.), "${t('ur', 'bikri-truck-open')}")]`,
    );
    await open.click();
    await waitForText(t('ur', 'bikri-saved-readonly'));
    await waitForNumbers('29,400', '10,084', '19,316', '12,880', '18,684', '1,580', '1,184');
    // Nothing in the screen can be typed in.
    const fields = await $$('main input');
    expect(fields.length).toBe(0);
    await shootScreen('bikri-saved-event1', 'ur');
  });

  it('5. Event 2: a lot from کریم sold to شاہد, a one-time buyer, saved with F10', async () => {
    await openRoute('/bikri/book');
    await waitForHeading('ur', 'glossary-bikri-book');
    await focusField('ur', 'bikri-header-truck');
    await type(`LHR-0077${Key.Enter}`);
    await typeAndPick('karim');
    await typeAndPick('lahore');
    await type(Key.Enter);
    await type(Key.Enter);
    await typeAndPick('apple');
    await typeAndPick('box');
    await type(`10${Key.Enter}1000${Key.Enter}50${Key.Enter}50${Key.Enter}${Key.Enter}`);
    await typePhoneCharge(1, '20');
    // The one-time buyer account: Enter goes to his name, then the other name, then his phone.
    await type(`03049999${Key.Enter}`);
    await type(`shahid${Key.Enter}`);
    await confirmSpellingIfAsked();
    await type(`${Key.Enter}0300-0000000${Key.Enter}`);
    await type(`10${Key.Enter}1200${Key.Enter}`);
    await waitForNumbers('12,800', '10,160', '1,540', '616', '462');
    await shootScreen('bikri-save-event2-typed', 'ur');
    await type(Key.F10);
    await waitForSaved(2);
  });

  it('6. آج کی بکری lists both بکری with their stored sums', async () => {
    await openRoute('/bikri/today');
    await waitForHeading('ur', 'layout-module-today');
    await waitForNumbers('29,400', '19,316', '12,000', '10,160');
    await shootScreen('bikri-today-list', 'ur');
  });

  it('7. the پختہ کھاتہ register shows شاہد, paid', async () => {
    await openRoute('/bikri/pukhta');
    await waitForHeading('ur', 'glossary-pukhta-khata');
    await waitForText('شاہد');
    const text = await pageText();
    expect(text).toContain('12,800');
    expect(text).toContain(t('ur', 'pukhta-state-paid'));
    await shootScreen('bikri-pukhta-register', 'ur');
  });

  it('8. اکرم’s balance on the accounts list is the 12,880 of his bill (PRD 6.2)', async () => {
    await openRoute('/index/accounts');
    await waitForText('اکرم ٹول ٹیکس');
    await waitForNumbers('12,880');
  });

  it('9. the global search finds بکری 1 by its number and by the start of its truck', async () => {
    await browser.keys([Key.Ctrl, 'k']);
    await type('LES');
    const found = await $('[role="option"]');
    await found.waitForDisplayed({ timeout: LIST_WAIT_MS });
    expect(await found.getText()).toContain('LES-1234');
    await type(Key.Enter);
    await waitForText(t('ur', 'bikri-saved-readonly'));
  });
});
