// Plan 17 T4 (D4-A34, PRD 16.4, FR-6.2, FR-6.5, FR-13.5, FR-6.7, E2E-39 split part): the جنرل grid
// worked with the keyboard, in Urdu, after spec 26.
//   1. The grid starts with two empty lines and Save is off.
//   2. Electricity 1,000 on the بنام side and روکڑ 900 on the جمع side do not balance: the backend's
//      totals and the difference show, Save stays off, and F10 says why in words.
//   3. The cash line is changed to 1,000: the entry balances, the split between the partners shows,
//      Save opens, and F10 saves it with its G- number.
//   4. A language switch in the middle of an entry keeps the lines, the cursor and the totals (F39).
// The numbers are proved by the Rust test `golden_day` (general entries); this spec proves the grid
// sends what was typed and shows what the backend says. It runs on Windows only (LOCAL-ONLY in the
// Mac phase: checked with `node --check`).

import { browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  openRoute,
  pageText,
  press,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import { cell, typeAndPick, waitForNumbers } from '../support/bikri-sale.mjs';

const WAIT_MS = 15_000;
const GRID = t('ur', 'cashbook-general-grid-label');

/**
 * Types an account by its name in the account cell of a row and goes on to its بنام cell.
 * @param {number} row
 * @param {string} words
 */
async function account(row, words) {
  await (await cell(GRID, row, 'account')).click();
  await typeAndPick(words);
}

describe('the جنرل grid by keyboard (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
    await openRoute('/rokar/general');
    await waitForHeading('ur', 'cashbook-general-screen-title');
  });

  it('1. starts with two empty lines and Save is off', async () => {
    await waitForText(t('ur', 'cashbook-general-totals-hint'));
    const save = await browser.$(
      `//button[starts-with(normalize-space(.), "${t('ur', 'layout-save-save')}")]`,
    );
    expect(await save.isEnabled()).toBe(false);
    await shootScreen('general-empty', 'ur');
  });

  it('2. an entry that does not balance shows the totals and cannot be saved', async () => {
    await account(0, 'بجلی');
    await type('1000');
    await type(Key.Enter);
    await type(Key.Enter);
    await type(Key.Enter);
    await account(1, 'روکڑ');
    await type(Key.Enter);
    await type('900');
    await type(Key.Enter);
    await waitForText(t('ur', 'cashbook-general-not-balanced'));
    await waitForNumbers('1,000', '900');
    await type(Key.F10);
    await waitForText(t('ur', 'err-general-not-balanced', { dr: '1,000', cr: '900' }));
    await shootScreen('general-unbalanced', 'ur');
  });

  it('3. at 1,000 on each side it balances, the split shows, and F10 saves it', async () => {
    await (await cell(GRID, 1, 'cr')).click();
    await type('1000');
    await type(Key.Enter);
    await waitForText(t('ur', 'cashbook-general-balanced'));
    await waitForText(t('ur', 'cashbook-general-split-title'));
    await shootScreen('general-balanced', 'ur');
    await type(Key.F10);
    await waitForText(
      t('ur', 'cashbook-saved', {
        kind: 'general',
        number: t('ur', 'vouchers-number-general', { number: 1 }),
      }),
    );
  });

  it('4. a language switch in the middle of an entry loses nothing', async () => {
    await account(0, 'بجلی');
    await type('500');
    await type(Key.Enter);
    await waitForNumbers('500');
    await press([Key.Alt, 'l']);
    await waitForHeading('en', 'cashbook-general-screen-title');
    expect(await pageText()).toContain('500');
    await shootScreen('general', 'en');
    await press([Key.Alt, 'l']);
    await waitForHeading('ur', 'cashbook-general-screen-title');
    await browser.waitUntil(async () => (await pageText()).includes('500'), { timeout: WAIT_MS });
  });
});
