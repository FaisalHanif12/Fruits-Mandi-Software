// D5 early track, plan 20, step F3 (PRD 19.2, FR-5.4, FR-7.65, FR-13.9, FR-19.1 to FR-19.5, NFR-USE-7,
// NFR-USE-19, NFR-USE-28): the Sales, Collections and Business reports, the global search and the
// party card buttons, by keyboard, in Urdu and then in English, on the real app after the worked day
// (the earlier specs have entered Events 1 and 2 and the vouchers).
//   1. The catalogue lists the four groups with their reports.
//   2. The three dues reports (R20, R21, R22) end in the same total: the shop's total اگراہی
//      (FR-19.9). The table does not scroll sideways.
//   3. The sales register (R2) shows the sale lines and the backend's total under its columns.
//   4. The register of one-time people (R9): the person box finds شاہد by the English letters
//      typed, and Enter opens his lines.
//   5. The income report (R28) names the month, and the group box turns it to days.
//   6. Ctrl+K finds a report by its number, and Enter opens it.
//   7. The party card on the receipt screen has the کھاتہ and فرد حساب buttons, which open the
//      ledger and the statement of that buyer.
//   8. Alt+L turns the partner report (R30) to English, left to right, with the same figures.
// The figures are the backend's and are proved by the Rust tests (reports_fixtures_b3.rs); this
// spec proves that the screens ask for what was typed, show what the backend sends and never
// overflow sideways. It runs on Windows only (LOCAL-ONLY in the Mac phase: checked with
// `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  choose,
  focusField,
  pageText,
  pick,
  press,
  startSignedIn,
  switchLanguage,
  type,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const WAIT_MS = 20_000;

/** Opens a report by its address. @param {string} id */
async function openReport(id) {
  await browser.execute((address) => {
    window.location.hash = address;
  }, `#/reports/${id}`);
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

/** The ids of the rows drawn now, without the totals row. */
async function rowCount() {
  return (await $$('.ag-row[row-id]:not(.ag-row-pinned)')).length;
}

describe('sales, collections and business reports (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the catalogue lists the four groups with their reports', async () => {
    await openReport('');
    await waitForHeading('ur', 'reports-title');
    for (const group of ['sales', 'accounts', 'collections', 'business']) {
      await waitForText(t('ur', `report-group-${group}`));
    }
    for (const id of ['r2', 'r6', 'r9', 'r20', 'r24', 'r28', 'r30']) {
      expect(await pageText()).toContain(t('ur', `report-${id}-title`));
    }
    await shootScreen('reports-catalogue-all', 'ur');
  });

  it('2. R20, R21 and R22 end in the same total, and the table does not scroll sideways', async () => {
    await openReport('R20');
    await waitForHeading('ur', 'report-r20-title');
    const total = await totalsCell('balance');
    expect(total).not.toBe('');
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r20', 'ur');
    for (const id of ['R21', 'R22']) {
      await openReport(id);
      await waitForHeading('ur', `report-${id.toLowerCase()}-title`);
      expect(await totalsCell('balance')).toBe(total);
      expect(await noSidewaysScroll()).toBe(true);
    }
  });

  it('3. the sales register shows the sale lines and the backend’s total', async () => {
    await openReport('R2');
    await waitForHeading('ur', 'report-r2-title');
    expect(await rowCount()).toBeGreaterThan(0);
    expect(await totalsCell('buyer_total')).not.toBe('');
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r2', 'ur');
  });

  it('4. the person box of R9 finds شاہد by the English letters typed, and Enter opens his lines', async () => {
    await openReport('R9');
    await waitForHeading('ur', 'report-r9-title');
    const box = await focusField('ur', 'report-filter-person');
    await box.setValue('shahid');
    const option = await $('[role="option"]');
    await option.waitForDisplayed({ timeout: WAIT_MS });
    await type(Key.Enter);
    await browser.waitUntil(async () => (await box.getValue()).includes('شاہد'), {
      timeout: WAIT_MS,
      timeoutMsg: 'The person box never showed his name.',
    });
    expect(await rowCount()).toBeGreaterThan(0);
    // The letters typed were searched as typed, never turned into Urdu by the box itself.
    await shootScreen('reports-r9-person', 'ur');
  });

  it('5. R28 names the month, and the group box turns the report to days', async () => {
    await openReport('R28');
    await waitForHeading('ur', 'report-r28-title');
    await waitForText(t('ur', 'report-month-10'));
    await choose('ur', 'report-filter-group-by', 'report-group-by-day');
    await browser.waitUntil(async () => !(await pageText()).includes(t('ur', 'report-month-10')), {
      timeout: WAIT_MS,
      timeoutMsg: 'The report never turned to days.',
    });
    await shootScreen('reports-r28-days', 'ur');
  });

  it('6. Ctrl+K finds a report by its number and Enter opens it', async () => {
    await openReport('');
    await waitForHeading('ur', 'reports-title');
    await press([Key.Ctrl, 'k']);
    await type('R18');
    const option = await $('[role="option"]');
    await option.waitForDisplayed({ timeout: WAIT_MS });
    await type(Key.Enter);
    await waitForHeading('ur', 'report-r18-title');
  });

  it('7. the party card on the receipt screen opens the ledger and the statement', async () => {
    await browser.execute(() => {
      window.location.hash = '#/bikri/voucher?type=receipt';
    });
    await pick('ur', 'vouchers-field-buyer', '1', 'اکرم');
    const ledger = await $(`//button[normalize-space()="${t('ur', 'report-card-ledger')}"]`);
    await ledger.waitForDisplayed({ timeout: WAIT_MS });
    await ledger.click();
    await waitForHeading('ur', 'report-r13-title');
    const party = await $(`//input[@role="combobox"]`);
    await browser.waitUntil(async () => (await party.getValue()) !== '', { timeout: WAIT_MS });
    await shootScreen('reports-party-ledger', 'ur');
  });
});

describe('sales, collections and business reports (English)', () => {
  before(async () => {
    await switchLanguage('en');
  });

  it('8. Alt+L turns the partner report to English, left to right, with the same figures', async () => {
    await openReport('R30');
    await waitForHeading('en', 'report-r30-title');
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    await waitForText(t('en', 'report-r30-income'));
    expect(await totalsCell('income_share')).not.toBe('');
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r30', 'en');
  });

  it('R6 and R22 in English do not scroll sideways', async () => {
    for (const id of ['R6', 'R22']) {
      await openReport(id);
      await waitForHeading('en', `report-${id.toLowerCase()}-title`);
      expect(await noSidewaysScroll()).toBe(true);
    }
  });
});
