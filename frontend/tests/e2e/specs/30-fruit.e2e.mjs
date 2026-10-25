// E2E-42 (D4R-A7, PRD 16.7, FR-16.10 to FR-16.12, FR-7.53, C27): fruit taken, typed with the keyboard,
// in Urdu and then in English. It uses fruit from another shop, because the worked day has sold
// every lot (nothing is left in باقی مال to take from); the lot source is proved by the component
// tests and by the Rust test `golden_day` (E2E-42). The numbers are proved there too; this spec
// proves the screens send what was typed and show what the backend sends.
//   1. The tab پھل is next to خرچہ on the خرچہ screen (Alt+K) and opens the fruit screen, which has
//      no entries and says what to do.
//   2. For the guests, from another shop (agent کریم, کیلا, پیٹی), 3 at 1,000: the screen shows the
//      value 3,000 and the equal parts of the three partners (1,000 each) before F10; F10 saves.
//   3. A language switch in the middle of the next entry (partner الف, 2 at 1,400) keeps what was
//      typed; F10 saves it in English.
//   4. The entry is edited with a reason, then cancelled with a reason: the row stays in the list
//      marked cancelled.
// It runs on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, expect } from '@wdio/globals';
import { OWNER, PARTNERS } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  choose,
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
import { waitForNumbers } from '../support/bikri-sale.mjs';

const WAIT_MS = 15_000;
const FRUIT_ONE = () => t('ur', 'fruit-number', { number: 1 });

/** The button of the source choice by its words. @param {'ur' | 'en'} language @param {string} id */
function sourceButton(language, id) {
  const group = `//*[@role="group"][@aria-label="${t(language, 'fruit-field-source')}"]`;
  return $(`${group}//button[contains(normalize-space(.), "${t(language, id)}")]`);
}

/** A cell of the list under the form: the first cell of the row with this id. @param {number} id */
function listRow(id) {
  return $(`.ag-row[row-id="${String(id)}"] .ag-cell`);
}

describe('E2E-42: fruit taken (Urdu, then English)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the tab پھل is next to خرچہ and opens a screen with no entries that says what to do', async () => {
    await press([Key.Alt, 'k']);
    await waitForHeading('ur', 'cashbook-expense-title');
    const strip = await $(`//nav[@aria-label="${t('ur', 'cashbook-tabs-label')}"]`);
    await strip.waitForDisplayed({ timeout: WAIT_MS });
    const tab = await strip.$(
      `.//button[contains(normalize-space(.), "${t('ur', 'cashbook-tab-fruit')}")]`,
    );
    await tab.click();
    await waitForHeading('ur', 'fruit-title');
    await waitForText(t('ur', 'fruit-list-empty'));
    await shootScreen('fruit-empty', 'ur');
  });

  it('2. for the guests, from another shop: the value and the equal parts show before F10; F10 saves', async () => {
    await choose('ur', 'fruit-field-for', 'fruit-for-guests');
    await (await sourceButton('ur', 'fruit-source-other')).click();
    await pick('ur', 'fruit-field-agent', 'karim', 'کریم');
    await pick('ur', 'glossary-qism-maal', 'banana', 'کیلا');
    await pick('ur', 'glossary-packing', 'box', 'پیٹی');
    await focusField('ur', 'fruit-field-qty');
    await type('3');
    await focusField('ur', 'fruit-field-rate');
    await type('1000');
    // PRD C27: 3,000 shared equally between the three partners, 1,000 each.
    await waitForNumbers('3,000', '1,000');
    await waitForText(t('ur', 'fruit-parts-title-guests'));
    await waitForText(t('ur', 'fruit-parts-note'));
    await shootScreen('fruit-guests', 'ur');
    await type(Key.F10);
    await waitForText(t('ur', 'fruit-saved', { number: FRUIT_ONE() }));
    // The list shows the entry with the backend's value.
    await waitForNumbers('3,000');
  });

  it('3. a language switch in the middle of the next entry loses nothing; F10 saves it in English', async () => {
    // The next entry keeps the guests and the source; this one is for partner الف.
    const forBox = await focusField('ur', 'fruit-field-for');
    await forBox.click();
    const partner = await $(
      `//*[@role="option"][contains(normalize-space(.), "${PARTNERS[0]?.nameUr ?? ''}")]`,
    );
    await partner.waitForDisplayed({ timeout: WAIT_MS });
    await partner.click();
    await pick('ur', 'fruit-field-agent', 'karim', 'کریم');
    await pick('ur', 'glossary-qism-maal', 'banana', 'کیلا');
    await pick('ur', 'glossary-packing', 'box', 'پیٹی');
    await focusField('ur', 'fruit-field-qty');
    await type('2');
    await focusField('ur', 'fruit-field-rate');
    await type('1400');
    await press([Key.Alt, 'l']);
    await waitForHeading('en', 'fruit-title');
    expect(await (await focusField('en', 'fruit-field-qty')).getValue()).toBe('2');
    expect((await (await focusField('en', 'fruit-field-rate')).getValue()).replace(/,/g, '')).toBe(
      '1400',
    );
    await shootScreen('fruit-entry', 'en');
    await type(Key.F10);
    await waitForText(t('en', 'fruit-saved', { number: t('en', 'fruit-number', { number: 2 }) }));
  });

  it('4. an entry is edited with a reason and then cancelled with a reason; it stays in the list as cancelled', async () => {
    await (await listRow(2)).click();
    await (await $(`//button[contains(normalize-space(.), "${t('en', 'fruit-edit')}")]`)).click();
    await waitForText(t('en', 'fruit-editing', { number: t('en', 'fruit-number', { number: 2 }) }));
    await focusField('en', 'fruit-field-qty');
    await type('1');
    await (await focusField('en', 'fruit-field-reason')).setValue('wrong count');
    await type(Key.F10);
    await waitForText(t('en', 'fruit-saved', { number: t('en', 'fruit-number', { number: 2 }) }));
    await (await listRow(2)).click();
    await (await $(`//button[contains(normalize-space(.), "${t('en', 'fruit-cancel')}")]`)).click();
    const dialog = await $('[role="alertdialog"]');
    await dialog.waitForDisplayed({ timeout: WAIT_MS });
    await type('mistake');
    await type(Key.Enter);
    await waitForText(
      t('en', 'fruit-cancelled-toast', { number: t('en', 'fruit-number', { number: 2 }) }),
    );
    const cancelledWords = t('en', 'fruit-for-cancelled', { name: PARTNERS[0]?.nameEn ?? '' });
    await waitForText(cancelledWords);
    expect(await pageText()).toContain(cancelledWords);
    await shootScreen('fruit-cancelled', 'en');
    await press([Key.Alt, 'l']);
    await waitForHeading('ur', 'fruit-title');
  });
});
