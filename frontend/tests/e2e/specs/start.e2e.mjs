// First end-to-end test (plan 01 T3.6): the window opens in Urdu, right to left; Alt+L switches
// to English, left to right. Screenshots of both languages at 1024 x 600, 1366 x 768 and
// 1920 x 1080 content size (NFR-USE-15, NFR-USE-17, NFR-USE-28). No visual comparison yet
// (D1-Q1 is open): the files are saved for people to look at.

import { $, browser, expect } from '@wdio/globals';
import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { Key } from 'webdriverio';
import { SCREENSHOT_DIR } from '../paths.mjs';

const TEST_NAME = 'start';
/** Content-area sizes in CSS pixels (Windows scaling 100% on the runner). */
const SIZES = [
  { width: 1024, height: 600 },
  { width: 1366, height: 768 },
  { width: 1920, height: 1080 },
];
/** Each language, its direction and the label of its switch button. */
const LANGUAGES = [
  { language: 'ur', direction: 'rtl', button: 'اردو' },
  { language: 'en', direction: 'ltr', button: 'English' },
];
const MAX_RESIZE_TRIES = 3;

/** @returns {Promise<{ width: number, height: number }>} */
async function contentSize() {
  return browser.execute(() => ({ width: window.innerWidth, height: window.innerHeight }));
}

/**
 * Resizes the window so the page content area is exactly width x height. The window frame
 * and title bar are outside the content, so the window is grown by the measured difference.
 * @param {number} width
 * @param {number} height
 */
async function setContentSize(width, height) {
  let request = { width, height };
  let actual = await contentSize();
  for (let attempt = 0; attempt < MAX_RESIZE_TRIES; attempt += 1) {
    await browser.setWindowSize(request.width, request.height);
    actual = await contentSize();
    if (actual.width === width && actual.height === height) return actual;
    request = {
      width: request.width + width - actual.width,
      height: request.height + height - actual.height,
    };
  }
  throw new Error(
    `The content area is ${String(actual.width)}x${String(actual.height)}, not ${String(width)}x${String(height)}. ` +
      'On the CI runner the screen may be smaller than the window; the screen resolution must be at least 1920x1080 plus the window frame.',
  );
}

describe('start screen', () => {
  it('opens in Urdu, right to left, with the app name and version', async () => {
    const html = await $('html');
    await expect(html).toHaveAttribute('dir', 'rtl');
    await expect(html).toHaveAttribute('lang', 'ur');
    await expect($('h1')).toHaveText('منڈی سافٹ ویئر');
    await expect($('[data-testid="app-version"]')).toHaveText(expect.stringContaining('ورژن'));
  });

  it('switches to English with Alt+L, and back to Urdu', async () => {
    await browser.keys([Key.Alt, 'l']);
    await expect($('html')).toHaveAttribute('dir', 'ltr');
    await expect($('html')).toHaveAttribute('lang', 'en');
    await expect($('h1')).toHaveText('Mandi Software');
    await expect($('[data-testid="app-version"]')).toHaveText(expect.stringContaining('Version'));

    await browser.keys([Key.Alt, 'l']);
    await expect($('html')).toHaveAttribute('dir', 'rtl');
    await expect($('h1')).toHaveText('منڈی سافٹ ویئر');
  });

  it('saves screenshots in Urdu and English at every window size', async () => {
    /** @type {{ file: string, language: string, requested: string, content: string }[]} */
    const saved = [];
    for (const { language, direction, button } of LANGUAGES) {
      await $(`button=${button}`).click();
      await expect($('html')).toHaveAttribute('dir', direction);
      for (const { width, height } of SIZES) {
        const actual = await setContentSize(width, height);
        const file = `${TEST_NAME}-${language}-${String(width)}x${String(height)}.png`;
        await browser.saveScreenshot(path.join(SCREENSHOT_DIR, file));
        saved.push({
          file,
          language,
          requested: `${String(width)}x${String(height)}`,
          content: `${String(actual.width)}x${String(actual.height)}`,
        });
      }
    }
    writeFileSync(
      path.join(SCREENSHOT_DIR, `${TEST_NAME}-screenshots.json`),
      JSON.stringify(saved, null, 2),
    );
    expect(saved).toHaveLength(SIZES.length * LANGUAGES.length);
  });
});
