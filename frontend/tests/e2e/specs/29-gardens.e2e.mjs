// E2E-40b (D4R-A4 to A6, PRD 12.11, FR-5.2, FR-7.67, FR-12.29, FR-12.30, FR-12.34 to FR-12.38, FR-6.7):
// a garden, typed with the keyboard, in Urdu and then in English. It is the loss story of the PRD
// (E2E-40b): the garden of الف 60% from the shop and ب 40% from his own pocket, rent 50,000, first
// installment 30,000, a second installment 20,000, a cost 5,000, and only the 10,000 of a sale in
// another shop coming in; then the garden is closed.
//   1. باغات is in the main bar after روکڑ, before مارکیٹ سفر. It opens the list, which is empty and
//      says what to do.
//   2. F2 opens the new garden. The shares come first (الف 60, ب 40, ج left empty), then the source
//      of each partner (ب from his pocket), then the details (both names, the main fruit, the rent
//      and the first installment). Its parts show before the save: 18,000 from the shop and 12,000
//      from a pocket. F10 makes the garden and opens it.
//   3. A language switch in the middle of an entry keeps what was typed (E2E-22 style).
//   4. An installment, a garden cost and a sale in another shop are typed and saved with F10. The
//      running balance reads -45,000 (money in 10,000, everything paid 55,000).
//   5. The closing card shows the result of the closing (E2E-40b: 10,000 goes to الف 6,000 and ب 4,000;
//      the result is -45,000). The close button opens the confirm box with Back focused: Enter leaves
//      the garden open; Tab to the other button and Enter closes it. The form is replaced by a note.
//   6. The undo of the closing asks for a reason and opens the garden again.
// The numbers themselves are proved by the Rust test `golden_day` (E2E-40, E2E-40b) and the fixtures
// of plan 17c; this spec proves the screens send what was typed and show what the backend sends. It
// runs on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  confirmSpellingIfAsked,
  focusField,
  pageText,
  pick,
  startSignedIn,
  switchLanguage,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import { buttonNamed, waitForNumbers } from '../support/bikri-sale.mjs';

const WAIT_MS = 15_000;
const GARDEN_ONE = () => t('ur', 'gardens-number', { number: 1 });

/** A button of the main bar by its words (the name is its aria-label). @param {string} id */
function navItem(id) {
  return $(`//nav//button[@aria-label="${t('ur', id)}"]`);
}

/**
 * The box of one partner's share, found by the words of its label.
 * @param {'ur' | 'en'} language
 * @param {string} name the partner's name as shown in that language
 */
async function shareBox(language, name) {
  const wanted = t(language, 'ventures-share-of', { name });
  const id = await browser.execute((text) => {
    const clean = (/** @type {string | null} */ value) =>
      (value ?? '').replace(/[⁨⁩]/g, '').replace(/\s+/g, ' ').trim();
    const label = Array.from(document.querySelectorAll('label')).find(
      (candidate) => clean(candidate.textContent) === text,
    );
    return label?.htmlFor ?? null;
  }, wanted);
  if (id === null || id === '') throw new Error(`No share box labelled "${wanted}"`);
  const box = await $(`[id="${id}"]`);
  await box.waitForDisplayed({ timeout: WAIT_MS });
  return box;
}

/** The type list of the garden screen: a button by its words. @param {string} id */
function typeButton(id) {
  const group = `//*[@role="group"][@aria-label="${t('ur', 'gardens-type-label')}"]`;
  return $(`${group}//button[contains(normalize-space(.), "${t('ur', id)}")]`);
}

/**
 * The button of one partner's money source (from the shop, or his own pocket).
 * @param {'ur' | 'en'} language
 * @param {string} name the partner's name as shown in that language
 * @param {'ventures-source-shop' | 'ventures-source-pocket'} optionId
 */
async function sourceButton(language, name, optionId) {
  const wanted = t(language, 'ventures-source-of', { name });
  const marker = await browser.execute((text) => {
    const clean = (/** @type {string | null} */ value) =>
      (value ?? '').replace(/[⁨⁩]/g, '').replace(/\s+/g, ' ').trim();
    const group = Array.from(document.querySelectorAll('[role="group"]')).find(
      (candidate) => clean(candidate.getAttribute('aria-label')) === text,
    );
    group?.setAttribute('data-e2e-sources', 'wanted');
    return group !== undefined;
  }, wanted);
  if (!marker) throw new Error(`No money source group labelled "${wanted}"`);
  return $(
    `//*[@data-e2e-sources="wanted"]//button[contains(normalize-space(.), "${t(language, optionId)}")]`,
  );
}

