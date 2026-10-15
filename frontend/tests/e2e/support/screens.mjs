// Window sizes and screenshots (NFR-USE-15, NFR-USE-17, NFR-USE-28). The content area is set to
// the size by the measured difference between the window and the page, then a picture is saved
// as <screen>__<language>__<width>x<height>.png in output/screenshots/, with an index file.

import { browser } from '@wdio/globals';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { SCREENSHOT_DIR } from '../paths.mjs';
import { readJson } from './json.mjs';

/** Content-area sizes in CSS pixels (Windows scaling 100% on the runner). */
export const SIZES = [
  { width: 1024, height: 600 },
  { width: 1366, height: 768 },
  { width: 1920, height: 1080 },
];
const MAX_RESIZE_TRIES = 3;
const INDEX_FILE = path.join(SCREENSHOT_DIR, 'index.json');

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
export async function setContentSize(width, height) {
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

/**
 * Saves a picture of the screen as it is now at each of the three sizes, and adds them to the
 * index. The page must be in the language already.
 * @param {string} screen a short name, for example "dashboard"
 * @param {string} language "ur" or "en"
 */
export async function shootScreen(screen, language) {
  mkdirSync(SCREENSHOT_DIR, { recursive: true });
  const index = readJson(
    INDEX_FILE,
    /** @type {{ file: string, screen: string, language: string, requested: string, content: string }[]} */ ([]),
  );
  for (const { width, height } of SIZES) {
    const actual = await setContentSize(width, height);
    const file = `${screen}__${language}__${String(width)}x${String(height)}.png`;
    await browser.saveScreenshot(path.join(SCREENSHOT_DIR, file));
    index.push({
      file,
      screen,
      language,
      requested: `${String(width)}x${String(height)}`,
      content: `${String(actual.width)}x${String(actual.height)}`,
    });
  }
  writeFileSync(INDEX_FILE, JSON.stringify(index, null, 2));
}
