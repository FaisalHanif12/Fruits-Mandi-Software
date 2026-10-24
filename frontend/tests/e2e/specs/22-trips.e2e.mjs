// E2E-34 (D4-A23, D4-A24, PRD 12.10, FR-5.2, FR-7.67, FR-12.29, FR-12.30): a market trip, typed with the
// keyboard, in Urdu and then in English.
//   1. مارکیٹ سفر is in the main bar after روکڑ and opens the list. The list is empty and says what
//      to do.
//   2. F2 opens the new trip. The shares come first (الف 40, ب 30, ج 30, total 100%), then the
//      source of each partner (from the shop's cash), then the details. F10 makes the trip.
//   3. A purchase of 20 پیٹی کیلا at 500 with freight 400 shows each partner's part before the save:
//      4,160 / 3,120 / 3,120 (E2E-34). F10 saves it with its number خ-1.
//   4. A language switch in the middle of the next entry keeps what was typed (E2E-22 style).
//   5. The type list has the three types; a trip cost and a sale in another shop are reachable.
//   6. The history lists the entry, page by page.
// The numbers themselves are proved by the Rust test `golden_day` (E2E-34, E2E-34b, E2E-41); this
// spec proves the screens send what was typed and show what the backend sends. It runs on Windows
// only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
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

/** A button of the main bar by its words (the name is its aria-label). @param {string} id */
function navItem(id) {
  return $(`//nav//button[@aria-label="${t('ur', id)}"]`);
}

describe('E2E-34: a market trip (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the main bar opens the list, which is empty and says what to do', async () => {
    const labels = await browser.execute(() =>
      Array.from(document.querySelectorAll('nav [data-nav-item]')).map((item) =>
        item.getAttribute('aria-label'),
      ),
    );
    const after = labels.indexOf(t('ur', 'layout-nav-rokar'));
    // باغات comes right after روکڑ and مارکیٹ سفر right after باغات (PRD FR-5.2).
    expect(labels[after + 1]).toBe(t('ur', 'layout-nav-gardens'));
    expect(labels[after + 2]).toBe(t('ur', 'layout-nav-trips'));
    await (await navItem('layout-nav-trips')).click();
    await waitForHeading('ur', 'trips-title');
    await waitForText(t('ur', 'trips-list-empty'));
    await shootScreen('trips-empty', 'ur');
  });

  it('2. F2 opens the new trip with the shares first; F10 makes it and opens it', async () => {
    await type(Key.F2);
    await waitForHeading('ur', 'trips-new-title');
    await waitForText(t('ur', 'partners-total-ok'));
    await shootScreen('trips-new', 'ur');
    await type(Key.F10);
    await waitForText(t('ur', 'trips-created', { number: t('ur', 'trips-number', { number: 1 }) }));
    await waitForText(t('ur', 'trips-status-open'));
    await shootScreen('trips-one-empty', 'ur');
  });

  it('3. a purchase shows each partner’s part before the save, then saves', async () => {
    await pick('ur', 'glossary-qism-maal', 'banana', 'کیلا');
    await pick('ur', 'glossary-packing', 'box', 'پیٹی');
    await focusField('ur', 'purchases-field-qty');
    await type('20');
    await focusField('ur', 'purchases-field-rate');
    await type('500');
    await focusField('ur', 'glossary-kiraya');
    await type('400');
    // E2E-34: value 10,000, paid now 10,400, parts 4,160 / 3,120 / 3,120.
    await waitForNumbers('10,000', '10,400', '4,160', '3,120');
    await shootScreen('trips-purchase-preview', 'ur');
    await type(Key.F10);
    await waitForText(
      t('ur', 'trips-saved-purchase', { number: t('ur', 'purchases-number', { number: 1 }) }),
    );
    await waitForText(t('ur', 'trips-entries-count', { shown: 1, total: 1 }));
  });

  it('4. a language switch in the middle of an entry keeps what was typed', async () => {
    await focusField('ur', 'purchases-field-qty');
    await type('7');
    await switchLanguage('en');
    const qty = await focusField('en', 'purchases-field-qty');
    expect(await qty.getValue()).toBe('7');
    await shootScreen('trips-one-entry', 'en');
    await switchLanguage('ur');
  });

  it('5. the type list reaches a trip cost and a sale in another shop, and keeps the day', async () => {
    const day = await focusField('ur', 'trips-field-date');
    const typed = await day.getValue();
    await (await buttonNamed('trips-type-cost')).click();
    await waitForText(t('ur', 'trips-cost-amount'));
    const same = await focusField('ur', 'trips-field-date');
    expect(await same.getValue()).toBe(typed);
    await (await buttonNamed('trips-type-other-sale')).click();
    await waitForText(t('ur', 'trips-sale-amount'));
    await shootScreen('trips-other-sale', 'ur');
  });

  it('6. the list shows the trip with the totals the backend sent', async () => {
    await type(Key.Escape);
    await waitForHeading('ur', 'trips-title');
    const text = await pageText();
    expect(text).toContain(t('ur', 'trips-number', { number: 1 }));
    expect(text).toContain(t('ur', 'trips-list-count', { shown: 1, total: 1 }));
  });
});

describe('a market trip in English', () => {
  it('shows the same screens left to right and keeps the numbers', async () => {
    await switchLanguage('en');
    await waitForText(t('en', 'trips-title'));
    await shootScreen('trips-list', 'en');
    await browser.waitUntil(async () => (await pageText()).includes('TRP-1'), {
      timeout: WAIT_MS,
      timeoutMsg: 'The English list never showed TRP-1.',
    });
    await switchLanguage('ur');
  });
});
