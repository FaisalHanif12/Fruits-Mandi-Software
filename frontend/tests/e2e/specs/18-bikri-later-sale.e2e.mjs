// E2E-16 and E2E-16b (D4-A1, D4-A2, PRD 12.8, 12.9, FR-12.14 to FR-12.17): goods that are not sold
// on the first day wait in باقی مال and are sold later as a posting of their own, typed with the
// keyboard only, in Urdu.
//   1. Types a بکری of 50 پیٹی کیلا from رحیم with only 45 sold (اکرم 20 at 600, ارشد 25 at 580)
//      and saves it with F10. The numbers of E2E-16 show before the save.
//   2. باقی مال lists the lot with 5 left. Enter on the row opens the later sale.
//   3. The later sale form holds the lot read-only and one new line that starts at the 5 left. ارشد
//      at 560 gives the numbers of E2E-16b (value 2,800, commissions 168 and 168, لاگا 40, he pays
//      3,008, shared income 376). F10 saves it and goes back to the list, which is now empty.
//   4. A second بکری with 5 نگ left is written off with a reason; the lot leaves the list; with the
//      tick "also written off" the write-off is found and undone, and the 5 are back.
// The numbers themselves are proved by the Rust test `golden_day` (E2E-16, E2E-16b); this spec
// proves the screens send what was typed and show what the backend sends. It runs on Windows only
// (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  openRoute,
  pageText,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import {
  buttonNamed,
  saveSale,
  typeSale,
  waitForNumberedMessage,
  waitForNumbers,
} from '../support/bikri-sale.mjs';

const ROW_WAIT_MS = 10_000;

/** The row of the stock list that says this many نگ are left. @param {string} left */
function stockRow(left) {
  return $(
    `//*[@role="group"][@aria-label="${t('ur', 'bikri-stock-label')}"]//div[@role="row"][.//div[@col-id="balance"][normalize-space(.)="${left}"]]`,
  );
}

describe('E2E-16 and E2E-16b: goods left and a later sale (Urdu)', () => {
  let number = 0;

  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. E2E-16: 45 of 50 sold; the numbers show before the save, then F10 saves', async () => {
    await typeSale(50, [
      { buyer: 'akram', qty: 20, rate: 600 },
      { buyer: 'arshad', qty: 25, rate: 580 },
    ]);
    await waitForNumbers('26,500', '16,590', '3,560');
    await shootScreen('bikri-later-sale-first', 'ur');
    number = await saveSale();
  });

  it('2. باقی مال lists the lot with 5 left, and Enter on the row starts the later sale', async () => {
    await openRoute('/bikri/stock');
    await waitForHeading('ur', 'layout-module-stock');
    const row = await stockRow('5');
    await row.waitForDisplayed({ timeout: ROW_WAIT_MS });
    await shootScreen('bikri-stock-list', 'ur');
    await (await row.$('div[col-id="number"]')).click();
    await type(Key.Enter);
    await waitForText(t('ur', 'bikri-later-no-lot-charges'));
  });

  it('3. E2E-16b: ارشد takes the 5 at 560; F10 saves the later sale and the list is empty', async () => {
    // The new line starts at the 5 left (PRD FR-7.52): buyer, Enter on the quantity, the rate.
    const buyer = await $('[role="group"] [col-id="buyer"]');
    await buyer.click();
    await type('arshad');
    const first = await $('[role="option"]');
    await first.waitForDisplayed({ timeout: ROW_WAIT_MS });
    await type(`${Key.Enter}${Key.Enter}560${Key.Enter}`);
    await waitForNumbers('2,800', '168', '3,008', '376', '152', '112');
    await shootScreen('bikri-later-sale-form', 'ur');
    await type(Key.F10);
    const saved = await waitForNumberedMessage('bikri-later-saved-toast');
    expect(saved).toBe(number);
    await waitForHeading('ur', 'layout-module-stock');
    await waitForText(t('ur', 'bikri-stock-empty'));
  });

  it('4. a write-off with a reason takes the lot off the list, and its undo brings it back', async () => {
    await typeSale(20, [{ buyer: 'akram', qty: 15, rate: 600 }]);
    await saveSale();
    await openRoute('/bikri/stock');
    await waitForHeading('ur', 'layout-module-stock');
    const row = await stockRow('5');
    await row.waitForDisplayed({ timeout: ROW_WAIT_MS });
    await (await row.$('div[col-id="number"]')).click();
    const writeOff = await buttonNamed('bikri-stock-write-off');
    await writeOff.click();
    const dialog = await $('[role="alertdialog"]');
    await dialog.waitForDisplayed({ timeout: ROW_WAIT_MS });
    // The quantity field has focus order first; the reason follows.
    const qty = await dialog.$('input');
    await qty.setValue('5');
    const inputs = await dialog.$$('input');
    const reason = inputs[1];
    if (reason === undefined) throw new Error('The write-off box has no reason field');
    await reason.setValue('kharab');
    await (await dialog.$(`button*=${t('ur', 'bikri-write-off-confirm')}`)).click();
    await waitForText(t('ur', 'bikri-write-off-done', { qty: 5 }));
    await waitForText(t('ur', 'bikri-stock-empty'));
    // Also the lots that are written off: the write-off is found and undone.
    const tick = await $(
      `//label[contains(normalize-space(.), "${t('ur', 'bikri-stock-include-written-off')}")]//input`,
    );
    await tick.click();
    const lot = await $('div[role="row"] div[col-id="number"]');
    await lot.waitForDisplayed({ timeout: ROW_WAIT_MS });
    await lot.click();
    await (await buttonNamed('bikri-stock-undo-write-off')).click();
    const undoDialog = await $('[role="alertdialog"]');
    await undoDialog.waitForDisplayed({ timeout: ROW_WAIT_MS });
    const undoInputs = await undoDialog.$$('input');
    const undoReason = undoInputs[0];
    if (undoReason === undefined) throw new Error('The undo box has no reason field');
    await undoReason.setValue('mil gaya');
    await (await undoDialog.$(`button*=${t('ur', 'bikri-write-off-undo-confirm')}`)).click();
    await waitForText(t('ur', 'bikri-write-off-undone'));
    const text = await pageText();
    expect(text).toContain('5');
    await shootScreen('bikri-stock-after-undo', 'ur');
  });
});
