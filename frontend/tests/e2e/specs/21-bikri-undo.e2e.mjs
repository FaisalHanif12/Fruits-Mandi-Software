// E2E-22-style check of the undo (D4-A8, PRD FR-7.47): right after a save the message has an undo
// button for 10 seconds, with Alt+Z, in Urdu.
//   1. Types and saves a بکری: اکرم 20 at 600. The message has the button "واپس لیں Alt+Z".
//   2. Alt+Z takes the save back by reversal and opens a NEW unsaved entry with the same values:
//      the truck number, the lot and the line are back, and the message says so.
//   3. F10 saves the corrected entry. It has a new number: the cancelled one is never used again.
// This spec runs on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import { Key, focusField, press, startSignedIn, waitForText } from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import { cell, saveSale, typeSale, waitForNumbers } from '../support/bikri-sale.mjs';

const MESSAGE_WAIT_MS = 10_000;

describe('undo straight after a save (Urdu)', () => {
  let first = 0;

  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the message after a save has the undo button with its key', async () => {
    await typeSale(20, [{ buyer: 'akram', qty: 20, rate: 600 }]);
    first = await saveSale();
    const button = await $(`//button[contains(normalize-space(.), "${t('ur', 'bikri-undo')}")]`);
    await button.waitForDisplayed({ timeout: MESSAGE_WAIT_MS });
    expect(await button.getText()).toContain('Alt+Z');
    await shootScreen('bikri-undo-message', 'ur');
  });

  it('2. Alt+Z brings the values back in a new unsaved entry', async () => {
    await press([Key.Alt, 'z']);
    await waitForText(t('ur', 'bikri-undo-done', { number: first }));
    const truck = await focusField('ur', 'bikri-header-truck');
    expect(await truck.getValue()).toBe('LHR-20');
    const qty = await cell(t('ur', 'bikri-lots-label'), 0, 'qty');
    expect(await qty.getText()).toBe('20');
    await waitForNumbers('12,000', '12,880');
    await shootScreen('bikri-undo-restored', 'ur');
  });

  it('3. F10 saves the corrected entry under a new number', async () => {
    const second = await saveSale();
    expect(second).toBeGreaterThan(first);
  });
});
