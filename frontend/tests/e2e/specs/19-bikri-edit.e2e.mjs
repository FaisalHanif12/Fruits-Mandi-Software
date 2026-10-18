// E2E-13 and E2E-14 (D4-A3, D4-A5, PRD 12.9, FR-12.18 to FR-12.21, FR-11.6): a saved بکری is
// corrected by editing one of its entries, in Urdu, with the keyboard only. The edit is a reversal
// plus a new entry on the same day; the lines that were not changed keep their stored values.
//   1. Types and saves a بکری: اکرم 20 at 600 (he pays 12,880).
//   2. Opens it from the "last saved" link and presses Edit. The loaded rate is 600.
//   3. Changes the rate to 610. The cell is marked and the list "آپ نے کیا بدلا" says it in words;
//      the numbers show his 13,092 before the save (E2E-13). F10 saves; the saved بکری shows it.
//   4. Edits it back to 600 and saves: every number returns to 12,880 (E2E-14).
// The numbers themselves are proved by the Rust test `golden_day` (E2E-13, E2E-14). This spec runs
// on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  pageText,
  startSignedIn,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import {
  buttonNamed,
  cell,
  openLastSaved,
  saveSale,
  typeSale,
  waitForNumberedMessage,
  waitForNumbers,
} from '../support/bikri-sale.mjs';

/** Changes the rate of the first line of the open edit to `rate` and presses Enter. @param {string} rate */
async function typeRate(rate) {
  const rateCell = await cell(t('ur', 'bikri-sales-label'), 0, 'rate');
  await rateCell.click();
  await type(`${rate}${Key.Enter}`);
}

describe('E2E-13 and E2E-14: edit a saved بکری and edit it back (Urdu)', () => {
  let number = 0;

  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. saves a بکری: اکرم 20 at 600', async () => {
    await typeSale(20, [{ buyer: 'akram', qty: 20, rate: 600 }]);
    await waitForNumbers('12,000', '12,880');
    number = await saveSale();
  });

  it('2. the saved بکری has Edit; it opens the entry with the loaded rate', async () => {
    await openLastSaved();
    await waitForText(t('ur', 'bikri-saved-readonly'));
    await (await buttonNamed('bikri-edit')).click();
    await waitForHeading('ur', 'bikri-edit-title');
    await waitForText(t('ur', 'bikri-edit-no-changes'));
    const rate = await cell(t('ur', 'bikri-sales-label'), 0, 'rate');
    expect(await rate.getText()).toBe('600');
  });

  it('3. E2E-13: rate 610 is marked and listed in words, shows 13,092, and F10 saves it', async () => {
    await typeRate('610');
    const rate = await cell(t('ur', 'bikri-sales-label'), 0, 'rate');
    await expect(rate).toHaveElementClass(expect.stringContaining('cell-changed'));
    await waitForText(
      t('ur', 'bikri-edit-change', {
        where: t('ur', 'bikri-edit-where-line', { lot: 1, line: 1 }),
        field: t('ur', 'bikri-col-rate'),
        before: '600',
        after: '610',
      }),
    );
    await waitForNumbers('12,200', '13,092');
    await shootScreen('bikri-edit-changed', 'ur');
    await type(Key.F10);
    const saved = await waitForNumberedMessage('bikri-edit-saved-toast');
    expect(saved).toBe(number);
    await waitForText(t('ur', 'bikri-saved-readonly'));
    await waitForNumbers('13,092');
    await shootScreen('bikri-edit-saved', 'ur');
  });

  it('4. E2E-14: editing the rate back to 600 returns every number to 12,880', async () => {
    await (await buttonNamed('bikri-edit')).click();
    await waitForHeading('ur', 'bikri-edit-title');
    await typeRate('600');
    await type(Key.F10);
    await waitForNumberedMessage('bikri-edit-saved-toast');
    await waitForText(t('ur', 'bikri-saved-readonly'));
    await waitForNumbers('12,880');
    const text = await pageText();
    expect(text).not.toContain('13,092');
  });
});
