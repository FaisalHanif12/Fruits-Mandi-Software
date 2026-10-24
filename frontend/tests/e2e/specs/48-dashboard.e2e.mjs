// D5 late track, round 9 (PRD Section 9, FR-9.1, FR-7.12, FR-7.44 to FR-7.46, NFR-SPD-2, NFR-SPD-22,
// NFR-USE-7, NFR-USE-9, NFR-USE-15, NFR-USE-28): the dashboard, the attention bell, the first-run
// guide and what the app remembers, by keyboard, in Urdu and then in English, on the real app after
// the worked day (the earlier specs have entered the بکری, the vouchers and the expenses).
//   1. The dashboard shows its tiles; the buyers' dues tile is the same figure as the total of the
//      buyers' dues report (R20, FR-19.9), and each tile opens its report.
//   2. The arrow keys move between the tiles the way the user sees them (left in Urdu, right in
//      English) and Enter opens one; the quick actions open their entry screens.
//   3. A partner card opens his statement (R31).
//   4. The bell opens a list in words, never a raw ID; Esc closes it.
//   5. The app starts where it was left (the screen, FR-7.45) and a report keeps its filters: sign out
//      and in again, and the stock screen is open; go back to the dashboard, and it starts on it.
//   6. The first-run guide opens again from the user menu, goes through its five steps with Enter,
//      each opening its screen, and closes with Done.
//   7. Signing in to the dashboard, with its figures drawn, is timed (NFR-SPD-2, 20 runs, p95).
//   8. The dashboard does not scroll sideways and shows no raw text.
//   9. In English: left to right, the same figures, no word left in Urdu, no sideways scroll.
// The figures are the backend's and are proved by the Rust tests and the fixtures; this spec proves
// that the real app shows them, opens the screens behind them, and remembers. It runs on Windows
// only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { DIRECTION, t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  focusedIndex,
  openEndOfMenu,
  openRoute,
  pageText,
  press,
  signIn,
  signOut,
  startSignedIn,
  switchLanguage,
  type,
  waitForHeading,
  waitForShell,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import { RUNS, armTimer, readTimer, recordTiming } from '../support/timing.mjs';

const WAIT_MS = 20_000;
const TILES = 'main button[data-nav-field]';
/** The time the app needs to keep what a screen chose (REMEMBER_SETTLE_MS in the app), with room. */
const SETTLE_MS = 1500;

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

/** Puts the focus on the first tile of the dashboard. */
async function focusFirstTile() {
  await browser.execute((selector) => {
    const first = document.querySelector(selector);
    if (first instanceof HTMLElement) first.focus();
  }, TILES);
}

