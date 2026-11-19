// D5 late track, round 11, garden slice (PRD 9.1, 18.2 I, 19.2 R34 and R36, FR-12.37, FR-7.46,
// NFR-USE-7, NFR-USE-15, NFR-USE-28): the gardens tile of the dashboard, the gardens report R36, the
// garden rows of the day summary R34 and the analytics view I, by keyboard, in Urdu and then in English,
// on the real app (a garden may or may not have been made by the earlier specs: both are right).
//   1. The dashboard shows the gardens tile with its words, never a raw ID, and does not scroll
//      sideways. With an open garden the tile says how many are open, and the open gardens are listed
//      under the partners; with none the tile says so.
//   2. Enter on the focused tile opens the list of gardens (where a new garden is made).
//   3. R36 shows the gardens (or says there are none), takes the layout "by partner" with the keys, and
//      never scrolls sideways.
//   4. R34 for a day shows the two garden rows (what was paid into the gardens, what came in).
//   5. View I opens from its tab, with its title, a chart or the empty message, a garden box in view, no
//      comparison box and no sideways scroll; F and J have no garden box.
//   6. Alt+L turns the tile, R36 and view I to English, left to right, with the same figures.
// The figures are the backend's (E2E-40) and are proved by the Rust tests and the fixtures; this spec
// proves that the real app shows them and opens the screens behind them. It runs on Windows only
// (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  openEndOfMenu,
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
const TILES = 'main button[data-nav-field]';

/** The tile whose label starts with a word, among the buttons of the screen. @param {string} text */
async function tileWith(text) {
  const found = await browser.execute(
    (selector, label) => {
      const tiles = Array.from(document.querySelectorAll(selector));
      return tiles.findIndex((tile) =>
        tile.textContent.replace(/[⁨⁩]/g, '').trim().startsWith(label),
      );
    },
    TILES,
    text,
  );
  if (found < 0) throw new Error(`No tile starts with "${text}".`);
  const tile = (await $$(TILES))[found];
  if (tile === undefined) throw new Error(`The tile "${text}" went away.`);
  return tile;
}

/** True when the page and the main area do not scroll sideways (NFR-USE-15). */
async function noSidewaysScroll() {
  return browser.execute(() => {
    const page = document.documentElement;
    const main = document.querySelector('main');
    return (
      page.scrollWidth <= window.innerWidth &&
      (main === null || main.scrollWidth <= main.clientWidth)
    );
  });
}

/** True when the page shows no raw message ID, markup or error code (NFR-USE-14). */
async function showsNoRawText() {
  const text = await pageText();
  return !/ERR_|<num>|<code>|dashboard-|gardens-/.test(text);
}

/** Opens the dashboard with the keyboard and waits for its tiles. @param {'ur' | 'en'} language */
async function openDashboard(language) {
  await openEndOfMenu('first');
  await waitForText(t(language, 'dashboard-quick-title'));
  await browser.waitUntil(async () => (await $$(TILES)).length > 0, {
    timeout: WAIT_MS,
    timeoutMsg: 'The dashboard never drew its tiles.',
  });
}

/** The words the tile shows when no garden is open, or how many are. @param {'ur' | 'en'} language */
async function tileSaysItsCount(language) {
  const tile = await tileWith(t(language, 'dashboard-tile-gardens'));
  const text = (await tile.getText()).replace(/[⁨⁩]/g, '');
  return (
    text.includes(t(language, 'dashboard-tile-gardens-none')) ||
    /\d/.test(text.replace(t(language, 'dashboard-tile-gardens'), ''))
  );
}

/** The number of rows drawn now, without the totals row. */
async function rowCount() {
  return (await $$('.ag-row[row-id]:not(.ag-row-pinned)')).length;
}

/** True when the table has a column with this id in its header. @param {string} column */
async function hasColumn(column) {
  return (await $$(`.ag-header-cell[col-id="${column}"]`)).length > 0;
}

/** Waits until the table has a row, or the page says there is nothing. @param {'ur' | 'en'} language */
async function waitForAnswer(language) {
  await browser.waitUntil(
    async () =>
      (await rowCount()) > 0 ||
      (await pageText()).includes(t(language, 'report-empty')) ||
      (await pageText()).includes(t(language, 'analytics-empty')),
    { timeout: WAIT_MS, timeoutMsg: 'The screen never answered.' },
  );
}

/**
 * Picks an option of a drop-down with the keys: the box has focus, Enter opens the list, the arrows
 * move from the option now chosen, Enter takes the one under the cursor.
 * @param {import('../support/messages.mjs').Language} language
 * @param {string} fieldId the label of the drop-down
 * @param {number} downs how many options to move down from the one now chosen
 */
