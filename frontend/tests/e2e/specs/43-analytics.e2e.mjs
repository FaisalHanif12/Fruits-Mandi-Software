// D5 early track, plan 21 (PRD 18.1 to 18.3, FR-18.2 to FR-18.5, FR-18.8, NFR-USE-7, NFR-USE-15,
// NFR-USE-19, NFR-USE-28): the تجزیات area, by keyboard, in Urdu and then in English, on the real app
// after the worked day (the earlier specs have entered Events 1 and 2 and the vouchers).
//   1. The main bar opens تجزیات on view A. The chart is drawn above the table, and the totals row is
//      the backend's. The table does not scroll sideways.
//   2. ← → on the tabs move between the six views; each opens with its title and its table.
//   3. Dates typed in the boxes ask again; the days are said under the filters.
//   4. Enter on a figure opens the lines behind it; Enter on a line opens its بکری; Esc comes back.
//   5. A view is saved under a name, found in the list of saved views, opened and deleted.
//   6. F9 shows the print preview of the view; Esc closes it.
//   7. Alt+L turns view D to English, left to right, with the same figures.
// The figures themselves are the backend's and are proved by `analytics.rs` (E2E-11); this spec proves
// the screen asks for what was pressed, shows what the backend sends and never overflows sideways. It
// runs on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  choose,
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
const SAVED_NAME = 'E2E کیلا کمیشن';

/** Opens a view by its address. @param {string} view */
async function openView(view) {
  await browser.execute((address) => {
    window.location.hash = address;
  }, `#/analytics/${view}`);
}

/** The text of a cell of the totals row. @param {string} column */
async function totalsCell(column) {
  const cell = await $(`.ag-row-pinned [col-id="${column}"]`);
  await cell.waitForDisplayed({ timeout: WAIT_MS });
  return (await cell.getText()).replace(/\s+/g, ' ').trim();
}

/** True when the page and the grid do not scroll sideways (NFR-USE-15). */
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

/** The number of rows drawn now, without the totals row. */
async function rowCount() {
  return (await $$('.ag-row[row-id]:not(.ag-row-pinned)')).length;
}