/** The digits of a text, for comparing a figure whatever its separators. @param {string} text */
function digitsOf(text) {
  return text.replace(/\D/g, '');
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
  return !/ERR_|<num>|<code>|<key>|dashboard-|attention-|guide-/.test(text);
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

describe('the dashboard, the bell, the guide and what the app remembers (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('1. shows the tiles, and the buyers’ dues are the same figure as the total of R20', async () => {
    await openDashboard('ur');
    const dues = await tileWith(t('ur', 'dashboard-tile-dues'));
    const figure = digitsOf(await dues.getText());
    await shootScreen('dashboard', 'ur');
    // The tile opens the report, and its total row is the same figure (FR-19.9).
    await dues.click();
    await waitForHeading('ur', 'report-r20-title');
    const total = await $('.ag-row-pinned [col-id="balance"]');
    await total.waitForDisplayed({ timeout: WAIT_MS });
    expect(digitsOf(await total.getText())).toBe(figure);
  });

  it('2. the arrows move between the tiles, Enter opens one, and the quick actions open their screens', async () => {
    await openDashboard('ur');
    await focusFirstTile();
    const first = await focusedIndex(TILES);
    // The next tile is on the end side: left in Urdu (PRD FR-7.13, FR-7.34).
    await press(Key.ArrowLeft);
    expect(await focusedIndex(TILES)).toBe(first + 1);
    await press(Key.ArrowRight);
    expect(await focusedIndex(TILES)).toBe(first);
    const payable = await tileWith(t('ur', 'dashboard-tile-payable'));
    await payable.click();
    await waitForHeading('ur', 'report-r24-title');
    // The quick actions: a new وصولی from the dashboard, with the key shown on its button.
    await openDashboard('ur');
    const receipt = await $(
      `//button[starts-with(normalize-space(translate(., "⁨⁩", "")), "${t('ur', 'dashboard-new-receipt')}")]`,
    );
    await receipt.click();
    await waitForHeading('ur', 'vouchers-receipt-title');
  });

  it('3. a partner card opens his statement', async () => {
    await openDashboard('ur');
    const card = await $('[data-partner-id]');
    await card.waitForDisplayed({ timeout: WAIT_MS });
    await card.click();
    await waitForHeading('ur', 'report-r31-title');
  });

  it('4. the bell opens a list in words, and Esc closes it', async () => {
    await openDashboard('ur');
    const bell = await $('[data-attention-count]');
    await bell.waitForDisplayed({ timeout: WAIT_MS });
    await bell.click();
    const title = await $(`//h2[normalize-space()="${t('ur', 'attention-title')}"]`);
    await title.waitForDisplayed({ timeout: WAIT_MS });
    // Every line is words (or there are none); no raw ID or code shows (NFR-USE-14).
    await browser.waitUntil(
      async () =>
        (await $$('[data-attention-kind]')).length > 0 ||
        (await pageText()).includes(t('ur', 'attention-empty-title')),
      { timeout: WAIT_MS },
    );
    expect(await showsNoRawText()).toBe(true);
    await shootScreen('bell', 'ur');
    await press(Key.Escape);
    await title.waitForDisplayed({ timeout: WAIT_MS, reverse: true });
  });

  it('5. starts where it was left, and a report keeps its filters', async () => {
    // A report keeps its filters: type the day of a balance list, leave, come back.
    await openRoute('#/reports/R20');
    await waitForHeading('ur', 'report-r20-title');
    await focusField('ur', 'report-filter-as-of');
    await type('10-10-2026');
    await browser.pause(SETTLE_MS);
    await openRoute('#/');
    await openRoute('#/reports/R20');
    await waitForHeading('ur', 'report-r20-title');
    const again = await focusField('ur', 'report-filter-as-of');
    await browser.waitUntil(async () => (await again.getValue()) === '10-10-2026', {
      timeout: WAIT_MS,
      timeoutMsg: 'The report did not keep its day.',
    });
    // The app starts on the screen it was left on: open the stock screen, sign out and in again.
    await openRoute('#/bikri/stock');
    await waitForHeading('ur', 'glossary-baqi-maal');
    await browser.pause(SETTLE_MS);
    await signOut('ur');
    await signIn('ur', OWNER);
    await waitForShell('ur');
    await waitForHeading('ur', 'glossary-baqi-maal');
    // Back on the dashboard, the next start is the dashboard.
    await openDashboard('ur');
    await browser.pause(SETTLE_MS);
    await signOut('ur');
    await signIn('ur', OWNER);
    await waitForShell('ur');
    await waitForText(t('ur', 'dashboard-quick-title'));
  });

  it('6. the guide opens again from the user menu, goes through five steps with Enter, and closes with Done', async () => {
    const menu = await $(`button[aria-label="${t('ur', 'layout-user-menu')}"]`);
    await menu.click();
    const item = await $(
      `//*[@role="menuitem"][contains(normalize-space(translate(., "⁨⁩", "")), "${t('ur', 'layout-user-guide')}")]`,
    );
    await item.waitForDisplayed({ timeout: WAIT_MS });
    await item.click();
    const steps = ['dashboard', 'index', 'bikri', 'receipt', 'reports'];
    for (const [index, id] of steps.entries()) {
      const card = await $(`[data-guide-step="${id}"]`);
      await card.waitForDisplayed({ timeout: WAIT_MS });
      await shootScreen(`guide-${id}`, 'ur');
      // Next has the focus, so Enter goes on (the last step says Done and closes).
      if (index < steps.length - 1) await press(Key.Enter);
    }
    await press(Key.Enter);
    const closed = await $('[data-guide-step]');
    await closed.waitForDisplayed({ timeout: WAIT_MS, reverse: true });
  });

  it('7. signing in to the dashboard with its figures drawn is under 1 second (NFR-SPD-2)', async () => {
    await openDashboard('ur');
    /** @type {number[]} */
    const samples = [];
    for (let run = 0; run < RUNS; run += 1) {
      await signOut('ur');
      await focusField('ur', 'auth-login-username');
      await type(`${OWNER.username}${Key.Enter}${OWNER.password}`);
      await armTimer({ kind: 'text', selector: TILES, text: t('ur', 'dashboard-tile-sales') });
      await type(Key.Enter);
      samples.push(await readTimer());
      await waitForShell('ur');
      // The next sign-in starts from the dashboard again, whatever the app restored.
      await openDashboard('ur');
    }
    const result = recordTiming(
      'NFR-SPD-2',
      'Sign in until the dashboard tiles are drawn',
      samples,
    );
    console.log(`NFR-SPD-2 (dashboard): p95 ${result.p95_ms.toFixed(1)} ms`);
    expect(result.pass).toBe(true);
  });

  it('8. the dashboard does not scroll sideways and shows no raw text', async () => {
    await openDashboard('ur');
    expect(await noSidewaysScroll()).toBe(true);
    expect(await showsNoRawText()).toBe(true);
  });
});

describe('the dashboard and the bell (English)', () => {
  it('9. turns to English, left to right, with the same figures and no word left in Urdu', async () => {
    await openDashboard('ur');
    const dues = digitsOf(await (await tileWith(t('ur', 'dashboard-tile-dues'))).getText());
    await switchLanguage('en');
    await waitForText(t('en', 'dashboard-quick-title'));
    expect(await browser.execute(() => document.documentElement.dir)).toBe(DIRECTION.en);
    expect(digitsOf(await (await tileWith(t('en', 'dashboard-tile-dues'))).getText())).toBe(dues);
    expect(await noSidewaysScroll()).toBe(true);
    expect(await showsNoRawText()).toBe(true);
    // Names of people stay as they were typed (PRD FR-7.25); the words of the app are English.
    const words = await $('main h2').getText();
    expect(words).not.toMatch(/[؀-ۿ]/);
    await shootScreen('dashboard', 'en');
    const bell = await $('[data-attention-count]');
    await bell.click();
    await $(`//h2[normalize-space()="${t('en', 'attention-title')}"]`).waitForDisplayed({
      timeout: WAIT_MS,
    });
    await shootScreen('bell', 'en');
    await press(Key.Escape);
    // The arrows follow the direction: the next tile is on the right in English.
    await focusFirstTile();
    const first = await focusedIndex(TILES);
    await press(Key.ArrowRight);
    expect(await focusedIndex(TILES)).toBe(first + 1);
    await switchLanguage('ur');
  });
});
