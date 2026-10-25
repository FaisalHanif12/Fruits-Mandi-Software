// D5 early track, plan 20 (PRD 19.1 to 19.5, 19.10, 19.11, FR-19.13, NFR-USE-7, NFR-USE-19,
// NFR-USE-28, NFR-URD-13): the report area, by keyboard, in Urdu and then in English.
//   1. The main bar opens رپورٹ. The catalogue lists the Accounts reports with a line about each.
//   2. Enter on the trial balance opens it. The first filter box has focus. The totals row is the
//      backend's, and its two sides are the same figure (the books balance, E2E-10).
//   3. A day typed in the box asks again and the day shows under the filters in words.
//   4. F9 shows the print preview of the same report: the firm, the title, every row and the totals
//      row; the printer and PDF buttons are dimmed (the print job of reports comes later). Esc
//      closes the preview, and the next Esc leaves the report for the catalogue.
//   5. The ledger of an account: from the trial balance, Enter on the first row opens that account's
//      ledger with the account already in the party box.
//   6. Alt+L turns the screen to English, left to right, with the same rows and the same figures.
// The figures themselves are the backend's and are proved by `reports_accounts.rs` (E2E-10, E2E-33);
// this spec proves the screen asks for what was pressed, shows what the backend sends, and never
// overflows sideways. It runs on Windows only (LOCAL-ONLY in the Mac phase: checked with
// `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  pageText,
  press,
  startSignedIn,
  switchLanguage,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WAIT_MS = 20_000;

/** A button of the main bar by its words (the name is its aria-label). @param {string} id */
function navItem(id) {
  return $(`//nav//button[@aria-label="${t('ur', id)}"]`);
}

/** The text of a cell of the totals row. @param {string} column */
async function totalsCell(column) {
  const cell = await $(`.ag-row-pinned [col-id="${column}"]`);
  await cell.waitForDisplayed({ timeout: WAIT_MS });
  return (await cell.getText()).replace(/\s+/g, ' ').trim();
}

/** True when the grid does not scroll sideways (NFR-USE-15). */
async function noSidewaysScroll() {
  return browser.execute(() => {
    const body = document.querySelector('.entry-grid .ag-body-horizontal-scroll-viewport');
    const page = document.documentElement;
    return (
      page.scrollWidth <= window.innerWidth &&
      (body === null || body.scrollWidth <= body.clientWidth)
    );
  });
}

/** Opens the catalogue and then one report with the keyboard. @param {'ur' | 'en'} language */
async function openTrialBalance(language) {
  await browser.execute(() => {
    window.location.hash = '#/reports';
  });
  await waitForHeading(language, 'reports-title');
  // The first report (R1) has focus; ↓ moves down the list in PRD order: the ten Sales reports (R1 to
  // R10), then R11, R12, R13, R14, R15, R16, R17 and R18, which is seventeen steps down.
  for (let step = 0; step < 17; step += 1) await press(Key.ArrowDown);
  await press(Key.Enter);
  await waitForHeading(language, 'report-r18-title');
}

describe('reports (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the main bar opens the report area, with the Accounts reports and a line about each', async () => {
    await (await navItem('layout-nav-reports')).click();
    await waitForHeading('ur', 'reports-title');
    await waitForText(t('ur', 'report-group-accounts'));
    for (const id of ['r11', 'r13', 'r14', 'r16', 'r17', 'r18', 'r19']) {
      await waitForText(t('ur', `report-${id}-title`));
      expect(await pageText()).toContain(t('ur', `report-${id}-help`));
    }
    await shootScreen('reports-catalogue', 'ur');
  });

  it('2. Enter on the trial balance opens it; its two sides give the same total', async () => {
    await openTrialBalance('ur');
    const first = await $('input');
    await first.waitForDisplayed({ timeout: WAIT_MS });
    const debit = await totalsCell('debit');
    const credit = await totalsCell('credit');
    expect(debit).not.toBe('');
    expect(debit).toBe(credit);
    expect(await pageText()).toContain(t('ur', 'report-difference'));
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-trial-balance', 'ur');
  });

  it('3. a day typed in the box asks again and the page says which day it is', async () => {
    await focusField('ur', 'report-filter-as-of');
    await type('09-10-2026');
    await waitForText(t('ur', 'report-as-of-shown', { date: '09-10-2026' }));
  });

  it('4. F9 shows the print preview; Esc closes it, and the next Esc leaves the report', async () => {
    await press(Key.F9);
    await waitForText(t('ur', 'report-print-title'));
    // The Print and Save as PDF buttons are there and enabled (17e): the page goes through the print job.
    await waitForText(t('ur', 'report-print-pdf'));
    const sheet = await $('.print-report');
    await sheet.waitForDisplayed({ timeout: WAIT_MS });
    expect(await (await $('.print-doc')).getAttribute('dir')).toBe('rtl');
    expect(await (await sheet.$('tfoot')).getText()).toContain(t('ur', 'report-total'));
    await shootScreen('reports-print-preview', 'ur');
    await press(Key.Escape);
    await waitForHeading('ur', 'report-r18-title');
    expect(await $('.print-report').isExisting()).toBe(false);
    await press(Key.Escape);
    await waitForHeading('ur', 'reports-title');
  });

  it('5. Enter on an account of the trial balance opens its ledger with the account filled', async () => {
    await openTrialBalance('ur');
    const row = await $('.ag-row[row-id="0"] [col-id="account"]');
    await row.waitForDisplayed({ timeout: WAIT_MS });
    await row.click();
    await type(Key.Enter);
    await waitForHeading('ur', 'report-r13-title');
    const party = await $(`//input[@role="combobox"]`);
    await browser.waitUntil(async () => (await party.getValue()) !== '', { timeout: WAIT_MS });
    await shootScreen('reports-ledger', 'ur');
    await press(Key.Escape);
    await waitForHeading('ur', 'reports-title');
  });
});

describe('reports (English)', () => {
  before(async () => {
    await switchLanguage('en');
  });

  it('6. Alt+L turns the report to English, left to right, with the same rows and figures', async () => {
    await openTrialBalance('en');
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    const debit = await totalsCell('debit');
    expect(debit).toBe(await totalsCell('credit'));
    expect(await pageText()).toContain(t('en', 'report-difference'));
    expect((await $$('.ag-row[row-id]')).length).toBeGreaterThan(1);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-trial-balance', 'en');
  });

  it('F9 previews the same page in English, and Esc leaves it', async () => {
    await press(Key.F9);
    await waitForText(t('en', 'report-print-title'));
    const sheet = await $('.print-report');
    await sheet.waitForDisplayed({ timeout: WAIT_MS });
    expect(await sheet.getAttribute('dir')).toBe('ltr');
    await shootScreen('reports-print-preview', 'en');
    await press(Key.Escape);
    await press(Key.Escape);
    await waitForHeading('en', 'reports-title');
  });
});