describe('analytics (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the main bar opens the area on view A, with its chart, its totals and no sideways scroll', async () => {
    await (await $(`//nav//button[@aria-label="${t('ur', 'layout-nav-analytics')}"]`)).click();
    await waitForHeading('ur', 'analytics-view-a-title');
    const chart = await $('[data-chart="bar"]');
    await chart.waitForDisplayed({ timeout: WAIT_MS });
    expect(await chart.$('svg').isExisting()).toBe(true);
    expect(await rowCount()).toBeGreaterThan(0);
    expect(await totalsCell('total_comm')).not.toBe('');
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('analytics-a', 'ur');
  });

  it('1b. a column that waits for room is offered, and comes in on a press (E2E-11 average rate)', async () => {
    const other = await $(
      `//main//button[normalize-space()="${t('ur', 'analytics-columns-other')}"]`,
    );
    if (await other.isExisting()) {
      await other.click();
      const average = await $('.ag-header-cell[col-id="avg_rate"]');
      await average.waitForDisplayed({ timeout: WAIT_MS });
      expect(await noSidewaysScroll()).toBe(true);
      const back = await $(
        `//main//button[normalize-space()="${t('ur', 'analytics-columns-main')}"]`,
      );
      await back.click();
    } else {
      // The window is wide enough for every column: the average rate is already there.
      expect(await $('.ag-header-cell[col-id="avg_rate"]').isExisting()).toBe(true);
    }
  });

  it('2. the arrow keys on the tabs move between the six views', async () => {
    const tab = await $(
      `//*[@role="tab"][normalize-space()="${t('ur', 'analytics-view-a-title')}"]`,
    );
    await tab.click();
    // In Urdu the next view is to the left.
    for (const view of ['b', 'c', 'd', 'e', 'h']) {
      await press(Key.ArrowLeft);
      await waitForHeading('ur', `analytics-view-${view}-title`);
      expect(await noSidewaysScroll()).toBe(true);
    }
    await shootScreen('analytics-h', 'ur');
    await openView('A');
    await waitForHeading('ur', 'analytics-view-a-title');
  });

  it('3. dates typed in the boxes ask again and the page says which days they are', async () => {
    await focusField('ur', 'report-filter-from');
    await type('01-10-2026');
    await focusField('ur', 'report-filter-to');
    await type('31-10-2026');
    await waitForText(t('ur', 'report-range-from-to', { from: '01-10-2026', to: '31-10-2026' }));
  });

  it('4. Enter on a figure opens the lines behind it; Enter on a line opens its بکری; Esc comes back', async () => {
    const first = await $('.ag-row[row-id="0"] [col-id="item"]');
    await first.waitForDisplayed({ timeout: WAIT_MS });
    await first.click();
    await type(Key.Enter);
    await waitForHeading('ur', 'analytics-lines-title');
    expect(await rowCount()).toBeGreaterThan(0);
    await shootScreen('analytics-lines', 'ur');
    await press(Key.Escape);
    await waitForHeading('ur', 'analytics-view-a-title');
    const again = await $('.ag-row[row-id="0"] [col-id="item"]');
    await again.click();
    await type(Key.Enter);
    await waitForHeading('ur', 'analytics-lines-title');
    const line = await $('.ag-row[row-id="0"] [col-id="date"]');
    await line.click();
    await type(Key.Enter);
    await browser.waitUntil(async () => (await browser.getUrl()).includes('/bikri/book'), {
      timeout: WAIT_MS,
      timeoutMsg: 'The line never opened its بکری.',
    });
    await openView('A');
    await waitForHeading('ur', 'analytics-view-a-title');
  });

  it('5. a view is saved under a name, opened from the list, and deleted', async () => {
    await (await $(`//button[normalize-space()="${t('ur', 'analytics-saved-save')}"]`)).click();
    const name = await focusField('ur', 'analytics-saved-name');
    await name.setValue(SAVED_NAME);
    await type(Key.Enter);
    await waitForText(t('ur', 'analytics-saved-done', { name: SAVED_NAME }));
    await openView('H');
    await waitForHeading('ur', 'analytics-view-h-title');
    await choose('ur', 'analytics-saved-label', 'analytics-saved-open');
    // The name of a saved view is a name, not a message: pick it from the list by its words.
    const option = await $(`//*[@role="option"][normalize-space()="${SAVED_NAME}"]`);
    await option.waitForDisplayed({ timeout: WAIT_MS });
    await option.click();
    await waitForHeading('ur', 'analytics-view-a-title');
    await (await $(`//button[normalize-space()="${t('ur', 'analytics-saved-delete')}"]`)).click();
    const confirm = await $(
      `//*[@role="alertdialog"]//button[normalize-space()="${t('ur', 'analytics-saved-delete')}"]`,
    );
    await confirm.waitForDisplayed({ timeout: WAIT_MS });
    await confirm.click();
    await waitForText(t('ur', 'analytics-saved-deleted'));
  });

  it('6. F9 shows the print preview of the view, and Esc closes it', async () => {
    await press(Key.F9);
    await waitForText(t('ur', 'report-print-title'));
    const sheet = await $('.print-report');
    await sheet.waitForDisplayed({ timeout: WAIT_MS });
    expect(await sheet.getAttribute('dir')).toBe('rtl');
    expect(await (await sheet.$('tfoot')).getText()).toContain(t('ur', 'report-total'));
    await shootScreen('analytics-print-preview', 'ur');
    await press(Key.Escape);
    await waitForHeading('ur', 'analytics-view-a-title');
    expect(await $('.print-report').isExisting()).toBe(false);
  });
});

describe('analytics (English)', () => {
  before(async () => {
    await switchLanguage('en');
  });

  it('7. Alt+L turns view D to English, left to right, with the same figures', async () => {
    await openView('D');
    await waitForHeading('en', 'analytics-view-d-title');
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    expect(await pageText()).toContain(t('en', 'analytics-chart-d'));
    expect(await totalsCell('total_comm')).not.toBe('');
    expect(await noSidewaysScroll()).toBe(true);
    const chart = await $('[data-chart="bar"]');
    expect(await chart.getAttribute('dir')).toBe('ltr');
    await shootScreen('analytics-d', 'en');
  });
});
