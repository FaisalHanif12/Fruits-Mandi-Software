// Plan 12 (D3-A3, PRD 12.1 to 12.5, E2E-26 screen part): Event 1 of the worked day (PRD 4.4) typed
// into the sale book with the keyboard only, in Urdu, then seen again in English.
//   1. Makes what Event 1 needs: the buyers اکرم ٹول ٹیکس and ارشد ملتانی, the item کیلا, the
//      packing پیٹی and سوات اڈا. (رحیم was made by spec 09.)
//   2. Opens the sale book: the cursor is on the truck number.
//   3. Types the truck, the زمیندار, the اڈا, the lot (50 نگ, freight 3,000, labour 250, munshiana
//      50, نقد 5,000, and فون 20 in the "more" panel of the lot, as PRD 4.4 has it), and the two
//      sale lines (اکرم 20 at 600, ارشد 30 at 580). Each sale line is three fields and three
//      Enters (NFR-USE-6).
//   4. E2E-26: the second line starts at 30 before its buyer is chosen, the rate is empty with
//      600 as grey text, and the working of the seller commission says 12,000 × 6 ÷ 100 = 720.
//   5. The numbers on the screen are the backend's, as in PRD 4.4: 12,880 and 18,684, net
//      19,316, gross 29,400, deductions 10,084, shared income 3,948, split 1,580 / 1,184 / 1,184.
//   6. Alt+L: the same numbers in English, with the names in English, and nothing lost.
//   7. The whole-sale view (PRD FR-12.1): the lines of every lot in one grid with a lot column, and
//      the numbers strip pinned to the bottom in the compact and normal windows.
//   8. Leaving the screen with this typed work asks first (PRD FR-7.9); "leave without saving"
//      throws the draft away, so the next spec starts clean.
// Nothing is saved here: saving the بکری is spec 13. The numbers themselves are proved by the Rust
// test `golden_day`; this spec proves the screen shows the same ones. It runs on Windows only
// (LOCAL-ONLY in the Mac phase: checked with `node --check`).

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
  startSignedIn,
  switchLanguage,
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
 * A cell of the lots grid or the sale lines grid, by the name of the grid, the row and the column.
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
 * Makes a buyer account the way spec 09 makes a seller.
 * @param {string} nameUr
 * @param {string} nameEn
 */
async function makeBuyer(nameUr, nameEn) {
  await openRoute('/index/accounts/new');
  await waitForHeading('ur', 'accounts-new-title');
  await choose('ur', 'accounts-field-type', 'accounts-type-buyer');
  await focusField('ur', 'accounts-field-name-ur');
  await type(`${nameUr}${Key.Enter}${nameEn}${Key.Enter}`);
  await type(Key.F10);
  await confirmSpellingIfAsked();
  await waitForText(t('ur', 'accounts-saved'));
}

