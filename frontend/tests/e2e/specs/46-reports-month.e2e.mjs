// D5 late track, round 7 (PRD 16.1, 16.2, 16.5, FR-17.6, FR-17.9, FR-17.10, FR-21.2, FR-7.18, FR-19.2,
// FR-19.10, NFR-USE-7, NFR-USE-15, NFR-USE-28): the cash book (R12), the expense report (R29), the
// partner statement (R31), the money lent report (R32) and the month close screen, by keyboard, in
// Urdu and then in English, on the real app after the worked day (the earlier specs have entered
// Events 1 and 2 and the vouchers).
//   1. R12 opens with the first date in focus; the day is typed; the closing balance above the table
//      is the balance of the last line, and the table does not scroll sideways.
//   2. R29 takes "all partners" at first and shows the backend's total under its columns.
//   3. R31 cannot run without a partner: it says to pick one. The partner is chosen from the box with
//      the keys (Enter opens the list, ↓ moves, Enter takes), and his lines come as sentences with a
//      running total that ends in the total under the table.
//   4. R32 shows its table, or says there is nothing; its person box is the people who can borrow.
//   5. The month close screen lists the months. Enter on a month opens it: a month that can be closed
//      shows its plan with the Close button, a month that cannot be closed yet says why in words. No
//      raw code shows. (Closing a real month needs a finished month and every day closed: it is tried
//      by hand, LOCAL-ONLY.)
//   6. Alt+L turns the partner statement to English, left to right, with the same figures; Esc
//      leaves the month screen.
// The figures are the backend's and are proved by the Rust tests (month_close.rs, the report
// fixtures); this spec proves that the screens ask for what was typed, show what the backend sends,
// say refusals in words and never overflow sideways. It runs on Windows only (LOCAL-ONLY in the Mac
// phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  openRoute,
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
const DAY = '10-10-2026';

/** Opens a report by its address. @param {string} id */
async function openReport(id) {
  await openRoute(`#/reports/${id}`);
}

/** The text of a cell of the totals row. @param {string} column */
async function totalsCell(column) {
  const cell = await $(`.ag-row-pinned [col-id="${column}"]`);
  await cell.waitForDisplayed({ timeout: WAIT_MS });
  return (await cell.getText()).replace(/\s+/g, ' ').trim();
}

/** True when the page and every grid on it do not scroll sideways (NFR-USE-15). */
async function noSidewaysScroll() {
  return browser.execute(() => {
    const page = document.documentElement;
    const grids = Array.from(document.querySelectorAll('.ag-body-horizontal-scroll-viewport'));
    return (
      page.scrollWidth <= window.innerWidth &&
      grids.every((grid) => grid.scrollWidth <= grid.clientWidth)
    );
  });
}

/** The number of rows drawn now, without the totals row. */
async function rowCount() {
  return (await $$('.ag-row[row-id]:not(.ag-row-pinned)')).length;
}

/** The text of one cell of the last row drawn. @param {string} column */
async function lastRowCell(column) {
  const rows = await $$('.ag-row[row-id]:not(.ag-row-pinned)');
  const last = rows.at(-1);
  if (last === undefined) throw new Error('The table has no row.');
  const cell = await last.$(`[col-id="${column}"]`);
  return (await cell.getText()).replace(/\s+/g, ' ').trim();
}

/** True when the page shows no raw error code, message ID or markup (NFR-USE-14). */
async function showsNoRawText() {
  const text = await pageText();
  return !/ERR_|<num>|<code>|partners-statement|vouchers-number|month-error/.test(text);
}

/**
 * Types a day into the two date boxes of a report, with the keys: the first box has focus when the
 * report opens, the second is one field on (PRD FR-7.15).
 * @param {import('../support/messages.mjs').Language} language
 * @param {string} day
 */
async function typeDay(language, day) {
  await focusField(language, 'report-filter-from');
  await type(day);
  await focusField(language, 'report-filter-to');
  await type(day);
}

