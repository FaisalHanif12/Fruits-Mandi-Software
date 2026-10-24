// D1 story, step 3: the shell. The main menu on the start side with eleven items, the dashboard, the
// menu by keyboard (arrows, Home, End, Enter), Alt+L in both directions, the text size keys, and
// Settings by keyboard. Timed: opening a screen from the menu (NFR-SPD-3) and the
// language switch (NFR-SPD-19), 20 times each, p95. Screenshots of the shell in both languages.

import { $, $$, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { DIRECTION, LANGUAGES, t } from '../support/messages.mjs';
import {
  Key,
  focusField,
  focusMenu,
  focusedIndex,
  openEndOfMenu,
  pageText,
  startSignedIn,
  tabTo,
  type,
  switchLanguage,
  waitForHeading,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import { RUNS, armTimer, readTimer, recordTiming } from '../support/timing.mjs';

const MENU_ITEMS = 'nav [data-nav-item]';
// The items of PRD FR-5.2 (باغات and مارکیٹ سفر included).
const MENU_COUNT = 11;

/** @param {string} name @param {string} action @param {number[]} samples */
function check(name, action, samples) {
  const result = recordTiming(name, action, samples);
  console.log(
    `${name}: p95 ${result.p95_ms.toFixed(1)} ms, threshold ${result.threshold_ms.toFixed(0)} ms`,
  );
  expect(result.pass).toBe(true);
}

describe('the shell (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  it('shows the main menu with eleven items, and the dashboard with its quick actions', async () => {
    expect(await $$(MENU_ITEMS)).toHaveLength(MENU_COUNT);
    const text = await pageText();
    expect(text).toContain(t('ur', 'dashboard-greeting', { name: OWNER.displayName }));
    // The dashboard has its quick actions at once, before its figures come (PRD Section 9, NFR-USE-9).
    expect(text).toContain(t('ur', 'dashboard-quick-title'));
    // The menu is on the start side: on the right in Urdu (PRD FR-5.2).
    const nav = await $('nav');
    const box = await browser.execute((el) => el.getBoundingClientRect().left, nav);
    const width = await browser.execute(() => window.innerWidth);
    expect(box).toBeGreaterThan(width / 2);
  });

  it('the status bar names the user, today and the state of the day', async () => {
    const bar = await $('footer');
    const text = (await bar.getText()).replace(/[⁨⁩]/g, '');
    expect(text).toContain(t('ur', 'layout-status-user', { name: OWNER.displayName }));
    expect(text).toContain(t('ur', 'layout-status-day-open'));
  });

  it('the menu works by keyboard: Tab reaches it, arrows move, Enter opens', async () => {
    await tabTo('nav');
    const before = await focusedIndex(MENU_ITEMS);
    expect(before).toBeGreaterThanOrEqual(0);
    await type(Key.ArrowDown);
    expect(await focusedIndex(MENU_ITEMS)).toBe(before + 1);
    await type(Key.ArrowUp);
    expect(await focusedIndex(MENU_ITEMS)).toBe(before);
    await openEndOfMenu('last');
    await waitForHeading('ur', 'settings-title');
    await shootScreen('settings', 'ur');
  });

  it('opening a screen from the menu is under 300 ms (NFR-SPD-3)', async () => {
    /** @type {number[]} */
    const samples = [];
    for (let run = 0; run < RUNS; run += 1) {
      const toSettings = run % 2 === 1;
      await focusMenu();
      await type(toSettings ? Key.End : Key.Home);
      await armTimer({
        kind: 'text',
        selector: 'h1',
        text: toSettings
          ? t('ur', 'settings-title')
          : t('ur', 'dashboard-greeting', { name: OWNER.displayName }),
      });
      await type(Key.Enter);
      samples.push(await readTimer());
    }
    check('NFR-SPD-3', 'Open a screen from the main menu', samples);
  });

  it('Settings by keyboard: the address is changed and saved with F10', async () => {
    await openEndOfMenu('last');
    await waitForHeading('ur', 'settings-title');
    const address = await focusField('ur', 'settings-firm-address');
    await address.setValue('Shop 2, Kot Radha Kishan');
    await type(Key.F10);
    await browser.waitUntil(async () => (await pageText()).includes(t('ur', 'settings-saved')), {
      timeoutMsg: 'The page never said the settings were saved.',
    });
  });

  it('Ctrl + makes the text bigger and Ctrl 0 brings it back', async () => {
    const zoomOf = () =>
      browser.execute(() => Number.parseFloat(document.documentElement.style.zoom || '1'));
    const normal = await zoomOf();
    await browser.keys([Key.Ctrl, '=']);
    expect(await zoomOf()).toBeGreaterThan(normal);
    await browser.keys([Key.Ctrl, '0']);
    expect(await zoomOf()).toBe(normal);
  });
});

describe('Alt+L, in both directions', () => {
  it('turns the whole page to English and back in under 500 ms (NFR-SPD-19)', async () => {
    /** @type {number[]} */
    const samples = [];
    for (let run = 0; run < RUNS; run += 1) {
      const toEnglish = run % 2 === 0;
      const target = toEnglish ? 'en' : 'ur';
      await armTimer({ kind: 'attribute', name: 'dir', value: DIRECTION[target] });
      await browser.keys([Key.Alt, 'l']);
      samples.push(await readTimer());
    }
    check('NFR-SPD-19', 'Alt+L until the page is drawn in the other language', samples);
    await switchLanguage('ur');
  });

  it('English: the menu is on the left, every word is English and nothing is left in Urdu', async () => {
    await switchLanguage('en');
    await waitForHeading('en', 'settings-title');
    const nav = await $('nav');
    const left = await browser.execute((el) => el.getBoundingClientRect().left, nav);
    expect(left).toBeLessThan(10);
    await expect($('html')).toHaveAttribute('lang', 'en');
    const words = await $('main').getText();
    expect(words).not.toMatch(/[؀-ۿ]/);
    await switchLanguage('ur');
  });
});

describe('screenshots of the shell', () => {
  for (const language of LANGUAGES) {
    it(`saves the dashboard and Settings in ${language} at three sizes`, async () => {
      await switchLanguage(language);
      await openEndOfMenu('first');
      await browser.waitUntil(async () =>
        (await pageText()).includes(t(language, 'dashboard-greeting', { name: OWNER.displayName })),
      );
      await shootScreen('dashboard', language);
      await openEndOfMenu('last');
      await waitForHeading(language, 'settings-title');
      await shootScreen('settings', language);
    });
  }
});