/**
 * Makes an item, a packing or an adda from its form under /index/<path>/new.
 * @param {string} path
 * @param {string} nameUr
 * @param {string} nameEn
 * @param {() => Promise<void>} [afterNames] fills the fields after the two names
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

describe('the sale book, Event 1 by keyboard (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. makes the buyers, the item, the packing and the adda that Event 1 uses', async () => {
    await makeBuyer('اکرم ٹول ٹیکس', 'Akram Toll Tax');
    await makeBuyer('ارشد ملتانی', 'Arshad Multani');
    await makeMaster('items', 'کیلا', 'Banana', () =>
      choose('ur', 'masters-item-group', 'masters-item-group-fruit'),
    );
    await makeMaster('packings', 'پیٹی', 'Box');
    await makeMaster('addas', 'سوات اڈا', 'Swat Adda');
  });

  it('2. opens the sale book with the cursor on the truck number and the empty state showing', async () => {
    await openRoute('/bikri/book');
    await waitForHeading('ur', 'glossary-bikri-book');
    const truck = await focusField('ur', 'bikri-header-truck');
    await expect(truck).toBeFocused();
    await waitForText(t('ur', 'bikri-empty-title'));
    await shootScreen('bikri-book-empty', 'ur');
  });

  it('3. types the truck, the زمیندار and the اڈا, and Enter goes on to the lots', async () => {
    await type(`LES-1234${Key.Enter}`);
    await typeAndPick('rahim');
    await typeAndPick('swat');
    // The total freight stays empty here: the freight is typed on the lot.
    await type(Key.Enter);
    const seller = await cell(t('ur', 'bikri-lots-label'), 0, 'seller');
    await browser.waitUntil(async () => (await seller.getText()).includes('رحیم'), {
      timeout: LIST_WAIT_MS,
      timeoutMsg: 'The seller of the lot did not start as the زمیندار.',
    });
  });

  it('4. types the lot: item, packing, 50 نگ, freight, labour, munshiana, نقد', async () => {
    // The seller is filled by the screen and is an Enter stop.
    await type(Key.Enter);
    await typeAndPick('banana');
    await typeAndPick('box');
    // Labour is typed as 250, the amount (PRD 4.4: 5 for each نگ × 50), so no method is chosen.
    await type(`50${Key.Enter}3000${Key.Enter}250${Key.Enter}50${Key.Enter}5000${Key.Enter}`);
    const buyer = await cell(t('ur', 'bikri-sales-label'), 0, 'buyer');
    await buyer.waitForDisplayed({ timeout: LIST_WAIT_MS });
  });

  it('4b. types the phone charge 20 in the "more" panel of the lot', async () => {
    const toggle = await withText(t('ur', 'bikri-more-toggle', { lot: 1 }));
    await toggle.click();
    await focusField('ur', 'glossary-phone');
    await type('20');
    // Back to the sale lines: the cursor goes to the buyer of line 1.
    const buyer = await cell(t('ur', 'bikri-sales-label'), 0, 'buyer');
    await buyer.click();
  });

  it('5. types line 1: buyer, qty, rate, three fields and three Enters', async () => {
    await typeAndPick('akram');
    await type(`20${Key.Enter}600${Key.Enter}`);
    await waitForNumbers('12,880');
  });

  it('6. E2E-26: line 2 starts at 30, the rate is empty with 600 as grey text', async () => {
    const sales = t('ur', 'bikri-sales-label');
    // The new line is blank (no buyer, no rate) and already starts at what the lot has left.
    const qty = await cell(sales, 1, 'qty');
    await browser.waitUntil(async () => (await qty.getText()).trim() === '30', {
      timeout: LIST_WAIT_MS,
      timeoutMsg: 'The second line did not start at what the lot has left, 30.',
    });
    const rate = await cell(sales, 1, 'rate');
    await expect(rate).toHaveElementClass('cell-hint');
    expect((await rate.getText()).trim()).toBe('600');
    await shootScreen('bikri-book-rate-hint', 'ur');
    // The buyer, Enter takes the 30, then the rate is typed (the hint is never typed for him).
    await typeAndPick('arshad');
    await type(`${Key.Enter}580${Key.Enter}`);
    await waitForNumbers('18,684');
  });

  it('7. the numbers are the backend’s: PRD 4.4 Event 1', async () => {
    await waitForNumbers('29,400', '10,084', '19,316', '31,564', '3,948', '1,580', '1,184');
    const text = await pageText();
    // The cash note must not show: cash in hand is 50,000, and the lot pays 5,000.
    expect(text).not.toContain(t('ur', 'common-cash-confirm-title'));
    await shootScreen('bikri-book-event1', 'ur');
  });

  it('8. ⓘ on the seller commission of line 1 shows 12,000 × 6 ÷ 100 = 720', async () => {
    const commission = await cell(t('ur', 'bikri-sales-label'), 0, 'sellerComm');
    await commission.click();
    await browser.keys([Key.Ctrl, 'i']);
    await waitForText(
      t('ur', 'working-percent', { base: '12,000', bps: '6', result: '720' }).replace(/\s+/g, ' '),
    );
    await type(Key.Escape);
  });
});

describe('the whole-sale view and the numbers strip (Urdu)', () => {
  it('9. the lines of every lot are in one grid with the lot first', async () => {
    const group = t('ur', 'bikri-sales-view-label');
    const whole = await $(
      `//*[@role="group"][@aria-label="${group}"]//button[contains(normalize-space(.), "${t('ur', 'bikri-summary-whole')}")]`,
    );
    await whole.click();
    const sales = t('ur', 'bikri-sales-label');
    const lot = await cell(sales, 0, 'lot');
    await browser.waitUntil(async () => (await lot.getText()).includes('رحیم'), {
      timeout: LIST_WAIT_MS,
      timeoutMsg: 'The lot column of the whole-sale view did not name the seller.',
    });
    // Both lines and their totals are there, and nothing scrolls sideways.
    await waitForNumbers('12,880', '18,684');
    const overflow = await browser.execute(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth,
    );
    expect(overflow).toBe(false);
    await shootScreen('bikri-book-whole', 'ur');
  });

  it('10. the numbers strip holds the gross, deductions, net and buyers’ bills', async () => {
    const strip = await $(`[aria-label="${t('ur', 'bikri-strip-label')}"]`);
    // The strip is for the compact and normal windows; the wide window has the summary beside.
    if (!(await strip.isExisting())) return;
    const text = await strip.getText();
    for (const figure of ['29,400', '10,084', '19,316', '31,564']) expect(text).toContain(figure);
  });

  it('11. back to the lines of one lot', async () => {
    const group = t('ur', 'bikri-sales-view-label');
    const one = await $(
      `//*[@role="group"][@aria-label="${group}"]//button[contains(normalize-space(.), "${t('ur', 'bikri-summary-lot', { lot: 1 })}")]`,
    );
    await one.click();
    const buyer = await cell(t('ur', 'bikri-sales-label'), 0, 'buyer');
    await buyer.waitForDisplayed({ timeout: LIST_WAIT_MS });
  });
});

describe('the same entry in English', () => {
  it('Alt+L turns the screen to English with the same numbers and nothing lost', async () => {
    await switchLanguage('en');
    await waitForNumbers('12,880', '18,684', '19,316', '29,400', '3,948');
    const text = await pageText();
    expect(text).toContain('Akram Toll Tax');
    expect(text).toContain('Arshad Multani');
    expect(text).toContain('Banana');
    await shootScreen('bikri-book-event1', 'en');
  });

  it('turns back to Urdu with the same numbers', async () => {
    await switchLanguage('ur');
    await waitForNumbers('12,880', '18,684', '19,316');
    expect(await pageText()).toContain('اکرم ٹول ٹیکس');
  });
});

describe('leaving with typed work (Urdu)', () => {
  it('12. asks "save changes?"; leaving without saving throws the draft away (PRD FR-7.9)', async () => {
    await switchLanguage('ur');
    await openRoute('/');
    const prompt = await $('[role="alertdialog"]');
    await prompt.waitForDisplayed({ timeout: LIST_WAIT_MS });
    const leave = await $(
      `//*[@role="alertdialog"]//button[contains(normalize-space(.), "${t('ur', 'layout-save-leave')}")]`,
    );
    await leave.click();
    await prompt.waitForDisplayed({ timeout: LIST_WAIT_MS, reverse: true });
    await waitForText(t('ur', 'dashboard-greeting', { name: OWNER.displayName }));
  });
});