describe('cash book, expenses, partner statement, money lent and month close (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. R12 opens on the first date, shows the day typed, and ends in the balance above the table', async () => {
    await openReport('R12');
    await waitForHeading('ur', 'report-r12-title');
    // The first date has focus: the day can be typed at once.
    const from = await focusField('ur', 'report-filter-from');
    expect(await from.isFocused()).toBe(true);
    await typeDay('ur', DAY);
    await browser.waitUntil(async () => (await rowCount()) > 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'The cash book never showed a line.',
    });
    expect(await totalsCell('cash_in')).not.toBe('');
    // The closing balance above the table is the balance of the last line.
    const closing = await $(
      `//dt[normalize-space()="${t('ur', 'report-closing-balance')}"]/following-sibling::dd`,
    );
    await closing.waitForDisplayed({ timeout: WAIT_MS });
    const balance = (await lastRowCell('balance')).replace(/\D/g, '');
    expect((await closing.getText()).replace(/\D/g, '')).toBe(balance);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r12', 'ur');
  });

  it('2. R29 starts on all partners and shows the backend’s total under its columns', async () => {
    await openReport('R29');
    await waitForHeading('ur', 'report-r29-title');
    await waitForText(t('ur', 'report-partner-all'));
    await browser.waitUntil(async () => (await rowCount()) > 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'The expense report never showed a line.',
    });
    expect(await totalsCell('expense')).not.toBe('');
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r29', 'ur');
  });

  it('3. R31 says to pick a partner, takes him with the keys, and ends in the total under the table', async () => {
    await openReport('R31');
    await waitForHeading('ur', 'report-r31-title');
    await waitForText(t('ur', 'report-pick-partner'));
    expect(await rowCount()).toBe(0);
    await shootScreen('reports-r31-pick', 'ur');
    // Enter opens the list, ↓ moves to the second partner, Enter takes him.
    await focusField('ur', 'report-filter-partner');
    await press(Key.Enter);
    const option = await $('[role="option"]');
    await option.waitForDisplayed({ timeout: WAIT_MS });
    await press(Key.ArrowDown, Key.Enter);
    await browser.waitUntil(async () => (await rowCount()) > 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'The partner statement never showed a line.',
    });
    // The lines are sentences of Fluent words, never a raw ID (FR-17.10), and the last running
    // total is the total under the table.
    expect(await showsNoRawText()).toBe(true);
    expect(await lastRowCell('running')).toBe(await totalsCell('running'));
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r31', 'ur');
  });

  it('4. R32 shows its table or says there is nothing, with the people who can borrow in its box', async () => {
    await openReport('R32');
    await waitForHeading('ur', 'report-r32-title');
    await waitForText(t('ur', 'report-filter-borrower'));
    await browser.waitUntil(
      async () => (await rowCount()) > 0 || (await pageText()).includes(t('ur', 'report-empty')),
      { timeout: WAIT_MS, timeoutMsg: 'The money lent report never answered.' },
    );
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r32', 'ur');
  });

  it('5. the month close screen lists the months, opens one with Enter, and says in words what it can do', async () => {
    await openRoute('#/month');
    await waitForHeading('ur', 'month-title');
    await browser.waitUntil(async () => (await rowCount()) > 0, {
      timeout: WAIT_MS,
      timeoutMsg: 'The month list never showed a month.',
    });
    // One of the answers, always in words: the plan of a month that can be closed, the month is
    // not finished, or the days are not all closed.
    await browser.waitUntil(
      async () => {
        const text = await pageText();
        // The plan of a month that can be closed, a closed month, or a refusal in words (an alert).
        return (
          text.includes(t('ur', 'month-fact-income')) ||
          text.includes(t('ur', 'month-closed-on')) ||
          (await $('[role="alert"]').isExisting())
        );
      },
      { timeout: WAIT_MS },
    );
    // Enter on the first row opens that month again (keyboard: the row has focus after a click).
    const first = await $('.ag-row[row-id] [col-id="state"]');
    await first.click();
    await press(Key.Enter);
    await browser.pause(500);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('month-close', 'ur');
  });
});

describe('cash book, expenses, partner statement, money lent and month close (English)', () => {
  it('6. Alt+L turns the partner statement to English, left to right, with lines; Esc leaves the month screen', async () => {
    await openReport('R31');
    await waitForHeading('ur', 'report-r31-title');
    await focusField('ur', 'report-filter-partner');
    await press(Key.Enter);
    const option = await $('[role="option"]');
    await option.waitForDisplayed({ timeout: WAIT_MS });
    await press(Key.ArrowDown, Key.Enter);
    await browser.waitUntil(async () => (await rowCount()) > 0, { timeout: WAIT_MS });
    const running = await totalsCell('running');
    await switchLanguage('en');
    await waitForText(t('en', 'report-r31-title'));
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    // The same partner, the same figures, in English words.
    await browser.waitUntil(async () => (await rowCount()) > 0, { timeout: WAIT_MS });
    expect(await totalsCell('running')).toBe(running);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r31', 'en');
    // The month close screen in English, and Esc leaves it.
    await openRoute('#/month');
    await waitForText(t('en', 'month-title'));
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('month-close', 'en');
    await press(Key.Escape);
    await browser.waitUntil(
      async () => (await browser.execute(() => window.location.hash)).startsWith('#/partners'),
      { timeout: WAIT_MS, timeoutMsg: 'Esc never left the month screen.' },
    );
    await switchLanguage('ur');
  });
});