describe('E2E-40b: a garden (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the main bar has باغات after روکڑ and before مارکیٹ سفر, and the list is empty and says what to do', async () => {
    const labels = await browser.execute(() =>
      Array.from(document.querySelectorAll('nav [data-nav-item]')).map((item) =>
        item.getAttribute('aria-label'),
      ),
    );
    const after = labels.indexOf(t('ur', 'layout-nav-rokar'));
    expect(labels[after + 1]).toBe(t('ur', 'layout-nav-gardens'));
    expect(labels[after + 2]).toBe(t('ur', 'layout-nav-trips'));
    await (await navItem('layout-nav-gardens')).click();
    await waitForHeading('ur', 'gardens-title');
    await waitForText(t('ur', 'gardens-list-empty'));
    await shootScreen('gardens-empty', 'ur');
  });

  it('2. F2 opens the new garden: shares first, then sources, then details; the parts show before F10', async () => {
    await type(Key.F2);
    await waitForHeading('ur', 'gardens-new-title');
    await waitForText(t('ur', 'partners-total-ok'));
    // الف 60%, ب 40%, ج not in it.
    await (await shareBox('ur', 'الف')).setValue('60');
    await (await shareBox('ur', 'ب')).setValue('40');
    await (await shareBox('ur', 'ج')).setValue('');
    // ب pays from his own pocket (FR-12.29, step 2 of the questions).
    await (await sourceButton('ur', 'ب', 'ventures-source-pocket')).click();
    await waitForText(t('ur', 'partners-total-ok'));
    await (await focusField('ur', 'gardens-field-name-ur')).setValue('شاہ پور باغ');
    await (await focusField('ur', 'gardens-field-name-en')).setValue('Shahpur Garden');
    await pick('ur', 'glossary-qism-maal', 'banana', 'کیلا');
    await focusField('ur', 'gardens-field-rent');
    await type('50000');
    await focusField('ur', 'gardens-field-paid-now');
    await type('30000');
    // PRD FR-12.30 and C23: 30,000 at 60 / 40 is 18,000 from the shop and 12,000 from a pocket.
    await waitForNumbers('18,000', '12,000');
    await waitForText(t('ur', 'gardens-new-sources-help'));
    // The reason of the first installment is filled in for him, visible and changeable.
    const reason = await focusField('ur', 'gardens-field-paid-now-reason');
    expect(await reason.getValue()).toBe(t('ur', 'gardens-first-installment-reason'));
    await shootScreen('gardens-new', 'ur');
    await type(Key.F10);
    await confirmSpellingIfAsked();
    await waitForText(t('ur', 'gardens-created', { number: GARDEN_ONE() }));
    await waitForText(t('ur', 'gardens-status-open'));
    await shootScreen('gardens-one-first-installment', 'ur');
  });

  it('3. a language switch in the middle of an entry keeps what was typed', async () => {
    await focusField('ur', 'gardens-installment-amount');
    await type('20000');
    await switchLanguage('en');
    const amount = await focusField('en', 'gardens-installment-amount');
    expect((await amount.getValue()).replace(/,/g, '')).toBe('20000');
    await shootScreen('gardens-one-entry', 'en');
    await switchLanguage('ur');
  });

  it('4. an installment, a cost and a sale in another shop are saved with F10; the balance is -45,000', async () => {
    // The second installment: 20,000 (typed in step 3) and why.
    await (await focusField('ur', 'gardens-payment-reason')).setValue('دوسری قسط');
    // 20,000 at 60 / 40 is 12,000 from the shop and 8,000 from a pocket.
    await waitForNumbers('12,000', '8,000');
    await type(Key.F10);
    await waitForText(t('ur', 'gardens-saved-installment', { number: 2 }));
    // A garden cost of 5,000: the type list keeps the day, and the sources stay.
    await (await typeButton('gardens-type-cost')).click();
    await waitForText(t('ur', 'gardens-cost-amount'));
    await focusField('ur', 'gardens-cost-amount');
    await type('5000');
    await (await focusField('ur', 'gardens-payment-reason')).setValue('مزدوری');
    await waitForNumbers('3,000', '2,000');
    await type(Key.F10);
    await waitForText(t('ur', 'gardens-saved-cost', { number: 1 }));
    // A sale in another shop: 20 نگ for 10,000 comes into روکڑ and into the garden.
    await (await typeButton('gardens-type-other-sale')).click();
    await waitForText(t('ur', 'gardens-sale-amount'));
    await (await focusField('ur', 'gardens-sale-shop')).setValue('حاجی');
    await focusField('ur', 'gardens-sale-qty');
    await type('20');
    await focusField('ur', 'gardens-sale-amount');
    await type('10000');
    await waitForText(t('ur', 'gardens-numbers-balance-after'));
    await type(Key.F10);
    await waitForText(t('ur', 'gardens-saved-other-sale', { number: 1 }));
    // Money in 10,000, everything paid 55,000: the running balance is -45,000.
    await waitForNumbers('-45,000');
    await shootScreen('gardens-one-three-entries', 'ur');
  });

  it('5. the closing card shows the result; the confirm box has Back focused; Tab and Enter close it', async () => {
    const closeButton = await buttonNamed('gardens-close-button');
    await closeButton.scrollIntoView();
    // E2E-40b: 10,000 goes to الف 6,000 and ب 4,000; the result is -45,000.
    await waitForNumbers('6,000', '4,000', '-45,000');
    await shootScreen('gardens-close-card', 'ur');
    await closeButton.click();
    const dialog = await $('[role="alertdialog"]');
    await dialog.waitForDisplayed({ timeout: WAIT_MS });
    await shootScreen('gardens-close-confirm', 'ur');
    // Back has the focus: Enter out of habit leaves the garden open (PRD FR-7.8).
    await type(Key.Enter);
    await dialog.waitForDisplayed({ timeout: WAIT_MS, reverse: true });
    expect(await (await buttonNamed('gardens-close-button')).isDisplayed()).toBe(true);
    // Again: Tab moves to the other button and Enter closes the garden.
    await (await buttonNamed('gardens-close-button')).click();
    await dialog.waitForDisplayed({ timeout: WAIT_MS });
    await type(Key.Tab);
    await type(Key.Enter);
    await waitForText(t('ur', 'gardens-closed-toast', { number: GARDEN_ONE() }));
    await waitForText(t('ur', 'gardens-reopen-button'));
    // A closed garden takes no entry: the form is gone.
    const text = await pageText();
    expect(text).not.toContain(t('ur', 'gardens-type-label'));
    // الف got 6,000 back for the 33,000 he paid: his net is -27,000; ب got 4,000 for 22,000: +4,000.
    await waitForNumbers('-27,000');
    await shootScreen('gardens-closed', 'ur');
  });

  it('6. the undo of the closing asks for a reason and opens the garden again', async () => {
    await (await buttonNamed('gardens-reopen-button')).click();
    const dialog = await $('[role="alertdialog"]');
    await dialog.waitForDisplayed({ timeout: WAIT_MS });
    await type('غلط دن');
    await type(Key.Enter);
    await waitForText(t('ur', 'gardens-reopened-toast', { number: GARDEN_ONE() }));
    await waitForText(t('ur', 'gardens-type-label'));
    await shootScreen('gardens-reopened', 'ur');
  });
});

describe('a garden in English', () => {
  it('shows the same screens left to right and keeps the numbers', async () => {
    await switchLanguage('en');
    await waitForText(t('en', 'gardens-title'));
    await browser.waitUntil(async () => (await pageText()).includes('GRD-1'), {
      timeout: WAIT_MS,
      timeoutMsg: 'The English garden screen never showed GRD-1.',
    });
    await shootScreen('gardens-one', 'en');
    await type(Key.Escape);
    await waitForHeading('en', 'gardens-title');
    await waitForText(t('en', 'common-list-count', { shown: 1, total: 1 }));
    await shootScreen('gardens-list', 'en');
    await switchLanguage('ur');
  });
});