async function pickWithKeys(language, fieldId, downs) {
  await focusField(language, fieldId);
  await press(Key.Enter);
  const option = await $('[role="option"]');
  await option.waitForDisplayed({ timeout: WAIT_MS });
  for (let step = 0; step < downs; step += 1) await press(Key.ArrowDown);
  await press(Key.Enter);
}

/** True when a box with this label is on the page now. @param {string} label */
async function hasBox(label) {
  return (await $$(`//label[normalize-space()="${label}"]`)).length > 0;
}

describe('the gardens tile of the dashboard (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. the dashboard shows the gardens tile with its words, and does not scroll sideways', async () => {
    await openDashboard('ur');
    const tile = await tileWith(t('ur', 'dashboard-tile-gardens'));
    await tile.waitForDisplayed({ timeout: WAIT_MS });
    // The tile reads its own summary: it says "checking" for a moment, then a figure or "none".
    await browser.waitUntil(async () => tileSaysItsCount('ur'), {
      timeout: WAIT_MS,
      timeoutMsg: 'The gardens tile never showed a figure or its empty words.',
    });
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('dashboard-gardens', 'ur');
  });

  it('2. Enter on the focused tile opens the list of gardens', async () => {
    const tile = await tileWith(t('ur', 'dashboard-tile-gardens'));
    await browser.execute((element) => {
      if (element instanceof HTMLElement) element.focus();
    }, tile);
    await press(Key.Enter);
    await waitForHeading('ur', 'gardens-title');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
  });
});

describe('the gardens report, the day summary and view I (Urdu)', () => {
  it('3. R36 shows the gardens or says there are none, takes the layout by partner with the keys, and does not scroll sideways', async () => {
    await openRoute('#/reports/R36');
    await waitForHeading('ur', 'report-r36-title');
    await waitForAnswer('ur');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r36-garden', 'ur');
    // By garden, by partner: by partner is one down.
    await pickWithKeys('ur', 'report-filter-garden-view', 1);
    await waitForAnswer('ur');
    if ((await rowCount()) > 0) expect(await hasColumn('partner')).toBe(true);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r36-partner', 'ur');
  });

  it('4. R34 for a day has the two garden rows', async () => {
    await openRoute('#/reports/R34');
    await waitForHeading('ur', 'report-r34-title');
    await focusField('ur', 'report-filter-from');
    await type('10-10-2026');
    await focusField('ur', 'report-filter-to');
    await type('10-10-2026');
    await waitForText(t('ur', 'report-r34-gardens-paid'));
    expect(await pageText()).toContain(t('ur', 'report-r34-gardens-in'));
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
  });

  it('5. view I opens with its title, a garden box in view and no comparison box; F and J have no garden box', async () => {
    await openRoute('#/analytics/I');
    await waitForHeading('ur', 'analytics-view-i-title');
    await waitForAnswer('ur');
    expect(await hasBox(t('ur', 'analytics-filter-garden'))).toBe(true);
    expect(await hasBox(t('ur', 'analytics-filter-compare'))).toBe(false);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('analytics-i', 'ur');
    for (const view of ['f', 'j']) {
      await openRoute(`#/analytics/${view.toUpperCase()}`);
      await waitForHeading('ur', `analytics-view-${view}-title`);
      expect(await hasBox(t('ur', 'analytics-filter-garden'))).toBe(false);
    }
  });
});

describe('the gardens tile of the dashboard (English)', () => {
  it('6. Alt+L turns the tile to English, left to right, with its figure', async () => {
    await openDashboard('ur');
    const before = await (await tileWith(t('ur', 'dashboard-tile-gardens'))).getText();
    const digits = before.replace(/\D/g, '');
    await switchLanguage('en');
    await waitForText(t('en', 'dashboard-quick-title'));
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    const tile = await tileWith(t('en', 'dashboard-tile-gardens'));
    await tile.waitForDisplayed({ timeout: WAIT_MS });
    expect((await tile.getText()).replace(/\D/g, '')).toBe(digits);
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('dashboard-gardens', 'en');
    await switchLanguage('ur');
  });
  it('6b. R36 and view I are in English, left to right, with the same rows', async () => {
    await openRoute('#/reports/R36');
    await waitForHeading('en', 'report-r36-title');
    await waitForAnswer('en');
    const rows = await rowCount();
    expect(await browser.execute(() => document.documentElement.dir)).toBe('ltr');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('reports-r36-garden', 'en');
    await openRoute('#/analytics/I');
    await waitForHeading('en', 'analytics-view-i-title');
    await waitForAnswer('en');
    expect(await showsNoRawText()).toBe(true);
    expect(await noSidewaysScroll()).toBe(true);
    await shootScreen('analytics-i', 'en');
    await switchLanguage('ur');
    await openRoute('#/reports/R36');
    await waitForHeading('ur', 'report-r36-title');
    await browser.waitUntil(async () => (await rowCount()) === rows, { timeout: WAIT_MS });
  });
});
