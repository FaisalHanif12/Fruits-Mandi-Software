// D5 late track, round 11, garden slice (PRD 9.1, FR-12.37, FR-7.46, NFR-USE-7, NFR-USE-15,
// NFR-USE-28): the gardens tile of the dashboard, by keyboard, in Urdu and then in English, on the
// real app (a garden may or may not have been made by the earlier specs: both are right).
//   1. The dashboard shows the gardens tile with its words, never a raw ID, and does not scroll
//      sideways. With an open garden the tile says how many are open; with none it says so.
//   2. Enter on the focused tile opens the list of gardens (where a new garden is made).
//   3. Alt+L turns the tile to English with the same figure, left to right.
// The figure is the backend's running balance of the open gardens (E2E-40) and is proved by the Rust
// tests and the fixtures; this spec proves that the real app shows it and opens the screen behind it.
// It runs on Windows only (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  openEndOfMenu,
  pageText,
  press,
  startSignedIn,
  switchLanguage,
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

describe('the gardens tile of the dashboard (English)', () => {
  it('3. Alt+L turns the tile to English, left to right, with its figure', async () => {
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
});
