// E2E-15 (D4-A6, D4-A7, PRD 12.9, FR-12.11, FR-12.22): a whole بکری is cancelled with a reason, in
// Urdu, with the keyboard only. Nothing is deleted: every entry is reversed on its own day and the
// بکری stays in the list marked cancelled.
//   1. Types and saves a بکری: اکرم 20 at 600.
//   2. Opens it and presses "بکری منسوخ کریں". The box names the entries that will be reversed. An
//      empty reason is told at the field and nothing is sent.
//   3. The reason is typed and Enter cancels. The message names the بکری; the saved view says it is
//      cancelled, in words, with the reason; آج کی بکری shows it with the status منسوخ.
// The numbers (cash back, the facts netting to zero, the split reversed) are proved by the Rust
// test `golden_day` (E2E-15). This spec runs on Windows only (LOCAL-ONLY in the Mac phase: checked
// with `node --check`).

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
import { buttonNamed, openLastSaved, saveSale, typeSale } from '../support/bikri-sale.mjs';

const DIALOG_WAIT_MS = 10_000;

describe('E2E-15: cancel a whole بکری with a reason (Urdu)', () => {
  let number = 0;

  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. saves a بکری: اکرم 20 at 600', async () => {
    await typeSale(20, [{ buyer: 'akram', qty: 20, rate: 600 }]);
    number = await saveSale();
  });

  it('2. the box names what it reverses, and an empty reason is told at the field', async () => {
    await openLastSaved();
    await waitForText(t('ur', 'bikri-saved-readonly'));
    await (await buttonNamed('bikri-cancel-bikri')).click();
    const dialog = await $('[role="alertdialog"]');
    await dialog.waitForDisplayed({ timeout: DIALOG_WAIT_MS });
    await waitForText(t('ur', 'bikri-cancel-reverses'));
    await shootScreen('bikri-cancel-box', 'ur');
    await type(Key.Enter);
    await waitForText(t('ur', 'err-required'));
  });

  it('3. the reason is typed and Enter cancels; the بکری stays, marked cancelled', async () => {
    await type('galat darj hui');
    await type(Key.Enter);
    await waitForText(t('ur', 'bikri-cancelled-toast', { number }));
    await waitForText(t('ur', 'bikri-saved-cancelled'));
    await shootScreen('bikri-cancelled', 'ur');
    await openRoute('/bikri/today');
    await waitForHeading('ur', 'layout-module-today');
    await waitForText(t('ur', 'bikri-status-cancelled'));
    expect(await pageText()).toContain(t('ur', 'bikri-status-cancelled'));
  });
});
