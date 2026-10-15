// Plan 11 (D3-A2): the partner overview and the setup page through the screens, by keyboard.
//   1. The overview shows the three partners made in the wizard (spec 01) with their shares.
//   2. The setup page lists them with 40, 30 and 30 and a green total of 100.
//   3. A new split of 50, 30 and 20 is typed with Enter between the boxes and saved with F10; the
//      overview then shows 50% for the first partner.
//   4. A start day in the past is refused at the date field, and nothing changes.
//   5. The split goes back to 40, 30 and 30, because the later specs (the worked day) use it.
// The split of each posting is proved by the Rust tests (`cargo test -p mandi-services --test
// partners`); this spec proves the screens lead to the same shares.

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER, PARTNERS } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  button,
  focusField,
  openRoute,
  pageText,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const OLD_DAY = '01-01-2020';

/** The share boxes of the setup page, in the order of the list. */
function shareBoxes() {
  return $$('input[aria-label][dir="ltr"]');
}

/** Types the shares into the boxes, starting at the first. @param {number[]} shares */
async function typeShares(shares) {
  const boxes = await shareBoxes();
  await boxes[0]?.click();
  await type(shares.map(String).join(Key.Enter));
}

/** Waits for the total box to show this total. @param {string} percent */
async function waitForTotal(percent) {
  const total = await $('[role="status"]');
  await browser.waitUntil(async () => (await total.getText()).includes(`${percent}%`), {
    timeout: 10_000,
    timeoutMsg: `the total did not become ${percent}%`,
  });
  return total;
}

describe('the partners (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('the overview shows the three partners and their shares', async () => {
    await openRoute('/partners');
    await waitForHeading('ur', 'partners-title');
    const text = await pageText();
    for (const partner of PARTNERS) {
      expect(text).toContain(partner.nameUr);
      expect(text).toContain(`${String(partner.share)}%`);
    }
    await shootScreen('partners', 'ur');
  });

  it('the setup page lists them with a green total of 100', async () => {
    await (await button('ur', 'partners-setup-open')).click();
    await waitForHeading('ur', 'partners-setup-title');
    const boxes = await shareBoxes();
    expect(boxes).toHaveLength(PARTNERS.length);
    for (const [index, partner] of PARTNERS.entries()) {
      await expect(boxes[index]).toHaveValue(String(partner.share));
    }
    const total = await waitForTotal('100');
    await expect(total).toHaveText(expect.stringContaining(t('ur', 'partners-total-ok')));
    await shootScreen('partners-setup', 'ur');
  });

  it('a total that is not 100 is red, and Save stays off', async () => {
    await typeShares([30, 30, 30]);
    const total = await waitForTotal('90');
    await expect(total).toHaveText(expect.stringContaining(t('ur', 'partners-total-bad')));
    await expect(await button('ur', 'partners-save')).toBeDisabled();
    await shootScreen('partners-setup-red', 'ur');
  });

  it('a new split of 50, 30 and 20 is saved with F10 and shown on the overview', async () => {
    await typeShares([50, 30, 20]);
    await waitForTotal('100');
    await type(Key.F10);
    // The message says when the shares start; wait for the words before the date.
    const [saved] = t('ur', 'partners-saved', { date: '@' }).split('@');
    await waitForText((saved ?? '').trim());
    await openRoute('/partners');
    await waitForHeading('ur', 'partners-title');
    expect(await pageText()).toContain('50%');
  });

  it('a start day in the past is refused at the date field', async () => {
    await openRoute('/partners/setup');
    await waitForHeading('ur', 'partners-setup-title');
    await waitForTotal('100');
    const from = await focusField('ur', 'partners-field-from');
    await browser.keys([Key.Ctrl, 'a']);
    await type(OLD_DAY);
    await type(Key.F10);
    await browser.waitUntil(async () => (await from.getAttribute('aria-invalid')) === 'true', {
      timeout: 10_000,
      timeoutMsg: 'the past day was not refused',
    });
  });

  it('the split goes back to 40, 30 and 30', async () => {
    await openRoute('/partners/setup');
    await waitForHeading('ur', 'partners-setup-title');
    await typeShares(PARTNERS.map((partner) => partner.share));
    await waitForTotal('100');
    await type(Key.F10);
    await openRoute('/partners');
    await waitForHeading('ur', 'partners-title');
    expect(await pageText()).toContain('40%');
  });
});
